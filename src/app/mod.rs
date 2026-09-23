use std::collections::HashMap;
use gpui_kit::*;
use crate::theme::*;
use crate::components::danger_zone::{
    danger_zone,
};
use crate::components::danger_zone_state::DangerZoneState;
use crate::components::identity_bar::identity_bar;
use crate::components::palette::palette_overlay;
use crate::components::sidebar::sidebar;
use crate::components::stat_strip::stat_strip;
use crate::components::titlebar::{burger_menu_overlay, titlebar, ServerTab};
use crate::metrics::{
    collector::{sample_server, CollectorPreviousState},
    MetricSample, ServerMetrics, ServerTimeSeriesBuffer, SurgeAlert,
};
use crate::vault::{ServerRecord, Vault, VaultStatus};
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::config::rules_editor::{default_hba_rules, rules_editor};
use crate::views::config::state::ConfigsState;
use crate::views::config::cron_editor::{cron_editor, default_cron_jobs, generate_crontab_content};
use crate::views::config::raw_config_editor;
use crate::views::files::{
    file_browser_view, FilesState,
};
use crate::views::users::{
    user_management_view, UsersState,
};
use crate::views::firewall::{
    default_active_ufw_state, detect_firewall_status, firewall_view, FirewallOperationalState,
    FirewallState,
};
use crate::config::{crawl_machine_configs, sample_config_content, ConfigFileState, CrowConfigManager};
use crate::os_detect::{classify_distro_family, detect_local_os_release, DistroFamily};
use crate::views::fleet::{fleet_overview_view, fleet_setup_view};
use crate::views::lock::{
    vault_lock_view, vault_setup_view, LockFieldFocus, LockState, SetupFieldFocus, SetupState, SetupStep,
};
use crate::views::onboard::{
    append_to_known_hosts, onboard_view, probe_host, OnboardFieldFocus, OnboardState, OnboardStep,
};
use crate::journal::{
    JournalEntry, JournalQuery,
    reader::read_journal_for_server,
    retention::{generate_journald_conf, read_retention_for_server, JournalTelemetry},
};
use crate::lab::{detect_local_engines, scan_local_test_nodes};
use crate::views::logs::{logs_explorer_view, JournalState};
use crate::views::overview::log_tail::{log_tail, socket_log_drawer};
use crate::views::overview::service_inspector::service_inspector_rail;
use crate::views::overview::{
    collector::{
        collect_processes_for_server, collect_services_for_server, collect_sockets_for_server,
    },
    services_table::services_table,
    OverviewState, ProcessUnit, ServiceUnit, SocketUnit,
};
use crate::views::settings::settings_view;

use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;
mod danger;
mod files;
mod firewall;
mod overview;
mod keys;
mod configs;
mod journal;
mod clankers;
mod lab;

use crate::keys::{expand_tilde, scan_directory, DiscoveredKey, KeyGenFieldFocus};
use crate::views::fleet::lab_state::LocalLabState;
use crate::views::settings::clankers_state::ClankersState;
use crate::views::settings::keys_state::KeysState;

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
    pub vault: Vault,
    pub config: CrowConfigManager,
    pub lock_state: LockState,
    pub setup_state: SetupState,
    pub screen: Screen,
    pub menu_open: bool,
    pub settings_section: SettingsSection,
    pub active_tab_id: String,
    pub active_view: String,
    pub overview: OverviewState,
    pub tabs: Vec<ServerTab>,
    pub configs: ConfigsState,
    pub palette_open: bool,
    pub sidebar_collapsed: bool,
    pub settings_dropdown_open: Option<String>,
    pub settings_custom_input: String,
    // SSH Key Management Hub
    pub keys: KeysState,
    // Server Enrollment Subsystem
    pub servers: Vec<ServerRecord>,
    /// This machine's own /etc/os-release family, detected once at startup —
    /// drives which config paths the crawler trusts (see crawl_machine_configs).
    pub local_distro_family: DistroFamily,
    // Files screen — a literal directory browser on top of the server layer.
    pub files: FilesState,
    // Danger Zone — typed-confirmation destructive host actions
    pub danger: DangerZoneState,
    pub onboard_state: OnboardState,
    // UI Components Lab Sandbox
    pub lab_state: LabState,
    // Text input caret and selection state
    pub cursor_blink: bool,
    pub input_cursor: usize,
    pub input_selection: Option<(usize, usize)>,
    pub input_drag_anchor: Option<usize>,
    pub _cursor_blink_task: Task<()>,
    // Real Stats & Metrics Telemetry Store (Lagged Turbo Buffer & Foreknowledge)
    pub metrics_store: HashMap<String, ServerMetrics>,
    pub buffered_stores: HashMap<String, ServerTimeSeriesBuffer>,
    pub metrics_lag_secs: u64,
    pub active_surge_alert: Option<SurgeAlert>,
    pub _metrics_poll_task: Task<()>,
    // Systemd Journal Log Explorer & Retention Boundaries
    pub journal: JournalState,
    // Local Lab & Test VMs Subsystem
    pub local_lab: LocalLabState,
    // About Crow Modal
    pub show_about_modal: bool,
    pub about_copied_toast: bool,
    // Clankers AI Providers & Usability
    pub clankers: ClankersState,
    // User Accounts & Authentication Subsystem
    pub users: UsersState,
    // Firewall & Network Security Subsystem
    pub firewall: FirewallState,
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
            settings_section: SettingsSection::General,
            active_tab_id: servers.first().map(|s| s.id.clone()).unwrap_or_default(),
            active_view: "overview".to_string(),
            overview: OverviewState::new(initial_services, initial_processes, initial_sockets),
            tabs,
            configs: ConfigsState::new(config_files, config_file_states, initial_selected_file, initial_cron_jobs, default_hba_rules()),
            palette_open: false,
            sidebar_collapsed: false,
            settings_dropdown_open: None,
            settings_custom_input: String::new(),
            keys: KeysState::new(enrolled_keys, key_groups, scan_paths, discovered_keys, scan_status_message),
            servers,
            local_distro_family,
            files: FilesState::default(),
            danger: DangerZoneState::default(),
            onboard_state,
            lab_state,
            cursor_blink: true,
            input_cursor: 0,
            input_selection: None,
            input_drag_anchor: None,
            _cursor_blink_task: cx.spawn(async move |entity, cx| {
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(530)).await;
                    let should_notify = entity.update(cx, |this, cx| {
                        if this.has_active_text_input() {
                            this.cursor_blink = !this.cursor_blink;
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
            metrics_store,
            buffered_stores,
            metrics_lag_secs: 24,
            active_surge_alert: None,
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

#[derive(Clone)]
#[allow(dead_code)]
pub struct BackgroundPollRequest {
    pub now_secs: u64,
    pub screen: Screen,
    pub active_view: String,
    pub active_services_tab: String,
    pub active_server: Option<ServerRecord>,
    pub prev_active_metrics: Option<ServerMetrics>,
    pub should_poll_overview_subtab: bool,
    pub should_poll_journal: bool,
    pub journal_query: Option<JournalQuery>,
    pub should_poll_retention: bool,
    pub fleet_servers: Vec<ServerRecord>,
    pub prev_fleet_metrics: HashMap<String, ServerMetrics>,
}

pub struct BackgroundPollResult {
    pub now_secs: u64,
    pub active_server_id: Option<String>,
    pub active_metrics: Option<ServerMetrics>,
    pub services_sample: Option<Vec<ServiceUnit>>,
    pub processes_sample: Option<Vec<ProcessUnit>>,
    pub sockets_sample: Option<Vec<SocketUnit>>,
    pub journal_entries: Option<Vec<JournalEntry>>,
    pub journal_telemetry: Option<JournalTelemetry>,
    pub fleet_samples: Vec<(String, String, ServerMetrics)>,
}

pub fn run_background_poll(
    req: BackgroundPollRequest,
    mut local_prev: CollectorPreviousState,
) -> (BackgroundPollResult, CollectorPreviousState) {
    let mut result = BackgroundPollResult {
        now_secs: req.now_secs,
        active_server_id: req.active_server.as_ref().map(|s| s.id.clone()),
        active_metrics: None,
        services_sample: None,
        processes_sample: None,
        sockets_sample: None,
        journal_entries: None,
        journal_telemetry: None,
        fleet_samples: Vec::new(),
    };

    if let Some(ref active_srv) = req.active_server {
        // 1. Sample active server metrics off-thread
        let updated_head = sample_server(active_srv, req.prev_active_metrics.as_ref(), &mut local_prev);
        result.active_metrics = Some(updated_head);

        // 2. Overview subtabs (ps, ss, systemctl) executed on worker threadpool
        if req.should_poll_overview_subtab {
            match req.active_services_tab.as_str() {
                "processes" => {
                    result.processes_sample = Some(collect_processes_for_server(active_srv));
                }
                "sockets" => {
                    result.sockets_sample = Some(collect_sockets_for_server(active_srv));
                }
                _ => {
                    result.services_sample = Some(collect_services_for_server(active_srv));
                }
            }
        }

        // 3. Journal query (only executed when actively viewing Logs or Overview)
        if req.should_poll_journal {
            if let Some(ref query) = req.journal_query {
                result.journal_entries = Some(read_journal_for_server(active_srv, query));
            }
            if req.should_poll_retention {
                let (_cfg, telemetry) = read_retention_for_server(&active_srv.host);
                result.journal_telemetry = Some(telemetry);
            }
        }
    }

    // 4. Fleet servers (when on Screen::Fleet) executed off-thread
    if !req.fleet_servers.is_empty() {
        for s in &req.fleet_servers {
            let prev = req.prev_fleet_metrics.get(&s.id);
            let updated = sample_server(s, prev, &mut local_prev);
            result.fleet_samples.push((s.id.clone(), s.name.clone(), updated));
        }
    }

    (result, local_prev)
}

impl CrowApp {
    /// The server behind the active tab (tabs are keyed by id, older ones by name).
    pub fn active_server(&self) -> Option<ServerRecord> {
        self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned()
    }

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
        if self.screen == Screen::Settings && self.settings_dropdown_open.is_some() {
            return true;
        }
        false
    }

    pub fn prepare_poll_request(&self) -> BackgroundPollRequest {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let active_srv = self.active_server();
        let prev_active_metrics = active_srv.as_ref().and_then(|srv| {
            self.buffered_stores.get(&srv.id).and_then(|b| b.head()).map(|h| h.metrics.clone()).or_else(|| self.metrics_store.get(&srv.id).cloned())
        });

        let should_poll_overview = self.screen == Screen::Server && self.active_view == "overview";
        let should_poll_journal = self.journal.live_tail
            && self.screen == Screen::Server
            && (self.active_view == "logs" || self.active_view == "overview");
        let should_poll_retention = self.screen == Screen::Server && self.active_view == "logs";

        let journal_query = if should_poll_journal {
            let mut q = self.journal.build_query();
            // In live-tail mode, fetch a lean window of 60 entries for rapid, non-laggy updates
            q.limit = 60;
            Some(q)
        } else {
            None
        };

        let (fleet_servers, prev_fleet_metrics) = if self.screen == Screen::Fleet {
            let mut prev_map = HashMap::new();
            for s in &self.servers {
                if let Some(m) = self.buffered_stores.get(&s.id).and_then(|b| b.head()).map(|h| h.metrics.clone()).or_else(|| self.metrics_store.get(&s.id).cloned()) {
                    prev_map.insert(s.id.clone(), m);
                }
            }
            (self.servers.clone(), prev_map)
        } else {
            (Vec::new(), HashMap::new())
        };

        BackgroundPollRequest {
            now_secs,
            screen: self.screen,
            active_view: self.active_view.clone(),
            active_services_tab: self.overview.active_tab.clone(),
            active_server: active_srv,
            prev_active_metrics,
            should_poll_overview_subtab: should_poll_overview,
            should_poll_journal,
            journal_query,
            should_poll_retention,
            fleet_servers,
            prev_fleet_metrics,
        }
    }

    pub fn apply_poll_result(&mut self, res: BackgroundPollResult) {
        if let Some(ref srv_id) = res.active_server_id {
            if let Some(updated_head) = res.active_metrics {
                // Retrieve current services/processes/sockets or new sample
                let services_sample = res.services_sample.unwrap_or_else(|| self.overview.services.clone());
                let processes_sample = res.processes_sample.unwrap_or_else(|| self.overview.processes.clone());
                let sockets_sample = res.sockets_sample.unwrap_or_else(|| self.overview.sockets.clone());

                // Ingest sample into ring buffer at T_head
                let buf = self.buffered_stores.entry(srv_id.clone()).or_insert_with(ServerTimeSeriesBuffer::default);
                buf.push_sample(MetricSample {
                    timestamp_secs: res.now_secs,
                    metrics: updated_head.clone(),
                    services: services_sample,
                    processes: processes_sample,
                    sockets: sockets_sample,
                });

                // Foreknowledge: scan lookahead window (T_playback, T_head] for upcoming surges
                self.active_surge_alert = buf.detect_upcoming_surge(self.metrics_lag_secs);

                // Playback: query lagged sample from local time-series ring buffer (lag_secs behind)
                if let Some(lagged) = buf.query_lagged(self.metrics_lag_secs) {
                    self.metrics_store.insert(srv_id.clone(), lagged.metrics.clone());
                    if let Some(active_srv) = self.servers.iter().find(|s| s.id == *srv_id) {
                        self.metrics_store.insert(active_srv.name.clone(), lagged.metrics.clone());
                    }

                    // Preserve row focus across replacements
                    if !lagged.services.is_empty() {
                        let focused_name = self.overview.services.iter().find(|s| s.is_focused).map(|s| s.name.clone());
                        self.overview.services = lagged.services.clone();
                        if let Some(name) = focused_name {
                            for svc in &mut self.overview.services {
                                svc.is_focused = svc.name == name;
                            }
                        }
                    }
                    if !lagged.processes.is_empty() {
                        let focused_pid = self.overview.processes.iter().find(|p| p.is_focused).map(|p| p.pid);
                        self.overview.processes = lagged.processes.clone();
                        if let Some(pid) = focused_pid {
                            for proc in &mut self.overview.processes {
                                proc.is_focused = proc.pid == pid;
                            }
                        }
                    }
                    if !lagged.sockets.is_empty() {
                        let focused_idx = self.overview.sockets.iter().position(|s| s.is_focused);
                        self.overview.sockets = lagged.sockets.clone();
                        if let Some(idx) = focused_idx {
                            if let Some(sock) = self.overview.sockets.get_mut(idx) {
                                sock.is_focused = true;
                            }
                        }
                    }
                } else {
                    self.metrics_store.insert(srv_id.clone(), updated_head.clone());
                    if let Some(active_srv) = self.servers.iter().find(|s| s.id == *srv_id) {
                        self.metrics_store.insert(active_srv.name.clone(), updated_head);
                    }
                }

                let buf_clone = buf.clone();
                if let Some(active_srv) = self.servers.iter().find(|s| s.id == *srv_id) {
                    self.buffered_stores.insert(active_srv.name.clone(), buf_clone);
                }
            }
        }

        if let Some(entries) = res.journal_entries {
            self.journal.entries = entries;
        }
        if let Some(telemetry) = res.journal_telemetry {
            self.journal.telemetry = telemetry;
        }

        if !res.fleet_samples.is_empty() {
            for (id, name, m) in res.fleet_samples {
                let buf = self.buffered_stores.entry(id.clone()).or_insert_with(ServerTimeSeriesBuffer::default);
                buf.push_sample(MetricSample {
                    timestamp_secs: res.now_secs,
                    metrics: m.clone(),
                    services: Vec::new(),
                    processes: Vec::new(),
                    sockets: Vec::new(),
                });
                if let Some(lagged) = buf.query_lagged(self.metrics_lag_secs) {
                    self.metrics_store.insert(id.clone(), lagged.metrics.clone());
                    self.metrics_store.insert(name.clone(), lagged.metrics.clone());
                } else {
                    self.metrics_store.insert(id.clone(), m.clone());
                    self.metrics_store.insert(name.clone(), m);
                }
                let buf_clone = buf.clone();
                self.buffered_stores.insert(name, buf_clone);
            }
        }
    }

    #[allow(dead_code)]
    pub fn poll_metrics(&mut self, local_prev: &mut CollectorPreviousState) {
        let req = self.prepare_poll_request();
        let (res, next_prev) = run_background_poll(req, local_prev.clone());
        *local_prev = next_prev;
        self.apply_poll_result(res);
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

    pub fn onboard_select_local_lab_node(&mut self, name: &str, port: &str, distro: &str, cx: &mut Context<Self>) {
        self.onboard_state.host = "127.0.0.1".to_string();
        self.onboard_state.port = port.to_string();
        self.onboard_state.label = name.to_string();
        self.onboard_state.user = "root".to_string();
        self.onboard_state.env = "LAB".to_string();
        self.onboard_state.role = format!("test-node · {}", distro);
        self.onboard_state.facts.distro = distro.to_string();
        cx.notify();
    }

    pub fn set_active_view(&mut self, view: &str, cx: &mut Context<Self>) {
        self.active_view = view.to_string();
        cx.notify();
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
            self.onboard_state = OnboardState::new(&self.keys.enrolled);
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

    pub fn switch_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(srv) = self.servers.iter().find(|s| s.id == tab_id || s.name == tab_id).cloned() {
            if !self.tabs.iter().any(|t| t.id == srv.id || t.name == srv.name) {
                let status_color = match srv.status.as_str() {
                    "online" => OK,
                    "warn" => WARN,
                    "crit" => CRIT,
                    _ => TEXT_FAINTER,
                };
                self.tabs.push(ServerTab {
                    id: srv.id.clone(),
                    name: srv.name.clone(),
                    status_color,
                    is_active: true,
                });
            }
            self.active_tab_id = srv.id.clone();
            for tab in &mut self.tabs {
                tab.is_active = tab.id == self.active_tab_id;
            }
            self.screen = Screen::Server;
            if self.active_view == "overview" {
                let tab_owned = self.overview.active_tab.clone();
                let srv_clone = srv.clone();
                cx.spawn(async move |entity, cx| {
                    match tab_owned.as_str() {
                        "processes" => {
                            let procs = cx.background_executor().spawn(async move {
                                collect_processes_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.overview.processes = procs;
                                cx.notify();
                            });
                        }
                        "sockets" => {
                            let socks = cx.background_executor().spawn(async move {
                                collect_sockets_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.overview.sockets = socks;
                                cx.notify();
                            });
                        }
                        _ => {
                            let svcs = cx.background_executor().spawn(async move {
                                collect_services_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.overview.services = svcs;
                                cx.notify();
                            });
                        }
                    }
                }).detach();
            } else if self.active_view == "files" {
                self.files.current_path = "/".to_string();
                self.files.pending_delete = None;
                self.load_file_listing(cx);
            }
        } else {
            self.active_tab_id = tab_id.to_string();
            self.screen = Screen::Server;
        }
        cx.notify();
    }

    pub fn close_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id || t.name == tab_id) {
            let removed = self.tabs.remove(pos);
            let was_active = self.active_tab_id == tab_id || self.active_tab_id == removed.id || self.active_tab_id == removed.name;
            if was_active {
                if let Some(next_tab) = self.tabs.get(pos).or_else(|| self.tabs.last()) {
                    let next_id = next_tab.id.clone();
                    self.switch_tab(&next_id, cx);
                } else {
                    self.active_tab_id.clear();
                    self.screen = Screen::Fleet;
                }
            }
            cx.notify();
        }
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

    pub fn start_onboarding(&mut self, cx: &mut Context<Self>) {
        self.onboard_state = OnboardState::new(&self.keys.enrolled);
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
            self.keys.enrolled.iter().find(|k| &k.id == kid).map(|k| k.name.as_str()).unwrap_or("ssh-key")
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
        self.keys.toast = Some(format!("Server '{}' enrolled into fleet", name));

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
                if let Some(ref mut gen) = this.keys.gen_modal {
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

                if let Some(ref mut grp) = this.keys.new_group_modal {
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

                if let Some(ref mut sp) = this.keys.add_scan_path_modal {
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

                if let Some(ref mut edit) = this.keys.edit_modal {
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

                if let Some(ref mut clk) = this.clankers.editing {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.close_edit_clanker_modal(cx);
                        return;
                    } else if key == "enter" {
                        this.submit_edit_clanker(cx);
                        return;
                    } else if key == "tab" {
                        clk.focus = match clk.focus {
                            ClankerModalFocus::ApiKey => ClankerModalFocus::Model,
                            ClankerModalFocus::Model => ClankerModalFocus::BaseUrl,
                            ClankerModalFocus::BaseUrl => ClankerModalFocus::ApiKey,
                        };
                        let target_len = match clk.focus {
                            ClankerModalFocus::ApiKey => clk.api_key_input.chars().count(),
                            ClankerModalFocus::Model => clk.model_input.chars().count(),
                            ClankerModalFocus::BaseUrl => clk.base_url_input.chars().count(),
                        };
                        this.input_cursor = target_len;
                        this.input_selection = None;
                        cx.notify();
                        return;
                    }

                    let handled = match clk.focus {
                        ClankerModalFocus::ApiKey => crate::components::handle_text_key_event(
                            &mut clk.api_key_input,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        ClankerModalFocus::Model => crate::components::handle_text_key_event(
                            &mut clk.model_input,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                        ClankerModalFocus::BaseUrl => crate::components::handle_text_key_event(
                            &mut clk.base_url_input,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        ),
                    };
                    if handled {
                        clk.error_message = None;
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

                // Logs Screen: journal search box keyboard interaction
                if this.screen == Screen::Server && this.active_view == "logs" && this.journal.search_focused {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.journal.search_focused = false;
                        cx.notify();
                        return;
                    } else if key == "enter" {
                        this.run_journal_query(cx);
                        return;
                    } else {
                        let changed = crate::components::handle_text_key_event(
                            &mut this.journal.search,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        );
                        if changed {
                            cx.notify();
                        }
                        return;
                    }
                }

                // Danger Zone confirm and Files new-folder prompt are native
                // gpui-component Input widgets now — they own their own focus
                // and keyboard handling, no manual routing needed here.

                // Config Screen: config file search box keyboard interaction
                if this.screen == Screen::Server && (this.active_view == "config" || this.active_view == "configure") && this.configs.search_focused {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.configs.search_focused = false;
                        this.configs.search_query.clear();
                        cx.notify();
                        return;
                    } else {
                        let changed = crate::components::handle_text_key_event(
                            &mut this.configs.search_query,
                            &mut this.input_cursor,
                            &mut this.input_selection,
                            ev,
                        );
                        if changed {
                            cx.notify();
                        }
                        return;
                    }
                }
                if ev.keystroke.key == "escape" {
                    if this.firewall.show_new_rule_modal {
                        this.close_new_firewall_rule_modal(cx);
                    } else if this.users.show_new_user_modal {
                        this.users.show_new_user_modal = false; cx.notify();
                    } else if this.show_about_modal {
                        this.close_about_modal(cx);
                    } else if this.menu_open {
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
                } else if key == "4" && is_mod {
                    this.set_screen(Screen::Server, cx);
                    this.set_view("logs", cx);
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
                            self.servers.len(),
                            self.servers.iter().filter(|s| s.agent_installed).count(),
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
                                    Screen::Server => {
                                        let active_srv = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id);
                                        let active_mtr = self.metrics_store.get(&self.active_tab_id).or_else(|| active_srv.and_then(|s| self.metrics_store.get(&s.id)));
                                        if self.servers.is_empty() || active_srv.is_none() {
                                            let app_fleet = app_view.clone();
                                            let app_add = app_view.clone();
                                            Some(
                                                div()
                                                    .size_full()
                                                    .bg(BG_APP)
                                                    .flex()
                                                    .flex_col()
                                                    .items_center()
                                                    .justify_center()
                                                    .gap(px(16.0))
                                                    .p(px(32.0))
                                                    .child(
                                                        div()
                                                            .size(px(48.0))
                                                            .border_1()
                                                            .border_color(BORDER_STRONG)
                                                            .bg(BG_PANEL)
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                crate::components::icons::tabler_icon(crate::components::icons::TablerIcon::Server)
                                                                    .size(px(24.0))
                                                                    .text_color(TEXT_MUTED),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_size(px(13.0))
                                                            .text_color(TEXT_MAX)
                                                            .child("NO ACTIVE SERVER SELECTED"),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .text_color(TEXT_DIM)
                                                            .child("Select a server tab or enroll a new Linux host to inspect services and config."),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(12.0))
                                                            .mt(px(8.0))
                                                            .child(
                                                                div()
                                                                    .id("empty-server-goto-fleet")
                                                                    .px(px(14.0))
                                                                    .py(px(7.0))
                                                                    .bg(BG_PANEL)
                                                                    .border_1()
                                                                    .border_color(BORDER_STRONG)
                                                                    .text_color(TEXT_PRIMARY)
                                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                                    .cursor_pointer()
                                                                    .font_family(FONT_MONO)
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_size(px(11.0))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        app_fleet.update(cx, |this, cx| {
                                                                            this.set_screen(Screen::Fleet, cx);
                                                                        });
                                                                    })
                                                                    .child("⬢ GO TO FLEET OVERVIEW"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("empty-server-enroll-btn")
                                                                    .px(px(14.0))
                                                                    .py(px(7.0))
                                                                    .bg(OK_BG)
                                                                    .border_1()
                                                                    .border_color(OK)
                                                                    .text_color(OK)
                                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                                    .cursor_pointer()
                                                                    .font_family(FONT_MONO)
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_size(px(11.0))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        app_add.update(cx, |this, cx| {
                                                                            this.set_screen(Screen::Onboard, cx);
                                                                        });
                                                                    })
                                                                    .child("+ ENROLL NEW SERVER"),
                                                            ),
                                                    ),
                                            )
                                        } else {
                                            Some(
                                                div()
                                                    .size_full()
                                                    .flex()
                                                    .flex_col()
                                                    // Server Identity Bar
                                                    .child(identity_bar(active_srv, app_view.clone()))
                                                    // Server Stat Strip
                                                    .child(stat_strip(active_mtr, self.metrics_lag_secs, self.active_surge_alert.as_ref()))
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
                                                                    if self.overview.active_tab == "sockets" {
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .flex_col()
                                                                            .child(
                                                                                div()
                                                                                    .flex_1()
                                                                                    .min_h(px(0.0))
                                                                                    .child(services_table(self, app_view.clone()))
                                                                            )
                                                                            .children(if self.overview.socket_drawer_open {
                                                                                Some(socket_log_drawer(self, app_view.clone()).into_any_element())
                                                                            } else {
                                                                                None
                                                                            })
                                                                    } else {
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .child(services_table(self, app_view.clone()))
                                                                            .child(if self.overview.active_tab == "services" {
                                                                                service_inspector_rail(self, app_view.clone()).into_any_element()
                                                                            } else {
                                                                                log_tail(&self.journal.entries, app_view.clone()).into_any_element()
                                                                            })
                                                                    }
                                                                )
                                                            } else if is_config {
                                                                let editor_view = if self.configs.selected_file == "journald.conf" {
                                                                    crate::views::config::journald_editor::journald_editor(
                                                                        &self.journal.retention,
                                                                        &self.journal.telemetry,
                                                                        self,
                                                                        app_view.clone(),
                                                                    ).into_any_element()
                                                                } else if self.configs.selected_file == "pg_hba.conf" {
                                                                    rules_editor(&self.configs.hba_rules, self, app_view.clone()).into_any_element()
                                                                } else if self.configs.selected_file == "crontab" || self.configs.selected_file.contains("cron") {
                                                                    cron_editor(&self.configs.cron_jobs, self, app_view.clone()).into_any_element()
                                                                } else if let Some(st) = self.configs.states.get(&self.configs.selected_file) {
                                                                    raw_config_editor(st, app_view.clone()).into_any_element()
                                                                } else {
                                                                    rules_editor(&self.configs.hba_rules, self, app_view.clone()).into_any_element()
                                                                };

                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(managed_files_rail(&self.configs.selected_file, self, app_view.clone()))
                                                                        .child(editor_view)
                                                                        .child(pending_diff_rail(self, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "logs" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(logs_explorer_view(app_view.clone(), self))
                                                                )
                                                            } else if self.active_view == "cron" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(cron_editor(&self.configs.cron_jobs, self, app_view.clone()))
                                                                        .child(pending_diff_rail(self, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "users" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(user_management_view(app_view.clone(), &self.users, &self.keys.enrolled))
                                                                )
                                                            } else if self.active_view == "files" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(file_browser_view(app_view.clone(), &self.files))
                                                                )
                                                            } else if self.active_view == "firewall" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(
                                                                            div()
                                                                                .flex_1()
                                                                                .min_w(px(0.0))
                                                                                .h_full()
                                                                                .child(firewall_view(app_view.clone(), &self.firewall, &self.configs.states))
                                                                        )
                                                                        .children(if self.firewall.show_audit_rail {
                                                                            Some(pending_diff_rail(self, app_view.clone()))
                                                                        } else {
                                                                            None
                                                                        })
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
                                            .child(danger_zone(&self.danger, app_view.clone())),
                                        )
                                    }
                                },
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
                            let active_name = self.servers.iter()
                                .find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id)
                                .map(|s| s.name.clone());
                            Some(burger_menu_overlay(app_view.clone(), self.screen, &self.active_view, active_name))
                        } else {
                            None
                        })
                        // 4. Command Palette Overlay (⌘K)
                        .children(if palette_open {
                            let scope = self.servers.iter()
                                .find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id)
                                .map(|s| s.name.as_str())
                                .unwrap_or("Fleet");
                            Some(palette_overlay(app_view.clone(), scope))
                        } else {
                            None
                        })
                        // 5. About Crow Modal
                        .children(if self.show_about_modal {
                            Some(crate::components::about::about_modal(app_view.clone(), self.about_copied_toast))
                        } else {
                            None
                        })
                )
            } else {
                None
            })
    }
}
