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
        let mut args: Vec<String> = [
            "-o", "BatchMode=yes",
            "-o", "StrictHostKeyChecking=yes",
            // Crow never needs them; override whatever ~/.ssh/config says.
            "-o", "ForwardAgent=no",
            "-o", "ForwardX11=no",
            "-o", "ConnectTimeout=8",
            "-o", "ServerAliveInterval=15",
            "-o", "ServerAliveCountMax=2",
            "-o", "ControlMaster=auto",
            "-o", "ControlPersist=600",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
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
                Some(path) => args.extend(["-i".into(), expand_home(path), "-o".into(), "IdentitiesOnly=yes".into()]),
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
}
