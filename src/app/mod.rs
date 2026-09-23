use std::collections::HashMap;
use gpui_kit::*;
use crate::theme::*;
use crate::components::danger_zone_state::DangerZoneState;
use crate::components::text_caret::TextCaret;
use crate::components::titlebar::ServerTab;
use crate::metrics::{
    collector::{sample_server, CollectorPreviousState},
    MetricSample, ServerTimeSeriesBuffer,
};
use crate::vault::{Vault, VaultStatus};
use crate::views::config::rules_editor::default_hba_rules;
use crate::views::config::state::ConfigsState;
use crate::views::config::cron_editor::{default_cron_jobs, generate_crontab_content};
use crate::views::files::FilesState;
use crate::views::users::UsersState;
use crate::views::firewall::{
    default_active_ufw_state, detect_firewall_status, FirewallOperationalState,
    FirewallState,
};
use crate::config::{crawl_machine_configs, sample_config_content, ConfigFileState, CrowConfigManager};
use crate::os_detect::{classify_distro_family, detect_local_os_release, DistroFamily};
use crate::views::lock::{
    LockState, SetupState,
};
use crate::views::onboard::{OnboardFieldFocus, OnboardState};
use crate::journal::{
    JournalQuery,
    reader::read_journal_for_server,
    retention::{generate_journald_conf, read_retention_for_server},
};
use crate::lab::{detect_local_engines, scan_local_test_nodes};
use crate::views::logs::JournalState;
use crate::views::overview::{
    collector::{
        collect_processes_for_server, collect_services_for_server, collect_sockets_for_server,
    },
    OverviewState,
};

use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;
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
mod overview;
mod keys;
mod configs;
mod journal;
mod clankers;
mod lab;

use crate::keys::{expand_tilde, scan_directory, DiscoveredKey};
use crate::views::fleet::lab_state::LocalLabState;
use crate::views::fleet::FleetState;
use crate::views::settings::clankers_state::ClankersState;
use crate::views::settings::keys_state::KeysState;
use crate::views::settings::state::SettingsState;

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

use gpui_kit::component::input::{InputState, TextareaState, OtpState};
use crate::app::poll::run_background_poll;

pub struct LabState {
    pub text_input: Entity<InputState>,
    pub cleanable_input: Entity<InputState>,
    pub password_input: Entity<InputState>,
    pub prefix_input: Entity<InputState>,
    pub textarea: Entity<TextareaState>,
    pub otp_input: Entity<OtpState>,
    pub custom_compare_text: String,
    pub custom_compare_cursor: usize,
    pub custom_compare_selection: Option<(usize, usize)>,
    pub custom_compare_drag_anchor: Option<usize>,
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

        // Load servers, SSH Key Management records and run initial scan
        let (servers, enrolled_keys, key_groups, scan_paths, discovered_keys, scan_status_message, clanker_providers) = {
            let db = vault.db();
            let res = if let Ok(db_guard) = db.lock() {
                let paths = db_guard.list_scan_paths().unwrap_or_default();
                let groups = db_guard.list_key_groups().unwrap_or_default();
                let keys = db_guard.list_ssh_keys().unwrap_or_default();
                let srvs = db_guard.list_servers().unwrap_or_default();
                let clankers = db_guard.list_clanker_providers().unwrap_or_default();
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
                (srvs, keys, groups, paths, discovered, Some(msg), clankers)
            } else {
                (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), None, Vec::new())
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

        let tabs: Vec<ServerTab> = servers.iter().take(5).map(|s| {
            ServerTab {
                id: s.id.clone(),
                name: s.name.clone(),
                status_color: match s.status.as_str() {
                    "online" => OK,
                    "warn" => WARN,
                    "crit" => CRIT,
                    _ => TEXT_FAINTER,
                },
                is_active: false,
            }
        }).collect();

        let onboard_state = OnboardState::new(&enrolled_keys);

        let lab_state = LabState {
            text_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Type something here to test gpui-component...")
                    .default_value("prod-db-cluster.internal")
            }),
            cleanable_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Type to reveal clear button...")
                    .default_value("search fleet by tag or region...")
            }),
            password_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Enter sensitive secret...")
                    .masked(true)
                    .default_value("crow_vault_master_key_9981")
            }),
            prefix_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("10.0.0.1")
                    .default_value("bastion.eu-west-1.aws")
            }),
            textarea: cx.new(|cx| {
                TextareaState::new(window, cx).default_value(
                    "# Fleet Deployment Manifest\nenv: production\nreplicas: 4\nregion: us-east-1\nauto_drain: true",
                )
            }),
            otp_input: cx.new(|cx| {
                OtpState::new(6, window, cx).default_value("849201")
            }),
            custom_compare_text: "prod-db-cluster.internal".to_string(),
            custom_compare_cursor: 24,
            custom_compare_selection: None,
            custom_compare_drag_anchor: None,
        };

        let mut metrics_store = HashMap::new();
        let mut buffered_stores = HashMap::new();
        let mut local_prev = CollectorPreviousState::default();
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for s in &servers {
            let m = sample_server(s, None, &mut local_prev);
            metrics_store.insert(s.id.clone(), m.clone());
            metrics_store.insert(s.name.clone(), m.clone());

            let mut buf = ServerTimeSeriesBuffer::default();
            buf.push_sample(MetricSample {
                timestamp_secs: now_secs,
                metrics: m,
                services: Vec::new(),
                processes: Vec::new(),
                sockets: Vec::new(),
            });
            buffered_stores.insert(s.id.clone(), buf.clone());
            buffered_stores.insert(s.name.clone(), buf);
        }

        let initial_journal = if let Some(first_srv) = servers.first() {
            read_journal_for_server(first_srv, &JournalQuery::default())
        } else {
            Vec::new()
        };

        let (journal_retention, journal_telemetry) = if let Some(first_srv) = servers.first() {
            read_retention_for_server(&first_srv.host)
        } else {
            read_retention_for_server("local")
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

        if let Some(first_srv) = servers.first() {
            if let Some(buf) = buffered_stores.get_mut(&first_srv.id) {
                if let Some(sample) = buf.samples.back_mut() {
                    sample.services = initial_services.clone();
                    sample.processes = initial_processes.clone();
                    sample.sockets = initial_sockets.clone();
                }
            }
            if let Some(buf) = buffered_stores.get_mut(&first_srv.name) {
                if let Some(sample) = buf.samples.back_mut() {
                    sample.services = initial_services.clone();
                    sample.processes = initial_processes.clone();
                    sample.sockets = initial_sockets.clone();
                }
            }
        }

        let lab_engines = detect_local_engines();
        let lab_nodes = scan_local_test_nodes(&servers);

        let local_distro_family = detect_local_os_release()
            .map(|d| classify_distro_family(&d))
            .unwrap_or(DistroFamily::Unknown);
        let config_files = crawl_machine_configs(local_distro_family);
        let mut config_file_states = HashMap::new();
        for f in &config_files {
            let content = if f.full_path.exists() {
                std::fs::read_to_string(&f.full_path).unwrap_or_else(|_| sample_config_content(&f.name))
            } else {
                sample_config_content(&f.name)
            };
            config_file_states.insert(
                f.name.clone(),
                ConfigFileState::new(f.full_path.clone(), f.name.clone(), content),
            );
        }
        let initial_hba = default_hba_rules();
        let initial_hba_text = crate::views::config::rules_editor::generate_hba_conf(&initial_hba);
        if let Some(st) = config_file_states.get_mut("pg_hba.conf") {
            st.update_content(initial_hba_text);
        }
        let initial_journald_text = generate_journald_conf(&journal_retention);
        if let Some(st) = config_file_states.get_mut("journald.conf") {
            st.baseline_content = initial_journald_text.clone();
            st.current_content = initial_journald_text;
        }
        let initial_cron_jobs = default_cron_jobs();
        let initial_cron_text = generate_crontab_content(&initial_cron_jobs);
        if let Some(st) = config_file_states.get_mut("crontab") {
            st.baseline_content = initial_cron_text.clone();
            st.current_content = initial_cron_text;
        }
        let initial_selected_file = config_files
            .first()
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "journald.conf".to_string());
        let initial_firewall_state = servers
            .first()
            .map(|s| detect_firewall_status(s))
            .unwrap_or_else(default_active_ufw_state);
        if let FirewallOperationalState::Active(ref summary) = initial_firewall_state {
            let fw_text = crate::views::firewall::generate_user_rules_content(&summary.rules);
            if let Some(st) = config_file_states.get_mut("user.rules") {
                st.baseline_content = fw_text.clone();
                st.current_content = fw_text;
            } else {
                let mut st = ConfigFileState::new(std::path::PathBuf::from("/etc/ufw/user.rules"), "user.rules".to_string(), fw_text.clone());
                st.baseline_content = fw_text;
                config_file_states.insert("user.rules".to_string(), st);
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
            settings: SettingsState::default(),
            fleet: FleetState::new(servers, tabs, local_distro_family, metrics_store, buffered_stores),
            active_view: "overview".to_string(),
            overview: OverviewState::new(initial_services, initial_processes, initial_sockets),
            configs: ConfigsState::new(config_files, config_file_states, initial_selected_file, initial_cron_jobs, default_hba_rules()),
            palette_open: false,
            sidebar_collapsed: false,
            keys: KeysState::new(enrolled_keys, key_groups, scan_paths, discovered_keys, scan_status_message),
            files: FilesState::default(),
            danger: DangerZoneState::default(),
            onboard_state,
            lab_state,
            caret: TextCaret { blink: true, ..TextCaret::default() },
            _cursor_blink_task: cx.spawn(async move |entity, cx| {
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(530)).await;
                    let should_notify = entity.update(cx, |this, cx| {
                        if this.has_active_text_input() {
                            this.caret.blink = !this.caret.blink;
                            cx.notify();
                            true
                        } else {
                            false
                        }
                    });
                    if should_notify.is_err() {
                        break;
                    }
                }
            }),
            _metrics_poll_task: cx.spawn(async move |entity, cx| {
                let mut local_prev = CollectorPreviousState::default();
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(2000)).await;
                    let req_res = entity.update(cx, |this, _cx| {
                        this.prepare_poll_request()
                    });
                    let req = match req_res {
                        Ok(r) => r,
                        Err(_) => break,
                    };

                    let (res, next_prev) = cx.background_executor().spawn(async move {
                        run_background_poll(req, local_prev)
                    }).await;

                    local_prev = next_prev;

                    if entity.update(cx, |this, cx| {
                        this.apply_poll_result(res);
                        cx.notify();
                    }).is_err() {
                        break;
                    }
                }
            }),
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
        if view == "services" {
            self.active_view = "overview".to_string();
            self.set_services_tab("services", cx);
        } else if view == "processes" {
            self.active_view = "overview".to_string();
            self.set_services_tab("processes", cx);
        } else if view == "sockets" {
            self.active_view = "overview".to_string();
            self.set_services_tab("sockets", cx);
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
