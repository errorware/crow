use crate::host::{host_for, Host, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;
use super::models::{
    default_active_ufw_state, default_ufw_rules, FirewallBackend, FirewallOperationalState,
    FirewallRule, FirewallStatusSummary, RuleAction, RuleDirection, RuleProtocol,
};

/// Detects the firewall backend and operational status on a server.
pub fn detect_firewall_status(server: &ServerRecord) -> FirewallOperationalState {
    match host_for(server) {
        Some(host) => detect_firewall(host.as_ref()),
        // No transport yet: infer from the server's probed facts.
        None => detect_remote_firewall(server),
    }
}

/// Firewall tools present on a host, found in one round trip. sbin is added
/// to PATH because non-root users often lack it (and ufw lives there).
fn detect_binaries(host: &dyn Host) -> Vec<(&'static str, String)> {
    const TOOLS: [&str; 4] = ["ufw", "firewall-cmd", "nft", "iptables"];
    let script = r#"PATH="$PATH:/usr/sbin:/sbin"; for b in "$@"; do p=$(command -v "$b") && echo "$b $p"; done; true"#;
    let mut argv = vec!["sh", "-c", script, "crow-which"];
    argv.extend(TOOLS);
    let stdout = host.exec(&argv, DEFAULT_TIMEOUT).map(|o| o.stdout).unwrap_or_default();
    TOOLS
        .iter()
        .filter_map(|tool| {
            let path = stdout.lines().find_map(|l| l.strip_prefix(tool).and_then(|rest| rest.strip_prefix(' ')))?;
            Some((*tool, path.trim().to_string()))
        })
        .collect()
}

/// Reads the host's real firewall state. ufw is read as root (its rules
/// aren't readable otherwise). When Crow can't read or doesn't manage what's
/// there, it says so rather than showing a stand-in ruleset.
pub fn detect_firewall(host: &dyn Host) -> FirewallOperationalState {
    let found = detect_binaries(host);
    let has = |tool: &str| found.iter().any(|(t, _)| *t == tool);
    let detected_binaries: Vec<String> = found.iter().map(|(t, p)| format!("{t} ({p})")).collect();

    if has("ufw") {
        return match host.exec_privileged(&["ufw", "status", "numbered"], &[], DEFAULT_TIMEOUT) {
            Ok(out) if out.stdout.contains("Status: active") => match parse_ufw_status(&out.stdout) {
                Some(summary) => FirewallOperationalState::Active(summary),
                None => FirewallOperationalState::Unmanaged {
                    detected_binaries,
                    reason: "ufw is active but its status output could not be parsed.".into(),
                },
            },
            Ok(_) => FirewallOperationalState::Inactive {
                backend: FirewallBackend::Ufw,
                reason: "UFW is installed but inactive. System netfilter packet filtering is disabled.".into(),
                detected_binaries,
                has_root: true,
            },
            Err(e) => FirewallOperationalState::Inactive {
                backend: FirewallBackend::Ufw,
                reason: format!("ufw is installed, but Crow couldn't read its rules as root: {e}"),
                detected_binaries,
                has_root: false,
            },
        };
    }

    let reason = if has("firewall-cmd") {
        "firewalld is installed. Crow manages ufw rules only for now; firewalld zones aren't read yet."
    } else if has("nft") || has("iptables") {
        "Only raw nftables/iptables tools were found. Crow manages ufw rules; raw rule sets aren't read yet."
    } else {
        "No firewall tooling (ufw, firewalld, nftables, iptables) was found on this host."
    };
    FirewallOperationalState::Unmanaged { detected_binaries, reason: reason.into() }
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
