//! Turn off SSH password login on the active server, safely (ERR-34).
//! The procedure and its checks live in `security::sshd_passwords`.

use gpui_kit::*;

use super::CrowApp;
use crate::security::sshd_passwords::{self, Preflight};
use crate::vault::ChangeRecord;

pub enum PasswordLoginStage {
    Checking,
    Ready(Preflight),
    /// Can't be offered here, and why.
    Unavailable(String),
    Running,
    Done(Result<Vec<String>, Vec<String>>),
}

pub struct PasswordLoginFlow {
    pub server_id: String,
    pub server_name: String,
    pub stage: PasswordLoginStage,
    /// "I understand" for accounts that would lose password login.
    pub ack: bool,
}

impl CrowApp {
    pub fn open_password_login(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let mut flow = PasswordLoginFlow { server_id: srv.id.clone(), server_name: srv.name.clone(), stage: PasswordLoginStage::Checking, ack: false };
        if srv.auth_method != "publickey" {
            flow.stage = PasswordLoginStage::Unavailable("Crow doesn't connect to this server with its own key yet. Set up key login first (Add Server → install Crow's key), then turn passwords off.".into());
            self.password_login = Some(flow);
            cx.notify();
            return;
        }
        self.password_login = Some(flow);
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { sshd_passwords::preflight(crate::host::host_for(&srv).as_ref()) }).await;
            let _ = entity.update(cx, |this, cx| {
                if let Some(f) = this.password_login.as_mut() {
                    f.stage = match result {
                        Ok(p) if !p.password_login_on => PasswordLoginStage::Done(Ok(vec!["✓ Password login is already off on this server.".into()])),
                        Ok(p) if !p.reads_dropins => PasswordLoginStage::Unavailable("This sshd_config doesn't include sshd_config.d, so Crow's drop-in wouldn't apply. Set PasswordAuthentication to no in the sshd_config sheet, then reload sshd.".into()),
                        Ok(p) => PasswordLoginStage::Ready(p),
                        Err(e) => PasswordLoginStage::Unavailable(format!("Couldn't read sshd's settings: {e}")),
                    };
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn toggle_password_login_ack(&mut self, cx: &mut Context<Self>) {
        if let Some(f) = self.password_login.as_mut() {
            f.ack = !f.ack;
            cx.notify();
        }
    }

    pub fn close_password_login(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.password_login.as_ref().map(|f| &f.stage), Some(PasswordLoginStage::Running)) {
            self.password_login = None;
            cx.notify();
        }
    }

    /// Runs the guarded procedure in the background; records the outcome.
    pub fn run_password_login_off(&mut self, cx: &mut Context<Self>) {
        let Some(flow) = self.password_login.as_mut() else { return };
        let PasswordLoginStage::Ready(pre) = &flow.stage else { return };
        if !pre.password_users.is_empty() && !flow.ack {
            return;
        }
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == flow.server_id).cloned() else { return };
        flow.stage = PasswordLoginStage::Running;
        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let db = self.vault.db();
        if let Ok(db) = db.lock() {
            let _ = db.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: "sshd.password_login_off".into(),
                target: sshd_passwords::DROPIN.into(),
                before_state: "PasswordAuthentication yes".into(),
                after_state: None,
                blast_radius: Some("SSH logins: passwords refused, keys only".into()),
                outcome: "pending".into(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let host = crate::host::host_for(&srv);
                    sshd_passwords::turn_off(host.as_ref(), &|| crate::host::ssh::fresh_key_login(&srv))
                })
                .await;
            if let Ok(db) = db.lock() {
                let (outcome, log) = match &result {
                    Ok(l) => ("success", l),
                    Err(l) => ("failed", l),
                };
                let _ = db.update_change_record_outcome(&record_id, outcome, Some(&log.join("\n")), &chrono::Utc::now().to_rfc3339());
            }
            let _ = entity.update(cx, |this, cx| {
                if result.is_ok() {
                    this.push_journal_action_marker("crow: SSH password login turned off (keys only)".into());
                }
                if let Some(f) = this.password_login.as_mut() {
                    f.stage = PasswordLoginStage::Done(result);
                }
                cx.notify();
            });
        })
        .detach();
    }
}
