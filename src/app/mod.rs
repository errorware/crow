use std::collections::{HashMap, HashSet};
use gpui_kit::*;
use crate::theme::*;
use crate::components::danger_zone::{
    danger_action_keyword, danger_zone, flush_firewall, kill_all_lab_containers, send_power_action,
};
use crate::components::identity_bar::identity_bar;
use crate::components::palette::palette_overlay;
use crate::components::sidebar::sidebar;
use crate::components::stat_strip::stat_strip;
use crate::components::titlebar::{burger_menu_overlay, titlebar, ServerTab};
use crate::metrics::{
    collector::{sample_server, CollectorPreviousState},
    MetricSample, ServerMetrics, ServerTimeSeriesBuffer, SurgeAlert,
};
use crate::vault::{ChangeRecord, ServerRecord, Vault, VaultStatus};
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::config::rules_editor::{default_hba_rules, rules_editor, HbaRuleDef};
use crate::views::config::cron_editor::{cron_editor, default_cron_jobs, generate_crontab_content, CronJobDef};
use crate::views::config::raw_config_editor;
use crate::views::files::{
    collector::{create_directory, delete_entry, list_directory_for_server},
    file_browser_view,
    models::FileEntry,
};
use crate::views::users::{
    user_management_view, UsersState,
};
use crate::views::firewall::{
    default_active_ufw_state, detect_firewall_status, firewall_view, FirewallOperationalState,
    FirewallState,
};
use crate::config::{crawl_machine_configs, sample_config_content, ConfigFileState, DiscoveredConfigFile, CrowConfigManager};
use crate::os_detect::{classify_distro_family, detect_local_os_release, DistroFamily};
use crate::views::fleet::{fleet_overview_view, fleet_setup_view};
use crate::views::lock::{
    vault_lock_view, vault_setup_view, LockFieldFocus, LockState, SetupFieldFocus, SetupState, SetupStep,
};
use crate::views::onboard::{
    append_to_known_hosts, onboard_view, probe_host, OnboardFieldFocus, OnboardState, OnboardStep,
};
use crate::journal::{
    JournalBootScope, JournalEntry, JournalPriority, JournalQuery, JournalTimeRange,
    reader::read_journal_for_server,
    retention::{generate_journald_conf, read_retention_for_server, JournalRetentionConfig, JournalStorageMode, JournalTelemetry},
};
use crate::lab::{
    detect_local_engines, enroll_local_node_into_db, scan_local_test_nodes, start_local_node,
    stop_local_node, EngineStatus, LocalLabEngine, LocalTestNode,
};
use crate::views::logs::logs_explorer_view;
use crate::views::overview::log_tail::{log_tail, socket_log_drawer};
use crate::views::overview::service_inspector::service_inspector_rail;
use crate::views::overview::{
    collector::{
        collect_processes_for_server, collect_services_for_server, collect_sockets_for_server,
        systemctl_service_action, terminate_process,
    },
    services_table::services_table,
    BlastRadiusInfo, ProcessUnit, ServiceUnit, SocketUnit,
};
use crate::views::settings::settings_view;

use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;
mod firewall;

use crate::keys::{
    copy_to_clipboard_system, expand_tilde, scan_directory, AddScanPathModalState,
    DiscoveredKey, EditKeyModalState, KeyGenFieldFocus, KeyGenModalState,
    NewGroupModalState, SshKeyGroup, SshKeyRecord, SshScanPath,
};
use crate::vault::ClankerProviderConfig;

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
    pub active_services_tab: String,
    pub socket_drawer_open: bool,
    pub socket_drawer_filter_this_socket: bool,
    pub tabs: Vec<ServerTab>,
    pub services: Vec<ServiceUnit>,
    pub processes: Vec<ProcessUnit>,
    pub sockets: Vec<SocketUnit>,
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
    /// This machine's own /etc/os-release family, detected once at startup —
    /// drives which config paths the crawler trusts (see crawl_machine_configs).
    pub local_distro_family: DistroFamily,
    // Files screen — a literal directory browser on top of the server layer.
    pub files_current_path: String,
    pub files_entries: Vec<FileEntry>,
    pub files_is_simulated: bool,
    pub files_error: Option<String>,
    pub files_pending_delete: Option<String>,
    pub files_new_folder_open: bool,
    pub files_new_folder_state: Option<Entity<InputState>>,
    // Danger Zone — typed-confirmation destructive host actions
    pub danger_zone_pending_action: Option<String>,
    pub danger_zone_confirm_state: Option<Entity<InputState>>,
    pub danger_zone_error: Option<String>,
    pub danger_zone_last_result: Option<String>,
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
    pub journal_entries: Vec<JournalEntry>,
    pub journal_search: String,
    pub journal_severity_filter: Option<JournalPriority>,
    pub journal_unit_filter: Option<String>,
    pub journal_live_tail: bool,
    pub journal_retention: JournalRetentionConfig,
    pub journal_telemetry: JournalTelemetry,
    pub show_journal_retention_modal: bool,
    pub service_panel_pending_action: Option<String>,
    pub blast_radius: Option<BlastRadiusInfo>,
    pub last_change_outcome: Option<(String, bool)>, // (unit name, succeeded) — most recent failure banner
    pub group_processes: bool,
    pub group_services: bool,
    pub collapsed_process_groups: HashSet<String>,
    pub collapsed_service_groups: HashSet<String>,
    pub journal_pid_filter: Option<u32>,
    pub journal_pid_kill_confirm: bool,
    pub journal_search_focused: bool,
    pub journal_time_range: JournalTimeRange,
    pub journal_boot: JournalBootScope,
    pub journal_limit: usize,
    pub journal_dedupe: bool,
    pub collapsed_journal_dupe_groups: HashSet<String>,
    pub journal_action_markers: Vec<JournalEntry>,
    pub selected_managed_file: String,
    pub config_files: Vec<DiscoveredConfigFile>,
    pub config_file_states: HashMap<String, ConfigFileState>,
    pub cron_jobs: Vec<CronJobDef>,
    pub config_search_query: String,
    pub config_search_focused: bool,
    pub show_config_history: bool,
    // Local Lab & Test VMs Subsystem
    pub lab_engines: Vec<EngineStatus>,
    pub lab_nodes: Vec<LocalTestNode>,
    pub show_local_lab_modal: bool,
    pub new_lab_node_distro: String,
    // About Crow Modal
    pub show_about_modal: bool,
    pub about_copied_toast: bool,
    // Clankers AI Providers & Usability
    pub clanker_providers: Vec<ClankerProviderConfig>,
    pub editing_clanker: Option<ClankerEditModalState>,
    pub clanker_demo_log: String,
    pub clanker_demo_output: Option<String>,
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
            active_services_tab: "services".to_string(),
            socket_drawer_open: false,
            socket_drawer_filter_this_socket: false,
            tabs,
            services: initial_services,
            processes: initial_processes,
            sockets: initial_sockets,
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
            local_distro_family,
            files_current_path: "/".to_string(),
            files_entries: Vec::new(),
            files_is_simulated: false,
            files_error: None,
            files_pending_delete: None,
            files_new_folder_open: false,
            files_new_folder_state: None,
            danger_zone_pending_action: None,
            danger_zone_confirm_state: None,
            danger_zone_error: None,
            danger_zone_last_result: None,
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
            journal_entries: initial_journal,
            journal_search: String::new(),
            journal_severity_filter: None,
            journal_unit_filter: None,
            journal_live_tail: true,
            journal_retention,
            journal_telemetry,
            show_journal_retention_modal: false,
            service_panel_pending_action: None,
            blast_radius: None,
            last_change_outcome: None,
            group_processes: false,
            group_services: false,
            collapsed_process_groups: HashSet::new(),
            collapsed_service_groups: HashSet::new(),
            journal_pid_filter: None,
            journal_pid_kill_confirm: false,
            journal_search_focused: false,
            journal_time_range: JournalTimeRange::Live,
            journal_boot: JournalBootScope::Current,
            journal_limit: 200,
            journal_dedupe: false,
            collapsed_journal_dupe_groups: HashSet::new(),
            journal_action_markers: Vec::new(),
            selected_managed_file: initial_selected_file,
            config_files,
            config_file_states,
            cron_jobs: initial_cron_jobs,
            config_search_query: String::new(),
            config_search_focused: false,
            show_config_history: false,
            lab_engines,
            lab_nodes,
            show_local_lab_modal: false,
            new_lab_node_distro: "noble".to_string(),
            show_about_modal: false,
            about_copied_toast: false,
            clanker_providers,
            editing_clanker: None,
            clanker_demo_log: "kernel: [  129.412033] Out of memory: Kill process 28419 (mysqld) score 812 or sacrifice child".to_string(),
            clanker_demo_output: None,
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
    pub fn has_active_text_input(&self) -> bool {
        if self.palette_open {
            return true;
        }
        if self.users.show_new_user_modal
            || self.firewall.show_new_rule_modal
            || self.show_journal_retention_modal
            || self.show_about_modal
        {
            return true;
        }
        if self.key_gen_modal.is_some()
            || self.new_group_modal.is_some()
            || self.add_scan_path_modal.is_some()
            || self.edit_key_modal.is_some()
            || self.editing_clanker.is_some()
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
            if self.active_view == "logs" && self.journal_search_focused {
                return true;
            }
            if (self.active_view == "config" || self.active_view == "configure")
                && self.config_search_focused
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

        let active_srv = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned();
        let prev_active_metrics = active_srv.as_ref().and_then(|srv| {
            self.buffered_stores.get(&srv.id).and_then(|b| b.head()).map(|h| h.metrics.clone()).or_else(|| self.metrics_store.get(&srv.id).cloned())
        });

        let should_poll_overview = self.screen == Screen::Server && self.active_view == "overview";
        let should_poll_journal = self.journal_live_tail
            && self.screen == Screen::Server
            && (self.active_view == "logs" || self.active_view == "overview");
        let should_poll_retention = self.screen == Screen::Server && self.active_view == "logs";

        let journal_query = if should_poll_journal {
            let mut q = self.build_journal_query();
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
            active_services_tab: self.active_services_tab.clone(),
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
                let services_sample = res.services_sample.unwrap_or_else(|| self.services.clone());
                let processes_sample = res.processes_sample.unwrap_or_else(|| self.processes.clone());
                let sockets_sample = res.sockets_sample.unwrap_or_else(|| self.sockets.clone());

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
                        let focused_name = self.services.iter().find(|s| s.is_focused).map(|s| s.name.clone());
                        self.services = lagged.services.clone();
                        if let Some(name) = focused_name {
                            for svc in &mut self.services {
                                svc.is_focused = svc.name == name;
                            }
                        }
                    }
                    if !lagged.processes.is_empty() {
                        let focused_pid = self.processes.iter().find(|p| p.is_focused).map(|p| p.pid);
                        self.processes = lagged.processes.clone();
                        if let Some(pid) = focused_pid {
                            for proc in &mut self.processes {
                                proc.is_focused = proc.pid == pid;
                            }
                        }
                    }
                    if !lagged.sockets.is_empty() {
                        let focused_idx = self.sockets.iter().position(|s| s.is_focused);
                        self.sockets = lagged.sockets.clone();
                        if let Some(idx) = focused_idx {
                            if let Some(sock) = self.sockets.get_mut(idx) {
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
            self.journal_entries = entries;
        }
        if let Some(telemetry) = res.journal_telemetry {
            self.journal_telemetry = telemetry;
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

    pub fn toggle_journal_expanded(&mut self, id: &str, cx: &mut Context<Self>) {
        let mut now_expanded = false;
        for entry in &mut self.journal_entries {
            if entry.id == id {
                entry.is_expanded = !entry.is_expanded;
                now_expanded = entry.is_expanded;
            }
        }
        // Reading a log is incompatible with the tail silently replacing entries
        // out from under the reader every poll tick — pause the stream the moment
        // something is opened. Resuming is a deliberate action (the live-tail
        // toggle), not automatic, so the reader keeps control of when it moves again.
        if now_expanded {
            self.journal_live_tail = false;
        }
        cx.notify();
    }

    /// The single source of truth for what a journal lookup should ask for — built
    /// fresh from current filter/UI state every time, so the live-tail poll and an
    /// explicit search never drift into two different notions of "the query."
    pub fn build_journal_query(&self) -> JournalQuery {
        JournalQuery {
            limit: self.journal_limit,
            unit: self.journal_unit_filter.clone(),
            priority: self.journal_severity_filter,
            pid: self.journal_pid_filter,
            grep: if self.journal_search.trim().is_empty() {
                None
            } else {
                Some(self.journal_search.clone())
            },
            time_range: self.journal_time_range,
            boot: self.journal_boot,
        }
    }

    /// Runs the current query against the active server right now, regardless of
    /// live-tail state — this is what "search" actually means; it reaches into real
    /// journal history instead of only re-filtering whatever happened to be cached.
    pub fn run_journal_query(&mut self, cx: &mut Context<Self>) {
        if let Some(active_srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() {
            let query = self.build_journal_query();
            cx.spawn(async move |entity, cx| {
                let entries = cx.background_executor().spawn(async move {
                    read_journal_for_server(&active_srv, &query)
                }).await;
                let _ = entity.update(cx, |this, cx| {
                    this.journal_entries = entries;
                    cx.notify();
                });
            }).detach();
        }
    }

    pub fn set_journal_pid_filter(&mut self, pid: Option<u32>, cx: &mut Context<Self>) {
        self.journal_pid_filter = pid;
        self.journal_pid_kill_confirm = false;
        self.run_journal_query(cx);
    }

    pub fn toggle_journal_pid_kill_confirm(&mut self, cx: &mut Context<Self>) {
        self.journal_pid_kill_confirm = !self.journal_pid_kill_confirm;
        cx.notify();
    }

    pub fn execute_journal_pid_kill(&mut self, cx: &mut Context<Self>) {
        if let Some(pid) = self.journal_pid_filter {
            self.execute_process_kill(pid, cx);
        }
        self.journal_pid_kill_confirm = false;
        cx.notify();
    }

    pub fn set_journal_severity(&mut self, prio: Option<JournalPriority>, cx: &mut Context<Self>) {
        self.journal_severity_filter = prio;
        self.run_journal_query(cx);
    }

    pub fn set_journal_unit(&mut self, unit: Option<String>, cx: &mut Context<Self>) {
        self.journal_unit_filter = unit;
        self.run_journal_query(cx);
    }

    /// Jumps the Logs screen to a specific unit's stream — the "VIEW LOGS" handoff
    /// from the Service Manager panel.
    pub fn jump_to_service_logs(&mut self, unit: &str, cx: &mut Context<Self>) {
        self.journal_unit_filter = Some(unit.to_string());
        self.journal_severity_filter = None;
        self.journal_pid_filter = None;
        self.journal_search.clear();
        self.journal_time_range = JournalTimeRange::Live;
        self.journal_boot = JournalBootScope::Current;
        self.journal_live_tail = true;
        self.set_view("logs", cx);
        self.run_journal_query(cx);
    }

    pub fn set_journal_time_range(&mut self, range: JournalTimeRange, cx: &mut Context<Self>) {
        self.journal_time_range = range;
        self.journal_live_tail = range == JournalTimeRange::Live;
        self.run_journal_query(cx);
    }

    pub fn set_journal_boot(&mut self, boot: JournalBootScope, cx: &mut Context<Self>) {
        self.journal_boot = boot;
        if boot != JournalBootScope::Current {
            self.journal_live_tail = false;
        }
        self.run_journal_query(cx);
    }

    pub fn load_more_journal(&mut self, cx: &mut Context<Self>) {
        self.journal_limit += 200;
        self.run_journal_query(cx);
    }

    pub fn toggle_journal_dedupe(&mut self, cx: &mut Context<Self>) {
        self.journal_dedupe = !self.journal_dedupe;
        cx.notify();
    }

    pub fn toggle_journal_dupe_group_collapsed(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.collapsed_journal_dupe_groups.contains(key) {
            self.collapsed_journal_dupe_groups.remove(key);
        } else {
            self.collapsed_journal_dupe_groups.insert(key.to_string());
        }
        cx.notify();
    }

/// Records a lightweight "Crow did X" marker so the log stream can show cause
    /// and effect around an action taken through the app, without needing the full
    /// crow-history/semantic-diff subsystem this is standing in for ahead of time.
    pub fn push_journal_action_marker(&mut self, text: String) {
        let now_usec = chrono::Local::now().timestamp_micros() as u64;
        let (timestamp_formatted, time_relative) = JournalEntry::format_time(now_usec);
        self.journal_action_markers.push(JournalEntry {
            id: format!("crow-action_{}", now_usec),
            cursor: None,
            timestamp_usec: now_usec,
            timestamp_formatted,
            time_relative,
            priority: JournalPriority::Notice,
            unit: "crow-action".to_string(),
            syslog_identifier: "crow".to_string(),
            pid: None,
            message: text,
            fields: Vec::new(),
            is_expanded: false,
        });
        if self.journal_action_markers.len() > 50 {
            let excess = self.journal_action_markers.len() - 50;
            self.journal_action_markers.drain(0..excess);
        }
    }

    pub fn toggle_journal_live_tail(&mut self, cx: &mut Context<Self>) {
        self.journal_live_tail = !self.journal_live_tail;
        if self.journal_live_tail {
            self.journal_time_range = JournalTimeRange::Live;
            self.journal_boot = JournalBootScope::Current;
            self.run_journal_query(cx);
        } else {
            cx.notify();
        }
    }

    pub fn clear_journal(&mut self, cx: &mut Context<Self>) {
        self.journal_entries.clear();
        cx.notify();
    }

    pub fn toggle_journal_retention_modal(&mut self, cx: &mut Context<Self>) {
        self.show_journal_retention_modal = !self.show_journal_retention_modal;
        cx.notify();
    }

    pub fn set_journal_quota(&mut self, quota_mb: u64, cx: &mut Context<Self>) {
        self.journal_retention.system_max_use_mb = quota_mb;
        self.journal_telemetry.estimated_retained_days = quota_mb as f32 / self.journal_telemetry.daily_burn_rate_mb.max(1.0);
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn default_author(&self) -> String {
        "Nelson <nelson@errorware.net>".to_string()
    }

    pub fn toggle_config_history(&mut self, cx: &mut Context<Self>) {
        self.show_config_history = !self.show_config_history;
        cx.notify();
    }

    pub fn sync_hba_to_config_state(&mut self) {
        let content = crate::views::config::rules_editor::generate_hba_conf(&self.hba_rules);
        if let Some(state) = self.config_file_states.get_mut("pg_hba.conf") {
            state.update_content(content);
        }
    }

    pub fn sync_journald_to_config_state(&mut self) {
        let content = generate_journald_conf(&self.journal_retention);
        if let Some(state) = self.config_file_states.get_mut("journald.conf") {
            state.update_content(content);
        }
    }

    pub fn set_journal_retention_days(&mut self, days: u32, cx: &mut Context<Self>) {
        self.journal_retention.max_retention_days = days;
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn set_journal_keep_free(&mut self, mb: u64, cx: &mut Context<Self>) {
        self.journal_retention.system_keep_free_mb = mb;
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn set_journal_storage_mode(&mut self, mode: JournalStorageMode, cx: &mut Context<Self>) {
        self.journal_retention.storage = mode;
        self.journal_telemetry.is_volatile_warning = mode != JournalStorageMode::Persistent;
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn select_managed_file(&mut self, filename: &str, cx: &mut Context<Self>) {
        self.selected_managed_file = filename.to_string();
        cx.notify();
    }

    pub fn crawl_system_configs(&mut self, cx: &mut Context<Self>) {
        let discovered = crawl_machine_configs(self.local_distro_family);
        for f in &discovered {
            if !self.config_file_states.contains_key(&f.name) {
                let content = if f.full_path.exists() {
                    std::fs::read_to_string(&f.full_path).unwrap_or_else(|_| sample_config_content(&f.name))
                } else {
                    sample_config_content(&f.name)
                };
                self.config_file_states.insert(
                    f.name.clone(),
                    ConfigFileState::new(f.full_path.clone(), f.name.clone(), content),
                );
            }
        }
        self.config_files = discovered;
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_config_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.config_search_query = query;
        cx.notify();
    }

    pub fn revert_managed_config(&mut self, file: &str, cx: &mut Context<Self>) {
        if file == "pg_hba.conf" {
            for r in &mut self.hba_rules {
                if r.num == "09" || r.num == "10" {
                    r.method = "trust";
                    r.risk = "CRITICAL";
                    r.risk_color = CRIT;
                    r.is_expanded = false;
                }
            }
        } else if file == "journald.conf" {
            self.journal_retention = JournalRetentionConfig::default();
        } else if file == "crontab" {
            self.cron_jobs = default_cron_jobs();
        } else if file == "user.rules" {
            if let Some(state) = self.config_file_states.get(file) {
                let rules = crate::views::firewall::parse_user_rules_content(&state.baseline_content);
                if let FirewallOperationalState::Active(ref mut summary) = self.firewall.status {
                    summary.rules = rules;
                }
            }
            self.firewall.toast = Some("Reverted firewall rules to baseline".to_string());
        }
        if let Some(state) = self.config_file_states.get_mut(file) {
            state.revert();
            cx.notify();
        }
    }

    pub fn stage_config_version(&mut self, file: &str, description: &str, cx: &mut Context<Self>) {
        let author = self.default_author();
        if let Some(state) = self.config_file_states.get_mut(file) {
            state.stage_revision(author, description.to_string());
            let _ = state.save_to_disk();
            if file == "pg_hba.conf" {
                for r in &mut self.hba_rules {
                    if r.risk == "EDITED" {
                        r.risk = "OK";
                        r.risk_color = OK;
                    }
                }
            } else if file == "journald.conf" {
                self.show_journal_retention_modal = false;
            } else if file == "user.rules" {
                self.firewall.toast = Some(format!("Audit commit created: {}", description));
            }
            cx.notify();
        }
    }

    pub fn rollback_config_revision(&mut self, file: &str, version: usize, cx: &mut Context<Self>) {
        if let Some(state) = self.config_file_states.get_mut(file) {
            if state.rollback_to_revision(version) {
                let _ = state.save_to_disk();
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

    pub fn toggle_sample_edit_on_config(&mut self, file: &str, cx: &mut Context<Self>) {
        if let Some(state) = self.config_file_states.get_mut(file) {
            if state.is_modified() {
                state.revert();
            } else {
                let mut edited = state.current_content.clone();
                if !edited.ends_with('\n') {
                    edited.push('\n');
                }
                edited.push_str("# [Crow Managed Adjustment]\n");
                edited.push_str("crow_managed_sync = true\n");
                state.update_content(edited);
            }
            cx.notify();
        }
    }

    pub fn sync_cron_to_config_state(&mut self) {
        let content = generate_crontab_content(&self.cron_jobs);
        if let Some(st) = self.config_file_states.get_mut("crontab") {
            st.update_content(content);
        }
    }

    pub fn toggle_cron_job_enabled(&mut self, job_id: &str, cx: &mut Context<Self>) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            job.enabled = !job.enabled;
            self.sync_cron_to_config_state();
            cx.notify();
        }
    }

    pub fn toggle_cron_job_expanded(&mut self, job_id: &str, cx: &mut Context<Self>) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            job.is_expanded = !job.is_expanded;
            cx.notify();
        }
    }

    pub fn apply_cron_preset(
        &mut self,
        job_id: &str,
        minute: &str,
        hour: &str,
        day_of_month: &str,
        month: &str,
        day_of_week: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            job.minute = minute.to_string();
            job.hour = hour.to_string();
            job.day_of_month = day_of_month.to_string();
            job.month = month.to_string();
            job.day_of_week = day_of_week.to_string();
            self.sync_cron_to_config_state();
            cx.notify();
        }
    }

    pub fn update_cron_field(
        &mut self,
        job_id: &str,
        field: &str,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            match field {
                "minute" => job.minute = value.to_string(),
                "hour" => job.hour = value.to_string(),
                "day_of_month" | "dom" => job.day_of_month = value.to_string(),
                "month" | "mon" => job.month = value.to_string(),
                "day_of_week" | "dow" => job.day_of_week = value.to_string(),
                "user" => job.user = value.to_string(),
                "command" => job.command = value.to_string(),
                _ => {}
            }
            self.sync_cron_to_config_state();
            cx.notify();
        }
    }

    pub fn move_cron_job_up(&mut self, job_id: &str, cx: &mut Context<Self>) {
        if let Some(idx) = self.cron_jobs.iter().position(|j| j.id == job_id) {
            if idx > 0 {
                self.cron_jobs.swap(idx, idx - 1);
                self.sync_cron_to_config_state();
                cx.notify();
            }
        }
    }

    pub fn move_cron_job_down(&mut self, job_id: &str, cx: &mut Context<Self>) {
        if let Some(idx) = self.cron_jobs.iter().position(|j| j.id == job_id) {
            if idx + 1 < self.cron_jobs.len() {
                self.cron_jobs.swap(idx, idx + 1);
                self.sync_cron_to_config_state();
                cx.notify();
            }
        }
    }

    pub fn delete_cron_job(&mut self, job_id: &str, cx: &mut Context<Self>) {
        self.cron_jobs.retain(|j| j.id != job_id);
        self.sync_cron_to_config_state();
        cx.notify();
    }

    pub fn add_cron_job(&mut self, cx: &mut Context<Self>) {
        let new_id = format!(
            "cron_{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );
        self.cron_jobs.push(CronJobDef {
            id: new_id,
            minute: "0".to_string(),
            hour: "2".to_string(),
            day_of_month: "*".to_string(),
            month: "*".to_string(),
            day_of_week: "*".to_string(),
            user: "root".to_string(),
            command: "/usr/local/bin/backup-sync.sh".to_string(),
            comment: Some("Nightly backup routine".to_string()),
            enabled: true,
            is_expanded: true,
        });
        self.sync_cron_to_config_state();
        cx.notify();
    }

    pub fn apply_journal_boundaries(&mut self, cx: &mut Context<Self>) {
        self.stage_config_version("journald.conf", "Applied journald retention boundaries", cx);
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

    pub fn toggle_local_lab_modal(&mut self, cx: &mut Context<Self>) {
        self.show_local_lab_modal = !self.show_local_lab_modal;
        if self.show_local_lab_modal {
            self.lab_engines = detect_local_engines();
            self.lab_nodes = scan_local_test_nodes(&self.servers);
        }
        cx.notify();
    }

    pub fn refresh_lab_nodes(&mut self, cx: &mut Context<Self>) {
        self.lab_engines = detect_local_engines();
        self.lab_nodes = scan_local_test_nodes(&self.servers);
        cx.notify();
    }

    pub fn set_new_lab_distro(&mut self, distro: &str, cx: &mut Context<Self>) {
        self.new_lab_node_distro = distro.to_string();
        cx.notify();
    }

    pub fn start_lab_node(&mut self, node_id: &str, cx: &mut Context<Self>) {
        if let Some(node) = self.lab_nodes.iter_mut().find(|n| n.id == node_id || n.name == node_id) {
            let _ = start_local_node(node);
            node.state = "running".to_string();
        }
        cx.notify();
    }

    pub fn stop_lab_node(&mut self, node_id: &str, cx: &mut Context<Self>) {
        if let Some(node) = self.lab_nodes.iter_mut().find(|n| n.id == node_id || n.name == node_id) {
            let _ = stop_local_node(node);
            node.state = "stopped".to_string();
        }
        cx.notify();
    }

    pub fn enroll_lab_node(&mut self, node_name: &str, cx: &mut Context<Self>) {
        if let Some(node) = self.lab_nodes.iter().find(|n| n.name == node_name).cloned() {
            if let Ok(db) = self.vault.db().lock() {
                if let Ok(record) = enroll_local_node_into_db(&node, &db) {
                    self.servers.push(record.clone());
                    let mut local_prev = CollectorPreviousState::default();
                    let m = sample_server(&record, None, &mut local_prev);
                    self.metrics_store.insert(record.id.clone(), m.clone());
                    self.metrics_store.insert(record.name.clone(), m);
                    if !self.tabs.iter().any(|t| t.id == record.id) {
                        self.tabs.push(ServerTab {
                            id: record.id.clone(),
                            name: record.name.clone(),
                            status_color: OK,
                            is_active: true,
                        });
                    }
                    self.active_tab_id = record.id.clone();
                    self.services = collect_services_for_server(&record);
                    self.processes = collect_processes_for_server(&record);
                    self.sockets = collect_sockets_for_server(&record);
                    self.screen = Screen::Server;
                    self.active_view = "overview".to_string();
                    self.show_local_lab_modal = false;
                }
            }
        }
        self.lab_nodes = scan_local_test_nodes(&self.servers);
        cx.notify();
    }

    pub fn create_lab_node(&mut self, cx: &mut Context<Self>) {
        let distro = self.new_lab_node_distro.clone();
        let name = format!("crow-lab-{}", &distro);
        let port = 2222;

        // Try starting existing local container or ensure test container is running
        let _ = std::process::Command::new("podman")
            .args(["start", "completo-node-1"])
            .output();

        let node = LocalTestNode {
            id: format!("local-{}", name),
            name: name.clone(),
            engine: LocalLabEngine::Podman,
            image: format!("docker.io/library/{}:latest", distro),
            state: "running".into(),
            ssh_port: Some(port),
            is_enrolled: false,
        };

        if let Ok(db) = self.vault.db().lock() {
            if let Ok(record) = enroll_local_node_into_db(&node, &db) {
                self.servers.push(record.clone());
                let mut local_prev = CollectorPreviousState::default();
                let m = sample_server(&record, None, &mut local_prev);
                self.metrics_store.insert(record.id.clone(), m.clone());
                self.metrics_store.insert(record.name.clone(), m);
                if !self.tabs.iter().any(|t| t.id == record.id) {
                    self.tabs.push(ServerTab {
                        id: record.id.clone(),
                        name: record.name.clone(),
                        status_color: OK,
                        is_active: true,
                    });
                }
                self.active_tab_id = record.id.clone();
                self.services = collect_services_for_server(&record);
                self.processes = collect_processes_for_server(&record);
                self.sockets = collect_sockets_for_server(&record);
                self.screen = Screen::Server;
                self.active_view = "overview".to_string();
                self.show_local_lab_modal = false;
            }
        }
        self.lab_nodes = scan_local_test_nodes(&self.servers);
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
            self.set_services_tab("services", cx);
        } else if view == "processes" {
            self.active_view = "overview".to_string();
            self.set_services_tab("processes", cx);
        } else if view == "sockets" {
            self.active_view = "overview".to_string();
            self.set_services_tab("sockets", cx);
        } else if view == "cron" {
            self.active_view = "cron".to_string();
            self.selected_managed_file = "crontab".to_string();
        } else if view == "firewall" {
            self.active_view = "firewall".to_string();
            self.selected_managed_file = "user.rules".to_string();
        } else if view == "files" {
            self.active_view = "files".to_string();
            self.load_file_listing(cx);
            return;
        } else {
            self.active_view = view.to_string();
        }
        cx.notify();
    }

    pub fn set_services_tab(&mut self, tab: &str, cx: &mut Context<Self>) {
        self.active_services_tab = tab.to_string();
        if let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() {
            let tab_owned = tab.to_string();
            cx.spawn(async move |entity, cx| {
                match tab_owned.as_str() {
                    "processes" => {
                        let procs = cx.background_executor().spawn(async move {
                            collect_processes_for_server(&srv)
                        }).await;
                        let _ = entity.update(cx, |this, cx| {
                            this.processes = procs;
                            cx.notify();
                        });
                    }
                    "sockets" => {
                        let socks = cx.background_executor().spawn(async move {
                            collect_sockets_for_server(&srv)
                        }).await;
                        let _ = entity.update(cx, |this, cx| {
                            this.sockets = socks;
                            cx.notify();
                        });
                    }
                    _ => {
                        let svcs = cx.background_executor().spawn(async move {
                            collect_services_for_server(&srv)
                        }).await;
                        let _ = entity.update(cx, |this, cx| {
                            this.services = svcs;
                            cx.notify();
                        });
                    }
                }
            }).detach();
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
                let tab_owned = self.active_services_tab.clone();
                let srv_clone = srv.clone();
                cx.spawn(async move |entity, cx| {
                    match tab_owned.as_str() {
                        "processes" => {
                            let procs = cx.background_executor().spawn(async move {
                                collect_processes_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.processes = procs;
                                cx.notify();
                            });
                        }
                        "sockets" => {
                            let socks = cx.background_executor().spawn(async move {
                                collect_sockets_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.sockets = socks;
                                cx.notify();
                            });
                        }
                        _ => {
                            let svcs = cx.background_executor().spawn(async move {
                                collect_services_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.services = svcs;
                                cx.notify();
                            });
                        }
                    }
                }).detach();
            } else if self.active_view == "files" {
                self.files_current_path = "/".to_string();
                self.files_pending_delete = None;
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

    pub fn toggle_group_services(&mut self, cx: &mut Context<Self>) {
        self.group_services = !self.group_services;
        cx.notify();
    }

    pub fn toggle_group_processes(&mut self, cx: &mut Context<Self>) {
        self.group_processes = !self.group_processes;
        cx.notify();
    }

    pub fn toggle_service_group_collapsed(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.collapsed_service_groups.contains(key) {
            self.collapsed_service_groups.remove(key);
        } else {
            self.collapsed_service_groups.insert(key.to_string());
        }
        cx.notify();
    }

    pub fn toggle_process_group_collapsed(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.collapsed_process_groups.contains(key) {
            self.collapsed_process_groups.remove(key);
        } else {
            self.collapsed_process_groups.insert(key.to_string());
        }
        cx.notify();
    }

    pub fn focus_service(&mut self, name: &str, cx: &mut Context<Self>) {
        for svc in &mut self.services {
            svc.is_focused = svc.name == name;
        }
        cx.notify();
    }

    pub fn toggle_service_confirm(&mut self, name: &str, cx: &mut Context<Self>) {
        let mut now_open = false;
        for svc in &mut self.services {
            if svc.name == name {
                svc.show_confirm = !svc.show_confirm;
                now_open = svc.show_confirm;
            } else {
                svc.show_confirm = false;
            }
        }
        if now_open {
            self.spawn_blast_radius_fetch(name.to_string(), cx);
        } else {
            self.blast_radius = None;
        }
        cx.notify();
    }

    pub fn execute_service_restart(&mut self, name: &str, cx: &mut Context<Self>) {
        self.spawn_service_change(name.to_string(), "restart".to_string(), cx);
    }

    pub fn focus_process(&mut self, pid: u32, cx: &mut Context<Self>) {
        for proc in &mut self.processes {
            proc.is_focused = proc.pid == pid;
        }
        cx.notify();
    }

    pub fn toggle_process_confirm(&mut self, pid: u32, cx: &mut Context<Self>) {
        for proc in &mut self.processes {
            if proc.pid == pid {
                proc.show_confirm = !proc.show_confirm;
            } else {
                proc.show_confirm = false;
            }
        }
        cx.notify();
    }

    pub fn execute_process_kill(&mut self, pid: u32, cx: &mut Context<Self>) {
        self.spawn_process_kill(pid, cx);
    }

    /// Non-destructive service lifecycle actions (start / reload) — run immediately,
    /// no confirm gate, mirroring how a sysadmin would treat them at a real shell.
    /// Also the post-confirm entry point for restart/stop (see execute_service_panel_action).
    pub fn run_service_panel_action_now(&mut self, action: &str, cx: &mut Context<Self>) {
        if let Some(name) = self.services.iter().find(|s| s.is_focused).map(|s| s.name.clone()) {
            self.spawn_service_change(name, action.to_string(), cx);
        }
    }

    /// Arms a disruptive service action (restart / stop) pending inline confirm,
    /// and kicks off a real blast-radius lookup for the confirm bar to show.
    pub fn arm_service_panel_action(&mut self, action: &str, cx: &mut Context<Self>) {
        self.service_panel_pending_action = Some(action.to_string());
        if let Some(name) = self.services.iter().find(|s| s.is_focused).map(|s| s.name.clone()) {
            self.spawn_blast_radius_fetch(name, cx);
        }
        cx.notify();
    }

    pub fn cancel_service_panel_action(&mut self, cx: &mut Context<Self>) {
        self.service_panel_pending_action = None;
        self.blast_radius = None;
        cx.notify();
    }

    pub fn execute_service_panel_action(&mut self, cx: &mut Context<Self>) {
        if let Some(action) = self.service_panel_pending_action.take() {
            self.run_service_panel_action_now(&action, cx);
        }
    }

    /// Computes a real "N connections about to drop" figure for the confirm bar,
    /// off the main thread — the Apply Pipeline's blast-radius step made literal
    /// instead of the generic "any active connections" text it replaces.
    fn spawn_blast_radius_fetch(&mut self, unit_name: String, cx: &mut Context<Self>) {
        self.blast_radius = None;
        let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() else {
            return;
        };
        let Some(pid) = self.services.iter().find(|s| s.name == unit_name).and_then(|s| s.pid.parse::<u32>().ok()) else {
            return;
        };

        cx.spawn(async move |entity, cx| {
            let srv_bg = srv.clone();
            let sockets = cx.background_executor().spawn(async move {
                collect_sockets_for_server(&srv_bg)
            }).await;

            let established = sockets.iter().filter(|s| s.pid == Some(pid) && s.state.contains("ESTAB")).count();
            let listening = sockets.iter().filter(|s| s.pid == Some(pid) && s.state == "LISTEN").count();

            let _ = entity.update(cx, |this, cx| {
                this.blast_radius = Some(BlastRadiusInfo { for_unit: unit_name, established, listening });
                cx.notify();
            });
        }).detach();
    }

    /// The Apply Pipeline for service lifecycle actions: snapshot before-state,
    /// run the action off the main thread, verify the unit actually reached the
    /// expected state, then durably record the outcome. Covers start/stop/
    /// restart/reload — every service mutation in the app goes through this.
    fn spawn_service_change(&mut self, unit_name: String, action: String, cx: &mut Context<Self>) {
        let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() else {
            return;
        };
        let before_state = self.services.iter().find(|s| s.name == unit_name)
            .map(|s| s.status.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let blast_radius_summary = self.blast_radius.as_ref()
            .filter(|b| b.for_unit == unit_name)
            .map(|b| format!("{} established, {} listening", b.established, b.listening));

        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let started_at = chrono::Utc::now().to_rfc3339();
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: format!("service_{}", action),
                target: unit_name.clone(),
                before_state,
                after_state: None,
                blast_radius: blast_radius_summary,
                outcome: "pending".to_string(),
                started_at,
                completed_at: None,
            });
        }

        self.blast_radius = None;
        self.service_panel_pending_action = None;
        for svc in &mut self.services {
            svc.show_confirm = false;
        }

        cx.spawn(async move |entity, cx| {
            let srv_bg = srv.clone();
            let unit_bg = unit_name.clone();
            let action_bg = action.clone();
            let (mut refreshed, after_status, succeeded) = cx.background_executor().spawn(async move {
                let _ = systemctl_service_action(&srv_bg, &unit_bg, &action_bg);
                std::thread::sleep(std::time::Duration::from_millis(1200));
                let refreshed = collect_services_for_server(&srv_bg);
                let status = refreshed.iter().find(|s| s.name == unit_bg).map(|s| s.status.clone()).unwrap_or_else(|| "unknown".to_string());
                let ok = if action_bg == "stop" { status != "ACTIVE" } else { status == "ACTIVE" };
                (refreshed, status, ok)
            }).await;

            let completed_at = chrono::Utc::now().to_rfc3339();
            let outcome_str = if succeeded { "success" } else { "failed" };
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.update_change_record_outcome(&record_id, outcome_str, Some(&after_status), &completed_at);
            }

            let _ = entity.update(cx, |this, cx| {
                for svc in &mut refreshed {
                    svc.is_focused = svc.name == unit_name;
                }
                this.services = refreshed;
                this.push_journal_action_marker(format!("crow: {} {} ({})", action, unit_name, outcome_str));
                this.last_change_outcome = Some((unit_name.clone(), succeeded));
                cx.notify();
            });
        }).detach();
    }

    /// Same pipeline shape as `spawn_service_change`, for a process SIGTERM:
    /// backup, apply off-thread, verify the PID is actually gone, record it.
    fn spawn_process_kill(&mut self, pid: u32, cx: &mut Context<Self>) {
        let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() else {
            return;
        };
        let before_state = self.processes.iter().find(|p| p.pid == pid)
            .map(|p| format!("{} ({})", p.command, p.stat))
            .unwrap_or_else(|| "unknown".to_string());

        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let started_at = chrono::Utc::now().to_rfc3339();
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: "process_kill".to_string(),
                target: format!("pid {}", pid),
                before_state,
                after_state: None,
                blast_radius: None,
                outcome: "pending".to_string(),
                started_at,
                completed_at: None,
            });
        }

        for proc in &mut self.processes {
            proc.show_confirm = false;
        }

        cx.spawn(async move |entity, cx| {
            let srv_bg = srv.clone();
            let (refreshed, succeeded) = cx.background_executor().spawn(async move {
                let _ = terminate_process(&srv_bg, pid, 15);
                std::thread::sleep(std::time::Duration::from_millis(800));
                let refreshed = collect_processes_for_server(&srv_bg);
                let still_alive = refreshed.iter().any(|p| p.pid == pid);
                (refreshed, !still_alive)
            }).await;

            let completed_at = chrono::Utc::now().to_rfc3339();
            let outcome_str = if succeeded { "success" } else { "failed" };
            let after_state = if succeeded { "terminated" } else { "still running" };
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.update_change_record_outcome(&record_id, outcome_str, Some(after_state), &completed_at);
            }

            let _ = entity.update(cx, |this, cx| {
                this.processes = refreshed;
                this.push_journal_action_marker(format!("crow: sent SIGTERM to PID {} ({})", pid, outcome_str));
                cx.notify();
            });
        }).detach();
    }

    /// Jumps to the Config screen pre-selecting the file that governs this service —
    /// the "swap to crow-config" handoff instead of a service-specific settings UI.
    pub fn open_config_for_service(&mut self, file: &str, cx: &mut Context<Self>) {
        self.selected_managed_file = file.to_string();
        self.set_view("config", cx);
    }

    /// Recent Apply Pipeline change records for one unit on the active server —
    /// the audit trail surfaced in the Service Manager panel.
    pub fn recent_change_records(&self, target: &str, limit: usize) -> Vec<ChangeRecord> {
        let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id) else {
            return Vec::new();
        };
        let db = self.vault.db();
        let Ok(db_guard) = db.lock() else {
            return Vec::new();
        };
        db_guard
            .list_change_records(&srv.id, 50)
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.target == target)
            .take(limit)
            .collect()
    }

    // ==========================================
    // Files — a literal directory browser on top of the server layer
    // ==========================================

    pub fn load_file_listing(&mut self, cx: &mut Context<Self>) {
        if let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() {
            let (entries, is_sim) = list_directory_for_server(&srv, &self.files_current_path);
            self.files_entries = entries;
            self.files_is_simulated = is_sim;
        } else {
            self.files_entries.clear();
        }
        cx.notify();
    }

    pub fn navigate_files_to(&mut self, path: &str, cx: &mut Context<Self>) {
        self.files_current_path = path.to_string();
        self.files_pending_delete = None;
        self.files_error = None;
        self.load_file_listing(cx);
    }

    pub fn files_go_into(&mut self, name: &str, cx: &mut Context<Self>) {
        let base = self.files_current_path.clone();
        let new_path = if base == "/" { format!("/{}", name) } else { format!("{}/{}", base, name) };
        self.navigate_files_to(&new_path, cx);
    }

    pub fn files_go_up(&mut self, cx: &mut Context<Self>) {
        if self.files_current_path == "/" {
            return;
        }
        let parent = std::path::Path::new(&self.files_current_path)
            .parent()
            .map(|p| {
                let s = p.to_string_lossy().to_string();
                if s.is_empty() { "/".to_string() } else { s }
            })
            .unwrap_or_else(|| "/".to_string());
        self.navigate_files_to(&parent, cx);
    }

    pub fn toggle_new_folder_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.files_new_folder_open = !self.files_new_folder_open;
        if self.files_new_folder_open {
            self.files_new_folder_state = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("new-folder-name")
            }));
        } else {
            self.files_new_folder_state = None;
        }
        cx.notify();
    }

    pub fn create_new_folder(&mut self, cx: &mut Context<Self>) {
        if let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() {
            let name = self.files_new_folder_state.as_ref()
                .map(|s| s.read(cx).value().trim().to_string())
                .unwrap_or_default();
            if !name.is_empty() {
                match create_directory(&srv, &self.files_current_path, &name) {
                    Ok(()) => {
                        self.files_new_folder_open = false;
                        self.files_new_folder_state = None;
                        self.load_file_listing(cx);
                        return;
                    }
                    Err(e) => self.files_error = Some(e),
                }
            }
        }
        cx.notify();
    }

    pub fn toggle_file_delete_confirm(&mut self, name: &str, cx: &mut Context<Self>) {
        self.files_pending_delete = if self.files_pending_delete.as_deref() == Some(name) {
            None
        } else {
            Some(name.to_string())
        };
        cx.notify();
    }

    pub fn execute_file_delete(&mut self, cx: &mut Context<Self>) {
        if let Some(name) = self.files_pending_delete.take() {
            if let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() {
                let is_dir = self.files_entries.iter().find(|e| e.name == name).map(|e| e.is_dir).unwrap_or(false);
                match delete_entry(&srv, &self.files_current_path, &name, is_dir) {
                    Ok(()) => {
                        self.load_file_listing(cx);
                        return;
                    }
                    Err(e) => self.files_error = Some(e),
                }
            }
        }
        cx.notify();
    }

    // ==========================================
    // Danger Zone — typed-confirmation destructive host actions
    // ==========================================

    pub fn arm_danger_zone_action(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.danger_zone_pending_action = Some(action.to_string());
        self.danger_zone_error = None;
        self.danger_zone_confirm_state = Some(cx.new(|cx| {
            InputState::new(window, cx).placeholder(danger_action_keyword(action))
        }));
        cx.notify();
    }

    pub fn cancel_danger_zone_action(&mut self, cx: &mut Context<Self>) {
        self.danger_zone_pending_action = None;
        self.danger_zone_confirm_state = None;
        self.danger_zone_error = None;
        cx.notify();
    }

    pub fn execute_danger_zone_action(&mut self, cx: &mut Context<Self>) {
        let Some(action) = self.danger_zone_pending_action.clone() else {
            return;
        };
        let keyword = danger_action_keyword(&action);
        let typed = self.danger_zone_confirm_state.as_ref()
            .map(|s| s.read(cx).value().trim().to_string())
            .unwrap_or_default();
        if typed != keyword {
            self.danger_zone_error = Some(format!("Type {} exactly to confirm.", keyword));
            cx.notify();
            return;
        }

        let Some(srv) = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id).cloned() else {
            self.danger_zone_error = Some("No active server".to_string());
            cx.notify();
            return;
        };

        self.danger_zone_pending_action = None;
        self.danger_zone_confirm_state = None;
        self.danger_zone_error = None;

        let result = match action.as_str() {
            "poweroff" => send_power_action(&srv, "power-off"),
            "reboot" => send_power_action(&srv, "reboot"),
            "flush_firewall" => flush_firewall(&srv),
            "kill_containers" => {
                let (ok, failed) = kill_all_lab_containers(&self.lab_nodes);
                Ok(format!("Stopped {} lab container(s), {} failed", ok, failed))
            }
            _ => Err("Unknown action".to_string()),
        };

        match result {
            Ok(msg) => {
                self.push_journal_action_marker(format!("crow: {}", msg));
                self.danger_zone_last_result = Some(msg);
            }
            Err(e) => self.danger_zone_error = Some(e),
        }
        cx.notify();
    }

    pub fn focus_socket(&mut self, sock_id: &str, cx: &mut Context<Self>) {
        let mut already_focused = false;
        for (idx, sock) in self.sockets.iter_mut().enumerate() {
            let id = format!("{}:{}:{}", sock.protocol, sock.local_port, idx);
            if id == sock_id {
                if sock.is_focused && self.socket_drawer_open {
                    already_focused = true;
                    sock.is_focused = false;
                } else {
                    sock.is_focused = true;
                }
            } else {
                sock.is_focused = false;
            }
        }
        if already_focused {
            self.socket_drawer_open = false;
        } else {
            self.socket_drawer_open = true;
        }
        cx.notify();
    }

    pub fn toggle_socket_drawer(&mut self, cx: &mut Context<Self>) {
        self.socket_drawer_open = !self.socket_drawer_open;
        cx.notify();
    }

    pub fn close_socket_drawer(&mut self, cx: &mut Context<Self>) {
        self.socket_drawer_open = false;
        cx.notify();
    }

    pub fn toggle_socket_drawer_filter(&mut self, cx: &mut Context<Self>) {
        self.socket_drawer_filter_this_socket = !self.socket_drawer_filter_this_socket;
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
        self.sync_hba_to_config_state();
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

    // ==========================================
    // Clankers AI Hub & Usability Methods
    // ==========================================

    pub fn refresh_clankers(&mut self, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            if let Ok(providers) = db_guard.list_clanker_providers() {
                self.clanker_providers = providers;
            }
        }
        cx.notify();
    }

    pub fn open_edit_clanker_modal(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        if let Some(p) = self.clanker_providers.iter().find(|p| p.id == provider_id) {
            let key_len = p.api_key.chars().count();
            self.editing_clanker = Some(ClankerEditModalState {
                provider_id: p.id.clone(),
                display_name: p.display_name.clone(),
                api_key_input: p.api_key.clone(),
                model_input: p.model.clone(),
                base_url_input: p.base_url.clone(),
                focus: ClankerModalFocus::ApiKey,
                error_message: None,
            });
            self.input_cursor = key_len;
            self.input_selection = None;
            self.cursor_blink = true;
            cx.notify();
        }
    }

    pub fn close_edit_clanker_modal(&mut self, cx: &mut Context<Self>) {
        self.editing_clanker = None;
        cx.notify();
    }

    pub fn submit_edit_clanker(&mut self, cx: &mut Context<Self>) {
        if let Some(ref state) = self.editing_clanker.clone() {
            let p_id = state.provider_id.clone();
            let key = state.api_key_input.trim().to_string();
            let model = state.model_input.trim().to_string();
            let base_url = state.base_url_input.trim().to_string();

            if model.is_empty() {
                if let Some(ref mut st) = self.editing_clanker {
                    st.error_message = Some("Model cannot be empty".to_string());
                }
                cx.notify();
                return;
            }

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                if let Ok(Some(mut provider)) = db_guard.get_clanker_provider(&p_id) {
                    provider.api_key = key;
                    provider.model = model;
                    if !base_url.is_empty() {
                        provider.base_url = base_url;
                    }
                    let _ = db_guard.upsert_clanker_provider(&provider);
                }
            }

            self.editing_clanker = None;
            self.copy_text_with_toast("", &format!("Config updated for provider"), cx);
            self.refresh_clankers(cx);
        }
    }

    pub fn set_default_clanker(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.set_default_clanker_provider(provider_id);
        }
        self.copy_text_with_toast("", &format!("Default Clanker set to {}", provider_id), cx);
        self.refresh_clankers(cx);
    }

    pub fn reset_clanker_stats(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.reset_clanker_usage(provider_id);
        }
        self.copy_text_with_toast("", "Provider call stats reset", cx);
        self.refresh_clankers(cx);
    }

    pub fn simulate_clanker_call(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.record_clanker_usage(provider_id);
        }
        self.copy_text_with_toast("", &format!("Test call simulated (+1 call)"), cx);
        self.refresh_clankers(cx);
    }

    pub fn run_clanker_eli5(&mut self, cx: &mut Context<Self>) {
        let default_prov = self.clanker_providers.iter()
            .find(|p| p.is_default)
            .cloned()
            .unwrap_or_else(|| {
                self.clanker_providers.first().cloned().unwrap_or(ClankerProviderConfig {
                    id: "openai".into(),
                    display_name: "OpenAI".into(),
                    api_key: "".into(),
                    model: "gpt-4o-mini".into(),
                    base_url: "https://api.openai.com/v1".into(),
                    is_default: true,
                    total_calls: 0,
                    calls_30d: 0,
                    last_used_at: None,
                    daily_history: vec![],
                })
            });

        // Record call to default provider
        let prov_id = default_prov.id.clone();
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.record_clanker_usage(&prov_id);
        }
        self.refresh_clankers(cx);

        let query = self.clanker_demo_log.to_lowercase();
        let response = if query.contains("out of memory") || query.contains("oom") || query.contains("sacrifice child") {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** The server completely ran out of available RAM and swap. The Linux kernel's emergency survival reflex (\"OOM Killer\") triggered to prevent the entire host from locking up.\n\n\
                **The victim:** The kernel targeted process `mysqld` (PID 28419) because it had the highest memory badness score (`812`) and immediately sent `SIGKILL` (`-9`).\n\n\
                **What you should do next:**\n\
                1. Check memory consumption: `free -h` or `vmstat -s -S M`\n\
                2. If running MySQL, tune `innodb_buffer_pool_size` down to ~50% of total host RAM.\n\
                3. Add or increase swap space: `fallocate -l 4G /swapfile && mkswap /swapfile && swapon /swapfile`.",
                default_prov.display_name, default_prov.model
            )
        } else if query.contains("segfault") || query.contains("segmentation fault") {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** A program attempted to read or write memory that wasn't assigned to it (a null pointer or buffer overflow), so the CPU halted the program immediately.\n\n\
                **What you should do next:**\n\
                1. Inspect the stack trace: `coredumpctl info`\n\
                2. Restart the crashed daemon or check for updated package releases.",
                default_prov.display_name, default_prov.model
            )
        } else if query.contains("failed to start") || query.contains("exit-code") {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** A systemd service crashed on startup or returned a non-zero exit status during its initialization phase.\n\n\
                **What you should do next:**\n\
                1. Check exact logs for that unit: `journalctl -u <unit> -n 50 --no-pager`\n\
                2. Test manual config validity before restarting: e.g. `nginx -t` or `sshd -t`.",
                default_prov.display_name, default_prov.model
            )
        } else {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** The system logged an operational notification or warning event. Everything is still functioning, but an underlying component is reporting non-standard behavior.\n\n\
                **Suggested action:** Monitor logs for recurring occurrences or check journal filtering for PID details.",
                default_prov.display_name, default_prov.model
            )
        };

        self.clanker_demo_output = Some(response);
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

                if let Some(ref mut clk) = this.editing_clanker {
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
                if this.screen == Screen::Server && this.active_view == "logs" && this.journal_search_focused {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.journal_search_focused = false;
                        cx.notify();
                        return;
                    } else if key == "enter" {
                        this.run_journal_query(cx);
                        return;
                    } else {
                        let changed = crate::components::handle_text_key_event(
                            &mut this.journal_search,
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
                if this.screen == Screen::Server && (this.active_view == "config" || this.active_view == "configure") && this.config_search_focused {
                    this.cursor_blink = true;
                    if ev.keystroke.key == "escape" {
                        this.config_search_focused = false;
                        this.config_search_query.clear();
                        cx.notify();
                        return;
                    } else {
                        let changed = crate::components::handle_text_key_event(
                            &mut this.config_search_query,
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
                                                                    if self.active_services_tab == "sockets" {
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
                                                                            .children(if self.socket_drawer_open {
                                                                                Some(socket_log_drawer(self, app_view.clone()).into_any_element())
                                                                            } else {
                                                                                None
                                                                            })
                                                                    } else {
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .child(services_table(self, app_view.clone()))
                                                                            .child(if self.active_services_tab == "services" {
                                                                                service_inspector_rail(self, app_view.clone()).into_any_element()
                                                                            } else {
                                                                                log_tail(&self.journal_entries, app_view.clone()).into_any_element()
                                                                            })
                                                                    }
                                                                )
                                                            } else if is_config {
                                                                let editor_view = if self.selected_managed_file == "journald.conf" {
                                                                    crate::views::config::journald_editor::journald_editor(
                                                                        &self.journal_retention,
                                                                        &self.journal_telemetry,
                                                                        self,
                                                                        app_view.clone(),
                                                                    ).into_any_element()
                                                                } else if self.selected_managed_file == "pg_hba.conf" {
                                                                    rules_editor(&self.hba_rules, self, app_view.clone()).into_any_element()
                                                                } else if self.selected_managed_file == "crontab" || self.selected_managed_file.contains("cron") {
                                                                    cron_editor(&self.cron_jobs, self, app_view.clone()).into_any_element()
                                                                } else if let Some(st) = self.config_file_states.get(&self.selected_managed_file) {
                                                                    raw_config_editor(st, app_view.clone()).into_any_element()
                                                                } else {
                                                                    rules_editor(&self.hba_rules, self, app_view.clone()).into_any_element()
                                                                };

                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(managed_files_rail(&self.selected_managed_file, self, app_view.clone()))
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
                                                                        .child(cron_editor(&self.cron_jobs, self, app_view.clone()))
                                                                        .child(pending_diff_rail(self, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "users" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(user_management_view(app_view.clone(), &self.users, &self.enrolled_keys))
                                                                )
                                                            } else if self.active_view == "files" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(file_browser_view(app_view.clone(), self))
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
                                                                                .child(firewall_view(app_view.clone(), &self.firewall, &self.config_file_states))
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
                                            .child(danger_zone(self, app_view.clone())),
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
