//! The server's Dokku PaaS page (`ERR-203`..`ERR-210`): single-round-trip
//! scanner, server bootstrap & plugin catalog, deploy key bridge, app
//! lifecycle & process scaling, backing services & storage mounts, staged
//! base64-encoded secrets manager, domains/ports/Let's Encrypt TLS, PTY
//! `dokku enter` console, and cross-surface Checks/Containers/Topology links.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::host_actions::{describe_commands, HostCommand};
use super::CrowApp;
use crate::dokku::{
    self, BackingService, DokkuApp, DokkuScan, ServiceKind,
};
use crate::host::host_for;
use crate::terminal::layout::Node;
use crate::vault::{ChangeRecord, ServerRecord};
use crate::views::overview::state::ServiceEli5;

/// How often an open Dokku page reads the server again.
const RESCAN_SECS: i64 = 15;
/// How often followed app/service logs are read again.
const FOLLOW_SECS: i64 = 3;
/// Dokku operations (plugin install, service creation, image deploy, Let's Encrypt)
/// can pull images or run apt/debconf; give them up to 10 minutes.
const DOKKU_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DokkuSelection {
    App(String),
    Service(ServiceKind, String),
}

impl DokkuSelection {
    pub fn key(&self) -> String {
        match self {
            Self::App(name) => format!("app:{name}"),
            Self::Service(kind, name) => format!("svc:{}:{name}", kind.id()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DokkuDrawerTab {
    #[default]
    Overview,
    Env,
    Domains,
    Services,
    Deploy,
    Logs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DokkuTopPanel {
    GlobalSetup,
    Plugins,
    DeployKeys,
    NewApp,
    NewService,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppLifecycleAction {
    Start,
    Stop,
    Restart,
    Rebuild,
    MaintenanceToggle(bool),
    Destroy,
}

impl AppLifecycleAction {
    pub fn subcommand(self) -> &'static str {
        match self {
            Self::Start => "ps:start",
            Self::Stop => "ps:stop",
            Self::Restart => "ps:restart",
            Self::Rebuild => "ps:rebuild",
            Self::MaintenanceToggle(true) => "maintenance:enable",
            Self::MaintenanceToggle(false) => "maintenance:disable",
            Self::Destroy => "apps:destroy",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceLifecycleAction {
    Start,
    Stop,
    Restart,
    Export,
    Destroy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingDokkuAction {
    BootstrapDokku,
    App { app: String, action: AppLifecycleAction },
    Service { kind: ServiceKind, name: String, action: ServiceLifecycleAction },
    Link { kind: ServiceKind, service: String, app: String, link: bool },
}

pub struct DokkuInputs {
    pub new_app_name: Entity<InputState>,
    pub new_service_name: Entity<InputState>,
    pub global_domain: Entity<InputState>,
    pub letsencrypt_email: Entity<InputState>,
    pub env_key: Entity<InputState>,
    pub env_val: Entity<InputState>,
    pub dotenv_blob: Entity<InputState>,
    pub domain_add: Entity<InputState>,
    pub port_add: Entity<InputState>,
    pub mount_host: Entity<InputState>,
    pub mount_container: Entity<InputState>,
    pub deploy_image: Entity<InputState>,
}

pub struct DokkuState {
    pub server_id: Option<String>,
    pub scan: Option<Result<DokkuScan, String>>,
    pub loading: bool,
    pub scanned_at: i64,
    /// Last scan per server id, for Fleet Topology & Containers correlation.
    pub by_server: HashMap<String, DokkuScan>,
    pub selected: Option<DokkuSelection>,
    pub tab: DokkuDrawerTab,
    pub top_panel: Option<DokkuTopPanel>,
    pub new_service_kind: ServiceKind,
    /// Staged Procfile scale changes per app: app -> (proc_type -> count).
    pub staged_scale: BTreeMap<String, BTreeMap<String, u32>>,
    /// On-demand config:export per app (secrets never fetched during background scans).
    pub env_cache: HashMap<String, Result<BTreeMap<String, String>, String>>,
    pub env_loading: HashSet<String>,
    /// Revealed keys (`"app:KEY"`).
    pub env_revealed: HashSet<String>,
    /// Staged env additions/edits per app.
    pub staged_env_set: BTreeMap<String, BTreeMap<String, String>>,
    /// Staged env removals per app.
    pub staged_env_unset: HashMap<String, HashSet<String>>,
    /// Whether applying staged env changes restarts the app (`true` by default).
    pub restart_on_env_apply: bool,
    /// Whether the `.env` bulk paste box is open in the Env tab.
    pub dotenv_paste_open: bool,
    /// Whether service link/unlink restarts the app (`true` by default).
    pub restart_on_link: bool,
    pub pending: Option<PendingDokkuAction>,
    pub busy: Option<String>,
    pub result: Option<(bool, String)>,
    /// (selection key, last log lines).
    pub logs: Option<(String, Result<String, String>)>,
    pub logs_loading: bool,
    pub logs_at: i64,
    pub follow: bool,
    pub log_process_filter: Option<String>,
    pub eli5: HashMap<String, Result<ServiceEli5, String>>,
    pub eli5_loading: HashSet<String>,
    pub eli5_open: bool,
}

impl Default for DokkuState {
    fn default() -> Self {
        Self {
            server_id: None,
            scan: None,
            loading: false,
            scanned_at: 0,
            by_server: HashMap::new(),
            selected: None,
            tab: DokkuDrawerTab::Overview,
            top_panel: None,
            new_service_kind: ServiceKind::Postgres,
            staged_scale: BTreeMap::new(),
            env_cache: HashMap::new(),
            env_loading: HashSet::new(),
            env_revealed: HashSet::new(),
            staged_env_set: BTreeMap::new(),
            staged_env_unset: HashMap::new(),
            restart_on_env_apply: true,
            dotenv_paste_open: false,
            restart_on_link: true,
            pending: None,
            busy: None,
            result: None,
            logs: None,
            logs_loading: false,
            logs_at: 0,
            follow: true,
            log_process_filter: None,
            eli5: HashMap::new(),
            eli5_loading: HashSet::new(),
            eli5_open: false,
        }
    }
}

impl DokkuState {
    pub fn current(&self) -> Option<&DokkuScan> {
        self.scan.as_ref().and_then(|s| s.as_ref().ok())
    }

    pub fn selected_app<'a>(&self, scan: &'a DokkuScan) -> Option<&'a DokkuApp> {
        match &self.selected {
            Some(DokkuSelection::App(name)) => scan.find_app(name),
            _ => None,
        }
    }

    pub fn selected_service<'a>(&self, scan: &'a DokkuScan) -> Option<&'a BackingService> {
        match &self.selected {
            Some(DokkuSelection::Service(kind, name)) => scan.find_service(*kind, name),
            _ => None,
        }
    }
}

fn run_dokku_commands(server: &ServerRecord, commands: &[HostCommand]) -> Result<(), String> {
    let host = host_for(server);
    for cmd in commands {
        let argv: Vec<&str> = cmd.argv.iter().map(String::as_str).collect();
        let ran = if cmd.as_user {
            host.exec_stdin(&argv, &cmd.stdin, DOKKU_TIMEOUT)
        } else {
            host.exec_privileged(&argv, &cmd.stdin, DOKKU_TIMEOUT)
        };
        ran.map_err(|e| match e {
            crate::host::HostError::Failed { stderr, .. } if !stderr.trim().is_empty() => {
                stderr
                    .lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            }
            other => other.to_string(),
        })?;
    }
    Ok(())
}

impl CrowApp {
    pub fn ensure_dokku_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dokku_inputs.is_some() {
            return;
        }
        self.dokku_inputs = Some(DokkuInputs {
            new_app_name: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-app-name"))),
            new_service_name: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-service-name"))),
            global_domain: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-global-domain"))),
            letsencrypt_email: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-le-email"))),
            env_key: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-env-key"))),
            env_val: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-env-val"))),
            dotenv_blob: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-dotenv"))),
            domain_add: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-domain"))),
            port_add: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-port"))),
            mount_host: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-mount-host"))),
            mount_container: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-mount-container"))),
            deploy_image: cx.new(|cx| InputState::new(window, cx).placeholder(t!("dokku-placeholder-deploy-image"))),
        });
    }

    /// Reads the active server's Dokku state in one SSH round trip.
    pub fn refresh_dokku(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        if self.dokku.server_id.as_deref() != Some(srv.id.as_str()) {
            let by_server = std::mem::take(&mut self.dokku.by_server);
            self.dokku = DokkuState {
                server_id: Some(srv.id.clone()),
                by_server,
                ..Default::default()
            };
        }
        if self.dokku.loading {
            return;
        }
        self.dokku.loading = true;
        self.dokku.scanned_at = chrono::Utc::now().timestamp();
        cx.notify();
        let server_id = srv.id.clone();
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { dokku::scan(host_for(&srv).as_ref()) })
                .await;
            let _ = entity.update(cx, |this, cx| {
                if this.dokku.server_id.as_deref() != Some(server_id.as_str()) {
                    return;
                }
                this.dokku.loading = false;
                if let Ok(scan) = &result {
                    this.dokku.by_server.insert(server_id, scan.clone());
                    // Clear selection if its target was destroyed.
                    match &this.dokku.selected {
                        Some(DokkuSelection::App(name)) if scan.find_app(name).is_none() => {
                            this.dokku.selected = None;
                        }
                        Some(DokkuSelection::Service(k, name)) if scan.find_service(*k, name).is_none() => {
                            this.dokku.selected = None;
                        }
                        _ => {}
                    }
                }
                this.dokku.scan = Some(result);
                cx.notify();
            });
        })
        .detach();
    }

    /// Every poll: an open Dokku page stays current, and followed logs keep coming.
    pub fn dokku_tick(&mut self, cx: &mut Context<Self>) {
        if self.screen != super::Screen::Server || self.active_view != "dokku" {
            return;
        }
        let now = chrono::Utc::now().timestamp();
        if now - self.dokku.scanned_at >= RESCAN_SECS && self.dokku.busy.is_none() {
            self.refresh_dokku(cx);
        }
        let watching_logs = match &self.dokku.selected {
            Some(DokkuSelection::App(_)) => self.dokku.tab == DokkuDrawerTab::Logs,
            Some(DokkuSelection::Service(_, _)) => true,
            None => false,
        };
        if self.dokku.follow && watching_logs && now - self.dokku.logs_at >= FOLLOW_SECS {
            self.load_dokku_logs(cx);
        }
    }

    pub fn toggle_dokku_top_panel(&mut self, panel: DokkuTopPanel, window: &mut Window, cx: &mut Context<Self>) {
        self.dokku.top_panel = if self.dokku.top_panel == Some(panel) { None } else { Some(panel) };
        if self.dokku.top_panel == Some(DokkuTopPanel::GlobalSetup) {
            if let (Some(scan), Some(inputs)) = (self.dokku.current().cloned(), self.dokku_inputs.as_ref()) {
                let global_domain = inputs.global_domain.clone();
                let le_email = inputs.letsencrypt_email.clone();
                if let Some(dom) = scan.global_domains.first().cloned() {
                    global_domain.update(cx, |s, cx| s.set_value(dom, window, cx));
                }
                if !scan.letsencrypt_email.is_empty() {
                    let email = scan.letsencrypt_email;
                    le_email.update(cx, |s, cx| s.set_value(email, window, cx));
                }
            }
        }
        cx.notify();
    }

    pub fn select_dokku_app(&mut self, app: &str, cx: &mut Context<Self>) {
        let next = DokkuSelection::App(app.to_string());
        if self.dokku.selected.as_ref() == Some(&next) {
            self.dokku.selected = None;
            cx.notify();
            return;
        }
        self.dokku.selected = Some(next);
        self.dokku.pending = None;
        self.dokku.eli5_open = false;
        self.dokku.logs = None;
        self.dokku.log_process_filter = None;
        if self.dokku.tab == DokkuDrawerTab::Env {
            self.load_dokku_app_env(app, false, cx);
        } else if self.dokku.tab == DokkuDrawerTab::Logs {
            self.load_dokku_logs(cx);
        }
        cx.notify();
    }

    pub fn select_dokku_service(&mut self, kind: ServiceKind, name: &str, cx: &mut Context<Self>) {
        let next = DokkuSelection::Service(kind, name.to_string());
        if self.dokku.selected.as_ref() == Some(&next) {
            self.dokku.selected = None;
            cx.notify();
            return;
        }
        self.dokku.selected = Some(next);
        self.dokku.pending = None;
        self.dokku.eli5_open = false;
        self.dokku.logs = None;
        self.load_dokku_logs(cx);
        cx.notify();
    }

    pub fn close_dokku_drawer(&mut self, cx: &mut Context<Self>) {
        self.dokku.selected = None;
        self.dokku.pending = None;
        cx.notify();
    }

    pub fn set_dokku_tab(&mut self, tab: DokkuDrawerTab, cx: &mut Context<Self>) {
        self.dokku.tab = tab;
        if let Some(DokkuSelection::App(app)) = self.dokku.selected.clone() {
            match tab {
                DokkuDrawerTab::Env if !self.dokku.env_cache.contains_key(&app) => {
                    self.load_dokku_app_env(&app, false, cx);
                }
                DokkuDrawerTab::Logs if self.dokku.logs.is_none() => {
                    self.load_dokku_logs(cx);
                }
                _ => {}
            }
        }
        cx.notify();
    }

    /// Jumps from Containers page directly to the owning Dokku app or backing service (`ERR-210`).
    pub fn open_in_dokku(&mut self, app_or_service: &str, cx: &mut Context<Self>) {
        self.set_view("dokku", cx);
        if let Some((kind_str, svc_name)) = app_or_service.strip_prefix("dokku.").and_then(|r| r.split_once('.')) {
            if let Some(kind) = ServiceKind::from_id(kind_str) {
                self.dokku.selected = Some(DokkuSelection::Service(kind, svc_name.to_string()));
                self.dokku.logs = None;
                self.load_dokku_logs(cx);
                cx.notify();
                return;
            }
        }
        self.dokku.selected = Some(DokkuSelection::App(app_or_service.to_string()));
        self.dokku.tab = DokkuDrawerTab::Overview;
        cx.notify();
    }

    /// Executes privileged Dokku commands with a 10-minute timeout, durable
    /// `ChangeRecord` audit logging (with optional secret-scrubbed summary),
    /// and an automatic rescan upon completion.
    fn execute_dokku_action(
        &mut self,
        kind: &str,
        target: &str,
        commands: Vec<HostCommand>,
        audit_summary: Option<String>,
        after: impl FnOnce(&mut CrowApp, &Result<(), String>, &mut Context<CrowApp>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(srv) = self.fleet.active_server() else { return };
        if !self.allowed(crate::team::Permission::Users, Some(&srv), cx) {
            return;
        }
        let summary = audit_summary.unwrap_or_else(|| describe_commands(&commands));
        self.dokku.busy = Some(summary.clone());
        self.dokku.result = None;
        self.dokku.pending = None;
        cx.notify();

        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.insert_change_record(&ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: kind.to_string(),
                target: target.to_string(),
                before_state: summary.clone(),
                after_state: None,
                blast_radius: None,
                outcome: "pending".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }

        let server_id = srv.id.clone();
        cx.spawn(async move |entity, cx| {
            let (result, rescan) = cx
                .background_executor()
                .spawn(async move {
                    let res = run_dokku_commands(&srv, &commands);
                    let scan = dokku::scan(host_for(&srv).as_ref());
                    (res, scan)
                })
                .await;
            let outcome = if result.is_ok() { "success" } else { "failed" };
            if let Ok(db_guard) = db.lock() {
                let err_text = result.as_ref().err().cloned();
                let _ = db_guard.update_change_record_outcome(
                    &record_id,
                    outcome,
                    err_text.as_deref(),
                    &chrono::Utc::now().to_rfc3339(),
                );
            }
            let _ = entity.update(cx, |this, cx| {
                this.push_journal_action_marker(format!("crow: {summary} ({outcome})"));
                this.dokku.busy = None;
                this.dokku.result = Some(match &result {
                    Ok(()) => (true, t!("app-dokku-done", summary = summary)),
                    Err(e) => (false, t!("app-dokku-failed", summary = summary, e = e)),
                });
                if let Ok(scan) = &rescan {
                    this.dokku.by_server.insert(server_id.clone(), scan.clone());
                }
                if this.dokku.server_id.as_deref() == Some(server_id.as_str()) {
                    this.dokku.scan = Some(rescan);
                    this.dokku.scanned_at = chrono::Utc::now().timestamp();
                }
                after(this, &result, cx);
                cx.notify();
            });
        })
        .detach();
    }

    // =========================================================================
    // Confirmations (Bootstrap, App Lifecycle, Service Lifecycle, Link/Unlink)
    // =========================================================================

    pub fn request_dokku_action(&mut self, action: PendingDokkuAction, cx: &mut Context<Self>) {
        self.dokku.pending = if self.dokku.pending.as_ref() == Some(&action) {
            None
        } else {
            Some(action)
        };
        self.dokku.result = None;
        cx.notify();
    }

    pub fn cancel_dokku_pending(&mut self, cx: &mut Context<Self>) {
        self.dokku.pending = None;
        cx.notify();
    }

    pub fn confirm_dokku_pending(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.dokku.pending.take() else { return };
        match pending {
            PendingDokkuAction::BootstrapDokku => {
                let script = concat!(
                    "set -e; ",
                    "export DEBIAN_FRONTEND=noninteractive; ",
                    "if ! command -v docker >/dev/null 2>&1; then curl -fsSL https://get.docker.com | sh; fi; ",
                    "wget -NP . https://dokku.com/install/v0.35.15/bootstrap.sh; ",
                    "echo 'dokku dokku/vhost_enable boolean true' | debconf-set-selections; ",
                    "echo 'dokku dokku/web_config boolean false' | debconf-set-selections; ",
                    "DOKKU_TAG=v0.35.15 bash bootstrap.sh; ",
                    "rm -f bootstrap.sh"
                );
                let cmd = HostCommand::new(vec!["sh".into(), "-c".into(), script.into()]);
                let summary = "bootstrap Dokku v0.35.15 + Docker".to_string();
                self.execute_dokku_action("dokku.bootstrap", "dokku", vec![cmd], Some(summary), |_, _, _| {}, cx);
            }
            PendingDokkuAction::App { app, action } => {
                let mut argv = vec!["dokku".to_string()];
                if action == AppLifecycleAction::Destroy {
                    argv.push("--force".to_string());
                }
                argv.push(action.subcommand().to_string());
                argv.push(app.clone());
                let kind = format!("dokku.{}", action.subcommand().replace(':', "."));
                let destroyed = action == AppLifecycleAction::Destroy;
                let app_clone = app.clone();
                self.execute_dokku_action(
                    &kind,
                    &app,
                    vec![HostCommand::new(argv)],
                    None,
                    move |this, res, _cx| {
                        if destroyed && res.is_ok() && this.dokku.selected == Some(DokkuSelection::App(app_clone)) {
                            this.dokku.selected = None;
                        }
                    },
                    cx,
                );
            }
            PendingDokkuAction::Service { kind, name, action } => {
                match action {
                    ServiceLifecycleAction::Export => {
                        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                        let dump_path = format!("/var/backups/crow-{}-{name}-{ts}.dump", kind.id());
                        let script = format!(
                            "mkdir -p /var/backups && dokku {}:export '{}' > '{}'",
                            kind.id(),
                            name,
                            dump_path
                        );
                        let cmd = HostCommand::new(vec!["sh".into(), "-c".into(), script]);
                        let summary = format!("dokku {}:export {name} > {dump_path}", kind.id());
                        self.execute_dokku_action(
                            "dokku.service.export",
                            &name,
                            vec![cmd],
                            Some(summary),
                            |_, _, _| {},
                            cx,
                        );
                    }
                    other => {
                        let verb = match other {
                            ServiceLifecycleAction::Start => "start",
                            ServiceLifecycleAction::Stop => "stop",
                            ServiceLifecycleAction::Restart => "restart",
                            ServiceLifecycleAction::Destroy => "destroy",
                            ServiceLifecycleAction::Export => unreachable!(),
                        };
                        let mut argv = vec!["dokku".to_string()];
                        if other == ServiceLifecycleAction::Destroy {
                            argv.push("--force".to_string());
                        }
                        argv.push(format!("{}:{verb}", kind.id()));
                        argv.push(name.clone());
                        let destroyed = other == ServiceLifecycleAction::Destroy;
                        let sel = DokkuSelection::Service(kind, name.clone());
                        self.execute_dokku_action(
                            &format!("dokku.service.{verb}"),
                            &name,
                            vec![HostCommand::new(argv)],
                            None,
                            move |this, res, _cx| {
                                if destroyed && res.is_ok() && this.dokku.selected == Some(sel) {
                                    this.dokku.selected = None;
                                }
                            },
                            cx,
                        );
                    }
                }
            }
            PendingDokkuAction::Link { kind, service, app, link } => {
                let verb = if link { "link" } else { "unlink" };
                let mut argv = vec!["dokku".to_string(), format!("{}:{verb}", kind.id())];
                if !self.dokku.restart_on_link {
                    argv.push("--no-restart".to_string());
                }
                argv.push(service.clone());
                argv.push(app.clone());
                let app_clone = app.clone();
                self.execute_dokku_action(
                    &format!("dokku.service.{verb}"),
                    &format!("{service}->{app}"),
                    vec![HostCommand::new(argv)],
                    None,
                    move |this, res, cx| {
                        if res.is_ok() && this.dokku.env_cache.contains_key(&app_clone) {
                            this.load_dokku_app_env(&app_clone, true, cx);
                        }
                    },
                    cx,
                );
            }
        }
    }

    // =========================================================================
    // Stage 1: Global Setup, Official Plugins & Deploy Keys (`ERR-205`)
    // =========================================================================

    pub fn save_dokku_global_domain(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let dom = inputs.global_domain.read(cx).value().trim().to_string();
        if let Err(e) = dokku::validate_domain(&dom) {
            self.dokku.result = Some((false, e));
            cx.notify();
            return;
        }
        let cmd = HostCommand::new(vec!["dokku".into(), "domains:set-global".into(), dom.clone()]);
        self.execute_dokku_action("dokku.domains.set_global", &dom, vec![cmd], None, |_, _, _| {}, cx);
    }

    pub fn save_dokku_letsencrypt_global(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let email = inputs.letsencrypt_email.read(cx).value().trim().to_string();
        if !email.contains('@') || email.contains(' ') {
            self.dokku.result = Some((false, t!("app-dokku-invalid-email").into()));
            cx.notify();
            return;
        }
        let mut cmds = Vec::new();
        let has_le = self
            .dokku
            .current()
            .is_some_and(|s| s.has_plugin("letsencrypt"));
        if !has_le {
            cmds.push(HostCommand::new(vec![
                "dokku".into(),
                "plugin:install".into(),
                "https://github.com/dokku/dokku-letsencrypt.git".into(),
            ]));
        }
        cmds.push(HostCommand::new(vec![
            "dokku".into(),
            "letsencrypt:set".into(),
            "--global".into(),
            "email".into(),
            email.clone(),
        ]));
        cmds.push(HostCommand::new(vec![
            "dokku".into(),
            "letsencrypt:cron-job".into(),
            "--add".into(),
        ]));
        self.execute_dokku_action("dokku.letsencrypt.global", &email, cmds, None, |_, _, _| {}, cx);
    }

    pub fn install_dokku_plugin(&mut self, id: &str, url: &str, cx: &mut Context<Self>) {
        let cmd = HostCommand::new(vec![
            "dokku".into(),
            "plugin:install".into(),
            url.to_string(),
            "--name".into(),
            id.to_string(),
        ]);
        self.execute_dokku_action("dokku.plugin.install", id, vec![cmd], None, |_, _, _| {}, cx);
    }

    pub fn authorize_dokku_ssh_key(&mut self, key_name: &str, public_key: &str, cx: &mut Context<Self>) {
        let clean_name: String = key_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect();
        let name = if clean_name.is_empty() { "crow-key".to_string() } else { clean_name };
        let mut cmd = HostCommand::new(vec!["dokku".into(), "ssh-keys:add".into(), name.clone()]);
        let mut stdin = public_key.trim().as_bytes().to_vec();
        stdin.push(b'\n');
        cmd.stdin = stdin;
        self.execute_dokku_action("dokku.ssh_keys.add", &name, vec![cmd], None, |_, _, _| {}, cx);
    }

    pub fn remove_dokku_ssh_key(&mut self, name: &str, cx: &mut Context<Self>) {
        let cmd = HostCommand::new(vec!["dokku".into(), "ssh-keys:remove".into(), name.to_string()]);
        self.execute_dokku_action("dokku.ssh_keys.remove", name, vec![cmd], None, |_, _, _| {}, cx);
    }

    // =========================================================================
    // Stage 2: App Creation, Process Scaling, Deploy & Console (`ERR-206`)
    // =========================================================================

    pub fn create_dokku_app(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let name = inputs.new_app_name.read(cx).value().trim().to_string();
        if let Err(e) = dokku::validate_name(&name) {
            self.dokku.result = Some((false, e));
            cx.notify();
            return;
        }
        let cmd = HostCommand::new(vec!["dokku".into(), "apps:create".into(), name.clone()]);
        let app_name = name.clone();
        self.execute_dokku_action(
            "dokku.apps.create",
            &name,
            vec![cmd],
            None,
            move |this, res, _cx| {
                if res.is_ok() {
                    this.dokku_inputs = None;
                    this.dokku.top_panel = None;
                    this.dokku.selected = Some(DokkuSelection::App(app_name));
                    this.dokku.tab = DokkuDrawerTab::Deploy;
                }
            },
            cx,
        );
    }

    pub fn adjust_dokku_scale(&mut self, app: &str, proc_type: &str, current: u32, delta: i32, cx: &mut Context<Self>) {
        let app_map = self.dokku.staged_scale.entry(app.to_string()).or_default();
        let base = app_map.get(proc_type).copied().unwrap_or(current);
        let next = (base as i32 + delta).clamp(0, 64) as u32;
        if next == current {
            app_map.remove(proc_type);
        } else {
            app_map.insert(proc_type.to_string(), next);
        }
        if app_map.is_empty() {
            self.dokku.staged_scale.remove(app);
        }
        cx.notify();
    }

    pub fn clear_dokku_staged_scale(&mut self, app: &str, cx: &mut Context<Self>) {
        self.dokku.staged_scale.remove(app);
        cx.notify();
    }

    pub fn apply_dokku_scale(&mut self, app: &str, cx: &mut Context<Self>) {
        let Some(staged) = self.dokku.staged_scale.remove(app) else { return };
        if staged.is_empty() {
            return;
        }
        let mut argv = vec!["dokku".to_string(), "ps:scale".to_string(), app.to_string()];
        for (proc_type, count) in &staged {
            argv.push(format!("{proc_type}={count}"));
        }
        self.execute_dokku_action("dokku.ps.scale", app, vec![HostCommand::new(argv)], None, |_, _, _| {}, cx);
    }

    pub fn deploy_dokku_image(&mut self, app: &str, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let image = inputs.deploy_image.read(cx).value().trim().to_string();
        if image.is_empty() || image.contains(' ') {
            self.dokku.result = Some((false, t!("app-dokku-enter-valid-image").into()));
            cx.notify();
            return;
        }
        let cmd = HostCommand::new(vec![
            "dokku".into(),
            "git:from-image".into(),
            app.to_string(),
            image,
        ]);
        self.execute_dokku_action(
            "dokku.git.from_image",
            app,
            vec![cmd],
            None,
            move |this, res, _cx| {
                if res.is_ok() {
                    this.dokku_inputs = None;
                }
            },
            cx,
        );
    }

    /// Opens an interactive PTY terminal tab running `dokku enter <app> <proc_type>` (`ERR-206`).
    pub fn open_dokku_console(&mut self, app: &str, proc_type: &str, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let cmd_line = format!("sudo -n dokku enter {app} {proc_type} || dokku enter {app} {proc_type}\r");
        let Some(pane_id) = self.terminal_detached_pane(cmd_line.into_bytes(), cx) else { return };
        if let Some(ws) = self.terminals.get_mut(&srv.id) {
            ws.tabs.push(super::terminal::TerminalTab {
                root: Node::Pane(pane_id),
                focused: pane_id,
                name: Some(format!("dokku:{app}")),
            });
            ws.active = ws.tabs.len() - 1;
        }
        self.set_view("terminal", cx);
    }

    // =========================================================================
    // Stage 3: Backing Services & Persistent Host Storage (`ERR-207`)
    // =========================================================================

    pub fn set_dokku_new_service_kind(&mut self, kind: ServiceKind, cx: &mut Context<Self>) {
        self.dokku.new_service_kind = kind;
        cx.notify();
    }

    pub fn toggle_dokku_restart_on_link(&mut self, cx: &mut Context<Self>) {
        self.dokku.restart_on_link = !self.dokku.restart_on_link;
        cx.notify();
    }

    pub fn create_dokku_service(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let name = inputs.new_service_name.read(cx).value().trim().to_string();
        if let Err(e) = dokku::validate_name(&name) {
            self.dokku.result = Some((false, e));
            cx.notify();
            return;
        }
        let kind = self.dokku.new_service_kind;
        let has_plugin = self.dokku.current().is_some_and(|s| s.has_plugin(kind.id()));
        let mut cmds = Vec::new();
        if !has_plugin {
            cmds.push(HostCommand::new(vec![
                "dokku".into(),
                "plugin:install".into(),
                kind.plugin_url().into(),
                "--name".into(),
                kind.id().into(),
            ]));
        }
        cmds.push(HostCommand::new(vec![
            "dokku".into(),
            format!("{}:create", kind.id()),
            name.clone(),
        ]));
        let svc_name = name.clone();
        self.execute_dokku_action(
            &format!("dokku.{}.create", kind.id()),
            &name,
            cmds,
            None,
            move |this, res, _cx| {
                if res.is_ok() {
                    this.dokku_inputs = None;
                    this.dokku.top_panel = None;
                    this.dokku.selected = Some(DokkuSelection::Service(kind, svc_name));
                }
            },
            cx,
        );
    }

    pub fn add_dokku_storage_mount(&mut self, app: &str, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let host_dir = inputs.mount_host.read(cx).value().trim().to_string();
        let container_dir = inputs.mount_container.read(cx).value().trim().to_string();
        if !host_dir.starts_with('/') || !container_dir.starts_with('/') || host_dir.contains(':') || container_dir.contains(':') {
            self.dokku.result = Some((false, t!("app-dokku-mount-paths-must-be-absolute").into()));
            cx.notify();
            return;
        }
        let spec = format!("{host_dir}:{container_dir}");
        let cmds = vec![
            HostCommand::new(vec!["mkdir".into(), "-p".into(), host_dir]),
            HostCommand::new(vec!["dokku".into(), "storage:mount".into(), app.to_string(), spec]),
            HostCommand::new(vec!["dokku".into(), "ps:restart".into(), app.to_string()]),
        ];
        self.execute_dokku_action(
            "dokku.storage.mount",
            app,
            cmds,
            None,
            move |this, res, _cx| {
                if res.is_ok() {
                    this.dokku_inputs = None;
                }
            },
            cx,
        );
    }

    pub fn remove_dokku_storage_mount(&mut self, app: &str, host_path: &str, container_path: &str, cx: &mut Context<Self>) {
        let spec = format!("{host_path}:{container_path}");
        let cmds = vec![
            HostCommand::new(vec!["dokku".into(), "storage:unmount".into(), app.to_string(), spec]),
            HostCommand::new(vec!["dokku".into(), "ps:restart".into(), app.to_string()]),
        ];
        self.execute_dokku_action("dokku.storage.unmount", app, cmds, None, |_, _, _| {}, cx);
    }

    // =========================================================================
    // Stage 4: Staged Environment & Secrets Manager (`ERR-208`)
    // =========================================================================

    pub fn load_dokku_app_env(&mut self, app: &str, force: bool, cx: &mut Context<Self>) {
        if !force && (self.dokku.env_cache.contains_key(app) || self.dokku.env_loading.contains(app)) {
            return;
        }
        let Some(srv) = self.fleet.active_server() else { return };
        let app_name = app.to_string();
        self.dokku.env_loading.insert(app_name.clone());
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let app_for_task = app_name.clone();
            let result = cx
                .background_executor()
                .spawn(async move { dokku::read_app_config(host_for(&srv).as_ref(), &app_for_task) })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.dokku.env_loading.remove(&app_name);
                this.dokku.env_cache.insert(app_name, result);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn toggle_dokku_env_reveal(&mut self, app: &str, key: &str, cx: &mut Context<Self>) {
        let id = format!("{app}:{key}");
        if !self.dokku.env_revealed.remove(&id) {
            self.dokku.env_revealed.insert(id);
        }
        cx.notify();
    }

    pub fn toggle_dokku_restart_on_env(&mut self, cx: &mut Context<Self>) {
        self.dokku.restart_on_env_apply = !self.dokku.restart_on_env_apply;
        cx.notify();
    }

    pub fn toggle_dokku_dotenv_paste(&mut self, cx: &mut Context<Self>) {
        self.dokku.dotenv_paste_open = !self.dokku.dotenv_paste_open;
        cx.notify();
    }

    pub fn stage_dokku_env_pair(&mut self, app: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let key = inputs.env_key.read(cx).value().trim().to_string();
        let val = inputs.env_val.read(cx).value().to_string();
        if let Err(e) = dokku::validate_env_key(&key) {
            self.dokku.result = Some((false, e));
            cx.notify();
            return;
        }
        if let Some(unset) = self.dokku.staged_env_unset.get_mut(app) {
            unset.remove(&key);
        }
        self.dokku
            .staged_env_set
            .entry(app.to_string())
            .or_default()
            .insert(key, val);
        let k_in = inputs.env_key.clone();
        let v_in = inputs.env_val.clone();
        k_in.update(cx, |s, cx| s.set_value("", window, cx));
        v_in.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }

    pub fn stage_dokku_dotenv_blob(&mut self, app: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let blob = inputs.dotenv_blob.read(cx).value().to_string();
        let parsed = match dokku::parse_dotenv(&blob) {
            Ok(m) if !m.is_empty() => m,
            Ok(_) => {
                self.dokku.result = Some((false, t!("app-dokku-no-valid-key-value-lines").into()));
                cx.notify();
                return;
            }
            Err(e) => {
                self.dokku.result = Some((false, e));
                cx.notify();
                return;
            }
        };
        let staged = self.dokku.staged_env_set.entry(app.to_string()).or_default();
        for (k, v) in parsed {
            if let Some(unset) = self.dokku.staged_env_unset.get_mut(app) {
                unset.remove(&k);
            }
            staged.insert(k, v);
        }
        let b_in = inputs.dotenv_blob.clone();
        b_in.update(cx, |s, cx| s.set_value("", window, cx));
        self.dokku.dotenv_paste_open = false;
        cx.notify();
    }

    pub fn stage_dokku_env_unset(&mut self, app: &str, key: &str, cx: &mut Context<Self>) {
        if let Some(set) = self.dokku.staged_env_set.get_mut(app) {
            if set.remove(key).is_some() {
                if set.is_empty() {
                    self.dokku.staged_env_set.remove(app);
                }
                cx.notify();
                return;
            }
        }
        let unset = self.dokku.staged_env_unset.entry(app.to_string()).or_default();
        if !unset.remove(key) {
            unset.insert(key.to_string());
        }
        cx.notify();
    }

    pub fn clear_dokku_staged_env(&mut self, app: &str, cx: &mut Context<Self>) {
        self.dokku.staged_env_set.remove(app);
        self.dokku.staged_env_unset.remove(app);
        cx.notify();
    }

    pub fn apply_dokku_staged_env(&mut self, app: &str, cx: &mut Context<Self>) {
        let set_map = self.dokku.staged_env_set.remove(app).unwrap_or_default();
        let mut unset_list: Vec<String> = self
            .dokku
            .staged_env_unset
            .remove(app)
            .unwrap_or_default()
            .into_iter()
            .collect();
        unset_list.sort();
        let (argvs, redacted_summary) =
            dokku::build_config_commands(app, &set_map, &unset_list, self.dokku.restart_on_env_apply);
        if argvs.is_empty() {
            return;
        }
        let cmds: Vec<HostCommand> = argvs.into_iter().map(HostCommand::new).collect();
        let app_name = app.to_string();
        self.execute_dokku_action(
            "dokku.config.apply",
            app,
            cmds,
            Some(redacted_summary),
            move |this, _res, cx| {
                this.load_dokku_app_env(&app_name, true, cx);
            },
            cx,
        );
    }

    // =========================================================================
    // Stage 5: Domains, Proxy Ports & Let's Encrypt TLS (`ERR-209`)
    // =========================================================================

    pub fn add_dokku_domain(&mut self, app: &str, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let dom = inputs.domain_add.read(cx).value().trim().to_string();
        if let Err(e) = dokku::validate_domain(&dom) {
            self.dokku.result = Some((false, e));
            cx.notify();
            return;
        }
        let cmd = HostCommand::new(vec!["dokku".into(), "domains:add".into(), app.to_string(), dom]);
        self.execute_dokku_action(
            "dokku.domains.add",
            app,
            vec![cmd],
            None,
            move |this, res, _cx| {
                if res.is_ok() {
                    this.dokku_inputs = None;
                }
            },
            cx,
        );
    }

    pub fn remove_dokku_domain(&mut self, app: &str, domain: &str, cx: &mut Context<Self>) {
        let cmd = HostCommand::new(vec![
            "dokku".into(),
            "domains:remove".into(),
            app.to_string(),
            domain.to_string(),
        ]);
        self.execute_dokku_action("dokku.domains.remove", app, vec![cmd], None, |_, _, _| {}, cx);
    }

    pub fn add_dokku_port_map(&mut self, app: &str, cx: &mut Context<Self>) {
        let Some(inputs) = self.dokku_inputs.as_ref() else { return };
        let spec = inputs.port_add.read(cx).value().trim().to_string();
        if let Err(e) = dokku::validate_port_spec(&spec) {
            self.dokku.result = Some((false, e));
            cx.notify();
            return;
        }
        let cmd = HostCommand::new(vec!["dokku".into(), "ports:add".into(), app.to_string(), spec]);
        self.execute_dokku_action(
            "dokku.ports.add",
            app,
            vec![cmd],
            None,
            move |this, res, _cx| {
                if res.is_ok() {
                    this.dokku_inputs = None;
                }
            },
            cx,
        );
    }

    pub fn remove_dokku_port_map(&mut self, app: &str, spec: &str, cx: &mut Context<Self>) {
        let cmd = HostCommand::new(vec![
            "dokku".into(),
            "ports:remove".into(),
            app.to_string(),
            spec.to_string(),
        ]);
        self.execute_dokku_action("dokku.ports.remove", app, vec![cmd], None, |_, _, _| {}, cx);
    }

    pub fn enable_dokku_letsencrypt(&mut self, app: &str, cx: &mut Context<Self>) {
        let mut cmds = Vec::new();
        let has_le = self.dokku.current().is_some_and(|s| s.has_plugin("letsencrypt"));
        if !has_le {
            cmds.push(HostCommand::new(vec![
                "dokku".into(),
                "plugin:install".into(),
                "https://github.com/dokku/dokku-letsencrypt.git".into(),
            ]));
        }
        cmds.push(HostCommand::new(vec![
            "dokku".into(),
            "letsencrypt:enable".into(),
            app.to_string(),
        ]));
        cmds.push(HostCommand::new(vec![
            "dokku".into(),
            "letsencrypt:cron-job".into(),
            "--add".into(),
        ]));
        self.execute_dokku_action("dokku.letsencrypt.enable", app, cmds, None, |_, _, _| {}, cx);
    }

    /// 1-click bridge to Fleet Setup → Checks (`ERR-210`).
    pub fn watch_dokku_domain_in_checks(&mut self, app: &str, domain: &str, https: bool, cx: &mut Context<Self>) {
        let scheme = if https { "https" } else { "http" };
        let url = format!("{scheme}://{domain}");
        let srv_id = self.fleet.active_server().map(|s| s.id);
        let label = self.add_http_check_for_domain(url.clone(), format!("{app} ({domain})"), srv_id, cx);
        self.dokku.result = Some((true, t!("app-dokku-watching-in-checks", label = label, url = url)));
        cx.notify();
    }

    // =========================================================================
    // Logs & AI Wand (`ERR-206`, `ERR-207`)
    // =========================================================================

    pub fn toggle_dokku_follow(&mut self, cx: &mut Context<Self>) {
        self.dokku.follow = !self.dokku.follow;
        if self.dokku.follow {
            self.load_dokku_logs(cx);
        }
        cx.notify();
    }

    pub fn set_dokku_log_filter(&mut self, proc_type: Option<String>, cx: &mut Context<Self>) {
        self.dokku.log_process_filter = proc_type;
        self.dokku.logs = None;
        self.load_dokku_logs(cx);
        cx.notify();
    }

    pub fn load_dokku_logs(&mut self, cx: &mut Context<Self>) {
        if self.dokku.logs_loading {
            return;
        }
        let Some(srv) = self.fleet.active_server() else { return };
        let Some(sel) = self.dokku.selected.clone() else { return };
        let sel_key = sel.key();
        let proc_filter = self.dokku.log_process_filter.clone();
        self.dokku.logs_loading = true;
        self.dokku.logs_at = chrono::Utc::now().timestamp();
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let host = host_for(&srv);
                    match &sel {
                        DokkuSelection::App(app) => {
                            dokku::app_logs(host.as_ref(), app, proc_filter.as_deref())
                        }
                        DokkuSelection::Service(kind, name) => {
                            dokku::service_logs(host.as_ref(), *kind, name)
                        }
                    }
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.dokku.logs_loading = false;
                this.dokku.logs_at = chrono::Utc::now().timestamp();
                if this.dokku.selected.as_ref().map(DokkuSelection::key).as_deref() == Some(sel_key.as_str()) {
                    this.dokku.logs = Some((sel_key, result));
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn explain_dokku_selection(&mut self, cx: &mut Context<Self>) {
        if self.dokku.eli5_open {
            self.dokku.eli5_open = false;
            cx.notify();
            return;
        }
        let Some(scan) = self.dokku.current() else { return };
        let Some(DokkuSelection::App(app_name)) = self.dokku.selected.clone() else { return };
        let Some(app) = scan.find_app(&app_name).cloned() else { return };
        self.dokku.eli5_open = true;
        let key = format!("dokku:{}", app.name);
        if matches!(self.dokku.eli5.get(&key), Some(Ok(_))) || self.dokku.eli5_loading.contains(&key) {
            cx.notify();
            return;
        }
        let Some((primary, backup)) = self.ai_providers() else {
            self.dokku.eli5.insert(key, Err(t!("app-containers-no-ai-provider-has-a").into()));
            cx.notify();
            return;
        } ;
        let processes = app.process_summary();
        let services = app
            .linked_services
            .iter()
            .map(|(k, n)| format!("{}:{n}", k.id()))
            .collect::<Vec<_>>()
            .join(", ");
        let ssl = app.ssl_enabled;
        let maintenance = app.maintenance;
        let state = if app.maintenance {
            "maintenance".to_string()
        } else if app.degraded() {
            "degraded".to_string()
        } else if app.running {
            "running".to_string()
        } else {
            "stopped".to_string()
        };
        let log_tail = self
            .dokku
            .logs
            .as_ref()
            .filter(|(k, _)| k == &format!("app:{app_name}"))
            .and_then(|(_, r)| r.as_ref().ok().cloned())
            .unwrap_or_default();

        self.dokku.eli5.remove(&key);
        self.dokku.eli5_loading.insert(key.clone());
        cx.notify();
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let answer = cx
                .background_executor()
                .spawn(async move {
                    crate::ai::explain_dokku_app(
                        &primary,
                        backup.as_ref(),
                        &crate::ai::DokkuAppFacts {
                            processes: &processes,
                            services: &services,
                            state: &state,
                            ssl,
                            maintenance,
                            logs: &log_tail,
                        },
                    )
                })
                .await;
            if let Ok(a) = &answer {
                if let Ok(db) = db.lock() {
                    let _ = db.record_clanker_call(&a.provider_id);
                }
            }
            let _ = entity.update(cx, |this, cx| {
                this.dokku.eli5_loading.remove(&key);
                let result = answer.map(|a| {
                    let (severity, body) = crate::ai::split_severity(&a.text);
                    ServiceEli5 {
                        severity,
                        body,
                        via: a.provider_label,
                        primary_failed: a.primary_failed,
                    }
                });
                this.dokku.eli5.insert(key, result);
                this.refresh_clankers(cx);
            });
        })
        .detach();
    }
}
