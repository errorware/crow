//! The exact `ufw` commands behind each Firewall screen action, as argument
//! lists (run as root through `Host::exec_privileged`, never through a shell).

use super::{FirewallStatusSummary, NewRuleState, RuleAction, RuleDirection, RuleProtocol};

pub type Argv = Vec<String>;

fn argv(parts: &[&str]) -> Argv {
    parts.iter().map(|s| s.to_string()).collect()
}

pub fn enable() -> Argv {
    argv(&["ufw", "--force", "enable"])
}

pub fn disable() -> Argv {
    argv(&["ufw", "disable"])
}

pub fn reload() -> Argv {
    argv(&["ufw", "reload"])
}

pub fn reset() -> Argv {
    argv(&["ufw", "--force", "reset"])
}

/// Deletes a rule by its number in `ufw status numbered`.
pub fn delete_numbered(number: usize) -> Argv {
    vec!["ufw".into(), "--force".into(), "delete".into(), number.to_string()]
}

fn proto_arg(proto: RuleProtocol) -> Option<&'static str> {
    match proto {
        RuleProtocol::Tcp => Some("tcp"),
        RuleProtocol::Udp => Some("udp"),
        RuleProtocol::Both | RuleProtocol::Any => None,
    }
}

fn action_arg(action: RuleAction) -> &'static str {
    match action {
        RuleAction::Allow => "allow",
        RuleAction::Deny => "deny",
        RuleAction::Reject => "reject",
        RuleAction::Limit => "limit",
    }
}

/// Quick-port toggle: removes the port's allow rule if there is one,
/// otherwise allows it inbound.
pub fn toggle_port(summary: Option<&FirewallStatusSummary>, port: u16, proto: RuleProtocol, label: &str) -> Argv {
    let port_str = port.to_string();
    let existing = summary.and_then(|s| s.rules.iter().find(|r| r.action == RuleAction::Allow && r.port == port_str));
    match existing {
        Some(rule) => delete_numbered(rule.number),
        None => {
            let target = match proto_arg(proto) {
                Some(p) => format!("{port}/{p}"),
                None => port_str,
            };
            vec!["ufw".into(), "allow".into(), target, "comment".into(), format!("{label} Service Ingress")]
        }
    }
}

/// A rule from the New Rule form, in ufw's full syntax:
/// `ufw <action> [in|out] from <src> to any [port <p>] [proto <p>] [comment <c>]`.
/// Returns an error for input ufw would reject.
pub fn new_rule(form: &NewRuleState) -> Result<Argv, String> {
    let port = form.port_input.trim();
    if !port.is_empty() && !port.split([':', ',']).all(|p| p.parse::<u16>().is_ok_and(|n| n > 0)) {
        return Err(format!("{port:?} is not a port, range (a:b) or list (a,b)"));
    }
    let source = form.source_input.trim();
    let source = if form.is_anywhere || source.is_empty() { "any" } else { source };
    if source.starts_with('-') || source.contains(char::is_whitespace) {
        return Err(format!("{source:?} is not an address or network"));
    }
    let mut cmd = vec!["ufw".to_string(), action_arg(form.action).to_string()];
    match form.direction {
        RuleDirection::Inbound => cmd.push("in".into()),
        RuleDirection::Outbound => cmd.push("out".into()),
        RuleDirection::Forward => return Err("forward rules are managed with `ufw route`, which Crow doesn't write yet".into()),
    }
    cmd.extend(["from".into(), source.into(), "to".into(), "any".into()]);
    if !port.is_empty() {
        cmd.extend(["port".into(), port.into()]);
    }
    if let Some(p) = proto_arg(form.protocol) {
        cmd.extend(["proto".into(), p.into()]);
    }
    let comment = form.comment_input.trim();
    if !comment.is_empty() {
        cmd.extend(["comment".into(), comment.into()]);
    }
    Ok(cmd)
}

/// How a command reads in toasts and the journal.
pub fn describe(cmd: &Argv) -> String {
    cmd.iter()
        .map(|a| if a.contains(' ') { format!("'{a}'") } else { a.clone() })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::firewall::default_active_ufw_state;
    use crate::views::firewall::FirewallOperationalState;

    fn form() -> NewRuleState {
        NewRuleState::default()
    }

    #[test]
    fn new_rule_builds_full_ufw_syntax() {
        let mut f = form();
        f.action = RuleAction::Allow;
        f.direction = RuleDirection::Inbound;
        f.protocol = RuleProtocol::Tcp;
        f.port_input = "5432".into();
        f.is_anywhere = false;
        f.source_input = "10.0.4.0/24".into();
        f.comment_input = "postgres from app tier".into();
        assert_eq!(
            new_rule(&f).unwrap(),
            argv(&["ufw", "allow", "in", "from", "10.0.4.0/24", "to", "any", "port", "5432", "proto", "tcp", "comment", "postgres from app tier"])
        );
        assert_eq!(describe(&new_rule(&f).unwrap()).split(' ').last(), Some("tier'"));
    }

    #[test]
    fn new_rule_rejects_bad_input_instead_of_passing_it_to_ufw() {
        let mut f = form();
        f.port_input = "22; rm -rf /".into();
        assert!(new_rule(&f).is_err());
        let mut f = form();
        f.port_input = "22".into();
        f.is_anywhere = false;
        f.source_input = "--dry-run".into();
        assert!(new_rule(&f).is_err());
        let mut f = form();
        f.port_input = "6000:6007".into();
        f.protocol = RuleProtocol::Both;
        assert!(new_rule(&f).unwrap().windows(2).any(|w| w == ["port", "6000:6007"]));
    }

    #[test]
    fn quick_port_deletes_existing_allow_or_adds_one() {
        let FirewallOperationalState::Active(summary) = default_active_ufw_state() else { panic!() };
        let existing = summary.rules.iter().find(|r| r.action == RuleAction::Allow).unwrap();
        let port: u16 = existing.port.parse().unwrap();
        assert_eq!(toggle_port(Some(&summary), port, RuleProtocol::Tcp, "x"), delete_numbered(existing.number));
        assert_eq!(toggle_port(Some(&summary), 8443, RuleProtocol::Tcp, "Alt HTTPS"), argv(&["ufw", "allow", "8443/tcp", "comment", "Alt HTTPS Service Ingress"]));
    }
}
