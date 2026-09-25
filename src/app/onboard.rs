use gpui_kit::*;
use gpui_kit::component::input::{InputEvent, InputState};
use zeroize::{Zeroize, Zeroizing};

use super::{CrowApp, Screen};
use crate::components::titlebar::ServerTab;
use crate::theme::{CRIT, OK, TEXT_FAINTER, WARN};
use crate::vault::ServerRecord;
use crate::host::host_for;
use crate::host::bootstrap::install_key_with_password;
use crate::vault::SshKeyRecord;
use crate::views::onboard::{
    gather_facts,
    probe_log,
    ProbeLog,
    probe_host,
    trust_host_keys,
    DetectedFacts,
    OnboardFieldFocus,
    OnboardState,
    OnboardStep,
};

/// The Add Server wizard's text fields: real gpui-component inputs (paste,
/// selection, undo), mirrored into `OnboardState` on every change.
pub struct OnboardInputs {
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub user: Entity<InputState>,
    pub password: Entity<InputState>,
    pub label: Entity<InputState>,
    pub tags: Entity<InputState>,
    _events: Vec<Subscription>,
}

impl OnboardInputs {
    pub fn get(&self, field: OnboardFieldFocus) -> Option<&Entity<InputState>> {
        match field {
            OnboardFieldFocus::Host => Some(&self.host),
            OnboardFieldFocus::Port => Some(&self.port),
            OnboardFieldFocus::User => Some(&self.user),
            OnboardFieldFocus::Password => Some(&self.password),
            OnboardFieldFocus::Label => Some(&self.label),
            OnboardFieldFocus::Tags => Some(&self.tags),
            OnboardFieldFocus::None => None,
        }
    }
}

fn onboard_field_mut(state: &mut OnboardState, field: OnboardFieldFocus) -> Option<&mut String> {
    match field {
        OnboardFieldFocus::Host => Some(&mut state.host),
        OnboardFieldFocus::Port => Some(&mut state.port),
        OnboardFieldFocus::User => Some(&mut state.user),
        OnboardFieldFocus::Password => Some(&mut state.password),
        OnboardFieldFocus::Label => Some(&mut state.label),
        OnboardFieldFocus::Tags => Some(&mut state.tags),
        OnboardFieldFocus::None => None,
    }
}

impl CrowApp {
    /// Creates the wizard's inputs from the current `OnboardState` the first
    /// time they're rendered, and applies a pending focus change.
    pub fn ensure_onboard_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.onboard_inputs.is_none() {
            let st = &self.onboard_state;
            let fields = [
                (OnboardFieldFocus::Host, st.host.clone(), "e.g. 10.0.4.32 or prod-db.internal", false),
                (OnboardFieldFocus::Port, st.port.clone(), "22", false),
                (OnboardFieldFocus::User, st.user.clone(), "root", false),
                (OnboardFieldFocus::Password, st.password.clone(), "password", true),
                (OnboardFieldFocus::Label, st.label.clone(), "e.g. prod-db-01", false),
                (OnboardFieldFocus::Tags, st.tags.clone(), "e.g. postgres, primary", false),
            ];
            let mut events = Vec::new();
            let mut made = Vec::new();
            for (field, value, placeholder, masked) in fields {
                let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).masked(masked).default_value(value));
                events.push(cx.subscribe_in(&input, window, move |this, input, ev: &InputEvent, window, cx| match ev {
                    InputEvent::Change => {
                        let mut value = input.read(cx).value().to_string();
                        if field == OnboardFieldFocus::Port {
                            let digits: String = value.chars().filter(char::is_ascii_digit).take(5).collect();
                            if digits != value {
                                input.update(cx, |i, cx| i.set_value(digits.clone(), window, cx));
                            }
                            value = digits;
                        }
                        if let Some(slot) = onboard_field_mut(&mut this.onboard_state, field) {
                            *slot = value;
                        }
                        this.onboard_state.error_message = None;
                        cx.notify();
                    }
                    InputEvent::Focus => {
                        this.onboard_state.focus = field;
                        cx.notify();
                    }
                    InputEvent::Blur => {
                        if this.onboard_state.focus == field {
                            this.onboard_state.focus = OnboardFieldFocus::None;
                            cx.notify();
                        }
                    }
                    InputEvent::PressEnter { .. } => this.onboard_next_step(cx),
                }));
                made.push(input);
            }
            let [host, port, user, password, label, tags]: [Entity<InputState>; 6] = made.try_into().ok().expect("six inputs");
            self.onboard_inputs = Some(OnboardInputs { host, port, user, password, label, tags, _events: events });
        }
        if std::mem::take(&mut self.onboard_focus_pending) {
            if let Some(input) = self.onboard_inputs.as_ref().and_then(|i| i.get(self.onboard_state.focus)) {
                input.update(cx, |i, cx| i.focus(window, cx));
            }
        }
    }

    pub fn onboard_select_local_lab_node(&mut self, name: &str, port: &str, distro: &str, cx: &mut Context<Self>) {
        self.onboard_state.host = "127.0.0.1".to_string();
        self.onboard_state.port = port.to_string();
        self.onboard_state.label = name.to_string();
        self.onboard_state.user = "root".to_string();
        self.onboard_state.env = "LAB".to_string();
        self.onboard_state.role = format!("test-node · {}", distro);
        self.onboard_state.facts.distro = distro.to_string();
        self.onboard_inputs = None;
        cx.notify();
    }

    pub fn start_onboarding(&mut self, cx: &mut Context<Self>) {
        self.onboard_state = OnboardState::new(&self.keys.enrolled);
        self.onboard_inputs = None;
        self.screen = Screen::Onboard;
        self.menu_open = false;
        self.palette_open = false;
        self.onboard_set_focus_select(OnboardFieldFocus::Host, false, cx);
    }

    /// Add Server, filled in from an enrolled server and waiting for its
    /// password: finishing it installs Crow's key and updates the same entry.
    pub fn start_onboarding_for(&mut self, srv: &ServerRecord, cx: &mut Context<Self>) {
        self.start_onboarding(cx);
        let o = &mut self.onboard_state;
        o.host = srv.host.clone();
        o.port = srv.port.to_string();
        o.user = srv.login_user.clone();
        o.label = srv.name.clone();
        o.env = srv.env.clone();
        o.role = srv.role.clone();
        o.group = srv.group_name.clone();
        o.tags = srv.tags.join(", ");
        o.jump_host_id = srv.jump_host_id.clone();
        o.auth_method = "password".into();
        o.selected_key_id = None;
        o.step = OnboardStep::Credentials;
        o.max_reached_step = OnboardStep::Credentials;
        self.onboard_inputs = None;
        self.onboard_set_focus_select(OnboardFieldFocus::Password, false, cx);
    }

    pub fn reload_servers(&mut self) {
        if let Ok(db_guard) = self.vault.db().lock() {
            self.fleet.servers = db_guard.list_servers().unwrap_or_default();
        }
        self.sync_ssh_directory();
    }

    /// Moves the wizard's focus to `focus`; the input is focused on the next
    /// render (focusing needs the window).
    pub fn onboard_set_focus_select(&mut self, focus: OnboardFieldFocus, _select_all: bool, cx: &mut Context<Self>) {
        self.onboard_state.focus = focus;
        self.onboard_focus_pending = true;
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

    /// The server being enrolled, as a record its transport can be built from.
    fn onboard_candidate(&self) -> ServerRecord {
        let o = &self.onboard_state;
        ServerRecord {
            id: "onboarding".into(),
            name: o.host.trim().to_string(),
            host: o.host.trim().to_string(),
            port: o.port.trim().parse::<u16>().unwrap_or(22),
            login_user: o.user.trim().to_string(),
            auth_method: o.auth_method.clone(),
            key_id: o.selected_key_id.clone(),
            jump_host_id: o.jump_host_id.clone(),
            ..ServerRecord::default()
        }
    }

    /// Crow's own key for servers enrolled with a password
    /// (`~/.ssh/crow_ed25519`): created once, recorded in the vault, reused.
    fn ensure_crow_key(&mut self, cx: &mut Context<Self>) -> Result<SshKeyRecord, String> {
        let exists = |k: &SshKeyRecord| k.name == CROW_KEY_NAME && k.private_key_path.as_deref().is_some_and(|p| crate::keys::expand_tilde(p).exists());
        if let Some(k) = self.keys.enrolled.iter().find(|k| exists(k)) {
            return Ok(k.clone());
        }
        let dir = dirs::home_dir().ok_or("no home directory")?.join(".ssh");
        // Never overwrite a key file that isn't recorded in this vault.
        let file = (0..100)
            .map(|i| if i == 0 { "crow_ed25519".to_string() } else { format!("crow_ed25519_{i}") })
            .find(|f| !dir.join(f).exists())
            .ok_or("no free crow_ed25519 file name in ~/.ssh")?;
        let host = std::fs::read_to_string("/etc/hostname").map(|h| h.trim().to_string()).unwrap_or_default();
        let comment = format!("crow@{}", if host.is_empty() { "workstation" } else { &host });
        let (record, ..) = crate::keys::generate_keypair(CROW_KEY_NAME, crate::keys::KeyAlgorithm::Ed25519, Some(&comment), "fleet", &dir, Some(&file))?;
        if let Ok(db) = self.vault.db().lock() {
            db.upsert_ssh_key(&record).map_err(|e| format!("saving Crow's key in the vault: {e}"))?;
        }
        self.refresh_keys(cx);
        Ok(record)
    }

    /// For password login: the password (taken out of the wizard state) and
    /// Crow's key to install with it. Logs why when it can't start.
    fn password_bootstrap_inputs(&mut self, cx: &mut Context<Self>) -> Result<Option<Bootstrap>, String> {
        if self.onboard_state.auth_method != "password" {
            return Ok(None);
        }
        if self.onboard_state.password.is_empty() {
            return Err("enter the server's password on step 2 (Credentials)".into());
        }
        let key = self.ensure_crow_key(cx)?;
        let jump = self.onboard_state.jump_host_id.as_ref().and_then(|id| self.fleet.servers.iter().find(|s| &s.id == id)).map(|j| {
            if j.login_user.is_empty() { format!("{}:{}", j.host, j.port) } else { format!("{}@{}:{}", j.login_user, j.host, j.port) }
        });
        Ok(Some(Bootstrap { password: Zeroizing::new(self.onboard_state.password.clone()), key, jump }))
    }

    /// Probes the address off the UI thread: TCP, banner, real host keys.
    /// When the key is already trusted, logs in (installing Crow's key first
    /// for password login) and reads the server's facts.
    pub fn onboard_run_probe(&mut self, cx: &mut Context<Self>) {
        let candidate = self.onboard_candidate();
        self.onboard_state.is_probing = true;
        self.onboard_state.probe_result = None;
        self.onboard_state.probe_logs.clear();
        self.onboard_state.facts = DetectedFacts::default();
        let bootstrap = self.password_bootstrap_inputs(cx);
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let (result, mut logs, facts, installed) = cx
                .background_executor()
                .spawn(async move {
                    let (result, mut logs) = probe_host(&candidate.host, candidate.port);
                    let (facts, installed) = if result.is_known_host {
                        connect_and_read(candidate, bootstrap, &mut logs)
                    } else {
                        (DetectedFacts::default(), None)
                    };
                    (result, logs, facts, installed)
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.onboard_state.host_key_accepted = result.is_known_host;
                this.onboard_state.probe_result = Some(result);
                this.onboard_state.probe_logs.append(&mut logs);
                this.onboard_state.facts = facts;
                this.onboard_state.is_probing = false;
                this.switch_onboarding_to_key(installed);
                cx.notify();
            });
        })
        .detach();
    }

    /// After a password bootstrap installed Crow's key: the server is enrolled
    /// with that key and the password is wiped.
    fn switch_onboarding_to_key(&mut self, installed: Option<String>) {
        if let Some(key_id) = installed {
            self.onboard_state.auth_method = "publickey".into();
            self.onboard_state.selected_key_id = Some(key_id);
            self.onboard_state.password.zeroize();
            self.onboard_inputs = None;
        }
    }

    /// Trusts exactly the host keys the server presented during the probe,
    /// then logs in and reads its facts over SSH. Refused when known_hosts
    /// already holds a different key for this host.
    pub fn onboard_accept_host_key(&mut self, cx: &mut Context<Self>) {
        let Some(result) = self.onboard_state.probe_result.as_ref() else { return };
        if result.host_key_mismatch {
            self.onboard_state.error_message = Some("The host key changed. Remove the old entry from ~/.ssh/known_hosts yourself if the change is expected.".into());
            cx.notify();
            return;
        }
        if result.scanned_keys.is_empty() {
            self.onboard_state.error_message = Some("No host key was fetched from the server; probe it again.".into());
            cx.notify();
            return;
        }
        if let Err(e) = trust_host_keys(&result.scanned_keys) {
            self.onboard_state.error_message = Some(format!("Couldn't write ~/.ssh/known_hosts: {e}"));
            cx.notify();
            return;
        }
        self.onboard_state.host_key_accepted = true;
        if let Some(res) = self.onboard_state.probe_result.as_mut() {
            res.is_known_host = true;
        }
        let candidate = self.onboard_candidate();
        let bootstrap = self.password_bootstrap_inputs(cx);
        self.onboard_state.is_probing = true;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let (facts, mut logs, installed) = cx
                .background_executor()
                .spawn(async move {
                    let mut logs = Vec::new();
                    let (facts, installed) = connect_and_read(candidate, bootstrap, &mut logs);
                    (facts, logs, installed)
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.onboard_state.probe_logs.append(&mut logs);
                this.onboard_state.facts = facts;
                this.onboard_state.is_probing = false;
                this.switch_onboarding_to_key(installed);
                cx.notify();
            });
        })
        .detach();
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
            // Only key login uses a key; a key picked earlier in the wizard
            // doesn't belong to a password or ssh-agent server.
            key_id: self.onboard_state.selected_key_id.clone().filter(|_| self.onboard_state.auth_method == "publickey"),
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
            agent_installed: false,
            agent_version: None,
            status: status.clone(),
            created_at: now.clone(),
            last_seen_at: Some(now),
            archived_at: None,
            purged_at: None,
            region_country: self.onboard_state.facts.region.as_ref().map(|r| r.country.clone()).unwrap_or_default(),
            region_city: self.onboard_state.facts.region.as_ref().map(|r| r.city.clone()).unwrap_or_default(),
            region_provider: self.onboard_state.facts.region.as_ref().map(|r| r.provider.clone()).unwrap_or_default(),
            region_code: self.onboard_state.facts.region.as_ref().map(|r| r.code.clone()).unwrap_or_default(),
            region_source: if self.onboard_state.facts.region.is_some() { "metadata".into() } else { String::new() },
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
        if !self.fleet.tabs.iter().any(|t| t.id == id) {
            self.fleet.tabs.push(ServerTab {
                id: id.clone(),
                name: name.clone(),
                status_color,
                is_active: true,
            });
        }
        self.fleet.active_tab_id = id.clone();
        self.reload_configs_for_active_server(cx);
        self.screen = Screen::Server;
        self.active_view = "overview".to_string();
        self.keys.toast = Some(format!("Server '{}' enrolled into fleet", name));
        // Nothing of the wizard outlives it, least of all a password.
        self.onboard_state.password.zeroize();
        self.onboard_inputs = None;

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

const CROW_KEY_NAME: &str = "crow";

/// What a password bootstrap needs, gathered on the UI thread.
pub struct Bootstrap {
    password: Zeroizing<String>,
    key: SshKeyRecord,
    jump: Option<String>,
}

/// Logs in to the server being enrolled and reads its facts. For password
/// login it first installs Crow's key with the password (once), then
/// connects with the key. Returns the facts and, when a key was installed,
/// its id.
fn connect_and_read(mut candidate: ServerRecord, bootstrap: Result<Option<Bootstrap>, String>, logs: &mut Vec<ProbeLog>) -> (DetectedFacts, Option<String>) {
    let mut installed = None;
    match bootstrap {
        Err(why) => {
            probe_log(logs, "✕", CRIT, format!("password login: {why}"), String::new());
            return (DetectedFacts::default(), None);
        }
        Ok(Some(b)) => match install_key_with_password(&candidate, b.jump.as_deref(), b.password, &b.key.public_key) {
            Ok(()) => {
                probe_log(logs, "✓", OK, format!("logged in with the password once; installed Crow's key ({})", b.key.fingerprint), "key-only from now on".into());
                candidate.auth_method = "publickey".into();
                candidate.key_id = Some(b.key.id.clone());
                installed = Some(b.key.id);
            }
            Err(e) => {
                probe_log(logs, "✕", CRIT, format!("password login: {e}"), String::new());
                return (DetectedFacts::default(), None);
            }
        },
        Ok(None) => {}
    }
    let (facts, fact_logs) = gather_facts(host_for(&candidate).as_ref());
    logs.extend(fact_logs);
    (facts, installed)
}
