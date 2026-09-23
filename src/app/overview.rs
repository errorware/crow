use gpui_kit::*;

use super::CrowApp;
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
