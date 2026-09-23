//! Host access seam: every command Crow runs and every file it touches on a
//! managed server goes through a [`Host`]. Collectors and actions ask
//! [`host_for`] for the server's transport and never branch on "is this
//! localhost" themselves.
//!
//! Today there are two transports — this machine ([`LocalHost`]) and Crow-managed
//! lab containers ([`ContainerHost`]). Remote servers have none yet (`host_for`
//! returns `None`); the SSH transport slots in there.

mod container;
mod local;

use std::sync::Arc;
use std::time::Duration;

use crate::vault::ServerRecord;

pub use container::ContainerHost;
pub use local::LocalHost;

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

/// A machine Crow can run commands on and read/write files of.
///
/// Every call is blocking, can fail, and is bounded by a timeout — call it from
/// a background executor, never the UI thread. The file methods have default
/// implementations in terms of [`Host::exec`] so a new transport only has to
/// provide `exec`/`exec_stdin`; `LocalHost` overrides them with direct I/O.
pub trait Host: Send + Sync {
    /// Short human label for messages, e.g. `local` or `podman:crow-lab-noble`.
    fn label(&self) -> String;

    /// Runs `argv` and returns its output; a non-zero exit is `HostError::Failed`.
    fn exec(&self, argv: &[&str], timeout: Duration) -> Result<ExecOutput, HostError> {
        self.exec_stdin(argv, &[], timeout)
    }

    /// Runs `argv` with `stdin` piped to it.
    fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError>;

    fn read_file(&self, path: &str) -> Result<String, HostError> {
        Ok(self.exec(&["cat", "--", path], DEFAULT_TIMEOUT)?.stdout)
    }

    fn exists(&self, path: &str) -> bool {
        self.exec(&["test", "-e", path], DEFAULT_TIMEOUT).is_ok()
    }

    fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, HostError> {
        // GNU find: one line per entry — type, octal mode, uid, gid, size, mtime, name.
        let out = self.exec(
            &["find", path, "-mindepth", "1", "-maxdepth", "1", "-printf", "%y %m %U %G %s %T@ %f\\n"],
            DEFAULT_TIMEOUT,
        )?;
        Ok(parse_find_listing(&out.stdout))
    }

    /// Replaces `path` with `content` atomically (temp file in the same
    /// directory, then rename), keeping the original file's mode when it exists.
    fn write_file_atomic(&self, path: &str, content: &str) -> Result<(), HostError> {
        let script = r#"set -e; t=$(mktemp "$1.crow.XXXXXX"); cat > "$t"; if [ -e "$1" ]; then chmod --reference="$1" "$t"; fi; mv -f "$t" "$1""#;
        self.exec_stdin(&["sh", "-c", script, "crow-write", path], content.as_bytes(), DEFAULT_TIMEOUT)?;
        Ok(())
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

/// Parses `find -printf '%y %m %U %G %s %T@ %f\n'` output.
pub fn parse_find_listing(stdout: &str) -> Vec<DirEntry> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(7, ' ');
            let kind = parts.next()?;
            let mode = u32::from_str_radix(parts.next()?, 8).ok()?;
            let uid = parts.next()?.parse().ok()?;
            let gid = parts.next()?.parse().ok()?;
            let size_bytes = parts.next()?.parse().ok()?;
            let mtime = parts.next()?.split('.').next()?.parse().ok()?;
            let name = parts.next()?.to_string();
            Some(DirEntry { name, is_dir: kind == "d", is_symlink: kind == "l", size_bytes, mode, uid, gid, mtime })
        })
        .collect()
}

/// True for the machine Crow itself runs on. Lab containers also sit on
/// 127.0.0.1, so they are recognized first by their `test-node` tag.
pub fn is_this_machine(server: &ServerRecord) -> bool {
    server.host == "127.0.0.1"
        || server.host == "localhost"
        || server.host == "::1"
        || server.name.to_lowercase() == "localhost"
        || server.tags.iter().any(|t| t == "localhost" || t == "local")
}

/// The transport for `server`, or `None` when Crow cannot reach it yet
/// (remote servers until the SSH transport lands).
pub fn host_for(server: &ServerRecord) -> Option<Arc<dyn Host>> {
    if server.tags.iter().any(|t| t == "test-node") {
        let engine = if server.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
        return Some(Arc::new(ContainerHost::new(engine, &server.name)));
    }
    if is_this_machine(server) {
        return Some(Arc::new(LocalHost));
    }
    None
}

/// The error an action reports for a server with no transport.
pub fn not_connected(server: &ServerRecord) -> HostError {
    HostError::Unreachable(format!("{} is not connected — Crow has no transport to it yet", server.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(host: &str, tags: &[&str]) -> ServerRecord {
        ServerRecord { host: host.into(), tags: tags.iter().map(|t| t.to_string()).collect(), ..ServerRecord::default() }
    }

    #[test]
    fn routing_prefers_lab_containers_over_localhost() {
        assert_eq!(host_for(&record("127.0.0.1", &["local", "test-node", "podman"])).unwrap().label(), "podman:");
        assert_eq!(host_for(&record("127.0.0.1", &["local", "test-node", "docker"])).unwrap().label(), "docker:");
        assert_eq!(host_for(&record("localhost", &[])).unwrap().label(), "local");
        assert!(host_for(&record("10.0.4.12", &[])).is_none());
    }

    #[test]
    fn parses_find_listing() {
        let out = "d 755 0 0 4096 1726000000.1234567890 ssh\nf 644 0 0 340 1726000100.0 hosts\nl 777 0 0 7 1726000200.5 mtab\n";
        let entries = parse_find_listing(out);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], DirEntry { name: "ssh".into(), is_dir: true, is_symlink: false, size_bytes: 4096, mode: 0o755, uid: 0, gid: 0, mtime: 1726000000 });
        assert!(entries[2].is_symlink);
    }

    #[test]
    fn find_listing_keeps_spaces_in_names() {
        let entries = parse_find_listing("f 644 1000 1000 12 1726000000.0 my notes.txt\n");
        assert_eq!(entries[0].name, "my notes.txt");
    }
}
