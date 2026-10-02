//! Bare nftables and iptables rulesets, read-only (ERR-78).
//!
//! Hosts without ufw or firewalld still filter with nft or iptables. Crow
//! reads `nft -j list ruleset` (or `iptables-save`), walks every chain hooked
//! on input (following jumps), and shows each rule that decides a packet's
//! fate: accept, drop or reject, by port and source. The chain policy is the
//! default. Rules that only match some traffic Crow can't place (loopback,
//! established connections, ICMP, marks) are shown but never count toward a
//! listening port's exposure. Crow never writes raw rulesets.

use serde_json::Value;

use super::models::{FirewallBackend, FirewallRule, FirewallStatusSummary, RuleAction, RuleDirection, RuleProtocol, ZoneScope};
use crate::host::{Host, DEFAULT_TIMEOUT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    V4,
    V6,
    Both,
    Other,
}

impl Family {
    fn from_nft(f: &str) -> Self {
        match f {
            "ip" => Self::V4,
            "ip6" => Self::V6,
            "inet" => Self::Both,
            _ => Self::Other,
        }
    }
    fn covers_v4(self) -> bool {
        matches!(self, Self::V4 | Self::Both)
    }
    fn covers_v6(self) -> bool {
        matches!(self, Self::V6 | Self::Both)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Verdict {
    Accept,
    Drop,
    Reject,
    Jump(String),
    Return,
    /// Logging, counting, marking: the packet carries on.
    None,
}

/// What a rule matches on. Anything in `other` is a condition Crow doesn't
/// place, so the rule can't speak for a port's exposure.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Conds {
    ports: Option<String>,
    proto: Option<String>,
    source: Option<String>,
    iface: Option<String>,
    ct: Option<String>,
    other: Vec<String>,
    limited: bool,
    ipv6_only: bool,
}

impl Conds {
    fn is_plain(&self) -> bool {
        self.iface.is_none() && self.ct.is_none() && self.other.is_empty()
    }
    fn describe(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(i) = &self.iface {
            out.push(format!("on {i}"));
        }
        if let Some(ct) = &self.ct {
            out.push(format!("{ct} connections"));
        }
        out.extend(self.other.iter().cloned());
        out
    }
}

#[derive(Clone, Debug)]
struct Rule {
    conds: Conds,
    verdict: Verdict,
    comment: Option<String>,
}

#[derive(Clone, Debug)]
struct Chain {
    family: Family,
    table: String,
    name: String,
    /// `ip filter input`, `INPUT (IPv4)`.
    label: String,
    input_hook: bool,
    priority: i64,
    policy: Option<RuleAction>,
    rules: Vec<Rule>,
}

#[derive(Clone, Debug, Default)]
struct Ruleset {
    chains: Vec<Chain>,
}

fn verdict_action(v: &str) -> Option<RuleAction> {
    match v.to_ascii_lowercase().as_str() {
        "accept" => Some(RuleAction::Allow),
        "drop" => Some(RuleAction::Deny),
        "reject" => Some(RuleAction::Reject),
        _ => None,
    }
}

// ---------- nftables ----------

/// The right-hand side of an nft match as text: `22`, `80,443`, `1000:2000`,
/// `10.0.0.0/8`, `established,related`.
fn nft_value(v: &Value) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(items) => items.iter().map(nft_value).collect::<Vec<_>>().join(","),
        Value::Object(o) => {
            if let Some(set) = o.get("set") {
                nft_value(set)
            } else if let Some(Value::Array(r)) = o.get("range") {
                r.iter().map(nft_value).collect::<Vec<_>>().join(":")
            } else if let Some(p) = o.get("prefix") {
                format!("{}/{}", nft_value(&p["addr"]), nft_value(&p["len"]))
            } else {
                v.to_string()
            }
        }
        other => other.to_string(),
    }
}

fn nft_match(m: &Value, conds: &mut Conds) {
    let left = &m["left"];
    let right = nft_value(&m["right"]);
    let negated = m["op"].as_str() == Some("!=");
    let raw = || format!("{} {} {right}", nft_left(left), m["op"].as_str().unwrap_or("=="));
    if let Some(p) = left.get("payload") {
        let (proto, field) = (p["protocol"].as_str().unwrap_or(""), p["field"].as_str().unwrap_or(""));
        match (proto, field) {
            ("tcp" | "udp" | "th", "dport") if !negated => {
                conds.ports = Some(right);
                if proto != "th" {
                    conds.proto = Some(proto.into());
                }
            }
            ("ip" | "ip6", "saddr") => {
                conds.source = Some(if negated { format!("not {right}") } else { right });
                conds.ipv6_only |= proto == "ip6";
            }
            ("ip", "protocol") | ("ip6", "nexthdr") if !negated && matches!(right.as_str(), "tcp" | "udp") => conds.proto = Some(right),
            _ => conds.other.push(raw()),
        }
    } else if let Some(meta) = left.get("meta") {
        match meta["key"].as_str().unwrap_or("") {
            "l4proto" if !negated && matches!(right.as_str(), "tcp" | "udp") => conds.proto = Some(right),
            "iifname" | "iif" => conds.iface = Some(if negated { format!("not {right}") } else { right }),
            "nfproto" if !negated && right == "ipv6" => conds.ipv6_only = true,
            "nfproto" if !negated && right == "ipv4" => {}
            _ => conds.other.push(raw()),
        }
    } else if left.get("ct").is_some_and(|ct| ct["key"].as_str() == Some("state")) {
        conds.ct = Some(if negated { format!("not {right}") } else { right });
    } else {
        conds.other.push(raw());
    }
}

/// `tcp dport`, `meta l4proto`, `ct state`: a match's left side as text.
fn nft_left(left: &Value) -> String {
    let obj = match left.as_object() {
        Some(o) => o,
        None => return nft_value(left),
    };
    obj.iter()
        .next()
        .map(|(kind, v)| match kind.as_str() {
            "payload" => format!("{} {}", v["protocol"].as_str().unwrap_or("?"), v["field"].as_str().unwrap_or("?")),
            "meta" | "ct" => format!("{kind} {}", v["key"].as_str().unwrap_or("?")),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

fn nft_rule(exprs: &[Value]) -> (Conds, Verdict) {
    let mut conds = Conds::default();
    let mut verdict = Verdict::None;
    for e in exprs {
        let Some((kind, body)) = e.as_object().and_then(|o| o.iter().next()) else { continue };
        match kind.as_str() {
            "match" => nft_match(body, &mut conds),
            "accept" => verdict = Verdict::Accept,
            "drop" => verdict = Verdict::Drop,
            "reject" => verdict = Verdict::Reject,
            "jump" | "goto" => verdict = Verdict::Jump(body["target"].as_str().unwrap_or_default().to_string()),
            "return" => verdict = Verdict::Return,
            "limit" => conds.limited = true,
            "counter" | "log" | "comment" => {}
            // An iptables-nft extension or anything else that filters.
            "xt" => conds.other.push(format!("iptables match {}", body["name"].as_str().unwrap_or("?"))),
            _ => {}
        }
    }
    (conds, verdict)
}

fn parse_nft(json: &str) -> Result<Ruleset, String> {
    let doc: Value = serde_json::from_str(json).map_err(|e| format!("nft's JSON couldn't be read: {e}"))?;
    let items = doc["nftables"].as_array().ok_or("nft's JSON has no \"nftables\" list")?;
    let mut set = Ruleset::default();
    for item in items {
        if let Some(c) = item.get("chain") {
            let family = c["family"].as_str().unwrap_or("");
            let (table, name) = (c["table"].as_str().unwrap_or("").to_string(), c["name"].as_str().unwrap_or("").to_string());
            set.chains.push(Chain {
                family: Family::from_nft(family),
                label: format!("{family} {table} {name}"),
                input_hook: c["hook"].as_str() == Some("input") && c["type"].as_str().unwrap_or("filter") == "filter",
                priority: c["prio"].as_i64().unwrap_or(0),
                policy: c["policy"].as_str().and_then(verdict_action),
                table,
                name,
                rules: Vec::new(),
            });
        } else if let Some(r) = item.get("rule") {
            let family = Family::from_nft(r["family"].as_str().unwrap_or(""));
            let (table, chain) = (r["table"].as_str().unwrap_or(""), r["chain"].as_str().unwrap_or(""));
            let (conds, verdict) = nft_rule(r["expr"].as_array().map(Vec::as_slice).unwrap_or(&[]));
            let comment = r["comment"].as_str().map(str::to_string);
            if let Some(c) = set.chains.iter_mut().find(|c| c.family == family && c.table == table && c.name == chain) {
                c.rules.push(Rule { conds, verdict, comment });
            }
        }
    }
    Ok(set)
}

// ---------- iptables ----------

/// Splits an `iptables-save` line into words, keeping quoted comments whole.
fn shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => quoted = !quoted,
            '\\' if quoted => cur.extend(chars.next()),
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    words.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

fn iptables_rule(words: &[String]) -> (Conds, Verdict, Option<String>) {
    let mut conds = Conds::default();
    let mut verdict = Verdict::None;
    let mut comment = None;
    let mut negate = false;
    let mut i = 0;
    let arg = |i: usize| words.get(i + 1).cloned().unwrap_or_default();
    while i < words.len() {
        let w = words[i].as_str();
        let not = std::mem::take(&mut negate);
        let mut takes_arg = true;
        match w {
            "!" => {
                negate = true;
                takes_arg = false;
            }
            "-p" | "--protocol" if !not => conds.proto = Some(arg(i)),
            "-s" | "--source" => conds.source = Some(if not { format!("not {}", arg(i)) } else { arg(i) }),
            "-i" | "--in-interface" => conds.iface = Some(if not { format!("not {}", arg(i)) } else { arg(i) }),
            "--dport" | "--destination-port" | "--dports" | "--destination-ports" if !not => conds.ports = Some(arg(i)),
            "--ctstate" | "--state" => conds.ct = Some(if not { format!("not {}", arg(i)) } else { arg(i).to_lowercase() }),
            "--comment" => comment = Some(arg(i)),
            "--limit" | "--limit-burst" => conds.limited = true,
            "-m" | "--match" | "--reject-with" | "--log-prefix" | "--log-level" => {}
            "-j" | "--jump" | "-g" | "--goto" => {
                let target = arg(i);
                verdict = match target.as_str() {
                    "ACCEPT" => Verdict::Accept,
                    "DROP" => Verdict::Drop,
                    "REJECT" => Verdict::Reject,
                    "RETURN" => Verdict::Return,
                    "LOG" | "NFLOG" | "ULOG" => Verdict::None,
                    _ => Verdict::Jump(target),
                };
            }
            other if other.starts_with('-') => {
                // An option Crow doesn't place: keep it with its argument.
                let value = words.get(i + 1).filter(|v| !v.starts_with('-') && *v != "!");
                takes_arg = value.is_some();
                conds.other.push(format!("{}{other}{}", if not { "! " } else { "" }, value.map(|v| format!(" {v}")).unwrap_or_default()));
            }
            _ => takes_arg = false,
        }
        i += if takes_arg { 2 } else { 1 };
    }
    (conds, verdict, comment)
}

/// One `iptables-save -t filter` (or ip6tables-save) output.
fn parse_iptables(text: &str, family: Family, set: &mut Ruleset) {
    let mut in_filter = false;
    let tag = if family == Family::V6 { "IPv6" } else { "IPv4" };
    for line in text.lines().map(str::trim) {
        if let Some(t) = line.strip_prefix('*') {
            in_filter = t == "filter";
        } else if !in_filter || line.starts_with('#') {
            continue;
        } else if let Some(decl) = line.strip_prefix(':') {
            let mut parts = decl.split_whitespace();
            let name = parts.next().unwrap_or_default().to_string();
            set.chains.push(Chain {
                family,
                table: "filter".into(),
                label: format!("{name} ({tag})"),
                input_hook: name == "INPUT",
                priority: 0,
                policy: parts.next().and_then(verdict_action),
                name,
                rules: Vec::new(),
            });
        } else if let Some(rest) = line.strip_prefix("-A ") {
            let words = shell_words(rest);
            let Some((chain, args)) = words.split_first() else { continue };
            let (mut conds, verdict, comment) = iptables_rule(args);
            conds.ipv6_only = family == Family::V6;
            if let Some(c) = set.chains.iter_mut().find(|c| c.family == family && &c.name == chain) {
                c.rules.push(Rule { conds, verdict, comment });
            }
        }
    }
}

// ---------- shared ----------

impl Conds {
    /// A rule reached through a jump also needs the jumping rule's matches.
    fn within(&self, outer: &Conds) -> Conds {
        let mut other = outer.other.clone();
        other.extend(self.other.iter().cloned());
        Conds {
            ports: self.ports.clone().or_else(|| outer.ports.clone()),
            proto: self.proto.clone().or_else(|| outer.proto.clone()),
            source: self.source.clone().or_else(|| outer.source.clone()),
            iface: self.iface.clone().or_else(|| outer.iface.clone()),
            ct: self.ct.clone().or_else(|| outer.ct.clone()),
            other,
            limited: self.limited || outer.limited,
            ipv6_only: self.ipv6_only || outer.ipv6_only,
        }
    }
}

/// Rules a packet arriving on input meets, in order, with jumps inlined.
/// `outer` holds the matches of the rules that jumped here.
fn walk(set: &Ruleset, chain: &Chain, outer: &Conds, via: &[String], out: &mut Vec<(Rule, String)>, depth: usize) {
    if depth > 8 {
        return;
    }
    for rule in &chain.rules {
        match &rule.verdict {
            Verdict::Jump(target) => {
                if let Some(next) = set.chains.iter().find(|c| c.family == chain.family && c.table == chain.table && &c.name == target) {
                    let mut path = via.to_vec();
                    path.push(target.clone());
                    walk(set, next, &rule.conds.within(outer), &path, out, depth + 1);
                }
            }
            // An unconditional return ends this chain.
            Verdict::Return if rule.conds == Conds::default() => return,
            Verdict::Accept | Verdict::Drop | Verdict::Reject => {
                out.push((Rule { conds: rule.conds.within(outer), ..rule.clone() }, via.join(" → ")));
            }
            Verdict::Return | Verdict::None => {}
        }
    }
}

fn to_summary(set: &Ruleset, backend: FirewallBackend, raw: String) -> FirewallStatusSummary {
    let mut inputs: Vec<&Chain> = set.chains.iter().filter(|c| c.input_hook).collect();
    inputs.sort_by_key(|c| c.priority);
    let mut rules = Vec::new();
    for chain in &inputs {
        let mut walked = Vec::new();
        walk(set, chain, &Conds::default(), &[chain.label.clone()], &mut walked, 0);
        for (rule, path) in walked {
            let c = &rule.conds;
            let n = rules.len() + 1;
            let action = match rule.verdict {
                Verdict::Accept if c.limited => RuleAction::Limit,
                Verdict::Accept => RuleAction::Allow,
                Verdict::Drop => RuleAction::Deny,
                _ => RuleAction::Reject,
            };
            let mut notes = c.describe();
            notes.extend(rule.comment.clone());
            notes.push(format!("chain {path}"));
            rules.push(FirewallRule {
                id: format!("raw-{n}"),
                number: n,
                action,
                direction: RuleDirection::Inbound,
                port: c.ports.clone().unwrap_or_default(),
                protocol: match c.proto.as_deref() {
                    Some("tcp") => RuleProtocol::Tcp,
                    Some("udp") => RuleProtocol::Udp,
                    _ => RuleProtocol::Any,
                },
                source: c.source.clone().unwrap_or_else(|| "Anywhere".into()),
                destination: "Anywhere".into(),
                comment: Some(notes.join(" · ")),
                is_ipv6: c.ipv6_only || chain.family == Family::V6,
                zone: Some(ZoneScope { name: chain.label.clone(), binding: c.describe().join(", "), counts_for_exposure: c.is_plain(), entry: None }),
            });
        }
    }

    let default_incoming = if inputs.iter().any(|c| matches!(c.policy, Some(RuleAction::Deny | RuleAction::Reject))) { RuleAction::Deny } else { RuleAction::Allow };
    let filters = |pick: fn(Family) -> bool| inputs.iter().any(|c| pick(c.family) && (c.policy == Some(RuleAction::Deny) || !c.rules.is_empty()));
    let notice = if inputs.is_empty() {
        Some("No chain is hooked on input: nothing filters incoming traffic.".to_string())
    } else if filters(Family::covers_v4) && !filters(Family::covers_v6) {
        Some("Only IPv4 is filtered: no input chain covers IPv6, so every port is open over IPv6.".to_string())
    } else if let Some(stacked) = [Family::covers_v4 as fn(Family) -> bool, Family::covers_v6]
        .iter()
        .map(|pick| inputs.iter().filter(|c| pick(c.family)).map(|c| c.label.as_str()).collect::<Vec<_>>())
        .find(|labels| labels.len() > 1)
    {
        Some(format!("{} input chains run one after another ({}); a packet any of them drops is dropped.", stacked.len(), stacked.join(", ")))
    } else {
        None
    };
    FirewallStatusSummary {
        backend,
        is_active: true,
        default_incoming,
        default_outgoing: RuleAction::Allow,
        default_forward: RuleAction::Deny,
        rules,
        raw_output: raw,
        notice,
    }
}

pub fn summarize_nft(json: &str) -> Result<FirewallStatusSummary, String> {
    Ok(to_summary(&parse_nft(json)?, FirewallBackend::Nftables, pretty_nft(json)))
}

pub fn summarize_iptables(v4: &str, v6: &str) -> FirewallStatusSummary {
    let mut set = Ruleset::default();
    parse_iptables(v4, Family::V4, &mut set);
    parse_iptables(v6, Family::V6, &mut set);
    let raw = if v6.trim().is_empty() { v4.to_string() } else { format!("{v4}\n# ---- ip6tables-save ----\n{v6}") };
    to_summary(&set, FirewallBackend::Iptables, raw)
}

/// The raw ruleset in nft's own syntax would be nicer, but the JSON is what
/// was read; pretty-print it so it's at least readable.
fn pretty_nft(json: &str) -> String {
    serde_json::from_str::<Value>(json).ok().and_then(|v| serde_json::to_string_pretty(&v).ok()).unwrap_or_else(|| json.to_string())
}

/// Reads the ruleset as root: nft first (it sees iptables-nft rules too),
/// then iptables-save when nft isn't there, can't do JSON, or has no input
/// chain while iptables does.
pub fn read(host: &dyn Host, has_nft: bool, has_iptables: bool) -> Result<FirewallStatusSummary, String> {
    let root = |argv: &[&str]| host.exec_privileged(argv, &[], DEFAULT_TIMEOUT).map(|o| o.stdout).map_err(|e| e.to_string());
    let mut nft_err = None;
    if has_nft {
        match root(&["nft", "-j", "list", "ruleset"]).and_then(|j| summarize_nft(&j)) {
            Ok(s) if !has_iptables || s.notice.as_deref() != Some("No chain is hooked on input: nothing filters incoming traffic.") => {
                // Show the ruleset as nft writes it, when it can.
                let text = root(&["nft", "list", "ruleset"]).unwrap_or_default();
                return Ok(FirewallStatusSummary { raw_output: if text.trim().is_empty() { s.raw_output.clone() } else { text }, ..s });
            }
            Ok(_) => {}
            Err(e) => nft_err = Some(e),
        }
    }
    if has_iptables {
        let v4 = root(&["iptables-save", "-t", "filter"])?;
        let v6 = root(&["ip6tables-save", "-t", "filter"]).unwrap_or_default();
        return Ok(summarize_iptables(&v4, &v6));
    }
    Err(nft_err.unwrap_or_else(|| "no nft or iptables to read".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::firewall::{correlate_port_firewall, FirewallOperationalState, PortFirewallMatch};

    /// `nft -j list ruleset` for Debian's stock /etc/nftables.conf plus
    /// the usual input rules.
    const NFT: &str = r#"{"nftables": [
      {"metainfo": {"version": "1.0.6", "json_schema_version": 1}},
      {"table": {"family": "inet", "name": "filter", "handle": 1}},
      {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
      {"chain": {"family": "inet", "table": "filter", "name": "web", "handle": 4}},
      {"chain": {"family": "inet", "table": "filter", "name": "output", "handle": 3, "type": "filter", "hook": "output", "prio": 0, "policy": "accept"}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 5, "expr": [{"match": {"op": "in", "left": {"ct": {"key": "state"}}, "right": ["established", "related"]}}, {"accept": null}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 6, "expr": [{"match": {"op": "==", "left": {"meta": {"key": "iifname"}}, "right": "lo"}}, {"accept": null}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 7, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "icmp", "field": "type"}}, "right": "echo-request"}}, {"accept": null}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 8, "comment": "admin", "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 22}}, {"counter": {"packets": 0, "bytes": 0}}, {"accept": null}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 9, "expr": [{"jump": {"target": "web"}}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 10, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "ip", "field": "saddr"}}, "right": {"prefix": {"addr": "10.0.0.0", "len": 8}}}}, {"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 5432}}, {"accept": null}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 11, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": {"range": [6000, 6007]}}}, {"reject": {"type": "tcp reset"}}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "web", "handle": 12, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": {"set": [80, 443]}}}, {"accept": null}]}},
      {"rule": {"family": "inet", "table": "filter", "chain": "output", "handle": 13, "expr": [{"accept": null}]}}
    ]}"#;

    #[test]
    fn nft_input_rules_by_port_and_source() {
        let s = summarize_nft(NFT).unwrap();
        assert_eq!(s.backend, FirewallBackend::Nftables);
        assert_eq!(s.default_incoming, RuleAction::Deny);
        assert_eq!(s.notice, None);
        let ports: Vec<(&str, RuleAction, &str)> = s.rules.iter().map(|r| (r.port.as_str(), r.action, r.source.as_str())).collect();
        assert_eq!(
            ports,
            vec![
                ("", RuleAction::Allow, "Anywhere"),
                ("", RuleAction::Allow, "Anywhere"),
                ("", RuleAction::Allow, "Anywhere"),
                ("22", RuleAction::Allow, "Anywhere"),
                ("80,443", RuleAction::Allow, "Anywhere"),
                ("5432", RuleAction::Allow, "10.0.0.0/8"),
                ("6000:6007", RuleAction::Reject, "Anywhere"),
            ],
            "output chain left out, jump to web inlined"
        );
        assert!(s.rules[0].comment.as_deref().unwrap().starts_with("established,related connections"));
        assert!(s.rules[3].comment.as_deref().unwrap().contains("admin"));
        assert!(s.rules[4].comment.as_deref().unwrap().contains("inet filter input → web"));
    }

    #[test]
    fn loopback_and_established_rules_dont_open_ports() {
        let state = FirewallOperationalState::Active(summarize_nft(NFT).unwrap());
        assert!(matches!(correlate_port_firewall(Some(&state), "22", "TCP"), PortFirewallMatch::Allowed { .. }));
        assert!(matches!(correlate_port_firewall(Some(&state), "443", "TCP"), PortFirewallMatch::Allowed { .. }));
        assert!(matches!(correlate_port_firewall(Some(&state), "6001", "TCP"), PortFirewallMatch::Denied { .. }));
        assert_eq!(correlate_port_firewall(Some(&state), "3306", "TCP"), PortFirewallMatch::NoRule, "ct state / lo / icmp accepts don't cover it");
    }

    #[test]
    fn ipv4_only_tables_warn_that_ipv6_is_open() {
        let json = r#"{"nftables": [
          {"chain": {"family": "ip", "table": "filter", "name": "INPUT", "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
          {"rule": {"family": "ip", "table": "filter", "chain": "INPUT", "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 22}}, {"xt": {"type": "match", "name": "comment"}}, {"accept": null}]}}
        ]}"#;
        let s = summarize_nft(json).unwrap();
        assert!(s.notice.unwrap().contains("open over IPv6"));
        assert!(!s.rules[0].zone.as_ref().unwrap().counts_for_exposure, "an iptables extension Crow can't read");
        let empty = summarize_nft(r#"{"nftables": [{"metainfo": {}}]}"#).unwrap();
        assert_eq!(empty.default_incoming, RuleAction::Allow);
        assert!(empty.notice.unwrap().contains("nothing filters"));
    }

    #[test]
    fn iptables_save_with_user_chains_and_comments() {
        let v4 = "# Generated by iptables-save v1.8.9\n*filter\n:INPUT DROP [0:0]\n:FORWARD DROP [0:0]\n:OUTPUT ACCEPT [0:0]\n:ssh-in - [0:0]\n\
-A INPUT -i lo -j ACCEPT\n\
-A INPUT -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT\n\
-A INPUT -p tcp -m tcp --dport 22 -j ssh-in\n\
-A INPUT -p tcp -m multiport --dports 80,443 -m comment --comment \"web traffic\" -j ACCEPT\n\
-A INPUT ! -s 192.168.0.0/16 -p udp -m udp --dport 161 -j DROP\n\
-A INPUT -p icmp -m icmp --icmp-type 8 -j ACCEPT\n\
-A INPUT -j LOG --log-prefix \"dropped: \"\n\
-A ssh-in -s 203.0.113.0/24 -j ACCEPT\n\
-A ssh-in -j RETURN\n\
-A ssh-in -j ACCEPT\n\
COMMIT\n";
        let v6 = "*filter\n:INPUT DROP [0:0]\n-A INPUT -p tcp --dport 22 -j ACCEPT\nCOMMIT\n";
        let s = summarize_iptables(v4, v6);
        assert_eq!(s.backend, FirewallBackend::Iptables);
        assert_eq!(s.notice, None, "both families filter");
        let rows: Vec<(&str, RuleAction, &str, bool)> = s.rules.iter().map(|r| (r.port.as_str(), r.action, r.source.as_str(), r.zone.as_ref().unwrap().counts_for_exposure)).collect();
        assert_eq!(
            rows,
            vec![
                ("", RuleAction::Allow, "Anywhere", false),
                ("", RuleAction::Allow, "Anywhere", false),
                ("22", RuleAction::Allow, "203.0.113.0/24", true),
                ("80,443", RuleAction::Allow, "Anywhere", true),
                ("161", RuleAction::Deny, "not 192.168.0.0/16", true),
                ("", RuleAction::Allow, "Anywhere", false),
                ("22", RuleAction::Allow, "Anywhere", true),
            ],
            "the jump is followed up to its RETURN; LOG decides nothing"
        );
        assert!(s.rules[3].comment.as_deref().unwrap().contains("web traffic"));
        assert!(s.rules[2].comment.as_deref().unwrap().contains("INPUT (IPv4) → ssh-in"));
        assert!(s.rules[6].is_ipv6);
    }

    #[test]
    fn quoted_words_stay_whole() {
        assert_eq!(shell_words(r#"INPUT -m comment --comment "a \"b\" c" -j ACCEPT"#), vec!["INPUT", "-m", "comment", "--comment", "a \"b\" c", "-j", "ACCEPT"]);
    }
}
