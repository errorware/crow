//! Fleet Setup → CHECKS (ERR-148): checks from outside, their uptime and
//! latency history, and alerts when they fail.

use std::collections::{HashMap, HashSet};

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::checks::{self, Check, Kind, Outcome};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormKind {
    #[default]
    Http,
    Tcp,
    Dns,
}

#[derive(Default)]
pub struct ChecksState {
    pub defs: Vec<Check>,
    pub last: HashMap<String, Outcome>,
    pub last_run: HashMap<String, i64>,
    pub fails: HashMap<String, u32>,
    pub running: HashSet<String>,
    /// (ts, ok, latency) per check, the last 7 days.
    pub history: HashMap<String, Vec<(i64, bool, Option<i64>)>>,
    pub last_prune: i64,
    pub form: FormKind,
    pub target: Option<Entity<InputState>>,
    pub extra: Option<Entity<InputState>>,
    pub name: Option<Entity<InputState>>,
    pub attach: Option<String>,
    pub every: i64,
    pub message: Option<(bool, String)>,
}

/// The alerts' server for a check: its server, or a fleet-wide label.
pub fn alert_server(c: &Check) -> String {
    c.server_id.clone().unwrap_or_else(|| "checks".into())
}

impl CrowApp {
    pub fn load_checks(&mut self) {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        self.checks.defs = db.flag(checks::CHECKS_FLAG).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        let since = chrono::Utc::now().timestamp() - 7 * 86_400;
        for c in &self.checks.defs {
            let h: Vec<_> = db.check_results(&c.id, since).unwrap_or_default().into_iter().map(|(ts, ok, l, _)| (ts, ok, l)).collect();
            self.checks.history.insert(c.id.clone(), h);
        }
        if self.checks.every == 0 {
            self.checks.every = 60;
        }
    }

    fn save_checks(&self) {
        if let (Ok(db), Ok(json)) = (self.vault.db().lock(), serde_json::to_string(&self.checks.defs)) {
            let _ = db.set_flag(checks::CHECKS_FLAG, &json);
        }
    }

    pub fn ensure_checks_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.checks.target.is_none() {
            self.checks.target = Some(cx.new(|cx| InputState::new(window, cx).placeholder("https://example.com/health · db.lan:5432 · app.example.com")));
            self.checks.extra = Some(cx.new(|cx| InputState::new(window, cx).placeholder("HTTP: a word the page must contain · DNS: the address it must resolve to")));
            self.checks.name = Some(cx.new(|cx| InputState::new(window, cx).placeholder("name (optional)")));
        }
        if self.checks.every == 0 {
            self.checks.every = 60;
        }
    }

    pub fn set_check_form(&mut self, k: FormKind, cx: &mut Context<Self>) {
        self.checks.form = k;
        cx.notify();
    }

    pub fn set_check_attach(&mut self, server: Option<String>, cx: &mut Context<Self>) {
        self.checks.attach = server;
        cx.notify();
    }

    pub fn set_check_every(&mut self, secs: i64, cx: &mut Context<Self>) {
        self.checks.every = secs;
        cx.notify();
    }

    pub fn add_check(&mut self, cx: &mut Context<Self>) {
        let v = |i: &Option<Entity<InputState>>| i.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        let (target, extra, name) = (v(&self.checks.target), v(&self.checks.extra), v(&self.checks.name));
        if target.is_empty() {
            self.checks.message = Some((false, "Type what to check first.".into()));
            cx.notify();
            return;
        }
        let kind = match self.checks.form {
            FormKind::Http => {
                if !(target.starts_with("http://") || target.starts_with("https://")) {
                    self.checks.message = Some((false, "An HTTP check needs a URL starting with http:// or https://.".into()));
                    cx.notify();
                    return;
                }
                Kind::Http { url: target.clone(), keyword: extra, status: None }
            }
            FormKind::Tcp => match target.rsplit_once(':').and_then(|(h, p)| p.parse::<u16>().ok().map(|p| (h.trim_matches(['[', ']']).to_string(), p))) {
                Some((host, port)) if !host.is_empty() => Kind::Tcp { host, port },
                _ => {
                    self.checks.message = Some((false, "A TCP check is host:port, like db.lan:5432.".into()));
                    cx.notify();
                    return;
                }
            },
            FormKind::Dns => Kind::Dns { name: target.clone(), expect: extra },
        };
        let check = Check { id: format!("chk{}", chrono::Utc::now().timestamp_millis()), name: if name.is_empty() { target } else { name }, kind, server_id: self.checks.attach.clone(), every_secs: self.checks.every.max(30), alert: true };
        self.checks.message = Some((true, format!("Added \"{}\": checked every {} s while Crow is open.", check.name, check.every_secs)));
        let id = check.id.clone();
        self.checks.defs.push(check);
        self.save_checks();
        self.run_check(&id, cx);
    }

    pub fn delete_check(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(c) = self.checks.defs.iter().find(|c| c.id == id).cloned() else { return };
        self.checks.defs.retain(|c| c.id != id);
        self.save_checks();
        let now = chrono::Utc::now().timestamp();
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.prune_check_results(now - checks::KEEP_SECS, Some(id));
            let open = db.list_alerts(i64::MAX).unwrap_or_default();
            let changes = crate::metrics::alerts::AlertChanges { resolved: open.iter().filter(|a| a.kind == c.alert_kind() && a.resolved_at.is_none()).map(|a| a.id.clone()).collect(), ..Default::default() };
            let _ = db.apply_alert_changes(&changes, now);
        }
        self.checks.history.remove(id);
        self.checks.last.remove(id);
        cx.notify();
    }

    pub fn toggle_check_alert(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(c) = self.checks.defs.iter_mut().find(|c| c.id == id) {
            c.alert = !c.alert;
        }
        self.save_checks();
        cx.notify();
    }

    /// Runs one check now, in the background.
    pub fn run_check(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(check) = self.checks.defs.iter().find(|c| c.id == id).cloned() else { return };
        if !self.checks.running.insert(check.id.clone()) {
            return;
        }
        self.checks.last_run.insert(check.id.clone(), chrono::Utc::now().timestamp());
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let c2 = check.clone();
            let outcome = cx.background_executor().spawn(async move { checks::run(&c2) }).await;
            let _ = entity.update(cx, |this, cx| {
                let now = chrono::Utc::now().timestamp();
                this.checks.running.remove(&check.id);
                let fails = this.checks.fails.entry(check.id.clone()).or_insert(0);
                *fails = if outcome.ok { 0 } else { *fails + 1 };
                let fails = *fails;
                if let Ok(db) = this.vault.db().lock() {
                    let _ = db.insert_check_result(&check.id, now, outcome.ok, outcome.latency_ms, &outcome.detail);
                    let open = db.list_alerts(i64::MAX).unwrap_or_default();
                    let _ = db.apply_alert_changes(&checks::alert_changes(&open, &check, &alert_server(&check), &outcome, fails, now), now);
                }
                let h = this.checks.history.entry(check.id.clone()).or_default();
                h.push((now, outcome.ok, outcome.latency_ms));
                h.retain(|(ts, _, _)| now - ts < 7 * 86_400);
                this.checks.last.insert(check.id.clone(), outcome);
                cx.notify();
            });
        })
        .detach();
    }

    /// Every poll: the checks that are due.
    pub fn checks_tick(&mut self, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        let due: Vec<String> = self.checks.defs.iter().filter(|c| now - self.checks.last_run.get(&c.id).copied().unwrap_or(0) >= c.every_secs).map(|c| c.id.clone()).collect();
        for id in due {
            self.run_check(&id, cx);
        }
        if now - self.checks.last_prune >= 3600 {
            self.checks.last_prune = now;
            if let Ok(db) = self.vault.db().lock() {
                let _ = db.prune_check_results(now - checks::KEEP_SECS, None);
            }
        }
    }
}
