use gpui_kit::component::input::InputState;
use gpui_kit::Entity;

use super::{FirewallOperationalState, NewRuleState, RuleAction};

/// A firewall change held back by the lock-out guard until CONFIRM is typed.
pub struct LockoutConfirm {
    pub commands: Vec<Vec<String>>,
    pub closes_modal: bool,
    pub reason: String,
    /// Created on the next render (inputs need the window).
    pub input: Option<Entity<InputState>>,
    pub error: Option<String>,
}
#[cfg(test)]
use super::default_active_ufw_state;

/// Firewall screen state: the detected backend status plus the UI around it
/// (search, filters, new-rule modal, audit rail, toast).
pub struct FirewallState {
    pub status: FirewallOperationalState,
    pub search_query: String,
    pub action_filter: Option<RuleAction>,
    pub show_new_rule_modal: bool,
    pub new_rule: NewRuleState,
    pub toast: Option<String>,
    pub show_audit_rail: bool,
    /// The command currently running on the server, if any.
    pub pending: Option<String>,
    /// Read-only backends: the ruleset as the host printed it, in place of
    /// the rule table.
    pub show_raw: bool,
    pub lockout: Option<LockoutConfirm>,
}

impl FirewallState {
    pub fn new(status: FirewallOperationalState) -> Self {
        Self {
            status,
            search_query: String::new(),
            action_filter: None,
            show_new_rule_modal: false,
            new_rule: NewRuleState::default(),
            toast: None,
            show_audit_rail: true,
            pending: None,
            show_raw: false,
            lockout: None,
        }
    }

    pub fn open_new_rule_modal(&mut self) {
        self.show_new_rule_modal = true;
        self.new_rule = NewRuleState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_the_rule_modal_resets_the_form() {
        let mut fw = FirewallState::new(default_active_ufw_state());
        fw.new_rule.port_input = "65000".into();
        fw.open_new_rule_modal();
        assert!(fw.show_new_rule_modal);
        assert_eq!(fw.new_rule.port_input, NewRuleState::default().port_input);
    }
}
