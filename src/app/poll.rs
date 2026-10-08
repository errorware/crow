use std::collections::HashMap;

use gpui_kit::{Context, Task};

use super::{CrowApp, Screen};
use crate::journal::{read_retention_for_server, JournalEntry, JournalQuery, JournalTelemetry};
use crate::journal::reader::read_journal_for_server;
use crate::metrics::{MetricSample, ServerMetrics, ServerTimeSeriesBuffer};
use crate::metrics::collector::{sample_server, CollectorStates};
use crate::vault::ServerRecord;
use crate::views::overview::{ProcessUnit, ServiceUnit, SocketUnit};
use crate::views::overview::collector::{
    collect_processes_for_server,
    collect_services_for_server,
    collect_sockets_for_server,
};

// ==========================================
// Background polling: metrics, overview tables, journal, fleet
// ==========================================

/// Which overview tables a poll tick should collect.
#[derive(Clone, Copy, Debug, Default)]
pub struct OverviewTables {
    pub services: bool,
    pub processes: bool,
    pub sockets: bool,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct BackgroundPollRequest {
    pub now_secs: u64,
    pub screen: Screen,
    pub active_view: String,
    pub tables: OverviewTables,
    pub active_server: Option<ServerRecord>,
    pub prev_active_metrics: Option<ServerMetrics>,
    pub should_poll_journal: bool,
    pub journal_query: Option<JournalQuery>,
    pub should_poll_retention: bool,
    pub fleet_servers: Vec<ServerRecord>,
    pub prev_fleet_metrics: HashMap<String, ServerMetrics>,
    /// This tick samples the whole fleet for history (ERR-84).
    pub history_tick: bool,
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
    pub history_tick: bool,
}

pub fn run_background_poll(
    req: BackgroundPollRequest,
    mut local_prev: CollectorStates,
) -> (BackgroundPollResult, CollectorStates) {
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
        history_tick: req.history_tick,
    };

    if let Some(ref active_srv) = req.active_server {
        // 1. Sample active server metrics off-thread
        let updated_head = sample_server(active_srv, req.prev_active_metrics.as_ref(), &mut local_prev);
        result.active_metrics = Some(updated_head);

        // 2. Overview subtabs (ps, ss, systemctl) executed on worker threadpool
        if req.tables.services {
            result.services_sample = Some(collect_services_for_server(active_srv));
        }
        if req.tables.processes {
            result.processes_sample = Some(collect_processes_for_server(active_srv));
        }
        if req.tables.sockets {
            result.sockets_sample = Some(collect_sockets_for_server(active_srv));
        }

        // 3. Journal query (only executed when actively viewing Logs or Overview)
        if req.should_poll_journal {
            if let Some(ref query) = req.journal_query {
                result.journal_entries = Some(read_journal_for_server(active_srv, query));
            }
            if req.should_poll_retention {
                let (_cfg, telemetry) = read_retention_for_server(active_srv);
                result.journal_telemetry = Some(telemetry);
            }
        }
    }

    // 4. Fleet servers (when on Screen::Fleet) executed off-thread
    if !req.fleet_servers.is_empty() {
        for s in &req.fleet_servers {
            // The active server was just sampled above; sampling it again in the
            // same tick would compute its rates over a near-zero interval.
            let updated = match (&result.active_server_id, &result.active_metrics) {
                (Some(active_id), Some(active)) if *active_id == s.id => active.clone(),
                _ => sample_server(s, req.prev_fleet_metrics.get(&s.id), &mut local_prev),
            };
            result.fleet_samples.push((s.id.clone(), s.name.clone(), updated));
        }
    }

    (result, local_prev)
}

/// Takes one metrics sample per server and seeds a time-series buffer with it,
/// both keyed by server id and by name.
pub fn seed_metrics(
    servers: &[ServerRecord],
) -> (HashMap<String, ServerMetrics>, HashMap<String, ServerTimeSeriesBuffer>) {
    let mut metrics_store = HashMap::new();
    let mut buffered_stores = HashMap::new();
    let mut local_prev = CollectorStates::default();
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    for s in servers {
        let m = sample_server(s, None, &mut local_prev);
        metrics_store.insert(s.id.clone(), m.clone());
        metrics_store.insert(s.name.clone(), m.clone());

        let mut buf = ServerTimeSeriesBuffer::default();
        buf.push_sample(MetricSample {
            timestamp_secs: now_secs,
            metrics: m,
            services: Vec::new(),
            processes: Vec::new(),
            sockets: Vec::new(),
        });
        buffered_stores.insert(s.id.clone(), buf.clone());
        buffered_stores.insert(s.name.clone(), buf);
    }
    (metrics_store, buffered_stores)
}

impl CrowApp {
    /// Every 2s: snapshot what to poll on the main thread, collect it on the
    /// background executor, then apply the result.
    pub(super) fn spawn_metrics_poll(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |entity, cx| {
                let mut local_prev = CollectorStates::default();
                let mut interval = std::time::Duration::from_secs(2);
                loop {
                    cx.background_executor().timer(interval).await;
                    if let Ok(secs) = entity.update(cx, |this, _cx| this.refresh_interval_secs()) {
                        interval = std::time::Duration::from_secs(secs);
                    }
                    // Auto-lock rides this tick; a locked Crow doesn't touch servers.
                    let req_res = entity.update(cx, |this, cx| {
                        this.check_auto_lock(cx);
                        (this.vault.status() != crate::vault::VaultStatus::Locked).then(|| this.prepare_poll_request())
                    });
                    let req = match req_res {
                        Ok(Some(r)) => r,
                        Ok(None) => continue,
                        Err(_) => break,
                    };
        
                    let (res, next_prev) = cx.background_executor().spawn(async move {
                        run_background_poll(req, local_prev)
                    }).await;
        
                    local_prev = next_prev;
        
                    if entity.update(cx, |this, cx| {
                        this.apply_poll_result(res);
                        this.maybe_check_certs(cx);
                        this.notify_tick(cx);
                        this.resolve_peer_names(cx);
                        // Overview on screen: keep the updates/CVE check current
                        // (a no-op while it's fresh; also covers startup).
                        if this.screen == super::Screen::Server && this.active_view == "overview" {
                            this.refresh_security(false, cx);
                        }
                        cx.notify();
                    }).is_err() {
                        break;
                    }
                }
            })
    }

    pub fn prepare_poll_request(&self) -> BackgroundPollRequest {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let active_srv = self.fleet.active_server();
        let prev_active_metrics = active_srv.as_ref().and_then(|srv| {
            self.fleet.buffered_stores.get(&srv.id).and_then(|b| b.head()).map(|h| h.metrics.clone()).or_else(|| self.fleet.metrics_store.get(&srv.id).cloned())
        });

        // The dashboard summarizes all three tables; each table page polls its own.
        let on_server = self.screen == Screen::Server;
        let tables = OverviewTables {
            services: on_server && (self.active_view == "overview" || self.active_view == "services"),
            processes: on_server && (self.active_view == "overview" || self.active_view == "processes"),
            sockets: on_server && (self.active_view == "overview" || self.active_view == "sockets"),
        };
        // The processes and sockets pages show the log tail / socket log drawer.
        let should_poll_journal = self.journal.live_tail
            && on_server
            && matches!(self.active_view.as_str(), "logs" | "processes" | "sockets");
        let should_poll_retention = self.screen == Screen::Server && self.active_view == "logs";

        let journal_query = if should_poll_journal {
            let mut q = self.journal.build_query();
            // In live-tail mode, fetch a lean window of 60 entries for rapid, non-laggy updates
            q.limit = 60;
            Some(q)
        } else {
            None
        };

        // Once a minute every server is sampled, whatever is on screen, and
        // the result goes into history (ERR-84).
        let history_tick = now_secs >= self.history.last_tick + crate::metrics::history::HISTORY_EVERY_SECS;
        let (fleet_servers, prev_fleet_metrics) = if self.screen == Screen::Fleet || history_tick {
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
            tables,
            active_server: active_srv,
            prev_active_metrics,
            should_poll_journal,
            journal_query,
            should_poll_retention,
            fleet_servers,
            prev_fleet_metrics,
            history_tick,
        }
    }

    /// Keeps each server's host key time current, in memory and in the
    /// vault, when a sample read it (ERR-81).
    fn note_host_key_mtime(&mut self, server_id: &str, m: &crate::metrics::ServerMetrics) {
        let Some(t) = m.host_key_oldest else { return };
        let Some(srv) = self.fleet.servers.iter_mut().find(|s| s.id == server_id) else { return };
        if srv.host_key_mtime == Some(t) {
            return;
        }
        srv.host_key_mtime = Some(t);
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_server_host_key_mtime(server_id, t);
        }
    }

    pub fn apply_poll_result(&mut self, res: BackgroundPollResult) {
        let (history_tick, now_secs) = (res.history_tick, res.now_secs);
        if let Some(ref srv_id) = res.active_server_id {
            if let Some(updated_head) = res.active_metrics {
                self.note_host_key_mtime(srv_id, &updated_head);
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
            self.journal.set_entries(entries);
        }
        if let Some(telemetry) = res.journal_telemetry {
            self.journal.telemetry = telemetry;
        }

        if !res.fleet_samples.is_empty() {
            for (id, name, m) in res.fleet_samples {
                self.note_host_key_mtime(&id, &m);
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
        if history_tick {
            self.record_history(now_secs as i64);
        }
    }

    /// Names the public peers on the connection map that haven't been looked
    /// up yet, through the active server's resolver, in the background
    /// (ERR-100). A batch at a time; each address is asked once per run.
    fn resolve_peer_names(&mut self, cx: &mut Context<Self>) {
        use crate::views::overview::collector::{bare_peer_addr, categorize_peer, reverse_names};
        use crate::views::overview::models::PeerCategory;
        if self.overview.peer_names_pending || self.screen != Screen::Server || self.active_view != "sockets" {
            return;
        }
        let mut wanted: Vec<String> = self
            .overview
            .sockets
            .iter()
            .map(|s| bare_peer_addr(&s.peer_addr).to_string())
            .filter(|a| categorize_peer(a) == PeerCategory::Public && !self.overview.peer_names.contains_key(a))
            .collect();
        wanted.sort();
        wanted.dedup();
        wanted.truncate(32);
        let Some(srv) = self.fleet.active_server() else { return };
        if wanted.is_empty() {
            return;
        }
        self.overview.peer_names_pending = true;
        cx.spawn(async move |entity, cx| {
            let names = cx.background_executor().spawn(async move { reverse_names(crate::host::host_for(&srv).as_ref(), &wanted) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.overview.peer_names_pending = false;
                this.overview.peer_names.extend(names);
                cx.notify();
            });
        })
        .detach();
    }

    /// Every few hours, reads every reachable server's TLS certificates in
    /// the background and turns what's about to expire into alerts (ERR-99).
    fn maybe_check_certs(&mut self, cx: &mut Context<Self>) {
        use crate::metrics::certs::{cert_changes, read_certs, CERT_CHECK_EVERY_SECS};
        use crate::views::fleet::state::FleetHealth;
        let now = chrono::Utc::now().timestamp();
        if now - self.cert_last_check < CERT_CHECK_EVERY_SECS || self.vault.status() == crate::vault::VaultStatus::Locked {
            return;
        }
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| self.fleet.health(s) == FleetHealth::Ok).cloned().collect();
        if servers.is_empty() {
            return;
        }
        self.cert_last_check = now;
        cx.spawn(async move |entity, cx| {
            let found = cx
                .background_executor()
                .spawn(async move { servers.iter().map(|s| (s.id.clone(), read_certs(crate::host::host_for(s).as_ref()))).collect::<Vec<_>>() })
                .await;
            let _ = entity.update(cx, |this, cx| {
                let now = chrono::Utc::now().timestamp();
                let db = this.vault.db();
                let Ok(db) = db.lock() else { return };
                for (server, certs) in found {
                    // Couldn't read: leave its alerts as they are.
                    let Some(certs) = certs else { continue };
                    let open: Vec<_> = db.list_alerts(i64::MAX).unwrap_or_default();
                    let _ = db.apply_alert_changes(&cert_changes(&open, &server, &certs, now), now);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Applies the alert rules to a round of history rows and stores the
    /// changes (ERR-85).
    fn evaluate_alerts(&mut self, rows: &[crate::metrics::history::HistoryRow]) {
        let now = chrono::Utc::now().timestamp();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let open: Vec<_> = db.list_alerts(i64::MAX).unwrap_or_default();
        let (changes, down_now) = crate::metrics::alerts::evaluate(&open, rows, &self.history_down, now);
        self.history_down = down_now;
        let _ = db.apply_alert_changes(&changes, now);
        // Host keys past the rotation policy (ERR-98).
        let keys: Vec<(String, Option<i64>)> = self.fleet.servers.iter().map(|s| (s.id.clone(), s.host_key_mtime)).collect();
        let policy = crate::app::archive::host_key_max_age_days(&self.config);
        let _ = db.apply_alert_changes(&crate::metrics::alerts::key_age_changes(&open, &keys, policy, now), now);
    }

    /// One history row per server from its newest sample, the watch session
    /// extended, and old rows pruned hourly (ERR-84).
    fn record_history(&mut self, now: i64) {
        use crate::metrics::history::{row_from, HISTORY_KEEP_SECS};
        use crate::views::fleet::state::FleetHealth;
        self.history.last_tick = now as u64;
        let rows: Vec<_> = self
            .fleet
            .servers
            .iter()
            .filter_map(|s| {
                let latest = self.fleet.buffered_stores.get(&s.id).and_then(|b| b.head()).map(|h| &h.metrics).or_else(|| self.fleet.metrics_store.get(&s.id));
                let down = match self.fleet.health(s) {
                    FleetHealth::Checking => return None,
                    FleetHealth::Ok => None,
                    FleetHealth::Down { label, detail } => Some(if detail.is_empty() { label.to_lowercase() } else { format!("{} · {detail}", label.to_lowercase()) }),
                };
                Some(row_from(&s.id, now, latest, down))
            })
            .collect();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let _ = db.insert_history_rows(&rows);
        let _ = db.touch_watch_session(self.history.watch_started, now);
        if now - self.history.last_prune >= 3600 {
            let _ = db.prune_history(now - HISTORY_KEEP_SECS);
            self.history.last_prune = now;
        }
        drop(db);
        self.history.rows_written += rows.len();
        self.evaluate_alerts(&rows);
    }

    #[allow(dead_code)]
    pub fn poll_metrics(&mut self, local_prev: &mut CollectorStates) {
        let req = self.prepare_poll_request();
        let (res, next_prev) = run_background_poll(req, local_prev.clone());
        *local_prev = next_prev;
        self.apply_poll_result(res);
    }
}
