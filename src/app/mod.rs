use gpui_kit::*;
use crate::components::danger_zone_state::DangerZoneState;
use crate::components::text_caret::TextCaret;
use crate::vault::{Vault, VaultStatus};
use crate::views::config::state::ConfigsState;
use crate::views::files::FilesState;
use crate::views::users::UsersState;
use crate::views::firewall::{
    default_active_ufw_state, detect_firewall_status,
    FirewallState,
};
use crate::host::LocalHost;
use crate::config::CrowConfigManager;
use crate::views::lock::{
    LockState, SetupState,
};
use crate::views::onboard::{OnboardFieldFocus, OnboardState};
use crate::journal::{
    JournalQuery,
    reader::read_journal_for_server,
    retention::{read_retention, read_retention_for_server},
};
use crate::lab::{detect_local_engines, scan_local_test_nodes};
use crate::views::logs::JournalState;
use crate::views::overview::{
    collector::{
        collect_processes_for_server, collect_services_for_server, collect_sockets_for_server,
    },
    OverviewState,
};

mod danger;
mod files;
mod firewall;
mod render;
mod keyboard;
mod poll;
mod tabs;
mod settings;
mod session;
mod onboard;
pub mod overview;
mod keys;
pub mod configs;
mod journal;
mod clankers;
mod lab;

use crate::views::fleet::lab_state::LocalLabState;
use crate::views::fleet::FleetState;
use crate::views::settings::clankers_state::ClankersState;
use crate::views::settings::keys_state::KeysState;
use crate::views::settings::state::SettingsState;
use crate::views::settings::lab::LabState;
use configs::{load_configs, log_config_core_self_check};
use poll::seed_metrics;
use tabs::initial_tabs;

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
    Components,
    Clankers,
}

impl SettingsSection {
    pub fn id_prefix(&self) -> &'static str {
        match self {
            SettingsSection::General => "general",
            SettingsSection::Connection => "connection",
            SettingsSection::Keys => "keys",
            SettingsSection::Security => "security",
            SettingsSection::Components => "components",
            SettingsSection::Clankers => "clankers",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClankerModalFocus {
    ApiKey,
    Model,
    BaseUrl,
}

#[derive(Clone, Debug)]
pub struct ClankerEditModalState {
    pub provider_id: String,
    pub display_name: String,
    pub api_key_input: String,
    pub model_input: String,
    pub base_url_input: String,
    pub focus: ClankerModalFocus,
    pub error_message: Option<String>,
}



/// The server pages that each show one overview table.
pub fn is_table_page(view: &str) -> bool {
    matches!(view, "services" | "processes" | "sockets")
}

pub struct CrowApp {
    focus_handle: FocusHandle,

    // App shell
    pub vault: Vault,
    pub config: CrowConfigManager,
    pub screen: Screen,
    /// Sub-view of the Server screen ("overview", "logs", "config", ...).
    pub active_view: String,
    pub menu_open: bool,
    pub palette_open: bool,
    pub sidebar_collapsed: bool,
    pub show_about_modal: bool,
    pub about_copied_toast: bool,
    /// Caret shared by the hand-rolled text inputs (one is focused at a time).
    pub caret: TextCaret,

    // Shared server context
    pub fleet: FleetState,

    // Feature state, one per screen/surface
    pub lock_state: LockState,
    pub setup_state: SetupState,
    pub onboard_state: OnboardState,
    pub overview: OverviewState,
    pub journal: JournalState,
    pub configs: ConfigsState,
    /// Live editor for the selected plain-text config (see ConfigTextEditor).
    pub config_text_editor: Option<configs::ConfigTextEditor>,
    /// Inline field edit in the structured config editor.
    pub structured_field_edit: Option<configs::StructuredFieldEdit>,
    /// A never-on-prod change awaiting typed confirmation.
    pub risk_confirm: Option<configs::RiskConfirm>,
    pub files: FilesState,
    pub danger: DangerZoneState,
    pub users: UsersState,
    pub firewall: FirewallState,
    pub keys: KeysState,
    pub local_lab: LocalLabState,
    pub clankers: ClankersState,
    pub settings: SettingsState,
    /// UI components sandbox (Settings → Components).
    pub lab_state: LabState,

    pub _cursor_blink_task: Task<()>,
    pub _metrics_poll_task: Task<()>,
}

impl CrowApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let vault = Vault::open_default().expect("Failed to initialize vault storage");
        let config = CrowConfigManager::load();

        // Vault records: servers, SSH key hub (with an initial disk scan), AI providers
        let (servers, keys, clanker_providers) = match vault.db().lock() {
            Ok(db) => (
                db.list_servers().unwrap_or_default(),
                KeysState::load(&db),
                db.list_clanker_providers().unwrap_or_default(),
            ),
            Err(_) => (Vec::new(), KeysState::new(Vec::new(), Vec::new(), Vec::new(), Vec::new(), None), Vec::new()),
        };

        crate::host::update_directory(&servers, &keys.enrolled);
        log_config_core_self_check();

        let tabs = initial_tabs(&servers);

        let onboard_state = OnboardState::new(&keys.enrolled);

        let (metrics_store, mut buffered_stores) = seed_metrics(&servers);

        let initial_journal = if let Some(first_srv) = servers.first() {
            read_journal_for_server(first_srv, &JournalQuery::default())
        } else {
            Vec::new()
        };

        let (journal_retention, journal_telemetry) = if let Some(first_srv) = servers.first() {
            read_retention_for_server(first_srv)
        } else {
            read_retention(&LocalHost)
        };

        let (initial_services, initial_processes, initial_sockets) = if let Some(first_srv) = servers.first() {
            (
                collect_services_for_server(first_srv),
                collect_processes_for_server(first_srv),
                collect_sockets_for_server(first_srv),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        // Backfill the first server's seeded sample with its initial tables.
        if let Some(first_srv) = servers.first() {
            for key in [&first_srv.id, &first_srv.name] {
                if let Some(sample) = buffered_stores.get_mut(key).and_then(|buf| buf.samples.back_mut()) {
                    sample.services = initial_services.clone();
                    sample.processes = initial_processes.clone();
                    sample.sockets = initial_sockets.clone();
                }
            }
        }

        let lab_engines = detect_local_engines();
        let lab_nodes = scan_local_test_nodes(&servers);

        let initial_firewall_state = servers
            .first()
            .map(|s| detect_firewall_status(s))
            .unwrap_or_else(default_active_ufw_state);
        let configs = load_configs(servers.first(), &journal_retention, &initial_firewall_state);

        Self {
            focus_handle: cx.focus_handle(),
            vault,
            config,
            lock_state: LockState::default(),
            setup_state: SetupState::default(),
            screen: Screen::Fleet,
            menu_open: false,
            settings: SettingsState::default(),
            fleet: FleetState::new(servers, tabs, metrics_store, buffered_stores),
            active_view: "overview".to_string(),
            overview: OverviewState::new(initial_services, initial_processes, initial_sockets),
            configs,
            config_text_editor: None,
            structured_field_edit: None,
            risk_confirm: None,
            palette_open: false,
            sidebar_collapsed: false,
            keys,
            files: FilesState::default(),
            danger: DangerZoneState::default(),
            onboard_state,
            lab_state: LabState::new(window, cx),
            caret: TextCaret { blink: true, ..TextCaret::default() },
            _cursor_blink_task: Self::spawn_cursor_blink(cx),
            _metrics_poll_task: Self::spawn_metrics_poll(cx),
            journal: JournalState::new(initial_journal, journal_retention, journal_telemetry),
            local_lab: LocalLabState::new(lab_engines, lab_nodes),
            show_about_modal: false,
            about_copied_toast: false,
            clankers: ClankersState::new(clanker_providers),
            users: UsersState::new(),
            firewall: FirewallState::new(initial_firewall_state),
        }
    }
}

impl CrowApp {
    pub fn has_active_text_input(&self) -> bool {
        if self.palette_open {
            return true;
        }
        if self.users.show_new_user_modal
            || self.firewall.show_new_rule_modal
            || self.journal.show_retention_modal
            || self.show_about_modal
        {
            return true;
        }
        if self.keys.any_modal_open()
            || self.clankers.editing.is_some()
        {
            return true;
        }
        if self.vault.status() == VaultStatus::Locked {
            return true;
        }
        if self.screen == Screen::VaultSetup {
            return true;
        }
        if self.screen == Screen::Onboard {
            return self.onboard_state.focus != OnboardFieldFocus::None;
        }
        if self.screen == Screen::Server {
            if self.active_view == "logs" && self.journal.search_focused {
                return true;
            }
            if (self.active_view == "config" || self.active_view == "configure")
                && self.configs.search_focused
            {
                return true;
            }
        }
        if self.screen == Screen::Settings && self.settings.dropdown_open.is_some() {
            return true;
        }
        false
    }

    pub fn open_about_modal(&mut self, cx: &mut Context<Self>) {
        self.show_about_modal = true;
        self.about_copied_toast = false;
        cx.notify();
    }

    pub fn close_about_modal(&mut self, cx: &mut Context<Self>) {
        self.show_about_modal = false;
        self.about_copied_toast = false;
        cx.notify();
    }

    pub fn set_active_view(&mut self, view: &str, cx: &mut Context<Self>) {
        self.active_view = view.to_string();
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
        if is_table_page(view) {
            // Services / Processes / Sockets are pages of their own; the
            // overview state remembers which table is on screen.
            self.active_view = view.to_string();
            self.set_services_tab(view, cx);
        } else if view == "overview" {
            self.active_view = view.to_string();
            self.refresh_overview_tables(cx);
        } else if view == "cron" {
            self.active_view = "cron".to_string();
            self.configs.selected_file = "crontab".to_string();
        } else if view == "firewall" {
            self.active_view = "firewall".to_string();
            self.configs.selected_file = "user.rules".to_string();
        } else if view == "files" {
            self.active_view = "files".to_string();
            self.load_file_listing(cx);
            return;
        } else {
            self.active_view = view.to_string();
        }
        cx.notify();
    }

}
