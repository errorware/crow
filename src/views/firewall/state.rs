use super::{
    default_active_ufw_state, default_ufw_rules, FirewallOperationalState, FirewallRule,
    FirewallStatusSummary, NewRuleState, RuleAction, RuleDirection, RuleProtocol,
};

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
        }
    }

    pub fn active_summary_mut(&mut self) -> Option<&mut FirewallStatusSummary> {
        match self.status {
            FirewallOperationalState::Active(ref mut summary) => Some(summary),
            _ => None,
        }
    }

    pub fn open_new_rule_modal(&mut self) {
        self.show_new_rule_modal = true;
        self.new_rule = NewRuleState::default();
    }

    pub fn toggle_active(&mut self) {
        match &mut self.status {
            FirewallOperationalState::Active(summary) => {
                summary.is_active = !summary.is_active;
                let status_str = if summary.is_active { "ENABLED (ufw enable)" } else { "DISABLED (ufw disable)" };
                self.toast = Some(format!("Executed: sudo {}", status_str));
            }
            FirewallOperationalState::Inactive { backend, .. } => {
                let be = *backend;
                self.status = FirewallOperationalState::Active(FirewallStatusSummary {
                    backend: be,
                    is_active: true,
                    default_incoming: RuleAction::Deny,
                    default_outgoing: RuleAction::Allow,
                    default_forward: RuleAction::Deny,
                    rules: default_ufw_rules(),
                    raw_output: "Status: active".into(),
                });
                self.toast = Some("Executed: sudo ufw enable".into());
            }
            FirewallOperationalState::Unmanaged { .. } => {
                self.status = default_active_ufw_state();
                self.toast = Some("Initialized UFW packet filter".into());
            }
        }
    }

    /// Returns false when there is no active rule set to change.
    pub fn toggle_quick_port(&mut self, port: u16, proto: RuleProtocol, label: &str) -> bool {
        let port_str = port.to_string();
        let Some(summary) = self.active_summary_mut() else { return false };
        let toast = if let Some(pos) = summary.rules.iter().position(|r| r.action == RuleAction::Allow && r.port == port_str) {
            summary.rules.remove(pos);
            format!("Executed: ufw delete allow {}/tcp ({})", port, label)
        } else {
            let next_num = summary.rules.iter().map(|r| r.number).max().unwrap_or(0) + 1;
            summary.rules.push(FirewallRule {
                id: format!("rule-{}", next_num),
                number: next_num,
                action: RuleAction::Allow,
                direction: RuleDirection::Inbound,
                port: port_str.clone(),
                protocol: proto,
                source: "Anywhere".to_string(),
                destination: "Anywhere".to_string(),
                comment: Some(format!("{} Service Ingress", label)),
                is_ipv6: false,
            });
            format!("Executed: ufw allow {}/tcp ({})", port, label)
        };
        self.toast = Some(toast);
        true
    }

    /// Returns false when there is no active rule set to change.
    pub fn delete_rule(&mut self, rule_id: &str) -> bool {
        let Some(summary) = self.active_summary_mut() else { return false };
        let mut toast = None;
        if let Some(pos) = summary.rules.iter().position(|r| r.id == rule_id) {
            let removed = summary.rules.remove(pos);
            toast = Some(format!("Executed: ufw delete [{}] ({})", removed.number, removed.display_port_proto()));
            // Re-index remaining rules
            for (i, r) in summary.rules.iter_mut().enumerate() {
                r.number = i + 1;
            }
        }
        if toast.is_some() {
            self.toast = toast;
        }
        true
    }

    /// Returns false when there is no active rule set to change.
    pub fn submit_new_rule(&mut self) -> bool {
        let form = &self.new_rule;
        let cmd = form.generate_ufw_command();
        let port = form.port_input.trim().to_string();
        let source = if form.is_anywhere || form.source_input.trim().is_empty() {
            "Anywhere".to_string()
        } else {
            form.source_input.trim().to_string()
        };
        let comment = if form.comment_input.trim().is_empty() {
            None
        } else {
            Some(form.comment_input.trim().to_string())
        };
        let (action, direction, protocol) = (form.action, form.direction, form.protocol);

        let Some(summary) = self.active_summary_mut() else { return false };
        let next_num = summary.rules.iter().map(|r| r.number).max().unwrap_or(0) + 1;
        summary.rules.push(FirewallRule {
            id: format!("rule-{}", next_num),
            number: next_num,
            action,
            direction,
            port,
            protocol,
            source,
            destination: "Anywhere".to_string(),
            comment,
            is_ipv6: false,
        });
        self.show_new_rule_modal = false;
        self.toast = Some(format!("Executed: {}", cmd));
        true
    }

    /// Returns false when there is no active rule set to change.
    pub fn flush_rules(&mut self) -> bool {
        let Some(summary) = self.active_summary_mut() else { return false };
        summary.rules.clear();
        self.toast = Some("Executed: sudo ufw reset (Flushed all firewall rules)".into());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> FirewallState {
        FirewallState::new(default_active_ufw_state())
    }

    #[test]
    fn quick_port_toggles_allow_rule() {
        let mut fw = active();
        let before = fw.active_summary_mut().unwrap().rules.len();
        assert!(fw.toggle_quick_port(8443, RuleProtocol::Tcp, "Alt HTTPS"));
        assert_eq!(fw.active_summary_mut().unwrap().rules.len(), before + 1);
        assert!(fw.toggle_quick_port(8443, RuleProtocol::Tcp, "Alt HTTPS"));
        assert_eq!(fw.active_summary_mut().unwrap().rules.len(), before);
    }

    #[test]
    fn delete_rule_renumbers() {
        let mut fw = active();
        let first_id = fw.active_summary_mut().unwrap().rules[0].id.clone();
        assert!(fw.delete_rule(&first_id));
        let numbers: Vec<usize> = fw.active_summary_mut().unwrap().rules.iter().map(|r| r.number).collect();
        assert_eq!(numbers, (1..=numbers.len()).collect::<Vec<_>>());
    }

    #[test]
    fn edits_are_refused_without_active_rules() {
        let mut fw = FirewallState::new(FirewallOperationalState::Unmanaged {
            detected_binaries: vec![],
            reason: "none".into(),
        });
        assert!(!fw.flush_rules());
        assert!(!fw.submit_new_rule());
    }
}
