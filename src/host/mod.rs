//! Host access seam: every command Crow runs and every file it touches on a
//! managed server goes through a [`Host`]. Collectors and actions ask
//! [`host_for`] for the server's transport and never branch on "is this
//! localhost" themselves.
//!
//! Transports: this machine ([`LocalHost`]), Crow-managed lab containers
//! ([`ContainerHost`]) and everything else over the system OpenSSH client
//! ([`SshHost`]).

pub mod bootstrap;
mod container;
mod local;
pub mod ssh;
pub mod reboot;

use std::collections::HashMap;
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

/// `list_dir` for many directories in one round trip: `$1` is a per-call
/// marker, the rest are directories. Each readable directory is announced by
/// a `<marker> DIR <path>` line followed by its `stat` lines.
const LIST_DIRS_SCRIPT: &str = r#"m=$1; shift; for d in "$@"; do (cd -- "$d" 2>/dev/null || exit 0; printf '%s DIR %s\n' "$m" "$d"; for f in * .[!.]* ..?*; do if [ -e "$f" ] || [ -L "$f" ]; then stat -c '%F|%a|%u|%g|%s|%Y|%n' -- "$f"; fi; done); done; true"#;

/// `read_file` for many files in one round trip: `$1` is a per-call marker,
/// the rest are paths. Each file is framed as `<marker> BEGIN <path>\n`,
/// its bytes, then `\n<marker> OK|ERR\n`.
const READ_FILES_SCRIPT: &str = r#"m=$1; shift; for f in "$@"; do printf '%s BEGIN %s\n' "$m" "$f"; if [ -f "$f" ] && cat -- "$f" 2>/dev/null; then printf '\n%s OK\n' "$m"; else printf '\n%s ERR\n' "$m"; fi; done"#;

/// A marker no file content or name will contain by accident.
fn batch_marker() -> String {
    format!("@@crow-{:032x}@@", rand::random::<u128>())
}

/// Parses `LIST_DIRS_SCRIPT` output.
pub fn parse_dir_batch(stdout: &str, marker: &str) -> HashMap<String, Vec<DirEntry>> {
    let header = format!("{marker} DIR ");
    let mut out: HashMap<String, Vec<DirEntry>> = HashMap::new();
    let mut current: Option<String> = None;
    let mut lines = String::new();
    let flush = |dir: Option<String>, lines: &mut String, out: &mut HashMap<String, Vec<DirEntry>>| {
        if let Some(d) = dir {
            out.insert(d, parse_stat_listing(lines));
        }
        lines.clear();
    };
    for line in stdout.lines() {
        if let Some(dir) = line.strip_prefix(&header) {
            flush(current.take(), &mut lines, &mut out);
            current = Some(dir.to_string());
        } else if current.is_some() {
            lines.push_str(line);
            lines.push('\n');
        }
    }
    flush(current, &mut lines, &mut out);
    out
}

/// Parses `READ_FILES_SCRIPT` output: path → content, or an error.
pub fn parse_file_batch(stdout: &str, marker: &str) -> HashMap<String, Result<String, String>> {
    let begin = format!("{marker} BEGIN ");
    let end = format!("\n{marker} ");
    let mut out = HashMap::new();
    let mut rest = stdout;
    while let Some(start) = rest.find(&begin) {
        let after = &rest[start + begin.len()..];
        let Some(nl) = after.find('\n') else { break };
        let path = after[..nl].to_string();
        let body = &after[nl + 1..];
        let Some(stop) = body.find(&end) else { break };
        let status_line = body[stop + end.len()..].lines().next().unwrap_or("");
        let result = if status_line == "OK" {
            Ok(body[..stop].to_string())
        } else {
            Err("not a readable file".to_string())
        };
        out.insert(path, result);
        rest = &body[stop + end.len()..];
    }
    out
}

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

    /// Releases any long-lived connection this transport keeps open, so a
    /// server that has left the fleet stops holding a session (ERR-32).
    /// Returns true when there was nothing to release or it closed cleanly.
    /// Local and container hosts keep no such connection.
    fn close_connection(&self) -> bool {
        true
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

    /// Lists many directories in one round trip. Directories that don't exist
    /// or can't be read are left out.
    fn list_dirs(&self, paths: &[&str]) -> HashMap<String, Vec<DirEntry>> {
        let marker = batch_marker();
        let mut argv = vec!["sh", "-c", LIST_DIRS_SCRIPT, "crow-ls", marker.as_str()];
        argv.extend(paths);
        match self.exec(&argv, DEFAULT_TIMEOUT) {
            Ok(out) => parse_dir_batch(&out.stdout, &marker),
            Err(_) => HashMap::new(),
        }
    }

    /// Reads many files in one round trip. Every path gets an entry: its
    /// content, or why it couldn't be read.
    fn read_files(&self, paths: &[&str]) -> HashMap<String, Result<String, String>> {
        if paths.is_empty() {
            return HashMap::new();
        }
        let marker = batch_marker();
        let mut argv = vec!["sh", "-c", READ_FILES_SCRIPT, "crow-cat", marker.as_str()];
        argv.extend(paths);
        let mut out = match self.exec(&argv, DEFAULT_TIMEOUT) {
            Ok(out) => parse_file_batch(&out.stdout, &marker),
            Err(e) => return paths.iter().map(|p| (p.to_string(), Err(e.to_string()))).collect(),
        };
        for p in paths {
            out.entry(p.to_string()).or_insert_with(|| Err("no output for this file".into()));
        }
        out
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
pub fn host_for(server: &ServerRecord) -> Arc<dyn Host> {
    if server.tags.iter().any(|t| t == "test-node") {
        let engine = if server.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
        return Arc::new(ContainerHost::new(engine, &server.name));
    }
    if is_this_machine(server) {
        return Arc::new(LocalHost);
    }
    Arc::new(SshHost::for_server(server))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs commands on this machine but uses the trait's default (remote)
    /// file methods, counting round trips.
    struct ShellHost(std::sync::atomic::AtomicUsize);
    impl Host for ShellHost {
        fn label(&self) -> String {
            "shell".into()
        }
        fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            LocalHost.exec_stdin(argv, stdin, timeout)
        }
    }

    #[test]
    fn batched_reads_and_listings_take_one_round_trip_each() {
        let dir = std::env::temp_dir().join(format!("crow-batch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        let path = |n: &str| dir.join(n).to_string_lossy().into_owned();
        // Content that looks like framing, no trailing newline, empty, unicode.
        let tricky = "line\n@@crow-fake@@ OK\n\nSTATUS ERR\nno newline at end";
        std::fs::write(path("a.conf"), tricky).unwrap();
        std::fs::write(path("empty.conf"), "").unwrap();
        std::fs::write(path("u.conf"), "naïve · ✓\n\n").unwrap();
        std::fs::write(path("sub/x"), "x").unwrap();

        let host = ShellHost(Default::default());
        let files = [path("a.conf"), path("empty.conf"), path("u.conf"), path("missing.conf"), path("sub")];
        let refs: Vec<&str> = files.iter().map(String::as_str).collect();
        let got = host.read_files(&refs);
        assert_eq!(host.0.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(got[&files[0]].as_deref(), Ok(tricky));
        assert_eq!(got[&files[1]].as_deref(), Ok(""));
        assert_eq!(got[&files[2]].as_deref(), Ok("naïve · ✓\n\n"));
        assert!(got[&files[3]].is_err(), "missing file");
        assert!(got[&files[4]].is_err(), "a directory is not a file");
        // Same answers as the direct local implementation.
        let local = LocalHost.read_files(&refs[..3]);
        for f in &refs[..3] {
            assert_eq!(local[*f], got[*f]);
        }

        let dirs = [path(""), path("sub"), path("nope")];
        let refs: Vec<&str> = dirs.iter().map(String::as_str).collect();
        let listed = host.list_dirs(&refs);
        assert_eq!(host.0.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert!(!listed.contains_key(&dirs[2]));
        let mut names: Vec<_> = listed[&dirs[0]].iter().map(|e| e.name.clone()).collect();
        names.sort();
        assert_eq!(names, ["a.conf", "empty.conf", "sub", "u.conf"]);
        assert_eq!(listed[&dirs[1]].len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn record(host: &str, port: u16, tags: &[&str]) -> ServerRecord {
        ServerRecord { host: host.into(), port, login_user: "ops".into(), tags: tags.iter().map(|t| t.to_string()).collect(), ..ServerRecord::default() }
    }

    #[test]
    fn routing_picks_container_then_this_machine_then_ssh() {
        assert_eq!(host_for(&record("127.0.0.1", 2222, &["local", "test-node", "podman"])).label(), "podman:");
        assert_eq!(host_for(&record("127.0.0.1", 2222, &["local", "test-node", "docker"])).label(), "docker:");
        assert_eq!(host_for(&record("localhost", 22, &[])).label(), "local");
        // Loopback on another port is a forwarded SSH endpoint, not this machine.
        assert_eq!(host_for(&record("127.0.0.1", 2222, &["local"])).label(), "ssh:ops@127.0.0.1");
        assert_eq!(host_for(&record("10.0.4.12", 22, &[])).label(), "ssh:ops@10.0.4.12");
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
