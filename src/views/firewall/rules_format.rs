use super::models::{FirewallRule, RuleAction, RuleDirection, RuleProtocol};

/// Generates canonical `/etc/ufw/user.rules` content from active firewall rules.
pub fn generate_user_rules_content(rules: &[FirewallRule]) -> String {
    let mut out = String::new();
    out.push_str("# /etc/ufw/user.rules - Managed by Crow Unified Engine\n");
    out.push_str("# Auditable, reversible, source-controlled firewall rules.\n");
    out.push_str("*filter\n");
    out.push_str(":ufw-user-input - [0:0]\n");
    out.push_str(":ufw-user-output - [0:0]\n");
    out.push_str(":ufw-user-forward - [0:0]\n\n");
    out.push_str("### RULES ###\n");

    for (idx, rule) in rules.iter().enumerate() {
        let chain = match rule.direction {
            RuleDirection::Inbound => "ufw-user-input",
            RuleDirection::Outbound => "ufw-user-output",
            RuleDirection::Forward => "ufw-user-forward",
        };

        let mut line = format!("-A {}", chain);

        // Protocol
        match rule.protocol {
            RuleProtocol::Tcp => line.push_str(" -p tcp"),
            RuleProtocol::Udp => line.push_str(" -p udp"),
            RuleProtocol::Both => line.push_str(" -p tcp,udp"),
            RuleProtocol::Any => {}
        }

        // Port
        if !rule.port.is_empty() && rule.port != "any" && rule.port != "*" {
            line.push_str(&format!(" --dport {}", rule.port));
        }

        // Source IP/subnet
        if !rule.source.is_empty() && !rule.source.starts_with("Anywhere") && rule.source != "0.0.0.0/0" {
            line.push_str(&format!(" -s {}", rule.source));
        }

        // Destination IP/subnet
        if !rule.destination.is_empty() && !rule.destination.starts_with("Anywhere") && rule.destination != "0.0.0.0/0" {
            line.push_str(&format!(" -d {}", rule.destination));
        }

        // Action / Rate Limit
        match rule.action {
            RuleAction::Allow => line.push_str(" -j ACCEPT"),
            RuleAction::Deny => line.push_str(" -j DROP"),
            RuleAction::Reject => line.push_str(" -j REJECT"),
            RuleAction::Limit => line.push_str(" -m limit --limit 3/minute -j ACCEPT"),
        }

        // Comment
        if let Some(ref c) = rule.comment {
            if !c.trim().is_empty() {
                let clean_comment = c.replace('"', "").replace('\'', "");
                line.push_str(&format!(" -m comment --comment \"{}\"", clean_comment));
            }
        } else {
            line.push_str(&format!(" -m comment --comment \"rule_{}\"", idx + 1));
        }

        out.push_str(&line);
        out.push('\n');
    }

    out.push_str("\nCOMMIT\n");
    out
}

/// Parses canonical `/etc/ufw/user.rules` content into `Vec<FirewallRule>`.
pub fn parse_user_rules_content(content: &str) -> Vec<FirewallRule> {
    let mut rules = Vec::new();
    let mut counter = 1;

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("-A") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let chain = parts[1];
        let direction = if chain.contains("input") {
            RuleDirection::Inbound
        } else if chain.contains("output") {
            RuleDirection::Outbound
        } else if chain.contains("forward") {
            RuleDirection::Forward
        } else {
            RuleDirection::Inbound
        };

        let mut action = RuleAction::Allow;
        let mut protocol = RuleProtocol::Any;
        let mut port = "any".to_string();
        let mut source = "Anywhere".to_string();
        let mut destination = "Anywhere".to_string();
        let mut comment: Option<String> = None;
        let mut is_ipv6 = false;

        let mut i = 2;
        while i < parts.len() {
            match parts[i] {
                "-p" if i + 1 < parts.len() => {
                    let proto = parts[i + 1].to_lowercase();
                    if proto.contains("tcp") && proto.contains("udp") {
                        protocol = RuleProtocol::Both;
                    } else if proto == "tcp" {
                        protocol = RuleProtocol::Tcp;
                    } else if proto == "udp" {
                        protocol = RuleProtocol::Udp;
                    } else {
                        protocol = RuleProtocol::Any;
                    }
                    i += 2;
                }
                "--dport" if i + 1 < parts.len() => {
                    port = parts[i + 1].to_string();
                    i += 2;
                }
                "-s" if i + 1 < parts.len() => {
                    source = parts[i + 1].to_string();
                    if source.contains(':') {
                        is_ipv6 = true;
                    }
                    i += 2;
                }
                "-d" if i + 1 < parts.len() => {
                    destination = parts[i + 1].to_string();
                    if destination.contains(':') {
                        is_ipv6 = true;
                    }
                    i += 2;
                }
                "-j" if i + 1 < parts.len() => {
                    match parts[i + 1].to_uppercase().as_str() {
                        "ACCEPT" => {
                            if action != RuleAction::Limit {
                                action = RuleAction::Allow;
                            }
                        }
                        "DROP" => action = RuleAction::Deny,
                        "REJECT" => action = RuleAction::Reject,
                        _ => {}
                    }
                    i += 2;
                }
                "-m" if i + 1 < parts.len() => {
                    if parts[i + 1] == "limit" {
                        action = RuleAction::Limit;
                        i += 2;
                    } else if parts[i + 1] == "comment" {
                        i += 2;
                        if i < parts.len() && parts[i] == "--comment" && i + 1 < parts.len() {
                            let rest = parts[i + 1..].join(" ");
                            let trimmed_c = rest.trim_matches(|c| c == '"' || c == '\'');
                            comment = Some(trimmed_c.to_string());
                            break;
                        }
                    } else {
                        i += 2;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        rules.push(FirewallRule {
            id: format!("rule-{}", counter),
            number: counter,
            action,
            direction,
            port,
            protocol,
            source,
            destination,
            comment,
            is_ipv6,
            zone: None,
        });
        counter += 1;
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_serialization_and_parsing() {
        let rules = vec![
            FirewallRule {
                id: "rule-1".into(),
                number: 1,
                action: RuleAction::Allow,
                direction: RuleDirection::Inbound,
                port: "22".into(),
                protocol: RuleProtocol::Tcp,
                source: "Anywhere".into(),
                destination: "Anywhere".into(),
                comment: Some("OpenSSH Access".into()),
                is_ipv6: false,
                zone: None,
            },
            FirewallRule {
                id: "rule-2".into(),
                number: 2,
                action: RuleAction::Deny,
                direction: RuleDirection::Inbound,
                port: "6379".into(),
                protocol: RuleProtocol::Tcp,
                source: "Anywhere".into(),
                destination: "Anywhere".into(),
                comment: Some("Block Redis".into()),
                is_ipv6: false,
                zone: None,
            },
            FirewallRule {
                id: "rule-3".into(),
                number: 3,
                action: RuleAction::Limit,
                direction: RuleDirection::Inbound,
                port: "80".into(),
                protocol: RuleProtocol::Tcp,
                source: "192.168.1.0/24".into(),
                destination: "Anywhere".into(),
                comment: Some("Rate limited web".into()),
                is_ipv6: false,
                zone: None,
            },
        ];

        let content = generate_user_rules_content(&rules);
        assert!(content.contains("*filter"));
        assert!(content.contains("-A ufw-user-input -p tcp --dport 22 -j ACCEPT"));
        assert!(content.contains("-A ufw-user-input -p tcp --dport 6379 -j DROP"));
        assert!(content.contains("-A ufw-user-input -p tcp --dport 80 -s 192.168.1.0/24 -m limit --limit 3/minute -j ACCEPT"));
        assert!(content.contains("COMMIT"));

        let parsed = parse_user_rules_content(&content);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].port, "22");
        assert_eq!(parsed[0].action, RuleAction::Allow);
        assert_eq!(parsed[0].comment.as_deref(), Some("OpenSSH Access"));

        assert_eq!(parsed[1].port, "6379");
        assert_eq!(parsed[1].action, RuleAction::Deny);

        assert_eq!(parsed[2].port, "80");
        assert_eq!(parsed[2].action, RuleAction::Limit);
        assert_eq!(parsed[2].source, "192.168.1.0/24");
    }
}
