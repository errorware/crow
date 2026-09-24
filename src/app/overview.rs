use gpui_kit::*;
use gpui_kit::component::input::{InputEvent, InputState};

use super::CrowApp;
use crate::views::overview::state::{ProcessFilter, ServiceFilter};
use crate::components::sidebar::NavBadges;
use crate::theme::{CRIT, OK, TEXT_FAINT, WARN};
use crate::views::firewall::FirewallOperationalState;
use crate::views::overview::summary::{summarize_services, summarize_sockets};
use crate::vault::{ChangeRecord, Vault};
use crate::views::fleet::FleetState;
use crate::views::overview::BlastRadiusInfo;
use crate::views::overview::collector::{
    collect_processes_for_server,
    collect_services_for_server,
    collect_sockets_for_server,
    systemctl_service_action,
    terminate_process,
};

/// A table page's live search box (Services, Processes).
pub struct TableSearch {
    pub input: Entity<InputState>,
    _events: Subscription,
}

/// The table pages with search, filters and paging.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TablePage {
    Services,
    Processes,
    Users,
}

impl TablePage {
    pub fn for_view(view: &str) -> Option<Self> {
        match view {
            "services" => Some(TablePage::Services),
            "processes" => Some(TablePage::Processes),
            "users" => Some(TablePage::Users),
            _ => None,
        }
    }
}

impl CrowApp {
    fn table_search_slot(&mut self, page: TablePage) -> &mut Option<TableSearch> {
        match page {
            TablePage::Services => &mut self.services_search,
            TablePage::Processes => &mut self.processes_search,
            TablePage::Users => &mut self.users_search,
        }
    }

    /// Creates the page's search box on first render (inputs need the
    /// window) and applies a pending focus request (the `/` key).
    pub fn ensure_table_search(&mut self, page: TablePage, window: &mut Window, cx: &mut Context<Self>) {
        if self.table_search_slot(page).is_none() {
            let (placeholder, query) = match page {
                TablePage::Services => ("search services…  ( / )", self.overview.service_query.clone()),
                TablePage::Processes => ("search command, user, pid…  ( / )", self.overview.process_query.clone()),
                TablePage::Users => ("search name, group, shell…  ( / )", self.users.search_query.clone()),
            };
            let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).default_value(query));
            let events = cx.subscribe(&input, move |this, input, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    let value = input.read(cx).value().to_string();
                    match page {
                        TablePage::Services => (this.overview.service_query, this.overview.service_page) = (value, 0),
                        TablePage::Processes => (this.overview.process_query, this.overview.process_page) = (value, 0),
                        TablePage::Users => this.users.search_query = value,
                    }
                    cx.notify();
                }
            });
            *self.table_search_slot(page) = Some(TableSearch { input, _events: events });
        }
        if std::mem::take(&mut self.table_search_focus_pending) {
            if let Some(s) = self.table_search_slot(page).as_ref() {
                let input = s.input.clone();
                input.update(cx, |i, cx| i.focus(window, cx));
            }
        }
    }

    pub fn set_service_filter(&mut self, filter: ServiceFilter, cx: &mut Context<Self>) {
        self.overview.service_filter = filter;
        self.overview.service_page = 0;
        cx.notify();
    }

    pub fn set_process_filter(&mut self, filter: ProcessFilter, cx: &mut Context<Self>) {
        self.overview.process_filter = filter;
        self.overview.process_page = 0;
        cx.notify();
    }

    pub fn set_table_page(&mut self, table: TablePage, page: usize, cx: &mut Context<Self>) {
        match table {
            TablePage::Services => self.overview.service_page = page,
            TablePage::Processes => self.overview.process_page = page,
            TablePage::Users => {}
        }
        cx.notify();
    }
}

/// How long a server's updates/CVE check stays fresh.
const SECURITY_TTL: std::time::Duration = std::time::Duration::from_secs(6 * 3600);

impl CrowApp {
    /// Reads the active server's pending updates and looks up the CVEs they
    /// fix (OSV.dev). Skipped when this server was checked recently, unless
    /// `force`.
    pub fn refresh_security(&mut self, force: bool, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let sec = &mut self.overview.security;
        let same_server = sec.server_id.as_deref() == Some(srv.id.as_str());
        if sec.loading && same_server {
            return;
        }
        if !force && same_server && sec.checked_at.is_some_and(|t| t.elapsed() < SECURITY_TTL) {
            return;
        }
        if !same_server {
            *sec = Default::default();
        }
        sec.server_id = Some(srv.id.clone());
        sec.loading = true;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let server_id = srv.id.clone();
            let (updates, cves) = cx
                .background_executor()
                .spawn(async move {
                    use crate::views::overview::updates::{parse_updates, UPDATES_PROBE};
                    let host = crate::host::host_for(&srv);
                    // As root the probe sees every process (for "running"
                    // packages); without sudo it still reads the rest.
                    let argv = ["sh", "-c", UPDATES_PROBE];
                    let out = host
                        .exec_privileged(&argv, &[], crate::host::DEFAULT_TIMEOUT)
                        .or_else(|_| host.exec(&argv, crate::host::DEFAULT_TIMEOUT));
                    match out {
                        Ok(o) => {
                            let report = parse_updates(&o.stdout);
                            let cves = crate::security::osv::lookup(&report);
                            (Ok(report), Some(cves))
                        }
                        Err(e) => (Err(format!("couldn't read updates: {e}")), None),
                    }
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                let sec = &mut this.overview.security;
                if sec.server_id.as_deref() != Some(server_id.as_str()) {
                    return; // switched servers meanwhile
                }
                sec.loading = false;
                sec.checked_at = Some(std::time::Instant::now());
                sec.checked_label = chrono::Local::now().format("%H:%M").to_string();
                sec.updates = Some(updates);
                sec.cves = cves;
                cx.notify();
            });
        })
        .detach();
    }
}

// ==========================================
// Server overview: services, processes, sockets, service manager
// ==========================================

/// Most recent durable change records for `target` on the active server.
pub fn recent_change_records(vault: &Vault, fleet: &FleetState, target: &str, limit: usize) -> Vec<ChangeRecord> {
    let Some(srv) = fleet.active_server() else {
        return Vec::new();
    };
    let db = vault.db();
    let Ok(db_guard) = db.lock() else {
        return Vec::new();
    };
    db_guard
        .list_change_records(&srv.id, 50)
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.target == target)
        .take(limit)
        .collect()
}

impl CrowApp {
    pub fn set_services_tab(&mut self, tab: &str, cx: &mut Context<Self>) {
        self.overview.active_tab = tab.to_string();
        if let Some(srv) = self.fleet.active_server() {
            let tab_owned = tab.to_string();
            cx.spawn(async move |entity, cx| {
                match tab_owned.as_str() {
                    "processes" => {
                        let procs = cx.background_executor().spawn(async move {
                            collect_processes_for_server(&srv)
                        }).await;
                        let _ = entity.update(cx, |this, cx| {
                            this.overview.processes = procs;
                            cx.notify();
                        });
                    }
                    "sockets" => {
                        let socks = cx.background_executor().spawn(async move {
                            collect_sockets_for_server(&srv)
                        }).await;
                        let _ = entity.update(cx, |this, cx| {
                            this.overview.sockets = socks;
                            cx.notify();
                        });
                    }
                    _ => {
                        let svcs = cx.background_executor().spawn(async move {
                            collect_services_for_server(&srv)
                        }).await;
                        let _ = entity.update(cx, |this, cx| {
                            this.overview.services = svcs;
                            cx.notify();
                        });
                    }
                }
            }).detach();
        }
        cx.notify();
    }

    /// Live sidebar badges, from data Crow actually has. Views without a
    /// meaningful count get none.
    pub fn nav_badges(&self) -> NavBadges {
        let mut badges = NavBadges::new();
        let services = summarize_services(&self.overview.services);
        if services.failed > 0 {
            badges.push(("services", format!("{} FAILED", services.failed), CRIT));
        } else if services.total > 0 {
            badges.push(("services", services.total.to_string(), TEXT_FAINT));
        }
        if !self.overview.processes.is_empty() {
            badges.push(("processes", self.overview.processes.len().to_string(), TEXT_FAINT));
        }
        let sockets = summarize_sockets(&self.overview.sockets);
        if sockets.listening > 0 {
            badges.push(("sockets", sockets.listening.to_string(), if sockets.all_interfaces > 0 { WARN } else { TEXT_FAINT }));
        }
        let unsaved = self.configs.states.values().filter(|st| st.is_modified()).count();
        if unsaved > 0 {
            badges.push(("config", unsaved.to_string(), WARN));
        }
        let errors = self.journal.entries.iter().filter(|e| e.priority.is_error()).count();
        if errors > 0 {
            badges.push(("logs", errors.to_string(), CRIT));
        }
        match &self.firewall.status {
            FirewallOperationalState::Active(summary) if summary.is_active => badges.push(("firewall", "ON".into(), OK)),
            FirewallOperationalState::Active(_) | FirewallOperationalState::Inactive { .. } => badges.push(("firewall", "OFF".into(), WARN)),
            FirewallOperationalState::Unmanaged { .. } => {}
        }
        badges
    }

    /// Collects all three tables for the dashboard in the background.
    pub fn refresh_overview_tables(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        cx.spawn(async move |entity, cx| {
            let (services, processes, sockets) = cx
                .background_executor()
                .spawn(async move {
                    (collect_services_for_server(&srv), collect_processes_for_server(&srv), collect_sockets_for_server(&srv))
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.overview.services = services;
                this.overview.processes = processes;
                this.overview.sockets = sockets;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn open_service_from_dashboard(&mut self, unit: &str, cx: &mut Context<Self>) {
        self.set_view("services", cx);
        self.focus_service(unit, cx);
    }

    pub fn open_process_from_dashboard(&mut self, pid: u32, cx: &mut Context<Self>) {
        self.set_view("processes", cx);
        self.focus_process(pid, cx);
    }

    pub fn open_socket_from_dashboard(&mut self, socket_id: &str, cx: &mut Context<Self>) {
        self.set_view("sockets", cx);
        self.focus_socket(socket_id, cx);
    }

    pub fn toggle_group_services(&mut self, cx: &mut Context<Self>) {
        self.overview.group_services = !self.overview.group_services;
        cx.notify();
    }

    pub fn toggle_group_processes(&mut self, cx: &mut Context<Self>) {
        self.overview.group_processes = !self.overview.group_processes;
        cx.notify();
    }

    pub fn toggle_service_group_collapsed(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.overview.collapsed_service_groups.contains(key) {
            self.overview.collapsed_service_groups.remove(key);
        } else {
            self.overview.collapsed_service_groups.insert(key.to_string());
        }
        cx.notify();
    }

    pub fn toggle_process_group_collapsed(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.overview.collapsed_process_groups.contains(key) {
            self.overview.collapsed_process_groups.remove(key);
        } else {
            self.overview.collapsed_process_groups.insert(key.to_string());
        }
        cx.notify();
    }

    pub fn focus_service(&mut self, name: &str, cx: &mut Context<Self>) {
        for svc in &mut self.overview.services {
            svc.is_focused = svc.name == name;
        }
        cx.notify();
    }

    pub fn toggle_service_confirm(&mut self, name: &str, cx: &mut Context<Self>) {
        let mut now_open = false;
        for svc in &mut self.overview.services {
            if svc.name == name {
                svc.show_confirm = !svc.show_confirm;
                now_open = svc.show_confirm;
            } else {
                svc.show_confirm = false;
            }
        }
        if now_open {
            self.spawn_blast_radius_fetch(name.to_string(), cx);
        } else {
            self.overview.blast_radius = None;
        }
        cx.notify();
    }

    pub fn execute_service_restart(&mut self, name: &str, cx: &mut Context<Self>) {
        self.spawn_service_change(name.to_string(), "restart".to_string(), cx);
    }

    pub fn focus_process(&mut self, pid: u32, cx: &mut Context<Self>) {
        for proc in &mut self.overview.processes {
            proc.is_focused = proc.pid == pid;
        }
        cx.notify();
    }

    pub fn toggle_process_confirm(&mut self, pid: u32, cx: &mut Context<Self>) {
        for proc in &mut self.overview.processes {
            if proc.pid == pid {
                proc.show_confirm = !proc.show_confirm;
            } else {
                proc.show_confirm = false;
            }
        }
        cx.notify();
    }

    pub fn execute_process_kill(&mut self, pid: u32, cx: &mut Context<Self>) {
        self.spawn_process_kill(pid, cx);
    }

    /// Non-destructive service lifecycle actions (start / reload) — run immediately,
    /// no confirm gate, mirroring how a sysadmin would treat them at a real shell.
    /// Also the post-confirm entry point for restart/stop (see execute_service_panel_action).
    pub fn run_service_panel_action_now(&mut self, action: &str, cx: &mut Context<Self>) {
        if let Some(name) = self.overview.services.iter().find(|s| s.is_focused).map(|s| s.name.clone()) {
            self.spawn_service_change(name, action.to_string(), cx);
        }
    }

    /// Arms a disruptive service action (restart / stop) pending inline confirm,
    /// and kicks off a real blast-radius lookup for the confirm bar to show.
    pub fn arm_service_panel_action(&mut self, action: &str, cx: &mut Context<Self>) {
        self.overview.service_panel_pending_action = Some(action.to_string());
        if let Some(name) = self.overview.services.iter().find(|s| s.is_focused).map(|s| s.name.clone()) {
            self.spawn_blast_radius_fetch(name, cx);
        }
        cx.notify();
    }

    pub fn cancel_service_panel_action(&mut self, cx: &mut Context<Self>) {
        self.overview.service_panel_pending_action = None;
        self.overview.blast_radius = None;
        cx.notify();
    }

    pub fn execute_service_panel_action(&mut self, cx: &mut Context<Self>) {
        if let Some(action) = self.overview.service_panel_pending_action.take() {
            self.run_service_panel_action_now(&action, cx);
        }
    }

    /// Computes a real "N connections about to drop" figure for the confirm bar,
    /// off the main thread — the Apply Pipeline's blast-radius step made literal
    /// instead of the generic "any active connections" text it replaces.
    fn spawn_blast_radius_fetch(&mut self, unit_name: String, cx: &mut Context<Self>) {
        self.overview.blast_radius = None;
        let Some(srv) = self.fleet.active_server() else {
            return;
        };
        let Some(pid) = self.overview.services.iter().find(|s| s.name == unit_name).and_then(|s| s.pid.parse::<u32>().ok()) else {
            return;
        };

        cx.spawn(async move |entity, cx| {
            let srv_bg = srv.clone();
            let sockets = cx.background_executor().spawn(async move {
                collect_sockets_for_server(&srv_bg)
            }).await;

            let established = sockets.iter().filter(|s| s.pid == Some(pid) && s.state.contains("ESTAB")).count();
            let listening = sockets.iter().filter(|s| s.pid == Some(pid) && s.state == "LISTEN").count();

            let _ = entity.update(cx, |this, cx| {
                this.overview.blast_radius = Some(BlastRadiusInfo { for_unit: unit_name, established, listening });
                cx.notify();
            });
        }).detach();
    }

    /// The Apply Pipeline for service lifecycle actions: snapshot before-state,
    /// run the action off the main thread, verify the unit actually reached the
    /// expected state, then durably record the outcome. Covers start/stop/
    /// restart/reload — every service mutation in the app goes through this.
    fn spawn_service_change(&mut self, unit_name: String, action: String, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else {
            return;
        };
        let before_state = self.overview.services.iter().find(|s| s.name == unit_name)
            .map(|s| s.status.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let blast_radius_summary = self.overview.blast_radius.as_ref()
            .filter(|b| b.for_unit == unit_name)
            .map(|b| format!("{} established, {} listening", b.established, b.listening));

        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let started_at = chrono::Utc::now().to_rfc3339();
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: format!("service_{}", action),
                target: unit_name.clone(),
                before_state,
                after_state: None,
                blast_radius: blast_radius_summary,
                outcome: "pending".to_string(),
                started_at,
                completed_at: None,
            });
        }

        self.overview.blast_radius = None;
        self.overview.service_panel_pending_action = None;
        for svc in &mut self.overview.services {
            svc.show_confirm = false;
        }

        cx.spawn(async move |entity, cx| {
            let srv_bg = srv.clone();
            let unit_bg = unit_name.clone();
            let action_bg = action.clone();
            let (mut refreshed, after_status, succeeded) = cx.background_executor().spawn(async move {
                let _ = systemctl_service_action(&srv_bg, &unit_bg, &action_bg);
                std::thread::sleep(std::time::Duration::from_millis(1200));
                let refreshed = collect_services_for_server(&srv_bg);
                let status = refreshed.iter().find(|s| s.name == unit_bg).map(|s| s.status.clone()).unwrap_or_else(|| "unknown".to_string());
                let ok = if action_bg == "stop" { status != "ACTIVE" } else { status == "ACTIVE" };
                (refreshed, status, ok)
            }).await;

            let completed_at = chrono::Utc::now().to_rfc3339();
            let outcome_str = if succeeded { "success" } else { "failed" };
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.update_change_record_outcome(&record_id, outcome_str, Some(&after_status), &completed_at);
            }

            let _ = entity.update(cx, |this, cx| {
                for svc in &mut refreshed {
                    svc.is_focused = svc.name == unit_name;
                }
                this.overview.services = refreshed;
                this.push_journal_action_marker(format!("crow: {} {} ({})", action, unit_name, outcome_str));
                this.overview.last_change_outcome = Some((unit_name.clone(), succeeded));
                cx.notify();
            });
        }).detach();
    }

    /// Same pipeline shape as `spawn_service_change`, for a process SIGTERM:
    /// backup, apply off-thread, verify the PID is actually gone, record it.
    fn spawn_process_kill(&mut self, pid: u32, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else {
            return;
        };
        let before_state = self.overview.processes.iter().find(|p| p.pid == pid)
            .map(|p| format!("{} ({})", p.command, p.stat))
            .unwrap_or_else(|| "unknown".to_string());

        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let started_at = chrono::Utc::now().to_rfc3339();
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: "process_kill".to_string(),
                target: format!("pid {}", pid),
                before_state,
                after_state: None,
                blast_radius: None,
                outcome: "pending".to_string(),
                started_at,
                completed_at: None,
            });
        }

        for proc in &mut self.overview.processes {
            proc.show_confirm = false;
        }

        cx.spawn(async move |entity, cx| {
            let srv_bg = srv.clone();
            let (refreshed, succeeded) = cx.background_executor().spawn(async move {
                let _ = terminate_process(&srv_bg, pid, 15);
                std::thread::sleep(std::time::Duration::from_millis(800));
                let refreshed = collect_processes_for_server(&srv_bg);
                let still_alive = refreshed.iter().any(|p| p.pid == pid);
                (refreshed, !still_alive)
            }).await;

            let completed_at = chrono::Utc::now().to_rfc3339();
            let outcome_str = if succeeded { "success" } else { "failed" };
            let after_state = if succeeded { "terminated" } else { "still running" };
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.update_change_record_outcome(&record_id, outcome_str, Some(after_state), &completed_at);
            }

            let _ = entity.update(cx, |this, cx| {
                this.overview.processes = refreshed;
                this.push_journal_action_marker(format!("crow: sent SIGTERM to PID {} ({})", pid, outcome_str));
                cx.notify();
            });
        }).detach();
    }

    /// Jumps to the Config screen pre-selecting the file that governs this service —
    /// the "swap to crow-config" handoff instead of a service-specific settings UI.
    pub fn open_config_for_service(&mut self, file: &str, cx: &mut Context<Self>) {
        self.configs.selected_file = file.to_string();
        self.set_view("config", cx);
    }

    /// Recent Apply Pipeline change records for one unit on the active server —
    /// the audit trail surfaced in the Service Manager panel.

    pub fn focus_socket(&mut self, sock_id: &str, cx: &mut Context<Self>) {
        let mut already_focused = false;
        for (idx, sock) in self.overview.sockets.iter_mut().enumerate() {
            let id = format!("{}:{}:{}", sock.protocol, sock.local_port, idx);
            if id == sock_id {
                if sock.is_focused && self.overview.socket_drawer_open {
                    already_focused = true;
                    sock.is_focused = false;
                } else {
                    sock.is_focused = true;
                }
            } else {
                sock.is_focused = false;
            }
        }
        if already_focused {
            self.overview.socket_drawer_open = false;
        } else {
            self.overview.socket_drawer_open = true;
        }
        cx.notify();
    }

    pub fn toggle_socket_drawer(&mut self, cx: &mut Context<Self>) {
        self.overview.socket_drawer_open = !self.overview.socket_drawer_open;
        cx.notify();
    }

    pub fn close_socket_drawer(&mut self, cx: &mut Context<Self>) {
        self.overview.socket_drawer_open = false;
        cx.notify();
    }

    pub fn toggle_socket_drawer_filter(&mut self, cx: &mut Context<Self>) {
        self.overview.socket_drawer_filter_this_socket = !self.overview.socket_drawer_filter_this_socket;
        cx.notify();
    }
}
