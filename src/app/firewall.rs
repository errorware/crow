use gpui_kit::*;

use super::CrowApp;
use crate::views::firewall::{FirewallOperationalState, RuleAction, RuleProtocol};

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
        if let FirewallOperationalState::Active(ref summary) = self.firewall.status {
            let content = crate::views::firewall::generate_user_rules_content(&summary.rules);
            if let Some(st) = self.config_file_states.get_mut("user.rules") {
                st.update_content(content);
            }
        }
    }

    pub fn toggle_firewall_audit_rail(&mut self, cx: &mut Context<Self>) {
        self.firewall.show_audit_rail = !self.firewall.show_audit_rail;
        cx.notify();
    }

    pub fn stage_firewall_rules(&mut self, message: &str, cx: &mut Context<Self>) {
        self.sync_firewall_to_config_state();
        self.stage_config_version("user.rules", message, cx);
    }

    pub fn toggle_firewall_active(&mut self, cx: &mut Context<Self>) {
        self.firewall.toggle_active();
        self.sync_firewall_to_config_state();
        cx.notify();
    }

    pub fn reload_firewall(&mut self, cx: &mut Context<Self>) {
        self.firewall.toast = Some("Executed: sudo ufw reload (Firewall reloaded)".into());
        cx.notify();
    }

    pub fn toggle_quick_port(&mut self, port: u16, proto: RuleProtocol, label: &str, cx: &mut Context<Self>) {
        if self.firewall.toggle_quick_port(port, proto, label) {
            self.sync_firewall_to_config_state();
            cx.notify();
        }
    }

    pub fn delete_firewall_rule(&mut self, rule_id: &str, cx: &mut Context<Self>) {
        if self.firewall.delete_rule(rule_id) {
            self.sync_firewall_to_config_state();
            cx.notify();
        }
    }

    pub fn submit_new_firewall_rule(&mut self, cx: &mut Context<Self>) {
        if self.firewall.submit_new_rule() {
            self.sync_firewall_to_config_state();
            cx.notify();
        }
    }

    #[allow(dead_code)]
    pub fn flush_firewall_rules(&mut self, cx: &mut Context<Self>) {
        if self.firewall.flush_rules() {
            self.sync_firewall_to_config_state();
            cx.notify();
        }
    }
}
