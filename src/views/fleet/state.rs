use std::collections::HashMap;

use crate::components::titlebar::ServerTab;
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
    pub show_archived: bool,
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
            archived: Vec::new(),
            show_archived: false,
            notice: None,
            pending_archive: None,
        }
    }

    /// Archived servers still holding data, with when their purge is due.
    pub fn archive_count(&self) -> usize {
        self.archived.len()
    }

    /// The server behind the active tab.
    pub fn active_server(&self) -> Option<ServerRecord> {
        self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned()
    }
}
