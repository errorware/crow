use std::collections::HashMap;

use super::{CrowApp, Screen};
use crate::vault::ServerRecord;
use crate::journal::JournalEntry;
use crate::journal::JournalQuery;
use crate::metrics::MetricSample;
use crate::metrics::ServerMetrics;
use crate::journal::JournalTelemetry;
use crate::views::overview::SocketUnit;
use crate::views::overview::ProcessUnit;
use crate::views::overview::ServiceUnit;
use crate::metrics::ServerTimeSeriesBuffer;
use crate::metrics::collector::sample_server;
use crate::journal::read_retention_for_server;
use crate::journal::reader::read_journal_for_server;
use crate::metrics::collector::CollectorPreviousState;
use crate::views::overview::collector::collect_sockets_for_server;
use crate::views::overview::collector::collect_services_for_server;
use crate::views::overview::collector::collect_processes_for_server;

// ==========================================
// Background polling: metrics, overview tables, journal, fleet
// ==========================================

#[derive(Clone)]
#[allow(dead_code)]
pub struct BackgroundPollRequest {
    pub now_secs: u64,
    pub screen: Screen,
    pub active_view: String,
    pub active_services_tab: String,
    pub active_server: Option<ServerRecord>,
    pub prev_active_metrics: Option<ServerMetrics>,
    pub should_poll_overview_subtab: bool,
    pub should_poll_journal: bool,
    pub journal_query: Option<JournalQuery>,
    pub should_poll_retention: bool,
    pub fleet_servers: Vec<ServerRecord>,
    pub prev_fleet_metrics: HashMap<String, ServerMetrics>,
}

pub struct BackgroundPollResult {
    pub now_secs: u64,
    pub active_server_id: Option<String>,
    pub active_metrics: Option<ServerMetrics>,
    pub services_sample: Option<Vec<ServiceUnit>>,
    pub processes_sample: Option<Vec<ProcessUnit>>,
    pub sockets_sample: Option<Vec<SocketUnit>>,
    pub journal_entries: Option<Vec<JournalEntry>>,
    pub journal_telemetry: Option<JournalTelemetry>,
    pub fleet_samples: Vec<(String, String, ServerMetrics)>,
}

pub fn run_background_poll(
    req: BackgroundPollRequest,
    mut local_prev: CollectorPreviousState,
) -> (BackgroundPollResult, CollectorPreviousState) {
    let mut result = BackgroundPollResult {
        now_secs: req.now_secs,
        active_server_id: req.active_server.as_ref().map(|s| s.id.clone()),
        active_metrics: None,
        services_sample: None,
        processes_sample: None,
        sockets_sample: None,
        journal_entries: None,
        journal_telemetry: None,
        fleet_samples: Vec::new(),
    };

    if let Some(ref active_srv) = req.active_server {
        // 1. Sample active server metrics off-thread
        let updated_head = sample_server(active_srv, req.prev_active_metrics.as_ref(), &mut local_prev);
        result.active_metrics = Some(updated_head);

        // 2. Overview subtabs (ps, ss, systemctl) executed on worker threadpool
        if req.should_poll_overview_subtab {
            match req.active_services_tab.as_str() {
                "processes" => {
                    result.processes_sample = Some(collect_processes_for_server(active_srv));
                }
                "sockets" => {
                    result.sockets_sample = Some(collect_sockets_for_server(active_srv));
                }
                _ => {
                    result.services_sample = Some(collect_services_for_server(active_srv));
                }
            }
        }

        // 3. Journal query (only executed when actively viewing Logs or Overview)
        if req.should_poll_journal {
            if let Some(ref query) = req.journal_query {
                result.journal_entries = Some(read_journal_for_server(active_srv, query));
            }
            if req.should_poll_retention {
                let (_cfg, telemetry) = read_retention_for_server(&active_srv.host);
                result.journal_telemetry = Some(telemetry);
            }
        }
    }

    // 4. Fleet servers (when on Screen::Fleet) executed off-thread
    if !req.fleet_servers.is_empty() {
        for s in &req.fleet_servers {
            let prev = req.prev_fleet_metrics.get(&s.id);
            let updated = sample_server(s, prev, &mut local_prev);
            result.fleet_samples.push((s.id.clone(), s.name.clone(), updated));
        }
    }

    (result, local_prev)
}

impl CrowApp {
    pub fn prepare_poll_request(&self) -> BackgroundPollRequest {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let active_srv = self.fleet.active_server();
        let prev_active_metrics = active_srv.as_ref().and_then(|srv| {
            self.fleet.buffered_stores.get(&srv.id).and_then(|b| b.head()).map(|h| h.metrics.clone()).or_else(|| self.fleet.metrics_store.get(&srv.id).cloned())
        });

        let should_poll_overview = self.screen == Screen::Server && self.active_view == "overview";
        let should_poll_journal = self.journal.live_tail
            && self.screen == Screen::Server
            && (self.active_view == "logs" || self.active_view == "overview");
        let should_poll_retention = self.screen == Screen::Server && self.active_view == "logs";

        let journal_query = if should_poll_journal {
            let mut q = self.journal.build_query();
            // In live-tail mode, fetch a lean window of 60 entries for rapid, non-laggy updates
            q.limit = 60;
            Some(q)
        } else {
            None
        };

        let (fleet_servers, prev_fleet_metrics) = if self.screen == Screen::Fleet {
            let mut prev_map = HashMap::new();
            for s in &self.fleet.servers {
                if let Some(m) = self.fleet.buffered_stores.get(&s.id).and_then(|b| b.head()).map(|h| h.metrics.clone()).or_else(|| self.fleet.metrics_store.get(&s.id).cloned()) {
                    prev_map.insert(s.id.clone(), m);
                }
            }
            (self.fleet.servers.clone(), prev_map)
        } else {
            (Vec::new(), HashMap::new())
        };

        BackgroundPollRequest {
            now_secs,
            screen: self.screen,
            active_view: self.active_view.clone(),
            active_services_tab: self.overview.active_tab.clone(),
            active_server: active_srv,
            prev_active_metrics,
            should_poll_overview_subtab: should_poll_overview,
            should_poll_journal,
            journal_query,
            should_poll_retention,
            fleet_servers,
            prev_fleet_metrics,
        }
    }

    pub fn apply_poll_result(&mut self, res: BackgroundPollResult) {
        if let Some(ref srv_id) = res.active_server_id {
            if let Some(updated_head) = res.active_metrics {
                // Retrieve current services/processes/sockets or new sample
                let services_sample = res.services_sample.unwrap_or_else(|| self.overview.services.clone());
                let processes_sample = res.processes_sample.unwrap_or_else(|| self.overview.processes.clone());
                let sockets_sample = res.sockets_sample.unwrap_or_else(|| self.overview.sockets.clone());

                // Ingest sample into ring buffer at T_head
                let buf = self.fleet.buffered_stores.entry(srv_id.clone()).or_insert_with(ServerTimeSeriesBuffer::default);
                buf.push_sample(MetricSample {
                    timestamp_secs: res.now_secs,
                    metrics: updated_head.clone(),
                    services: services_sample,
                    processes: processes_sample,
                    sockets: sockets_sample,
                });

                // Foreknowledge: scan lookahead window (T_playback, T_head] for upcoming surges
                self.fleet.active_surge_alert = buf.detect_upcoming_surge(self.fleet.metrics_lag_secs);

                // Playback: query lagged sample from local time-series ring buffer (lag_secs behind)
                if let Some(lagged) = buf.query_lagged(self.fleet.metrics_lag_secs) {
                    self.fleet.metrics_store.insert(srv_id.clone(), lagged.metrics.clone());
                    if let Some(active_srv) = self.fleet.servers.iter().find(|s| s.id == *srv_id) {
                        self.fleet.metrics_store.insert(active_srv.name.clone(), lagged.metrics.clone());
                    }

                    // Preserve row focus across replacements
                    if !lagged.services.is_empty() {
                        let focused_name = self.overview.services.iter().find(|s| s.is_focused).map(|s| s.name.clone());
                        self.overview.services = lagged.services.clone();
                        if let Some(name) = focused_name {
                            for svc in &mut self.overview.services {
                                svc.is_focused = svc.name == name;
                            }
                        }
                    }
                    if !lagged.processes.is_empty() {
                        let focused_pid = self.overview.processes.iter().find(|p| p.is_focused).map(|p| p.pid);
                        self.overview.processes = lagged.processes.clone();
                        if let Some(pid) = focused_pid {
                            for proc in &mut self.overview.processes {
                                proc.is_focused = proc.pid == pid;
                            }
                        }
                    }
                    if !lagged.sockets.is_empty() {
                        let focused_idx = self.overview.sockets.iter().position(|s| s.is_focused);
                        self.overview.sockets = lagged.sockets.clone();
                        if let Some(idx) = focused_idx {
                            if let Some(sock) = self.overview.sockets.get_mut(idx) {
                                sock.is_focused = true;
                            }
                        }
                    }
                } else {
                    self.fleet.metrics_store.insert(srv_id.clone(), updated_head.clone());
                    if let Some(active_srv) = self.fleet.servers.iter().find(|s| s.id == *srv_id) {
                        self.fleet.metrics_store.insert(active_srv.name.clone(), updated_head);
                    }
                }

                let buf_clone = buf.clone();
                if let Some(active_srv) = self.fleet.servers.iter().find(|s| s.id == *srv_id) {
                    self.fleet.buffered_stores.insert(active_srv.name.clone(), buf_clone);
                }
            }
        }

        if let Some(entries) = res.journal_entries {
            self.journal.entries = entries;
        }
        if let Some(telemetry) = res.journal_telemetry {
            self.journal.telemetry = telemetry;
        }

        if !res.fleet_samples.is_empty() {
            for (id, name, m) in res.fleet_samples {
                let buf = self.fleet.buffered_stores.entry(id.clone()).or_insert_with(ServerTimeSeriesBuffer::default);
                buf.push_sample(MetricSample {
                    timestamp_secs: res.now_secs,
                    metrics: m.clone(),
                    services: Vec::new(),
                    processes: Vec::new(),
                    sockets: Vec::new(),
                });
                if let Some(lagged) = buf.query_lagged(self.fleet.metrics_lag_secs) {
                    self.fleet.metrics_store.insert(id.clone(), lagged.metrics.clone());
                    self.fleet.metrics_store.insert(name.clone(), lagged.metrics.clone());
                } else {
                    self.fleet.metrics_store.insert(id.clone(), m.clone());
                    self.fleet.metrics_store.insert(name.clone(), m);
                }
                let buf_clone = buf.clone();
                self.fleet.buffered_stores.insert(name, buf_clone);
            }
        }
    }

    #[allow(dead_code)]
    pub fn poll_metrics(&mut self, local_prev: &mut CollectorPreviousState) {
        let req = self.prepare_poll_request();
        let (res, next_prev) = run_background_poll(req, local_prev.clone());
        *local_prev = next_prev;
        self.apply_poll_result(res);
    }
}
