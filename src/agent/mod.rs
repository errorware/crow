//! Headless background agent (`crow --agent` and `crow --agent-once`) (ERR-207).
//!
//! Keeps watching enrolled servers when the Crow desktop window is closed:
//! - Coordinates with the GUI via `agent_state.gui_last_tick` in SQLite WAL so
//!   enrolled servers are never double-polled while the GUI is open.
//! - Samples enrolled servers over SSH (`ServerRecord::agent_installed == true`),
//!   captures top offending processes when CPU/memory thresholds are crossed,
//!   writes `HistoryRow` samples, extends `watch_sessions`, and evaluates both
//!   standard and performance bottleneck alerts.

pub mod service;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::config::CrowConfigManager;
use crate::host::{connection_state, update_directory, ConnectionState};
use crate::metrics::bottlenecks::{
    evaluate_server_bottlenecks, BottleneckConfig, BottleneckContext, CPU_WARN_PCT, MEM_WARN_PCT,
};
use crate::metrics::collector::{sample_server, CollectorStates};
use crate::metrics::history::{row_from, HistoryRow, HISTORY_KEEP_SECS};
use crate::metrics::ServerMetrics;
use crate::vault::VaultDb;
use crate::views::overview::collector::collect_processes_for_server;

pub const DEFAULT_AGENT_INTERVAL_SECS: u64 = 60;
pub const GUI_ACTIVE_YIELD_WINDOW_SECS: i64 = 55;
pub const FLAG_INTERVAL_SECS: &str = "agent.interval_secs";

/// Returns `true` when the Crow GUI has polled recently enough that the
/// background agent should yield and avoid duplicate SSH probes.
pub fn should_yield_to_gui(gui_last_tick: i64, now: i64) -> bool {
    gui_last_tick > 0 && now.saturating_sub(gui_last_tick) < GUI_ACTIVE_YIELD_WINDOW_SECS
}

/// Configured background agent poll interval in seconds (`30`, `60`, `120`, `300`).
pub fn configured_interval_secs(cfg: &CrowConfigManager) -> u64 {
    cfg.saved_int(FLAG_INTERVAL_SECS)
        .and_then(|v| u64::try_from(v).ok())
        .map(|s| s.clamp(15, 3600))
        .unwrap_or(DEFAULT_AGENT_INTERVAL_SECS)
}

pub fn configured_interval_secs_from_db(db: &VaultDb) -> u64 {
    db.flag(FLAG_INTERVAL_SECS)
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|s| s.clamp(15, 3600))
        .unwrap_or(DEFAULT_AGENT_INTERVAL_SECS)
}

pub struct AgentRuntime {
    pub pid: u32,
    pub started_at: i64,
    pub last_prune: i64,
    pub collector_states: CollectorStates,
    pub prev_metrics: HashMap<String, ServerMetrics>,
    pub down_before: HashSet<String>,
}

impl AgentRuntime {
    pub fn new(now: i64) -> Self {
        Self {
            pid: std::process::id(),
            started_at: now,
            last_prune: 0,
            collector_states: CollectorStates::default(),
            prev_metrics: HashMap::new(),
            down_before: HashSet::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentTickOutcome {
    pub yielded_to_gui: bool,
    pub polled_servers: usize,
    pub alerts_opened: usize,
    pub error: Option<String>,
}

/// Executes a single background poll pass against `db`.
/// When `force` is `true` (e.g. `crow --agent-once` or "Sample Now" in Settings),
/// polls enrolled servers even if the GUI is currently open.
pub fn run_agent_tick(
    db: &VaultDb,
    rt: &mut AgentRuntime,
    now: i64,
    force: bool,
) -> AgentTickOutcome {
    if db.is_initialized().unwrap_or(false) {
        let msg = "Vault master password is enabled; background agent requires OS-keyring mode".to_string();
        let _ = db.record_agent_heartbeat(rt.pid, rt.started_at, now, 0, Some(&msg));
        return AgentTickOutcome {
            yielded_to_gui: false,
            polled_servers: 0,
            alerts_opened: 0,
            error: Some(msg),
        };
    }

    if !force {
        if let Ok(state) = db.get_agent_state() {
            if should_yield_to_gui(state.gui_last_tick, now) {
                let _ = db.record_agent_heartbeat(
                    rt.pid,
                    rt.started_at,
                    now,
                    state.last_polled_servers,
                    None,
                );
                return AgentTickOutcome {
                    yielded_to_gui: true,
                    polled_servers: 0,
                    alerts_opened: 0,
                    error: None,
                };
            }
        }
    }

    let all_servers = match db.list_servers() {
        Ok(s) => s,
        Err(e) => {
            let msg = e.to_string();
            let _ = db.record_agent_heartbeat(rt.pid, rt.started_at, now, 0, Some(&msg));
            return AgentTickOutcome {
                yielded_to_gui: false,
                polled_servers: 0,
                alerts_opened: 0,
                error: Some(msg),
            };
        }
    };
    let keys = db.list_ssh_keys().unwrap_or_default();
    update_directory(&all_servers, &keys);

    let enrolled: Vec<_> = all_servers.iter().filter(|s| s.agent_installed).cloned().collect();
    if enrolled.is_empty() {
        let _ = db.record_agent_heartbeat(rt.pid, rt.started_at, now, 0, None);
        return AgentTickOutcome {
            yielded_to_gui: false,
            polled_servers: 0,
            alerts_opened: 0,
            error: None,
        };
    }

    let b_cfg = BottleneckConfig::from_db(db);

    let mut rows: Vec<HistoryRow> = Vec::with_capacity(enrolled.len());
    let mut contexts: HashMap<String, BottleneckContext> = HashMap::new();

    for srv in &enrolled {
        let prev = rt.prev_metrics.get(&srv.id);
        let mut m = sample_server(srv, prev, &mut rt.collector_states);
        // On the very first sample for a server, `/proc/stat` has no delta yet so
        // `cpu_known` is false. Wait briefly and take a second reading so even
        // `--agent-once` records a real CPU rate.
        if m.reachable && !m.cpu_known {
            std::thread::sleep(Duration::from_millis(350));
            m = sample_server(srv, Some(&m), &mut rt.collector_states);
        }

        if let Some(mtime) = m.host_key_oldest {
            if srv.host_key_mtime != Some(mtime) {
                let _ = db.set_server_host_key_mtime(&srv.id, mtime);
            }
        }

        let down = if m.reachable {
            None
        } else {
            match connection_state(&srv.id) {
                Some(ConnectionState::Unreachable(why)) => Some(format!("unreachable · {why}")),
                Some(ConnectionState::AuthFailed(why)) => Some(format!("auth failed · {why}")),
                Some(ConnectionState::HostKeyRejected(why)) => Some(format!("host key changed · {why}")),
                _ => Some("no answer to the metrics probe".into()),
            }
        };

        let procs = if m.reachable && (m.cpu_pct >= CPU_WARN_PCT - 5.0 || m.mem_pct >= MEM_WARN_PCT - 5.0) {
            collect_processes_for_server(srv)
        } else {
            Vec::new()
        };
        let ctx = BottleneckContext::from_metrics_and_processes(
            m.vcpu_count,
            &m.disk_mount,
            &procs,
        );
        contexts.insert(srv.id.clone(), ctx);

        rows.push(row_from(&srv.id, now, Some(&m), down));
        rt.prev_metrics.insert(srv.id.clone(), m);
    }

    let _ = db.insert_history_rows(&rows);
    let _ = db.touch_watch_session(rt.started_at, now);
    if now - rt.last_prune >= 3600 {
        let _ = db.prune_history(now - HISTORY_KEEP_SECS);
        rt.last_prune = now;
    }

    let open = db.list_alerts(i64::MAX).unwrap_or_default();
    let (std_changes, down_now) = crate::metrics::alerts::evaluate(&open, &rows, &rt.down_before, now);
    rt.down_before = down_now;
    let mut alerts_opened = std_changes.opened.len();
    let _ = db.apply_alert_changes(&std_changes, now);

    let open_after_std = db.list_alerts(i64::MAX).unwrap_or_default();
    for srv in &enrolled {
        let hist = db.list_history(&srv.id, now - 6 * 3600).unwrap_or_default();
        let ctx = contexts.get(&srv.id).cloned().unwrap_or_default();
        let b_changes = evaluate_server_bottlenecks(&open_after_std, &srv.id, &hist, &ctx, &b_cfg, now);
        alerts_opened += b_changes.opened.len();
        let _ = db.apply_alert_changes(&b_changes, now);
    }

    let _ = db.record_agent_heartbeat(rt.pid, rt.started_at, now, enrolled.len(), None);

    AgentTickOutcome {
        yielded_to_gui: false,
        polled_servers: enrolled.len(),
        alerts_opened,
        error: None,
    }
}

/// Entrypoint for `crow --agent` and `crow --agent-once`.
pub fn agent_main(once: bool) -> i32 {
    crate::vault::set_current_actor(Some("crow-agent".to_string()));
    let now = chrono::Utc::now().timestamp();
    let mut rt = AgentRuntime::new(now);

    if once {
        let db = match VaultDb::open_or_create(None) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("crow --agent-once: failed to open vault db: {e}");
                return 1;
            }
        };
        let outcome = run_agent_tick(&db, &mut rt, now, true);
        if let Some(err) = outcome.error {
            eprintln!("crow --agent-once: {err}");
            return 1;
        }
        return 0;
    }

    loop {
        let tick_now = chrono::Utc::now().timestamp();
        let mut sleep_secs = DEFAULT_AGENT_INTERVAL_SECS;
        if let Ok(db) = VaultDb::open_or_create(None) {
            sleep_secs = configured_interval_secs_from_db(&db);
            let _ = run_agent_tick(&db, &mut rt, tick_now, false);
        }
        std::thread::sleep(Duration::from_secs(sleep_secs));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yields_when_gui_heartbeat_is_recent() {
        assert!(should_yield_to_gui(1000, 1020));
        assert!(!should_yield_to_gui(1000, 1060));
        assert!(!should_yield_to_gui(0, 1000));
    }

    #[test]
    fn agent_tick_yields_when_gui_active_unless_forced() {
        let db = VaultDb::open_in_memory().unwrap();
        db.touch_gui_heartbeat(1000).unwrap();

        let mut rt = AgentRuntime::new(1000);
        let out = run_agent_tick(&db, &mut rt, 1010, false);
        assert!(out.yielded_to_gui);
        assert_eq!(out.polled_servers, 0);

        let forced = run_agent_tick(&db, &mut rt, 1010, true);
        assert!(!forced.yielded_to_gui);
        let state = db.get_agent_state().unwrap();
        assert_eq!(state.agent_last_tick, Some(1010));
    }
}
