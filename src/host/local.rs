use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{DirEntry, ExecOutput, Host, HostError};

/// The machine Crow itself runs on: commands via `std::process`, files via `std::fs`.
pub struct LocalHost;

impl Host for LocalHost {
    fn label(&self) -> String {
        "local".to_string()
    }

    fn is_local(&self) -> bool {
        true
    }

    fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
        run_command(argv, stdin, timeout)
    }

    fn read_file(&self, path: &str) -> Result<String, HostError> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn list_dirs(&self, paths: &[&str]) -> std::collections::HashMap<String, Vec<DirEntry>> {
        paths.iter().filter_map(|p| Some((p.to_string(), self.list_dir(p).ok()?))).collect()
    }

    fn read_files(&self, paths: &[&str]) -> std::collections::HashMap<String, Result<String, String>> {
        paths.iter().map(|p| (p.to_string(), self.read_file(p).map_err(|e| e.to_string()))).collect()
    }

    fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>, HostError> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path)?.flatten() {
            let (Ok(file_type), Ok(meta)) = (entry.file_type(), entry.metadata()) else { continue };
            out.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir: file_type.is_dir(),
                is_symlink: file_type.is_symlink(),
                size_bytes: meta.size(),
                mode: meta.mode() & 0o7777,
                uid: meta.uid(),
                gid: meta.gid(),
                mtime: meta.mtime(),
            });
        }
        Ok(out)
    }

    fn write_file_atomic(&self, path: &str, content: &str) -> Result<(), HostError> {
        let target = Path::new(path);
        let dir = target.parent().ok_or_else(|| HostError::Io(format!("{path} has no parent directory")))?;
        let file_name = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let tmp = dir.join(format!(".{file_name}.crow-{}", std::process::id()));
        std::fs::write(&tmp, content)?;
        if let Ok(meta) = std::fs::metadata(target) {
            let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(meta.mode() & 0o7777));
        }
        std::fs::rename(&tmp, target).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            HostError::from(e)
        })
    }

    fn create_dir(&self, path: &str) -> Result<(), HostError> {
        Ok(std::fs::create_dir(path)?)
    }

    fn remove(&self, path: &str, is_dir: bool) -> Result<(), HostError> {
        if is_dir {
            std::fs::remove_dir(path).map_err(|e| HostError::Io(format!("{e} (directory must be empty)")))
        } else {
            Ok(std::fs::remove_file(path)?)
        }
    }
}

/// Spawns `argv` locally, feeds it `stdin`, and waits up to `timeout`,
/// killing the child if it runs over. Non-zero exit maps to `HostError::Failed`.
pub(super) fn run_command(argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
    run_command_env(argv, stdin, timeout, &[])
}

/// `run_command` with extra environment variables for the child.
pub(super) fn run_command_env(argv: &[&str], stdin: &[u8], timeout: Duration, env: &[(&str, &str)]) -> Result<ExecOutput, HostError> {
    let (program, args) = argv.split_first().ok_or_else(|| HostError::Io("empty command".into()))?;
    let mut child = Command::new(program)
        .args(args)
        .envs(env.iter().copied())
        .stdin(if stdin.is_empty() { Stdio::null() } else { Stdio::piped() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Feed stdin and drain stdout/stderr on their own threads so a chatty
    // child can't deadlock on a full pipe while we wait on it.
    let stdin_writer = child.stdin.take().map(|mut pipe| {
        let data = stdin.to_vec();
        std::thread::spawn(move || {
            let _ = pipe.write_all(&data);
        })
    });
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        })
    };
    let stdout = drain(child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let stderr = drain(child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>));

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(HostError::Timeout);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if let Some(w) = stdin_writer {
        let _ = w.join();
    }
    let out = ExecOutput {
        stdout: String::from_utf8_lossy(&stdout.join().unwrap_or_default()).to_string(),
        stderr: String::from_utf8_lossy(&stderr.join().unwrap_or_default()).to_string(),
    };
    if status.success() {
        Ok(out)
    } else {
        Err(HostError::Failed { status: status.code().unwrap_or(-1), stderr: out.stderr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_captures_output_and_stdin() {
        let out = LocalHost.exec_stdin(&["cat"], b"hello", Duration::from_secs(5)).unwrap();
        assert_eq!(out.stdout, "hello");
    }

    #[test]
    fn exec_reports_failure_and_timeout() {
        let err = LocalHost.exec(&["sh", "-c", "echo nope >&2; exit 3"], Duration::from_secs(5)).unwrap_err();
        assert_eq!(err, HostError::Failed { status: 3, stderr: "nope\n".into() });
        assert_eq!(LocalHost.exec(&["sleep", "5"], Duration::from_millis(100)).unwrap_err(), HostError::Timeout);
    }

    #[test]
    fn atomic_write_replaces_content_and_keeps_mode() {
        let dir = std::env::temp_dir().join(format!("crow-host-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("app.conf");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        let p = path.to_str().unwrap();

        LocalHost.write_file_atomic(p, "new").unwrap();
        assert_eq!(LocalHost.read_file(p).unwrap(), "new");
        assert_eq!(std::fs::metadata(&path).unwrap().mode() & 0o777, 0o640);

        let listing = LocalHost.list_dir(dir.to_str().unwrap()).unwrap();
        assert_eq!(listing.len(), 1, "temp file must not be left behind");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn default_file_methods_work_over_exec() {
        // Exercise the trait's exec-based defaults (what ContainerHost/SSH use)
        // against the local shell.
        struct ExecOnly;
        impl Host for ExecOnly {
            fn label(&self) -> String { "exec-only".into() }
            fn exec_stdin(&self, argv: &[&str], stdin: &[u8], timeout: Duration) -> Result<ExecOutput, HostError> {
                run_command(argv, stdin, timeout)
            }
        }
        let dir = std::env::temp_dir().join(format!("crow-host-exec-{}", std::process::id()));
        let d = dir.to_str().unwrap();
        let h = ExecOnly;
        h.create_dir(d).unwrap();
        let f = format!("{d}/a.conf");
        h.write_file_atomic(&f, "x = 1\n").unwrap();
        assert!(h.exists(&f));
        assert_eq!(h.read_file(&f).unwrap(), "x = 1\n");
        let listing = h.list_dir(d).unwrap();
        assert_eq!(listing.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["a.conf"]);
        h.remove(&f, false).unwrap();
        h.remove(d, true).unwrap();
        assert!(!h.exists(d));
    }
}
