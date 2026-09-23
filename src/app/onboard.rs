use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::components::titlebar::ServerTab;
use crate::theme::{CRIT, OK, TEXT_FAINTER, WARN};
use crate::vault::ServerRecord;
use crate::views::onboard::{
    append_to_known_hosts,
    probe_host,
    OnboardFieldFocus,
    OnboardState,
    OnboardStep,
};

impl CrowApp {
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

    pub fn start_onboarding(&mut self, cx: &mut Context<Self>) {
        self.onboard_state = OnboardState::new(&self.keys.enrolled);
        self.screen = Screen::Onboard;
        self.menu_open = false;
        self.palette_open = false;
        self.onboard_set_focus_select(OnboardFieldFocus::Host, false, cx);
    }

    pub fn reload_servers(&mut self) {
        if let Ok(db_guard) = self.vault.db().lock() {
            self.fleet.servers = db_guard.list_servers().unwrap_or_default();
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
        self.caret.blink = true;
        self.caret.drag_anchor = None;
        if select_all && text_len > 0 {
            self.caret.selection = Some((0, text_len));
            self.caret.cursor = text_len;
        } else {
            self.caret.selection = None;
            self.caret.cursor = text_len;
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
        if !self.fleet.tabs.iter().any(|t| t.id == id) {
            self.fleet.tabs.push(ServerTab {
                id: id.clone(),
                name: name.clone(),
                status_color,
                is_active: true,
            });
        }
        self.fleet.active_tab_id = id.clone();
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
