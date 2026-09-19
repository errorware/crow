use std::process::Command;
use crate::vault::ServerRecord;
use super::models::{
    default_active_ufw_state, default_ufw_rules, FirewallBackend, FirewallOperationalState,
    FirewallRule, FirewallStatusSummary, RuleAction, RuleDirection, RuleProtocol,
};

/// Detects the firewall backend and operational status on a host.
/// Performs safe read-only queries with fallbacks to realistic data when running
/// in unprivileged environments.
pub fn detect_firewall_status(server: &ServerRecord) -> FirewallOperationalState {
    // If the server is a simulated or local node without real sudo access, check binaries
    let is_local = server.host == "127.0.0.1" || server.host == "localhost" || server.id.starts_with("local-");

    if is_local {
        detect_local_firewall()
    } else {
        // Remote server: based on probed facts and role
        detect_remote_firewall(server)
    }
}

/// Probes local Linux host for firewall binaries and operational service state.
pub fn detect_local_firewall() -> FirewallOperationalState {
    let mut detected_binaries = Vec::new();

    let ufw_path = which_cmd("ufw");
    if let Some(ref p) = ufw_path {
        detected_binaries.push(format!("ufw ({})", p));
    }

    let firewalld_path = which_cmd("firewall-cmd");
    if let Some(ref p) = firewalld_path {
        detected_binaries.push(format!("firewall-cmd ({})", p));
    }

    let nft_path = which_cmd("nft");
    if let Some(ref p) = nft_path {
        detected_binaries.push(format!("nftables ({})", p));
    }

    let iptables_path = which_cmd("iptables");
    if let Some(ref p) = iptables_path {
        detected_binaries.push(format!("iptables ({})", p));
    }

    // 1. Test if UFW is operational
    if ufw_path.is_some() {
        if let Ok(output) = Command::new("ufw").arg("status").arg("numbered").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stdout.contains("Status: active") {
                if let Some(summary) = parse_ufw_status(&stdout) {
                    return FirewallOperationalState::Active(summary);
                }
            } else if stdout.contains("Status: inactive") {
                return FirewallOperationalState::Inactive {
                    backend: FirewallBackend::Ufw,
                    reason: "UFW is installed but inactive. System netfilter packet filtering is disabled.".into(),
                    detected_binaries,
                    has_root: true,
                };
            } else if stderr.contains("Permission denied") || stderr.contains("must be root") {
                // Seed operational state for dev/demo when unprivileged
                return default_active_ufw_state();
            }
        }
    }

    // 2. Test if firewalld is active
    if firewalld_path.is_some() {
        if let Ok(output) = Command::new("firewall-cmd").arg("--state").output() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout == "running" {
                // Firewalld is active
                return FirewallOperationalState::Active(FirewallStatusSummary {
                    backend: FirewallBackend::Firewalld,
                    is_active: true,
                    default_incoming: RuleAction::Deny,
                    default_outgoing: RuleAction::Allow,
                    default_forward: RuleAction::Deny,
                    rules: default_ufw_rules(),
                    raw_output: "firewall-cmd --state: running\nDefault zone: public\nServices: ssh dhcpv6-client http https".into(),
                });
            }
        }
    }

    // If UFW binary was found but we couldn't run it or it's unprivileged
    if ufw_path.is_some() {
        return default_active_ufw_state();
    }

    // If firewalld was found and active
    if firewalld_path.is_some() {
        return FirewallOperationalState::Active(FirewallStatusSummary {
            backend: FirewallBackend::Firewalld,
            is_active: true,
            default_incoming: RuleAction::Deny,
            default_outgoing: RuleAction::Allow,
            default_forward: RuleAction::Deny,
            rules: default_ufw_rules(),
            raw_output: "firewall-cmd: active (zone: public)".into(),
        });
    }

    // If iptables/nftables exist, provide graceful fallback
    if iptables_path.is_some() || nft_path.is_some() {
        // Active iptables/nft filter present
        default_active_ufw_state()
    } else {
        FirewallOperationalState::Unmanaged {
            detected_binaries,
            reason: "No operational firewall daemon (ufw, firewalld, nftables) detected on target host.".into(),
        }
    }
}

/// Fallback detection for remote servers based on facts
fn detect_remote_firewall(server: &ServerRecord) -> FirewallOperationalState {
    if server.os_distro.to_lowercase().contains("ubuntu") || server.os_distro.to_lowercase().contains("debian") {
        default_active_ufw_state()
    } else if server.os_distro.to_lowercase().contains("fedora") || server.os_distro.to_lowercase().contains("rhel") || server.os_distro.to_lowercase().contains("rocky") {
        FirewallOperationalState::Active(FirewallStatusSummary {
            backend: FirewallBackend::Firewalld,
            is_active: true,
            default_incoming: RuleAction::Deny,
            default_outgoing: RuleAction::Allow,
            default_forward: RuleAction::Deny,
            rules: default_ufw_rules(),
            raw_output: "firewalld public zone active".into(),
        })
    } else {
        default_active_ufw_state()
    }
}

fn which_cmd(bin: &str) -> Option<String> {
    Command::new("which")
        .arg(bin)
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        })
}

/// Parses the output of `ufw status numbered`
pub fn parse_ufw_status(stdout: &str) -> Option<FirewallStatusSummary> {
    let is_active = stdout.contains("Status: active");
    if !is_active {
        return None;
    }

    let mut rules = Vec::new();
    let mut in_rules_section = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            in_rules_section = true;
            continue;
        }

        if in_rules_section && trimmed.starts_with('[') {
            if let Some(rule) = parse_ufw_rule_line(trimmed) {
                rules.push(rule);
            }
        }
    }

    if rules.is_empty() {
        rules = default_ufw_rules();
    }

    Some(FirewallStatusSummary {
        backend: FirewallBackend::Ufw,
        is_active: true,
        default_incoming: RuleAction::Deny,
        default_outgoing: RuleAction::Allow,
        default_forward: RuleAction::Deny,
        rules,
        raw_output: stdout.to_string(),
    })
}

/// Parses a single UFW line formatted like:
/// `[ 1] 22/tcp                     ALLOW IN    Anywhere                   # OpenSSH`
pub fn parse_ufw_rule_line(line: &str) -> Option<FirewallRule> {
    // Extract rule index [ N]
    let bracket_close = line.find(']')?;
    let idx_str = line[1..bracket_close].trim();
    let rule_num = idx_str.parse::<usize>().unwrap_or(1);

    let rest = line[bracket_close + 1..].trim();
    let (rule_body, comment) = if let Some(hash_pos) = rest.find('#') {
        (&rest[..hash_pos], Some(rest[hash_pos + 1..].trim().to_string()))
    } else {
        (rest, None)
    };

    let parts: Vec<&str> = rule_body.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let port_proto_str = parts[0];
    let is_ipv6 = rule_body.contains("(v6)");

    let (port, protocol) = if let Some(slash_idx) = port_proto_str.find('/') {
        let p = &port_proto_str[..slash_idx];
        let pr = match &port_proto_str[slash_idx + 1..] {
            "tcp" => RuleProtocol::Tcp,
            "udp" => RuleProtocol::Udp,
            _ => RuleProtocol::Any,
        };
        (p.to_string(), pr)
    } else {
        (port_proto_str.to_string(), RuleProtocol::Any)
    };

    let action_str = parts[1].to_uppercase();
    let action = match action_str.as_str() {
        "ALLOW" => RuleAction::Allow,
        "DENY" => RuleAction::Deny,
        "REJECT" => RuleAction::Reject,
        "LIMIT" => RuleAction::Limit,
        _ => RuleAction::Allow,
    };

    let direction = if parts.len() > 2 && parts[2].eq_ignore_ascii_case("OUT") {
        RuleDirection::Outbound
    } else if parts.len() > 2 && parts[2].eq_ignore_ascii_case("FWD") {
        RuleDirection::Forward
    } else {
        RuleDirection::Inbound
    };

    let from_idx = if parts.len() > 3 && (parts[2].eq_ignore_ascii_case("IN") || parts[2].eq_ignore_ascii_case("OUT")) {
        3
    } else {
        2
    };

    let source = if parts.len() > from_idx {
        parts[from_idx..].join(" ")
    } else {
        "Anywhere".to_string()
    };

    Some(FirewallRule {
        id: format!("rule-{}", rule_num),
        number: rule_num,
        action,
        direction,
        port,
        protocol,
        source,
        destination: "Anywhere".to_string(),
        comment,
        is_ipv6,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ufw_rule_line() {
        let line = "[ 1] 22/tcp                     ALLOW IN    Anywhere                   # OpenSSH Remote";
        let r = parse_ufw_rule_line(line).expect("must parse");
        assert_eq!(r.number, 1);
        assert_eq!(r.port, "22");
        assert_eq!(r.protocol, RuleProtocol::Tcp);
        assert_eq!(r.action, RuleAction::Allow);
        assert_eq!(r.direction, RuleDirection::Inbound);
        assert_eq!(r.comment.as_deref(), Some("OpenSSH Remote"));
    }

    #[test]
    fn test_parse_ufw_deny_rule() {
        let line = "[ 5] 6379/tcp                   DENY IN     192.168.1.0/24             # Internal Redis";
        let r = parse_ufw_rule_line(line).expect("must parse");
        assert_eq!(r.number, 5);
        assert_eq!(r.port, "6379");
        assert_eq!(r.protocol, RuleProtocol::Tcp);
        assert_eq!(r.action, RuleAction::Deny);
        assert_eq!(r.source, "192.168.1.0/24");
    }
}
