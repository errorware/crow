//! Fleet Setup → CERTIFICATES (ERR-146): every server's certificates,
//! whether they renew on their own, and a renewal from Crow.

use std::collections::{HashMap, HashSet};

use gpui_kit::*;

use super::fleet_run::{record_step, RunJob};
use super::CrowApp;
use crate::certs::{self, Inventory, Renewal};
use crate::host::host_for;
use crate::metrics::certs::{read_certs, CertInfo, CERT_CHECK_EVERY_SECS};
use crate::views::fleet::run::{FleetRun, StepOutcome};

/// The alert kind for a server whose certificates won't renew on their own.
pub const AUTORENEW_ALERT: &str = "cert-autorenew";

#[derive(Default)]
pub struct CertificatesState {
    pub inventory: HashMap<String, Result<Inventory, String>>,
    pub certs: HashMap<String, Vec<CertInfo>>,
    pub reading: HashSet<String>,
    pub last_check: i64,
    pub message: Option<(bool, String)>,
}

/// The certbot certificate a served certificate file belongs to
/// (`/etc/letsencrypt/live/<name>/…`).
pub fn certbot_name(path: &str) -> Option<&str> {
    path.strip_prefix("/etc/letsencrypt/live/")?.split('/').next().filter(|n| !n.is_empty())
}

impl CrowApp {
    /// Reads every reachable server's certificates and renewal setup;
    /// without `force`, only every 6 hours.
    pub fn refresh_certificates(&mut self, force: bool, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        if !force && now - self.certificates.last_check < CERT_CHECK_EVERY_SECS {
            return;
        }
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| self.fleet.health(s).is_ok() && !self.certificates.reading.contains(&s.id)).cloned().collect();
        if servers.is_empty() {
            return;
        }
        self.certificates.last_check = now;
        for srv in servers {
            self.certificates.reading.insert(srv.id.clone());
            let id = srv.id.clone();
            cx.spawn(async move |entity, cx| {
                let (inv, found) = cx
                    .background_executor()
                    .spawn(async move {
                        let host = host_for(&srv);
                        (certs::read_inventory(host.as_ref()), read_certs(host.as_ref()))
                    })
                    .await;
                let _ = entity.update(cx, |this, cx| {
                    this.certificates.reading.remove(&id);
                    if let Ok(inv) = &inv {
                        this.autorenew_alert(&id, inv.auto_renew_problem());
                    }
                    this.certificates.inventory.insert(id.clone(), inv);
                    if let Some(found) = found {
                        this.certificates.certs.insert(id, found);
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    /// Opens, updates or resolves a server's broken-auto-renewal alert.
    fn autorenew_alert(&mut self, server_id: &str, problem: Option<String>) {
        use crate::metrics::alerts::{Alert, AlertChanges};
        let now = chrono::Utc::now().timestamp();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let open = db.list_alerts(i64::MAX).unwrap_or_default();
        let existing = open.iter().find(|a| a.server_id == server_id && a.kind == AUTORENEW_ALERT && a.resolved_at.is_none());
        let mut changes = AlertChanges::default();
        match (problem, existing) {
            (Some(p), Some(a)) => changes.updated.push((a.id.clone(), "WARN".into(), p, now)),
            (Some(p), None) => changes.opened.push(Alert {
                id: format!("alert-{server_id}-{AUTORENEW_ALERT}-{now}"),
                server_id: server_id.into(),
                kind: AUTORENEW_ALERT.into(),
                level: "WARN".into(),
                detail: p,
                opened_at: now,
                last_seen: now,
                resolved_at: None,
                acknowledged_at: None,
            }),
            (None, Some(a)) => changes.resolved.push(a.id.clone()),
            (None, None) => {}
        }
        let _ = db.apply_alert_changes(&changes, now);
    }

    pub fn certificates_tick(&mut self, cx: &mut Context<Self>) {
        if self.vault.status() != crate::vault::VaultStatus::Locked {
            self.refresh_certificates(false, cx);
        }
    }

    /// Plans renewing one certificate on one server, on the fleet runner.
    pub fn plan_cert_renewal(&mut self, server_id: &str, cert_path: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == server_id).cloned() else { return };
        let Some(Ok(inv)) = self.certificates.inventory.get(server_id).cloned() else { return };
        let label = self.certificates.certs.get(server_id).and_then(|c| c.iter().find(|c| c.path == cert_path)).map(|c| c.name.clone()).unwrap_or_default();
        let renewal = match (certbot_name(cert_path), &inv.acme_sh) {
            (Some(name), _) if inv.certbot => Renewal::Certbot { name: name.to_string() },
            (_, Some(script)) if !label.is_empty() => Renewal::AcmeSh { script: script.clone(), domain: label.clone() },
            _ => {
                self.certificates.message = Some((false, format!("{cert_path} isn't managed by certbot or acme.sh on {}: renew it where it came from.", srv.name)));
                cx.notify();
                return;
            }
        };
        let what = match &renewal {
            Renewal::Certbot { name } => format!("certbot: dry run, renew {name}, reload{}{}, confirm", if inv.nginx { " nginx" } else { "" }, if inv.apache { " apache" } else { "" }),
            Renewal::AcmeSh { domain, .. } => format!("acme.sh: renew {domain}, reload, confirm"),
        };
        let (db, path, what2) = (self.vault.db(), cert_path.to_string(), what.clone());
        let job: RunJob = Box::new(move || {
            let host = host_for(&srv);
            let confirm = || read_certs(host.as_ref()).and_then(|c| c.into_iter().find(|c| c.path == path)).and_then(|c| chrono::DateTime::from_timestamp(c.expires, 0)).map(|t| t.format("%Y-%m-%d").to_string());
            let result = match certs::renew(host.as_ref(), &renewal, inv.nginx, inv.apache, &confirm) {
                Ok(log) if log.iter().any(|l| l.starts_with('=')) => Ok(StepOutcome::Skipped(log.last().cloned().unwrap_or_default())),
                Ok(log) => Ok(StepOutcome::Done(log.last().cloned().unwrap_or_default())),
                Err(log) => Err(log.last().cloned().unwrap_or_default()),
            };
            record_step(&db, &srv, "cert.renew", &what2, &result);
            result
        });
        let step = FleetRun::step(server_id, &self.fleet.servers.iter().find(|s| s.id == server_id).map(|s| s.name.clone()).unwrap_or_default(), what);
        // Read the certificates again once it's done.
        self.certificates.last_check = 0;
        self.open_fleet_run(FleetRun::new(format!("RENEW {}", if label.is_empty() { cert_path.to_string() } else { label }), "RENEW", vec![step], Vec::new()), vec![job], window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::certbot_name;

    #[test]
    fn certbot_names_come_from_the_live_folder() {
        assert_eq!(certbot_name("/etc/letsencrypt/live/example.com/fullchain.pem"), Some("example.com"));
        assert_eq!(certbot_name("/etc/ssl/certs/site.pem"), None);
    }
}
