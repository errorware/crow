use crow_config_core::ConfigPlugin;
use crow_config_core::edit::ConfigDocument;
use crow_config_schemas::PgHbaPlugin;
use gpui_kit::*;

use std::collections::HashMap;

use crow_config_core::edit::EditOp;
use gpui_kit::component::input::InputState;

use super::CrowApp;
use crate::config::plugins::{self, apply_edit as apply_structured_edit, editor_for, ConfigEditor, RiskFinding, StructuredFormat};
use crate::theme::{FONT_MONO, TEXT_FAINT};
use crate::views::config::journald_editor::journald_editor;
use crate::views::config::raw_config_editor;
use crate::views::config::structured_editor::{structured_editor, ActiveFieldEdit};
use gpui_kit::component::input::{EditorState, InputEvent};
use crate::views::config::text_editor::{highlighter_factory, CONFIG_LANGUAGE};
use std::sync::Arc;

use crate::config::{crawl_all_configs, load_config_file_states};
use crate::host::{host_for, Host, LocalHost};
use crate::os_detect::{classify_distro_family, detect_os_release, DistroFamily};
use crate::config::{drift, history};
use crate::views::config::state::AppliedBaseline;
use crate::vault::{ChangeRecord, KeyringState, ServerRecord, VaultError, BASELINE_FLEET};

/// "the fleet" or "group web".
pub fn scope_label(scope: &str) -> String {
    if scope == BASELINE_FLEET { "the fleet".into() } else { format!("group {scope}") }
}
use crate::views::config::state::ConfigsState;
use crate::views::firewall::generate_user_rules_content;
use crate::journal::retention::parse_journald_conf;
use crate::views::firewall::FirewallOperationalState;

pub struct ConfigSearchInput {
    pub input: Entity<InputState>,
    pub _events: Subscription,
}

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

pub fn user_rules_read_only_reason() -> &'static str {
    READ_ONLY_FILES[0].1
}

/// Discovers and loads config files from `server` (this machine when there are
/// no servers) and blocks writes wherever the content isn't the host's real file.
pub fn load_configs(server: Option<&ServerRecord>, firewall: &FirewallOperationalState) -> ConfigsState {
    let host: Arc<dyn Host> = match server {
        Some(srv) => host_for(srv),
        None => Arc::new(LocalHost),
    };
    let family = detect_os_release(host.as_ref()).map(|d| classify_distro_family(&d)).unwrap_or(DistroFamily::Unknown);
    // States for every file (the Cron screen edits crontab's); the list
    // only shows files the Config screen edits itself.
    let mut all = crawl_all_configs(host.as_ref(), family);
    let mut states = load_config_file_states(host.as_ref(), &all);
    // sudoers is root-only: read apart, as root (ERR-22).
    for (file, state) in crate::config::sudoers::load_sudoers(host.as_ref()) {
        states.insert(file.name.clone(), state);
        all.push(file);
    }
    let files: Vec<_> = all.into_iter().filter(|f| editor_for(f.schema_kind).is_listed()).collect();
    let selected = files.first().map(|f| f.name.clone()).unwrap_or_else(|| "journald.conf".to_string());

    // sshd's included drop-ins (ERR-12): read through the same transport.
    let sshd_includes = states
        .get("sshd_config")
        .and_then(|st| plugins::to_ir(StructuredFormat::Sshd, &st.current_content).ok())
        .map(|ir| plugins::load_sshd_includes(host.as_ref(), &ir))
        .unwrap_or_default();
    let mut configs = ConfigsState::new(files, states, selected);
    configs.sshd_includes = sshd_includes;
    configs.server_id = server.map(|s| s.id.clone());
    configs.family = family;
    if let Some(crontab) = configs.states.get("crontab").map(|st| st.current_content.clone()) {
        configs.reload_cron_from(&crontab);
    }
    // ufw's own rules file, mirrored from its status; not for other backends.
    if let FirewallOperationalState::Active(ref summary) = firewall.clone().ufw_only() {
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

/// A change to a file that can lock you out, on a server whose provider can
/// snapshot it: offered a whole-server snapshot first (ERR-48).
pub struct SnapshotOffer {
    pub file: String,
    pub description: String,
    pub provider: String,
    /// Remember the choice for next time.
    pub remember: bool,
    /// A snapshot is being taken; the change waits for it.
    pub busy: bool,
    pub error: Option<String>,
}

/// Files whose mistakes cut Crow (and you) off from the server.
pub fn is_lockout_risk(file: &str) -> bool {
    let name = file.rsplit('/').next().unwrap_or(file);
    name == "sshd_config" || file.contains("sshd_config.d/") || name == "user.rules" || name == "user6.rules"
}

/// app_flags key for the remembered choice: "always" or "never".
const SNAPSHOT_PREF: &str = "apply.snapshot_before_lockout_risk";

/// Row id of an inline input for a directive not yet in the file.
pub const NEW_DIRECTIVE_PREFIX: &str = "new:";

/// What must be typed to stage a never-on-prod change.
pub const RISK_CONFIRM_KEYWORD: &str = "CONFIRM";

/// Author recorded on staged config revisions.
/// Who revisions are attributed to: this machine's user (ERR-72).
pub fn default_author() -> String {
    static AUTHOR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    AUTHOR.get_or_init(crate::config::history::local_author).clone()
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
        self.configs.structured_format(file)
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

    /// Opens an inline input for a directive that isn't in the file yet;
    /// Enter adds it with the typed value (nothing is added if left empty).
    pub fn begin_new_directive(&mut self, file: &str, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.begin_structured_field_edit(file, &format!("{NEW_DIRECTIVE_PREFIX}{name}"), name, "", false, window, cx);
    }

    pub fn commit_structured_field_edit(&mut self, cx: &mut Context<Self>) {
        let Some(edit) = self.structured_field_edit.take() else { return };
        let text = edit.input.read(cx).value().trim().to_string();
        if edit.row_id.starts_with(NEW_DIRECTIVE_PREFIX) {
            if !text.is_empty() {
                self.insert_structured_row(&edit.file, Some((edit.field.clone(), text)), cx);
            } else {
                cx.notify();
            }
            return;
        }
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
        let ir = plugins::to_ir(format, &state.current_content).ok();
        // sshd: after the last global line, never inside a trailing Match block.
        let last_row = match format {
            StructuredFormat::Sshd => ir.as_ref().and_then(|ir| plugins::sshd_sheet(ir, &[]).insert_after),
            _ => ir.as_ref().and_then(|ir| ir.rows.last().map(|r| r.row_id.clone())),
        };
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
            // The kernel's default, so a new row changes nothing until edited.
            (StructuredFormat::Sysctl, None) => {
                fields.insert("key".to_string(), serde_json::json!("vm.swappiness"));
                fields.insert("value".to_string(), serde_json::json!("60"));
            }
            // A rule that grants nothing until it's edited.
            (StructuredFormat::Sudoers, None) => {
                fields.insert("who".to_string(), serde_json::json!("nobody"));
                fields.insert("rule".to_string(), serde_json::json!("ALL=(root) /usr/bin/true"));
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
            ConfigEditor::Structured(format) if !self.configs.text_mode.contains(&selected) => {
                if format == StructuredFormat::Sysctl {
                    if let Some(ir) = self.configs.states.get(&selected).and_then(|st| plugins::to_ir(format, &st.current_content).ok()) {
                        self.ensure_sysctl_live(&ir, cx);
                    }
                }
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

    pub fn toggle_config_history(&mut self, cx: &mut Context<Self>) {
        self.configs.show_history = !self.configs.show_history;
        cx.notify();
    }

    pub fn select_managed_file(&mut self, filename: &str, cx: &mut Context<Self>) {
        self.configs.selected_file = filename.to_string();
        self.configs.open_enum = None;
        self.configs.adding_row = false;
        self.configs.edit_error = None;
        self.structured_field_edit = None;
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
                    // The firewall may have been read while these loaded.
                    if let FirewallOperationalState::Active(ref summary) = this.firewall.status.clone().ufw_only() {
                        let text = generate_user_rules_content(&summary.rules);
                        this.configs.seed_baseline("user.rules", text, Some("/etc/ufw/user.rules"));
                        this.configs.block_writes("user.rules", user_rules_read_only_reason().to_string());
                    }
                    this.apply_journald_from_configs();
                    if this.configs.states.contains_key(&selected) {
                        this.configs.selected_file = selected;
                    }
                    this.sync_config_history(cx);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Reads, in the background, the running value of each key in a sysctl
    /// file that hasn't been read yet on this server.
    fn ensure_sysctl_live(&mut self, ir: &crow_config_core::ir::ConfigDocumentIr, cx: &mut Context<Self>) {
        if self.configs.sysctl_live_loading {
            return;
        }
        let keys: Vec<String> = ir
            .rows
            .iter()
            .filter_map(|r| r.get_field("key")?.value.as_str().map(str::to_string))
            .filter(|k| !k.is_empty() && !self.configs.sysctl_live.contains_key(k))
            .collect();
        if keys.is_empty() {
            return;
        }
        let host: Arc<dyn Host> = match self.configs_server() {
            Some(srv) => host_for(&srv),
            None => Arc::new(LocalHost),
        };
        let server = self.configs.server_id.clone();
        self.configs.sysctl_live_loading = true;
        cx.spawn(async move |entity, cx| {
            let asked = keys.clone();
            let live = cx.background_executor().spawn(async move { crate::config::sysctl_live::read_live(host.as_ref(), &keys) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.configs.sysctl_live_loading = false;
                // The server changed meanwhile: these values aren't its.
                if this.configs.server_id != server {
                    return;
                }
                for key in asked {
                    let value = live.get(&key).cloned().unwrap_or(crate::config::sysctl_live::LiveValue::Unreadable);
                    this.configs.sysctl_live.insert(key, value);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes one file's current content back to the configs' server.
    fn write_config_file(&self, file: &str) -> Result<(), String> {
        // Configs with no server are this machine's; with servers enrolled
        // that only happens while the active server's are still loading,
        // and writing then would land on the wrong machine.
        if self.configs.server_id.is_none() && !self.fleet.servers.is_empty() {
            return Err("this server's configs are still loading; nothing was written".into());
        }
        let state = self.configs.states.get(file).ok_or_else(|| format!("{file} is not loaded"))?;
        let host: Arc<dyn Host> = match self.configs_server() {
            Some(srv) => host_for(&srv),
            None => Arc::new(LocalHost),
        };
        let format = self.structured_format_of(file);
        // sudoers: visudo and the keep-sudo guard run in the install itself.
        if format == Some(StructuredFormat::Sudoers) {
            if let Some(reason) = &state.write_blocked {
                return Err(reason.clone());
            }
            // Who Crow is there: the server's login, or this machine's user.
            let login = self.configs_server().map(|s| s.login_user.clone()).unwrap_or_else(|| std::env::var("USER").unwrap_or_default());
            return crate::config::sudoers::install_sudoers(host.as_ref(), &state.path.to_string_lossy(), &state.current_content, &login).map_err(|e| format!("{file}: {e}"));
        }
        if state.write_blocked.is_none() {
            if let Some(format) = format {
                plugins::validate_on_host(host.as_ref(), format, &state.current_content).map_err(|e| format!("{file}: {e}"))?;
            }
        }
        state.save_to(host.as_ref()).map_err(|e| format!("{file}: {e}"))
    }

    pub fn ensure_config_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.config_search.is_none() {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Filter configs...")
                    .default_value(&self.configs.search_query)
            });
            let events = cx.subscribe(&input, |this, input, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.configs.search_query = input.read(cx).value().to_string();
                    cx.notify();
                }
            });
            self.config_search = Some(ConfigSearchInput { input, _events: events });
        }
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
        self.stage_or_offer_snapshot(file, description, cx);
    }

    /// Before a lockout-risk change on a server whose provider can snapshot,
    /// offers a snapshot first (or follows the remembered choice).
    fn stage_or_offer_snapshot(&mut self, file: &str, description: &str, cx: &mut Context<Self>) {
        let provider = self.active_provider_actions().filter(|p| p.snapshots);
        let Some(provider) = provider.filter(|_| is_lockout_risk(file)) else {
            return self.stage_config_version_confirmed(file, description, cx);
        };
        let pref = self.vault.db().lock().ok().and_then(|db| db.flag(SNAPSHOT_PREF));
        match pref.as_deref() {
            Some("never") => self.stage_config_version_confirmed(file, description, cx),
            Some("always") => self.snapshot_then_stage(file.to_string(), description.to_string(), cx),
            _ => {
                self.snapshot_offer = Some(SnapshotOffer { file: file.to_string(), description: description.to_string(), provider: provider.name, remember: false, busy: false, error: None });
                cx.notify();
            }
        }
    }

    /// The offer's answer: `snapshot` or apply without; optionally remembered.
    pub fn answer_snapshot_offer(&mut self, snapshot: bool, cx: &mut Context<Self>) {
        let Some(offer) = self.snapshot_offer.as_ref() else { return };
        if offer.busy {
            return;
        }
        if offer.remember {
            if let Ok(db) = self.vault.db().lock() {
                let _ = db.set_flag(SNAPSHOT_PREF, if snapshot { "always" } else { "never" });
            }
        }
        let (file, description) = (offer.file.clone(), offer.description.clone());
        if snapshot {
            self.snapshot_then_stage(file, description, cx);
        } else {
            self.snapshot_offer = None;
            self.stage_config_version_confirmed(&file, &description, cx);
        }
    }

    pub fn cancel_snapshot_offer(&mut self, cx: &mut Context<Self>) {
        if self.snapshot_offer.as_ref().is_some_and(|o| !o.busy) {
            self.snapshot_offer = None;
            cx.notify();
        }
    }

    pub fn toggle_snapshot_offer_remember(&mut self, cx: &mut Context<Self>) {
        if let Some(o) = self.snapshot_offer.as_mut() {
            o.remember = !o.remember;
            cx.notify();
        }
    }

    /// Asks the provider for a snapshot and applies the change only once it
    /// was accepted; the revision notes the restore point.
    fn snapshot_then_stage(&mut self, file: String, description: String, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let Some(account) = self.providers.accounts.iter().find(|a| a.id == srv.provider_account).cloned() else { return };
        let provider_name = crate::providers::display_name(&account.plugin);
        let settings = match (self.vault.key(), self.vault.db().lock()) {
            (Some(key), Ok(db)) => crate::providers::load_settings(&db, key, &account),
            _ => Err(self.vault.secrets_blocker().unwrap_or_else(|| "the vault is busy".into())),
        };
        let offer = self.snapshot_offer.get_or_insert_with(|| SnapshotOffer { file: file.clone(), description: description.clone(), provider: provider_name.clone(), remember: false, busy: false, error: None });
        let settings = match settings {
            Ok(s) => s,
            Err(e) => {
                offer.error = Some(format!("No snapshot, nothing applied: {e}"));
                cx.notify();
                return;
            }
        };
        offer.busy = true;
        offer.error = None;
        cx.notify();
        let instance = srv.provider_instance.clone();
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let provider = crate::providers::connect(&account, settings).map_err(|e| e.to_string())?;
                    crate::providers::run_action(provider.as_ref(), &instance, crate::providers::ProviderAction::Snapshot)
                })
                .await;
            let _ = entity.update(cx, |this, cx| match result {
                Ok(msg) => {
                    this.snapshot_offer = None;
                    this.push_journal_action_marker(format!("crow: {msg}"));
                    this.stage_config_version_confirmed(&file, &format!("{description} · restore point: {msg}"), cx);
                }
                Err(e) => {
                    if let Some(o) = this.snapshot_offer.as_mut() {
                        o.busy = false;
                        o.error = Some(format!("The snapshot failed, so nothing was applied: {e}"));
                    }
                    cx.notify();
                }
            });
        })
        .detach();
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
        self.stage_or_offer_snapshot(&pending.file, &pending.description, cx);
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
        let before = self.configs.states.get(file).map(|st| st.baseline_content.clone()).unwrap_or_default();
        if let Some(state) = self.configs.states.get_mut(file) {
            state.stage_revision(author, description.to_string());
        }
        self.record_config_write(file, &before, description, "config.write");
        if let Some(state) = self.configs.states.get_mut(file) {
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
        self.configs.preview_revision = None;
        // What's on the host now is the restored revision.
        if let Some(st) = self.configs.states.get_mut(file) {
            st.baseline_content = st.current_content.clone();
        }
        self.record_config_write(file, &before.baseline_content, &format!("Restored v{version}"), "config.restore");
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

    /// Records a successful write of `file` in its history and the audit
    /// trail (ERR-72): a revision holding what's on the host now, and a
    /// change record from `before` to it. The file is already written, so a
    /// failure here is reported but undoes nothing.
    fn record_config_write(&mut self, file: &str, before: &str, message: &str, action_kind: &str) {
        let Some(st) = self.configs.states.get(file) else { return };
        if !st.read_from_host {
            return;
        }
        let path = st.path.to_string_lossy().into_owned();
        let content = st.baseline_content.clone();
        let server_id = self.configs.server_id.clone().unwrap_or_else(|| history::LOCAL_SERVER_ID.to_string());
        let server_name = self.configs_server().map(|s| s.name).unwrap_or_else(|| "this machine".into());
        let key = self.vault.key().cloned();
        let db = self.vault.db();
        let result = db.lock().map_err(|_| VaultError::Crypto("the vault is busy".into())).and_then(|db| {
            let rev = history::record(&db, key.as_ref(), &server_id, &path, &content, &self.default_author(), message, history::SOURCE_CROW)?;
            let now = chrono::Utc::now().to_rfc3339();
            db.insert_change_record(&ChangeRecord {
                id: format!("chg-{}", rev.id.trim_start_matches("cfgrev-")),
                server_id: server_id.clone(),
                server_name,
                action_kind: action_kind.into(),
                target: path.clone(),
                before_state: format!("sha256:{}", history::sha256_hex(before)),
                after_state: Some(format!("sha256:{}", rev.sha256)),
                blast_radius: None,
                outcome: "applied".into(),
                started_at: now.clone(),
                completed_at: Some(now),
            })?;
            let st = self.configs.states.get_mut(file).expect("checked above");
            history::load_into(&db, key.as_ref(), &server_id, st)
        });
        self.refresh_drift();
        if let Err(e) = result {
            self.configs.history_error = Some(format!("{file} was written, but its history wasn't recorded: {e}"));
        }
    }

    /// Recomputes baselines in force for the loaded configs' server and the
    /// fleet's drift from them (ERR-74). Reads recorded history only.
    pub fn refresh_drift(&mut self) {
        let key = self.vault.key().cloned();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        self.fleet.drift = drift::fleet_drift(&db, key.as_ref(), &self.fleet.servers).ok();
        let group = self.configs_server().map(|s| s.group_name).unwrap_or_default();
        self.configs.baseline_scope = if group.trim().is_empty() { BASELINE_FLEET.to_string() } else { group.trim().to_string() };
        let baselines = db.list_config_baselines().unwrap_or_default();
        let names: HashMap<&str, &str> = self.fleet.servers.iter().chain(&self.fleet.archived).map(|s| (s.id.as_str(), s.name.as_str())).collect();
        self.configs.baselines = self
            .configs
            .states
            .values()
            .filter_map(|st| {
                let path = st.path.to_string_lossy().into_owned();
                let b = drift::baseline_for(&baselines, &path, &group)?.clone();
                let rev = db.get_config_revision(&b.revision_id).ok()??;
                let applied = AppliedBaseline {
                    from_server: names.get(rev.server_id.as_str()).map(|n| n.to_string()).unwrap_or_else(|| rev.server_id.clone()),
                    content: history::open_content(&rev, key.as_ref()),
                    sha256: rev.sha256,
                    baseline: b,
                };
                Some((path, applied))
            })
            .collect();
    }

    /// Pins revision `version` of `file` as the known-good version for this
    /// server's group, or the fleet when it has none (ERR-74).
    pub fn pin_config_baseline(&mut self, file: &str, version: usize, cx: &mut Context<Self>) {
        let Some(st) = self.configs.states.get(file) else { return };
        let Some(rev) = st.revisions.iter().find(|r| r.version == version && !r.id.is_empty()) else { return };
        let path = st.path.to_string_lossy().into_owned();
        let scope = self.configs.baseline_scope.clone();
        let result = self.vault.db().lock().map_err(|_| VaultError::Crypto("the vault is busy".into())).and_then(|db| {
            db.set_config_baseline(&path, &scope, &rev.id, &self.default_author())?;
            self.audit_baseline(&db, &path, &format!("pinned v{version} for {}", scope_label(&scope)))
        });
        self.configs.history_error = result.err().map(|e| format!("baseline not pinned: {e}"));
        self.refresh_drift();
        cx.notify();
    }

    pub fn unpin_config_baseline(&mut self, path: &str, scope: &str, cx: &mut Context<Self>) {
        let result = self.vault.db().lock().map_err(|_| VaultError::Crypto("the vault is busy".into())).and_then(|db| {
            db.clear_config_baseline(path, scope)?;
            self.audit_baseline(&db, path, &format!("unpinned for {}", scope_label(scope)))
        });
        self.configs.history_error = result.err().map(|e| format!("baseline not unpinned: {e}"));
        self.refresh_drift();
        cx.notify();
    }

    /// Puts the baseline's text in the editor as an unsaved edit; saving it
    /// goes through the usual diff, validator and history.
    pub fn load_baseline_into_editor(&mut self, file: &str, cx: &mut Context<Self>) {
        let Some(path) = self.configs.states.get(file).map(|st| st.path.to_string_lossy().into_owned()) else { return };
        let Some(content) = self.configs.baselines.get(&path).and_then(|b| b.content.clone()) else { return };
        if let Some(st) = self.configs.states.get_mut(file) {
            st.update_content(content);
        }
        cx.notify();
    }

    fn audit_baseline(&self, db: &crate::vault::VaultDb, path: &str, what: &str) -> Result<(), VaultError> {
        let now = chrono::Utc::now().to_rfc3339();
        let srv = self.configs_server();
        db.insert_change_record(&ChangeRecord {
            id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
            server_id: srv.as_ref().map(|s| s.id.clone()).unwrap_or_else(|| history::LOCAL_SERVER_ID.into()),
            server_name: srv.map(|s| s.name).unwrap_or_else(|| "this machine".into()),
            action_kind: "config.baseline".into(),
            target: path.into(),
            before_state: what.into(),
            after_state: None,
            blast_radius: None,
            outcome: "success".into(),
            started_at: now.clone(),
            completed_at: Some(now),
        })
    }

    /// Reconciles the loaded configs with their recorded history (ERR-72).
    /// Waits for the data key while the keyring is still being read (called
    /// again from `on_data_key_ready`), and makes one if the keyring is
    /// empty: sealing history only ever makes things safer, so no stance
    /// prompt. With no keyring at all, only hashes are kept.
    pub fn sync_config_history(&mut self, cx: &mut Context<Self>) {
        if self.vault.is_password_auth_enabled() {
            if !self.vault.is_unlocked() {
                return;
            }
        } else {
            match self.vault.keyring_state {
                KeyringState::Loading => return,
                KeyringState::Empty if self.vault.key().is_none() => {
                    self.ensure_data_key(cx, |this, _, cx| this.sync_config_history_now(cx));
                    return;
                }
                _ => {}
            }
        }
        self.sync_config_history_now(cx);
    }

    fn sync_config_history_now(&mut self, cx: &mut Context<Self>) {
        let server_id = self.configs.server_id.clone().unwrap_or_else(|| history::LOCAL_SERVER_ID.to_string());
        let key = self.vault.key().cloned();
        let db = self.vault.db();
        let result = match db.lock() {
            Ok(db) => history::sync(&db, key.as_ref(), &server_id, &mut self.configs.states),
            Err(_) => return,
        };
        self.configs.history_sealed = Some(key.is_some());
        self.refresh_drift();
        self.configs.history_error = result.err().map(|e| format!("couldn't be read or recorded: {e}"));
        cx.notify();
    }

    pub fn apply_journal_boundaries(&mut self, cx: &mut Context<Self>) {
        self.stage_config_version("journald.conf", "Applied journald retention boundaries", cx);
    }

    // --- SSH Key Management Subsystem ---
}

#[cfg(test)]
mod lockout_tests {
    use super::is_lockout_risk;

    #[test]
    fn sshd_and_firewall_rules_are_lockout_risks() {
        for f in ["sshd_config", "/etc/ssh/sshd_config", "sshd_config.d/50-crow.conf", "user.rules", "/etc/ufw/user6.rules"] {
            assert!(is_lockout_risk(f), "{f}");
        }
        for f in ["hosts", "crontab", "journald.conf", "pg_hba.conf"] {
            assert!(!is_lockout_risk(f), "{f}");
        }
    }
}
