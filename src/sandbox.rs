//! Crow as a Flatpak (rs.crow.Crow): the sandbox has its own /usr and
//! /etc, so what Crow runs on *this computer* (its local server, lab
//! containers and VMs, the local shell, plugins' local actions) goes
//! through `flatpak-spawn --host`. SSH, curl and the vault stay inside;
//! the runtime ships ssh and curl.

use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

/// Running inside a Flatpak sandbox.
pub fn flatpak() -> bool {
    static IN: OnceLock<bool> = OnceLock::new();
    *IN.get_or_init(|| Path::new("/.flatpak-info").exists())
}

/// Keeps the vault and settings where a native install keeps them
/// (~/.config/crow), so moving between the two loses nothing; the manifest
/// grants that folder, and the vault's key is shared through the session
/// keyring (`app::keyring`). Data (GeoIP, plugins' working folders) stays
/// in the app's own folder, as does the keyring library's, which reads
/// XDG_DATA_HOME. Call first thing in `main`, before any thread starts.
pub fn share_native_folders() {
    if !flatpak() {
        return;
    }
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
    }
}

/// `argv` as a command on this computer. Outside a sandbox it's `argv`
/// itself, with `env` (when given) as its whole environment and `cwd`.
/// Inside, `flatpak-spawn --host` runs it in the desktop session's
/// environment plus `env`, which goes over a pipe rather than the command
/// line, where every user could read it; PATH stays the host's own.
pub fn host_command(argv: &[&str], env: Option<&[(String, String)]>, cwd: Option<&Path>) -> std::io::Result<Command> {
    let (program, args) = argv.split_first().ok_or_else(|| std::io::Error::other("empty command"))?;
    if !flatpak() {
        let mut c = Command::new(program);
        c.args(args);
        if let Some(env) = env {
            c.env_clear().envs(env.iter().map(|(k, v)| (k, v)));
        }
        if let Some(cwd) = cwd {
            c.current_dir(cwd);
        }
        return Ok(c);
    }
    let mut c = Command::new("flatpak-spawn");
    c.args(["--host", "--watch-bus"]);
    if let Some(cwd) = cwd {
        c.arg(format!("--directory={}", cwd.display()));
    }
    if let Some(env) = env.filter(|e| !e.is_empty()) {
        use std::os::fd::AsRawFd;
        use std::os::unix::process::CommandExt;
        let (reader, mut writer) = std::io::pipe()?;
        let mut blob = Vec::new();
        for (k, v) in env.iter().filter(|(k, _)| k != "PATH") {
            blob.extend_from_slice(format!("{k}={v}\0").as_bytes());
        }
        // Small enough for the pipe's buffer: written whole before spawning.
        std::io::Write::write_all(&mut writer, &blob)?;
        drop(writer);
        let fd = reader.as_raw_fd();
        c.arg(format!("--env-fd={fd}"));
        // SAFETY: fcntl on a descriptor this closure owns, between fork and
        // exec; it only lets the child inherit it.
        unsafe {
            c.pre_exec(move || {
                let _keep = &reader;
                if libc::fcntl(fd, libc::F_SETFD, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    c.arg("--").arg(program).args(args);
    Ok(c)
}

/// A terminal's program and arguments, run on this computer: inside a
/// sandbox, `flatpak-spawn` carries the terminal type across.
pub fn host_terminal(program: String, args: Vec<String>) -> (String, Vec<String>) {
    if !flatpak() {
        return (program, args);
    }
    let mut full: Vec<String> = ["--host", "--watch-bus", "--env=TERM=xterm-256color", "--env=COLORTERM=truecolor"].iter().map(|s| s.to_string()).collect();
    if let Some(home) = std::env::var_os("HOME") {
        full.push(format!("--directory={}", home.to_string_lossy()));
    }
    full.push("--".into());
    full.push(program);
    full.extend(args);
    ("flatpak-spawn".into(), full)
}

#[cfg(test)]
mod tests {
    use super::host_command;

    #[test]
    fn outside_a_sandbox_commands_run_as_given() {
        if super::flatpak() {
            return;
        }
        let env = vec![("ONLY".to_string(), "this".to_string())];
        let out = host_command(&["sh", "-c", "echo \"$ONLY:${HOME:-none}\"; pwd"], Some(&env), Some(Path::new("/"))).unwrap().output().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "this:none\n/\n");
    }

    use std::path::Path;
}
