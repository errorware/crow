use crow_config_core::ConfigPlugin;
use crow_config_core::edit::ConfigDocument;
use crow_config_schemas::PgHbaPlugin;
use gpui_kit::*;

use std::collections::HashMap;

use crow_config_core::edit::EditOp;
use gpui_kit::component::input::InputState;

use super::CrowApp;
use crate::config::plugins::{self, apply_edit as apply_structured_edit, editor_for, ConfigEditor, DedicatedScreen, RiskFinding, StructuredFormat};
use crate::theme::{FONT_MONO, TEXT_FAINT};
use crate::views::config::journald_editor::journald_editor;
use crate::views::config::raw_config_editor;
use crate::views::config::structured_editor::{screen_handoff, structured_editor, ActiveFieldEdit};
use super::Screen;
use gpui_kit::component::input::{EditorState, InputEvent};
use crate::views::config::text_editor::{highlighter_factory, CONFIG_LANGUAGE};
use std::sync::Arc;

use crate::config::{crawl_configs, load_config_file_state};
use crate::host::{host_for, not_connected, Host, LocalHost};
use crate::os_detect::{classify_distro_family, detect_os_release, DistroFamily};
use crate::vault::ServerRecord;
use crate::views::config::state::ConfigsState;
use crate::views::firewall::generate_user_rules_content;
use crate::journal::retention::parse_journald_conf;
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

/// Files that stay read-only in the Config screen, and why.
const READ_ONLY_FILES: &[(&str, &str)] = &[(
    "user.rules",
    "ufw writes this file itself — change rules on the Firewall screen, which runs ufw",
)];

/// Discovers and loads config files from `server` (this machine when there are
/// no servers), seeds the structured editors' files from their models, and
/// blocks writes wherever the content isn't the host's real file.
pub fn load_configs(server: Option<&ServerRecord>, firewall: &FirewallOperationalState) -> ConfigsState {
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

    let mut configs = ConfigsState::new(files, states, selected);
    configs.server_id = server.map(|s| s.id.clone());
    configs.family = family;
    if let Some(crontab) = configs.states.get("crontab").map(|st| st.current_content.clone()) {
        configs.reload_cron_from(&crontab);
    }
    if let FirewallOperationalState::Active(ref summary) = firewall {
        let fw_text = generate_user_rules_content(&summary.rules);
        configs.seed_baseline("user.rules", fw_text, Some("/etc/ufw/user.rules"));
    }
    for (file, reason) in READ_ONLY_FILES {
        configs.block_writes(file, reason.to_string());
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

/// A structured-editor field being edited inline.
pub struct StructuredFieldEdit {
    pub file: String,
    pub row_id: String,
    pub field: String,
    /// String-list fields (e.g. hosts' hostnames) are edited space-separated.
    pub is_list: bool,
    pub input: Entity<InputState>,
    _events: Subscription,
}

/// A staged change waiting for typed confirmation because it introduces
/// never-on-prod values.
pub struct RiskConfirm {
    pub file: String,
    pub description: String,
    pub findings: Vec<RiskFinding>,
    /// Created when the editor next renders (inputs need the window).
    pub input: Option<Entity<InputState>>,
    pub error: Option<String>,
}

/// What must be typed to stage a never-on-prod change.
pub const RISK_CONFIRM_KEYWORD: &str = "CONFIRM";

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

    /// The crow-config plugin that structures `file`, if any.
    pub fn structured_format_of(&self, file: &str) -> Option<StructuredFormat> {
        let kind = self.configs.files.iter().find(|f| f.name == file).and_then(|f| f.schema_kind)?;
        match editor_for(Some(kind)) {
            ConfigEditor::Structured(format) => Some(format),
            _ => None,
        }
    }

    /// Applies one crow-config edit to `file`'s current text. Row ids are line
    /// numbers, so any open option list or inline edit is closed afterwards —
    /// the view re-derives rows from the new text.
    pub fn apply_structured_op(&mut self, file: &str, op: EditOp, cx: &mut Context<Self>) {
        let Some(format) = self.structured_format_of(file) else { return };
        let Some(state) = self.configs.states.get_mut(file) else { return };
        match apply_structured_edit(format, &state.current_content, &op) {
            Ok(text) => {
                state.update_content(text);
                self.configs.edit_error = None;
            }
            Err(e) => self.configs.edit_error = Some(e),
        }
        self.configs.open_enum = None;
        self.configs.adding_row = false;
        self.structured_field_edit = None;
        cx.notify();
    }

    pub fn toggle_structured_enum(&mut self, row_id: &str, field: &str, cx: &mut Context<Self>) {
        let key = (row_id.to_string(), field.to_string());
        self.configs.open_enum = if self.configs.open_enum.as_ref() == Some(&key) { None } else { Some(key) };
        cx.notify();
    }

    pub fn set_structured_value(&mut self, file: &str, row_id: &str, field: &str, value: serde_json::Value, cx: &mut Context<Self>) {
        let op = EditOp::UpdateField { row_id: row_id.to_string(), field_name: field.to_string(), new_value: value };
        self.apply_structured_op(file, op, cx);
    }

    /// Opens an inline text input on a field; Enter or leaving the field commits.
    #[allow(clippy::too_many_arguments)]
    pub fn begin_structured_field_edit(
        &mut self,
        file: &str,
        row_id: &str,
        field: &str,
        current: &str,
        is_list: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(|cx| InputState::new(window, cx).default_value(current.to_string()));
        input.update(cx, |i, cx| i.focus(window, cx));
        let events = cx.subscribe(&input, |this, _input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                this.commit_structured_field_edit(cx);
            }
        });
        self.configs.open_enum = None;
        self.structured_field_edit = Some(StructuredFieldEdit {
            file: file.to_string(),
            row_id: row_id.to_string(),
            field: field.to_string(),
            is_list,
            input,
            _events: events,
        });
        cx.notify();
    }

    pub fn commit_structured_field_edit(&mut self, cx: &mut Context<Self>) {
        let Some(edit) = self.structured_field_edit.take() else { return };
        let text = edit.input.read(cx).value().trim().to_string();
        let value = if edit.is_list {
            serde_json::json!(text.split_whitespace().collect::<Vec<_>>())
        } else {
            serde_json::Value::String(text)
        };
        self.set_structured_value(&edit.file, &edit.row_id, &edit.field, value, cx);
    }

    pub fn move_structured_row(&mut self, file: &str, row_id: &str, after: Option<String>, before: Option<String>, cx: &mut Context<Self>) {
        let op = EditOp::MoveRow { row_id: row_id.to_string(), after_row_id: after, before_row_id: before };
        self.apply_structured_op(file, op, cx);
    }

    pub fn delete_structured_row(&mut self, file: &str, row_id: &str, cx: &mut Context<Self>) {
        self.apply_structured_op(file, EditOp::DeleteRow { row_id: row_id.to_string() }, cx);
    }

    /// Appends a row. Keyed formats pass the directive and its value; list and
    /// table formats get an editable placeholder row.
    pub fn insert_structured_row(&mut self, file: &str, keyed: Option<(String, String)>, cx: &mut Context<Self>) {
        let Some(format) = self.structured_format_of(file) else { return };
        let Some(state) = self.configs.states.get(file) else { return };
        let last_row = plugins::to_ir(format, &state.current_content).ok().and_then(|ir| ir.rows.last().map(|r| r.row_id.clone()));
        let mut fields = HashMap::new();
        match (format, keyed) {
            (_, Some((key, value))) => {
                fields.insert(key, serde_json::Value::String(value));
            }
            (StructuredFormat::Hosts, None) => {
                fields.insert("address".to_string(), serde_json::json!("127.0.0.1"));
                fields.insert("hostnames".to_string(), serde_json::json!(["new-host.local"]));
            }
            (StructuredFormat::PgHba, None) => {
                fields.insert("type".to_string(), serde_json::json!("host"));
                fields.insert("address".to_string(), serde_json::json!("127.0.0.1/32"));
            }
            (StructuredFormat::Sshd, None) => return,
        }
        self.apply_structured_op(file, EditOp::InsertRow { after_row_id: last_row, fields }, cx);
    }

    /// The editor for the selected config file, chosen by the plugin registry
    /// — the same lookup that decides the file's CROW UI label.
    pub fn config_editor_view(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let app = cx.entity();
        let selected = self.configs.selected_file.clone();
        let kind = self.configs.files.iter().find(|f| f.name == selected).and_then(|f| f.schema_kind);
        let editor = editor_for(kind);
        if let Some(confirm) = self.risk_confirm.as_mut().filter(|c| c.file == selected && c.input.is_none()) {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder(RISK_CONFIRM_KEYWORD));
            input.update(cx, |i, cx| i.focus(window, cx));
            confirm.input = Some(input);
        }

        match editor {
            ConfigEditor::Journald => {
                return journald_editor(&self.journal.retention, &self.journal.telemetry, &self.configs, app).into_any_element();
            }
            ConfigEditor::Screen(screen) => return screen_handoff(&selected, screen, app).into_any_element(),
            ConfigEditor::Structured(format) if !self.configs.text_mode.contains(&selected) => {
                if let Some(st) = self.configs.states.get(&selected) {
                    match plugins::to_ir(format, &st.current_content) {
                        Ok(ir) => {
                            let active = self.structured_field_edit.as_ref().filter(|e| e.file == selected).map(|e| ActiveFieldEdit {
                                row_id: &e.row_id,
                                field: &e.field,
                                input: &e.input,
                            });
                            let confirm = self.risk_confirm.as_ref().filter(|c| c.file == selected);
                            return structured_editor(st, format, &ir, &self.configs, active, confirm, app).into_any_element();
                        }
                        // Unparseable: fall through to the text editor.
                        Err(e) => self.configs.edit_error = Some(format!("{selected} could not be parsed: {e}")),
                    }
                }
            }
            _ => {}
        }

        let can_structure = matches!(editor, ConfigEditor::Structured(_));
        match self.config_text_editor(&selected, window, cx) {
            Some(text_editor) => raw_config_editor(&self.configs.states[&selected], &text_editor, can_structure, app).into_any_element(),
            None => div()
                .flex_1()
                .p(px(20.0))
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_FAINT)
                .child("No config file selected.")
                .into_any_element(),
        }
    }

    /// Opens the Crow screen that owns a config file.
    pub fn open_dedicated_screen(&mut self, screen: DedicatedScreen, cx: &mut Context<Self>) {
        match screen {
            DedicatedScreen::Cron => self.set_view("cron", cx),
            DedicatedScreen::Firewall => self.set_view("firewall", cx),
            DedicatedScreen::Users => self.set_view("users", cx),
            DedicatedScreen::Settings => self.set_screen(Screen::Settings, cx),
        }
    }

    pub fn toggle_config_history(&mut self, cx: &mut Context<Self>) {
        self.configs.show_history = !self.configs.show_history;
        cx.notify();
    }

    /// Selects a file; files owned by a dedicated screen open that screen.
    pub fn select_managed_file(&mut self, filename: &str, cx: &mut Context<Self>) {
        self.configs.selected_file = filename.to_string();
        self.configs.open_enum = None;
        self.configs.adding_row = false;
        self.configs.edit_error = None;
        self.structured_field_edit = None;
        let kind = self.configs.files.iter().find(|f| f.name == filename).and_then(|f| f.schema_kind);
        if let ConfigEditor::Screen(screen) = editor_for(kind) {
            self.open_dedicated_screen(screen, cx);
        }
        cx.notify();
    }

    /// The journald editor shows the settings of the loaded journald.conf.
    pub fn apply_journald_from_configs(&mut self) {
        if let Some(st) = self.configs.states.get("journald.conf") {
            self.journal.retention = parse_journald_conf(&st.current_content);
        }
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
        let fresh = load_configs(server.as_ref(), &self.firewall.status);
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
        let firewall = self.firewall.status.clone();
        cx.spawn(async move |entity, cx| {
            let loaded = cx
                .background_executor()
                .spawn(async move { load_configs(server.as_ref(), &firewall) })
                .await;
            let _ = entity.update(cx, |this, cx| {
                let still_active = this.fleet.active_server().map(|s| s.id) == target;
                if still_active && !this.configs.has_unsaved_changes() {
                    let selected = std::mem::take(&mut this.configs.selected_file);
                    this.configs = loaded;
                    this.apply_journald_from_configs();
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
        if state.write_blocked.is_none() {
            if let Some(format) = self.structured_format_of(file) {
                plugins::validate_on_host(host.as_ref(), format, &state.current_content).map_err(|e| format!("{file}: {e}"))?;
            }
        }
        state.save_to(host.as_ref()).map_err(|e| format!("{file}: {e}"))
    }

    #[allow(dead_code)]
    pub fn set_config_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.configs.search_query = query;
        cx.notify();
    }

    pub fn revert_managed_config(&mut self, file: &str, cx: &mut Context<Self>) {
        if file == "journald.conf" {
            if let Some(st) = self.configs.states.get(file) {
                self.journal.retention = parse_journald_conf(&st.baseline_content);
            }
        } else if file == "crontab" {
            if let Some(baseline) = self.configs.states.get(file).map(|st| st.baseline_content.clone()) {
                self.configs.reload_cron_from(&baseline);
            }
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
    /// them as a new revision. A failed write leaves the edits pending. A change
    /// introducing never-on-prod values first asks for typed confirmation.
    pub fn stage_config_version(&mut self, file: &str, description: &str, cx: &mut Context<Self>) {
        if let (Some(format), Some(st)) = (self.structured_format_of(file), self.configs.states.get(file)) {
            let findings = plugins::new_never_on_prod_values(format, &st.baseline_content, &st.current_content);
            if !findings.is_empty() {
                self.configs.selected_file = file.to_string();
                // The confirmation lives in the structured view.
                self.configs.text_mode.remove(file);
                self.risk_confirm = Some(RiskConfirm {
                    file: file.to_string(),
                    description: description.to_string(),
                    findings,
                    input: None,
                    error: None,
                });
                cx.notify();
                return;
            }
        }
        self.stage_config_version_confirmed(file, description, cx);
    }

    /// Stages the pending never-on-prod change if the keyword was typed.
    pub fn confirm_risky_stage(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.risk_confirm.as_mut() else { return };
        let typed = pending.input.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        if typed != RISK_CONFIRM_KEYWORD {
            pending.error = Some(format!("Type {RISK_CONFIRM_KEYWORD} exactly to stage this change."));
            cx.notify();
            return;
        }
        let pending = self.risk_confirm.take().expect("checked above");
        self.stage_config_version_confirmed(&pending.file, &pending.description, cx);
    }

    pub fn cancel_risky_stage(&mut self, cx: &mut Context<Self>) {
        self.risk_confirm = None;
        cx.notify();
    }

    fn stage_config_version_confirmed(&mut self, file: &str, description: &str, cx: &mut Context<Self>) {
        let author = self.default_author();
        if let Err(e) = self.write_config_file(file) {
            self.configs.save_error = Some(e);
            cx.notify();
            return;
        }
        self.configs.save_error = None;
        if let Some(state) = self.configs.states.get_mut(file) {
            state.stage_revision(author, description.to_string());
            let staged = state.baseline_content.clone();
            if file == "crontab" {
                // Job ids are line numbers in the text they came from; re-read.
                self.configs.reload_cron_from(&staged);
            }
            if file == "journald.conf" {
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
        if let Some(text) = self.configs.states.get(file).map(|st| st.current_content.clone()) {
            match file {
                "crontab" => self.configs.reload_cron_from(&text),
                "journald.conf" => self.journal.retention = parse_journald_conf(&text),
                _ => {}
            }
        }
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

    // --- SSH Key Management Subsystem ---
}
