use std::collections::HashMap;

use crate::components::titlebar::ServerTab;
use crate::metrics::{ServerMetrics, ServerTimeSeriesBuffer, SurgeAlert};
use crate::os_detect::DistroFamily;
use crate::vault::ServerRecord;

/// Enrolled servers, the open tabs over them, and their live metrics.
pub struct FleetState {
    pub servers: Vec<ServerRecord>,
    pub tabs: Vec<ServerTab>,
    /// Id of the active tab's server (older tabs may carry the server name).
    pub active_tab_id: String,
    /// This machine's own /etc/os-release family, detected once at startup —
    /// drives which config paths the crawler trusts (see crawl_machine_configs).
    pub local_distro_family: DistroFamily,
    /// Latest metrics per server, keyed by both id and name.
    pub metrics_store: HashMap<String, ServerMetrics>,
    /// Lagged time-series buffers per server, keyed by both id and name.
    pub buffered_stores: HashMap<String, ServerTimeSeriesBuffer>,
    pub metrics_lag_secs: u64,
    pub active_surge_alert: Option<SurgeAlert>,
}

impl FleetState {
    pub fn new(
        servers: Vec<ServerRecord>,
        tabs: Vec<ServerTab>,
        local_distro_family: DistroFamily,
        metrics_store: HashMap<String, ServerMetrics>,
        buffered_stores: HashMap<String, ServerTimeSeriesBuffer>,
    ) -> Self {
        let active_tab_id = servers.first().map(|s| s.id.clone()).unwrap_or_default();
        Self {
            servers,
            tabs,
            active_tab_id,
            local_distro_family,
            metrics_store,
            buffered_stores,
            metrics_lag_secs: 24,
            active_surge_alert: None,
        }
    }

    /// The server behind the active tab.
    pub fn active_server(&self) -> Option<ServerRecord> {
        self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned()
    }
}
