//! The lock-out guard (ERR-77): before a firewall change runs, Crow plays it
//! against the rules it read and refuses, unless the user types CONFIRM, if
//! the SSH port it's connected through would stop being allowed.
//!
//! Established connections usually survive (conntrack), so the damage shows
//! up later: the next SSH login, Crow's included, is refused.

use super::models::{rule_covers_port, FirewallRule, FirewallStatusSummary, RuleAction, RuleDirection, RuleProtocol};

/// Who can reach the SSH port.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SshAccess {
    Open,
    /// Only from these sources.
    OnlyFrom(Vec<String>),
    Closed,
}

fn applies(rule: &FirewallRule, port: u16) -> bool {
    rule.direction == RuleDirection::Inbound
        && !matches!(rule.protocol, RuleProtocol::Udp)
        && !rule.zone.as_ref().is_some_and(|z| !z.counts_for_exposure)
        && rule_covers_port(&rule.port, &port.to_string(), port)
}

fn anywhere(source: &str) -> bool {
    matches!(source.trim(), "" | "Anywhere" | "Anywhere (v6)" | "any" | "0.0.0.0/0" | "::/0")
}

/// First match wins, as in ufw and in firewalld's deny-then-allow order.
/// A deny from one source only shuts that source out, so it's passed over.
pub fn ssh_access(summary: &FirewallStatusSummary, port: u16) -> SshAccess {
    let mut from = Vec::new();
    for rule in summary.rules.iter().filter(|r| applies(r, port)) {
        match (rule.action, anywhere(&rule.source)) {
            (RuleAction::Allow | RuleAction::Limit, true) => return SshAccess::Open,
            (RuleAction::Allow | RuleAction::Limit, false) => from.push(rule.source.clone()),
            (RuleAction::Deny | RuleAction::Reject, true) => break,
            (RuleAction::Deny | RuleAction::Reject, false) => {}
        }
    }
    if summary.default_incoming == RuleAction::Allow && from.is_empty() {
        return SshAccess::Open;
    }
    if from.is_empty() {
        SshAccess::Closed
    } else {
        SshAccess::OnlyFrom(from)
    }
}

/// Why a change needs CONFIRM, or `None` when it's safe. `after` is `None`
/// when Crow can't tell what the rules will be.
pub fn risk(before: Option<&FirewallStatusSummary>, after: Option<&FirewallStatusSummary>, port: u16) -> Option<String> {
    let Some(after) = after else {
        return Some(format!("Crow can't see which rules will apply after this, so it can't check that port {port}/tcp, the SSH port it's connected through, stays allowed."));
    };
    let before = before.map(|b| ssh_access(b, port)).unwrap_or(SshAccess::Open);
    match (before, ssh_access(after, port)) {
        (SshAccess::Open, SshAccess::OnlyFrom(from)) => Some(format!(
            "After this, port {port}/tcp (Crow's SSH connection) is only allowed from {}. If Crow doesn't connect from there, the next login is refused.",
            from.join(", ")
        )),
        (SshAccess::Open | SshAccess::OnlyFrom(_), SshAccess::Closed) => Some(format!(
            "This stops port {port}/tcp, the SSH port Crow is connected through, from being allowed. The next SSH login, Crow's included, would be refused."
        )),
        _ => None,
    }
}

/// The rules with `rule` gone, and everything else that goes with it: a
/// firewalld service or rich rule stands for several rows.
pub fn without(summary: &FirewallStatusSummary, rule: &FirewallRule) -> FirewallStatusSummary {
    let same = |r: &FirewallRule| match (&r.zone, &rule.zone) {
        (Some(a), Some(b)) if b.entry.is_some() => a.name == b.name && a.entry == b.entry,
        _ => r.id == rule.id,
    };
    FirewallStatusSummary { rules: summary.rules.iter().filter(|r| !same(r)).cloned().collect(), ..summary.clone() }
}

/// The rules with `added` in place: denies first (firewalld checks deny
/// rich rules before any allow), everything else appended (ufw appends).
pub fn with(summary: &FirewallStatusSummary, added: Vec<FirewallRule>, denies_first: bool) -> FirewallStatusSummary {
    let mut rules = summary.rules.clone();
    for rule in added {
        if denies_first && matches!(rule.action, RuleAction::Deny | RuleAction::Reject) {
            rules.insert(0, rule);
        } else {
            rules.push(rule);
        }
    }
    FirewallStatusSummary { rules, ..summary.clone() }
}

/// A rule as the guard sees it.
pub fn planned(action: RuleAction, port: &str, protocol: RuleProtocol, source: &str) -> FirewallRule {
    FirewallRule {
        id: "planned".into(),
        number: 0,
        action,
        direction: RuleDirection::Inbound,
        port: port.into(),
        protocol,
        source: if source.trim().is_empty() { "Anywhere".into() } else { source.into() },
        destination: "Anywhere".into(),
        comment: None,
        is_ipv6: false,
        zone: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::firewall::{default_active_ufw_state, FirewallOperationalState};

    fn ufw() -> FirewallStatusSummary {
        let FirewallOperationalState::Active(s) = default_active_ufw_state() else { panic!() };
        s
    }

    #[test]
    fn deleting_the_ssh_allow_is_a_lockout() {
        let s = ufw();
        assert_eq!(ssh_access(&s, 22), SshAccess::Open);
        let after: FirewallStatusSummary = s.rules.iter().filter(|r| r.port == "22").fold(s.clone(), |acc, r| without(&acc, r));
        assert!(risk(Some(&s), Some(&after), 22).unwrap().contains("would be refused"));
    }

    #[test]
    fn unrelated_changes_and_source_denies_are_safe() {
        let s = ufw();
        assert_eq!(risk(Some(&s), Some(&with(&s, vec![planned(RuleAction::Allow, "8443", RuleProtocol::Tcp, "")], false)), 22), None);
        let deny_one = with(&s, vec![planned(RuleAction::Deny, "22", RuleProtocol::Tcp, "203.0.113.9")], true);
        assert_eq!(risk(Some(&s), Some(&deny_one), 22), None, "shuts out one address, not Crow");
        let deny_all = with(&s, vec![planned(RuleAction::Deny, "22", RuleProtocol::Tcp, "")], true);
        assert!(risk(Some(&s), Some(&deny_all), 22).is_some());
        assert!(risk(Some(&s), Some(&deny_all), 2222).is_none(), "Crow connects on another port");
    }

    #[test]
    fn narrowing_to_a_source_and_unknown_outcomes_ask_first() {
        let s = FirewallStatusSummary { rules: vec![planned(RuleAction::Allow, "22", RuleProtocol::Tcp, "")], default_incoming: RuleAction::Deny, ..ufw() };
        let narrowed = FirewallStatusSummary { rules: vec![planned(RuleAction::Allow, "22", RuleProtocol::Tcp, "10.0.0.0/8")], ..s.clone() };
        assert!(risk(Some(&s), Some(&narrowed), 22).unwrap().contains("only allowed from 10.0.0.0/8"));
        assert!(risk(Some(&s), None, 22).is_some());
        let open_default = FirewallStatusSummary { rules: Vec::new(), default_incoming: RuleAction::Allow, ..s };
        assert_eq!(ssh_access(&open_default, 22), SshAccess::Open);
    }
}
