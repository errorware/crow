//! Fleet Setup → INCIDENTS (ERR-149): a timeline around an alert (or any
//! window on a server): alerts, Crow's changes, check failures and error
//! logs, a clanker summary, and a Markdown export.

use std::collections::HashMap;

use gpui_kit::*;

use super::CrowApp;
use crate::host::host_for;
use crate::incident::{self, Event};
use crate::journal::{JournalPriority, JournalQuery};

pub struct IncidentScope {
    pub title: String,
    pub server_ids: Vec<String>,
    pub from: i64,
    pub to: i64,
}

#[derive(Default)]
pub struct IncidentsState {
    pub scope: Option<IncidentScope>,
    pub events: Vec<Event>,
    pub loading: usize,
    pub summarizing: bool,
    pub summary: Option<Result<(String, String), String>>,
    pub message: Option<(bool, String)>,
}

impl CrowApp {
    /// The timeline around alert `id`: from 30 minutes before it opened to
    /// 10 minutes after it resolved (or now).
    pub fn open_incident_for_alert(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(a) = self.vault.db().lock().ok().and_then(|db| db.list_alerts(0).ok()).and_then(|v| v.into_iter().find(|a| a.id == id)) else { return };
        let name = self.fleet.servers.iter().find(|s| s.id == a.server_id).map(|s| s.name.clone()).unwrap_or_else(|| a.server_id.clone());
        let now = chrono::Utc::now().timestamp();
        let to = a.resolved_at.map(|r| (r + 600).min(now)).unwrap_or(now);
        self.load_incident(IncidentScope { title: format!("{} on {name}", a.detail), server_ids: vec![a.server_id.clone()], from: a.opened_at - 1800, to }, cx);
    }

    /// The last `secs` on one server.
    pub fn open_incident_window(&mut self, server_id: &str, secs: i64, cx: &mut Context<Self>) {
        let name = self.fleet.servers.iter().find(|s| s.id == server_id).map(|s| s.name.clone()).unwrap_or_default();
        let now = chrono::Utc::now().timestamp();
        self.load_incident(IncidentScope { title: format!("{name}, the last {}", if secs >= 86_400 { format!("{} days", secs / 86_400) } else { format!("{} hours", secs / 3600) }), server_ids: vec![server_id.to_string()], from: now - secs, to: now }, cx);
    }

    /// Gathers what's local (alerts, changes, checks) now, and the error
    /// logs from each server in the background.
    fn load_incident(&mut self, scope: IncidentScope, cx: &mut Context<Self>) {
        let names: HashMap<String, String> = self.fleet.servers.iter().map(|s| (s.id.clone(), s.name.clone())).collect();
        let (alerts, changes) = {
            let db = self.vault.db();
            let Ok(db) = db.lock() else { return };
            let alerts: Vec<_> = db.list_alerts(scope.from).unwrap_or_default().into_iter().filter(|a| scope.server_ids.contains(&a.server_id)).collect();
            let changes: Vec<_> = db.list_all_change_records(5000).unwrap_or_default().into_iter().filter(|c| scope.server_ids.contains(&c.server_id)).collect();
            (alerts, changes)
        };
        let failures: Vec<(String, String, i64, String)> = {
            let db = self.vault.db();
            let guard = db.lock().ok();
            self.checks
                .defs
                .iter()
                .filter(|c| c.server_id.as_ref().is_some_and(|s| scope.server_ids.contains(s)))
                .flat_map(|c| {
                    guard.as_ref().and_then(|db| db.check_results(&c.id, scope.from).ok()).unwrap_or_default().into_iter().filter(|r| !r.1).map(move |r| (c.name.clone(), c.server_id.clone().unwrap_or_default(), r.0, r.3)).collect::<Vec<_>>()
                })
                .collect()
        };
        let src = incident::Sources { from: scope.from, to: scope.to, names: &names, alerts: &alerts, changes: &changes, check_failures: &failures, logs: &[] };
        self.incidents.events = incident::build(&src);
        self.incidents.summary = None;
        self.incidents.message = None;
        // Error logs (err and worse), from each server, in the window.
        let query = JournalQuery { limit: 300, priority: Some(JournalPriority::Err), window: Some((scope.from, scope.to)), ..Default::default() };
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| scope.server_ids.contains(&s.id) && self.fleet.health(s).is_ok()).cloned().collect();
        self.incidents.loading = servers.len();
        let (from, to) = (scope.from, scope.to);
        self.incidents.scope = Some(scope);
        for srv in servers {
            let (q, id, names) = (query.clone(), srv.id.clone(), names.clone());
            cx.spawn(async move |entity, cx| {
                let lines = cx.background_executor().spawn(async move { crate::journal::reader::read_journal(host_for(&srv).as_ref(), &q).unwrap_or_default() }).await;
                let _ = entity.update(cx, |this, cx| {
                    this.incidents.loading = this.incidents.loading.saturating_sub(1);
                    let logs = vec![(id, lines)];
                    let src = incident::Sources { from, to, names: &names, alerts: &[], changes: &[], check_failures: &[], logs: &logs };
                    this.incidents.events.extend(incident::build(&src));
                    this.incidents.events.sort_by(|a, b| a.ts.cmp(&b.ts).then(a.kind.cmp(&b.kind)));
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub fn summarize_incident(&mut self, cx: &mut Context<Self>) {
        if self.incidents.events.is_empty() || self.incidents.summarizing {
            return;
        }
        let Some((primary, backup)) = self.ai_providers() else {
            self.incidents.summary = Some(Err("No AI provider has a key yet. Add one in Settings → Clankers (AI).".into()));
            cx.notify();
            return;
        };
        let lines = incident::as_lines(&self.incidents.events, 400);
        let context = "an incident timeline: alerts, changes Crow made, failed outside checks and error log lines, oldest first. Write the likely sequence of events and the most probable cause, in a few short paragraphs; say what's uncertain".to_string();
        self.incidents.summarizing = true;
        cx.notify();
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let answer = cx.background_executor().spawn(async move { crate::ai::explain_logs_with_fallback(&primary, backup.as_ref(), &context, &lines) }).await;
            if let Ok(a) = &answer {
                if let Ok(db) = db.lock() {
                    let _ = db.record_clanker_call(&a.provider_id);
                }
            }
            let _ = entity.update(cx, |this, cx| {
                this.incidents.summarizing = false;
                this.incidents.summary = Some(answer.map(|a| (a.text, a.provider_label)));
                cx.notify();
            });
        })
        .detach();
    }

    pub fn export_incident(&mut self, cx: &mut Context<Self>) {
        let Some(scope) = &self.incidents.scope else { return };
        let summary = self.incidents.summary.as_ref().and_then(|s| s.as_ref().ok()).map(|(t, _)| t.as_str());
        let md = incident::markdown(&scope.title, scope.from, scope.to, &self.incidents.events, summary);
        let now = chrono::Local::now();
        let dir = dirs::download_dir().or_else(dirs::document_dir).or_else(dirs::home_dir).unwrap_or_else(std::env::temp_dir);
        let path = dir.join(format!("crow-incident-{}.md", now.format("%Y%m%d-%H%M")));
        self.incidents.message = Some(match std::fs::write(&path, md) {
            Ok(()) => (true, format!("Timeline written to {}", path.display())),
            Err(e) => (false, format!("Couldn't write {}: {e}", path.display())),
        });
        cx.notify();
    }
}
