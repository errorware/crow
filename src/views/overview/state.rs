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
    /// Services page: live search text, status filter and page (0-based).
    pub service_query: String,
    pub service_filter: ServiceFilter,
    pub service_page: usize,
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
            service_query: String::new(),
            service_filter: ServiceFilter::Live,
            service_page: 0,
        }
    }
}

/// Services shown per page.
pub const SERVICES_PAGE_SIZE: usize = 50;

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
    let pages = items.len().div_ceil(SERVICES_PAGE_SIZE).max(1);
    let page = page.min(pages - 1);
    let start = page * SERVICES_PAGE_SIZE;
    (&items[start.min(items.len())..(start + SERVICES_PAGE_SIZE).min(items.len())], page, pages)
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
