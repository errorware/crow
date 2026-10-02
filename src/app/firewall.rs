use gpui_kit::*;

use super::CrowApp;
use super::host_actions::HostCommand;
use crate::views::firewall::commands;
use crate::views::firewall::{detect_firewall_status, FirewallBackend, FirewallOperationalState, RuleAction, RuleProtocol};

// ==========================================
// Firewall & Network Security Methods
// ==========================================

impl CrowApp {
    pub fn set_firewall_search(&mut self, query: &str, cx: &mut Context<Self>) {
        self.firewall.search_query = query.to_string();
        cx.notify();
    }

    pub fn set_firewall_action_filter(&mut self, filter: Option<RuleAction>, cx: &mut Context<Self>) {
        self.firewall.action_filter = filter;
        cx.notify();
    }

    pub fn open_new_firewall_rule_modal(&mut self, cx: &mut Context<Self>) {
        self.firewall.open_new_rule_modal();
        cx.notify();
    }

    pub fn close_new_firewall_rule_modal(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_new_rule_modal = false;
        cx.notify();
    }

    /// Mirrors the in-memory rule set into the `user.rules` config file state so
    /// the pending-diff rail and versioning see firewall edits.
    pub fn sync_firewall_to_config_state(&mut self) {
        if let FirewallOperationalState::Active(ref summary) = self.firewall.status.clone().ufw_only() {
            let content = crate::views::firewall::generate_user_rules_content(&summary.rules);
            if let Some(st) = self.configs.states.get_mut("user.rules") {
                st.update_content(content);
            }
        }
    }

    pub fn toggle_firewall_audit_rail(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_audit_rail = !self.firewall.show_audit_rail;
        cx.notify();
    }

    pub fn toggle_firewall_raw(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_raw = !self.firewall.show_raw;
        cx.notify();
    }

    pub fn stage_firewall_rules(&mut self, message: &str, cx: &mut Context<Self>) {
        self.sync_firewall_to_config_state();
        self.stage_config_version("user.rules", message, cx);
    }

    /// Reads the active server's firewall in the background (startup with a
    /// remote first server, and every tab switch). The result is dropped if
    /// the active server changed meanwhile.
    pub fn refresh_firewall_for_active_server(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        self.firewall.status = FirewallOperationalState::Unmanaged {
            detected_binaries: Vec::new(),
            reason: format!("Reading {}'s firewall…", srv.name),
        };
        let id = srv.id.clone();
        cx.spawn(async move |entity, cx| {
            let status = cx.background_executor().spawn(async move { detect_firewall_status(&srv) }).await;
            let _ = entity.update(cx, |this, cx| {
                if this.fleet.active_server().is_some_and(|s| s.id == id) {
                    this.firewall.status = status;
                    // ufw's own rules file, as load_configs mirrors it.
                    if let FirewallOperationalState::Active(ref summary) = this.firewall.status.clone().ufw_only() {
                        let text = crate::views::firewall::generate_user_rules_content(&summary.rules);
                        this.configs.seed_baseline("user.rules", text, Some("/etc/ufw/user.rules"));
                        this.configs.block_writes("user.rules", crate::app::configs::user_rules_read_only_reason().into());
                    }
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Runs a ufw command on the active server, then shows its real state.
    fn run_firewall_command(&mut self, argv: Vec<String>, closes_modal: bool, cx: &mut Context<Self>) {
        // Every command here is a ufw command; other backends are read-only.
        let backend = self.firewall.status.backend();
        if backend != FirewallBackend::Ufw {
            self.firewall.toast = Some(match backend {
                FirewallBackend::NoneDetected => "Not applied: ufw isn't installed on this host.".to_string(),
                b => format!("Not applied: Crow reads {} but doesn't change it yet.", b.label()),
            });
            cx.notify();
            return;
        }
        let target = argv.get(1..).map(|rest| rest.join(" ")).unwrap_or_default();
        self.firewall.pending = Some(commands::describe(&argv));
        self.run_host_action(
            "firewall",
            &target,
            vec![HostCommand::new(argv)],
            |srv| detect_firewall_status(srv),
            move |this, status, result, summary, cx| {
                this.firewall.status = status;
                this.firewall.pending = None;
                this.firewall.toast = Some(match &result {
                    Ok(()) => format!("Ran: {summary}"),
                    Err(e) => format!("Failed: {summary} — {e}"),
                });
                if closes_modal && result.is_ok() {
                    this.firewall.show_new_rule_modal = false;
                }
                this.sync_firewall_to_config_state();
                cx.notify();
            },
            cx,
        );
    }

    pub fn toggle_firewall_active(&mut self, cx: &mut Context<Self>) {
        let enabled = matches!(&self.firewall.status, FirewallOperationalState::Active(s) if s.is_active);
        let argv = if enabled { commands::disable() } else { commands::enable() };
        self.run_firewall_command(argv, false, cx);
    }

    pub fn reload_firewall(&mut self, cx: &mut Context<Self>) {
        self.run_firewall_command(commands::reload(), false, cx);
    }

    pub fn toggle_quick_port(&mut self, port: u16, proto: RuleProtocol, label: &str, cx: &mut Context<Self>) {
        let summary = match &self.firewall.status {
            FirewallOperationalState::Active(s) => Some(s),
            _ => None,
        };
        let argv = commands::toggle_port(summary, port, proto, label);
        self.run_firewall_command(argv, false, cx);
    }

    pub fn delete_firewall_rule(&mut self, rule_id: &str, cx: &mut Context<Self>) {
        let number = match &self.firewall.status {
            FirewallOperationalState::Active(s) => s.rules.iter().find(|r| r.id == rule_id).map(|r| r.number),
            _ => None,
        };
        if let Some(n) = number {
            self.run_firewall_command(commands::delete_numbered(n), false, cx);
        }
    }

    pub fn submit_new_firewall_rule(&mut self, cx: &mut Context<Self>) {
        match commands::new_rule(&self.firewall.new_rule) {
            Ok(argv) => self.run_firewall_command(argv, true, cx),
            Err(e) => {
                self.firewall.toast = Some(format!("Not applied: {e}"));
                cx.notify();
            }
        }
    }

    #[allow(dead_code)]
    pub fn flush_firewall_rules(&mut self, cx: &mut Context<Self>) {
        self.run_firewall_command(commands::reset(), false, cx);
    }
}
