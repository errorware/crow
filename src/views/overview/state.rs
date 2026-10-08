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
    /// The wand's plain-words explanations, by "server id/unit", for the
    /// session; which one is open; which are being asked.
    pub service_eli5: std::collections::HashMap<String, Result<ServiceEli5, String>>,
    pub service_eli5_open: Option<String>,
    pub service_eli5_loading: std::collections::HashSet<String>,
    /// (unit name, succeeded) — most recent failure banner.
    pub last_change_outcome: Option<(String, bool)>,
    pub socket_drawer_open: bool,
    pub socket_drawer_filter_this_socket: bool,
    /// Services page: live search text, status filter and page (0-based).
    pub service_query: String,
    pub service_filter: ServiceFilter,
    pub service_page: usize,
    /// Processes page: the same, for processes.
    pub process_query: String,
    pub process_filter: ProcessFilter,
    pub process_page: usize,
    /// Sockets page: Table vs Map subview, direction filter, hide loopback, selected connection
    pub sockets_subview: SocketsViewMode,
    pub map_filter: MapFilter,
    pub map_hide_loopback: bool,
    pub map_process_focus: Option<String>,
    pub selected_connection_id: Option<String>,
    /// Reverse DNS names of public peers, by address (ERR-100); `None`
    /// when the address has no name. Names are public, so one cache serves
    /// every server.
    pub peer_names: std::collections::HashMap<String, Option<String>>,
    pub peer_names_pending: bool,
    /// Overview's Updates & Security card.
    pub security: SecurityState,
    /// Height of the log panel under the Processes table (drag to resize).
    pub process_log_height: f32,
    /// The Sockets page's log drawer, dragged to size like Processes' log.
    pub socket_log_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketsViewMode {
    Table,
    Map,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapFilter {
    Both,
    Incoming,
    Outgoing,
}

impl MapFilter {
    pub const ALL: [MapFilter; 3] = [MapFilter::Both, MapFilter::Incoming, MapFilter::Outgoing];

    pub fn label(&self) -> &'static str {
        match self {
            MapFilter::Both => "ALL TRAFFIC",
            MapFilter::Incoming => "INCOMING",
            MapFilter::Outgoing => "OUTGOING",
        }
    }
}

/// Pending updates and CVEs for one server, refreshed at most every few
/// hours (the OSV lookup leaves the machine).
#[derive(Default)]
pub struct SecurityState {
    pub server_id: Option<String>,
    pub loading: bool,
    pub checked_at: Option<std::time::Instant>,
    pub checked_label: String,
    pub updates: Option<Result<super::updates::UpdatesReport, String>>,
    pub cves: Option<Result<crate::security::osv::CveReport, String>>,
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
            service_eli5: Default::default(),
            service_eli5_open: None,
            service_eli5_loading: Default::default(),
            last_change_outcome: None,
            socket_drawer_open: false,
            socket_drawer_filter_this_socket: false,
            service_query: String::new(),
            service_filter: ServiceFilter::Live,
            service_page: 0,
            process_query: String::new(),
            process_filter: ProcessFilter::Apps,
            process_page: 0,
            sockets_subview: SocketsViewMode::Table,
            map_filter: MapFilter::Both,
            map_hide_loopback: false,
            map_process_focus: None,
            selected_connection_id: None,
            peer_names: std::collections::HashMap::new(),
            peer_names_pending: false,
            security: SecurityState::default(),
            process_log_height: 240.0,
            socket_log_height: 250.0,
        }
    }
}

/// Rows shown per page on the Services and Processes pages.
pub const TABLE_PAGE_SIZE: usize = 50;

/// Which services the Services page lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceFilter {
    /// Running or in trouble: active, degraded and failed (the default —
    /// hides the long tail of stopped and not-found units).
    Live,
    Failed,
    /// Stopped, exited and not-found units.
    Inactive,
    All,
}

impl ServiceFilter {
    pub const ALL: [ServiceFilter; 4] = [ServiceFilter::Live, ServiceFilter::Failed, ServiceFilter::Inactive, ServiceFilter::All];

    pub fn label(self) -> &'static str {
        match self {
            ServiceFilter::Live => "LIVE",
            ServiceFilter::Failed => "FAILED",
            ServiceFilter::Inactive => "INACTIVE",
            ServiceFilter::All => "ALL",
        }
    }

    pub fn matches(self, status: &str) -> bool {
        match self {
            ServiceFilter::Live => matches!(status, "ACTIVE" | "DEGRADED" | "PENDING" | "FAILED"),
            ServiceFilter::Failed => status == "FAILED",
            ServiceFilter::Inactive => !matches!(status, "ACTIVE" | "DEGRADED" | "PENDING" | "FAILED"),
            ServiceFilter::All => true,
        }
    }
}

/// Failed first, then degraded, active, and everything else.
pub fn service_status_rank(status: &str) -> u8 {
    match status {
        "FAILED" => 0,
        "DEGRADED" | "PENDING" => 1,
        "ACTIVE" => 2,
        _ => 3,
    }
}

/// The services matching `filter` and `query` (case-insensitive, name or
/// description), sorted by status rank then name.
pub fn filter_services<'a>(services: &'a [ServiceUnit], filter: ServiceFilter, query: &str) -> Vec<&'a ServiceUnit> {
    let q = query.trim().to_lowercase();
    let mut out: Vec<&ServiceUnit> = services
        .iter()
        .filter(|s| filter.matches(&s.status))
        .filter(|s| q.is_empty() || s.name.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q))
        .collect();
    out.sort_by(|a, b| service_status_rank(&a.status).cmp(&service_status_rank(&b.status)).then_with(|| a.name.cmp(&b.name)));
    out
}

/// The page of `items` to show (clamped to the last page) and the page count.
pub fn page_of<T>(items: &[T], page: usize) -> (&[T], usize, usize) {
    let pages = items.len().div_ceil(TABLE_PAGE_SIZE).max(1);
    let page = page.min(pages - 1);
    let start = page * TABLE_PAGE_SIZE;
    (&items[start.min(items.len())..(start + TABLE_PAGE_SIZE).min(items.len())], page, pages)
}

/// Which processes the Processes page lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessFilter {
    /// Everything but kernel threads (the default: on a typical box more
    /// than half the list is kworkers and friends).
    Apps,
    Root,
    /// Using at least 1% CPU right now.
    Busy,
    Kernel,
    All,
}

impl ProcessFilter {
    pub const ALL: [ProcessFilter; 5] = [ProcessFilter::Apps, ProcessFilter::Root, ProcessFilter::Busy, ProcessFilter::Kernel, ProcessFilter::All];

    pub fn label(self) -> &'static str {
        match self {
            ProcessFilter::Apps => "APPS",
            ProcessFilter::Root => "ROOT",
            ProcessFilter::Busy => "CPU > 1%",
            ProcessFilter::Kernel => "KERNEL",
            ProcessFilter::All => "ALL",
        }
    }

    pub fn matches(self, p: &ProcessUnit) -> bool {
        match self {
            ProcessFilter::Apps => !p.is_kernel,
            ProcessFilter::Root => !p.is_kernel && p.user == "root",
            ProcessFilter::Busy => p.cpu >= 1.0,
            ProcessFilter::Kernel => p.is_kernel,
            ProcessFilter::All => true,
        }
    }
}

/// The processes matching `filter` and `query` (command or user, case-
/// insensitive, or a PID prefix), in their collected order (highest CPU first).
pub fn filter_processes<'a>(processes: &'a [ProcessUnit], filter: ProcessFilter, query: &str) -> Vec<&'a ProcessUnit> {
    let q = query.trim().to_lowercase();
    processes
        .iter()
        .filter(|p| filter.matches(p))
        .filter(|p| q.is_empty() || p.command.to_lowercase().contains(&q) || p.user.to_lowercase().contains(&q) || p.pid.to_string().starts_with(&q))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc(name: &str, status: &str, desc: &str) -> ServiceUnit {
        ServiceUnit {
            name: name.into(),
            status: status.into(),
            status_color_hex: 0,
            pid: String::new(),
            cpu: String::new(),
            mem: String::new(),
            rss: String::new(),
            uptime: String::new(),
            description: desc.into(),
            is_focused: false,
            show_confirm: false,
        }
    }

    #[test]
    fn live_filter_hides_stopped_units_and_search_matches_name_or_description() {
        let all = vec![
            svc("nginx.service", "ACTIVE", "A high performance web server"),
            svc("apport.service", "INACTIVE", "apport.service"),
            svc("clamav.service", "FAILED", "Clam AntiVirus"),
            svc("cron.service", "ACTIVE", "Regular background program processing daemon"),
        ];
        let names = |v: Vec<&ServiceUnit>| v.into_iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(filter_services(&all, ServiceFilter::Live, "")), ["clamav.service", "cron.service", "nginx.service"], "failed first, then by name");
        assert_eq!(names(filter_services(&all, ServiceFilter::Inactive, "")), ["apport.service"]);
        assert_eq!(names(filter_services(&all, ServiceFilter::All, "WEB")), ["nginx.service"], "description, case-insensitive");
        assert_eq!(names(filter_services(&all, ServiceFilter::Live, "cron")), ["cron.service"]);
        assert!(filter_services(&all, ServiceFilter::Failed, "nginx").is_empty());
    }

    #[test]
    fn apps_filter_hides_kernel_threads_and_search_matches_pid_user_or_command() {
        let p = |pid: u32, user: &str, cpu: f32, cmd: &str, kernel: bool| ProcessUnit {
            pid,
            user: user.into(),
            cpu,
            mem: 0.0,
            rss: String::new(),
            stat: "S".into(),
            time: String::new(),
            command: cmd.into(),
            is_kernel: kernel,
            is_focused: false,
            show_confirm: false,
        };
        let all = vec![p(812, "postgres", 4.0, "postgres", false), p(1, "root", 0.0, "systemd", false), p(134, "root", 0.0, "kworker/0:1", true)];
        let cmds = |v: Vec<&ProcessUnit>| v.into_iter().map(|p| p.command.clone()).collect::<Vec<_>>();
        assert_eq!(cmds(filter_processes(&all, ProcessFilter::Apps, "")), ["postgres", "systemd"], "keeps the collected order");
        assert_eq!(cmds(filter_processes(&all, ProcessFilter::Root, "")), ["systemd"]);
        assert_eq!(cmds(filter_processes(&all, ProcessFilter::Busy, "")), ["postgres"]);
        assert_eq!(cmds(filter_processes(&all, ProcessFilter::Kernel, "")), ["kworker/0:1"]);
        assert_eq!(cmds(filter_processes(&all, ProcessFilter::All, "81")), ["postgres"], "PID prefix");
        assert_eq!(cmds(filter_processes(&all, ProcessFilter::All, "ROOT")), ["systemd", "kworker/0:1"], "user");
    }

    #[test]
    fn pages_are_clamped() {
        let items: Vec<usize> = (0..120).collect();
        let (p, page, pages) = page_of(&items, 0);
        assert_eq!((p.len(), page, pages), (50, 0, 3));
        let (p, page, _) = page_of(&items, 9);
        assert_eq!((p.first(), p.len(), page), (Some(&100), 20, 2), "past the end shows the last page");
        let empty: Vec<usize> = Vec::new();
        assert_eq!(page_of(&empty, 3).2, 1);
    }
}

/// A service explained in plain words (the wand in the Services drawer).
#[derive(Clone, Debug, PartialEq)]
pub struct ServiceEli5 {
    /// How risky stopping or restarting it is, and why.
    pub severity: Option<(crate::ai::Severity, String)>,
    pub body: String,
    /// "Mistral AI · mistral-small-2603".
    pub via: String,
    /// When the backup answered: why the primary didn't.
    pub primary_failed: Option<String>,
}
