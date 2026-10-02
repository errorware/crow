//! Password bootstrap (ERR-33): log in once with a password and install a
//! Crow key, so every later connection is key-only. The password is never
//! stored.
//!
//! OpenSSH reads passwords from a terminal or an `SSH_ASKPASS` program. Crow
//! is its own askpass program: ssh runs the Crow binary with the prompt as
//! its argument and `CROW_ASKPASS_SOCK` in its environment; that process
//! connects to a unix socket Crow is serving (in a 0700 directory), receives
//! the password once, and prints it for ssh. The password never appears in
//! argv, an environment variable or a file.

use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::time::Duration;

use zeroize::Zeroizing;

use super::local::run_command_env;
use super::ssh::{classify_ssh_failure, shell_quote};
use super::{ConnectionState, HostError};
use crate::vault::ServerRecord;

/// Set in the environment of the ssh child; tells the Crow binary it was
/// started as ssh's askpass helper.
pub const ASKPASS_SOCK_ENV: &str = "CROW_ASKPASS_SOCK";

/// Appends the key on stdin to `~/.ssh/authorized_keys` unless it's already
/// there, with the modes sshd insists on.
const INSTALL_KEY_SCRIPT: &str = r#"umask 077; mkdir -p "$HOME/.ssh" && chmod 700 "$HOME/.ssh" && touch "$HOME/.ssh/authorized_keys" && chmod 600 "$HOME/.ssh/authorized_keys" && IFS= read -r k && { grep -qxF "$k" "$HOME/.ssh/authorized_keys" || printf '%s\n' "$k" >> "$HOME/.ssh/authorized_keys"; } && echo crow-key-installed"#;

#[derive(Debug, PartialEq)]
pub enum BootstrapError {
    /// The server refused the password.
    PasswordRejected(String),
    /// The local ssh is older than 8.4 (no `SSH_ASKPASS_REQUIRE`).
    SshTooOld(String),
    /// Anything else: unreachable, host key, timeout, remote failure.
    Failed(String),
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapError::PasswordRejected(d) => write!(f, "password rejected — check it and try again ({d})"),
            BootstrapError::SshTooOld(v) => write!(f, "this machine's ssh ({v}) is older than OpenSSH 8.4 and can't take a password from Crow; install the key yourself (ssh-copy-id) and choose key login"),
            BootstrapError::Failed(d) => write!(f, "{d}"),
        }
    }
}

/// `(major, minor)` from `ssh -V` output such as `OpenSSH_9.6p1 Ubuntu-3, ...`.
pub fn parse_openssh_version(text: &str) -> Option<(u32, u32)> {
    let v = text.split("OpenSSH_").nth(1)?;
    let mut parts = v.split(|c: char| !c.is_ascii_digit());
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

/// Serves `password` to exactly one askpass connection, from a socket in a
/// fresh 0700 directory. Dropping it removes the directory.
struct AskpassServer {
    dir: PathBuf,
    sock: PathBuf,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AskpassServer {
    fn start(password: Zeroizing<String>) -> std::io::Result<Self> {
        use std::os::unix::fs::PermissionsExt;
        let base = dirs::runtime_dir().unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!("crow-askpass-{:016x}", rand::random::<u64>()));
        std::fs::create_dir(&dir)?;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        let sock = dir.join("s");
        let listener = UnixListener::bind(&sock)?;
        let thread = std::thread::spawn(move || {
            // One answer only: a second prompt (wrong password) gets nothing.
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.write_all(password.as_bytes());
            }
        });
        Ok(Self { dir, sock, thread: Some(thread) })
    }
}

impl Drop for AskpassServer {
    fn drop(&mut self) {
        // Unblock the accept if ssh never asked, then clean up.
        let _ = std::os::unix::net::UnixStream::connect(&self.sock);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The askpass side, run when ssh starts the Crow binary as `SSH_ASKPASS`.
/// Answers password prompts only; returns the process exit code.
pub fn askpass_main(sock: &str, prompt: &str) -> i32 {
    let p = prompt.to_lowercase();
    if !(p.contains("password") || p.contains("passphrase")) {
        return 1;
    }
    let Ok(mut stream) = std::os::unix::net::UnixStream::connect(sock) else { return 1 };
    let mut password = Zeroizing::new(String::new());
    if stream.read_to_string(&mut password).is_err() || password.is_empty() {
        return 1;
    }
    println!("{}", password.as_str());
    0
}

/// The ssh arguments for one password login to `server` running `remote`.
pub fn bootstrap_args(server: &ServerRecord, jump: Option<&str>, remote: &str) -> Result<Vec<String>, BootstrapError> {
    if server.host.is_empty() || server.host.starts_with('-') {
        return Err(BootstrapError::Failed(format!("{:?} is not a valid host", server.host)));
    }
    let mut args: Vec<String> = [
        "-o", "BatchMode=no",
        "-o", "StrictHostKeyChecking=yes",
        "-o", "PreferredAuthentications=password,keyboard-interactive",
        "-o", "PubkeyAuthentication=no",
        // Password login only: no agent, so none can prompt (ERR-87).
        "-o", "IdentityAgent=none",
        "-o", "NumberOfPasswordPrompts=1",
        "-o", "ControlMaster=no",
        "-o", "ControlPath=none",
        "-o", "ConnectTimeout=10",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(["-p".into(), server.port.to_string()]);
    if !server.login_user.is_empty() {
        args.extend(["-l".into(), server.login_user.clone()]);
    }
    if let Some(j) = jump {
        args.extend(["-J".into(), j.to_string()]);
    }
    args.extend(["--".into(), server.host.clone(), remote.to_string()]);
    Ok(args)
}

/// Logs in to `server` with `password` once and appends `public_key` (an
/// OpenSSH public key line) to the login user's authorized_keys.
pub fn install_key_with_password(server: &ServerRecord, jump: Option<&str>, password: Zeroizing<String>, public_key: &str) -> Result<(), BootstrapError> {
    let exe = std::env::current_exe().map_err(|e| BootstrapError::Failed(format!("can't locate the Crow binary for askpass: {e}")))?;
    install_key_via("ssh", &exe.to_string_lossy(), server, jump, password, public_key)
}

/// `install_key_with_password` with the ssh client and askpass program given
/// (tests point them at a known_hosts wrapper and the built crow binary).
fn install_key_via(ssh: &str, askpass_program: &str, server: &ServerRecord, jump: Option<&str>, password: Zeroizing<String>, public_key: &str) -> Result<(), BootstrapError> {
    let version = run_command_env(&[ssh, "-V"], &[], Duration::from_secs(5), &[])
        .map(|o| format!("{}{}", o.stdout, o.stderr))
        .unwrap_or_default();
    match parse_openssh_version(&version) {
        Some(v) if v < (8, 4) => return Err(BootstrapError::SshTooOld(version.trim().to_string())),
        None => return Err(BootstrapError::Failed(format!("couldn't run ssh: {}", version.trim()))),
        _ => {}
    }

    let remote = format!("sh -c {}", shell_quote(INSTALL_KEY_SCRIPT));
    let args = bootstrap_args(server, jump, &remote)?;
    let askpass = AskpassServer::start(password).map_err(|e| BootstrapError::Failed(format!("askpass socket: {e}")))?;
    let sock = askpass.sock.to_string_lossy().into_owned();

    let mut argv: Vec<&str> = vec![ssh];
    argv.extend(args.iter().map(String::as_str));
    let stdin = format!("{}\n", public_key.trim());
    let env = [("SSH_ASKPASS", askpass_program), ("SSH_ASKPASS_REQUIRE", "force"), (ASKPASS_SOCK_ENV, sock.as_str())];
    let result = run_command_env(&argv, stdin.as_bytes(), Duration::from_secs(45), &env);
    drop(askpass);

    match result {
        Ok(out) if out.stdout.contains("crow-key-installed") => Ok(()),
        Ok(out) => Err(BootstrapError::Failed(format!("the key install script didn't finish: {}", out.stderr.trim()))),
        Err(HostError::Failed { status: 255, stderr }) => match classify_ssh_failure(&stderr) {
            ConnectionState::AuthFailed(d) => Err(BootstrapError::PasswordRejected(d)),
            other => Err(BootstrapError::Failed(format!("{}: {}", other.label().to_lowercase(), other.detail().unwrap_or_default()))),
        },
        Err(HostError::Failed { stderr, .. }) => Err(BootstrapError::Failed(format!("installing the key failed: {}", stderr.trim()))),
        Err(HostError::Timeout) => Err(BootstrapError::Failed("timed out logging in with the password".into())),
        Err(e) => Err(BootstrapError::Failed(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openssh_versions() {
        assert_eq!(parse_openssh_version("OpenSSH_9.6p1 Ubuntu-3ubuntu13, OpenSSL 3.0.13"), Some((9, 6)));
        assert_eq!(parse_openssh_version("OpenSSH_8.4p1 Debian-5"), Some((8, 4)));
        assert_eq!(parse_openssh_version("OpenSSH_7.9p1"), Some((7, 9)));
        assert_eq!(parse_openssh_version("Dropbear v2022.83"), None);
    }

    #[test]
    fn password_login_args_are_password_only_and_end_options_before_the_host() {
        let srv = ServerRecord { host: "203.0.113.7".into(), port: 22, login_user: "root".into(), ..Default::default() };
        let args = bootstrap_args(&srv, None, "sh -c 'x'").unwrap();
        let joined = args.join(" ");
        assert!(joined.contains("PreferredAuthentications=password,keyboard-interactive"));
        assert!(joined.contains("PubkeyAuthentication=no") && joined.contains("StrictHostKeyChecking=yes"));
        assert_eq!(&args[args.len() - 3..], ["--", "203.0.113.7", "sh -c 'x'"]);
        let bad = ServerRecord { host: "-oProxyCommand=x".into(), ..Default::default() };
        assert!(bootstrap_args(&bad, None, "true").is_err());
    }

    /// The askpass pair hands the password over once, and only for
    /// password prompts.
    #[test]
    fn askpass_serves_the_password_once() {
        let server = AskpassServer::start(Zeroizing::new("s3cret pass".into())).unwrap();
        let sock = server.sock.to_string_lossy().into_owned();
        assert_eq!(askpass_main(&sock, "Are you sure you want to continue connecting?"), 1);
        // The real helper prints to stdout; read the socket the same way here.
        let mut got = String::new();
        std::os::unix::net::UnixStream::connect(&server.sock).unwrap().read_to_string(&mut got).unwrap();
        assert_eq!(got, "s3cret pass");
        let dir = server.dir.clone();
        drop(server);
        assert!(!dir.exists(), "socket directory removed");
    }

    /// End to end against a real password-only sshd:
    /// `CROW_LIVE_PW=host:port:user:password CROW_LIVE_KNOWN_HOSTS=/path cargo test live_password_bootstrap -- --ignored`
    /// (build the app first: the askpass helper is target/debug/crow).
    #[test]
    #[ignore]
    fn live_password_bootstrap() {
        let spec = std::env::var("CROW_LIVE_PW").expect("CROW_LIVE_PW=host:port:user:password");
        let kh = std::env::var("CROW_LIVE_KNOWN_HOSTS").expect("CROW_LIVE_KNOWN_HOSTS");
        let p: Vec<&str> = spec.splitn(4, ':').collect();
        let srv = ServerRecord { host: p[0].into(), port: p[1].parse().unwrap(), login_user: p[2].into(), ..Default::default() };
        let dir = std::env::temp_dir().join(format!("crow-live-pw-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wrapper = dir.join("ssh");
        std::fs::write(&wrapper, format!("#!/bin/sh\nexec ssh -o UserKnownHostsFile={kh} \"$@\"\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let crow = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/crow");
        let key = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHm0uQ3lfNqoGkhrA7Y2Hm8m1a9pR1S1a3o3m8b7n0tX crow-live-test";

        let wrong = install_key_via(wrapper.to_str().unwrap(), crow, &srv, None, Zeroizing::new("definitely-wrong".into()), key);
        println!("wrong password -> {wrong:?}");
        assert!(matches!(wrong, Err(BootstrapError::PasswordRejected(_))));

        let ok = install_key_via(wrapper.to_str().unwrap(), crow, &srv, None, Zeroizing::new(p[3].into()), key);
        println!("right password -> {ok:?}");
        assert_eq!(ok, Ok(()));
        // Idempotent: a second install doesn't duplicate the line.
        assert_eq!(install_key_via(wrapper.to_str().unwrap(), crow, &srv, None, Zeroizing::new(p[3].into()), key), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
