use std::collections::HashSet;

use super::{BlastRadiusInfo, ProcessUnit, ServiceUnit, SocketUnit};

/// Server overview state: the services/processes/sockets tables, their
/// grouping, the service manager panel and the socket log drawer.
pub struct OverviewState {
    /// "services", "processes" or "sockets".
    pub active_tab: String,
    pub services: Vec<ServiceUnit>,
    pub processes: Vec<ProcessUnit>,
    pub sockets: Vec<SocketUnit>,
    pub group_services: bool,
    pub group_processes: bool,
    pub collapsed_service_groups: HashSet<String>,
    pub collapsed_process_groups: HashSet<String>,
    pub service_panel_pending_action: Option<String>,
    pub blast_radius: Option<BlastRadiusInfo>,
    /// (unit name, succeeded) — most recent failure banner.
    pub last_change_outcome: Option<(String, bool)>,
    pub socket_drawer_open: bool,
    pub socket_drawer_filter_this_socket: bool,
}

impl OverviewState {
    pub fn new(services: Vec<ServiceUnit>, processes: Vec<ProcessUnit>, sockets: Vec<SocketUnit>) -> Self {
        Self {
            active_tab: "services".to_string(),
            services,
            processes,
            sockets,
            group_services: false,
            group_processes: false,
            collapsed_service_groups: HashSet::new(),
            collapsed_process_groups: HashSet::new(),
            service_panel_pending_action: None,
            blast_radius: None,
            last_change_outcome: None,
            socket_drawer_open: false,
            socket_drawer_filter_this_socket: false,
        }
    }
}
