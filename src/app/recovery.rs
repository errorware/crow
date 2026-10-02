//! The connection recovery panel (ERR-89): diagnose why a server can't be
//! reached, and offer the fix.

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::host::diagnose::{diagnose, Diagnosis, KeyTried};
use crate::host::{connection_state, host_for, ConnectionState, Host, SshHost, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;
use crate::views::onboard::probe::{probe_host, trust_host_keys, ProbeResult};

pub struct RecoveryPanel {
    pub server_id: String,
    /// `None` while the host is being probed.
    pub diagnosis: Option<Diagnosis>,
    probe: Option<ProbeResult>,
    /// Typed server name, to trust a new host key.
    pub confirm: Entity<InputState>,
    pub busy: Option<String>,
    pub error: Option<String>,
}

impl CrowApp {
    fn key_tried(&self, srv: &ServerRecord) -> KeyTried {
        let path = srv.key_id.as_ref().and_then(|id| self.keys.enrolled.iter().find(|k| &k.id == id)).and_then(|k| k.private_key_path.clone());
        match (srv.auth_method.as_str(), path) {
            ("publickey", Some(p)) => {
                let full = match (p.strip_prefix("~/"), dirs::home_dir()) {
                    (Some(rest), Some(home)) => home.join(rest),
                    _ => std::path::PathBuf::from(&p),
                };
                KeyTried::File { exists: full.exists(), path: p }
            }
            _ => KeyTried::Agent,
        }
    }

    fn jump_name(&self, srv: &ServerRecord) -> Option<String> {
        srv.jump_host_id.as_ref().and_then(|id| self.fleet.servers.iter().find(|s| &s.id == id)).map(|j| j.name.clone())
    }

    /// Opens the panel for `server_id` and probes the host directly in the
    /// background (unless a jump host is in the way).
    pub fn open_recovery(&mut self, server_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == server_id).cloned() else { return };
        let confirm = cx.new(|cx| InputState::new(window, cx).placeholder(srv.name.clone()));
        self.recovery = Some(RecoveryPanel { server_id: srv.id.clone(), diagnosis: None, probe: None, confirm, busy: Some("checking the host…".into()), error: None });
        cx.notify();
        self.rediagnose(srv, cx);
    }

    fn rediagnose(&mut self, srv: ServerRecord, cx: &mut Context<Self>) {
        let jump = self.jump_name(&srv);
        let key = self.key_tried(&srv);
        cx.spawn(async move |entity, cx| {
            let target = srv.clone();
            let for_agent = srv.clone();
            let approval_agent = cx.background_executor().spawn(async move { crate::host::ssh::approval_agent_for(&for_agent) }).await;
            let probe = if jump.is_none() {
                Some(cx.background_executor().spawn(async move { probe_host(&target.host, if target.port == 0 { 22 } else { target.port }).0 }).await)
            } else {
                None
            };
            let _ = entity.update(cx, |this, cx| {
                let Some(panel) = this.recovery.as_mut().filter(|p| p.server_id == srv.id) else { return };
                let state = connection_state(&srv.id).unwrap_or(ConnectionState::Unreachable(String::new()));
                panel.diagnosis = Some(diagnose(&srv, &state, probe.as_ref(), &key, jump.as_deref(), approval_agent));
                panel.probe = probe;
                panel.busy = None;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn close_recovery(&mut self, cx: &mut Context<Self>) {
        self.recovery = None;
        cx.notify();
    }

    /// Drops the held connection, connects again, and re-diagnoses; closes
    /// the panel if it now works.
    pub fn recovery_retry(&mut self, cx: &mut Context<Self>) {
        let Some(panel) = self.recovery.as_mut() else { return };
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == panel.server_id).cloned() else { return };
        panel.busy = Some("connecting…".into());
        panel.error = None;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let target = srv.clone();
            let ok = cx
                .background_executor()
                .spawn(async move {
                    SshHost::for_server(&target).close_connection();
                    host_for(&target).exec(&["true"], DEFAULT_TIMEOUT).is_ok()
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                if ok {
                    this.recovery = None;
                    this.fleet.notice = Some(format!("{} is reachable again", srv.name));
                    this.reload_configs_for_active_server(cx);
                    this.refresh_firewall_for_active_server(cx);
                    cx.notify();
                } else {
                    this.rediagnose(srv, cx);
                }
            });
        })
        .detach();
    }

    /// Hands over to onboarding's password login, which installs Crow's key
    /// once and never stores the password.
    pub fn recovery_install_key(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.recovery.as_ref().and_then(|p| self.fleet.servers.iter().find(|s| s.id == p.server_id)).cloned() else { return };
        self.recovery = None;
        self.start_onboarding_for(&srv, cx);
    }

    /// Replaces the host's known_hosts entry with the keys it presents now,
    /// pins the new fingerprint, and records it. Only after the server's name
    /// is typed.
    pub fn recovery_trust_new_key(&mut self, cx: &mut Context<Self>) {
        let Some(panel) = self.recovery.as_mut() else { return };
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == panel.server_id).cloned() else { return };
        if panel.confirm.read(cx).value().trim() != srv.name {
            panel.error = Some(format!("Type {} exactly to trust the new key.", srv.name));
            cx.notify();
            return;
        }
        let Some(probe) = panel.probe.clone().filter(|p| !p.scanned_keys.is_empty()) else {
            panel.error = Some("No key was fetched from the server; nothing to trust.".into());
            cx.notify();
            return;
        };
        let port = if srv.port == 0 { 22 } else { srv.port };
        let pattern = if port == 22 { srv.host.clone() } else { format!("[{}]:{port}", srv.host) };
        // ssh-keygen -R keeps the old file as known_hosts.old.
        let removed = crate::host::LocalHost.exec(&["ssh-keygen", "-R", &pattern], DEFAULT_TIMEOUT);
        let result = removed.map_err(|e| e.to_string()).and_then(|_| trust_host_keys(&probe.scanned_keys).map_err(|e| e.to_string()));
        if let Err(e) = result {
            panel.error = Some(format!("known_hosts wasn't changed: {e}"));
            cx.notify();
            return;
        }
        let before = srv.host_key_fingerprint.clone().unwrap_or_default();
        if let Ok(db) = self.vault.db().lock() {
            let mut updated = srv.clone();
            updated.host_key_fingerprint = Some(probe.host_key_fingerprint.clone());
            let _ = db.upsert_server(&updated);
            let now = chrono::Utc::now().to_rfc3339();
            let _ = db.insert_change_record(&crate::vault::ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: "ssh.host_key".into(),
                target: pattern.clone(),
                before_state: before,
                after_state: Some(probe.host_key_fingerprint.clone()),
                blast_radius: Some("known_hosts entry replaced (old file kept as known_hosts.old)".into()),
                outcome: "success".into(),
                started_at: now.clone(),
                completed_at: Some(now),
            });
        }
        self.reload_servers();
        self.recovery_retry(cx);
    }
}
