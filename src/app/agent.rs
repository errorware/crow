//! App state & actions for `crow --agent` and performance bottleneck rules (ERR-208, ERR-209).

use std::collections::HashMap;

use gpui_kit::*;

use super::CrowApp;
use crate::agent::service::{self, ServiceStatus};
use crate::agent::{
    configured_interval_secs_from_db, run_agent_tick, AgentRuntime, DEFAULT_AGENT_INTERVAL_SECS,
    FLAG_INTERVAL_SECS,
};
use crate::metrics::alerts::Alert;
use crate::metrics::bottlenecks::{is_bottleneck_kind, BottleneckConfig};
use crate::vault::{AgentHeartbeatState, ChangeRecord, VaultDb};

#[derive(Clone, Debug)]
pub struct AgentState {
    pub service: ServiceStatus,
    pub heartbeat: AgentHeartbeatState,
    pub interval_secs: u64,
    pub bottlenecks: BottleneckConfig,
    pub latest_sample_by_server: HashMap<String, i64>,
    pub open_bottlenecks: Vec<Alert>,
    pub busy: bool,
    /// `(is_ok, message)` banner shown on Settings → Agent.
    pub notice: Option<(bool, String)>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            service: ServiceStatus::default(),
            heartbeat: AgentHeartbeatState::default(),
            interval_secs: DEFAULT_AGENT_INTERVAL_SECS,
            bottlenecks: BottleneckConfig::default(),
            latest_sample_by_server: HashMap::new(),
            open_bottlenecks: Vec::new(),
            busy: false,
            notice: None,
        }
    }
}

impl AgentState {
    #[allow(dead_code)]
    pub fn load(db: &VaultDb) -> Self {
        let heartbeat = db.get_agent_state().unwrap_or_default();
        let interval_secs = configured_interval_secs_from_db(db);
        let bottlenecks = BottleneckConfig::from_db(db);
        let latest_sample_by_server = db.latest_history_ts_by_server().unwrap_or_default();
        let open_bottlenecks = db
            .list_alerts(i64::MAX)
            .unwrap_or_default()
            .into_iter()
            .filter(|a| a.resolved_at.is_none() && is_bottleneck_kind(&a.kind))
            .collect();

        Self {
            service: ServiceStatus::default(),
            heartbeat,
            interval_secs,
            bottlenecks,
            latest_sample_by_server,
            open_bottlenecks,
            busy: false,
            notice: None,
        }
    }

    pub fn bottlenecks_for_server(&self, server_id: &str) -> Vec<&Alert> {
        self.open_bottlenecks
            .iter()
            .filter(|a| a.server_id == server_id && a.resolved_at.is_none())
            .collect()
    }
}

impl CrowApp {
    /// Refreshes the database-backed agent state and queries the OS user-service
    /// status on the background executor.
    pub fn refresh_agent_state(&mut self, cx: &mut Context<Self>) {
        if let Ok(db) = self.vault.db().lock() {
            self.agent.heartbeat = db.get_agent_state().unwrap_or_default();
            self.agent.interval_secs = configured_interval_secs_from_db(&db);
            self.agent.bottlenecks = BottleneckConfig::from_db(&db);
            self.agent.latest_sample_by_server = db.latest_history_ts_by_server().unwrap_or_default();
            self.agent.open_bottlenecks = db
                .list_alerts(i64::MAX)
                .unwrap_or_default()
                .into_iter()
                .filter(|a| a.resolved_at.is_none() && is_bottleneck_kind(&a.kind))
                .collect();
        }
        cx.spawn(async move |entity, cx| {
            let status = cx.background_executor().spawn(async move { service::query_status() }).await;
            let _ = entity.update(cx, |this, cx| {
                this.agent.service = status;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn install_and_start_agent_service(&mut self, cx: &mut Context<Self>) {
        if self.agent.busy {
            return;
        }
        self.agent.busy = true;
        self.agent.notice = None;
        cx.notify();

        cx.spawn(async move |entity, cx| {
            let res = cx
                .background_executor()
                .spawn(async move { service::install_and_start() })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.agent.busy = false;
                match res {
                    Ok(st) => {
                        this.agent.service = st;
                        this.agent.notice = Some((true, t!("settings-agent-service-installed-and-started").to_string()));
                    }
                    Err(e) => {
                        this.agent.notice = Some((false, e));
                    }
                }
                this.refresh_agent_state(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn stop_and_uninstall_agent_service(&mut self, cx: &mut Context<Self>) {
        if self.agent.busy {
            return;
        }
        self.agent.busy = true;
        self.agent.notice = None;
        cx.notify();

        cx.spawn(async move |entity, cx| {
            let res = cx
                .background_executor()
                .spawn(async move { service::stop_and_uninstall() })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.agent.busy = false;
                match res {
                    Ok(st) => {
                        this.agent.service = st;
                        this.agent.notice = Some((true, t!("settings-agent-service-stopped-and-removed").to_string()));
                    }
                    Err(e) => {
                        this.agent.notice = Some((false, e));
                    }
                }
                this.refresh_agent_state(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn run_agent_sample_now(&mut self, cx: &mut Context<Self>) {
        if self.agent.busy {
            return;
        }
        self.agent.busy = true;
        self.agent.notice = None;
        cx.notify();

        cx.spawn(async move |entity, cx| {
            let outcome = cx
                .background_executor()
                .spawn(async move {
                    let now = chrono::Utc::now().timestamp();
                    let db = VaultDb::open_or_create(None).map_err(|e| e.to_string())?;
                    let mut rt = AgentRuntime::new(now);
                    Ok::<_, String>(run_agent_tick(&db, &mut rt, now, true))
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.agent.busy = false;
                match outcome {
                    Ok(out) => {
                        if let Some(err) = out.error {
                            this.agent.notice = Some((false, err));
                        } else {
                            this.agent.notice = Some((
                                true,
                                t!(
                                    "settings-agent-sampled-enrolled-servers",
                                    count = out.polled_servers,
                                    alerts = out.alerts_opened
                                ),
                            ));
                        }
                    }
                    Err(e) => {
                        this.agent.notice = Some((false, e));
                    }
                }
                this.history_view = None;
                this.refresh_agent_state(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn set_agent_interval_secs(&mut self, secs: u64, cx: &mut Context<Self>) {
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(FLAG_INTERVAL_SECS, &secs.to_string());
        }
        self.agent.interval_secs = secs;
        cx.notify();
    }

    pub fn toggle_agent_bottleneck_rule(&mut self, flag_key: &str, enabled: bool, cx: &mut Context<Self>) {
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(flag_key, if enabled { "true" } else { "false" });
            self.agent.bottlenecks = BottleneckConfig::from_db(&db);
        }
        cx.notify();
    }

    pub fn set_server_agent_enrolled(&mut self, server_id: &str, enrolled: bool, cx: &mut Context<Self>) {
        let srv_opt = self.fleet.servers.iter().find(|s| s.id == server_id).cloned();
        if !self.allowed(crate::team::Permission::Vault, srv_opt.as_ref(), cx) {
            return;
        }
        let srv_name = srv_opt
            .map(|s| s.name)
            .unwrap_or_else(|| server_id.to_string());

        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_server_agent_enrolled(server_id, enrolled);
            let now = chrono::Utc::now().to_rfc3339();
            let _ = db.insert_change_record(&ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: server_id.to_string(),
                server_name: srv_name,
                action_kind: "server.agent_enroll".into(),
                target: if enrolled { "enrolled".into() } else { "unenrolled".into() },
                before_state: format!("agent_enrolled={}", !enrolled),
                after_state: Some(format!("agent_enrolled={enrolled}")),
                blast_radius: None,
                outcome: "success".into(),
                started_at: now.clone(),
                completed_at: Some(now),
            });
        }
        for srv in &mut self.fleet.servers {
            if srv.id == server_id {
                srv.agent_installed = enrolled;
            }
        }
        self.refresh_agent_state(cx);
        cx.notify();
    }

    pub fn set_all_servers_agent_enrolled(&mut self, enrolled: bool, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_all_servers_agent_enrolled(enrolled);
        }
        for srv in &mut self.fleet.servers {
            srv.agent_installed = enrolled;
        }
        self.refresh_agent_state(cx);
        cx.notify();
    }
}
