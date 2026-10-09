//! Fleet Setup → HARDENING (ERR-145): the posture check's findings, a
//! guarded fix for each, accepted risks, and the posture report.

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::fleet_run::{record_step, RunJob};
use super::CrowApp;
use crate::host::{host_for, transport_kind, TransportKind};
use crate::security::hardening::{self, Accepted, Finding};
use crate::views::fleet::run::{FleetRun, StepOutcome};

#[derive(Default)]
pub struct HardeningState {
    pub accepted: Vec<Accepted>,
    pub reason: Option<Entity<InputState>>,
    pub message: Option<(bool, String)>,
}

impl CrowApp {
    pub fn load_hardening(&mut self) {
        self.hardening.accepted = self.vault.db().lock().ok().and_then(|db| db.flag(hardening::ACCEPTED_FLAG)).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
    }

    fn save_accepted(&self) {
        if let (Ok(db), Ok(json)) = (self.vault.db().lock(), serde_json::to_string(&self.hardening.accepted)) {
            let _ = db.set_flag(hardening::ACCEPTED_FLAG, &json);
        }
    }

    pub fn ensure_hardening_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.hardening.reason.is_none() {
            self.hardening.reason = Some(cx.new(|cx| InputState::new(window, cx).placeholder("why this risk is acceptable (needed to accept one)")));
        }
    }

    /// Findings on a server, with the acceptance if there is one.
    pub fn server_findings(&self, server_id: &str) -> Option<Vec<(Finding, Option<Accepted>)>> {
        let p = self.topology.posture.get(server_id)?;
        Some(hardening::findings(p).into_iter().map(|f| {
            let a = self.hardening.accepted.iter().find(|a| a.server_id == server_id && a.code == f.code()).cloned();
            (f, a)
        }).collect())
    }

    pub fn accept_risk(&mut self, server_id: &str, code: &str, cx: &mut Context<Self>) {
        let reason = self.hardening.reason.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        if reason.is_empty() {
            self.hardening.message = Some((false, "Write why the risk is acceptable first: the reason goes in the report and the audit log.".into()));
            cx.notify();
            return;
        }
        let by = self.default_author();
        let at = chrono::Utc::now().to_rfc3339();
        self.hardening.accepted.retain(|a| !(a.server_id == server_id && a.code == code));
        self.hardening.accepted.push(Accepted { server_id: server_id.into(), code: code.into(), reason: reason.clone(), by, at });
        self.save_accepted();
        self.audit_posture(server_id, "posture.accept", &format!("accepted {code}: {reason}"));
        self.hardening.message = Some((true, "Accepted: it's no longer flagged on the map or here, and the report lists it with your reason.".into()));
        self.reload_topology_alerts();
        cx.notify();
    }

    pub fn unaccept_risk(&mut self, server_id: &str, code: &str, cx: &mut Context<Self>) {
        self.hardening.accepted.retain(|a| !(a.server_id == server_id && a.code == code));
        self.save_accepted();
        self.audit_posture(server_id, "posture.accept", &format!("no longer accepting {code}"));
        cx.notify();
    }

    fn audit_posture(&self, server_id: &str, kind: &str, what: &str) {
        let name = self.fleet.servers.iter().find(|s| s.id == server_id).map(|s| s.name.clone()).unwrap_or_default();
        if let Ok(db) = self.vault.db().lock() {
            let now = chrono::Utc::now().to_rfc3339();
            let _ = db.insert_change_record(&crate::vault::ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: server_id.into(),
                server_name: name.clone(),
                action_kind: kind.into(),
                target: name,
                before_state: what.into(),
                after_state: None,
                blast_radius: None,
                outcome: "success".into(),
                started_at: now.clone(),
                completed_at: Some(now),
            });
        }
    }

    /// Plans fixing finding `code` on `server` (or on every server that has
    /// it, not accepted), on the fleet runner.
    pub fn plan_hardening_fix(&mut self, code: &str, server: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<(String, String)>) = (Vec::new(), Vec::new(), Vec::new());
        // Sync key paths: fresh logins resolve them through the SSH directory.
        self.sync_ssh_directory();
        for srv in self.fleet.servers.clone() {
            if server.is_some_and(|s| s != srv.id) {
                continue;
            }
            let Some(found) = self.server_findings(&srv.id) else { continue };
            let Some((finding, accepted)) = found.into_iter().find(|(f, _)| f.code() == code) else { continue };
            if accepted.is_some() {
                excluded.push((srv.name.clone(), "accepted as a risk".into()));
                continue;
            }
            if transport_kind(&srv) != TransportKind::Ssh {
                excluded.push((srv.name.clone(), "not an SSH server (Crow proves every fix with a fresh SSH login)".into()));
                continue;
            }
            if !self.fleet.health(&srv).is_ok() {
                excluded.push((srv.name.clone(), "not reachable right now".into()));
                continue;
            }
            if !finding.fixable(&srv.login_user) {
                excluded.push((srv.name.clone(), "Crow logs in as root with a key: prohibit-password is already the safest setting it can keep".into()));
                continue;
            }
            let what = finding.fix(&srv.login_user);
            steps.push(FleetRun::step(&srv.id, &srv.name, what.clone()));
            let ssh_ports = self.topology.posture.get(&srv.id).map(|p| p.ssh_ports.clone()).unwrap_or_default();
            let db = self.vault.db();
            jobs.push(Box::new(move || {
                let host = host_for(&srv);
                let login = || crate::host::ssh::fresh_key_login(&srv);
                let outcome = match &finding {
                    Finding::RootPasswordLogin => hardening::set_root_login(host.as_ref(), "prohibit-password", &srv.login_user, &login),
                    Finding::RootKeyLogin(_) => hardening::set_root_login(host.as_ref(), if srv.login_user == "root" { "prohibit-password" } else { "no" }, &srv.login_user, &login),
                    Finding::PasswordLogin => crate::security::sshd_passwords::turn_off(host.as_ref(), &login),
                    Finding::NoFirewall(ports) => hardening::enable_firewall(host.as_ref(), &ssh_ports, ports, &login),
                };
                let result = match outcome {
                    Ok(log) => Ok(StepOutcome::Done(log.last().cloned().unwrap_or_default())),
                    Err(log) => Err(log.iter().rev().take(2).rev().cloned().collect::<Vec<_>>().join(" ")),
                };
                record_step(&db, &srv, "posture.fix", &what, &result);
                result
            }));
        }
        let n = steps.len();
        // Read the posture again once it's done.
        self.topology.posture_checked_at = 0;
        let title = format!("HARDEN · {} · {n} SERVER{}", code.to_uppercase(), if n == 1 { "" } else { "S" });
        self.open_fleet_run(FleetRun::new(title, "HARDEN", steps, excluded), jobs, window, cx);
    }

    /// Writes the posture report (Markdown) to the downloads folder.
    pub fn export_posture_report(&mut self, cx: &mut Context<Self>) {
        let rows: Vec<_> = self
            .fleet
            .servers
            .iter()
            .map(|s| (s.name.clone(), self.topology.posture.get(&s.id).cloned(), self.server_findings(&s.id).unwrap_or_default()))
            .collect();
        let fixed: Vec<(String, String, String)> = self
            .vault
            .db()
            .lock()
            .ok()
            .and_then(|db| db.list_all_change_records(2000).ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.action_kind == "posture.fix" && r.outcome == "success")
            .map(|r| (r.server_name, r.before_state, r.started_at.get(..16).unwrap_or(&r.started_at).replace('T', " ")))
            .collect();
        let now = chrono::Local::now();
        let md = hardening::report_markdown(&rows, &fixed, &now.format("%Y-%m-%d %H:%M").to_string());
        let dir = dirs::download_dir().or_else(dirs::document_dir).or_else(dirs::home_dir).unwrap_or_else(std::env::temp_dir);
        let path = dir.join(format!("crow-posture-{}.md", now.format("%Y%m%d-%H%M")));
        self.hardening.message = Some(match std::fs::write(&path, md) {
            Ok(()) => (true, format!("Report written to {}", path.display())),
            Err(e) => (false, format!("Couldn't write the report to {}: {e}", path.display())),
        });
        cx.notify();
    }
}
