//! Fleet Setup → DRIFT (ERR-143): baselined files re-read on every server,
//! drift raised as alerts, brought back or accepted, and a fleet-wide
//! search through every config file Crow has read.

use std::collections::{HashMap, HashSet};

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::fleet_run::{record_step, RunJob};
use super::CrowApp;
use crate::config::{drift, history};
use crate::host::host_for;
use crate::vault::{ChangeRecord, BASELINE_FLEET};
use crate::views::fleet::run::FleetRun;

/// One line (or file name) that matched a search.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchHit {
    pub server_id: String,
    pub server: String,
    pub path: String,
    /// (line number, the line) when the match is inside the file.
    pub line: Option<(usize, String)>,
}

/// Up to this many hits are shown.
pub const MAX_HITS: usize = 200;

/// Finds `query` (case-insensitive) in the file paths and contents Crow
/// has recorded: `files` are (server id, server name, path, content).
pub fn search(files: &[(String, String, String, Option<String>)], query: &str) -> Vec<SearchHit> {
    let q = query.trim().to_lowercase();
    if q.len() < 2 {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for (id, name, path, content) in files {
        let hit = |line| SearchHit { server_id: id.clone(), server: name.clone(), path: path.clone(), line };
        if path.to_lowercase().contains(&q) {
            hits.push(hit(None));
        }
        for (i, l) in content.as_deref().unwrap_or("").lines().enumerate() {
            if l.to_lowercase().contains(&q) {
                hits.push(hit(Some((i + 1, l.trim().to_string()))));
                if hits.len() >= MAX_HITS {
                    return hits;
                }
            }
        }
    }
    hits
}

#[derive(Default)]
pub struct DriftPageState {
    pub checking: HashSet<String>,
    pub last_check: i64,
    /// (server id, path) whose diff is shown.
    pub selected: Option<(String, String)>,
    pub search: Option<Entity<InputState>>,
    pub hits: Vec<SearchHit>,
    pub searched: String,
    pub message: Option<String>,
    /// A search hit opened on another server: selected once its configs load.
    pub open_path: Option<String>,
    _subs: Vec<Subscription>,
}

impl CrowApp {
    pub fn ensure_drift_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.drift_page.search.is_some() {
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("a file (sshd_config) or a setting (PermitRootLogin) on any server…"));
        let sub = cx.subscribe(&input, |this, input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::PressEnter { .. }) {
                let q = input.read(cx).value().to_string();
                this.search_fleet_configs(&q, cx);
            }
        });
        self.drift_page.search = Some(input);
        self.drift_page._subs.push(sub);
    }

    /// Searches every config file Crow has recorded, on every server; no
    /// server is contacted.
    pub fn search_fleet_configs(&mut self, query: &str, cx: &mut Context<Self>) {
        self.drift_page.searched = query.trim().to_string();
        let key = self.vault.key().cloned();
        let names: HashMap<&str, &str> = self.fleet.servers.iter().map(|s| (s.id.as_str(), s.name.as_str())).collect();
        let files: Vec<(String, String, String, Option<String>)> = self
            .vault
            .db()
            .lock()
            .ok()
            .and_then(|db| db.latest_config_revisions().ok())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| {
                let name = names.get(r.server_id.as_str())?.to_string();
                let content = history::open_content(&r, key.as_ref());
                Some((r.server_id, name, r.path, content))
            })
            .collect();
        self.drift_page.hits = search(&files, query);
        cx.notify();
    }

    /// Opens a hit: that server's Config screen at that file.
    pub fn open_search_hit(&mut self, hit: &SearchHit, cx: &mut Context<Self>) {
        let same = self.fleet.active_tab_id == hit.server_id && self.configs.server_id.as_deref() == Some(hit.server_id.as_str());
        self.switch_tab(&hit.server_id, cx);
        self.set_view("config", cx);
        if same {
            if let Some(name) = self.configs.states.iter().find(|(_, st)| st.path.to_string_lossy() == hit.path).map(|(n, _)| n.clone()) {
                self.select_managed_file(&name, cx);
            }
        } else {
            self.drift_page.open_path = Some(hit.path.clone());
        }
    }

    /// Every 15 minutes (or now, with `force`): re-reads each server's
    /// baselined files, then raises or resolves drift alerts.
    pub fn check_drift(&mut self, force: bool, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        if !force && now - self.drift_page.last_check < drift::DRIFT_EVERY_SECS {
            return;
        }
        let baselines = self.vault.db().lock().ok().and_then(|db| db.list_config_baselines().ok()).unwrap_or_default();
        if baselines.is_empty() {
            self.drift_page.last_check = now;
            return;
        }
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| self.fleet.health(s).is_ok() && !self.drift_page.checking.contains(&s.id)).cloned().collect();
        if servers.is_empty() {
            return;
        }
        self.drift_page.last_check = now;
        let key = self.vault.key().cloned();
        for srv in servers {
            let paths = drift::paths_for(&baselines, &srv.group_name);
            if paths.is_empty() {
                continue;
            }
            self.drift_page.checking.insert(srv.id.clone());
            let (db, key, id) = (self.vault.db(), key.clone(), srv.id.clone());
            cx.spawn(async move |entity, cx| {
                let (read, failed) = cx.background_executor().spawn(async move { drift::read_and_record(host_for(&srv).as_ref(), &db, key.as_ref(), &srv.id, &paths) }).await;
                let _ = entity.update(cx, |this, cx| {
                    this.drift_page.checking.remove(&id);
                    if !failed.is_empty() {
                        this.drift_page.message = Some(format!("couldn't read {} on a server: {}", failed[0].0, failed[0].1));
                    }
                    this.refresh_drift();
                    let checked: HashMap<String, Vec<String>> = [(id, read)].into();
                    this.apply_drift_alerts(&checked);
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    fn apply_drift_alerts(&mut self, checked: &HashMap<String, Vec<String>>) {
        let now = chrono::Utc::now().timestamp();
        let entries = self.fleet.drift.clone().unwrap_or_default();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let open = db.list_alerts(i64::MAX).unwrap_or_default();
        let _ = db.apply_alert_changes(&drift::alert_changes(&open, &entries, checked, now), now);
    }

    pub fn drift_tick(&mut self, cx: &mut Context<Self>) {
        if self.vault.status() != crate::vault::VaultStatus::Locked {
            self.check_drift(false, cx);
        }
    }

    /// (baseline, server copy) of the selected drift entry, when both
    /// contents were kept.
    pub fn drift_selected_texts(&self) -> Option<(String, String)> {
        let (sid, path) = self.drift_page.selected.as_ref()?;
        let entry = self.fleet.drift.iter().flatten().find(|e| &e.server_id == sid && &e.path == path)?;
        let key = self.vault.key();
        let db = self.vault.db();
        let db = db.lock().ok()?;
        let baselines = db.list_config_baselines().ok()?;
        let b = baselines.iter().find(|b| &b.path == path && b.scope == entry.scope)?;
        let base = history::open_content(&db.get_config_revision(&b.revision_id).ok()??, key)?;
        let now = history::open_content(&db.list_config_revisions(sid, path).ok()?.pop()?, key)?;
        Some((base, now))
    }

    pub fn select_drift(&mut self, server_id: &str, path: &str, cx: &mut Context<Self>) {
        self.drift_page.selected = Some((server_id.to_string(), path.to_string()));
        cx.notify();
    }

    /// Pushes the baseline back to `server` (or every drifted server) for
    /// `path`, on the fleet runner.
    pub fn plan_bring_back(&mut self, path: &str, server: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh_drift();
        let key = self.vault.key().cloned();
        let author = self.default_author();
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<(String, String)>) = (Vec::new(), Vec::new(), Vec::new());
        {
            let db = self.vault.db();
            let Ok(guard) = db.lock() else { return };
            let baselines = guard.list_config_baselines().unwrap_or_default();
            for e in self.fleet.drift.iter().flatten().filter(|e| e.drift.is_drift() && e.path == path && server.is_none_or(|s| s == e.server_id)) {
                let Some(srv) = self.fleet.servers.iter().find(|s| s.id == e.server_id).cloned() else { continue };
                let content = baselines.iter().find(|b| b.path == e.path && b.scope == e.scope).and_then(|b| guard.get_config_revision(&b.revision_id).ok().flatten()).and_then(|rev| history::open_content(&rev, key.as_ref()));
                let Some(baseline) = content else {
                    excluded.push((srv.name.clone(), "the baseline's content wasn't kept (only its hash), so there's nothing to push".into()));
                    continue;
                };
                if !self.fleet.health(&srv).is_ok() {
                    excluded.push((srv.name.clone(), "not reachable right now".into()));
                    continue;
                }
                let what = format!("put {path} back to the {} baseline", if e.scope == BASELINE_FLEET { "fleet".to_string() } else { e.scope.clone() });
                steps.push(FleetRun::step(&srv.id, &srv.name, what.clone()));
                let (db, key, author, path) = (self.vault.db(), key.clone(), author.clone(), path.to_string());
                jobs.push(Box::new(move || {
                    let host = host_for(&srv);
                    let target = crate::config::push::PushTarget { server_id: &srv.id, server_name: &srv.name, path: &path, baseline: &baseline, author: &author, login_user: &srv.login_user, message: "Brought back to the baseline", context: "drift: bring back" };
                    let result = crate::config::push::push_baseline(host.as_ref(), &db, key.as_ref(), &target);
                    record_step(&db, &srv, "config.drift_restore", &what, &result);
                    result
                }));
            }
        }
        let n = steps.len();
        self.drift_page.message = None;
        // After the run, read the files again so the alerts resolve.
        self.drift_page.last_check = 0;
        self.open_fleet_run(FleetRun::new(format!("BRING BACK {path} · {n} SERVER{}", if n == 1 { "" } else { "S" }), "RESTORE", steps, excluded), jobs, window, cx);
    }

    /// Accepts a server's copy: its latest revision becomes the baseline
    /// for the scope it drifted from.
    pub fn accept_drift(&mut self, server_id: &str, path: &str, cx: &mut Context<Self>) {
        let Some(entry) = self.fleet.drift.iter().flatten().find(|e| e.server_id == server_id && e.path == path).cloned() else { return };
        let name = self.fleet.servers.iter().find(|s| s.id == server_id).map(|s| s.name.clone()).unwrap_or_else(|| server_id.to_string());
        let author = self.default_author();
        let result = self.vault.db().lock().map_err(|_| "the vault is busy".to_string()).and_then(|db| {
            let rev = db.list_config_revisions(server_id, path).map_err(|e| e.to_string())?.pop().ok_or("Crow has no copy of it from that server")?;
            if rev.sealed.is_none() {
                return Err("only the hash of that copy was kept, so it couldn't be pushed anywhere: open the vault and read it again".into());
            }
            db.set_config_baseline(path, &entry.scope, &rev.id, &author).map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().to_rfc3339();
            db.insert_change_record(&ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: server_id.into(),
                server_name: name.clone(),
                action_kind: "config.baseline".into(),
                target: path.into(),
                before_state: format!("accepted {name}'s copy as the {} baseline", crate::app::configs::scope_label(&entry.scope)),
                after_state: Some(format!("sha256:{}", rev.sha256)),
                blast_radius: Some("drift: accept".into()),
                outcome: "success".into(),
                started_at: now.clone(),
                completed_at: Some(now),
            })
            .map_err(|e| e.to_string())
        });
        self.drift_page.message = Some(match result {
            Ok(()) => format!("{name}'s {path} is now the baseline for {}. Other servers are compared with it from now on.", crate::app::configs::scope_label(&entry.scope)),
            Err(e) => format!("Not accepted: {e}"),
        });
        self.refresh_drift();
        // Resolve what's now matching, everywhere this baseline applies.
        let checked: HashMap<String, Vec<String>> = self.fleet.drift.iter().flatten().filter(|e| e.path == path).map(|e| (e.server_id.clone(), vec![path.to_string()])).collect();
        self.apply_drift_alerts(&checked);
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::search;

    #[test]
    fn search_finds_files_and_settings_across_servers() {
        let files = vec![
            ("a".to_string(), "web-1".to_string(), "/etc/ssh/sshd_config".to_string(), Some("Port 22\nPermitRootLogin no\n".to_string())),
            ("b".to_string(), "web-2".to_string(), "/etc/ssh/sshd_config".to_string(), Some("Port 22\n  PermitRootLogin yes\n".to_string())),
            ("b".to_string(), "web-2".to_string(), "/etc/nginx/nginx.conf".to_string(), None),
        ];
        let hits = search(&files, "permitroot");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].line, Some((2, "PermitRootLogin yes".to_string())));
        let by_name = search(&files, "nginx");
        assert_eq!((by_name.len(), by_name[0].line.clone()), (1, None), "a path match, content unknown");
        assert!(search(&files, "x").is_empty(), "one letter finds everything: wait for more");
    }
}
