//! Crow's built-in plugins (ERR-138): integrations that do nothing until
//! they're switched on in Settings → Plugins. Each has a category and
//! capabilities in ERR-41's vocabulary; only enabled ones are probed, and
//! only they add their buttons and actions.

use std::time::Duration;

use crate::host::{Host, HostError, LocalHost};
use crate::lab::multipass::{self, InstallStep, MultipassStatus};

pub struct BuiltinPlugin {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub category: &'static str,
    pub capabilities: &'static [&'static str],
}

pub const BUILTIN: &[BuiltinPlugin] = &[
    BuiltinPlugin {
        id: "multipass",
        name: "Multipass",
        about: "Ubuntu VMs on this machine: launch one with Crow's key and it joins the fleet; import, power and delete them.",
        category: "provider.vms",
        capabilities: &["vms.launch", "vms.import", "vms.power"],
    },
    BuiltinPlugin {
        id: "podman",
        name: "Podman",
        about: "Lab containers on this machine, enrolled as test nodes.",
        category: "provider.containers",
        capabilities: &["containers.list", "containers.power"],
    },
    BuiltinPlugin {
        id: "distrobox",
        name: "Distrobox",
        about: "Distrobox environments (Podman or Docker underneath), as lab containers.",
        category: "provider.containers",
        capabilities: &["containers.list", "containers.power"],
    },
    BuiltinPlugin {
        id: "docker",
        name: "Docker",
        about: "Lab containers run by Docker.",
        category: "provider.containers",
        capabilities: &["containers.list", "containers.power"],
    },
    BuiltinPlugin {
        id: "linode",
        name: "Linode",
        about: "Your Linode instances: import them, read their regions, power them and take snapshots.",
        category: "provider.hosts",
        capabilities: &["instances.list", "instances.power", "snapshots"],
    },
    BuiltinPlugin {
        id: "upcloud",
        name: "UpCloud",
        about: "Your UpCloud servers: import them, read their zones, power them and take snapshots.",
        category: "provider.hosts",
        capabilities: &["instances.list", "instances.power", "snapshots"],
    },
];

pub fn get(id: &str) -> Option<&'static BuiltinPlugin> {
    BUILTIN.iter().find(|p| p.id == id)
}

/// The vault flag that holds whether `id` is on.
pub fn flag_key(id: &str) -> String {
    format!("plugin.{id}.enabled")
}

/// Set once the first enabled set has been chosen (from what was in use).
pub const INITIALIZED_FLAG: &str = "plugins.initialized";

/// Plugins in the containers category: any of them shows the lab.
pub const CONTAINER_PLUGINS: [&str; 3] = ["podman", "distrobox", "docker"];

/// Whether a plugin can do its job here, and if not, what to do about it.
#[derive(Clone, Debug, PartialEq)]
pub enum PluginStatus {
    Ready(String),
    /// Installed, but not usable yet: why, and the steps that fix it.
    Problem { summary: String, steps: Vec<InstallStep> },
    /// Not on this machine: how to install it.
    Missing { steps: Vec<InstallStep> },
    /// Compiled out of this build.
    Unavailable,
}

impl PluginStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }
}

fn step(why: &str, command: &str) -> InstallStep {
    InstallStep { why: why.into(), command: command.into() }
}

const QUICK: Duration = Duration::from_secs(15);

fn has_command(name: &str) -> bool {
    LocalHost.exec(&["sh", "-c", "command -v \"$1\"", "crow-which", name], QUICK).is_ok()
}

/// `argv`'s output, or the first line of what it said when it failed.
fn run(argv: &[&str]) -> Result<String, String> {
    LocalHost.exec(argv, QUICK).map(|o| o.stdout.trim().to_string()).map_err(|e| match e {
        HostError::Failed { stderr, .. } => stderr.lines().find(|l| !l.trim().is_empty()).unwrap_or("failed").trim().to_string(),
        other => other.to_string(),
    })
}

/// Docker's status from asking its daemon (`docker version` without a
/// server answers only for the client). `answer` is the server version or
/// the error.
pub fn docker_status(installed: bool, answer: Result<String, String>) -> PluginStatus {
    if !installed {
        return PluginStatus::Missing { steps: vec![step("Install Docker Engine (or use Podman, already on many distros)", "https://docs.docker.com/engine/install/")] };
    }
    match answer {
        Ok(v) if !v.is_empty() => PluginStatus::Ready(format!("Docker {v}")),
        Err(e) if e.contains("permission denied") => PluginStatus::Problem {
            summary: "installed, but you can't reach Docker's daemon".into(),
            steps: vec![step(
                "Add yourself to the docker group, then log out and back in (that group is as good as root on this machine)",
                "sudo usermod -aG docker $USER",
            )],
        },
        Err(e) if e.contains("Cannot connect") || e.contains("Is the docker daemon running") => PluginStatus::Problem {
            summary: "installed, but Docker's daemon isn't running".into(),
            steps: vec![step("Start Docker now and at every boot", "sudo systemctl enable --now docker")],
        },
        Ok(_) => PluginStatus::Problem { summary: "the daemon gave no version".into(), steps: Vec::new() },
        Err(e) => PluginStatus::Problem { summary: e, steps: Vec::new() },
    }
}

/// Probes plugin `id` on this machine. Blocking (it runs the tools);
/// call it off the UI thread, and only for plugins being looked at.
pub fn status(id: &str) -> PluginStatus {
    match id {
        "multipass" => match multipass::detect() {
            MultipassStatus::Ready { daemon, fixes, .. } if fixes.is_empty() => PluginStatus::Ready(format!("Multipass {daemon}")),
            MultipassStatus::Ready { fixes, .. } => PluginStatus::Problem { summary: "running, but VMs won't get a network".into(), steps: fixes },
            MultipassStatus::DaemonDown { detail, steps, .. } => PluginStatus::Problem { summary: format!("installed, but its daemon isn't answering ({detail})"), steps },
            MultipassStatus::Missing { steps } => PluginStatus::Missing { steps },
        },
        "podman" => {
            if !has_command("podman") {
                return PluginStatus::Missing { steps: vec![step("Install Podman", "sudo dnf install podman   # or: sudo apt install podman")] };
            }
            match run(&["podman", "info", "--format", "{{.Version.Version}}"]) {
                Ok(v) => PluginStatus::Ready(format!("Podman {v}")),
                Err(e) => PluginStatus::Problem { summary: e, steps: Vec::new() },
            }
        }
        "docker" => {
            let installed = has_command("docker");
            docker_status(installed, if installed { run(&["docker", "version", "--format", "{{.Server.Version}}"]) } else { Err(String::new()) })
        }
        "distrobox" => {
            if !has_command("distrobox") {
                return PluginStatus::Missing { steps: vec![step("Install Distrobox", "sudo dnf install distrobox   # or: sudo apt install distrobox")] };
            }
            let version = run(&["distrobox", "--version"]).unwrap_or_default();
            if status("podman").is_ready() || status("docker").is_ready() {
                PluginStatus::Ready(version.replace("distrobox: ", "Distrobox "))
            } else {
                PluginStatus::Problem { summary: "needs Podman or Docker working underneath".into(), steps: Vec::new() }
            }
        }
        "linode" | "upcloud" => match crate::providers::factory(id) {
            Some(_) => PluginStatus::Ready("built in".into()),
            None => PluginStatus::Unavailable,
        },
        _ => PluginStatus::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_is_ready_only_when_its_daemon_answers() {
        assert_eq!(docker_status(true, Ok("29.7.2".into())), PluginStatus::Ready("Docker 29.7.2".into()));
        // What this machine said on 2026-10-08, the user not in the docker group.
        let denied = docker_status(true, Err("permission denied while trying to connect to the docker API at unix:///var/run/docker.sock".into()));
        assert!(matches!(&denied, PluginStatus::Problem { summary, steps } if summary.contains("can't reach") && steps[0].command.contains("usermod -aG docker")));
        let down = docker_status(true, Err("Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?".into()));
        assert!(matches!(&down, PluginStatus::Problem { steps, .. } if steps[0].command.contains("systemctl enable --now docker")));
        assert!(matches!(docker_status(false, Err(String::new())), PluginStatus::Missing { .. }));
    }

    #[test]
    fn every_plugin_has_a_category_and_flag() {
        for p in BUILTIN {
            assert!(p.category.starts_with("provider."), "{}", p.id);
            assert!(!p.capabilities.is_empty());
            assert_eq!(flag_key(p.id), format!("plugin.{}.enabled", p.id));
        }
    }

    /// Prints every plugin's status on this machine:
    ///   cargo test live_plugin_status -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_plugin_status() {
        for p in BUILTIN {
            eprintln!("{:10} {:?}", p.id, status(p.id));
        }
    }
}
