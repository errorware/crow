pub mod multipass;

use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::vault::{ServerRecord, VaultDb};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalLabEngine {
    Distrobox,
    Podman,
    Multipass,
    Docker,
}

impl LocalLabEngine {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Distrobox => "Distrobox",
            Self::Podman => "Podman",
            Self::Multipass => "Multipass",
            Self::Docker => "Docker",
        }
    }

    pub fn command_bin(&self) -> &'static str {
        match self {
            Self::Distrobox => "distrobox",
            Self::Podman => "podman",
            Self::Multipass => "multipass",
            Self::Docker => "docker",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTestNode {
    pub id: String,
    pub name: String,
    pub engine: LocalLabEngine,
    pub image: String,
    pub state: String, // "running", "stopped", "exited"
    pub ssh_port: Option<u16>,
    pub is_enrolled: bool,
}

impl LocalTestNode {
    pub fn is_running(&self) -> bool {
        self.state.to_lowercase() == "running" || self.state.to_lowercase().starts_with("up")
    }

    pub fn endpoint_display(&self) -> String {
        if let Some(port) = self.ssh_port {
            format!("127.0.0.1:{}", port)
        } else {
            "local bridge".to_string()
        }
    }

    pub fn distro_display(&self) -> String {
        let img = self.image.to_lowercase();
        if img.contains("ubuntu") {
            "Ubuntu".to_string()
        } else if img.contains("debian") {
            "Debian".to_string()
        } else if img.contains("alpine") {
            "Alpine".to_string()
        } else if img.contains("fedora") {
            "Fedora".to_string()
        } else if img.contains("completo") {
            "Ubuntu 22.04 (SSH Lab)".to_string()
        } else {
            self.image.clone()
        }
    }
}

/// Scans the local host for existing test containers and VMs
pub fn scan_local_test_nodes(enrolled_servers: &[ServerRecord]) -> Vec<LocalTestNode> {
    let mut nodes = Vec::new();

    // 1. Query podman containers (which also includes distrobox containers)
    if let Ok(output) = Command::new("podman")
        .args(["ps", "-a", "--format", "json"])
        .output()
    {
        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(parsed) = parse_podman_json(&json_str) {
                for mut node in parsed {
                    // Check if already enrolled in Crow
                    node.is_enrolled = enrolled_servers.iter().any(|s| {
                        s.host == "127.0.0.1" && s.port == node.ssh_port.unwrap_or(0) || s.name == node.name
                    });
                    nodes.push(node);
                }
            }
        }
    }

    nodes
}

#[derive(Deserialize)]
struct PodmanContainerEntry {
    #[serde(rename = "Id")]
    id: Option<String>,
    #[serde(rename = "Names")]
    names: Option<Vec<String>>,
    #[serde(rename = "Image")]
    image: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "Labels")]
    labels: Option<serde_json::Value>,
    #[serde(rename = "Ports")]
    ports: Option<Vec<PodmanPortEntry>>,
}

#[derive(Deserialize)]
struct PodmanPortEntry {
    host_port: Option<u16>,
    container_port: Option<u16>,
}

/// Parses the output of `podman ps -a --format json`
pub fn parse_podman_json(json_str: &str) -> Result<Vec<LocalTestNode>, serde_json::Error> {
    let entries: Vec<PodmanContainerEntry> = serde_json::from_str(json_str)?;
    let mut nodes = Vec::new();

    for e in entries {
        let name = e.names.as_ref().and_then(|n| n.first()).cloned().unwrap_or_else(|| "unnamed".into());
        let id = e.id.clone().unwrap_or_else(|| name.clone());
        let image = e.image.unwrap_or_else(|| "unknown".into());
        let state = e.state.unwrap_or_else(|| "stopped".into());

        // Check if created via distrobox
        let is_distrobox = e.labels.as_ref()
            .and_then(|l| l.get("manager"))
            .and_then(|m| m.as_str())
            .map(|m| m == "distrobox")
            .unwrap_or(false);

        let engine = if is_distrobox {
            LocalLabEngine::Distrobox
        } else {
            LocalLabEngine::Podman
        };

        // Find mapped SSH port (port 22)
        let mut ssh_port = None;
        if let Some(ref ports) = e.ports {
            for p in ports {
                if p.container_port == Some(22) || p.host_port.is_some() {
                    ssh_port = p.host_port;
                    break;
                }
            }
        }

        nodes.push(LocalTestNode {
            id,
            name,
            engine,
            image,
            state,
            ssh_port,
            is_enrolled: false,
        });
    }

    Ok(nodes)
}

/// Starts a local test node using its container manager
pub fn start_local_node(node: &LocalTestNode) -> Result<(), String> {
    let cmd_name = match node.engine {
        LocalLabEngine::Distrobox => "distrobox",
        LocalLabEngine::Podman => "podman",
        LocalLabEngine::Docker => "docker",
        LocalLabEngine::Multipass => "multipass",
    };

    let args = match node.engine {
        LocalLabEngine::Distrobox => vec!["enter", &node.name, "--", "true"],
        LocalLabEngine::Multipass => vec!["start", &node.name],
        _ => vec!["start", &node.name],
    };

    let out = Command::new(cmd_name)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd_name, e))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Stops a local test node using its container manager
pub fn stop_local_node(node: &LocalTestNode) -> Result<(), String> {
    let cmd_name = match node.engine {
        LocalLabEngine::Distrobox => "distrobox",
        LocalLabEngine::Podman => "podman",
        LocalLabEngine::Docker => "docker",
        LocalLabEngine::Multipass => "multipass",
    };

    let args = match node.engine {
        LocalLabEngine::Distrobox => vec!["stop", &node.name, "--yes"],
        LocalLabEngine::Multipass => vec!["stop", &node.name],
        _ => vec!["stop", &node.name],
    };

    let out = Command::new(cmd_name)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd_name, e))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Creates a ServerRecord representing this local test node ready to be enrolled in Crow's SQLite database
pub fn build_server_record_for_node(node: &LocalTestNode) -> ServerRecord {
    let now = chrono::Utc::now().to_rfc3339();
    let port = node.ssh_port.unwrap_or(2222);
    let id = format!("local-{}", node.name.to_lowercase().replace(' ', "-"));

    ServerRecord {
        id,
        name: node.name.clone(),
        host: "127.0.0.1".to_string(),
        port,
        login_user: "root".to_string(),
        auth_method: "agent".to_string(),
        key_id: None,
        jump_host_id: None,
        env: "LAB".to_string(),
        role: format!("test-node · {}", node.engine.label().to_lowercase()),
        group_name: "local-lab".to_string(),
        tags: vec!["local".into(), "test-node".into(), node.engine.label().to_lowercase()],
        host_key_fingerprint: None,
        os_distro: node.distro_display(),
        // Not probed: left empty rather than guessed (a container shares
        // the host's kernel and sees the host's memory and disk).
        os_kernel: String::new(),
        arch: String::new(),
        memory_total: String::new(),
        disk_total: String::new(),
        agent_installed: false,
        agent_version: None,
        status: if node.is_running() { "online".into() } else { "offline".into() },
        created_at: now.clone(),
        last_seen_at: Some(now),
        archived_at: None,
        purged_at: None,
        ..Default::default()
    }
}

/// Enrolls a local test node into SQLite database
pub fn enroll_local_node_into_db(node: &LocalTestNode, db: &VaultDb) -> Result<ServerRecord, String> {
    let srv = build_server_record_for_node(node);
    db.upsert_server(&srv).map_err(|e| e.to_string())?;
    Ok(srv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_podman_json() {
        let sample = r#"[
            {
                "Id": "abc123456",
                "Names": ["test-node-ubuntu"],
                "Image": "docker.io/library/ubuntu:24.04",
                "State": "running",
                "Ports": [
                    { "host_port": 2222, "container_port": 22 }
                ],
                "Labels": { "manager": "podman" }
            }
        ]"#;

        let nodes = parse_podman_json(sample).unwrap();
        assert_eq!(nodes.len(), 1);
        let n = &nodes[0];
        assert_eq!(n.name, "test-node-ubuntu");
        assert_eq!(n.ssh_port, Some(2222));
        assert!(n.is_running());
        assert_eq!(n.distro_display(), "Ubuntu");
    }

    #[test]
    fn test_build_server_record() {
        let node = LocalTestNode {
            id: "node-1".into(),
            name: "completo-node-1".into(),
            engine: LocalLabEngine::Podman,
            image: "localhost/completo-test-node:latest".into(),
            state: "running".into(),
            ssh_port: Some(2222),
            is_enrolled: false,
        };

        let rec = build_server_record_for_node(&node);
        assert_eq!(rec.name, "completo-node-1");
        assert_eq!(rec.host, "127.0.0.1");
        assert_eq!(rec.port, 2222);
        assert_eq!(rec.env, "LAB");
    }
}
