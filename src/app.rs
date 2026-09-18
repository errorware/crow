use gpui_kit::*;
use crate::theme::*;
use crate::components::danger_zone::danger_zone;
use crate::components::identity_bar::identity_bar;
use crate::components::palette::palette_overlay;
use crate::components::sidebar::sidebar;
use crate::components::stat_strip::stat_strip;
use crate::components::titlebar::{burger_menu_overlay, titlebar, ServerTab};
use crate::vault::{Vault, VaultStatus};
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::config::rules_editor::{default_hba_rules, rules_editor, HbaRuleDef};
use crate::views::fleet::{fleet_overview_view, fleet_setup_view};
use crate::views::lock::{
    vault_lock_view, vault_setup_view, LockFieldFocus, LockState, SetupFieldFocus, SetupState, SetupStep,
};
use crate::views::onboard::onboard_view;
use crate::views::overview::log_tail::log_tail;
use crate::views::overview::services_table::{default_services, services_table, ServiceUnit};
use crate::views::settings::settings_view;

use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;
use crate::config::CrowConfigManager;

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
}

impl CrowApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let vault = Vault::open_default().expect("Failed to initialize vault storage");
        let config = CrowConfigManager::load();

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
            tabs: vec![
                ServerTab { id: "edge-01", name: "edge-01", status_color: OK, is_active: true },
                ServerTab { id: "edge-02", name: "edge-02", status_color: OK, is_active: false },
                ServerTab { id: "db-primary", name: "db-primary", status_color: WARN, is_active: false },
                ServerTab { id: "worker-04", name: "worker-04", status_color: CRIT, is_active: false },
                ServerTab { id: "bastion", name: "bastion", status_color: TEXT_FAINTER, is_active: false },
            ],
            services: default_services(),
            hba_rules: default_hba_rules(),
            palette_open: false,
            sidebar_collapsed: false,
            settings_dropdown_open: None,
            settings_custom_input: String::new(),
        }
    }

    pub fn lock(&mut self, cx: &mut Context<Self>) {
        if self.vault.is_password_auth_enabled() {
            self.vault.lock();
            self.lock_state.password_input.clear();
            self.lock_state.totp_input.clear();
            self.lock_state.error_message = None;
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
                    if key == "tab" {
                        this.lock_state.active_focus = match this.lock_state.active_focus {
                            LockFieldFocus::Password => LockFieldFocus::Totp,
                            LockFieldFocus::Totp => LockFieldFocus::Password,
                        };
                        cx.notify();
                    } else if key == "enter" {
                        this.submit_unlock(cx);
                    } else if key == "backspace" {
                        match this.lock_state.active_focus {
                            LockFieldFocus::Password => { this.lock_state.password_input.pop(); }
                            LockFieldFocus::Totp => { this.lock_state.totp_input.pop(); }
                        }
                        this.lock_state.error_message = None;
                        cx.notify();
                    } else if !is_mod {
                        let char_to_insert = ev.keystroke.key_char.as_deref().or(if ev.keystroke.key.chars().count() == 1 {
                            Some(ev.keystroke.key.as_str())
                        } else {
                            None
                        });
                        if let Some(c) = char_to_insert {
                            match this.lock_state.active_focus {
                                LockFieldFocus::Password => { this.lock_state.password_input.push_str(c); }
                                LockFieldFocus::Totp => {
                                    if this.lock_state.totp_input.len() < 6 && c.chars().all(|d| d.is_ascii_digit()) {
                                        this.lock_state.totp_input.push_str(c);
                                    }
                                }
                            }
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
                                cx.notify();
                            }
                        }
                        SetupStep::ConfigureCredentials => {
                            if ev.keystroke.key == "escape" {
                                this.setup_state.step = SetupStep::WarningNotice;
                                cx.notify();
                            } else if key == "tab" {
                                this.setup_state.active_focus = match this.setup_state.active_focus {
                                    SetupFieldFocus::Password => SetupFieldFocus::ConfirmPassword,
                                    SetupFieldFocus::ConfirmPassword => SetupFieldFocus::TotpConfirm,
                                    SetupFieldFocus::TotpConfirm => SetupFieldFocus::Password,
                                };
                                cx.notify();
                            } else if key == "enter" {
                                this.submit_setup(cx);
                            } else if key == "backspace" {
                                match this.setup_state.active_focus {
                                    SetupFieldFocus::Password => { this.setup_state.password_input.pop(); }
                                    SetupFieldFocus::ConfirmPassword => { this.setup_state.confirm_input.pop(); }
                                    SetupFieldFocus::TotpConfirm => { this.setup_state.totp_confirm_input.pop(); }
                                }
                                this.setup_state.error_message = None;
                                cx.notify();
                            } else if !is_mod {
                                let char_to_insert = ev.keystroke.key_char.as_deref().or(if ev.keystroke.key.chars().count() == 1 {
                                    Some(ev.keystroke.key.as_str())
                                } else {
                                    None
                                });
                                if let Some(c) = char_to_insert {
                                    match this.setup_state.active_focus {
                                        SetupFieldFocus::Password => { this.setup_state.password_input.push_str(c); }
                                        SetupFieldFocus::ConfirmPassword => { this.setup_state.confirm_input.push_str(c); }
                                        SetupFieldFocus::TotpConfirm => {
                                            if this.setup_state.totp_confirm_input.len() < 6 && c.chars().all(|d| d.is_ascii_digit()) {
                                                this.setup_state.totp_confirm_input.push_str(c);
                                            }
                                        }
                                    }
                                    this.setup_state.error_message = None;
                                    cx.notify();
                                }
                            }
                        }
                    }
                    return;
                }

                // If on Settings with dropdown open, handle dropdown typing / escape / enter
                if this.screen == Screen::Settings {
                    if let Some(open_row_id) = this.settings_dropdown_open.clone() {
                        if ev.keystroke.key == "escape" {
                            this.close_settings_dropdown(cx);
                            return;
                        } else if key == "enter" {
                            this.apply_settings_custom_input(&open_row_id, cx);
                            return;
                        } else if key == "backspace" {
                            this.settings_custom_input.pop();
                            cx.notify();
                            return;
                        } else if !is_mod {
                            let char_to_insert = ev.keystroke.key_char.as_deref().or(if ev.keystroke.key.chars().count() == 1 {
                                Some(ev.keystroke.key.as_str())
                            } else {
                                None
                            });
                            if let Some(c) = char_to_insert {
                                let field_is_int = this
                                    .config
                                    .get_field(&open_row_id)
                                    .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Other(cow) if cow == "integer"))
                                    .unwrap_or(false);
                                if !field_is_int || c.chars().all(|d| d.is_ascii_digit()) {
                                    this.settings_custom_input.push_str(c);
                                    cx.notify();
                                }
                                return;
                            }
                        }
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
                    this.set_screen(Screen::Onboard, cx);
                } else if key == "f" && is_mod && is_shift {
                    this.set_screen(Screen::FleetSetup, cx);
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
                    div().size_full().child(vault_lock_view(app_view.clone(), &self.lock_state))
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
                                            .child(fleet_overview_view(app_view.clone())),
                                    ),
                                    Screen::Settings => Some(
                                        div()
                                            .size_full()
                                            .child(settings_view(
                                                app_view.clone(),
                                                &self.config,
                                                self.settings_section,
                                                self.vault.is_password_auth_enabled(),
                                                self.settings_dropdown_open.as_deref(),
                                                &self.settings_custom_input,
                                            )),
                                    ),
                                    Screen::Onboard => Some(
                                        div()
                                            .size_full()
                                            .child(onboard_view(app_view.clone())),
                                    ),
                                    Screen::FleetSetup => Some(
                                        div()
                                            .size_full()
                                            .child(fleet_setup_view(app_view.clone())),
                                    ),
                                    Screen::VaultSetup => Some(
                                        div()
                                            .size_full()
                                            .child(vault_setup_view(app_view.clone(), &self.setup_state)),
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
