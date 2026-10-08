//! Fleet Setup → PATCHING (ERR-141): who's behind on updates, each group's
//! maintenance window, and runs that apply updates and reboot in order.

use std::collections::{BTreeMap, HashMap, HashSet};

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::fleet_run::{record_step, RunJob};
use super::CrowApp;
use crate::host::{host_for, transport_kind, TransportKind};
use crate::patching::{self, Scope, Summary, Window};
use crate::views::fleet::run::{FleetRun, StepOutcome};

/// Which servers a run covers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Target {
    #[default]
    All,
    Group(String),
    /// (id, name): one server, from its Updates card.
    Server(String, String),
}

impl Target {
    pub fn includes(&self, s: &crate::vault::ServerRecord) -> bool {
        match self {
            Target::All => true,
            Target::Group(g) => s.group_name.trim() == g,
            Target::Server(id, _) => s.id == *id,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Target::All => "every server".into(),
            Target::Group(g) => format!("group {g}"),
            Target::Server(_, name) => name.clone(),
        }
    }
}

#[derive(Default)]
pub struct PatchingState {
    pub summaries: HashMap<String, Summary>,
    pub scanning: HashSet<String>,
    pub last_scan: i64,
    pub windows: BTreeMap<String, Window>,
    pub window_inputs: HashMap<String, Entity<InputState>>,
    pub window_errors: HashMap<String, String>,
    pub target: Target,
    pub reboot_after: bool,
    /// A patch run is on the runner: read the fleet again when it ends.
    pub rescan_after_run: bool,
    _subs: Vec<Subscription>,
}

impl CrowApp {
    /// The last scan and the windows, from the vault.
    pub fn load_patching(&mut self) {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        if let Some(s) = db.flag(patching::SUMMARIES_FLAG).and_then(|j| serde_json::from_str::<HashMap<String, Summary>>(&j).ok()) {
            self.patching.last_scan = s.values().map(|x| x.checked_at).max().unwrap_or(0);
            self.patching.summaries = s;
        }
        self.patching.windows = self
            .fleet
            .groups
            .iter()
            .filter_map(|g| db.flag(&patching::window_flag(g)).and_then(|w| Window::parse(&w).ok()).map(|w| (g.clone(), w)))
            .collect();
    }

    fn save_summaries(&self) {
        if let (Ok(db), Ok(json)) = (self.vault.db().lock(), serde_json::to_string(&self.patching.summaries)) {
            let _ = db.set_flag(patching::SUMMARIES_FLAG, &json);
        }
    }

    /// Reads reachable servers' pending updates in the background: those
    /// never read or read more than 6 h ago, or all of them with `force`.
    pub fn scan_fleet_updates(&mut self, force: bool, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        let stale = |id: &str| self.patching.summaries.get(id).is_none_or(|x| now - x.checked_at >= patching::SCAN_EVERY_SECS);
        let targets: Vec<_> = self
            .fleet
            .servers
            .iter()
            .filter(|s| self.fleet.health(s).is_ok() && !self.patching.scanning.contains(&s.id) && (force || stale(&s.id)))
            .cloned()
            .collect();
        for srv in targets {
            self.patching.scanning.insert(srv.id.clone());
            let id = srv.id.clone();
            cx.spawn(async move |entity, cx| {
                let report = cx.background_executor().spawn(async move { patching::read_report(host_for(&srv).as_ref()) }).await;
                let _ = entity.update(cx, |this, cx| {
                    let now = chrono::Utc::now().timestamp();
                    this.patching.scanning.remove(&id);
                    this.patching.last_scan = now;
                    this.patching.summaries.insert(id, match report {
                        Ok(r) => Summary::from_report(&r, now),
                        Err(e) => Summary::failed(e, now),
                    });
                    if this.patching.scanning.is_empty() {
                        this.save_summaries();
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    /// Every poll: keeps the fleet's reading fresh.
    pub fn patching_tick(&mut self, cx: &mut Context<Self>) {
        if self.vault.status() != crate::vault::VaultStatus::Locked && !self.fleet.servers.is_empty() {
            self.scan_fleet_updates(false, cx);
        }
    }

    /// When a patch run ends: read the fleet again.
    pub fn after_patch_run(&mut self, cx: &mut Context<Self>) {
        if std::mem::take(&mut self.patching.rescan_after_run) {
            self.scan_fleet_updates(true, cx);
        }
    }

    /// One input per group for its window, made when the page shows.
    pub fn ensure_window_inputs(&mut self, window: &mut gpui_kit::Window, cx: &mut Context<Self>) {
        for g in self.fleet.groups.clone() {
            if self.patching.window_inputs.contains_key(&g) {
                continue;
            }
            let value = self.patching.windows.get(&g).map(|w| w.describe()).unwrap_or_default();
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("any time · e.g. Sun 02-05").default_value(value));
            let group = g.clone();
            let sub = cx.subscribe(&input, move |this, input, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                    let text = input.read(cx).value().to_string();
                    this.save_window(&group, &text, cx);
                }
            });
            self.patching.window_inputs.insert(g, input);
            self.patching._subs.push(sub);
        }
    }

    /// Saves a group's window; blank removes it (any time).
    pub fn save_window(&mut self, group: &str, text: &str, cx: &mut Context<Self>) {
        let text = text.trim();
        let parsed = if text.is_empty() { Ok(None) } else { Window::parse(text).map(Some) };
        match parsed {
            Ok(w) => {
                self.patching.window_errors.remove(group);
                if let Ok(db) = self.vault.db().lock() {
                    let _ = db.set_flag(&patching::window_flag(group), &w.map(|w| w.describe()).unwrap_or_default());
                }
                match w {
                    Some(w) => self.patching.windows.insert(group.to_string(), w),
                    None => self.patching.windows.remove(group),
                };
            }
            Err(e) => {
                self.patching.window_errors.insert(group.to_string(), e);
            }
        }
        cx.notify();
    }

    pub fn set_patch_target(&mut self, target: Target, cx: &mut Context<Self>) {
        self.patching.target = target;
        cx.notify();
    }

    pub fn toggle_reboot_after(&mut self, cx: &mut Context<Self>) {
        self.patching.reboot_after = !self.patching.reboot_after;
        cx.notify();
    }

    /// Servers a run may touch now, and the ones left out with why.
    fn patch_candidates(&self, target: &Target) -> (Vec<crate::vault::ServerRecord>, Vec<(String, String)>) {
        let mut ok = Vec::new();
        let mut excluded = Vec::new();
        for s in self.fleet.servers.iter().filter(|s| target.includes(s)) {
            let why = if transport_kind(s) == TransportKind::Local {
                Some("the machine Crow runs on: patch it yourself".to_string())
            } else if !self.fleet.health(s).is_ok() {
                Some("not reachable right now".to_string())
            } else {
                patching::outside_window(s.group_name.trim(), &self.patching.windows)
            };
            match why {
                Some(w) => excluded.push((s.name.clone(), w)),
                None => ok.push(s.clone()),
            }
        }
        (ok, excluded)
    }

    /// Plans applying `scope` updates to the target's servers, the exact
    /// packages from the last reading, then (if asked) rebooting those
    /// that need it, bastions after the servers behind them.
    pub fn plan_patch(&mut self, scope: Scope, window: &mut gpui_kit::Window, cx: &mut Context<Self>) {
        let target = self.patching.target.clone();
        let reboot_after = self.patching.reboot_after;
        let (servers, mut excluded) = self.patch_candidates(&target);
        let ordered = if reboot_after { patching::reboot_order(servers, &self.fleet.servers) } else { servers };
        let (mut steps, mut jobs): (Vec<_>, Vec<RunJob>) = (Vec::new(), Vec::new());
        for srv in ordered {
            let Some(summary) = self.patching.summaries.get(&srv.id).cloned() else {
                excluded.push((srv.name.clone(), "its updates haven't been read yet: REFRESH first".into()));
                continue;
            };
            if let Some(e) = &summary.error {
                excluded.push((srv.name.clone(), format!("couldn't read its updates: {e}")));
                continue;
            }
            let packages = summary.packages(scope);
            if packages.is_empty() {
                excluded.push((srv.name.clone(), format!("no {} pending", scope.label())));
                continue;
            }
            let reboot = if reboot_after {
                match self.reboot_work(&srv) {
                    Ok(r) => Some(r),
                    Err(why) => {
                        excluded.push((srv.name.clone(), format!("can't be rebooted from here ({why}), so it's left out of a run that reboots")));
                        continue;
                    }
                }
            } else {
                None
            };
            let shown: Vec<&str> = packages.iter().take(6).map(String::as_str).collect();
            let more = packages.len().saturating_sub(shown.len());
            let what = format!(
                "{} {} {}: {}{}{}",
                summary.manager,
                packages.len(),
                if packages.len() == 1 { "package" } else { "packages" },
                shown.join(", "),
                if more > 0 { format!(" +{more}") } else { String::new() },
                if reboot.is_some() { " · then reboot if needed" } else { "" }
            );
            steps.push(FleetRun::step(&srv.id, &srv.name, what.clone()));
            let db = self.vault.db();
            let manager = summary.manager.clone();
            jobs.push(Box::new(move || {
                let host = host_for(&srv);
                let result = patching::apply(host.as_ref(), &manager, &packages).and_then(|applied| {
                    let done = applied.describe();
                    match reboot {
                        Some((_, work)) if applied.reboot_required => match work()? {
                            StepOutcome::Done(m) | StepOutcome::Skipped(m) => Ok(StepOutcome::Done(format!("{done}; rebooted: {m}"))),
                        },
                        _ => Ok(StepOutcome::Done(done)),
                    }
                });
                record_step(&db, &srv, "patch.apply", &what, &result);
                result
            }));
        }
        let n = steps.len();
        self.patching.rescan_after_run = n > 0;
        let title = format!("APPLY {} · {} · {n} SERVER{}", scope.label().to_uppercase(), target.label().to_uppercase(), if n == 1 { "" } else { "S" });
        self.open_fleet_run(FleetRun::new(title, "PATCH", steps, excluded), jobs, window, cx);
    }

    /// Plans rebooting the target's servers that say they need it, in
    /// order, inside their windows.
    pub fn plan_reboot_needed(&mut self, window: &mut gpui_kit::Window, cx: &mut Context<Self>) {
        let target = self.patching.target.clone();
        let (servers, mut excluded) = self.patch_candidates(&target);
        let needing: Vec<_> = servers.into_iter().filter(|s| self.patching.summaries.get(&s.id).is_some_and(|x| x.reboot_required)).collect();
        let (mut steps, mut jobs): (Vec<_>, Vec<RunJob>) = (Vec::new(), Vec::new());
        for srv in patching::reboot_order(needing, &self.fleet.servers) {
            match self.reboot_work(&srv) {
                Err(why) => excluded.push((srv.name.clone(), why)),
                Ok((how, work)) => {
                    steps.push(FleetRun::step(&srv.id, &srv.name, how.clone()));
                    let db = self.vault.db();
                    jobs.push(Box::new(move || {
                        let result = work();
                        record_step(&db, &srv, "fleet.reboot", &how, &result);
                        result
                    }));
                }
            }
        }
        let n = steps.len();
        self.patching.rescan_after_run = n > 0;
        let title = format!("REBOOT WHERE NEEDED · {} · {n} SERVER{}", target.label().to_uppercase(), if n == 1 { "" } else { "S" });
        self.open_fleet_run(FleetRun::new(title, "REBOOT", steps, excluded), jobs, window, cx);
    }

    /// A single server's Updates card: apply its updates on the runner.
    pub fn plan_patch_server(&mut self, server_id: &str, scope: Scope, window: &mut gpui_kit::Window, cx: &mut Context<Self>) {
        // Fresh from the card: the overview's reading is the newest.
        if let Some(Ok(r)) = self.overview.security.updates.as_ref().filter(|_| self.overview.security.server_id.as_deref() == Some(server_id)) {
            self.patching.summaries.insert(server_id.to_string(), Summary::from_report(r, chrono::Utc::now().timestamp()));
        }
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == server_id) else { return };
        let saved = (self.patching.target.clone(), self.patching.reboot_after);
        self.patching.target = Target::Server(srv.id.clone(), srv.name.clone());
        self.patching.reboot_after = false;
        self.plan_patch(scope, window, cx);
        (self.patching.target, self.patching.reboot_after) = saved;
    }
}
