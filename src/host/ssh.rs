//! Remote servers over the system OpenSSH client.
//!
//! Crow drives `ssh` rather than embedding an SSH library, so it uses exactly
//! what a terminal would: the user's agent, `~/.ssh/config`, jump hosts and
//! `known_hosts` (where onboarding records accepted host keys). A ControlMaster
//! socket per server keeps one connection open, so the 2-second metrics poll
//! reuses it instead of reconnecting.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::Duration;

use super::local::run_command;
use super::{ExecOutput, Host, HostError};
use crate::vault::{ServerRecord, SshKeyRecord};

/// How a server can currently be reached, as seen by its last SSH command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    /// The server rejected our credentials (key, agent) or needs a password.
    AuthFailed(String),
    /// The host key is not in known_hosts, or changed since it was accepted.
    HostKeyRejected(String),
    /// Refused, timed out, DNS failure, no route.
    Unreachable(String),
}

impl ConnectionState {
    pub fn label(&self) -> &'static str {
        match self {
            ConnectionState::Connected => "CONNECTED",
            ConnectionState::AuthFailed(_) => "AUTH FAILED",
            ConnectionState::HostKeyRejected(_) => "HOST KEY",
            ConnectionState::Unreachable(_) => "UNREACHABLE",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            ConnectionState::Connected => None,
            ConnectionState::AuthFailed(d) | ConnectionState::HostKeyRejected(d) | ConnectionState::Unreachable(d) => Some(d),
        }
    }
}

/// Settings → Connection & SSH, as applied to every connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SshSettings {
    /// Seconds to wait for the TCP connection.
    pub connect_timeout: u32,
    /// Seconds between keepalives (two missed ones drop the connection).
    pub keepalive_interval: u32,
    /// Reuse one connection per server (ControlMaster).
    pub control_master: bool,
}

impl Default for SshSettings {
    fn default() -> Self {
        Self { connect_timeout: 8, keepalive_interval: 15, control_master: true }
    }
}

fn ssh_settings() -> &'static RwLock<SshSettings> {
    static SETTINGS: OnceLock<RwLock<SshSettings>> = OnceLock::new();
    SETTINGS.get_or_init(Default::default)
}

/// Applies saved settings to connections opened from now on.
pub fn set_ssh_settings(s: SshSettings) {
    if let Ok(mut current) = ssh_settings().write() {
        *current = s;
    }
}

/// The `-o` options every connection gets. Security ones are fixed;
/// the rest come from Settings → Connection & SSH.
fn base_options(settings: &SshSettings) -> Vec<String> {
    let fixed = [
        "BatchMode=yes".to_string(),
        "StrictHostKeyChecking=yes".to_string(),
        // Crow never needs them; override whatever ~/.ssh/config says.
        "ForwardAgent=no".to_string(),
        "ForwardX11=no".to_string(),
        "ServerAliveCountMax=2".to_string(),
    ];
    let configured = [
        format!("ConnectTimeout={}", settings.connect_timeout.max(1)),
        format!("ServerAliveInterval={}", settings.keepalive_interval.max(1)),
        format!("ControlMaster={}", if settings.control_master { "auto" } else { "no" }),
        format!("ControlPersist={}", if settings.control_master { "600" } else { "no" }),
    ];
    fixed.into_iter().chain(configured).flat_map(|o| ["-o".to_string(), o]).collect()
}

/// Whether the private key at `path` works without an agent: an OpenSSH key
/// with no passphrase. Encrypted, unreadable or unrecognised keys keep the
/// agent (it may hold the unlocked key). Cached by path and mtime, since
/// connections are set up every few seconds.
fn key_usable_without_agent(path: &str) -> bool {
    static CACHE: OnceLock<Mutex<HashMap<String, (std::time::SystemTime, bool)>>> = OnceLock::new();
    let Ok(mtime) = std::fs::metadata(path).and_then(|m| m.modified()) else { return false };
    let cache = CACHE.get_or_init(Default::default);
    if let Some((t, ok)) = cache.lock().ok().and_then(|c| c.get(path).copied()) {
        if t == mtime {
            return ok;
        }
    }
    let ok = std::fs::read_to_string(path).ok().is_some_and(|text| key_text_usable_without_agent(&text));
    if let Ok(mut c) = cache.lock() {
        c.insert(path.to_string(), (mtime, ok));
    }
    ok
}

fn key_text_usable_without_agent(text: &str) -> bool {
    ssh_key::PrivateKey::from_openssh(text).is_ok_and(|k| !k.is_encrypted())
}

/// The agent behind `ssh -G` output (or `SSH_AUTH_SOCK` when the config sets
/// none) if it's one that asks you to approve each use of a key.
pub fn approval_agent_name(ssh_g: &str, auth_sock: Option<&str>) -> Option<&'static str> {
    let configured = ssh_g.lines().find_map(|l| l.strip_prefix("identityagent ")).map(str::trim);
    let socket = match configured {
        Some("none") => return None,
        Some("SSH_AUTH_SOCK") | None => auth_sock?,
        Some(path) => path,
    };
    let s = socket.to_lowercase();
    if s.contains("1password") || s.contains("2bua8c4s2c") {
        Some("1Password")
    } else if s.contains("secretive") {
        Some("Secretive")
    } else {
        None
    }
}

/// The approving agent ssh would use for `server` (`ssh -G` with Crow's own
/// options), if any.
pub fn approval_agent_for(server: &ServerRecord) -> Option<&'static str> {
    let host = SshHost::for_server(server);
    let mut argv: Vec<&str> = vec![host.program.as_str(), "-G"];
    argv.extend(host.args.iter().map(String::as_str));
    let out = run_command(&argv, &[], Duration::from_secs(5)).ok()?;
    approval_agent_name(&out.stdout, std::env::var("SSH_AUTH_SOCK").ok().as_deref())
}

/// Turns connection args into ones for a brand-new login: no reused
/// (ControlMaster) connection, keys only. ssh keeps the first value of each
/// option, so the existing ones are replaced, not appended to.
fn fresh_login_args(mut args: Vec<String>) -> Vec<String> {
    for a in args.iter_mut() {
        if a.starts_with("ControlMaster=") {
            *a = "ControlMaster=no".into();
        } else if a.starts_with("ControlPersist=") {
            *a = "ControlPersist=no".into();
        } else if a.starts_with("ControlPath=") {
            *a = "ControlPath=none".into();
        }
    }
    let at = args.iter().position(|a| a == "--").unwrap_or(args.len());
    args.splice(at..at, ["-o".to_string(), "PreferredAuthentications=publickey".to_string()]);
    args
}

/// Logs in to `server` anew, with its key only, and runs `true`: proof that
/// key login works on its own, not just through a connection Crow holds.
pub fn fresh_key_login(server: &ServerRecord) -> Result<(), String> {
    let host = SshHost::for_server(server);
    if let Some(why) = &host.unsupported {
        return Err(why.clone());
    }
    let mut argv: Vec<String> = vec![host.program.clone()];
    argv.extend(fresh_login_args(host.args.clone()));
    argv.push("true".into());
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    run_command(&argv, &[], Duration::from_secs(20)).map(|_| ()).map_err(|e| match e {
        HostError::Failed { stderr, .. } => stderr.trim().lines().last().unwrap_or("login failed").to_string(),
        other => other.to_string(),
    })
}

fn states() -> &'static Mutex<HashMap<String, ConnectionState>> {
    static STATES: OnceLock<Mutex<HashMap<String, ConnectionState>>> = OnceLock::new();
    STATES.get_or_init(Default::default)
}

/// Last known connection state of a server, if Crow has talked to it over SSH.
pub fn connection_state(server_id: &str) -> Option<ConnectionState> {
    states().lock().ok()?.get(server_id).cloned()
}

fn record_state(server_id: &str, state: ConnectionState) {
    if let Ok(mut map) = states().lock() {
        map.insert(server_id.to_string(), state);
    }
}

/// What SSH needs to know beyond a server's own record: key file paths and
/// the other servers (for jump hosts). The app refreshes it whenever servers
/// or keys change; collectors run on worker threads and read it from here.
#[derive(Default)]
struct Directory {
    key_paths: HashMap<String, String>,
    servers: HashMap<String, ServerRecord>,
}

fn directory() -> &'static RwLock<Directory> {
    static DIRECTORY: OnceLock<RwLock<Directory>> = OnceLock::new();
    DIRECTORY.get_or_init(Default::default)
}

pub fn update_directory(servers: &[ServerRecord], keys: &[SshKeyRecord]) {
    if let Ok(mut dir) = directory().write() {
        dir.servers = servers.iter().map(|s| (s.id.clone(), s.clone())).collect();
        dir.key_paths = keys.iter().filter_map(|k| Some((k.id.clone(), k.private_key_path.clone()?))).collect();
    }
}

/// Where ssh keeps its multiplexing sockets. Unix socket paths are short
/// (~104 bytes), so this stays near the root and ssh hashes the rest (%C).
fn control_dir() -> PathBuf {
    let base = dirs::runtime_dir().unwrap_or_else(std::env::temp_dir);
    let dir = base.join("crow-ssh");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
    }
    dir
}

/// Quotes one argument for the remote POSIX shell ssh hands the command to.
pub fn shell_quote(arg: &str) -> String {
    if !arg.is_empty() && arg.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_./=:@%+,".contains(&b)) {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}

pub struct SshHost {
    server_id: String,
    label: String,
    /// ssh arguments up to and including the destination.
    args: Vec<String>,
    /// Why this server can't be used at all (e.g. password auth), if so.
    unsupported: Option<String>,
    program: String,
}

impl SshHost {
    pub fn for_server(server: &ServerRecord) -> Self {
        let dir = directory().read().ok();
        let key_path = server.key_id.as_ref().and_then(|id| dir.as_ref()?.key_paths.get(id).cloned());
        let jump = server
            .jump_host_id
            .as_ref()
            .and_then(|id| dir.as_ref()?.servers.get(id).cloned())
            .map(|j| format!("{}@{}:{}", j.login_user, j.host, if j.port == 0 { 22 } else { j.port }));
        Self::new(server, key_path, jump, control_dir())
    }

    pub(crate) fn new(server: &ServerRecord, key_path: Option<String>, jump: Option<String>, control_dir: PathBuf) -> Self {
        let port = if server.port == 0 { 22 } else { server.port };
        let settings = ssh_settings().read().map(|s| *s).unwrap_or_default();
        let mut args = base_options(&settings);
        args.push("-o".into());
        args.push(format!("ControlPath={}/%C", control_dir.display()));
        args.extend(["-p".into(), port.to_string()]);
        if !server.login_user.is_empty() {
            args.extend(["-l".into(), server.login_user.clone()]);
        }
        let mut unsupported = None;
        match server.auth_method.as_str() {
            "password" => {
                unsupported = Some("This server is set to password login; Crow connects with keys, installed once with the password.".to_string());
            }
            "publickey" => match &key_path {
                Some(path) => {
                    args.extend(["-i".into(), expand_home(path), "-o".into(), "IdentitiesOnly=yes".into()]);
                    // A passphrase-free key file needs no agent. Keeping ssh
                    // away from it stops agents that ask you to approve every
                    // use (1Password, Secretive) from stalling Crow (ERR-87).
                    if key_usable_without_agent(&expand_home(path)) {
                        args.extend(["-o".into(), "IdentityAgent=none".into()]);
                    }
                }
                None if server.key_id.is_some() => {
                    unsupported = Some("the enrolled key for this server has no private key file".to_string());
                }
                None => {}
            },
            _ => {} // "agent": ssh uses the agent and ~/.ssh/config by default
        }
        if let Some(jump) = jump {
            args.extend(["-J".into(), jump]);
        }
        // The destination comes from the vault; a value starting with '-' would
        // be read by ssh as an option (e.g. -oProxyCommand=...). Refuse it, and
        // end option parsing before the destination regardless.
        if server.host.starts_with('-') || server.host.trim().is_empty() {
            unsupported = Some(format!("invalid host address {:?}", server.host));
        }
        args.push("--".into());
        args.push(server.host.clone());
        Self {
            server_id: server.id.clone(),
            label: format!("ssh:{}@{}", server.login_user, server.host),
            args,
            unsupported,
            program: "ssh".into(),
        }
    }

    /// ssh with a terminal for an interactive shell (ERR-93): the same
    /// options every Crow connection uses (shared connection, strict host
    /// keys, the enrolled key, the jump host), plus -tt.
    pub fn interactive_command(&self) -> Result<(String, Vec<String>), String> {
        if let Some(why) = &self.unsupported {
            return Err(why.clone());
        }
        let mut args = vec!["-tt".to_string()];
        args.extend(self.args.iter().cloned());
        Ok((self.program.clone(), args))
    }

    #[cfg(test)]
    fn with_program(mut self, program: &str) -> Self {
        self.program = program.to_string();
        self
    }
}

fn expand_home(path: &str) -> String {
    match (path.strip_prefix("~/"), dirs::home_dir()) {
        (Some(rest), Some(home)) => home.join(rest).display().to_string(),
        _ => path.to_string(),
    }
}

/// Maps an ssh client failure (exit 255) to a connection state.
pub fn classify_ssh_failure(stderr: &str) -> ConnectionState {
    let s = stderr.to_lowercase();
    let first_line = stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("ssh failed").trim().to_string();
    if s.contains("host key verification failed") || s.contains("remote host identification has changed") || s.contains("no matching host key") {
        ConnectionState::HostKeyRejected(first_line)
    } else if s.contains("permission denied") || s.contains("too many authentication failures") || s.contains("no more authentication methods") {
        ConnectionState::AuthFailed(first_line)
    } else {
        ConnectionState::Unreachable(first_line)
    }
}

impl Host for SshHost {
    fn label(&self) -> String {
        self.label.clone()
    }

    /// Closes the multiplexed connection this server's ControlMaster holds
    /// (ERR-32): `ssh -O exit` with the same options resolves to the same `%C`
    /// control socket that `exec_stdin` opened.
    fn close_connection(&self) -> bool {
        let mut argv: Vec<&str> = vec![self.program.as_str(), "-O", "exit"];
        argv.extend(self.args.iter().map(String::as_str));
        run_command(&argv, &[], Duration::from_secs(5)).is_ok()
    }

    fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
        if let Some(why) = &self.unsupported {
            record_state(&self.server_id, ConnectionState::AuthFailed(why.clone()));
            return Err(HostError::Unreachable(why.clone()));
        }
        let remote = argv.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" ");
        let mut full: Vec<&str> = vec![self.program.as_str()];
        // args end with `-- <destination>`; everything after is the command.
        full.extend(self.args.iter().map(String::as_str));
        full.push(&remote);
        match run_command(&full, stdin, timeout) {
            // ssh exits 255 for its own failures; anything else is the remote command's status.
            Err(HostError::Failed { status: 255, stderr }) => {
                let state = classify_ssh_failure(&stderr);
                let message = format!("{}: {}", state.label().to_lowercase(), state.detail().unwrap_or_default());
                record_state(&self.server_id, state);
                Err(HostError::Unreachable(message))
            }
            Err(HostError::Timeout) => {
                record_state(&self.server_id, ConnectionState::Unreachable("timed out".into()));
                Err(HostError::Timeout)
            }
            other => {
                record_state(&self.server_id, ConnectionState::Connected);
                other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::FileTypeExt;

    fn server(auth: &str, key_id: Option<&str>) -> ServerRecord {
        ServerRecord {
            id: "srv-1".into(),
            name: "db-01".into(),
            host: "10.0.4.12".into(),
            port: 2222,
            login_user: "ops".into(),
            auth_method: auth.into(),
            key_id: key_id.map(Into::into),
            ..ServerRecord::default()
        }
    }

    #[test]
    fn builds_batch_multiplexed_args_with_key_and_jump() {
        let h = SshHost::new(&server("publickey", Some("k1")), Some("/keys/id".into()), Some("root@bastion:22".into()), "/run/crow".into());
        let a = h.args.join(" ");
        assert!(a.contains("BatchMode=yes") && a.contains("StrictHostKeyChecking=yes"));
        assert!(a.contains("ControlMaster=auto") && a.contains("ControlPath=/run/crow/%C"));
        assert!(a.contains("-p 2222") && a.contains("-l ops"));
        assert!(a.contains("-i /keys/id -o IdentitiesOnly=yes"));
        assert!(a.contains("-J root@bastion:22"));
        assert_eq!(&h.args[h.args.len() - 2..], ["--", "10.0.4.12"]);
        assert!(h.unsupported.is_none());
    }

    #[test]
    fn a_host_that_looks_like_an_option_is_refused() {
        let mut srv = server("agent", None);
        srv.id = "srv-evil".into();
        srv.host = "-oProxyCommand=touch /tmp/crow-pwned".into();
        let h = SshHost::new(&srv, None, None, "/run/crow".into());
        assert!(h.unsupported.as_deref().unwrap().contains("invalid host"));
        assert!(h.exec(&["true"], Duration::from_secs(1)).is_err());
        assert!(!std::path::Path::new("/tmp/crow-pwned").exists());
    }

    #[test]
    fn password_auth_is_reported_not_attempted() {
        let h = SshHost::new(&server("password", None), None, None, "/run/crow".into());
        let err = h.exec(&["true"], Duration::from_secs(1)).unwrap_err();
        assert!(matches!(err, HostError::Unreachable(m) if m.contains("password")));
        assert!(matches!(connection_state("srv-1"), Some(ConnectionState::AuthFailed(_))));
    }

    #[test]
    fn quoting_survives_the_remote_shell() {
        assert_eq!(shell_quote("journalctl"), "journalctl");
        assert_eq!(shell_quote("_PID=42"), "_PID=42");
        assert_eq!(shell_quote("it's here"), "'it'\\''s here'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn ssh_failures_are_classified() {
        assert!(matches!(classify_ssh_failure("ops@10.0.4.12: Permission denied (publickey).\n"), ConnectionState::AuthFailed(_)));
        assert!(matches!(classify_ssh_failure("Host key verification failed.\n"), ConnectionState::HostKeyRejected(_)));
        assert!(matches!(classify_ssh_failure("ssh: connect to host 10.0.4.12 port 22: Connection refused\n"), ConnectionState::Unreachable(_)));
    }

    /// Runs SshHost against a stand-in `ssh` that executes the remote command
    /// with a local shell — the argv building, quoting and every exec-based
    /// file operation go through the same path a real server would.
    #[test]
    fn end_to_end_through_a_fake_ssh_client() {
        let dir = std::env::temp_dir().join(format!("crow-fake-ssh-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("ssh");
        // Last argument is the remote command string; everything before is ssh options.
        std::fs::write(&fake, "#!/bin/sh\nfor last; do :; done\nexec sh -c \"$last\"\n").unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut srv = server("agent", None);
        srv.id = "srv-fake".into();
        let h = SshHost::new(&srv, None, None, dir.clone()).with_program(fake.to_str().unwrap());

        // Executing a just-written script can hit ETXTBSY when a parallel test
        // forks while the file is still open for writing; retry briefly.
        let mut attempt = 0;
        let out = loop {
            match h.exec(&["printf", "%s|%s", "it's", "a b"], Duration::from_secs(5)) {
                Err(HostError::Io(e)) if e.contains("busy") && attempt < 20 => {
                    attempt += 1;
                    std::thread::sleep(Duration::from_millis(50));
                }
                other => break other.unwrap(),
            }
        };
        assert_eq!(out.stdout, "it's|a b");
        assert_eq!(connection_state("srv-fake"), Some(ConnectionState::Connected));

        let work = dir.join("work dir");
        let w = work.to_str().unwrap();
        h.create_dir(w).unwrap();
        let file = format!("{w}/app.conf");
        h.write_file_atomic(&file, "key = 'value'\n").unwrap();
        assert_eq!(h.read_file(&file).unwrap(), "key = 'value'\n");
        assert_eq!(h.list_dir(w).unwrap().iter().map(|e| e.name.clone()).collect::<Vec<_>>(), vec!["app.conf"]);

        // A failing remote command is the command's failure, not a connection problem.
        let err = h.exec(&["sh", "-c", "exit 3"], Duration::from_secs(5)).unwrap_err();
        assert!(matches!(err, HostError::Failed { status: 3, .. }));
        assert_eq!(connection_state("srv-fake"), Some(ConnectionState::Connected));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Live test against a real sshd. Opt-in:
    ///   CROW_LIVE_SSH="127.0.0.1:22222:root:/path/to/key" \
    ///   CROW_LIVE_KNOWN_HOSTS=/path/to/known_hosts cargo test live_ssh -- --ignored
    /// (a known_hosts file for that endpoint, e.g. from `ssh-keyscan -p 22222 127.0.0.1`).
    #[test]
    #[ignore]
    fn live_ssh_round_trip() {
        let spec = std::env::var("CROW_LIVE_SSH").expect("set CROW_LIVE_SSH=host:port:user:key");
        let known_hosts = std::env::var("CROW_LIVE_KNOWN_HOSTS").expect("set CROW_LIVE_KNOWN_HOSTS");
        let parts: Vec<&str> = spec.splitn(4, ':').collect();
        let dir = std::env::temp_dir().join(format!("crow-live-ssh-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Real ssh, pointed at the test known_hosts instead of the user's.
        let wrapper = |kh: &str| {
            let path = dir.join(format!("ssh-{}", kh.len()));
            std::fs::write(&path, format!("#!/bin/sh\nexec ssh -o UserKnownHostsFile={kh} \"$@\"\n")).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            path.display().to_string()
        };
        let srv = ServerRecord {
            id: "live".into(),
            host: parts[0].into(),
            port: parts[1].parse().unwrap(),
            login_user: parts[2].into(),
            auth_method: "publickey".into(),
            key_id: Some("k".into()),
            ..ServerRecord::default()
        };
        let h = SshHost::new(&srv, Some(parts[3].into()), None, dir.clone()).with_program(&wrapper(&known_hosts));

        let out = h.exec(&["uname", "-s"], Duration::from_secs(20)).unwrap();
        assert_eq!(out.stdout.trim(), "Linux");
        assert_eq!(connection_state("live"), Some(ConnectionState::Connected));
        // The ControlMaster socket keeps the connection for the next command.
        assert!(std::fs::read_dir(&dir).unwrap().flatten().any(|e| e.file_type().map(|t| t.is_socket()).unwrap_or(false)));
        let started = std::time::Instant::now();
        h.exec(&["true"], Duration::from_secs(5)).unwrap();
        assert!(started.elapsed() < Duration::from_millis(500), "multiplexed exec took {:?}", started.elapsed());

        assert_eq!(h.exec_privileged(&["id", "-u"], &[], Duration::from_secs(5)).unwrap().stdout.trim(), "0");
        let f = "/tmp/crow live test.conf";
        h.write_file_privileged(f, "a = 'b'\n").unwrap();
        assert_eq!(h.read_file(f).unwrap(), "a = 'b'\n");
        assert!(h.list_dir("/tmp").unwrap().iter().any(|e| e.name == "crow live test.conf"));
        h.remove(f, false).unwrap();
        assert!(matches!(h.exec(&["sh", "-c", "exit 7"], Duration::from_secs(5)), Err(HostError::Failed { status: 7, .. })));

        // An unknown host key is rejected, not silently accepted.
        let empty_kh = dir.join("empty_known_hosts");
        std::fs::write(&empty_kh, "").unwrap();
        let mut strict = srv.clone();
        strict.id = "live-strict".into();
        let h2 = SshHost::new(&strict, Some(parts[3].into()), None, dir.join("sub")).with_program(&wrapper(empty_kh.to_str().unwrap()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        assert!(h2.exec(&["true"], Duration::from_secs(20)).is_err());
        assert!(matches!(connection_state("live-strict"), Some(ConnectionState::HostKeyRejected(_))));

        // A closed port is unreachable.
        let mut closed = srv.clone();
        closed.id = "live-closed".into();
        closed.port = 1;
        let h3 = SshHost::new(&closed, Some(parts[3].into()), None, dir.join("sub")).with_program(&wrapper(&known_hosts));
        assert!(h3.exec(&["true"], Duration::from_secs(20)).is_err());
        assert!(matches!(connection_state("live-closed"), Some(ConnectionState::Unreachable(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_reach_the_ssh_options_and_security_ones_stay_fixed() {
        let args = base_options(&SshSettings { connect_timeout: 20, keepalive_interval: 30, control_master: false }).join(" ");
        assert!(args.contains("ConnectTimeout=20") && args.contains("ServerAliveInterval=30"));
        assert!(args.contains("ControlMaster=no") && args.contains("ControlPersist=no"));
        for fixed in ["BatchMode=yes", "StrictHostKeyChecking=yes", "ForwardAgent=no", "ForwardX11=no"] {
            assert!(args.contains(fixed), "{fixed} is always set");
        }
        let default = base_options(&SshSettings::default()).join(" ");
        assert!(default.contains("ControlMaster=auto") && default.contains("ControlPersist=600"));
    }

    #[test]
    fn fresh_login_never_reuses_a_connection_and_uses_keys_only() {
        let args = fresh_login_args(vec!["-o".into(), "ControlMaster=auto".into(), "-o".into(), "ControlPersist=600".into(), "-o".into(), "ControlPath=/run/crow/%C".into(), "--".into(), "10.0.0.1".into()]);
        let joined = args.join(" ");
        assert!(joined.contains("ControlMaster=no") && joined.contains("ControlPersist=no") && joined.contains("ControlPath=none"));
        assert!(!joined.contains("ControlMaster=auto"));
        let dashdash = args.iter().position(|a| a == "--").unwrap();
        assert_eq!(args[dashdash - 1], "PreferredAuthentications=publickey", "before the destination");
    }

    #[test]
    fn passphrase_free_key_files_skip_the_agent_and_encrypted_ones_keep_it() {
        let dir = std::env::temp_dir().join(format!("crow-err87-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let keygen = |name: &str, passphrase: &str| {
            let path = dir.join(name);
            let ok = std::process::Command::new("ssh-keygen").args(["-q", "-t", "ed25519", "-N", passphrase, "-f"]).arg(&path).status().is_ok_and(|s| s.success());
            ok.then(|| path.to_string_lossy().into_owned())
        };
        let (Some(plain), Some(locked)) = (keygen("plain", ""), keygen("locked", "a passphrase")) else {
            eprintln!("ssh-keygen not available; skipping");
            return;
        };
        assert!(key_text_usable_without_agent(&std::fs::read_to_string(&plain).unwrap()));
        assert!(!key_text_usable_without_agent(&std::fs::read_to_string(&locked).unwrap()), "an encrypted key may live unlocked in the agent");
        assert!(!key_text_usable_without_agent("-----BEGIN RSA PRIVATE KEY-----\nnot openssh\n"), "unrecognised: keep the agent");

        let srv = ServerRecord { id: "s".into(), host: "10.0.0.1".into(), login_user: "root".into(), auth_method: "publickey".into(), key_id: Some("k".into()), ..Default::default() };
        let args = |key: &str| SshHost::new(&srv, Some(key.to_string()), None, dir.clone()).args.join(" ");
        assert!(args(&plain).contains("IdentityAgent=none"), "Crow's kind of key: no agent, no prompts");
        assert!(!args(&locked).contains("IdentityAgent"), "a passphrase key keeps the agent");
        let agent = ServerRecord { auth_method: "agent".into(), key_id: None, ..srv.clone() };
        assert!(!SshHost::new(&agent, None, None, dir.clone()).args.join(" ").contains("IdentityAgent"), "agent logins keep the agent");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn recognises_agents_that_ask_for_approval() {
        let one = "identityagent /Users/me/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock\n";
        assert_eq!(approval_agent_name(one, None), Some("1Password"));
        assert_eq!(approval_agent_name("user root\n", Some("/home/me/.1password/agent.sock")), Some("1Password"), "from SSH_AUTH_SOCK");
        assert_eq!(approval_agent_name("identityagent SSH_AUTH_SOCK\n", Some("/Users/me/Library/Containers/com.maxgoedjen.Secretive.SecretAgent/Data/socket.ssh")), Some("Secretive"));
        assert_eq!(approval_agent_name("identityagent none\n", Some("/home/me/.1password/agent.sock")), None, "Crow turned the agent off");
        assert_eq!(approval_agent_name("user root\n", Some("/run/user/1000/keyring/ssh")), None);
    }
}
