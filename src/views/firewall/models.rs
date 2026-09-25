use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallBackend {
    Ufw,
    Firewalld,
    Nftables,
    Iptables,
    NoneDetected,
}

impl FirewallBackend {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ufw => "UFW (Uncomplicated Firewall)",
            Self::Firewalld => "firewalld",
            Self::Nftables => "nftables",
            Self::Iptables => "iptables",
            Self::NoneDetected => "Unmanaged / None Detected",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Ufw => "UFW",
            Self::Firewalld => "FIREWALLD",
            Self::Nftables => "NFTABLES",
            Self::Iptables => "IPTABLES",
            Self::NoneDetected => "NONE",
        }
    }

    #[allow(dead_code)]
    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::Ufw => "ufw",
            Self::Firewalld => "firewall-cmd",
            Self::Nftables => "nft",
            Self::Iptables => "iptables",
            Self::NoneDetected => "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleDirection {
    Inbound,
    Outbound,
    Forward,
}

impl RuleDirection {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Inbound => "IN",
            Self::Outbound => "OUT",
            Self::Forward => "FWD",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny,
    Reject,
    Limit,
}

impl RuleAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::Deny => "DENY",
            Self::Reject => "REJECT",
            Self::Limit => "LIMIT",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleProtocol {
    Tcp,
    Udp,
    Both,
    Any,
}

#[allow(dead_code)]
impl RuleProtocol {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
            Self::Both => "TCP+UDP",
            Self::Any => "ANY",
        }
    }

    pub fn to_ufw_arg(&self) -> Option<&'static str> {
        match self {
            Self::Tcp => Some("tcp"),
            Self::Udp => Some("udp"),
            Self::Both => None,
            Self::Any => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: String,
    pub number: usize,
    pub action: RuleAction,
    pub direction: RuleDirection,
    pub port: String,
    pub protocol: RuleProtocol,
    pub source: String,
    pub destination: String,
    pub comment: Option<String>,
    pub is_ipv6: bool,
}

impl FirewallRule {
    pub fn display_port_proto(&self) -> String {
        if self.port.is_empty() {
            return "any".to_string();
        }
        match self.protocol {
            RuleProtocol::Any => self.port.clone(),
            RuleProtocol::Both => format!("{}/tcp,udp", self.port),
            RuleProtocol::Tcp => format!("{}/tcp", self.port),
            RuleProtocol::Udp => format!("{}/udp", self.port),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirewallStatusSummary {
    pub backend: FirewallBackend,
    pub is_active: bool,
    pub default_incoming: RuleAction,
    pub default_outgoing: RuleAction,
    pub default_forward: RuleAction,
    pub rules: Vec<FirewallRule>,
    pub raw_output: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallOperationalState {
    Active(FirewallStatusSummary),
    Inactive {
        backend: FirewallBackend,
        reason: String,
        detected_binaries: Vec<String>,
        has_root: bool,
    },
    Unmanaged {
        detected_binaries: Vec<String>,
        reason: String,
    },
}

#[allow(dead_code)]
impl FirewallOperationalState {
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Active(s) if s.is_active)
    }

    pub fn backend(&self) -> FirewallBackend {
        match self {
            Self::Active(s) => s.backend,
            Self::Inactive { backend, .. } => *backend,
            Self::Unmanaged { .. } => FirewallBackend::NoneDetected,
        }
    }

    pub fn rules_count(&self) -> usize {
        match self {
            Self::Active(s) => s.rules.len(),
            _ => 0,
        }
    }
}

/// Firewall rule correlation outcome for a listening or established port
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortFirewallMatch {
    Allowed { rule_number: usize, comment: Option<String> },
    Denied { rule_number: usize, comment: Option<String> },
    AllowedDefault,
    NoRule,
    Inactive,
}

impl PortFirewallMatch {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Allowed { .. } => "UFW: ALLOWED",
            Self::Denied { .. } => "UFW: BLOCKED",
            Self::AllowedDefault => "UFW: ALLOW (DEFAULT)",
            Self::NoRule => "UFW: NO INBOUND RULE",
            Self::Inactive => "FIREWALL: INACTIVE",
        }
    }

    pub fn color(&self) -> gpui_kit::Rgba {
        match self {
            Self::Allowed { .. } | Self::AllowedDefault => crate::theme::OK,
            Self::Denied { .. } => crate::theme::CRIT,
            Self::NoRule => crate::theme::WARN,
            Self::Inactive => crate::theme::TEXT_MUTED,
        }
    }
}

pub fn correlate_port_firewall(
    status: Option<&FirewallOperationalState>,
    port: &str,
    protocol: &str,
) -> PortFirewallMatch {
    let Some(status) = status else {
        return PortFirewallMatch::Inactive;
    };
    match status {
        FirewallOperationalState::Active(summary) if summary.is_active => {
            let port_num: u16 = port.parse().unwrap_or(0);
            let proto_upper = protocol.to_uppercase();
            let is_tcp = proto_upper.contains("TCP");
            let is_udp = proto_upper.contains("UDP");

            for rule in &summary.rules {
                if rule.direction != RuleDirection::Inbound {
                    continue;
                }
                let proto_match = match rule.protocol {
                    RuleProtocol::Tcp => is_tcp,
                    RuleProtocol::Udp => is_udp,
                    RuleProtocol::Both | RuleProtocol::Any => is_tcp || is_udp,
                };
                if !proto_match {
                    continue;
                }
                let port_match = if rule.port.is_empty() {
                    true
                } else if rule.port == port {
                    true
                } else if let Some((start_s, end_s)) = rule.port.split_once(':') {
                    if let (Ok(start), Ok(end)) = (start_s.parse::<u16>(), end_s.parse::<u16>()) {
                        port_num >= start && port_num <= end
                    } else {
                        false
                    }
                } else {
                    false
                };

                if port_match {
                    match rule.action {
                        RuleAction::Allow | RuleAction::Limit => {
                            return PortFirewallMatch::Allowed {
                                rule_number: rule.number,
                                comment: rule.comment.clone(),
                            };
                        }
                        RuleAction::Deny | RuleAction::Reject => {
                            return PortFirewallMatch::Denied {
                                rule_number: rule.number,
                                comment: rule.comment.clone(),
                            };
                        }
                    }
                }
            }

            match summary.default_incoming {
                RuleAction::Allow => PortFirewallMatch::AllowedDefault,
                _ => PortFirewallMatch::NoRule,
            }
        }
        _ => PortFirewallMatch::Inactive,
    }
}

/// Generates baseline seeded UFW rules for an active production Linux node
/// A representative ufw ruleset, for tests.
#[cfg(test)]
pub fn default_ufw_rules() -> Vec<FirewallRule> {
    vec![
        FirewallRule {
            id: "rule-1".into(),
            number: 1,
            action: RuleAction::Allow,
            direction: RuleDirection::Inbound,
            port: "22".into(),
            protocol: RuleProtocol::Tcp,
            source: "Anywhere".into(),
            destination: "Anywhere".into(),
            comment: Some("OpenSSH Remote Administration".into()),
            is_ipv6: false,
        },
        FirewallRule {
            id: "rule-2".into(),
            number: 2,
            action: RuleAction::Allow,
            direction: RuleDirection::Inbound,
            port: "80".into(),
            protocol: RuleProtocol::Tcp,
            source: "Anywhere".into(),
            destination: "Anywhere".into(),
            comment: Some("Nginx HTTP Ingress".into()),
            is_ipv6: false,
        },
        FirewallRule {
            id: "rule-3".into(),
            number: 3,
            action: RuleAction::Allow,
            direction: RuleDirection::Inbound,
            port: "443".into(),
            protocol: RuleProtocol::Tcp,
            source: "Anywhere".into(),
            destination: "Anywhere".into(),
            comment: Some("Nginx HTTPS Ingress (TLS/SSL)".into()),
            is_ipv6: false,
        },
        FirewallRule {
            id: "rule-4".into(),
            number: 4,
            action: RuleAction::Allow,
            direction: RuleDirection::Inbound,
            port: "5432".into(),
            protocol: RuleProtocol::Tcp,
            source: "10.0.4.0/24".into(),
            destination: "Anywhere".into(),
            comment: Some("PostgreSQL Database Internal Subnet".into()),
            is_ipv6: false,
        },
        FirewallRule {
            id: "rule-5".into(),
            number: 5,
            action: RuleAction::Deny,
            direction: RuleDirection::Inbound,
            port: "6379".into(),
            protocol: RuleProtocol::Tcp,
            source: "Anywhere".into(),
            destination: "Anywhere".into(),
            comment: Some("Block Public Redis Port".into()),
            is_ipv6: false,
        },
        FirewallRule {
            id: "rule-6".into(),
            number: 6,
            action: RuleAction::Limit,
            direction: RuleDirection::Inbound,
            port: "22".into(),
            protocol: RuleProtocol::Tcp,
            source: "Anywhere (v6)".into(),
            destination: "Anywhere (v6)".into(),
            comment: Some("SSH Brute-Force Rate Limit IPv6".into()),
            is_ipv6: true,
        },
    ]
}

/// Baseline active firewall state for servers
/// An active ufw with `default_ufw_rules`, for tests.
#[cfg(test)]
pub fn default_active_ufw_state() -> FirewallOperationalState {
    let rules = default_ufw_rules();
    FirewallOperationalState::Active(FirewallStatusSummary {
        backend: FirewallBackend::Ufw,
        is_active: true,
        default_incoming: RuleAction::Deny,
        default_outgoing: RuleAction::Allow,
        default_forward: RuleAction::Deny,
        rules,
        raw_output: r#"Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), disabled (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
[ 1] 22/tcp                     ALLOW IN    Anywhere                   # OpenSSH Remote Administration
[ 2] 80/tcp                     ALLOW IN    Anywhere                   # Nginx HTTP Ingress
[ 3] 443/tcp                    ALLOW IN    Anywhere                   # Nginx HTTPS Ingress (TLS/SSL)
[ 4] 5432/tcp                   ALLOW IN    10.0.4.0/24                # PostgreSQL Database Internal Subnet
[ 5] 6379/tcp                   DENY IN     Anywhere                   # Block Public Redis Port
[ 6] 22/tcp (v6)                LIMIT IN    Anywhere (v6)              # SSH Brute-Force Rate Limit IPv6
"#
        .to_string(),
    })
}

/// Quick port toggle helper model
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickPortPreset {
    pub name: &'static str,
    pub port: u16,
    pub protocol: RuleProtocol,
    pub default_comment: &'static str,
}

pub fn common_quick_ports() -> &'static [QuickPortPreset] {
    &[
        QuickPortPreset {
            name: "SSH",
            port: 22,
            protocol: RuleProtocol::Tcp,
            default_comment: "OpenSSH Secure Shell",
        },
        QuickPortPreset {
            name: "HTTP",
            port: 80,
            protocol: RuleProtocol::Tcp,
            default_comment: "Web Server (HTTP)",
        },
        QuickPortPreset {
            name: "HTTPS",
            port: 443,
            protocol: RuleProtocol::Tcp,
            default_comment: "Secure Web Server (HTTPS)",
        },
        QuickPortPreset {
            name: "PostgreSQL",
            port: 5432,
            protocol: RuleProtocol::Tcp,
            default_comment: "PostgreSQL DB Cluster",
        },
        QuickPortPreset {
            name: "Redis",
            port: 6379,
            protocol: RuleProtocol::Tcp,
            default_comment: "Redis Key-Value Cache",
        },
        QuickPortPreset {
            name: "Docker Swarm",
            port: 2377,
            protocol: RuleProtocol::Tcp,
            default_comment: "Docker Cluster Management",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ufw_rules_consistency() {
        let rules = default_ufw_rules();
        assert_eq!(rules.len(), 6);
        assert_eq!(rules[0].port, "22");
        assert_eq!(rules[0].action, RuleAction::Allow);
        assert_eq!(rules[4].port, "6379");
        assert_eq!(rules[4].action, RuleAction::Deny);
    }

    #[test]
    fn test_firewall_operational_state() {
        let active = default_active_ufw_state();
        assert!(active.is_operational());
        assert_eq!(active.backend(), FirewallBackend::Ufw);
        assert_eq!(active.rules_count(), 6);

        let inactive = FirewallOperationalState::Inactive {
            backend: FirewallBackend::Ufw,
            reason: "UFW is disabled".into(),
            detected_binaries: vec!["/usr/sbin/ufw".into()],
            has_root: false,
        };
        assert!(!inactive.is_operational());
        assert_eq!(inactive.backend(), FirewallBackend::Ufw);
        assert_eq!(inactive.rules_count(), 0);
    }

    #[test]
    fn test_rule_display_port_proto() {
        let rule = FirewallRule {
            id: "test".into(),
            number: 1,
            action: RuleAction::Allow,
            direction: RuleDirection::Inbound,
            port: "8080".into(),
            protocol: RuleProtocol::Tcp,
            source: "Anywhere".into(),
            destination: "Anywhere".into(),
            comment: None,
            is_ipv6: false,
        };
        assert_eq!(rule.display_port_proto(), "8080/tcp");
    }

    #[test]
    fn test_correlate_port_firewall() {
        let active = default_active_ufw_state();

        // Port 22 is allowed in default rules (rule #1)
        let m22 = correlate_port_firewall(Some(&active), "22", "TCP");
        assert!(matches!(m22, PortFirewallMatch::Allowed { rule_number: 1, .. }));

        // Port 443 is allowed (rule #3)
        let m443 = correlate_port_firewall(Some(&active), "443", "TCP");
        assert!(matches!(m443, PortFirewallMatch::Allowed { rule_number: 3, .. }));

        // Port 6379 is denied (rule #5)
        let m6379 = correlate_port_firewall(Some(&active), "6379", "TCP");
        assert!(matches!(m6379, PortFirewallMatch::Denied { rule_number: 5, .. }));

        // Port 9999 has no rule and default incoming is Deny
        let m9999 = correlate_port_firewall(Some(&active), "9999", "TCP");
        assert_eq!(m9999, PortFirewallMatch::NoRule);

        // Inactive firewall
        let minactive = correlate_port_firewall(None, "22", "TCP");
        assert_eq!(minactive, PortFirewallMatch::Inactive);
    }
}
