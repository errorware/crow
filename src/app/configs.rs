use crow_config_core::ConfigPlugin;
use crow_config_core::edit::ConfigDocument;
use crow_config_schemas::PgHbaPlugin;
use gpui_kit::*;

use super::CrowApp;
use gpui_kit::component::input::{EditorState, InputEvent};
use crate::views::config::text_editor::{highlighter_factory, CONFIG_LANGUAGE};
use std::sync::Arc;

use crate::config::{crawl_configs, load_config_file_state};
use crate::host::{host_for, not_connected, Host, LocalHost};
use crate::journal::retention::generate_journald_conf;
use crate::os_detect::{classify_distro_family, detect_os_release, DistroFamily};
use crate::vault::ServerRecord;
use crate::views::config::cron_editor::generate_crontab_content;
use crate::views::config::rules_editor::default_hba_rules;
use crate::views::config::state::ConfigsState;
use crate::views::firewall::generate_user_rules_content;
use crate::journal::retention::JournalRetentionConfig;
use crate::theme::{CRIT, OK, WARN};
use crate::views::config::cron_editor::default_cron_jobs;
use crate::views::firewall::FirewallOperationalState;

// ==========================================
// Config files: discovery, staging, versions
// ==========================================

/// Startup smoke check that crow-config-core and its pg_hba schema plugin
/// parse a representative file; logs the parsed rule count.
pub fn log_config_core_self_check() {
    let plugin = PgHbaPlugin::new();
    let sample_pg_hba = r#"
# PostgreSQL Client Authentication Configuration File
local   all             postgres                                peer
host    all             all             127.0.0.1/32            scram-sha-256
host    all             all             ::1/128                 scram-sha-256
host    acme_prod       acme_app        10.0.4.19/32            scram-sha-256
host    all             all             0.0.0.0/0               md5
host    all             all             10.0.4.0/24             scram-sha-256
"#;
    if let Ok(doc) = ConfigDocument::parse(&plugin, sample_pg_hba) {
        if let Ok(ir) = doc.to_ir() {
            println!(
                "Crow Core: parsed {} pg_hba rules from schema plugin: {}",
                ir.rows.len(),
                plugin.manifest().plugin.name
            );
        }
    }
}

/// Files whose structured editor renders Crow's own sample model rather than
/// the host's file. Writing them back would replace a real config with demo
/// content, so they stay read-only until the editors parse the real files.
const SAMPLE_MODEL_FILES: &[(&str, &str)] = &[
    ("pg_hba.conf", "pg_hba rules editor"),
    ("crontab", "cron editor"),
    ("journald.conf", "journald retention editor"),
    ("user.rules", "firewall rules editor"),
];

/// Discovers and loads config files from `server` (this machine when there are
/// no servers), seeds the structured editors' files from their models, and
/// blocks writes wherever the content isn't the host's real file.
pub fn load_configs(
    server: Option<&ServerRecord>,
    journal_retention: &JournalRetentionConfig,
    firewall: &FirewallOperationalState,
) -> ConfigsState {
    let host: Option<Arc<dyn Host>> = match server {
        Some(srv) => host_for(srv),
        None => Some(Arc::new(LocalHost)),
    };
    let family = match (&host, server) {
        (Some(h), _) => detect_os_release(h.as_ref()).map(|d| classify_distro_family(&d)).unwrap_or(DistroFamily::Unknown),
        (None, Some(srv)) => classify_distro_family(&srv.os_distro),
        (None, None) => DistroFamily::Unknown,
    };
    let files = crawl_configs(host.as_deref(), family);
    let states = files.iter().map(|f| (f.name.clone(), load_config_file_state(host.as_deref(), f))).collect();
    let selected = files.first().map(|f| f.name.clone()).unwrap_or_else(|| "journald.conf".to_string());

    let mut configs = ConfigsState::new(files, states, selected, default_cron_jobs(), default_hba_rules());
    configs.server_id = server.map(|s| s.id.clone());
    configs.family = family;
    configs.sync_hba();
    configs.seed_baseline("journald.conf", generate_journald_conf(journal_retention), None);
    configs.seed_baseline("crontab", generate_crontab_content(&configs.cron_jobs), None);
    if let FirewallOperationalState::Active(ref summary) = firewall {
        let fw_text = generate_user_rules_content(&summary.rules);
        configs.seed_baseline("user.rules", fw_text, Some("/etc/ufw/user.rules"));
    }
    for (file, editor) in SAMPLE_MODEL_FILES {
        configs.block_writes(file, format!("The {editor} shows Crow's sample model, not this server's {file} — saving is disabled until it parses the real file"));
    }
    configs
}

/// The live text editor for the selected plain-text config file. Lives on
/// the app, not in `ConfigsState`: configs are loaded on a background thread
/// and GPUI entities/subscriptions must stay on the UI thread.
pub struct ConfigTextEditor {
    pub file: String,
    pub editor: Entity<EditorState>,
    _changes: Subscription,
}

/// Author recorded on staged config revisions.
pub fn default_author() -> String {
    "Nelson <nelson@errorware.net>".to_string()
}

impl CrowApp {
    pub fn default_author(&self) -> String {
        default_author()
    }

    /// The text editor for `file`, created on first use and kept in sync with
    /// the file's edit state: typing flows into the state through the change
    /// subscription, and changes made elsewhere (revert, rollback, reload) are
    /// pushed back into the editor here.
    pub fn config_text_editor(&mut self, file: &str, window: &mut Window, cx: &mut Context<Self>) -> Option<Entity<EditorState>> {
        let content = self.configs.states.get(file)?.current_content.clone();
        let reuse = self.config_text_editor.as_ref().filter(|t| t.file == file).map(|t| t.editor.clone());
        let editor = match reuse {
            Some(editor) => {
                if editor.read(cx).value().as_ref() != content {
                    editor.update(cx, |e, cx| e.set_value(content, window, cx));
                }
                editor
            }
            None => {
                let editor = cx.new(|cx| {
                    let mut state = EditorState::new(window, cx).language(CONFIG_LANGUAGE).default_value(content);
                    state.set_highlighter_factory(highlighter_factory(file), cx);
                    state
                });
                let file_name = file.to_string();
                let changes = cx.subscribe(&editor, move |this, editor, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::Change) {
                        let text = editor.read(cx).value().to_string();
                        if let Some(st) = this.configs.states.get_mut(&file_name) {
                            st.update_content(text);
                        }
                        cx.notify();
                    }
                });
                self.config_text_editor = Some(ConfigTextEditor { file: file.to_string(), editor: editor.clone(), _changes: changes });
                editor
            }
        };
        Some(editor)
    }

    pub fn toggle_config_history(&mut self, cx: &mut Context<Self>) {
        self.configs.show_history = !self.configs.show_history;
        cx.notify();
    }

    pub fn select_managed_file(&mut self, filename: &str, cx: &mut Context<Self>) {
        self.configs.selected_file = filename.to_string();
        cx.notify();
    }

    /// The server the loaded configs belong to (not necessarily the active tab,
    /// when unsaved edits were kept across a tab switch).
    fn configs_server(&self) -> Option<ServerRecord> {
        let id = self.configs.server_id.as_ref()?;
        self.fleet.servers.iter().find(|s| &s.id == id).cloned()
    }

    /// Re-scans the configs' server for files, keeping existing edit states.
    pub fn crawl_system_configs(&mut self, cx: &mut Context<Self>) {
        let server = self.configs_server();
        let fresh = load_configs(server.as_ref(), &self.journal.retention, &self.firewall.status);
        for (name, st) in fresh.states {
            self.configs.states.entry(name).or_insert(st);
        }
        self.configs.files = fresh.files;
        cx.notify();
    }

    /// Reloads configs for the active server in the background, unless there
    /// are unsaved edits — those stay attached to the server they were read
    /// from. The result is dropped if the active server or edits changed while
    /// it loaded.
    pub fn reload_configs_for_active_server(&mut self, cx: &mut Context<Self>) {
        let server = self.fleet.active_server();
        let target = server.as_ref().map(|s| s.id.clone());
        if self.configs.has_unsaved_changes() || target == self.configs.server_id {
            return;
        }
        let retention = self.journal.retention.clone();
        let firewall = self.firewall.status.clone();
        cx.spawn(async move |entity, cx| {
            let loaded = cx
                .background_executor()
                .spawn(async move { load_configs(server.as_ref(), &retention, &firewall) })
                .await;
            let _ = entity.update(cx, |this, cx| {
                let still_active = this.fleet.active_server().map(|s| s.id) == target;
                if still_active && !this.configs.has_unsaved_changes() {
                    let selected = std::mem::take(&mut this.configs.selected_file);
                    this.configs = loaded;
                    if this.configs.states.contains_key(&selected) {
                        this.configs.selected_file = selected;
                    }
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Writes one file's current content back to the configs' server.
    fn write_config_file(&self, file: &str) -> Result<(), String> {
        let state = self.configs.states.get(file).ok_or_else(|| format!("{file} is not loaded"))?;
        let host: Arc<dyn Host> = match self.configs_server() {
            Some(srv) => host_for(&srv).ok_or_else(|| not_connected(&srv).to_string())?,
            None => Arc::new(LocalHost),
        };
        state.save_to(host.as_ref()).map_err(|e| format!("{file}: {e}"))
    }

    #[allow(dead_code)]
    pub fn set_config_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.configs.search_query = query;
        cx.notify();
    }

    pub fn revert_managed_config(&mut self, file: &str, cx: &mut Context<Self>) {
        if file == "pg_hba.conf" {
            for r in &mut self.configs.hba_rules {
                if r.num == "09" || r.num == "10" {
                    r.method = "trust";
                    r.risk = "CRITICAL";
                    r.risk_color = CRIT;
                    r.is_expanded = false;
                }
            }
        } else if file == "journald.conf" {
            self.journal.retention = JournalRetentionConfig::default();
        } else if file == "crontab" {
            self.configs.cron_jobs = default_cron_jobs();
        } else if file == "user.rules" {
            if let Some(state) = self.configs.states.get(file) {
                let rules = crate::views::firewall::parse_user_rules_content(&state.baseline_content);
                if let FirewallOperationalState::Active(ref mut summary) = self.firewall.status {
                    summary.rules = rules;
                }
            }
            self.firewall.toast = Some("Reverted firewall rules to baseline".to_string());
        }
        if let Some(state) = self.configs.states.get_mut(file) {
            state.revert();
            cx.notify();
        }
    }

    /// Writes a file's edits to its server and, only if that succeeded, records
    /// them as a new revision. A failed write leaves the edits pending.
    pub fn stage_config_version(&mut self, file: &str, description: &str, cx: &mut Context<Self>) {
        let author = self.default_author();
        if let Err(e) = self.write_config_file(file) {
            self.configs.save_error = Some(e);
            cx.notify();
            return;
        }
        self.configs.save_error = None;
        if let Some(state) = self.configs.states.get_mut(file) {
            state.stage_revision(author, description.to_string());
            if file == "pg_hba.conf" {
                for r in &mut self.configs.hba_rules {
                    if r.risk == "EDITED" {
                        r.risk = "OK";
                        r.risk_color = OK;
                    }
                }
            } else if file == "journald.conf" {
                self.journal.show_retention_modal = false;
            } else if file == "user.rules" {
                self.firewall.toast = Some(format!("Audit commit created: {}", description));
            }
            cx.notify();
        }
    }

    /// Restores a revision and writes it to the server; on a failed write the
    /// file stays at the revision it was on.
    pub fn rollback_config_revision(&mut self, file: &str, version: usize, cx: &mut Context<Self>) {
        let Some(before) = self.configs.states.get(file).cloned() else { return };
        if !self.configs.states.get_mut(file).is_some_and(|st| st.rollback_to_revision(version)) {
            return;
        }
        if let Err(e) = self.write_config_file(file) {
            self.configs.states.insert(file.to_string(), before);
            self.configs.save_error = Some(e);
            cx.notify();
            return;
        }
        self.configs.save_error = None;
        if let Some(state) = self.configs.states.get_mut(file) {
            {
                if file == "user.rules" {
                    let rules = crate::views::firewall::parse_user_rules_content(&state.current_content);
                    if let FirewallOperationalState::Active(ref mut summary) = self.firewall.status {
                        summary.rules = rules;
                    }
                    self.firewall.toast = Some(format!("Rolled back firewall to revision v{}", version));
                }
                cx.notify();
            }
        }
    }

    pub fn apply_journal_boundaries(&mut self, cx: &mut Context<Self>) {
        self.stage_config_version("journald.conf", "Applied journald retention boundaries", cx);
    }

    pub fn toggle_rule_expand(&mut self, num: &str, cx: &mut Context<Self>) {
        for r in &mut self.configs.hba_rules {
            if r.num == num {
                r.is_expanded = !r.is_expanded;
            }
        }
        cx.notify();
    }

    pub fn set_rule_method(&mut self, rule_num: &str, method: &'static str, cx: &mut Context<Self>) {
        for r in &mut self.configs.hba_rules {
            if r.num == rule_num {
                r.method = method;
                r.risk = if method == "scram-sha-256" || method == "cert" {
                    "OK"
                } else if method == "trust" {
                    "CRITICAL"
                } else {
                    "REVIEW"
                };
                r.risk_color = if r.risk == "OK" {
                    OK
                } else if r.risk == "CRITICAL" {
                    CRIT
                } else {
                    WARN
                };
            }
        }
        self.configs.sync_hba();
        cx.notify();
    }

    // --- SSH Key Management Subsystem ---
}
