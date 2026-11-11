//! Cross-platform user-level background service manager for `crow --agent` (ERR-206).
//!
//! Installs, starts, stops, and removes the user service with zero `sudo`/root
//! privileges and zero host environment pollution:
//! - Linux: `~/.config/systemd/user/rs.crow.Crow-agent.service` (`systemd --user`)
//! - macOS: `~/Library/LaunchAgents/rs.crow.Crow.agent.plist` (`launchd`)

use std::path::{Path, PathBuf};

pub const SYSTEMD_UNIT_NAME: &str = "rs.crow.Crow-agent.service";
pub const LAUNCHD_LABEL: &str = "rs.crow.Crow.agent";
pub const LAUNCHD_PLIST_NAME: &str = "rs.crow.Crow.agent.plist";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceBackend {
    SystemdUser,
    LaunchdUser,
    Unsupported,
}

impl ServiceBackend {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::LaunchdUser
        } else if cfg!(target_os = "linux") {
            Self::SystemdUser
        } else {
            Self::Unsupported
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SystemdUser => "systemd --user",
            Self::LaunchdUser => "macOS LaunchAgent",
            Self::Unsupported => "Unsupported OS",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServiceStatus {
    pub backend: Option<ServiceBackend>,
    pub unit_path: Option<PathBuf>,
    pub installed: bool,
    pub running: bool,
    pub service_pid: Option<u32>,
    pub detail: String,
}

pub fn systemd_unit_path(home: &Path) -> PathBuf {
    home.join(".config")
        .join("systemd")
        .join("user")
        .join(SYSTEMD_UNIT_NAME)
}

pub fn launchd_plist_path(home: &Path) -> PathBuf {
    home.join("Library")
        .join("LaunchAgents")
        .join(LAUNCHD_PLIST_NAME)
}

pub fn current_unit_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match ServiceBackend::current() {
        ServiceBackend::SystemdUser => Some(systemd_unit_path(&home)),
        ServiceBackend::LaunchdUser => Some(launchd_plist_path(&home)),
        ServiceBackend::Unsupported => None,
    }
}

/// Generates the `systemd --user` unit file content for `crow --daemon`.
/// When `in_flatpak` is true, invokes `flatpak run --command=crow rs.crow.Crow --daemon`.
pub fn generate_systemd_unit(exe_path: &Path, in_flatpak: bool) -> String {
    let exec_start = if in_flatpak {
        "/usr/bin/flatpak run --command=crow rs.crow.Crow --daemon".to_string()
    } else {
        let escaped = exe_path.display().to_string().replace('"', "\\\"");
        format!("\"{escaped}\" --daemon")
    };

    format!(
        "[Unit]\n\
         Description=Crow Fleet Background Monitor & Daemon\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={exec_start}\n\
         Restart=on-failure\n\
         RestartSec=10\n\
         \n\
         [Install]\n\
         WantedBy=default.target\n"
    )
}

/// Generates the macOS `LaunchAgent` plist XML content for `crow --daemon`.
pub fn generate_launchd_plist(exe_path: &Path) -> String {
    let escaped = xml_escape(&exe_path.display().to_string());
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
             <key>Label</key>\n\
             <string>{LAUNCHD_LABEL}</string>\n\
             <key>ProgramArguments</key>\n\
             <array>\n\
                 <string>{escaped}</string>\n\
                 <string>--daemon</string>\n\
             </array>\n\
             <key>RunAtLoad</key>\n\
             <true/>\n\
             <key>KeepAlive</key>\n\
             <dict>\n\
                 <key>SuccessfulExit</key>\n\
                 <false/>\n\
             </dict>\n\
             <key>ProcessType</key>\n\
             <string>Background</string>\n\
         </dict>\n\
         </plist>\n"
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Queries the current OS user-service status without modifying anything.
pub fn query_status() -> ServiceStatus {
    let backend = ServiceBackend::current();
    let Some(home) = dirs::home_dir() else {
        return ServiceStatus {
            backend: Some(backend),
            detail: "HOME directory unavailable".into(),
            ..Default::default()
        };
    };

    match backend {
        ServiceBackend::SystemdUser => query_systemd_status(&home),
        ServiceBackend::LaunchdUser => query_launchd_status(&home),
        ServiceBackend::Unsupported => ServiceStatus {
            backend: Some(backend),
            detail: "Background user service is supported on Linux and macOS".into(),
            ..Default::default()
        },
    }
}

fn query_systemd_status(home: &Path) -> ServiceStatus {
    let path = systemd_unit_path(home);
    let installed = path.exists();
    if !installed {
        return ServiceStatus {
            backend: Some(ServiceBackend::SystemdUser),
            unit_path: Some(path),
            installed: false,
            running: false,
            service_pid: None,
            detail: "Not installed".into(),
        };
    }

    let active_out = crate::sandbox::host_command(
        &["systemctl", "--user", "is-active", SYSTEMD_UNIT_NAME],
        None,
        None,
    )
    .and_then(|mut c| c.output());

    let state_str = active_out
        .as_ref()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    let running = state_str == "active";

    let service_pid = if running {
        crate::sandbox::host_command(
            &[
                "systemctl",
                "--user",
                "show",
                SYSTEMD_UNIT_NAME,
                "--property=MainPID",
                "--value",
            ],
            None,
            None,
        )
        .and_then(|mut c| c.output())
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
        .filter(|&pid| pid > 0)
    } else {
        None
    };

    ServiceStatus {
        backend: Some(ServiceBackend::SystemdUser),
        unit_path: Some(path),
        installed: true,
        running,
        service_pid,
        detail: if running {
            "Active (systemd --user)".into()
        } else {
            format!("Installed ({state_str})")
        },
    }
}

fn query_launchd_status(home: &Path) -> ServiceStatus {
    let path = launchd_plist_path(home);
    let installed = path.exists();
    if !installed {
        return ServiceStatus {
            backend: Some(ServiceBackend::LaunchdUser),
            unit_path: Some(path),
            installed: false,
            running: false,
            service_pid: None,
            detail: "Not installed".into(),
        };
    }

    let uid = unsafe { libc::geteuid() };
    let target = format!("gui/{uid}/{LAUNCHD_LABEL}");
    let out = crate::sandbox::host_command(&["launchctl", "print", &target], None, None)
        .and_then(|mut c| c.output());

    let (running, service_pid) = match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let pid = text
                .lines()
                .find_map(|l| l.trim().strip_prefix("pid = "))
                .and_then(|v| v.trim().parse::<u32>().ok())
                .filter(|&p| p > 0);
            let is_running = pid.is_some() || text.contains("state = running");
            (is_running, pid)
        }
        _ => (false, None),
    };

    ServiceStatus {
        backend: Some(ServiceBackend::LaunchdUser),
        unit_path: Some(path),
        installed: true,
        running,
        service_pid,
        detail: if running {
            "Active (LaunchAgent)".into()
        } else {
            "Installed (stopped)".into()
        },
    }
}

/// Installs the user service file and starts it immediately.
pub fn install_and_start() -> Result<ServiceStatus, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine HOME directory".to_string())?;
    let exe = std::env::current_exe().map_err(|e| format!("Failed to locate crow binary: {e}"))?;

    match ServiceBackend::current() {
        ServiceBackend::SystemdUser => {
            let path = systemd_unit_path(&home);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
            }
            let unit = generate_systemd_unit(&exe, crate::sandbox::flatpak());
            std::fs::write(&path, unit)
                .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

            run_host_checked(&["systemctl", "--user", "daemon-reload"])?;
            run_host_checked(&["systemctl", "--user", "enable", "--now", SYSTEMD_UNIT_NAME])?;
            Ok(query_systemd_status(&home))
        }
        ServiceBackend::LaunchdUser => {
            let path = launchd_plist_path(&home);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
            }
            let plist = generate_launchd_plist(&exe);
            std::fs::write(&path, plist)
                .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

            let uid = unsafe { libc::geteuid() };
            let domain = format!("gui/{uid}");
            let path_str = path.display().to_string();
            let _ = crate::sandbox::host_command(
                &["launchctl", "bootout", &domain, &path_str],
                None,
                None,
            )
            .and_then(|mut c| c.output());
            run_host_checked(&["launchctl", "bootstrap", &domain, &path_str])?;
            Ok(query_launchd_status(&home))
        }
        ServiceBackend::Unsupported => Err("Background service is only supported on Linux and macOS".into()),
    }
}

/// Stops and removes the user service file.
pub fn stop_and_uninstall() -> Result<ServiceStatus, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine HOME directory".to_string())?;

    match ServiceBackend::current() {
        ServiceBackend::SystemdUser => {
            let path = systemd_unit_path(&home);
            let _ = crate::sandbox::host_command(
                &["systemctl", "--user", "disable", "--now", SYSTEMD_UNIT_NAME],
                None,
                None,
            )
            .and_then(|mut c| c.output());
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
            }
            let _ = crate::sandbox::host_command(&["systemctl", "--user", "daemon-reload"], None, None)
                .and_then(|mut c| c.output());
            Ok(query_systemd_status(&home))
        }
        ServiceBackend::LaunchdUser => {
            let path = launchd_plist_path(&home);
            let uid = unsafe { libc::geteuid() };
            let domain = format!("gui/{uid}");
            let path_str = path.display().to_string();
            let _ = crate::sandbox::host_command(
                &["launchctl", "bootout", &domain, &path_str],
                None,
                None,
            )
            .and_then(|mut c| c.output());
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
            }
            Ok(query_launchd_status(&home))
        }
        ServiceBackend::Unsupported => Err("Background service is only supported on Linux and macOS".into()),
    }
}

fn run_host_checked(argv: &[&str]) -> Result<(), String> {
    let out = crate::sandbox::host_command(argv, None, None)
        .and_then(|mut c| c.output())
        .map_err(|e| format!("Failed to execute `{}`: {e}", argv.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("exit code {:?}", out.status.code())
        };
        return Err(format!("`{}` failed: {detail}", argv.join(" ")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_native_and_flatpak_systemd_unit() {
        let native = generate_systemd_unit(Path::new("/home/user/.local/bin/crow"), false);
        assert!(native.contains("ExecStart=\"/home/user/.local/bin/crow\" --daemon"));
        assert!(native.contains("Restart=on-failure"));
        assert!(native.contains("WantedBy=default.target"));

        let flatpak = generate_systemd_unit(Path::new("/app/bin/crow"), true);
        assert!(flatpak.contains("ExecStart=/usr/bin/flatpak run --command=crow rs.crow.Crow --daemon"));
    }

    #[test]
    fn generates_macos_launchd_plist() {
        let plist = generate_launchd_plist(Path::new("/Applications/Crow.app/Contents/MacOS/crow"));
        assert!(plist.contains("<string>rs.crow.Crow.agent</string>"));
        assert!(plist.contains("<string>/Applications/Crow.app/Contents/MacOS/crow</string>"));
        assert!(plist.contains("<string>--daemon</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
    }
}
