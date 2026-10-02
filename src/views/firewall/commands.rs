//! The exact `ufw` and `firewall-cmd` commands behind each Firewall screen
//! action, as argument lists (run as root through `Host::exec_privileged`,
//! never through a shell).

use super::models::ZoneEntry;
use super::{FirewallRule, FirewallStatusSummary, NewRuleState, RuleAction, RuleDirection, RuleProtocol};

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

// ---------- firewalld (ERR-77) ----------

/// One firewalld change, made to the running config and to the saved one.
/// There's no `--reload`: a reload would also drop any runtime-only changes
/// already there. `zone` `None` is the default zone.
pub fn firewalld(zone: Option<&str>, op: String) -> Vec<Argv> {
    let zone = zone.map(|z| format!("--zone={z}"));
    let mut runtime = vec!["firewall-cmd".to_string()];
    runtime.extend(zone.clone());
    runtime.push(op.clone());
    let mut permanent = vec!["firewall-cmd".to_string(), "--permanent".to_string()];
    permanent.extend(zone);
    permanent.push(op);
    vec![runtime, permanent]
}

pub fn firewalld_reload() -> Argv {
    argv(&["firewall-cmd", "--reload"])
}

/// Removes the zone entry a rule was read from: a whole service, a port, or
/// a rich rule.
pub fn firewalld_remove(rule: &FirewallRule) -> Result<Vec<Argv>, String> {
    let zone = rule.zone.as_ref().ok_or("this rule isn't from a firewalld zone")?;
    let op = match &zone.entry {
        Some(ZoneEntry::Service(s)) => format!("--remove-service={s}"),
        Some(ZoneEntry::Port(p)) => format!("--remove-port={p}"),
        Some(ZoneEntry::RichRule(r)) => format!("--remove-rich-rule={r}"),
        Some(ZoneEntry::Target) => {
            return Err(format!("zone {0} accepts everything through its target, not a rule; change it with firewall-cmd --permanent --zone={0} --set-target=default", zone.name))
        }
        None => return Err("Crow doesn't know which zone entry this rule came from".into()),
    };
    Ok(firewalld(Some(&zone.name), op))
}

/// Quick-port toggle: removes what allows the port, or opens it in the
/// default zone.
pub fn firewalld_toggle_port(summary: Option<&FirewallStatusSummary>, port: u16, proto: RuleProtocol) -> Result<Vec<Argv>, String> {
    match quick_port_rule(summary, port) {
        Some(rule) => firewalld_remove(rule),
        None => Ok(protos(proto).iter().flat_map(|p| firewalld(None, format!("--add-port={port}/{p}"))).collect()),
    }
}

/// The rule a quick-port toggle would remove: the first allow for exactly
/// that port that counts for exposure.
pub fn quick_port_rule(summary: Option<&FirewallStatusSummary>, port: u16) -> Option<&FirewallRule> {
    let port = port.to_string();
    summary?.rules.iter().find(|r| r.action == RuleAction::Allow && r.port == port && r.zone.as_ref().is_none_or(|z| z.counts_for_exposure))
}

fn protos(proto: RuleProtocol) -> &'static [&'static str] {
    match proto {
        RuleProtocol::Tcp => &["tcp"],
        RuleProtocol::Udp => &["udp"],
        RuleProtocol::Both | RuleProtocol::Any => &["tcp", "udp"],
    }
}

/// A rule from the New Rule form for firewalld: a plain port in the default
/// zone when it's allowed from anywhere, a rich rule otherwise. Comments
/// have nowhere to go in firewalld, so they're left out.
pub fn firewalld_new_rule(form: &NewRuleState) -> Result<Vec<Argv>, String> {
    let port = form.port_input.trim();
    if !port.is_empty() && !port.split([':', ',', '-']).all(|p| p.parse::<u16>().is_ok_and(|n| n > 0)) {
        return Err(format!("{port:?} is not a port, range (a:b) or list (a,b)"));
    }
    if form.direction != RuleDirection::Inbound {
        return Err("firewalld zones filter incoming traffic; outgoing and forwarded traffic need a policy, which Crow doesn't write yet".into());
    }
    let source = form.source_input.trim();
    let source = (!form.is_anywhere && !source.is_empty() && !source.eq_ignore_ascii_case("anywhere")).then_some(source);
    if let Some(s) = source {
        if s.starts_with('-') || s.contains(|c: char| c.is_whitespace() || c == '"') {
            return Err(format!("{s:?} is not an address or network"));
        }
    }
    let ports: Vec<String> = if port.is_empty() { Vec::new() } else { port.split(',').map(|p| p.replace(':', "-")).collect() };
    let verdict = match form.action {
        RuleAction::Allow => "accept",
        RuleAction::Deny => "drop",
        RuleAction::Reject => "reject",
        // ufw's limit is 6 new connections per 30 seconds per address.
        RuleAction::Limit => "accept limit value=\"12/m\"",
    };
    let mut ops = Vec::new();
    match (source, ports.is_empty()) {
        (None, true) => return Err("give a port, or a source to apply this to everything from".into()),
        (None, false) if form.action == RuleAction::Allow => {
            for p in &ports {
                for proto in protos(form.protocol) {
                    ops.push(format!("--add-port={p}/{proto}"));
                }
            }
        }
        _ => {
            let head = match source {
                Some(s) => format!("rule family=\"{}\" source address=\"{s}\"", if s.contains(':') { "ipv6" } else { "ipv4" }),
                None => "rule".to_string(),
            };
            if ports.is_empty() {
                ops.push(format!("--add-rich-rule={head} {verdict}"));
            }
            for p in &ports {
                for proto in protos(form.protocol) {
                    ops.push(format!("--add-rich-rule={head} port port=\"{p}\" protocol=\"{proto}\" {verdict}"));
                }
            }
        }
    }
    Ok(ops.into_iter().flat_map(|op| firewalld(None, op)).collect())
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

    #[test]
    fn firewalld_changes_hit_runtime_and_permanent() {
        let mut f = form();
        f.port_input = "8080,9000:9010".into();
        f.protocol = RuleProtocol::Tcp;
        let cmds = firewalld_new_rule(&f).unwrap();
        assert_eq!(cmds[0], argv(&["firewall-cmd", "--add-port=8080/tcp"]));
        assert_eq!(cmds[1], argv(&["firewall-cmd", "--permanent", "--add-port=8080/tcp"]));
        assert_eq!(cmds[3], argv(&["firewall-cmd", "--permanent", "--add-port=9000-9010/tcp"]));
        assert!(!cmds.iter().flatten().any(|a| a == "--reload"));

        f.is_anywhere = false;
        f.source_input = "10.0.4.0/24".into();
        f.port_input = "5432".into();
        assert_eq!(firewalld_new_rule(&f).unwrap()[0][1], r#"--add-rich-rule=rule family="ipv4" source address="10.0.4.0/24" port port="5432" protocol="tcp" accept"#);
        f.source_input = "10.0.4.0/24\" accept; rule".into();
        assert!(firewalld_new_rule(&f).is_err(), "no quotes smuggled into a rich rule");
        let mut f = form();
        f.direction = RuleDirection::Outbound;
        assert!(firewalld_new_rule(&f).is_err());
    }

    #[test]
    fn firewalld_removes_the_entry_a_rule_came_from() {
        use super::super::models::ZoneScope;
        let mut rule = crate::views::firewall::lockout::planned(RuleAction::Allow, "22", RuleProtocol::Tcp, "");
        let zone = |entry| Some(ZoneScope { name: "public".into(), binding: String::new(), counts_for_exposure: true, entry: Some(entry) });
        rule.zone = zone(ZoneEntry::Service("ssh".into()));
        assert_eq!(firewalld_remove(&rule).unwrap()[1], argv(&["firewall-cmd", "--permanent", "--zone=public", "--remove-service=ssh"]));
        rule.zone = zone(ZoneEntry::Port("6000-6007/tcp".into()));
        assert_eq!(firewalld_remove(&rule).unwrap()[0], argv(&["firewall-cmd", "--zone=public", "--remove-port=6000-6007/tcp"]));
        rule.zone = zone(ZoneEntry::Target);
        assert!(firewalld_remove(&rule).is_err());
    }
}
