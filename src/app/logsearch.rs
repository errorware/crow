//! Fleet Setup → LOG SEARCH (ERR-147): the journal of a group's servers,
//! searched at once; saved searches that alert; a clanker summary.

use std::collections::HashSet;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::patching::Target;
use super::CrowApp;
use crate::host::host_for;
use crate::journal::reader::read_journal;
use crate::journal::JournalTimeRange;
use crate::logsearch::{self, FleetHit, SavedSearch, ServerResult};

pub struct LogSearchState {
    pub text: Option<Entity<InputState>>,
    pub unit: Option<Entity<InputState>>,
    pub name: Option<Entity<InputState>>,
    pub priority: Option<u8>,
    pub range: JournalTimeRange,
    pub limit: usize,
    pub target: Target,
    pub running: HashSet<String>,
    pub results: Vec<ServerResult>,
    pub hits: Vec<FleetHit>,
    pub searched: String,
    pub summarizing: bool,
    pub summary: Option<Result<(String, String), String>>,
    pub saved: Vec<SavedSearch>,
    pub last_alert_run: i64,
    pub message: Option<(bool, String)>,
    _subs: Vec<Subscription>,
}

impl Default for LogSearchState {
    fn default() -> Self {
        Self { text: None, unit: None, name: None, priority: None, range: JournalTimeRange::Last1h, limit: 200, target: Target::All, running: HashSet::new(), results: Vec::new(), hits: Vec::new(), searched: String::new(), summarizing: false, summary: None, saved: Vec::new(), last_alert_run: 0, message: None, _subs: Vec::new() }
    }
}

impl CrowApp {
    pub fn load_log_searches(&mut self) {
        self.log_search.saved = self.vault.db().lock().ok().and_then(|db| db.flag(logsearch::SAVED_FLAG)).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
    }

    fn save_log_searches(&self) {
        if let (Ok(db), Ok(json)) = (self.vault.db().lock(), serde_json::to_string(&self.log_search.saved)) {
            let _ = db.set_flag(logsearch::SAVED_FLAG, &json);
        }
    }

    pub fn ensure_log_search_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.log_search.text.is_some() {
            return;
        }
        let text = cx.new(|cx| InputState::new(window, cx).placeholder("text or regex, e.g. Out of memory|segfault"));
        let unit = cx.new(|cx| InputState::new(window, cx).placeholder("unit (optional), e.g. nginx.service"));
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("a name to save it as"));
        let subs = [&text, &unit]
            .into_iter()
            .map(|i| {
                cx.subscribe(i, |this, _i, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.run_log_search(cx);
                    }
                })
            })
            .collect();
        self.log_search.text = Some(text);
        self.log_search.unit = Some(unit);
        self.log_search.name = Some(name);
        self.log_search._subs = subs;
    }

    fn current_search(&self, cx: &App) -> SavedSearch {
        let v = |i: &Option<Entity<InputState>>| i.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        SavedSearch {
            id: String::new(),
            name: v(&self.log_search.name),
            text: v(&self.log_search.text),
            unit: v(&self.log_search.unit),
            priority: self.log_search.priority,
            group: match &self.log_search.target {
                Target::Group(g) => g.clone(),
                _ => String::new(),
            },
            alert: false,
            level: "WARN".into(),
        }
    }

    pub fn set_log_priority(&mut self, p: Option<u8>, cx: &mut Context<Self>) {
        self.log_search.priority = p;
        cx.notify();
    }

    pub fn set_log_range(&mut self, r: JournalTimeRange, cx: &mut Context<Self>) {
        self.log_search.range = r;
        cx.notify();
    }

    pub fn set_log_limit(&mut self, n: usize, cx: &mut Context<Self>) {
        self.log_search.limit = n;
        cx.notify();
    }

    pub fn set_log_target(&mut self, t: Target, cx: &mut Context<Self>) {
        self.log_search.target = t;
        cx.notify();
    }

    /// Runs the search on every reachable server of the target at once.
    pub fn run_log_search(&mut self, cx: &mut Context<Self>) {
        let search = self.current_search(cx);
        self.start_search(search, cx);
    }

    fn start_search(&mut self, search: SavedSearch, cx: &mut Context<Self>) {
        let query = search.query(self.log_search.range, self.log_search.limit);
        let target = self.log_search.target.clone();
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| target.includes(s) && self.fleet.health(s).is_ok()).cloned().collect();
        if servers.is_empty() {
            self.log_search.message = Some((false, format!("No reachable server in {}.", target.label())));
            cx.notify();
            return;
        }
        self.log_search.results.clear();
        self.log_search.hits.clear();
        self.log_search.summary = None;
        self.log_search.message = None;
        self.log_search.searched = if search.text.is_empty() { "everything".into() } else { search.text.clone() };
        for srv in servers {
            self.log_search.running.insert(srv.id.clone());
            let (q, id, name) = (query.clone(), srv.id.clone(), srv.name.clone());
            cx.spawn(async move |entity, cx| {
                let lines = cx.background_executor().spawn(async move { read_journal(host_for(&srv).as_ref(), &q).ok_or_else(|| "couldn't read its journal (no journalctl, or unreachable)".to_string()) }).await;
                let _ = entity.update(cx, |this, cx| {
                    this.log_search.running.remove(&id);
                    this.log_search.results.push((id, name, lines));
                    this.log_search.hits = logsearch::merge(&this.log_search.results);
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub fn save_log_search(&mut self, cx: &mut Context<Self>) {
        let mut s = self.current_search(cx);
        if s.name.is_empty() {
            self.log_search.message = Some((false, "Name the search to save it.".into()));
            cx.notify();
            return;
        }
        if s.text.is_empty() && s.unit.is_empty() && s.priority.is_none() {
            self.log_search.message = Some((false, "A saved search needs text, a unit or a priority: one matching everything would always alert.".into()));
            cx.notify();
            return;
        }
        s.id = format!("ls{}", chrono::Utc::now().timestamp_millis());
        self.log_search.message = Some((true, format!("Saved \"{}\". Switch ALERT on to be told when it matches.", s.name)));
        self.log_search.saved.push(s);
        self.save_log_searches();
        cx.notify();
    }

    pub fn run_saved_search(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(s) = self.log_search.saved.iter().find(|s| s.id == id).cloned() else { return };
        let set = |i: &Option<Entity<InputState>>, v: &str, window: &mut Window, cx: &mut Context<Self>| {
            if let Some(i) = i {
                i.update(cx, |x, cx| x.set_value(v.to_string(), window, cx));
            }
        };
        set(&self.log_search.text.clone(), &s.text, window, cx);
        set(&self.log_search.unit.clone(), &s.unit, window, cx);
        set(&self.log_search.name.clone(), &s.name, window, cx);
        self.log_search.priority = s.priority;
        self.log_search.target = if s.group.is_empty() { Target::All } else { Target::Group(s.group.clone()) };
        self.start_search(s, cx);
    }

    /// Cycles a saved search's alert: off → WARN → CRIT → off.
    pub fn cycle_saved_alert(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(s) = self.log_search.saved.iter_mut().find(|s| s.id == id) {
            (s.alert, s.level) = match (s.alert, s.level.as_str()) {
                (false, _) => (true, "WARN".into()),
                (true, "WARN") => (true, "CRIT".into()),
                _ => (false, "WARN".into()),
            };
        }
        self.save_log_searches();
        self.log_search.last_alert_run = 0;
        cx.notify();
    }

    pub fn delete_saved_search(&mut self, id: &str, cx: &mut Context<Self>) {
        self.log_search.saved.retain(|s| s.id != id);
        self.save_log_searches();
        // Its alerts go with it.
        let kind = format!("logsearch:{id}");
        let now = chrono::Utc::now().timestamp();
        if let Ok(db) = self.vault.db().lock() {
            let open = db.list_alerts(i64::MAX).unwrap_or_default();
            let changes = crate::metrics::alerts::AlertChanges { resolved: open.iter().filter(|a| a.kind == kind && a.resolved_at.is_none()).map(|a| a.id.clone()).collect(), ..Default::default() };
            let _ = db.apply_alert_changes(&changes, now);
        }
        cx.notify();
    }

    /// Every 10 minutes: alerting searches over the last 15 minutes.
    pub fn log_alert_tick(&mut self, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        if now - self.log_search.last_alert_run < logsearch::ALERT_EVERY_SECS || self.vault.status() == crate::vault::VaultStatus::Locked {
            return;
        }
        let searches: Vec<SavedSearch> = self.log_search.saved.iter().filter(|s| s.alert).cloned().collect();
        self.log_search.last_alert_run = now;
        if searches.is_empty() {
            return;
        }
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| self.fleet.health(s).is_ok()).cloned().collect();
        for search in searches {
            for srv in servers.iter().filter(|s| search.group.is_empty() || s.group_name.trim() == search.group).cloned() {
                let (search, id) = (search.clone(), srv.id.clone());
                let q = search.query(JournalTimeRange::Last15m, 50);
                cx.spawn(async move |entity, cx| {
                    let lines = cx.background_executor().spawn(async move { read_journal(host_for(&srv).as_ref(), &q) }).await;
                    let _ = entity.update(cx, |this, _cx| {
                        let Some(lines) = lines else { return };
                        let now = chrono::Utc::now().timestamp();
                        let db = this.vault.db();
                        let Ok(db) = db.lock() else { return };
                        let open = db.list_alerts(i64::MAX).unwrap_or_default();
                        let sample = lines.first().map(|e| e.message.as_str());
                        let _ = db.apply_alert_changes(&logsearch::alert_changes(&open, &search, &id, lines.len(), sample, now), now);
                    });
                })
                .detach();
            }
        }
    }

    /// Sends the matching lines (only those) to the clanker for a summary.
    pub fn summarize_log_search(&mut self, cx: &mut Context<Self>) {
        if self.log_search.hits.is_empty() || self.log_search.summarizing {
            return;
        }
        let Some((primary, backup)) = self.ai_providers() else {
            self.log_search.summary = Some(Err("No AI provider has a key yet. Add one in Settings → Clankers (AI).".into()));
            cx.notify();
            return;
        };
        let lines = logsearch::summary_input(&self.log_search.hits, 300);
        let context = format!("journal lines matching \"{}\" across {} servers ({}), oldest first, each prefixed with time, server and unit", self.log_search.searched, self.log_search.results.len(), self.log_search.range.label());
        self.log_search.summarizing = true;
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
                this.log_search.summarizing = false;
                this.log_search.summary = Some(answer.map(|a| (a.text, a.provider_label)));
                cx.notify();
            });
        })
        .detach();
    }
}
