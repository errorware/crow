//! User plugins in Settings → Plugins (ERR-177): found in the plugins
//! folder, reviewed, then accepted or refused. Nothing of a plugin runs in
//! this stage; accepting only pins its files.

use std::collections::{BTreeMap, HashMap};

use gpui_kit::*;

use super::CrowApp;
use crate::plugin_host::folder::{self, Installed, Pin, Trust};
use crate::plugin_host::manifest::{BlockKind, Effect, Place, Runs};
use crate::plugin_host::run::{self, RunResult, ServerValues, Values};
use crate::vault::{ChangeRecord, PluginRun, AUDIT_SERVER_ID};

/// The last run of one action on one server, and whether one is going.
#[derive(Debug, Clone, Default)]
pub struct RunSlot {
    pub running: bool,
    pub last: Option<RunResult>,
    /// Someone pressed its button: the run goes in the audit log too.
    pub pressed: bool,
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
                self.run_plugin_action(&plugin, &a, true, cx);
                seen.push(a);
            }
        }
        cx.notify();
    }

    /// Runs one action of an accepted plugin on the active server. `auto`:
    /// a page reading what it shows, kept in the run history but not the
    /// audit log; a button press is both.
    pub fn run_plugin_action(&mut self, plugin_id: &str, action_id: &str, auto: bool, cx: &mut Context<Self>) {
        let Some(server) = self.fleet.active_server() else { return };
        let key: RunKey = (plugin_id.to_string(), action_id.to_string(), server.id.clone());
        if !auto {
            self.plugins.user.shown_output = Some(key.clone());
        }
        // Already going (the page reading it): a press makes it a pressed run.
        if let Some(slot) = self.plugins.user.runs.get_mut(&key).filter(|s| s.running) {
            slot.pressed |= !auto;
            cx.notify();
            return;
        }
        let fail = |this: &mut CrowApp, why: String| {
            this.plugins.user.runs.insert(key.clone(), RunSlot { running: false, last: Some(RunResult::failed(why)), pressed: false });
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
        if action.effect != Effect::Read || action.on != Runs::Server {
            fail(self, t!("app-user-plugins-later-stage").to_string());
            cx.notify();
            return;
        }
        let (settings, secrets, missing) = run::settings_values(&m.settings, &BTreeMap::new());
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
        let mut inputs = BTreeMap::new();
        for f in &action.inputs {
            match run::default_value(f).map(|d| run::check_value(f, &d)) {
                Some(Ok(v)) => {
                    inputs.insert(f.key.clone(), v);
                }
                _ if f.required => {
                    fail(self, t!("app-user-plugins-needs-input", name = f.label));
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }
        let values = Values { inputs, settings, secrets, server: Some(ServerValues { name: server.name.clone(), host: server.host.clone(), user: server.login_user.clone(), port: server.port }) };
        let nonce = run::nonce();
        let program = match run::remote_program(&action, &m.requires.server, &now.dir, &values, &nonce) {
            Ok(p) => p,
            Err(e) => {
                fail(self, e);
                cx.notify();
                return;
            }
        };
        let slot = self.plugins.user.runs.entry(key.clone()).or_default();
        slot.running = true;
        slot.pressed = !auto;
        cx.notify();
        let host = crate::host::host_for(&server);
        let timeout = std::time::Duration::from_secs(action.timeout.0);
        let (root, hash, plugin_name) = (action.root, now.hash.clone(), now.name().to_string());
        let output = action.output.clone();
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { run::run_program(host.as_ref(), &program, root, timeout, &nonce) }).await;
            let _ = entity.update(cx, |this, cx| {
                let summary = run::summary(&output, &result);
                let pressed = this.plugins.user.runs.get(&key).is_some_and(|s| s.pressed);
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
                    });
                    if pressed {
                        let _ = db.insert_change_record(&ChangeRecord {
                            id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                            server_id: server.id.clone(),
                            server_name: server.name.clone(),
                            action_kind: format!("plugin.{}.{}", key.0, key.1),
                            target: plugin_name.clone(),
                            before_state: summary,
                            after_state: result.error.clone(),
                            blast_radius: None,
                            outcome: if result.ok { "success" } else { "failed" }.to_string(),
                            started_at: chrono::Utc::now().to_rfc3339(),
                            completed_at: None,
                        });
                    }
                }
                this.plugins.user.runs.insert(key, RunSlot { running: false, last: Some(result), pressed: false });
                cx.notify();
            });
        })
        .detach();
    }

    fn user_plugin_pin(&self, id: &str) -> Pin {
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
