//! User plugins in Settings → Plugins (ERR-177): found in the plugins
//! folder, reviewed, then accepted or refused. Nothing of a plugin runs in
//! this stage; accepting only pins its files.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::plugin_host::folder::{self, Installed, Pin, Trust};
use crate::plugin_host::manifest::{BlockKind, Effect, FieldKind, Place, Runs};
use crate::plugin_host::local;
use crate::plugin_host::run::{self, RunResult, ServerValues, Values};

/// One run, ready to go on a background thread: lines to the sink, stops
/// when the flag is set.
type Job = Box<dyn FnOnce(&mut dyn FnMut(bool, &str), &AtomicBool) -> RunResult + Send>;

/// The run slot a plugin's setup uses.
pub const SETUP: &str = "__setup";
pub const HERE: &str = "__here";
use crate::vault::{ChangeRecord, PluginRun, ServerRecord, AUDIT_SERVER_ID};

/// The last run of one action on one server, and the one going, if any.
#[derive(Debug, Clone, Default)]
pub struct RunSlot {
    pub running: bool,
    pub last: Option<RunResult>,
    /// Someone pressed its button: the run goes in the audit log too.
    pub pressed: bool,
    /// Lines of the run going, as they arrive (true: stderr).
    pub live: Arc<Mutex<Vec<(bool, String)>>>,
    pub stop: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// A page reading what it shows.
    Auto,
    /// A button, or RUN on the confirm panel.
    Pressed,
    /// Another action's dry run.
    DryRun,
}

/// A change or danger action waiting on its confirm panel.
#[derive(Debug, Clone)]
pub struct PendingRun {
    pub key: RunKey,
    /// Values of choice and bool inputs.
    pub picked: BTreeMap<String, String>,
    /// The values the dry run last ran with: RUN needs the same.
    pub dry_ran_with: Option<BTreeMap<String, String>>,
    pub error: Option<String>,
    /// From a fleet page: RUN plans a fleet run; the dry run is on the canary.
    pub fleet: bool,
}

/// The log kept with a change: its output, at most 256 KB.
fn cap_log(r: &RunResult) -> String {
    let mut log = r.stdout.clone();
    if !r.stderr.trim().is_empty() {
        log.push_str("\n--- stderr ---\n");
        log.push_str(&r.stderr);
    }
    if log.len() > 256 * 1024 {
        let mut end = 256 * 1024;
        while !log.is_char_boundary(end) {
            end -= 1;
        }
        log.truncate(end);
    }
    log
}

/// (plugin id, action id, server id).
pub type RunKey = (String, String, String);

/// A plugin page's view id: `plugin:<plugin id>:<page id>`.
pub fn page_view(plugin: &str, page: &str) -> String {
    format!("plugin:{plugin}:{page}")
}

pub fn parse_page_view(view: &str) -> Option<(&str, &str)> {
    view.strip_prefix("plugin:")?.split_once(':')
}

#[derive(Default)]
pub struct UserPluginsState {
    pub installed: Vec<Installed>,
    pub scanning: bool,
    /// The plugin whose review is open.
    pub reviewing: Option<String>,
    /// ACCEPT was pressed once; the second press accepts.
    pub confirming: Option<String>,
    /// (ok, text) after accept, refuse, revoke or a failed folder.
    pub message: Option<(bool, String)>,
    pub runs: HashMap<RunKey, RunSlot>,
    /// The action whose output is shown under a page's buttons.
    pub shown_output: Option<RunKey>,
    /// Per plugin, the files its setup last succeeded with (their hash).
    pub setup_hashes: HashMap<String, String>,
    pub pending: Option<PendingRun>,
    /// The settings form under a review.
    pub settings: super::user_plugin_settings::SettingsForm,
    /// Where fleet pages' actions go.
    pub fleet: super::user_plugin_fleet::PluginFleetState,
    pub pending_inputs: Vec<(String, Entity<InputState>)>,
    pub keyword_input: Option<Entity<InputState>>,
}

impl UserPluginsState {
    pub fn get(&self, id: &str) -> Option<&Installed> {
        self.installed.iter().find(|p| p.id == id)
    }
}

impl UserPluginsState {
    pub fn slot(&self, plugin: &str, action: &str, server: &str) -> Option<&RunSlot> {
        self.runs.get(&(plugin.to_string(), action.to_string(), server.to_string()))
    }

    /// Fleet pages of accepted plugins: (plugin id, page id, label, icon name).
    pub fn fleet_pages(&self) -> Vec<(String, String, String, Option<String>)> {
        self.installed
            .iter()
            .filter(|p| p.usable())
            .filter_map(|p| p.report.manifest.as_ref().map(|m| (p, m)))
            .flat_map(|(p, m)| m.pages.iter().filter(|pg| pg.place == Place::Fleet).map(move |pg| (p.id.clone(), pg.id.clone(), pg.label.clone(), pg.icon.clone())))
            .collect()
    }

    /// Server pages of accepted plugins: (view id, label, icon name).
    pub fn server_pages(&self) -> Vec<(String, String, Option<String>)> {
        self.installed
            .iter()
            .filter(|p| p.usable())
            .filter_map(|p| p.report.manifest.as_ref().map(|m| (p, m)))
            .flat_map(|(p, m)| m.pages.iter().filter(|pg| pg.place == Place::Server).map(move |pg| (page_view(&p.id, &pg.id), pg.label.clone(), pg.icon.clone())))
            .collect()
    }
}

impl CrowApp {
    /// A plugin page opened: read what its blocks show, on the active server.
    pub fn open_plugin_page(&mut self, view: &str, cx: &mut Context<Self>) {
        self.active_view = view.to_string();
        self.refresh_plugin_page(cx);
    }

    /// Runs the read actions the page's status, kv and table blocks show.
    pub fn refresh_plugin_page(&mut self, cx: &mut Context<Self>) {
        let Some((plugin, page)) = parse_page_view(&self.active_view).map(|(a, b)| (a.to_string(), b.to_string())) else { return };
        let actions: Vec<String> = self
            .plugins
            .user
            .get(&plugin)
            .and_then(|p| p.report.manifest.as_ref())
            .and_then(|m| m.pages.iter().find(|pg| pg.id == page))
            .map(|pg| pg.blocks.iter().filter(|b| matches!(b.kind, BlockKind::Status | BlockKind::Kv | BlockKind::Table)).filter_map(|b| b.action.clone()).collect())
            .unwrap_or_default();
        let mut seen = Vec::new();
        for a in actions {
            if !seen.contains(&a) {
                self.start_plugin_run(&plugin, &a, BTreeMap::new(), RunMode::Auto, cx);
                seen.push(a);
            }
        }
        cx.notify();
    }

    /// A page's button. A read runs at once; a change or danger action opens
    /// its confirm panel: inputs, the dry run, then confirm.
    pub fn press_plugin_action(&mut self, plugin_id: &str, action_id: &str, cx: &mut Context<Self>) {
        let Some(server) = self.fleet.active_server() else { return };
        let action = self.plugins.user.get(plugin_id).and_then(|p| p.report.manifest.as_ref()).and_then(|m| m.actions.iter().find(|a| a.id == action_id)).cloned();
        let Some(action) = action else { return };
        if action.effect == Effect::Read {
            let inputs = action.inputs.iter().filter_map(|f| run::default_value(f).map(|d| (f.key.clone(), d))).collect();
            self.start_plugin_run(plugin_id, action_id, inputs, RunMode::Pressed, cx);
            return;
        }
        let picked = action.inputs.iter().filter(|f| !matches!(f.kind, FieldKind::Text | FieldKind::Number)).filter_map(|f| run::default_value(f).map(|d| (f.key.clone(), d))).collect();
        self.plugins.user.pending = Some(PendingRun { key: (plugin_id.to_string(), action_id.to_string(), server.id.clone()), picked, dry_ran_with: None, error: None, fleet: false });
        self.plugins.user.pending_inputs.clear();
        self.plugins.user.keyword_input = None;
        cx.notify();
    }

    pub fn cancel_plugin_run(&mut self, cx: &mut Context<Self>) {
        self.plugins.user.pending = None;
        self.plugins.user.pending_inputs.clear();
        self.plugins.user.keyword_input = None;
        cx.notify();
    }

    /// Text boxes for the confirm panel's text and number inputs and its keyword.
    pub fn ensure_plugin_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(action) = self.pending_action() else { return };
        if self.plugins.user.pending_inputs.is_empty() {
            for f in action.inputs.iter().filter(|f| matches!(f.kind, FieldKind::Text | FieldKind::Number)) {
                let default = run::default_value(f).unwrap_or_default();
                let state = cx.new(|cx| InputState::new(window, cx).placeholder(f.label.clone()).default_value(default));
                self.plugins.user.pending_inputs.push((f.key.clone(), state));
            }
        }
        if action.effect == Effect::Danger && self.plugins.user.keyword_input.is_none() {
            self.plugins.user.keyword_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder(action.keyword.clone().unwrap_or_else(|| "CONFIRM".into()))));
        }
    }

    pub fn pending_action(&self) -> Option<crate::plugin_host::manifest::Action> {
        let p = self.plugins.user.pending.as_ref()?;
        self.plugins.user.get(&p.key.0)?.report.manifest.as_ref()?.actions.iter().find(|a| a.id == p.key.1).cloned()
    }

    pub fn pick_plugin_input(&mut self, key: &str, value: String, cx: &mut Context<Self>) {
        if let Some(p) = self.plugins.user.pending.as_mut() {
            p.picked.insert(key.to_string(), value);
            p.error = None;
        }
        cx.notify();
    }

    /// The confirm panel's values, checked.
    fn pending_values(&self, cx: &App) -> Result<BTreeMap<String, String>, String> {
        let p = self.plugins.user.pending.as_ref().ok_or_else(String::new)?;
        let action = self.pending_action().ok_or_else(String::new)?;
        let mut out = BTreeMap::new();
        for f in &action.inputs {
            let raw = match self.plugins.user.pending_inputs.iter().find(|(k, _)| k == &f.key) {
                Some((_, input)) => input.read(cx).value().to_string(),
                None => p.picked.get(&f.key).cloned().unwrap_or_default(),
            };
            let v = run::check_value(f, &raw)?;
            if !v.is_empty() {
                out.insert(f.key.clone(), v);
            }
        }
        Ok(out)
    }

    /// DRY RUN: the action's dry run, with the values on screen.
    pub fn dry_run_pending(&mut self, cx: &mut Context<Self>) {
        let Some(action) = self.pending_action() else { return };
        let Some(dry) = action.dry_run.clone() else { return };
        let values = match self.pending_values(cx) {
            Ok(v) => v,
            Err(e) => {
                if let Some(p) = self.plugins.user.pending.as_mut() {
                    p.error = Some(e);
                }
                cx.notify();
                return;
            }
        };
        let (plugin, server_id) = self.plugins.user.pending.as_ref().map(|p| (p.key.0.clone(), p.key.2.clone())).unwrap_or_default();
        if let Some(p) = self.plugins.user.pending.as_mut() {
            p.dry_ran_with = Some(values.clone());
            p.error = None;
        }
        if let Some(server) = self.fleet.servers.iter().find(|s| s.id == server_id).cloned() {
            self.start_plugin_run_on(server, &plugin, &dry, values, RunMode::DryRun, cx);
        }
    }

    /// RUN on the confirm panel: after the dry run (if there is one) with
    /// these same values, and the keyword for a danger action.
    pub fn confirm_plugin_run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(action) = self.pending_action() else { return };
        let Some(p) = self.plugins.user.pending.clone() else { return };
        let fail = |this: &mut CrowApp, e: String, cx: &mut Context<Self>| {
            if let Some(p) = this.plugins.user.pending.as_mut() {
                p.error = Some(e);
            }
            cx.notify();
        };
        let values = match self.pending_values(cx) {
            Ok(v) => v,
            Err(e) => return fail(self, e, cx),
        };
        if let Some(dry) = &action.dry_run {
            let dry_slot = self.plugins.user.slot(&p.key.0, dry, &p.key.2);
            if p.dry_ran_with.as_ref() != Some(&values) {
                return fail(self, t!("app-user-plugins-dry-run-first").to_string(), cx);
            }
            if dry_slot.is_some_and(|s| s.running) {
                return fail(self, t!("app-user-plugins-dry-run-still-going").to_string(), cx);
            }
            if !dry_slot.and_then(|s| s.last.as_ref()).is_some_and(|r| r.ok) {
                return fail(self, t!("app-user-plugins-dry-run-failed").to_string(), cx);
            }
        }
        if action.effect == Effect::Danger {
            let want = action.keyword.clone().unwrap_or_else(|| "CONFIRM".into());
            let typed = self.plugins.user.keyword_input.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
            if typed != want {
                return fail(self, t!("app-user-plugins-type-keyword", keyword = want), cx);
            }
        }
        self.cancel_plugin_run(cx);
        if p.fleet {
            self.plan_plugin_fleet_run(&p.key.0, &p.key.1, values, window, cx);
        } else {
            self.start_plugin_run(&p.key.0, &p.key.1, values, RunMode::Pressed, cx);
        }
    }

    pub fn stop_plugin_run(&mut self, plugin: &str, action: &str, server: &str, cx: &mut Context<Self>) {
        if let Some(slot) = self.plugins.user.runs.get(&(plugin.to_string(), action.to_string(), server.to_string())) {
            slot.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        cx.notify();
    }

    /// Runs one action of an accepted plugin on the active server, streaming
    /// its output into the run's slot. Reads a page does by itself go in the
    /// run history only; pressed runs and dry runs go in the audit log too.
    pub fn start_plugin_run(&mut self, plugin_id: &str, action_id: &str, inputs: BTreeMap<String, String>, mode: RunMode, cx: &mut Context<Self>) {
        let Some(server) = self.fleet.active_server() else { return };
        self.start_plugin_run_on(server, plugin_id, action_id, inputs, mode, cx);
    }

    /// `start_plugin_run` on a given server (a fleet action's dry run goes
    /// to its canary).
    pub fn start_plugin_run_on(&mut self, server: ServerRecord, plugin_id: &str, action_id: &str, inputs: BTreeMap<String, String>, mode: RunMode, cx: &mut Context<Self>) {
        let key: RunKey = (plugin_id.to_string(), action_id.to_string(), server.id.clone());
        if mode != RunMode::Auto {
            self.plugins.user.shown_output = Some(key.clone());
        }
        // Already going (the page reading it): a press makes it a pressed run.
        if let Some(slot) = self.plugins.user.runs.get_mut(&key).filter(|s| s.running) {
            slot.pressed |= mode != RunMode::Auto;
            cx.notify();
            return;
        }
        let fail = |this: &mut CrowApp, why: String| {
            this.plugins.user.runs.insert(key.clone(), RunSlot { last: Some(RunResult::failed(why)), ..Default::default() });
        };
        let Some(shown) = self.plugins.user.get(plugin_id).cloned() else { return };
        // The files must still be the ones accepted, checked at every run.
        let now = folder::load(&shown.dir, &self.user_plugin_pin(plugin_id));
        if now.trust != Trust::Accepted {
            if let Some(p) = self.plugins.user.installed.iter_mut().find(|p| p.id == plugin_id) {
                *p = now;
            }
            fail(self, t!("app-user-plugins-not-accepted-now").to_string());
            cx.notify();
            return;
        }
        let Some(m) = now.report.manifest.clone() else { return };
        let Some(action) = m.actions.iter().find(|a| a.id == action_id).cloned() else { return };
        // Only a confirmed run changes anything: never a page's own read.
        if action.effect != Effect::Read && mode != RunMode::Pressed {
            return;
        }
        let stored = self.user_plugin_settings(plugin_id, &m.settings);
        let (settings, secrets, missing) = run::settings_values(&m.settings, &stored);
        let hide = secrets.clone();
        let needs: Vec<String> = action
            .templates()
            .flat_map(|(t, _)| t.vars().cloned().collect::<Vec<_>>())
            .filter_map(|v| match v {
                crate::plugin_host::manifest::Var::Setting(k) | crate::plugin_host::manifest::Var::Secret(k) => Some(k),
                _ => None,
            })
            .collect();
        let missing: Vec<String> = missing.into_iter().filter(|(k, _)| needs.contains(k)).map(|(_, why)| why).collect();
        if !missing.is_empty() {
            fail(self, t!("app-user-plugins-needs-settings", names = missing.join(", ")));
            cx.notify();
            return;
        }
        // An action gets the inputs it declares; a required one must be there.
        let mut values_in = BTreeMap::new();
        for f in &action.inputs {
            match inputs.get(&f.key).cloned().or_else(|| run::default_value(f)).map(|raw| run::check_value(f, &raw)) {
                Some(Ok(v)) if !v.is_empty() => {
                    values_in.insert(f.key.clone(), v);
                }
                Some(Err(e)) => {
                    fail(self, e);
                    cx.notify();
                    return;
                }
                _ if f.required => {
                    fail(self, t!("app-user-plugins-needs-input", name = f.label));
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }
        let nonce = run::nonce();
        let timeout = std::time::Duration::from_secs(action.timeout.0);
        let job: Job = match action.on {
            Runs::Server => {
                let values = Values { inputs: values_in.clone(), settings, secrets, server: Some(ServerValues { name: server.name.clone(), host: server.host.clone(), user: server.login_user.clone(), port: server.port }), local: None };
                let program = match run::remote_program(&action, &m.requires.server, &now.dir, &values, &nonce) {
                    Ok(p) => p,
                    Err(e) => {
                        fail(self, e);
                        cx.notify();
                        return;
                    }
                };
                let host = crate::host::host_for(&server);
                let root = action.root;
                Box::new(move |sink, stop| run::run_streaming(host.as_ref(), &program, root, timeout, &nonce, sink, stop))
            }
            Runs::Local => {
                if m.setup.is_some() && self.user_plugin_setup_hash(plugin_id).as_deref() != Some(now.hash.as_str()) {
                    fail(self, t!("app-user-plugins-needs-install").to_string());
                    cx.notify();
                    return;
                }
                let missing = local::missing_here(&m.requires.local);
                if !missing.is_empty() {
                    fail(self, t!("app-user-plugins-missing-here", names = missing.join(", ")));
                    cx.notify();
                    return;
                }
                let data_dir = local::data_dir(plugin_id);
                let prepared = local::run_dir(&data_dir, &nonce).and_then(|rd| {
                    let (inventory, ssh_config) = local::write_targets(&rd, std::slice::from_ref(&server))?;
                    let paths = local::LocalPaths { plugin_dir: now.dir.clone(), data_dir: data_dir.clone(), inventory, ssh_config, results: rd.join("results") };
                    let values = Values { inputs: values_in.clone(), settings, secrets, server: None, local: Some(paths.clone()) };
                    let env = local::environment(&data_dir, &rd, &action.env, &values)?;
                    let argv = local::argv(action.command.as_deref(), action.script.as_deref(), action.interpreter, &action.args, &now.dir, &values)?;
                    Ok((rd, paths, env, argv))
                });
                let (rd, paths, env, argv) = match prepared {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = std::fs::remove_dir_all(local::data_dir(plugin_id).join("runs").join(&nonce));
                        fail(self, e);
                        cx.notify();
                        return;
                    }
                };
                let host_alias = local::alias(&server);
                Box::new(move |sink, stop| {
                    let mut r = local::run(&argv, &paths.data_dir, &env, timeout, sink, stop);
                    // What the program said about this server, if it wrote results.
                    r.per_host = local::read_results(&paths.results);
                    if let Some(mine) = r.per_host.iter().find(|h| h["host"] == host_alias.as_str()) {
                        if mine.get("ok").and_then(|o| o.as_bool()) == Some(false) {
                            r.ok = false;
                        }
                    }
                    let _ = std::fs::remove_dir_all(&rd);
                    r
                })
            }
        };
        let live: Arc<Mutex<Vec<(bool, String)>>> = Arc::default();
        let stop: Arc<AtomicBool> = Arc::default();
        let previous = self.plugins.user.runs.remove(&key).and_then(|s| s.last);
        self.plugins.user.runs.insert(key.clone(), RunSlot { running: true, last: previous, pressed: mode != RunMode::Auto, live: live.clone(), stop: stop.clone() });
        cx.notify();
        let (hash, plugin_name) = (now.hash.clone(), now.name().to_string());
        let output = action.output.clone();
        let keeps_log = action.effect != Effect::Read;
        let detail = if values_in.is_empty() { String::new() } else { values_in.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ") };
        // Redraw while lines arrive.
        let pump_live = live.clone();
        cx.spawn(async move |entity, cx| {
            let mut seen = 0;
            loop {
                cx.background_executor().timer(std::time::Duration::from_millis(150)).await;
                let n = pump_live.lock().map(|l| l.len()).unwrap_or(0);
                let going = entity.update(cx, |this, cx| {
                    if n != seen {
                        cx.notify();
                    }
                    this.plugins.user.runs.values().any(|s| s.running && Arc::ptr_eq(&s.live, &pump_live))
                });
                seen = n;
                if !going.unwrap_or(false) {
                    break;
                }
            }
        })
        .detach();
        cx.spawn(async move |entity, cx| {
            let (sink_live, run_stop, hide_live) = (live.clone(), stop.clone(), hide.clone());
            let result = cx
                .background_executor()
                .spawn(async move {
                    let mut sink = |is_err: bool, line: &str| {
                        if let Ok(mut l) = sink_live.lock() {
                            if l.len() < 20_000 {
                                l.push((is_err, super::user_plugin_settings::redact(line, &hide_live)));
                            }
                        }
                    };
                    let mut r = job(&mut sink, &run_stop);
                    r.stdout = super::user_plugin_settings::redact(&r.stdout, &hide_live);
                    r.stderr = super::user_plugin_settings::redact(&r.stderr, &hide_live);
                    r.error = r.error.map(|e| super::user_plugin_settings::redact(&e, &hide_live));
                    r
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                let summary = if result.stopped { t!("app-user-plugins-stopped").to_string() } else { run::summary(&output, &result) };
                let pressed = this.plugins.user.runs.get(&key).is_some_and(|s| s.pressed);
                let log = if keeps_log { cap_log(&result) } else { String::new() };
                if let Ok(db) = this.vault.db().lock() {
                    let _ = db.insert_plugin_run(&PluginRun {
                        plugin_id: key.0.clone(),
                        action_id: key.1.clone(),
                        server_id: key.2.clone(),
                        started_at: result.started_at,
                        finished_at: result.finished_at,
                        ok: result.ok,
                        exit_code: result.exit_code,
                        summary: summary.clone(),
                        output_bytes: result.output_bytes(),
                        plugin_hash: hash.clone(),
                        dry_run: mode == RunMode::DryRun,
                        stopped: result.stopped,
                        log,
                    });
                    if pressed {
                        let _ = db.insert_change_record(&ChangeRecord {
                            id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                            server_id: server.id.clone(),
                            server_name: server.name.clone(),
                            action_kind: format!("plugin.{}.{}", key.0, key.1),
                            target: plugin_name.clone(),
                            before_state: [summary, detail].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · "),
                            after_state: result.error.clone(),
                            blast_radius: None,
                            outcome: if result.ok { "success" } else { "failed" }.to_string(),
                            started_at: chrono::Utc::now().to_rfc3339(),
                            completed_at: None,
                        });
                    }
                }
                this.plugins.user.runs.insert(key, RunSlot { last: Some(result), ..Default::default() });
                cx.notify();
            });
        })
        .detach();
    }

    /// The plugin files whose setup last succeeded (their hash).
    pub fn user_plugin_setup_hash(&self, id: &str) -> Option<String> {
        let db = self.vault.db();
        let db = db.lock().ok()?;
        db.flag(&format!("user_plugin.{id}.setup")).filter(|s| !s.is_empty())
    }

    /// INSTALL: the plugin's setup, on this machine, in its data folder.
    /// Its actions on this machine wait until it succeeds with these files.
    pub fn run_plugin_setup(&mut self, plugin_id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let key: RunKey = (plugin_id.to_string(), SETUP.to_string(), HERE.to_string());
        if self.plugins.user.runs.get(&key).is_some_and(|s| s.running) {
            return;
        }
        let fail = |this: &mut CrowApp, why: String, cx: &mut Context<Self>| {
            this.plugins.user.runs.insert(key.clone(), RunSlot { last: Some(RunResult::failed(why)), ..Default::default() });
            cx.notify();
        };
        let Some(shown) = self.plugins.user.get(plugin_id).cloned() else { return };
        let now = folder::load(&shown.dir, &self.user_plugin_pin(plugin_id));
        if now.trust != Trust::Accepted {
            return fail(self, t!("app-user-plugins-not-accepted-now").to_string(), cx);
        }
        let Some(setup) = now.report.manifest.as_ref().and_then(|m| m.setup.clone()) else { return };
        let missing = local::missing_here(&now.report.manifest.as_ref().map(|m| m.requires.local.clone()).unwrap_or_default());
        if !missing.is_empty() {
            return fail(self, t!("app-user-plugins-missing-here", names = missing.join(", ")), cx);
        }
        let fields = now.report.manifest.as_ref().map(|m| m.settings.clone()).unwrap_or_default();
        let stored = self.user_plugin_settings(plugin_id, &fields);
        let (settings, secrets, _) = run::settings_values(&fields, &stored);
        let hide = secrets.clone();
        let nonce = run::nonce();
        let data_dir = local::data_dir(plugin_id);
        let prepared = local::run_dir(&data_dir, &nonce).and_then(|rd| {
            let paths = local::LocalPaths { plugin_dir: now.dir.clone(), data_dir: data_dir.clone(), ..Default::default() };
            let values = Values { settings, secrets, local: Some(paths), ..Default::default() };
            let env = local::environment(&data_dir, &rd, &setup.env, &values)?;
            let argv = local::argv(setup.command.as_deref(), setup.script.as_deref(), setup.interpreter, &setup.args, &now.dir, &values)?;
            Ok((rd, env, argv))
        });
        let (rd, env, argv) = match prepared {
            Ok(p) => p,
            Err(e) => return fail(self, e, cx),
        };
        let live: Arc<Mutex<Vec<(bool, String)>>> = Arc::default();
        let stop: Arc<AtomicBool> = Arc::default();
        self.plugins.user.runs.insert(key.clone(), RunSlot { running: true, pressed: true, live: live.clone(), stop: stop.clone(), ..Default::default() });
        cx.notify();
        let timeout = std::time::Duration::from_secs(setup.timeout.0);
        let (hash, name, id) = (now.hash.clone(), now.name().to_string(), plugin_id.to_string());
        let pump = live.clone();
        cx.spawn(async move |entity, cx| loop {
            cx.background_executor().timer(std::time::Duration::from_millis(200)).await;
            let going = entity.update(cx, |this, cx| {
                cx.notify();
                this.plugins.user.runs.values().any(|s| s.running && Arc::ptr_eq(&s.live, &pump))
            });
            if !going.unwrap_or(false) {
                break;
            }
        })
        .detach();
        cx.spawn(async move |entity, cx| {
            let (sink_live, run_stop, dir) = (live.clone(), stop.clone(), data_dir.clone());
            let result = cx
                .background_executor()
                .spawn(async move {
                    let mut sink = |is_err: bool, line: &str| {
                        if let Ok(mut l) = sink_live.lock() {
                            if l.len() < 20_000 {
                                l.push((is_err, super::user_plugin_settings::redact(line, &hide)));
                            }
                        }
                    };
                    let mut r = local::run(&argv, &dir, &env, timeout, &mut sink, &run_stop);
                    let _ = std::fs::remove_dir_all(&rd);
                    r.stdout = super::user_plugin_settings::redact(&r.stdout, &hide);
                    r.stderr = super::user_plugin_settings::redact(&r.stderr, &hide);
                    r
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                if let Ok(db) = this.vault.db().lock() {
                    if result.ok {
                        let _ = db.set_flag(&format!("user_plugin.{id}.setup"), &hash);
                    }
                }
                if result.ok {
                    this.plugins.user.setup_hashes.insert(id.clone(), hash.clone());
                }
                if let Ok(db) = this.vault.db().lock() {
                    let _ = db.insert_plugin_run(&PluginRun {
                        plugin_id: id.clone(),
                        action_id: SETUP.into(),
                        server_id: HERE.into(),
                        started_at: result.started_at,
                        finished_at: result.finished_at,
                        ok: result.ok,
                        exit_code: result.exit_code,
                        summary: if result.stopped { t!("app-user-plugins-stopped").to_string() } else { run::summary(&Default::default(), &result) },
                        output_bytes: result.output_bytes(),
                        plugin_hash: hash.clone(),
                        dry_run: false,
                        stopped: result.stopped,
                        log: cap_log(&result),
                    });
                    let _ = db.insert_change_record(&ChangeRecord {
                        id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                        server_id: AUDIT_SERVER_ID.to_string(),
                        server_name: "Crow".to_string(),
                        action_kind: "plugin.setup".into(),
                        target: name.clone(),
                        before_state: hash.clone(),
                        after_state: result.error.clone(),
                        blast_radius: None,
                        outcome: if result.ok { "success" } else { "failed" }.to_string(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        completed_at: None,
                    });
                }
                this.plugins.user.runs.insert(key, RunSlot { last: Some(result), ..Default::default() });
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn user_plugin_pin(&self, id: &str) -> Pin {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return Pin::default() };
        Pin { accepted: db.flag(&folder::accepted_flag(id)).filter(|s| !s.is_empty()), refused: db.flag(&folder::refused_flag(id)).filter(|s| !s.is_empty()) }
    }

    /// Reads the plugins folder in the background, then pins from the vault.
    pub fn scan_user_plugins(&mut self, cx: &mut Context<Self>) {
        if self.plugins.user.scanning {
            return;
        }
        self.plugins.user.scanning = true;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            // Checked without pins first (files only), so the vault isn't
            // touched off the main thread.
            let found = cx.background_executor().spawn(async move { folder::scan(&folder::plugins_dir(), |_| Pin::default()) }).await;
            let _ = entity.update(cx, |this, cx| {
                let mut found = found;
                for p in &mut found {
                    p.trust = folder::trust(&p.report, &p.hash, &this.user_plugin_pin(&p.id));
                    if let Some(h) = this.user_plugin_setup_hash(&p.id) {
                        this.plugins.user.setup_hashes.insert(p.id.clone(), h);
                    }
                }
                this.plugins.user.installed = found;
                this.plugins.user.scanning = false;
                if let Some(id) = &this.plugins.user.reviewing {
                    if this.plugins.user.get(id).is_none() {
                        this.plugins.user.reviewing = None;
                    }
                }
                this.plugins.user.confirming = None;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn open_plugins_folder(&mut self, cx: &mut Context<Self>) {
        let dir = folder::plugins_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.plugins.user.message = Some((false, t!("app-user-plugins-couldnt-create", path = dir.display(), e = e)));
            cx.notify();
            return;
        }
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&dir).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }

    pub fn toggle_user_plugin_review(&mut self, id: &str, cx: &mut Context<Self>) {
        self.plugins.user.confirming = None;
        self.plugins.user.reviewing = if self.plugins.user.reviewing.as_deref() == Some(id) { None } else { Some(id.to_string()) };
        cx.notify();
    }

    /// First press arms; the second accepts, if the files are still the
    /// ones on screen.
    pub fn accept_user_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        if self.plugins.user.confirming.as_deref() != Some(id) {
            self.plugins.user.confirming = Some(id.to_string());
            cx.notify();
            return;
        }
        self.plugins.user.confirming = None;
        let Some(shown) = self.plugins.user.get(id).cloned() else { return };
        // Read again: what's accepted must be exactly what was reviewed.
        let now = folder::load(&shown.dir, &Pin::default());
        if now.hash != shown.hash || !now.report.ok() {
            self.plugins.user.message = Some((false, t!("app-user-plugins-changed-while-reviewing", name = shown.name())));
            self.scan_user_plugins(cx);
            return;
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&folder::accepted_flag(id), &now.hash);
            let _ = db.set_flag(&folder::refused_flag(id), "");
        }
        self.audit_user_plugin("plugin.accept", &now);
        self.plugins.user.message = Some((true, t!("app-user-plugins-accepted", name = now.name())));
        self.plugins.user.reviewing = None;
        self.scan_user_plugins(cx);
    }

    pub fn refuse_user_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(p) = self.plugins.user.get(id).cloned() else { return };
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&folder::refused_flag(id), &p.hash);
            let _ = db.set_flag(&folder::accepted_flag(id), "");
        }
        self.audit_user_plugin("plugin.refuse", &p);
        self.plugins.user.message = Some((true, t!("app-user-plugins-refused", name = p.name())));
        self.plugins.user.reviewing = None;
        self.plugins.user.confirming = None;
        self.scan_user_plugins(cx);
    }

    /// Takes back an acceptance: the plugin is new again.
    pub fn revoke_user_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(p) = self.plugins.user.get(id).cloned() else { return };
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&folder::accepted_flag(id), "");
        }
        self.audit_user_plugin("plugin.revoke", &p);
        self.plugins.user.message = Some((true, t!("app-user-plugins-revoked", name = p.name())));
        self.scan_user_plugins(cx);
    }

    fn audit_user_plugin(&self, kind: &str, p: &Installed) {
        let version = p.report.manifest.as_ref().map(|m| m.plugin.version.clone()).unwrap_or_default();
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.insert_change_record(&ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: AUDIT_SERVER_ID.to_string(),
                server_name: "Crow".to_string(),
                action_kind: kind.to_string(),
                target: format!("{} {version} ({})", p.name(), p.id),
                before_state: p.hash.clone(),
                after_state: None,
                blast_radius: None,
                outcome: "success".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }
    }
}

/// Short words for a plugin's state, for its row.
pub fn trust_label(t: Trust) -> &'static str {
    match t {
        Trust::New => t!("app-user-plugins-state-new"),
        Trust::Accepted => t!("app-user-plugins-state-accepted"),
        Trust::Changed => t!("app-user-plugins-state-changed"),
        Trust::Refused => t!("app-user-plugins-state-refused"),
        Trust::Invalid => t!("app-user-plugins-state-invalid"),
    }
}
