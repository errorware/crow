use std::collections::HashMap;

use gpui_kit::component::input::InputState;
use gpui_kit::Entity;

use crate::components::titlebar::ServerTab;
use crate::host::{connection_state, transport_kind, ConnectionState, TransportKind};
use crate::metrics::{ServerMetrics, ServerTimeSeriesBuffer, SurgeAlert};
use crate::vault::ServerRecord;

/// Enrolled servers, the open tabs over them, and their live metrics.
pub struct FleetState {
    pub servers: Vec<ServerRecord>,
    pub tabs: Vec<ServerTab>,
    /// Id of the active tab's server (older tabs may carry the server name).
    pub active_tab_id: String,
    /// Latest metrics per server, keyed by both id and name.
    pub metrics_store: HashMap<String, ServerMetrics>,
    /// Lagged time-series buffers per server, keyed by both id and name.
    pub buffered_stores: HashMap<String, ServerTimeSeriesBuffer>,
    pub metrics_lag_secs: u64,
    pub active_surge_alert: Option<SurgeAlert>,
    /// Height of the Fleet page's alerts/activity panel (drag to resize).
    pub bottom_panel_height: f32,
    /// Servers that left the fleet but were never deleted (ERR-32).
    pub archived: Vec<ServerRecord>,
    /// Which Fleet tab is showing: the active fleet or the archive.
    /// Which page of the Fleet screen is showing.
    pub page: FleetPage,
    /// One-line feedback after an archive, restore or purge.
    pub notice: Option<String>,
    /// Server awaiting archive confirmation.
    pub pending_archive: Option<String>,
    /// Fleet list filters: environment tab, and region (country code; ""
    /// for servers with no known region) under the BY REGION tab (ERR-36).
    pub env_filter: crate::app::region::FleetEnvFilter,
    pub region_bar_open: bool,
    pub region_filter: Option<String>,
    /// A region detection run is in progress / its result.
    pub region_detecting: bool,
    pub region_note: Option<String>,
    /// Server groups (ERR-97): the BY GROUP bar, its filter, the new-group
    /// box, the server whose group is being changed, and the known groups.
    pub group_bar_open: bool,
    pub group_filter: Option<String>,
    pub new_group_input: Option<Entity<InputState>>,
    pub group_assign_modal_server: Option<String>,
    pub groups: Vec<String>,
    /// The host-key rotation policy in days, for the stat (ERR-98).
    pub host_key_policy_days: Option<i64>,
    /// Every server's last-read copy of each baselined file against its
    /// baseline (ERR-74); `None` until computed.
    pub drift: Option<Vec<crate::config::drift::DriftEntry>>,
}

impl FleetState {
    pub fn new(
        servers: Vec<ServerRecord>,
        tabs: Vec<ServerTab>,
        metrics_store: HashMap<String, ServerMetrics>,
        buffered_stores: HashMap<String, ServerTimeSeriesBuffer>,
    ) -> Self {
        let active_tab_id = servers.first().map(|s| s.id.clone()).unwrap_or_default();
        Self {
            servers,
            tabs,
            active_tab_id,
            metrics_store,
            buffered_stores,
            metrics_lag_secs: 24,
            active_surge_alert: None,
            bottom_panel_height: 260.0,
            env_filter: Default::default(),
            region_bar_open: false,
            region_filter: None,
            region_detecting: false,
            region_note: None,
            group_bar_open: false,
            group_filter: None,
            new_group_input: None,
            group_assign_modal_server: None,
            groups: Vec::new(),
            host_key_policy_days: Some(365),
            drift: None,
            archived: Vec::new(),
            page: FleetPage::Active,
            notice: None,
            pending_archive: None,
        }
    }

    /// Archived servers still holding data, with when their purge is due.
    pub fn archive_count(&self) -> usize {
        self.archived.len()
    }

    /// `server`'s live health: its SSH connection state and newest sample
    /// (not the lagged one the tables play back) (ERR-71).
    pub fn health(&self, server: &ServerRecord) -> FleetHealth {
        let conn = (transport_kind(server) == TransportKind::Ssh).then(|| connection_state(&server.id)).flatten();
        let latest = self.buffered_stores.get(&server.id).and_then(|b| b.head()).map(|h| &h.metrics).or_else(|| self.metrics_store.get(&server.id));
        FleetHealth::of(server, conn.as_ref(), latest)
    }

    /// Open tabs with their server's live health color.
    pub fn tabs_with_health(&self) -> Vec<(ServerTab, gpui_kit::Rgba)> {
        self.tabs.iter().map(|t| {
            let color = self.servers.iter().find(|s| s.id == t.id || s.name == t.id).map(|s| self.health(s).color()).unwrap_or(crate::theme::TEXT_FAINTER);
            (t.clone(), color)
        }).collect()
    }

    /// The server behind the active tab.
    pub fn active_server(&self) -> Option<ServerRecord> {
        self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned()
    }
}

/// A fleet row's health, from what Crow saw on the wire just now — never
/// from the status stored at enrollment (ERR-71).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FleetHealth {
    /// Connected and the last probe answered.
    Ok,
    /// Not probed yet this session.
    Checking,
    /// Crow can't reach it; `label` is the badge, `detail` the reason.
    Down { label: &'static str, detail: String },
}

impl FleetHealth {
    /// `connection` is the SSH transport's state (`None` for lab and local
    /// servers, or an SSH server not tried yet); `metrics` is the latest
    /// sample, if one was taken.
    pub fn of(server: &ServerRecord, connection: Option<&ConnectionState>, metrics: Option<&ServerMetrics>) -> Self {
        match (transport_kind(server), connection) {
            (TransportKind::Ssh, None) => FleetHealth::Checking,
            (TransportKind::Ssh, Some(ConnectionState::Connected)) => match metrics {
                Some(m) if !m.reachable => FleetHealth::Down { label: "NO DATA", detail: "connected, but the metrics probe failed".into() },
                _ => FleetHealth::Ok,
            },
            (TransportKind::Ssh, Some(state)) => FleetHealth::Down { label: state.label(), detail: state.detail().unwrap_or_default().to_string() },
            (_, _) => match metrics {
                None => FleetHealth::Checking,
                Some(m) if m.reachable => FleetHealth::Ok,
                Some(_) if transport_kind(server) == TransportKind::Container => FleetHealth::Down { label: "STOPPED", detail: "lab container isn't answering".into() },
                Some(_) => FleetHealth::Down { label: "NO DATA", detail: "the metrics probe failed on this machine".into() },
            },
        }
    }

    pub fn is_ok(&self) -> bool {
        *self == FleetHealth::Ok
    }

    /// Dot / badge color.
    pub fn color(&self) -> gpui_kit::Rgba {
        match self {
            FleetHealth::Ok => crate::theme::OK,
            FleetHealth::Checking => crate::theme::TEXT_FAINTER,
            FleetHealth::Down { .. } => crate::theme::CRIT,
        }
    }
}

/// Host key ages across servers (ERR-81), from each key file's time as last
/// read from the server.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyAges {
    /// (age in days, server name) of the oldest key.
    pub oldest: Option<(i64, String)>,
    /// Under a year, one to two years, over two years.
    pub buckets: [usize; 3],
    /// Servers whose key files Crow hasn't read yet.
    pub unknown: usize,
}

pub fn host_key_ages(servers: &[ServerRecord], now_unix: i64) -> KeyAges {
    let mut ages = KeyAges::default();
    for s in servers {
        let Some(t) = s.host_key_mtime else {
            ages.unknown += 1;
            continue;
        };
        let days = ((now_unix - t) / 86_400).max(0);
        ages.buckets[match days { d if d < 365 => 0, d if d < 730 => 1, _ => 2 }] += 1;
        if ages.oldest.as_ref().is_none_or(|(d, _)| days > *d) {
            ages.oldest = Some((days, s.name.clone()));
        }
    }
    ages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(tags: &[&str], host: &str) -> ServerRecord {
        ServerRecord { id: "s1".into(), name: "s1".into(), host: host.into(), port: 22, tags: tags.iter().map(|t| t.to_string()).collect(), status: "online".into(), ..Default::default() }
    }

    fn metrics(reachable: bool) -> ServerMetrics {
        ServerMetrics { reachable, ..Default::default() }
    }

    #[test]
    fn ssh_health_follows_the_connection_not_the_stored_status() {
        let s = server(&[], "203.0.113.7");
        assert_eq!(FleetHealth::of(&s, None, None), FleetHealth::Checking, "stored \"online\" is not evidence");
        assert_eq!(FleetHealth::of(&s, Some(&ConnectionState::Connected), Some(&metrics(true))), FleetHealth::Ok);
        let down = FleetHealth::of(&s, Some(&ConnectionState::Unreachable("Connection refused".into())), Some(&metrics(false)));
        assert_eq!(down, FleetHealth::Down { label: "UNREACHABLE", detail: "Connection refused".into() });
        assert!(matches!(FleetHealth::of(&s, Some(&ConnectionState::HostKeyRejected("changed".into())), None), FleetHealth::Down { label: "HOST KEY", .. }));
        assert!(matches!(FleetHealth::of(&s, Some(&ConnectionState::Connected), Some(&metrics(false))), FleetHealth::Down { label: "NO DATA", .. }));
    }

    #[test]
    fn lab_health_follows_the_probe() {
        let s = server(&["test-node", "podman"], "127.0.0.1");
        assert_eq!(FleetHealth::of(&s, None, None), FleetHealth::Checking);
        assert_eq!(FleetHealth::of(&s, None, Some(&metrics(true))), FleetHealth::Ok);
        assert!(matches!(FleetHealth::of(&s, None, Some(&metrics(false))), FleetHealth::Down { label: "STOPPED", .. }));
    }

    #[test]
    fn key_ages_bucket_by_year_and_count_unread() {
        let day = 86_400;
        let now = 1_000 * day;
        let srv = |name: &str, t: Option<i64>| ServerRecord { id: name.into(), name: name.into(), host_key_mtime: t, ..Default::default() };
        let ages = host_key_ages(&[srv("new", Some(now - 10 * day)), srv("old", Some(now - 800 * day)), srv("mid", Some(now - 400 * day)), srv("unread", None)], now);
        assert_eq!(ages, KeyAges { oldest: Some((800, "old".into())), buckets: [1, 1, 1], unknown: 1 });
    }
}

/// The Fleet screen's pages: the servers, the archived ones, and the
/// fleet-wide Danger Zone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FleetPage {
    #[default]
    Active,
    Archived,
    Danger,
}
