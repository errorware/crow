use gpui_kit::Rgba;
use serde::{Deserialize, Serialize};
use crate::theme::*;

/// Real, computed connection counts for a service about to be restarted/stopped —
/// the Apply Pipeline's "blast radius" made literal, in place of a generic warning.
#[derive(Clone, Debug, Default)]
pub struct BlastRadiusInfo {
    pub for_unit: String,
    pub established: usize,
    pub listening: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceUnit {
    pub name: String,
    pub status: String, // "ACTIVE", "DEGRADED", "FAILED", "INACTIVE"
    pub status_color_hex: u32,
    pub pid: String,
    pub cpu: String,
    pub mem: String,
    pub rss: String,
    pub uptime: String,
    pub description: String,
    pub is_focused: bool,
    pub show_confirm: bool,
}

impl ServiceUnit {
    pub fn status_color(&self) -> Rgba {
        match self.status.as_str() {
            "ACTIVE" => OK,
            "DEGRADED" | "PENDING" => WARN,
            "FAILED" => CRIT,
            _ => TEXT_MUTED,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessUnit {
    pub pid: u32,
    pub user: String,
    pub cpu: f32,
    pub mem: f32,
    pub rss: String,
    pub stat: String, // "R", "S", "D", "Z", "I", etc.
    pub time: String,
    pub command: String,
    /// A kernel thread (kthreadd, PID 2, or one of its children).
    #[serde(default)]
    pub is_kernel: bool,
    pub is_focused: bool,
    pub show_confirm: bool,
}

impl ProcessUnit {
    pub fn stat_color(&self) -> Rgba {
        if self.stat.starts_with('R') {
            OK
        } else if self.stat.starts_with('D') {
            WARN
        } else if self.stat.starts_with('Z') {
            CRIT
        } else {
            TEXT_MUTED
        }
    }

    pub fn stat_label(&self) -> &'static str {
        if self.stat.starts_with('R') {
            "RUN"
        } else if self.stat.starts_with('S') {
            "SLEEP"
        } else if self.stat.starts_with('D') {
            "DISK"
        } else if self.stat.starts_with('Z') {
            "ZOMBIE"
        } else {
            "IDLE"
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SocketUnit {
    pub protocol: String, // "TCP", "UDP", "TCP6", "UDP6"
    pub state: String,    // "LISTEN", "ESTAB", "UNCONN", "TIME-WAIT", etc.
    pub local_addr: String,
    pub local_port: String,
    pub peer_addr: String,
    pub peer_port: String,
    pub process: String,
    pub pid: Option<u32>,
    pub is_focused: bool,
    #[serde(default)]
    pub bytes_sent: Option<u64>,
    #[serde(default)]
    pub bytes_recv: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerCategory {
    Loopback,
    Private,
    Public,
}

impl PeerCategory {
    pub fn label(&self) -> &'static str {
        match self {
            PeerCategory::Loopback => "LOOPBACK",
            PeerCategory::Private => "PRIVATE NET",
            PeerCategory::Public => "PUBLIC INTERNET",
        }
    }

    pub fn color(&self) -> Rgba {
        match self {
            PeerCategory::Loopback => TEXT_FAINT,
            PeerCategory::Private => hex_rgb(0xa78bfa), // Light purple
            PeerCategory::Public => hex_rgb(0x38bdf8),  // Cyan
        }
    }
}

/// Resolved connection model for the live connection map view
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionMapItem {
    pub socket: SocketUnit,
    pub direction: ConnectionDirection,
    pub peer_category: PeerCategory,
    pub remote_host: Option<String>,
}

impl SocketUnit {
    pub fn proto_color(&self) -> Rgba {
        if self.protocol.starts_with("TCP") {
            hex_rgb(0x60a5fa) // Light Blue
        } else {
            hex_rgb(0xfb923c) // Orange
        }
    }

    pub fn state_color(&self) -> Rgba {
        match self.state.to_uppercase().as_str() {
            "LISTEN" => OK,
            "ESTAB" | "ESTABLISHED" => hex_rgb(0x38bdf8), // Cyan
            "TIME-WAIT" | "CLOSE-WAIT" => WARN,
            _ => TEXT_MUTED,
        }
    }
}
