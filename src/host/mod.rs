//! Host access seam: every command Crow runs and every file it touches on a
//! managed server goes through a [`Host`]. Collectors and actions ask
//! [`host_for`] for the server's transport and never branch on "is this
//! localhost" themselves.
//!
//! Transports: this machine ([`LocalHost`]), Crow-managed lab containers
//! ([`ContainerHost`]) and everything else over the system OpenSSH client
//! ([`SshHost`]).

mod container;
mod local;
pub mod ssh;

use std::sync::Arc;
use std::time::Duration;

use crate::vault::ServerRecord;

pub use container::ContainerHost;
pub use local::LocalHost;
pub use ssh::{connection_state, update_directory, ConnectionState, SshHost};

/// Default budget for a single command; long enough for `journalctl` on a busy
/// box, short enough that a wedged host can't stall a poll tick forever.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostError {
    /// No transport to this server (or it could not be reached).
    Unreachable(String),
    /// The command did not finish within its timeout and was killed.
    Timeout,
    /// The command ran and exited non-zero.
    Failed { status: i32, stderr: String },
    /// Local I/O around the transport failed (spawn, pipes, filesystem).
    Io(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::Unreachable(why) => write!(f, "host unreachable: {why}"),
            HostError::Timeout => write!(f, "command timed out"),
            HostError::Failed { status, stderr } => {
                let msg = stderr.trim();
                if msg.is_empty() { write!(f, "exited with status {status}") } else { write!(f, "{msg}") }
            }
            HostError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for HostError {}

impl From<std::io::Error> for HostError {
    fn from(e: std::io::Error) -> Self {
        HostError::Io(e.to_string())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExecOutput {
    pub stdout: String,
    pub stderr: String,
}

/// One directory entry with the metadata the Files screen shows.
#[derive(Clone, Debug, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size_bytes: u64,
    /// Permission bits (the low 12 bits of st_mode).
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    /// Seconds since the Unix epoch.
    pub mtime: i64,
}

/// Writes stdin to "$1" atomically: temp file beside it, keep the original
/// mode (and owner, when root), then rename over it. Uses `stat -c`, which
/// GNU coreutils and BusyBox both support (BusyBox has no `chmod --reference`).
const ATOMIC_WRITE_SCRIPT: &str = r#"set -e; t=$(mktemp "$1.crow.XXXXXX"); cat > "$t"; if [ -e "$1" ]; then chmod "$(stat -c %a "$1")" "$t"; chown "$(stat -c %u:%g "$1")" "$t" 2>/dev/null || true; fi; mv -f "$t" "$1""#;

/// Lists "$1" one entry per line as `type|mode|uid|gid|size|mtime|path`, with
/// `stat -c` (GNU and BusyBox alike; `find -printf` is GNU-only). The globs
/// cover dotfiles; unmatched globs stay literal and are skipped.
const LIST_DIR_SCRIPT: &str = r#"cd -- "$1" || exit 1; for f in * .[!.]* ..?*; do if [ -e "$f" ] || [ -L "$f" ]; then stat -c '%F|%a|%u|%g|%s|%Y|%n' -- "$f"; fi; done"#;

/// Runs "$@" as root: as-is when already root, else via non-interactive sudo.
const PRIVILEGED_WRAPPER: &str = r#"if [ "$(id -u)" = 0 ]; then exec "$@"; else exec sudo -n -- "$@"; fi"#;

/// A machine Crow can run commands on and read/write files of.
///
/// Every call is blocking, can fail, and is bounded by a timeout — call it from
/// a background executor, never the UI thread. The file methods have default
/// implementations in terms of [`Host::exec`] so a new transport only has to
/// provide `exec`/`exec_stdin`; `LocalHost` overrides them with direct I/O.
pub trait Host: Send + Sync {
    /// Short human label for messages, e.g. `local` or `podman:crow-lab-noble`.
    fn label(&self) -> String;

    /// True only for the machine Crow itself runs on.
    fn is_local(&self) -> bool {
        false
    }

    /// Runs `argv` and returns its output; a non-zero exit is `HostError::Failed`.
    fn exec(&self, argv: &[&str], timeout: Duration) -> Result<ExecOutput, HostError> {
        self.exec_stdin(argv, &[], timeout)
    }

    /// Runs `argv` with `stdin` piped to it.
    fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError>;

    /// Runs `argv` as root: directly when already root, otherwise through
    /// `sudo -n` (never prompts — a sudo that needs a password fails clearly).
    fn exec_privileged(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
        let mut full: Vec<&str> = vec!["sh", "-c", PRIVILEGED_WRAPPER, "crow-priv"];
        full.extend_from_slice(argv);
        self.exec_stdin(&full, stdin, timeout).map_err(|e| match e {
            HostError::Failed { status, stderr } if stderr.contains("sudo:") && stderr.contains("password") => HostError::Failed {
                status,
                stderr: "sudo needs a password on this host; Crow runs sudo non-interactively (allow NOPASSWD for this user, or connect as root)".into(),
            },
            other => other,
        })
    }

    fn read_file(&self, path: &str) -> Result<String, HostError> {
        Ok(self.exec(&["cat", "--", path], DEFAULT_TIMEOUT)?.stdout)
    }

    fn exists(&self, path: &str) -> bool {
        self.exec(&["test", "-e", path], DEFAULT_TIMEOUT).is_ok()
    }

    fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, HostError> {
        let out = self.exec(&["sh", "-c", LIST_DIR_SCRIPT, "crow-ls", path], DEFAULT_TIMEOUT)?;
        Ok(parse_stat_listing(&out.stdout))
    }

    /// Replaces `path` with `content` atomically (temp file in the same
    /// directory, then rename), keeping the original file's mode when it exists.
    fn write_file_atomic(&self, path: &str, content: &str) -> Result<(), HostError> {
        self.exec_stdin(&["sh", "-c", ATOMIC_WRITE_SCRIPT, "crow-write", path], content.as_bytes(), DEFAULT_TIMEOUT)?;
        Ok(())
    }

    /// Like `write_file_atomic`, falling back to root (see `exec_privileged`)
    /// when the file isn't writable as the connected user — e.g. under /etc.
    fn write_file_privileged(&self, path: &str, content: &str) -> Result<(), HostError> {
        match self.write_file_atomic(path, content) {
            Ok(()) => Ok(()),
            Err(HostError::Failed { .. }) | Err(HostError::Io(_)) => self
                .exec_privileged(&["sh", "-c", ATOMIC_WRITE_SCRIPT, "crow-write", path], content.as_bytes(), DEFAULT_TIMEOUT)
                .map(|_| ()),
            Err(e) => Err(e),
        }
    }

    fn create_dir(&self, path: &str) -> Result<(), HostError> {
        self.exec(&["mkdir", "--", path], DEFAULT_TIMEOUT).map(|_| ())
    }

    /// Removes a file, or an empty directory — never recursive.
    fn remove(&self, path: &str, is_dir: bool) -> Result<(), HostError> {
        let argv: [&str; 3] = if is_dir { ["rmdir", "--", path] } else { ["rm", "--", path] };
        self.exec(&argv, DEFAULT_TIMEOUT).map(|_| ())
    }
}

/// Parses `stat -c '%F|%a|%u|%g|%s|%Y|%n'` lines (see `LIST_DIR_SCRIPT`).
pub fn parse_stat_listing(stdout: &str) -> Vec<DirEntry> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(7, '|');
            let kind = parts.next()?;
            let mode = u32::from_str_radix(parts.next()?, 8).ok()?;
            let uid = parts.next()?.parse().ok()?;
            let gid = parts.next()?.parse().ok()?;
            let size_bytes = parts.next()?.parse().ok()?;
            let mtime = parts.next()?.parse().ok()?;
            let name = parts.next()?.to_string();
            Some(DirEntry {
                name,
                is_dir: kind == "directory",
                is_symlink: kind == "symbolic link",
                size_bytes,
                mode,
                uid,
                gid,
                mtime,
            })
        })
        .collect()
}

/// True for the machine Crow itself runs on: a loopback address on the
/// standard SSH port (or none). Loopback on another port is an SSH endpoint
/// forwarded somewhere else — e.g. a container at 127.0.0.1:2222.
pub fn is_this_machine(server: &ServerRecord) -> bool {
    let loopback = matches!(server.host.as_str(), "127.0.0.1" | "localhost" | "::1");
    loopback && matches!(server.port, 0 | 22)
}

/// Which transport `host_for` picks for a server.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportKind {
    Local,
    Container,
    Ssh,
}

pub fn transport_kind(server: &ServerRecord) -> TransportKind {
    if server.tags.iter().any(|t| t == "test-node") {
        TransportKind::Container
    } else if is_this_machine(server) {
        TransportKind::Local
    } else {
        TransportKind::Ssh
    }
}

/// The transport for `server`: its lab container, this machine, or SSH.
pub fn host_for(server: &ServerRecord) -> Option<Arc<dyn Host>> {
    if server.tags.iter().any(|t| t == "test-node") {
        let engine = if server.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
        return Some(Arc::new(ContainerHost::new(engine, &server.name)));
    }
    if is_this_machine(server) {
        return Some(Arc::new(LocalHost));
    }
    Some(Arc::new(SshHost::for_server(server)))
}

/// The error an action reports for a server with no transport.
pub fn not_connected(server: &ServerRecord) -> HostError {
    HostError::Unreachable(format!("{} is not connected — Crow has no transport to it yet", server.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(host: &str, port: u16, tags: &[&str]) -> ServerRecord {
        ServerRecord { host: host.into(), port, login_user: "ops".into(), tags: tags.iter().map(|t| t.to_string()).collect(), ..ServerRecord::default() }
    }

    #[test]
    fn routing_picks_container_then_this_machine_then_ssh() {
        assert_eq!(host_for(&record("127.0.0.1", 2222, &["local", "test-node", "podman"])).unwrap().label(), "podman:");
        assert_eq!(host_for(&record("127.0.0.1", 2222, &["local", "test-node", "docker"])).unwrap().label(), "docker:");
        assert_eq!(host_for(&record("localhost", 22, &[])).unwrap().label(), "local");
        // Loopback on another port is a forwarded SSH endpoint, not this machine.
        assert_eq!(host_for(&record("127.0.0.1", 2222, &["local"])).unwrap().label(), "ssh:ops@127.0.0.1");
        assert_eq!(host_for(&record("10.0.4.12", 22, &[])).unwrap().label(), "ssh:ops@10.0.4.12");
    }

    #[test]
    fn privileged_exec_runs_directly_as_root_or_via_sudo() {
        // As a normal user this goes through `sudo -n`, which either works
        // (NOPASSWD) or fails with a clear message — never a prompt.
        match LocalHost.exec_privileged(&["id", "-u"], &[], DEFAULT_TIMEOUT) {
            Ok(out) => assert_eq!(out.stdout.trim(), "0"),
            Err(HostError::Failed { stderr, .. }) => assert!(!stderr.is_empty()),
            Err(e) => panic!("unexpected: {e:?}"),
        }
    }

    #[test]
    fn parses_stat_listing() {
        let out = "directory|755|0|0|4096|1726000000|ssh
regular file|644|0|0|340|1726000100|hosts
symbolic link|777|0|0|7|1726000200|mtab
regular empty file|600|1000|1000|0|1726000300|my notes|draft.txt
";
        let entries = parse_stat_listing(out);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0], DirEntry { name: "ssh".into(), is_dir: true, is_symlink: false, size_bytes: 4096, mode: 0o755, uid: 0, gid: 0, mtime: 1726000000 });
        assert!(entries[2].is_symlink);
        // Names may contain the separator and spaces: the name is the last field.
        assert_eq!(entries[3].name, "my notes|draft.txt");
    }

}
