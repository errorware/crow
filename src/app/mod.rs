use gpui_kit::*;
use crate::components::danger_zone_state::DangerZoneState;
use crate::components::text_caret::TextCaret;
use crate::vault::{Vault, VaultStatus};
use crate::views::config::state::ConfigsState;
use crate::views::files::FilesState;
use crate::views::users::UsersState;
use crate::views::firewall::{detect_firewall_status, FirewallOperationalState, FirewallState};
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
mod users;
mod host_actions;
mod render;
mod keyboard;
mod poll;
mod tabs;
mod settings;
mod session;
pub mod onboard;
pub mod overview;
mod keys;
pub mod appearance;
pub mod archive;
pub mod configs;
pub mod region;
mod journal;
mod clankers;
pub mod providers;
mod secrets;
pub mod vault_manage;
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
    Servers,
    Components,
    Clankers,
    Providers,
    Personalisation,
}

impl SettingsSection {
    pub fn id_prefix(&self) -> &'static str {
        match self {
            SettingsSection::General => "general",
            SettingsSection::Connection => "connection",
            SettingsSection::Keys => "keys",
            SettingsSection::Security => "security",
            SettingsSection::Servers => "servers",
            SettingsSection::Components => "components",
            SettingsSection::Clankers => "clankers",
            SettingsSection::Providers => "providers",
            SettingsSection::Personalisation => "appearance",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClankerEditModalState {
    pub provider_id: String,
    pub display_name: String,
    /// Whether a key is already stored (the field starts empty either way).
    pub has_key: bool,
    /// Current model and endpoint, to prefill the inputs.
    pub model: String,
    pub base_url: String,
    pub error_message: Option<String>,
}

/// The Clankers edit dialog's inputs (created on render: they need the window).
pub struct ClankerInputs {
    pub provider_id: String,
    pub key: Entity<gpui_kit::component::input::InputState>,
    pub model: Entity<gpui_kit::component::input::InputState>,
    pub base_url: Entity<gpui_kit::component::input::InputState>,
    pub _events: Vec<Subscription>,
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
    /// The Add Server wizard's text fields (created on first render: inputs
    /// need the window).
    pub onboard_inputs: Option<onboard::OnboardInputs>,
    /// Focus `onboard_state.focus`'s field on the next render.
    pub onboard_focus_pending: bool,
    /// The Services and Processes pages' search boxes (created on first render).
    pub services_search: Option<overview::TableSearch>,
    pub processes_search: Option<overview::TableSearch>,
    pub users_search: Option<overview::TableSearch>,
    pub logs_search: Option<overview::TableSearch>,
    /// New User dialog's text inputs (created while it's open).
    pub new_user_inputs: Option<crate::views::users::NewUserInputs>,
    /// Inspector SET PASSWORD inputs (while the form is open).
    pub password_inputs: Option<crate::views::users::PasswordInputs>,
    /// The Fleet page's background picture, as drawn.
    pub fleet_background: appearance::FleetBackground,
    /// The identity bar's country picker is open (manual region override).
    pub region_picker_open: bool,
    /// Focus the current table page's search box on the next render (`/`).
    pub table_search_focus_pending: bool,
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
    /// Settings → Providers: accounts and the open form (ERR-45).
    pub providers: providers::ProvidersState,
    pub provider_inputs: Option<providers::ProviderFormInputs>,
    pub clanker_inputs: Option<ClankerInputs>,
    /// Fleet → IMPORT FROM PROVIDERS.
    pub import: providers::ImportState,
    /// Snapshot-first offer before a lockout-risk change (ERR-48).
    pub snapshot_offer: Option<configs::SnapshotOffer>,
    /// Something the user should know about their stored secrets (moved,
    /// lost with the keyring, ...), shown on Providers and Clankers.
    pub secrets_notice: Option<String>,
    /// Last key press or click, for auto-lock.
    pub last_activity: std::time::Instant,
    /// The security stance panel is open (titlebar badge).
    pub stance_panel_open: bool,
    /// Older AI keys are still in plain text (checked when keys load, so
    /// rendering never waits on the database).
    pub plaintext_ai_keys: bool,
    /// A secret save waiting for the user to choose a stance (first secret
    /// while Open, ERR-60).
    pub pending_secret: Option<secrets::PendingSecret>,
    /// The stance choice's "I understand" box.
    pub stance_choice_ack: bool,
    /// Vault & Security: change password / go back to Open.
    pub vault_form: vault_manage::VaultFormState,
    pub vault_form_inputs: Option<vault_manage::VaultFormInputs>,
    pub settings: SettingsState,
    /// UI components sandbox (Settings → Components).
    pub lab_state: LabState,

    pub _cursor_blink_task: Task<()>,
    pub _metrics_poll_task: Task<()>,
    /// Deletes archived servers' stored data once its window has closed.
    pub _archive_purge_task: Task<()>,
}

impl CrowApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let vault = Vault::open_default().expect("Failed to initialize vault storage");
        let config = CrowConfigManager::load();

        // Vault records: servers, SSH key hub (with an initial disk scan), AI providers
        let (servers, archived, keys, clanker_providers) = match vault.db().lock() {
            Ok(db) => (
                db.list_servers().unwrap_or_default(),
                db.list_archived_servers().unwrap_or_default(),
                KeysState::load(&db),
                db.list_clanker_providers().unwrap_or_default(),
            ),
            Err(_) => (Vec::new(), Vec::new(), KeysState::new(Vec::new(), Vec::new(), Vec::new(), Vec::new(), None), Vec::new()),
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
            .unwrap_or_else(|| FirewallOperationalState::Unmanaged {
                detected_binaries: Vec::new(),
                reason: "No server is enrolled yet.".into(),
            });
        let configs = load_configs(servers.first(), &initial_firewall_state);
        // The journald editor shows the real journald.conf when there is one.
        let journal_retention = configs
            .states
            .get("journald.conf")
            .map(|st| crate::journal::retention::parse_journald_conf(&st.current_content))
            .unwrap_or(journal_retention);

        let mut app = Self {
            focus_handle: cx.focus_handle(),
            vault,
            config,
            lock_state: LockState::default(),
            setup_state: SetupState::default(),
            screen: Screen::Fleet,
            menu_open: false,
            settings: SettingsState::default(),
            fleet: {
                let mut fleet = FleetState::new(servers, tabs, metrics_store, buffered_stores);
                fleet.archived = archived;
                fleet
            },
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
            onboard_inputs: None,
            onboard_focus_pending: false,
            services_search: None,
            processes_search: None,
            users_search: None,
            logs_search: None,
            new_user_inputs: None,
            password_inputs: None,
            fleet_background: Default::default(),
            region_picker_open: false,
            table_search_focus_pending: false,
            lab_state: LabState::new(window, cx),
            caret: TextCaret { blink: true, ..TextCaret::default() },
            _cursor_blink_task: Self::spawn_cursor_blink(cx),
            _metrics_poll_task: Self::spawn_metrics_poll(cx),
            _archive_purge_task: Self::spawn_archive_purge(cx),
            journal: JournalState::new(initial_journal, journal_retention, journal_telemetry),
            local_lab: LocalLabState::new(lab_engines, lab_nodes),
            show_about_modal: false,
            about_copied_toast: false,
            clankers: ClankersState::new(clanker_providers),
            users: UsersState::new(),
            firewall: FirewallState::new(initial_firewall_state),
            providers: Default::default(),
            provider_inputs: None,
            clanker_inputs: None,
            import: Default::default(),
            snapshot_offer: None,
            secrets_notice: None,
            last_activity: std::time::Instant::now(),
            stance_panel_open: false,
            plaintext_ai_keys: false,
            pending_secret: None,
            stance_choice_ack: false,
            vault_form: Default::default(),
            vault_form_inputs: None,
        };
        app.refresh_providers();
        app.apply_settings();
        app.load_keyring_key(cx);
        app
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
        if self.keys.any_modal_open() {
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

            if (self.active_view == "config" || self.active_view == "configure")
                && self.configs.search_focused
            {
                return true;
            }
        }
        if self.screen == Screen::Settings && (self.provider_inputs.is_some() || self.vault_form_inputs.is_some() || self.clanker_inputs.is_some()) {
            return true;
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
            self.refresh_security(false, cx);
        } else if view == "cron" {
            self.active_view = "cron".to_string();
            self.configs.selected_file = "crontab".to_string();
        } else if view == "firewall" {
            self.active_view = "firewall".to_string();
            self.configs.selected_file = "user.rules".to_string();
        } else if view == "config" {
            self.active_view = view.to_string();
            // Cron and Firewall select their own files; the Config screen only
            // shows files it lists.
            if !self.configs.files.iter().any(|f| f.name == self.configs.selected_file) {
                if let Some(first) = self.configs.files.first() {
                    self.configs.selected_file = first.name.clone();
                }
            }
        } else if view == "users" {
            self.active_view = view.to_string();
            self.refresh_users(cx);
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
