use gpui_kit::*;
use crate::theme::*;
use crate::components::danger_zone::danger_zone;
use crate::components::identity_bar::identity_bar;
use crate::components::palette::palette_overlay;
use crate::components::sidebar::sidebar;
use crate::components::stat_strip::stat_strip;
use crate::components::titlebar::{burger_menu_overlay, titlebar, ServerTab};
use crate::vault::{ServerRecord, Vault, VaultStatus};
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::config::rules_editor::{default_hba_rules, rules_editor, HbaRuleDef};
use crate::views::fleet::{fleet_overview_view, fleet_setup_view};
use crate::views::lock::{
    vault_lock_view, vault_setup_view, LockFieldFocus, LockState, SetupFieldFocus, SetupState, SetupStep,
};
use crate::views::onboard::{
    append_to_known_hosts, onboard_view, probe_host, OnboardFieldFocus, OnboardState, OnboardStep,
};
use crate::views::overview::log_tail::log_tail;
use crate::views::overview::services_table::{default_services, services_table, ServiceUnit};
use crate::views::settings::settings_view;

use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;
use crate::config::CrowConfigManager;
use crate::keys::{
    copy_to_clipboard_system, expand_tilde, scan_directory, AddScanPathModalState,
    DiscoveredKey, EditKeyModalState, KeyGenFieldFocus, KeyGenModalState,
    NewGroupModalState, SshKeyGroup, SshKeyRecord, SshScanPath,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Server,
    Fleet,
    Settings,
    Onboard,
    FleetSetup,
    VaultSetup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Connection,
    Keys,
    Security,
}

impl SettingsSection {
    pub fn id_prefix(&self) -> &'static str {
        match self {
            SettingsSection::General => "general",
            SettingsSection::Connection => "connection",
            SettingsSection::Keys => "keys",
            SettingsSection::Security => "security",
        }
    }
}

pub struct CrowApp {
    focus_handle: FocusHandle,
    pub vault: Vault,
    pub config: CrowConfigManager,
    pub lock_state: LockState,
    pub setup_state: SetupState,
    pub screen: Screen,
    pub menu_open: bool,
    pub settings_section: SettingsSection,
    pub active_tab_id: String,
    pub active_view: String,
    pub active_services_tab: String,
    pub tabs: Vec<ServerTab>,
    pub services: Vec<ServiceUnit>,
    pub hba_rules: Vec<HbaRuleDef>,
    pub palette_open: bool,
    pub sidebar_collapsed: bool,
    pub settings_dropdown_open: Option<String>,
    pub settings_custom_input: String,
    // SSH Key Management Hub
    pub enrolled_keys: Vec<SshKeyRecord>,
    pub key_groups: Vec<SshKeyGroup>,
    pub scan_paths: Vec<SshScanPath>,
    pub discovered_keys: Vec<DiscoveredKey>,
    pub selected_key_group_filter: Option<String>,
    pub scan_status_message: Option<String>,
    pub key_gen_modal: Option<KeyGenModalState>,
    pub new_group_modal: Option<NewGroupModalState>,
    pub add_scan_path_modal: Option<AddScanPathModalState>,
    pub edit_key_modal: Option<EditKeyModalState>,
    pub key_toast: Option<String>,
    // Server Enrollment Subsystem
    pub servers: Vec<ServerRecord>,
    pub onboard_state: OnboardState,
    // Text input caret and selection state
    pub cursor_blink: bool,
    pub input_cursor: usize,
    pub input_selection: Option<(usize, usize)>,
    pub input_drag_anchor: Option<usize>,
    pub _cursor_blink_task: Task<()>,
}

impl CrowApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let vault = Vault::open_default().expect("Failed to initialize vault storage");
        let config = CrowConfigManager::load();

        // Load servers, SSH Key Management records and run initial scan
        let (servers, enrolled_keys, key_groups, scan_paths, discovered_keys, scan_status_message) = {
            let db = vault.db();
            let res = if let Ok(db_guard) = db.lock() {
                let paths = db_guard.list_scan_paths().unwrap_or_default();
                let groups = db_guard.list_key_groups().unwrap_or_default();
                let keys = db_guard.list_ssh_keys().unwrap_or_default();
                let srvs = db_guard.list_servers().unwrap_or_default();
                let mut discovered = Vec::new();
                for p in &paths {
                    let expanded = expand_tilde(&p.path);
                    let found = scan_directory(&expanded, &keys);
                    for k in found {
                        if !discovered.iter().any(|d: &DiscoveredKey| d.fingerprint == k.fingerprint) {
                            discovered.push(k);
                        }
                    }
                }
                let new_count = discovered.iter().filter(|d| !d.is_enrolled).count();
                let msg = format!(
                    "Scanned {} path{} · {} key{} found ({} new)",
                    paths.len(),
                    if paths.len() == 1 { "" } else { "s" },
                    discovered.len(),
                    if discovered.len() == 1 { "" } else { "s" },
                    new_count
                );
                (srvs, keys, groups, paths, discovered, Some(msg))
            } else {
                (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), None)
            };
            res
        };

        // Demonstrate integration with crow-config-core & crow-config-schemas
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

        let tabs = if !servers.is_empty() {
            servers.iter().take(5).map(|s| {
                ServerTab {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    status_color: match s.status.as_str() {
                        "online" => OK,
                        "warn" => WARN,
                        "crit" => CRIT,
                        _ => TEXT_FAINTER,
                    },
                    is_active: s.id == "edge-01",
                }
            }).collect()
        } else {
            vec![
                ServerTab { id: "edge-01".into(), name: "edge-01".into(), status_color: OK, is_active: true },
                ServerTab { id: "edge-02".into(), name: "edge-02".into(), status_color: OK, is_active: false },
                ServerTab { id: "db-primary".into(), name: "db-primary".into(), status_color: WARN, is_active: false },
                ServerTab { id: "worker-04".into(), name: "worker-04".into(), status_color: CRIT, is_active: false },
                ServerTab { id: "bastion".into(), name: "bastion".into(), status_color: TEXT_FAINTER, is_active: false },
            ]
        };

        let onboard_state = OnboardState::new(&enrolled_keys);

        Self {
            focus_handle: cx.focus_handle(),
            vault,
            config,
            lock_state: LockState::default(),
            setup_state: SetupState::default(),
            screen: Screen::Fleet,
            menu_open: false,
            settings_section: SettingsSection::General,
            active_tab_id: "edge-01".to_string(),
            active_view: "overview".to_string(),
            active_services_tab: "services".to_string(),
            tabs,
            services: default_services(),
            hba_rules: default_hba_rules(),
            palette_open: false,
            sidebar_collapsed: false,
            settings_dropdown_open: None,
            settings_custom_input: String::new(),
            enrolled_keys,
            key_groups,
            scan_paths,
            discovered_keys,
            selected_key_group_filter: None,
            scan_status_message,
            key_gen_modal: None,
            new_group_modal: None,
            add_scan_path_modal: None,
            edit_key_modal: None,
            key_toast: None,
            servers,
            onboard_state,
            cursor_blink: true,
            input_cursor: 0,
            input_selection: None,
            input_drag_anchor: None,
            _cursor_blink_task: cx.spawn(async move |entity, cx| {
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(530)).await;
                    if entity.update(cx, |this, cx| {
                        this.cursor_blink = !this.cursor_blink;
                        cx.notify();
                    }).is_err() {
                        break;
                    }
                }
            }),
        }
    }

    pub fn lock(&mut self, cx: &mut Context<Self>) {
        if self.vault.is_password_auth_enabled() {
            self.vault.lock();
            self.lock_state.password_input.clear();
            self.lock_state.totp_input.clear();
            self.lock_state.error_message = None;
            self.input_cursor = 0;
            self.input_selection = None;
            self.menu_open = false;
            self.palette_open = false;
            cx.notify();
        } else {
            self.screen = Screen::Settings;
            self.settings_section = SettingsSection::Security;
            self.menu_open = false;
            self.palette_open = false;
            cx.notify();
        }
    }

    pub fn submit_unlock(&mut self, cx: &mut Context<Self>) {
        let pwd = self.lock_state.password_input.clone();
        if pwd.is_empty() {
            self.lock_state.error_message = Some("Password cannot be empty".into());
            cx.notify();
            return;
        }
        let totp = self.lock_state.totp_input.trim();
        if totp.is_empty() {
            self.lock_state.error_message = Some("6-digit 2FA code is required".into());
            cx.notify();
            return;
        }

        match self.vault.unlock(&pwd, totp) {
            Ok(_) => {
                self.lock_state.password_input.clear();
                self.lock_state.totp_input.clear();
                self.lock_state.error_message = None;
                self.input_cursor = 0;
                self.input_selection = None;
                cx.notify();
            }
            Err(e) => {
                self.lock_state.error_message = Some(e.to_string());
                cx.notify();
            }
        }
    }

    pub fn submit_setup(&mut self, cx: &mut Context<Self>) {
        let pwd = self.setup_state.password_input.clone();
        let confirm = self.setup_state.confirm_input.clone();

        if pwd.len() < 8 {
            self.setup_state.error_message = Some("Password must be at least 8 characters".into());
            cx.notify();
            return;
        }
        if pwd != confirm {
            self.setup_state.error_message = Some("Passwords do not match".into());
            cx.notify();
            return;
        }

        let code = self.setup_state.totp_confirm_input.trim();
        if code.is_empty() {
            self.setup_state.error_message = Some("Please enter the 6-digit confirmation code from your authenticator".into());
            cx.notify();
            return;
        }
        if !crate::vault::verify_totp_code(&self.setup_state.totp_secret, code) {
            self.setup_state.error_message = Some("Invalid 6-digit code. Check your authenticator and try again".into());
            cx.notify();
            return;
        }

        match self.vault.initialize(&pwd, &self.setup_state.totp_secret) {
            Ok(_) => {
                self.setup_state = SetupState::default();
                self.input_cursor = 0;
                self.input_selection = None;
                self.screen = Screen::Fleet;
                cx.notify();
            }
            Err(e) => {
                self.setup_state.error_message = Some(e.to_string());
                cx.notify();
            }
        }
    }

    pub fn set_screen(&mut self, screen: Screen, cx: &mut Context<Self>) {
        if screen == Screen::Onboard && self.screen != Screen::Onboard {
            self.onboard_state = OnboardState::new(&self.enrolled_keys);
            self.input_cursor = self.onboard_state.host.chars().count();
            self.input_selection = None;
        } else if screen == Screen::VaultSetup && self.screen != Screen::VaultSetup {
            self.setup_state = SetupState::default();
            self.input_cursor = 0;
            self.input_selection = None;
        }
        self.screen = screen;
        self.menu_open = false;
        self.palette_open = false;
        cx.notify();
    }

    pub fn toggle_menu(&mut self, cx: &mut Context<Self>) {
        self.menu_open = !self.menu_open;
        if self.menu_open {
            self.palette_open = false;
        }
        cx.notify();
    }

    pub fn close_menu(&mut self, cx: &mut Context<Self>) {
        self.menu_open = false;
        cx.notify();
    }

    pub fn set_settings_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.settings_section = section;
        cx.notify();
    }

    pub fn update_config_field(&mut self, row_id: &str, new_value: serde_json::Value, cx: &mut Context<Self>) {
        if let Err(e) = self.config.update_field(row_id, new_value) {
            eprintln!("Failed to update config field {}: {:?}", row_id, e);
        }
        cx.notify();
    }

    pub fn reset_config_section(&mut self, sec_prefix: &str, cx: &mut Context<Self>) {
        if let Err(e) = self.config.reset_section(sec_prefix) {
            eprintln!("Failed to reset config section {}: {:?}", sec_prefix, e);
        }
        cx.notify();
    }

    pub fn save_config(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {:?}", e);
        }
        cx.notify();
    }

    pub fn open_config_file(&self) {
        let path = &self.config.path;
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(path).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/c", "start", ""]).arg(path).spawn();
    }

    pub fn toggle_settings_dropdown(&mut self, row_id: &str, initial_val: &str, cx: &mut Context<Self>) {
        if self.settings_dropdown_open.as_deref() == Some(row_id) {
            self.settings_dropdown_open = None;
            self.settings_custom_input.clear();
        } else {
            self.settings_dropdown_open = Some(row_id.to_string());
            self.settings_custom_input = initial_val.to_string();
            self.input_cursor = self.settings_custom_input.chars().count();
            self.input_selection = None;
            self.cursor_blink = true;
        }
        cx.notify();
    }

    pub fn close_settings_dropdown(&mut self, cx: &mut Context<Self>) {
        self.settings_dropdown_open = None;
        self.settings_custom_input.clear();
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_settings_custom_input(&mut self, val: String, cx: &mut Context<Self>) {
        self.settings_custom_input = val;
        cx.notify();
    }

    pub fn apply_settings_custom_input(&mut self, row_id: &str, cx: &mut Context<Self>) {
        let trimmed = self.settings_custom_input.trim();
        let field_is_int = self
            .config
            .get_field(row_id)
            .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Other(cow) if cow == "integer"))
            .unwrap_or(false);

        if field_is_int {
            if let Ok(n) = trimmed.parse::<i64>() {
                self.update_config_field(row_id, serde_json::Value::Number(serde_json::Number::from(n)), cx);
                self.settings_dropdown_open = None;
                self.settings_custom_input.clear();
            }
        } else {
            self.update_config_field(row_id, serde_json::Value::String(trimmed.to_string()), cx);
            self.settings_dropdown_open = None;
            self.settings_custom_input.clear();
        }
        cx.notify();
    }

    pub fn toggle_palette(&mut self, cx: &mut Context<Self>) {
        self.palette_open = !self.palette_open;
        if self.palette_open {
            self.menu_open = false;
        }
        cx.notify();
    }

    pub fn close_palette(&mut self, cx: &mut Context<Self>) {
        self.palette_open = false;
        cx.notify();
    }

    pub fn set_view(&mut self, view: &str, cx: &mut Context<Self>) {
        if view == "services" {
            self.active_view = "overview".to_string();
            self.active_services_tab = "services".to_string();
        } else if view == "processes" {
            self.active_view = "overview".to_string();
            self.active_services_tab = "processes".to_string();
        } else {
            self.active_view = view.to_string();
        }
        cx.notify();
    }

    pub fn set_services_tab(&mut self, tab: &str, cx: &mut Context<Self>) {
        self.active_services_tab = tab.to_string();
        cx.notify();
    }

    pub fn switch_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        self.active_tab_id = tab_id.to_string();
        self.screen = Screen::Server;
        cx.notify();
    }

    pub fn focus_service(&mut self, name: &str, cx: &mut Context<Self>) {
        for svc in &mut self.services {
            svc.is_focused = svc.name == name;
        }
        cx.notify();
    }

    pub fn toggle_service_confirm(&mut self, name: &str, cx: &mut Context<Self>) {
        for svc in &mut self.services {
            if svc.name == name {
                svc.show_confirm = !svc.show_confirm;
            } else {
                svc.show_confirm = false;
            }
        }
        cx.notify();
    }

    pub fn toggle_rule_expand(&mut self, num: &str, cx: &mut Context<Self>) {
        for r in &mut self.hba_rules {
            if r.num == num {
                r.is_expanded = !r.is_expanded;
            }
        }
        cx.notify();
    }

    pub fn set_rule_method(&mut self, rule_num: &str, method: &'static str, cx: &mut Context<Self>) {
        for r in &mut self.hba_rules {
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
        cx.notify();
    }

    // --- SSH Key Management Subsystem ---

    pub fn refresh_keys(&mut self, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            self.scan_paths = db_guard.list_scan_paths().unwrap_or_default();
            self.key_groups = db_guard.list_key_groups().unwrap_or_default();
            self.enrolled_keys = db_guard.list_ssh_keys().unwrap_or_default();
            let mut discovered = Vec::new();
            for p in &self.scan_paths {
                let expanded = expand_tilde(&p.path);
                let found = scan_directory(&expanded, &self.enrolled_keys);
                for k in found {
                    if !discovered.iter().any(|d: &DiscoveredKey| d.fingerprint == k.fingerprint) {
                        discovered.push(k);
                    }
                }
            }
            let new_count = discovered.iter().filter(|d| !d.is_enrolled).count();
            self.scan_status_message = Some(format!(
                "Scanned {} path{} · {} key{} found ({} new)",
                self.scan_paths.len(),
                if self.scan_paths.len() == 1 { "" } else { "s" },
                discovered.len(),
                if discovered.len() == 1 { "" } else { "s" },
                new_count
            ));
            self.discovered_keys = discovered;
        }
        cx.notify();
    }

    pub fn set_key_group_filter(&mut self, group_id: Option<String>, cx: &mut Context<Self>) {
        self.selected_key_group_filter = group_id;
        cx.notify();
    }

    pub fn import_discovered_key(&mut self, fingerprint: &str, group_id: Option<&str>, cx: &mut Context<Self>) {
        let key_opt = self.discovered_keys.iter().find(|k| k.fingerprint == fingerprint).cloned();
        if let Some(disc) = key_opt {
            let target_group = group_id.unwrap_or("fleet");
            let id = format!("key-{}", &fingerprint.replace("SHA256:", "").chars().take(12).collect::<String>());
            let record = SshKeyRecord {
                id,
                name: disc.suggested_name,
                group_id: target_group.to_string(),
                public_key: disc.public_key,
                fingerprint: disc.fingerprint,
                algorithm: disc.algorithm,
                comment: disc.comment,
                private_key_path: if disc.has_private_key { Some(disc.file_path) } else { None },
                attached_servers: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.upsert_ssh_key(&record);
            }
            self.key_toast = Some(format!("Enrolled '{}' into group '{}'", record.name, target_group));
            self.refresh_keys(cx);
        }
    }

    pub fn delete_enrolled_key(&mut self, key_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.delete_ssh_key(key_id);
        }
        self.key_toast = Some("Removed key from memory (file preserved)".to_string());
        self.refresh_keys(cx);
    }

    pub fn open_key_gen_modal(&mut self, cx: &mut Context<Self>) {
        self.key_gen_modal = Some(KeyGenModalState::default());
        self.input_cursor = 0;
        self.input_selection = None;
        self.cursor_blink = true;
        cx.notify();
    }

    pub fn close_key_gen_modal(&mut self, cx: &mut Context<Self>) {
        self.key_gen_modal = None;
        self.refresh_keys(cx);
    }

    pub fn submit_key_generation(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.key_gen_modal {
            let name = state.name_input.trim();
            if name.is_empty() {
                state.error_message = Some("Key name cannot be empty".to_string());
                cx.notify();
                return;
            }

            let target_dir = expand_tilde(state.custom_dir_input.trim());
            let comment = if state.comment_input.trim().is_empty() {
                None
            } else {
                Some(state.comment_input.trim())
            };

            match crate::keys::generate_keypair(
                name,
                state.algo,
                comment,
                &state.group_id,
                &target_dir,
                None,
            ) {
                Ok((record, pub_key_openssh, priv_path, _pub_path)) => {
                    let db = self.vault.db();
                    if let Ok(db_guard) = db.lock() {
                        let _ = db_guard.upsert_ssh_key(&record);
                    }
                    state.error_message = None;
                    state.generated_public_key = Some(pub_key_openssh);
                    state.generated_priv_path = Some(priv_path.display().to_string());
                    state.generated_fingerprint = Some(record.fingerprint);
                    cx.notify();
                }
                Err(err) => {
                    state.error_message = Some(err);
                    cx.notify();
                }
            }
        }
    }

    pub fn open_new_group_modal(&mut self, cx: &mut Context<Self>) {
        self.new_group_modal = Some(NewGroupModalState {
            name_input: String::new(),
            color_input: "#4ade80".to_string(),
            error_message: None,
        });
        self.input_cursor = 0;
        self.input_selection = None;
        self.cursor_blink = true;
        cx.notify();
    }

    pub fn close_new_group_modal(&mut self, cx: &mut Context<Self>) {
        self.new_group_modal = None;
        cx.notify();
    }

    pub fn submit_new_group(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.new_group_modal {
            let name = state.name_input.trim();
            if name.is_empty() {
                state.error_message = Some("Group name cannot be empty".to_string());
                cx.notify();
                return;
            }
            let slug = name.to_lowercase().replace(' ', "-").replace('_', "-");
            let color = if state.color_input.is_empty() { "#60a5fa" } else { &state.color_input };

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.add_key_group(&slug, name, color);
            }
            self.new_group_modal = None;
            self.refresh_keys(cx);
        }
    }

    pub fn delete_key_group(&mut self, group_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.delete_key_group(group_id);
        }
        if self.selected_key_group_filter.as_deref() == Some(group_id) {
            self.selected_key_group_filter = None;
        }
        self.refresh_keys(cx);
    }

    pub fn open_add_scan_path_modal(&mut self, cx: &mut Context<Self>) {
        self.add_scan_path_modal = Some(AddScanPathModalState::default());
        self.input_cursor = 0;
        self.input_selection = None;
        self.cursor_blink = true;
        cx.notify();
    }

    pub fn close_add_scan_path_modal(&mut self, cx: &mut Context<Self>) {
        self.add_scan_path_modal = None;
        cx.notify();
    }

    pub fn submit_add_scan_path(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.add_scan_path_modal {
            let path = state.path_input.trim();
            if path.is_empty() {
                state.error_message = Some("Path cannot be empty".to_string());
                cx.notify();
                return;
            }
            let expanded = expand_tilde(path);
            if !expanded.exists() || !expanded.is_dir() {
                state.error_message = Some(format!("Directory does not exist: {}", expanded.display()));
                cx.notify();
                return;
            }

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.add_scan_path(path);
            }
            self.add_scan_path_modal = None;
            self.refresh_keys(cx);
        }
    }

    pub fn remove_scan_path(&mut self, id: i64, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.remove_scan_path(id);
        }
        self.refresh_keys(cx);
    }

    pub fn open_edit_key_modal(&mut self, key_id: &str, cx: &mut Context<Self>) {
        if let Some(key) = self.enrolled_keys.iter().find(|k| k.id == key_id) {
            let name_len = key.name.chars().count();
            self.edit_key_modal = Some(EditKeyModalState {
                key_id: key.id.clone(),
                name_input: key.name.clone(),
                group_id: key.group_id.clone(),
                attached_servers: key.attached_servers.clone(),
                error_message: None,
            });
            self.input_cursor = name_len;
            self.input_selection = None;
            self.cursor_blink = true;
            cx.notify();
        }
    }

    pub fn close_edit_key_modal(&mut self, cx: &mut Context<Self>) {
        self.edit_key_modal = None;
        cx.notify();
    }

    pub fn submit_edit_key(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.edit_key_modal {
            let name = state.name_input.trim();
            if name.is_empty() {
                state.error_message = Some("Key name cannot be empty".to_string());
                cx.notify();
                return;
            }

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.update_ssh_key_name_and_group(&state.key_id, name, &state.group_id);
                let _ = db_guard.update_ssh_key_attached_servers(&state.key_id, &state.attached_servers);
            }
            self.edit_key_modal = None;
            self.refresh_keys(cx);
        }
    }

    pub fn toggle_edit_key_server(&mut self, server_id: &str, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.edit_key_modal {
            if let Some(idx) = state.attached_servers.iter().position(|s| s == server_id) {
                state.attached_servers.remove(idx);
            } else {
                state.attached_servers.push(server_id.to_string());
            }
            cx.notify();
        }
    }

    pub fn copy_text_with_toast(&mut self, text: &str, toast: &str, cx: &mut Context<Self>) {
        copy_to_clipboard_system(text);
        self.key_toast = Some(toast.to_string());
        cx.notify();
    }

    pub fn clear_key_toast(&mut self, cx: &mut Context<Self>) {
        self.key_toast = None;
        cx.notify();
    }

    // --- Server Enrollment Subsystem ---

    pub fn start_onboarding(&mut self, cx: &mut Context<Self>) {
        self.onboard_state = OnboardState::new(&self.enrolled_keys);
        self.screen = Screen::Onboard;
        self.menu_open = false;
        self.palette_open = false;
        self.onboard_set_focus_select(OnboardFieldFocus::Host, false, cx);
    }

    pub fn reload_servers(&mut self) {
        if let Ok(db_guard) = self.vault.db().lock() {
            self.servers = db_guard.list_servers().unwrap_or_default();
        }
    }

    pub fn onboard_set_focus_select(&mut self, focus: OnboardFieldFocus, select_all: bool, cx: &mut Context<Self>) {
        self.onboard_state.focus = focus;
        let text_len = match focus {
            OnboardFieldFocus::Host => self.onboard_state.host.chars().count(),
            OnboardFieldFocus::Port => self.onboard_state.port.chars().count(),
            OnboardFieldFocus::User => self.onboard_state.user.chars().count(),
            OnboardFieldFocus::Password => self.onboard_state.password.chars().count(),
            OnboardFieldFocus::Label => self.onboard_state.label.chars().count(),
            OnboardFieldFocus::Tags => self.onboard_state.tags.chars().count(),
            OnboardFieldFocus::None => 0,
        };
        self.cursor_blink = true;
        self.input_drag_anchor = None;
        if select_all && text_len > 0 {
            self.input_selection = Some((0, text_len));
            self.input_cursor = text_len;
        } else {
            self.input_selection = None;
            self.input_cursor = text_len;
        }
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn onboard_set_focus(&mut self, focus: OnboardFieldFocus, cx: &mut Context<Self>) {
        self.onboard_set_focus_select(focus, false, cx);
    }

    pub fn onboard_set_step(&mut self, step: OnboardStep, cx: &mut Context<Self>) {
        self.onboard_state.step = step;
        if step.num() > self.onboard_state.max_reached_step.num() {
            self.onboard_state.max_reached_step = step;
        }
        let focus = match step {
            OnboardStep::Address => OnboardFieldFocus::Host,
            OnboardStep::Credentials => OnboardFieldFocus::User,
            OnboardStep::VerifyHost => {
                if self.onboard_state.probe_result.is_none() {
                    self.onboard_run_probe(cx);
                    return;
                }
                OnboardFieldFocus::None
            }
            OnboardStep::Classify => OnboardFieldFocus::Label,
            OnboardStep::Finish => OnboardFieldFocus::None,
        };
        self.onboard_set_focus_select(focus, false, cx);
    }

    pub fn onboard_next_step(&mut self, cx: &mut Context<Self>) {
        match self.onboard_state.step {
            OnboardStep::Address => {
                if self.onboard_state.host.trim().is_empty() {
                    self.onboard_state.error_message = Some("Host address is required".into());
                    cx.notify();
                    return;
                }
                if self.onboard_state.port.trim().is_empty() {
                    self.onboard_state.port = "22".into();
                }
                self.onboard_state.error_message = None;
                self.onboard_state.step = OnboardStep::Credentials;
                if self.onboard_state.step.num() > self.onboard_state.max_reached_step.num() {
                    self.onboard_state.max_reached_step = self.onboard_state.step;
                }
                self.onboard_set_focus_select(OnboardFieldFocus::User, false, cx);
            }
            OnboardStep::Credentials => {
                if self.onboard_state.user.trim().is_empty() {
                    self.onboard_state.error_message = Some("SSH user username is required".into());
                    cx.notify();
                    return;
                }
                self.onboard_state.error_message = None;
                self.onboard_state.step = OnboardStep::VerifyHost;
                if self.onboard_state.step.num() > self.onboard_state.max_reached_step.num() {
                    self.onboard_state.max_reached_step = self.onboard_state.step;
                }
                self.onboard_set_focus_select(OnboardFieldFocus::None, false, cx);
                self.onboard_run_probe(cx);
            }
            OnboardStep::VerifyHost => {
                self.onboard_state.error_message = None;
                self.onboard_state.step = OnboardStep::Classify;
                if self.onboard_state.step.num() > self.onboard_state.max_reached_step.num() {
                    self.onboard_state.max_reached_step = self.onboard_state.step;
                }
                self.onboard_set_focus_select(OnboardFieldFocus::Label, false, cx);
            }
            OnboardStep::Classify => {
                if self.onboard_state.label.trim().is_empty() {
                    self.onboard_state.error_message = Some("Server name / label is required".into());
                    cx.notify();
                    return;
                }
                self.onboard_state.error_message = None;
                self.onboard_state.step = OnboardStep::Finish;
                if self.onboard_state.step.num() > self.onboard_state.max_reached_step.num() {
                    self.onboard_state.max_reached_step = self.onboard_state.step;
                }
                self.onboard_set_focus_select(OnboardFieldFocus::None, false, cx);
            }
            OnboardStep::Finish => {
                self.submit_server_enrollment(cx);
            }
        }
    }

    pub fn onboard_prev_step(&mut self, cx: &mut Context<Self>) {
        self.onboard_state.error_message = None;
        match self.onboard_state.step {
            OnboardStep::Address => {}
            OnboardStep::Credentials => {
                self.onboard_state.step = OnboardStep::Address;
                self.onboard_set_focus_select(OnboardFieldFocus::Host, false, cx);
            }
            OnboardStep::VerifyHost => {
                self.onboard_state.step = OnboardStep::Credentials;
                self.onboard_set_focus_select(OnboardFieldFocus::User, false, cx);
            }
            OnboardStep::Classify => {
                self.onboard_state.step = OnboardStep::VerifyHost;
                self.onboard_set_focus_select(OnboardFieldFocus::None, false, cx);
            }
            OnboardStep::Finish => {
                self.onboard_state.step = OnboardStep::Classify;
                self.onboard_set_focus_select(OnboardFieldFocus::Label, false, cx);
            }
        }
    }

    pub fn onboard_run_probe(&mut self, cx: &mut Context<Self>) {
        let host = self.onboard_state.host.trim().to_string();
        let port = self.onboard_state.port.trim().parse::<u16>().unwrap_or(22);
        let key_name = if let Some(ref kid) = self.onboard_state.selected_key_id {
            self.enrolled_keys.iter().find(|k| &k.id == kid).map(|k| k.name.as_str()).unwrap_or("ssh-key")
        } else {
            "ssh-agent"
        };
        let role = self.onboard_state.role.clone();

        let (result, logs, facts) = probe_host(&host, port, key_name, &role);

        self.onboard_state.host_key_accepted = result.is_known_host;
        self.onboard_state.probe_result = Some(result);
        self.onboard_state.probe_logs = logs;
        self.onboard_state.facts = facts;
        self.onboard_state.is_probing = false;
        cx.notify();
    }

    pub fn onboard_accept_host_key(&mut self, cx: &mut Context<Self>) {
        let host = self.onboard_state.host.trim();
        let port = self.onboard_state.port.trim().parse::<u16>().unwrap_or(22);
        let _ = append_to_known_hosts(
            host,
            port,
            "ssh-ed25519",
            "AAAAC3NzaC1lZDI1NTE5AAAAIC0pReYk4+8qV2wz7nN8d89gC19P2Q3L5v9a7BcD1E8F",
        );
        self.onboard_state.host_key_accepted = true;
        if let Some(ref mut res) = self.onboard_state.probe_result {
            res.is_known_host = true;
        }
        cx.notify();
    }

    pub fn submit_server_enrollment(&mut self, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().to_rfc3339();
        let label = self.onboard_state.label.trim().to_string();
        let name = if label.is_empty() {
            self.onboard_state.host.clone()
        } else {
            label
        };
        let id = name.to_lowercase().replace(' ', "-").replace('.', "-");
        let port = self.onboard_state.port.trim().parse::<u16>().unwrap_or(22);
        let tags: Vec<String> = self.onboard_state.tags
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        let status = if let Some(ref p) = self.onboard_state.probe_result {
            if p.is_reachable { "online".to_string() } else { "offline".to_string() }
        } else {
            "online".to_string()
        };
        let host_key_fingerprint = self.onboard_state.probe_result.as_ref().map(|p| p.host_key_fingerprint.clone());
        let record = ServerRecord {
            id: id.clone(),
            name: name.clone(),
            host: self.onboard_state.host.trim().to_string(),
            port,
            login_user: self.onboard_state.user.trim().to_string(),
            auth_method: self.onboard_state.auth_method.clone(),
            key_id: self.onboard_state.selected_key_id.clone(),
            jump_host_id: self.onboard_state.jump_host_id.clone(),
            env: self.onboard_state.env.clone(),
            role: self.onboard_state.role.clone(),
            group_name: self.onboard_state.group.clone(),
            tags,
            host_key_fingerprint,
            os_distro: self.onboard_state.facts.distro.clone(),
            os_kernel: self.onboard_state.facts.kernel.clone(),
            arch: self.onboard_state.facts.arch.clone(),
            memory_total: self.onboard_state.facts.memory.clone(),
            disk_total: self.onboard_state.facts.disk.clone(),
            agent_installed: self.onboard_state.install_agent,
            agent_version: if self.onboard_state.install_agent { Some("0.9.4".into()) } else { None },
            status: status.clone(),
            created_at: now.clone(),
            last_seen_at: Some(now),
        };

        // Persist to SQLite
        {
            if let Ok(db_guard) = self.vault.db().lock() {
                let _ = db_guard.upsert_server(&record);
                if let Some(ref kid) = record.key_id {
                    let _ = db_guard.attach_server_to_key(kid, &record.name);
                }
            }
        }

        // Reload servers & keys from DB
        self.reload_servers();
        self.refresh_keys(cx);

        // Add to tabs if not already present, switch active tab to it
        let status_color = match status.as_str() {
            "online" => OK,
            "warn" => WARN,
            "crit" => CRIT,
            _ => TEXT_FAINTER,
        };
        if !self.tabs.iter().any(|t| t.id == id) {
            self.tabs.push(ServerTab {
                id: id.clone(),
                name: name.clone(),
                status_color,
                is_active: true,
            });
        }
        self.active_tab_id = id.clone();
        self.screen = Screen::Server;
        self.active_view = "overview".to_string();
        self.key_toast = Some(format!("Server '{}' enrolled into fleet", name));

        cx.notify();
    }



    pub fn onboard_cycle_focus(&mut self, _reverse: bool, cx: &mut Context<Self>) {
        let next_focus = match self.onboard_state.step {
            OnboardStep::Address => match self.onboard_state.focus {
                OnboardFieldFocus::Host => OnboardFieldFocus::Port,
                _ => OnboardFieldFocus::Host,
            },
            OnboardStep::Credentials => {
                if self.onboard_state.auth_method == "password" {
                    match self.onboard_state.focus {
                        OnboardFieldFocus::User => OnboardFieldFocus::Password,
                        _ => OnboardFieldFocus::User,
                    }
                } else {
                    OnboardFieldFocus::User
                }
            }
            OnboardStep::VerifyHost => OnboardFieldFocus::None,
            OnboardStep::Classify => match self.onboard_state.focus {
                OnboardFieldFocus::Label => OnboardFieldFocus::Tags,
                _ => OnboardFieldFocus::Label,
            },
            OnboardStep::Finish => OnboardFieldFocus::None,
        };
        self.onboard_set_focus_select(next_focus, false, cx);
    }
}

impl Render for CrowApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_overview = self.active_view == "overview";
        let is_config = self.active_view == "config";
        let palette_open = self.palette_open;
        let menu_open = self.menu_open;
        let screen = self.screen;
        let app_view = cx.entity();
        let vault_status = self.vault.status();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _window, cx| {
                let is_mod = ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control;
                let is_shift = ev.keystroke.modifiers.shift;
                let key = ev.keystroke.key.to_lowercase();

                // If vault is locked, keyboard events are dedicated to unlocking
                if this.vault.status() == VaultStatus::Locked {
                    this.cursor_blink = true;
                    if key == "tab" {
                        this.lock_state.active_focus = match this.lock_state.active_focus {
                            LockFieldFocus::Password => LockFieldFocus::Totp,
                            LockFieldFocus::Totp => LockFieldFocus::Password,
                        };
                        let len = match this.lock_state.active_focus {
                            LockFieldFocus::Password => this.lock_state.password_input.chars().count(),
                            LockFieldFocus::Totp => this.lock_state.totp_input.chars().count(),
                        };
                        this.input_cursor = len;
                        this.input_selection = None;
                        cx.notify();
                    } else if key == "enter" {
                        this.submit_unlock(cx);
                    } else {
                        let is_totp = this.lock_state.active_focus == LockFieldFocus::Totp;
                        let text = match this.lock_state.active_focus {
                            LockFieldFocus::Password => &mut this.lock_state.password_input,
                            LockFieldFocus::Totp => &mut this.lock_state.totp_input,
                        };
                        let changed = crate::components::handle_text_key_event(text, &mut this.input_cursor, &mut this.input_selection, ev);
                        if is_totp {
                            this.lock_state.totp_input.retain(|c| c.is_ascii_digit());
                            if this.lock_state.totp_input.chars().count() > 6 {
                                let s: String = this.lock_state.totp_input.chars().take(6).collect();
                                this.lock_state.totp_input = s;
                                this.input_cursor = this.input_cursor.min(6);
                            }
                        }
                        if changed {
                            this.lock_state.error_message = None;
                            cx.notify();
                        }
                    }
                    return;
                }

                // If on Vault Setup Screen
                if this.screen == Screen::VaultSetup {
                    match this.setup_state.step {
                        SetupStep::WarningNotice => {
                            if ev.keystroke.key == "escape" {
                                this.setup_state = SetupState::default();
                                this.set_screen(Screen::Settings, cx);
                            } else if key == "enter" {
                                this.setup_state.step = SetupStep::ConfigureCredentials;
                                if this.setup_state.totp_secret.is_empty() {
                                    this.setup_state.totp_secret = crate::vault::generate_totp_secret();
                                }
                                this.input_cursor = this.setup_state.password_input.chars().count();
                                this.input_selection = None;
                                this.cursor_blink = true;
                                cx.notify();
                            }
                        }
                        SetupStep::ConfigureCredentials => {
                            this.cursor_blink = true;
                            if ev.keystroke.key == "escape" {
                                this.setup_state.step = SetupStep::WarningNotice;
                                cx.notify();
                            } else if key == "tab" {
                                this.setup_state.active_focus = match this.setup_state.active_focus {
                                    SetupFieldFocus::Password => SetupFieldFocus::ConfirmPassword,
                                    SetupFieldFocus::ConfirmPassword => SetupFieldFocus::TotpConfirm,
                                    SetupFieldFocus::TotpConfirm => SetupFieldFocus::Password,
                                };
                                let len = match this.setup_state.active_focus {
                                    SetupFieldFocus::Password => this.setup_state.password_input.chars().count(),
                                    SetupFieldFocus::ConfirmPassword => this.setup_state.confirm_input.chars().count(),
                                    SetupFieldFocus::TotpConfirm => this.setup_state.totp_confirm_input.chars().count(),
                                };
                                this.input_cursor = len;
                                this.input_selection = None;
                                cx.notify();
                            } else if key == "enter" {
                                this.submit_setup(cx);
                            } else {
                                let is_totp = this.setup_state.active_focus == SetupFieldFocus::TotpConfirm;
                                let text = match this.setup_state.active_focus {
                                    SetupFieldFocus::Password => &mut this.setup_state.password_input,
                                    SetupFieldFocus::ConfirmPassword => &mut this.setup_state.confirm_input,
                                    SetupFieldFocus::TotpConfirm => &mut this.setup_state.totp_confirm_input,
                                };
                                let changed = crate::components::handle_text_key_event(text, &mut this.input_cursor, &mut this.input_selection, ev);
                                if is_totp {
                                    this.setup_state.totp_confirm_input.retain(|c| c.is_ascii_digit());
                                    if this.setup_state.totp_confirm_input.chars().count() > 6 {
                                        let s: String = this.setup_state.totp_confirm_input.chars().take(6).collect();
                                        this.setup_state.totp_confirm_input = s;
                                        this.input_cursor = this.input_cursor.min(6);
                                    }
                                }
                                if changed {
                                    this.setup_state.error_message = None;
                                    cx.notify();
                                }
                            }
                        }
                    }
                    return;
                }

                // SSH Key Management Modal Keyboard Routing
                if let Some(ref mut gen) = this.key_gen_modal {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.close_key_gen_modal(cx);
                    } else if key == "enter" {
                        if gen.generated_public_key.is_some() {
                            this.close_key_gen_modal(cx);
                        } else {
                            this.submit_key_generation(cx);
                        }
                    } else if key == "tab" {
                        gen.active_focus = match gen.active_focus {
                            KeyGenFieldFocus::Name => KeyGenFieldFocus::Comment,
                            KeyGenFieldFocus::Comment => KeyGenFieldFocus::Directory,
                            KeyGenFieldFocus::Directory => KeyGenFieldFocus::Name,
                        };
                        this.input_cursor = match gen.active_focus {
                            KeyGenFieldFocus::Name => gen.name_input.chars().count(),
                            KeyGenFieldFocus::Comment => gen.comment_input.chars().count(),
                            KeyGenFieldFocus::Directory => gen.custom_dir_input.chars().count(),
                        };
                        this.input_selection = None;
                        cx.notify();
                    } else {
                        let target = match gen.active_focus {
                            KeyGenFieldFocus::Name => &mut gen.name_input,
                            KeyGenFieldFocus::Comment => &mut gen.comment_input,
                            KeyGenFieldFocus::Directory => &mut gen.custom_dir_input,
                        };
                        if crate::components::handle_text_key_event(
                            target,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ) {
                            gen.error_message = None;
                            cx.notify();
                        }
                    }
                    return;
                }

                if let Some(ref mut grp) = this.new_group_modal {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.close_new_group_modal(cx);
                    } else if key == "enter" {
                        this.submit_new_group(cx);
                    } else if crate::components::handle_text_key_event(
                        &mut grp.name_input,
                        &mut this.input_cursor,
                        &mut this.input_selection,
                        ev,
                    ) {
                        grp.error_message = None;
                        cx.notify();
                    }
                    return;
                }

                if let Some(ref mut sp) = this.add_scan_path_modal {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.close_add_scan_path_modal(cx);
                    } else if key == "enter" {
                        this.submit_add_scan_path(cx);
                    } else if crate::components::handle_text_key_event(
                        &mut sp.path_input,
                        &mut this.input_cursor,
                        &mut this.input_selection,
                        ev,
                    ) {
                        sp.error_message = None;
                        cx.notify();
                    }
                    return;
                }

                if let Some(ref mut edit) = this.edit_key_modal {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.close_edit_key_modal(cx);
                    } else if key == "enter" {
                        this.submit_edit_key(cx);
                    } else if crate::components::handle_text_key_event(
                        &mut edit.name_input,
                        &mut this.input_cursor,
                        &mut this.input_selection,
                        ev,
                    ) {
                        edit.error_message = None;
                        cx.notify();
                    }
                    return;
                }

                // If on Settings with dropdown open, handle dropdown typing / escape / enter
                if this.screen == Screen::Settings {
                    if let Some(open_row_id) = this.settings_dropdown_open.clone() {
                        this.cursor_blink = true;
                        if ev.keystroke.key == "escape" {
                            this.close_settings_dropdown(cx);
                            return;
                        } else if key == "enter" {
                            this.apply_settings_custom_input(&open_row_id, cx);
                            return;
                        } else if crate::components::handle_text_key_event(
                            &mut this.settings_custom_input,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ) {
                            let field_is_int = this
                                .config
                                .get_field(&open_row_id)
                                .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Other(cow) if cow == "integer"))
                                .unwrap_or(false);
                            if field_is_int {
                                this.settings_custom_input.retain(|c| c.is_ascii_digit());
                                this.input_cursor = this.input_cursor.min(this.settings_custom_input.chars().count());
                            }
                            cx.notify();
                            return;
                        }
                    }
                }

                // Onboard Screen Keyboard Interaction
                if this.screen == Screen::Onboard {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.set_screen(Screen::Fleet, cx);
                        return;
                    } else if key == "enter" {
                        this.onboard_next_step(cx);
                        return;
                    } else if key == "tab" {
                        let shift = is_shift;
                        this.onboard_cycle_focus(shift, cx);
                        return;
                    }

                    let handled = match this.onboard_state.focus {
                        OnboardFieldFocus::Host => crate::components::handle_text_key_event(
                            &mut this.onboard_state.host,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        OnboardFieldFocus::Port => {
                            let res = crate::components::handle_text_key_event(
                                &mut this.onboard_state.port,
                                &mut this.input_cursor,
                                &mut this.input_selection,
                                ev,
                            );
                            if res {
                                this.onboard_state.port.retain(|c| c.is_ascii_digit());
                                if this.onboard_state.port.len() > 5 {
                                    this.onboard_state.port.truncate(5);
                                    this.input_cursor = this.input_cursor.min(this.onboard_state.port.len());
                                }
                            }
                            res
                        }
                        OnboardFieldFocus::User => crate::components::handle_text_key_event(
                            &mut this.onboard_state.user,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        OnboardFieldFocus::Password => crate::components::handle_text_key_event(
                            &mut this.onboard_state.password,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        OnboardFieldFocus::Label => crate::components::handle_text_key_event(
                            &mut this.onboard_state.label,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        OnboardFieldFocus::Tags => crate::components::handle_text_key_event(
                            &mut this.onboard_state.tags,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        OnboardFieldFocus::None => false,
                    };

                    if handled {
                        this.onboard_state.error_message = None;
                        cx.notify();
                        return;
                    }
                }

                // Normal Screens shortcuts
                if ev.keystroke.key == "escape" {
                    if this.menu_open {
                        this.menu_open = false;
                        cx.notify();
                    } else if this.palette_open {
                        this.palette_open = false;
                        cx.notify();
                    } else if this.screen != Screen::Server && this.screen != Screen::Fleet {
                        this.set_screen(Screen::Fleet, cx);
                    }
                } else if key == "l" && is_mod && is_shift {
                    this.lock(cx);
                } else if key == "k" && is_mod {
                    this.toggle_palette(cx);
                } else if key == "\\" && is_mod {
                    this.sidebar_collapsed = !this.sidebar_collapsed;
                    cx.notify();
                } else if key == "1" && is_mod {
                    this.set_screen(Screen::Fleet, cx);
                } else if key == "2" && is_mod {
                    this.set_screen(Screen::Server, cx);
                    this.set_view("overview", cx);
                } else if key == "3" && is_mod {
                    this.set_screen(Screen::Server, cx);
                    this.set_view("config", cx);
                } else if key == "," && is_mod {
                    this.set_screen(Screen::Settings, cx);
                } else if key == "s" && is_mod && this.screen == Screen::Settings {
                    this.save_config(cx);
                } else if key == "/" && is_mod && this.screen == Screen::Settings {
                    this.open_config_file();
                } else if key == "n" && is_mod {
                    this.start_onboarding(cx);
                } else if key == "f" && is_mod && is_shift {
                    this.set_screen(Screen::FleetSetup, cx);
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _ev: &MouseUpEvent, _window, cx| {
                if this.input_drag_anchor.is_some() {
                    this.input_drag_anchor = None;
                    cx.notify();
                }
            }))
            .size_full()
            .bg(BG_WINDOW)
            .text_color(TEXT_PRIMARY)
            .font_family(FONT_MONO)
            .text_size(px(12.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .border_1()
            .border_color(BORDER_DEFAULT)
            .relative()
            // If locked, show full lock screen
            .children(if vault_status == VaultStatus::Locked {
                Some(
                    div().size_full().child(vault_lock_view(app_view.clone(), self))
                )
            } else {
                None
            })
            // If not locked (Disabled or Unlocked), show normal app
            .children(if vault_status != VaultStatus::Locked {
                Some(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        // 1. Frameless Titlebar
                        .child(titlebar(
                            &self.tabs,
                            &self.active_tab_id,
                            self.screen,
                            self.menu_open,
                            app_view.clone(),
                        ))
                        // 2. Main Screen Area
                        .child(
                            div()
                                .flex_1()
                                .min_h(px(0.0))
                                .flex()
                                .flex_col()
                                .w_full()
                                .children(match screen {
                                    Screen::Server => Some(
                                        div()
                                            .size_full()
                                            .flex()
                                            .flex_col()
                                            // Server Identity Bar
                                            .child(identity_bar(app_view.clone()))
                                            // Server Stat Strip
                                            .child(stat_strip())
                                            // Main Server Body: Sidebar + Content
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_h(px(0.0))
                                                    .flex()
                                                    .w_full()
                                                    // Sidebar
                                                    .child(sidebar(&self.active_view, self.sidebar_collapsed, app_view.clone()))
                                                    // Content Area (Overview or Config or other server view)
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .min_w(px(0.0))
                                                            .h_full()
                                                            .flex()
                                                            .children(if is_overview {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(services_table(&self.services, &self.active_services_tab, app_view.clone()))
                                                                        .child(log_tail())
                                                                )
                                                            } else if is_config {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(managed_files_rail())
                                                                        .child(rules_editor(&self.hba_rules, app_view.clone()))
                                                                        .child(pending_diff_rail())
                                                                )
                                                            } else if self.active_view == "logs" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(log_tail())
                                                                )
                                                            } else {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .flex_col()
                                                                        .bg(BG_APP)
                                                                        .child(
                                                                            div()
                                                                                .h(px(34.0))
                                                                                .flex_none()
                                                                                .flex()
                                                                                .items_center()
                                                                                .px(px(12.0))
                                                                                .bg(BG_PANEL)
                                                                                .border_b_1()
                                                                                .border_color(BORDER_PANEL)
                                                                                .child(
                                                                                    div()
                                                                                        .font_family(FONT_MONO)
                                                                                        .text_size(px(11.0))
                                                                                        .font_weight(FontWeight::SEMIBOLD)
                                                                                        .text_color(TEXT_PRIMARY)
                                                                                        .child(self.active_view.to_uppercase()),
                                                                                ),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .flex_1()
                                                                                .flex()
                                                                                .items_center()
                                                                                .justify_center()
                                                                                .gap(px(8.0))
                                                                                .font_family(FONT_MONO)
                                                                                .text_size(px(11.5))
                                                                                .text_color(TEXT_FAINT)
                                                                                .child(div().size(px(6.0)).rounded_full().bg(OK))
                                                                                .child(format!("{} · agent discovery stream pending", self.active_view)),
                                                                        ),
                                                                )
                                                            })
                                                    ),
                                            )
                                            // Persistent Danger Zone Strip
                                            .child(danger_zone()),
                                    ),
                                    Screen::Fleet => Some(
                                        div()
                                            .size_full()
                                            .child(fleet_overview_view(app_view.clone(), self)),
                                    ),
                                    Screen::Settings => Some(
                                        div()
                                            .size_full()
                                            .child(settings_view(
                                                app_view.clone(),
                                                self,
                                                self.settings_section,
                                            )),
                                    ),
                                    Screen::Onboard => Some(
                                        div()
                                            .size_full()
                                            .child(onboard_view(app_view.clone(), self)),
                                    ),
                                    Screen::FleetSetup => Some(
                                        div()
                                            .size_full()
                                            .child(fleet_setup_view(app_view.clone())),
                                    ),
                                    Screen::VaultSetup => Some(
                                        div()
                                            .size_full()
                                            .child(vault_setup_view(app_view.clone(), self)),
                                    ),
                                }),
                        )
                        // 3. Burger Menu Overlay
                        .children(if menu_open {
                            Some(burger_menu_overlay(app_view.clone(), self.screen))
                        } else {
                            None
                        })
                        // 4. Command Palette Overlay (⌘K)
                        .children(if palette_open {
                            Some(palette_overlay(app_view.clone()))
                        } else {
                            None
                        })
                )
            } else {
                None
            })
    }
}
