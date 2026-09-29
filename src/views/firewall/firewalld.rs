//! firewalld, read-only (ERR-76): zones as Crow's rule model.
//!
//! What's read is the runtime config, which is what's enforced. When the
//! permanent config differs, the summary says so: those runtime changes go
//! away on the next reload or reboot.

use std::collections::HashMap;

use super::models::{FirewallBackend, FirewallRule, FirewallStatusSummary, RuleAction, RuleDirection, RuleProtocol, ZoneScope};
use crate::host::{Host, DEFAULT_TIMEOUT};

/// One zone from `firewall-cmd --list-all-zones`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Zone {
    pub name: String,
    pub is_default: bool,
    pub active: bool,
    pub target: String,
    pub interfaces: Vec<String>,
    pub sources: Vec<String>,
    pub services: Vec<String>,
    /// (port or range "a-b", protocol)
    pub ports: Vec<(String, String)>,
    pub rich_rules: Vec<String>,
}

pub fn parse_zones(text: &str) -> Vec<Zone> {
    let mut zones: Vec<Zone> = Vec::new();
    let mut in_rich = false;
    for line in text.lines() {
        if line.trim().is_empty() {
            in_rich = false;
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            // "name" or "name (default, active)"
            let (name, flags) = match line.split_once(" (") {
                Some((n, f)) => (n.trim(), f.trim_end_matches(')')),
                None => (line.trim(), ""),
            };
            zones.push(Zone {
                name: name.to_string(),
                is_default: flags.split(',').any(|f| f.trim() == "default"),
                active: flags.split(',').any(|f| f.trim() == "active"),
                ..Default::default()
            });
            in_rich = false;
            continue;
        }
        let Some(zone) = zones.last_mut() else { continue };
        let trimmed = line.trim();
        if in_rich && trimmed.starts_with("rule") {
            zone.rich_rules.push(trimmed.to_string());
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else { continue };
        let words = || value.split_whitespace().map(str::to_string).collect::<Vec<_>>();
        in_rich = false;
        match key {
            "target" => zone.target = value.trim().to_string(),
            "interfaces" => zone.interfaces = words(),
            "sources" => zone.sources = words(),
            "services" => zone.services = words(),
            "ports" => zone.ports = value.split_whitespace().filter_map(split_port).collect(),
            "rich rules" => in_rich = true,
            _ => {}
        }
    }
    zones
}

fn split_port(p: &str) -> Option<(String, String)> {
    let (port, proto) = p.split_once('/')?;
    Some((port.to_string(), proto.to_string()))
}

/// Ports and included services of each service, from a run of
/// `firewall-cmd --info-service=NAME` outputs each preceded by `@@svc NAME`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServiceInfo {
    pub ports: Vec<(String, String)>,
    pub includes: Vec<String>,
}

pub fn parse_service_infos(text: &str) -> HashMap<String, ServiceInfo> {
    let mut out: HashMap<String, ServiceInfo> = HashMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("@@svc ") {
            current = Some(name.trim().to_string());
            out.entry(name.trim().to_string()).or_default();
            continue;
        }
        let Some(name) = &current else { continue };
        let Some((key, value)) = line.trim().split_once(':') else { continue };
        let info = out.get_mut(name).expect("inserted above");
        match key {
            "ports" => info.ports = value.split_whitespace().filter_map(split_port).collect(),
            "includes" => info.includes = value.split_whitespace().map(str::to_string).collect(),
            _ => {}
        }
    }
    out
}

/// A service's ports, including those of the services it includes.
fn service_ports(name: &str, infos: &HashMap<String, ServiceInfo>, seen: &mut Vec<String>) -> Vec<(String, String)> {
    if seen.iter().any(|s| s == name) {
        return Vec::new();
    }
    seen.push(name.to_string());
    let Some(info) = infos.get(name) else { return Vec::new() };
    let mut ports = info.ports.clone();
    for inc in &info.includes {
        ports.extend(service_ports(inc, infos, seen));
    }
    ports
}

/// The parts of a rich rule Crow understands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichRule {
    pub source: Option<String>,
    pub port: Option<(String, String)>,
    pub service: Option<String>,
    pub action: RuleAction,
    pub ipv6: bool,
}

/// `rule family="ipv4" source address="10.0.0.0/24" port port="5432"
/// protocol="tcp" accept`. `None` for rules without a plain verdict
/// (logging only, forward-port, mark, ...), which are shown raw.
pub fn parse_rich_rule(rule: &str) -> Option<RichRule> {
    let attr = |key: &str| -> Option<String> {
        let start = rule.find(&format!("{key}=\""))? + key.len() + 2;
        let end = rule[start..].find('"')? + start;
        Some(rule[start..end].to_string())
    };
    let words: Vec<&str> = rule.split_whitespace().collect();
    // The verdict is a bare word, possibly followed by a limit clause.
    let action = words.iter().rev().find_map(|w| match *w {
        "accept" => Some(RuleAction::Allow),
        "reject" => Some(RuleAction::Reject),
        "drop" => Some(RuleAction::Deny),
        _ => None,
    })?;
    let action = if action == RuleAction::Allow && rule.contains(" limit value=") { RuleAction::Limit } else { action };
    let source = rule.contains("source ").then(|| attr("address")).flatten().map(|a| if rule.contains("source NOT ") { format!("not {a}") } else { a });
    let port = rule.contains(" port port=").then(|| Some((attr("port")?, attr("protocol")?))).flatten();
    let service = rule.contains("service name=").then(|| attr("name")).flatten();
    Some(RichRule { source, port, service, action, ipv6: attr("family").as_deref() == Some("ipv6") })
}

/// Active zones (bound to an interface or source) and the default zone, as
/// rules. `permanent_differs` notes runtime-only changes.
pub fn zones_to_summary(zones: &[Zone], services: &HashMap<String, ServiceInfo>, raw: String, permanent_differs: bool) -> FirewallStatusSummary {
    let mut rules = Vec::new();
    let mut n = 0;
    let mut push = |rules: &mut Vec<FirewallRule>, action: RuleAction, port: String, proto: &str, source: String, comment: String, zone: &ZoneScope, ipv6: bool| {
        n += 1;
        rules.push(FirewallRule {
            id: format!("fwd-{n}"),
            number: n,
            action,
            direction: RuleDirection::Inbound,
            // Ranges use ufw's "a:b", which the rest of Crow understands.
            port: port.replace('-', ":"),
            protocol: match proto {
                "tcp" => RuleProtocol::Tcp,
                "udp" => RuleProtocol::Udp,
                _ => RuleProtocol::Any,
            },
            source,
            destination: "Anywhere".into(),
            comment: Some(comment),
            is_ipv6: ipv6,
            zone: Some(zone.clone()),
        });
    };
    let default_zone = zones.iter().find(|z| z.is_default);
    for z in zones.iter().filter(|z| z.is_default || !z.interfaces.is_empty() || !z.sources.is_empty()) {
        let binding = match (z.sources.is_empty(), z.interfaces.is_empty()) {
            (false, _) => format!("from {}", z.sources.join(", ")),
            (true, false) => format!("on {}", z.interfaces.join(", ")),
            (true, true) => "every interface not in another zone".to_string(),
        };
        // What a listening port is exposed through: the default zone (it
        // takes every unassigned interface) and zones bound to sources.
        // An interface-only zone like docker's covers that interface only.
        let scope = ZoneScope { name: z.name.clone(), binding: binding.clone(), counts_for_exposure: z.is_default || !z.sources.is_empty() };
        let source = if z.sources.is_empty() { "Anywhere".to_string() } else { z.sources.join(", ") };
        if z.target == "ACCEPT" {
            push(&mut rules, RuleAction::Allow, String::new(), "", source.clone(), format!("zone {} accepts all traffic ({binding})", z.name), &scope, false);
        }
        for svc in &z.services {
            let ports = service_ports(svc, services, &mut Vec::new());
            if ports.is_empty() {
                push(&mut rules, RuleAction::Allow, String::new(), "", source.clone(), format!("service {svc} (ports not known) · zone {}", z.name), &scope, false);
            }
            for (port, proto) in ports {
                push(&mut rules, RuleAction::Allow, port, &proto, source.clone(), format!("service {svc} · zone {}", z.name), &scope, false);
            }
        }
        for (port, proto) in &z.ports {
            push(&mut rules, RuleAction::Allow, port.clone(), proto, source.clone(), format!("port · zone {}", z.name), &scope, false);
        }
        for raw_rule in &z.rich_rules {
            match parse_rich_rule(raw_rule) {
                Some(r) => {
                    let src = r.source.clone().unwrap_or_else(|| source.clone());
                    match (&r.port, &r.service) {
                        (Some((port, proto)), _) => push(&mut rules, r.action, port.clone(), proto, src, format!("rich rule · zone {}", z.name), &scope, r.ipv6),
                        (None, Some(svc)) => {
                            for (port, proto) in service_ports(svc, services, &mut Vec::new()) {
                                push(&mut rules, r.action, port, &proto, src.clone(), format!("rich rule, service {svc} · zone {}", z.name), &scope, r.ipv6);
                            }
                        }
                        (None, None) => push(&mut rules, r.action, String::new(), "", src, format!("rich rule · zone {}", z.name), &scope, r.ipv6),
                    }
                }
                // Shown, not interpreted: it doesn't take part in correlation.
                None => push(&mut rules, RuleAction::Allow, String::new(), "", source.clone(), format!("rich rule (not interpreted): {raw_rule}"), &ZoneScope { counts_for_exposure: false, ..scope.clone() }, false),
            }
        }
    }
    let default_incoming = match default_zone.map(|z| z.target.as_str()) {
        Some("ACCEPT") => RuleAction::Allow,
        Some("DROP") => RuleAction::Deny,
        // "default" and "%%REJECT%%" both reject what no rule allows.
        _ => RuleAction::Reject,
    };
    FirewallStatusSummary {
        backend: FirewallBackend::Firewalld,
        is_active: true,
        default_incoming,
        default_outgoing: RuleAction::Allow,
        default_forward: RuleAction::Deny,
        rules,
        raw_output: raw,
        notice: permanent_differs.then(|| "The running firewall differs from its saved (permanent) config: a reload or reboot will drop the runtime-only changes.".to_string()),
    }
}

/// Runs a read-only firewall-cmd query as the connected user, then as root:
/// polkit lets a desktop user query, but over SSH firewalld's D-Bus policy
/// usually wants root.
fn query(host: &dyn Host, argv: &[&str]) -> Result<String, String> {
    host.exec(argv, DEFAULT_TIMEOUT)
        .or_else(|_| host.exec_privileged(argv, &[], DEFAULT_TIMEOUT))
        .map(|o| o.stdout)
        .map_err(|e| e.to_string())
}

/// Whether the running zones say something the saved ones don't. The
/// "(active)" flag and interface bindings are left out: NetworkManager binds
/// interfaces to zones at runtime, so they're never in the permanent config.
fn runtime_only_changes(runtime: &[Zone], permanent: &[Zone]) -> bool {
    let strip = |zs: &[Zone]| zs.iter().map(|z| Zone { active: false, interfaces: Vec::new(), ..z.clone() }).collect::<Vec<_>>();
    strip(runtime) != strip(permanent)
}

/// Reads firewalld's runtime config, the services it uses, and whether the
/// permanent config differs.
pub fn read(host: &dyn Host) -> Result<FirewallStatusSummary, String> {
    let runtime = query(host, &["firewall-cmd", "--list-all-zones"])?;
    let zones = parse_zones(&runtime);
    let permanent = query(host, &["firewall-cmd", "--permanent", "--list-all-zones"]);
    let permanent_differs = permanent.map(|p| runtime_only_changes(&zones, &parse_zones(&p))).unwrap_or(false);

    // Services used by bound zones, and whatever they include, two rounds.
    let mut wanted: Vec<String> = zones.iter().flat_map(|z| z.services.iter().cloned()).collect();
    wanted.extend(zones.iter().flat_map(|z| z.rich_rules.iter()).filter_map(|r| parse_rich_rule(r)?.service));
    let mut infos = HashMap::new();
    for _ in 0..2 {
        wanted.sort();
        wanted.dedup();
        wanted.retain(|s| !infos.contains_key(s));
        if wanted.is_empty() {
            break;
        }
        let script = r#"for s in "$@"; do echo "@@svc $s"; firewall-cmd --info-service="$s"; done; true"#;
        let mut argv = vec!["sh", "-c", script, "crow-fwd"];
        argv.extend(wanted.iter().map(String::as_str));
        let out = query(host, &argv).unwrap_or_default();
        let parsed = parse_service_infos(&out);
        wanted = parsed.values().flat_map(|i| i.includes.clone()).collect();
        infos.extend(parsed);
    }
    Ok(zones_to_summary(&zones, &infos, runtime, permanent_differs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::firewall::models::{correlate_port_firewall, FirewallOperationalState, PortFirewallMatch};

    const FEDORA: &str = include_str!("fixtures/firewalld_list_all_zones.txt");

    #[test]
    fn parses_a_real_fedora_listing() {
        let zones = parse_zones(FEDORA);
        assert_eq!(zones.len(), 13);
        let ws = zones.iter().find(|z| z.name == "FedoraWorkstation").unwrap();
        assert!(ws.is_default && ws.active);
        assert_eq!(ws.target, "default");
        assert_eq!(ws.interfaces, vec!["wlo1"]);
        assert_eq!(ws.services, vec!["dhcpv6-client", "samba-client", "ssh"]);
        assert_eq!(ws.ports, vec![("1025-65535".to_string(), "udp".to_string()), ("1025-65535".to_string(), "tcp".to_string())]);
        let docker = zones.iter().find(|z| z.name == "docker").unwrap();
        assert!(docker.active && !docker.is_default);
        assert_eq!(docker.target, "ACCEPT");
    }

    #[test]
    fn services_follow_includes() {
        let infos = parse_service_infos("@@svc ssh\n  ports: 22/tcp\n  includes: \n@@svc samba-client\n  ports: 138/udp\n  includes: netbios-ns\n@@svc netbios-ns\n  ports: 137/udp\n");
        assert_eq!(service_ports("samba-client", &infos, &mut Vec::new()), vec![("138".into(), "udp".into()), ("137".into(), "udp".into())]);
    }

    #[test]
    fn rich_rules() {
        let r = parse_rich_rule(r#"rule family="ipv4" source address="10.0.0.0/24" port port="5432" protocol="tcp" accept"#).unwrap();
        assert_eq!((r.source.as_deref(), r.port.clone(), r.action), (Some("10.0.0.0/24"), Some(("5432".into(), "tcp".into())), RuleAction::Allow));
        let r = parse_rich_rule(r#"rule family="ipv4" source address="203.0.113.9" reject"#).unwrap();
        assert_eq!((r.source.as_deref(), r.action), (Some("203.0.113.9"), RuleAction::Reject));
        let r = parse_rich_rule(r#"rule service name="http" accept limit value="10/m""#).unwrap();
        assert_eq!((r.service.as_deref(), r.action), (Some("http"), RuleAction::Limit));
        assert!(parse_rich_rule(r#"rule family="ipv4" forward-port port="80" protocol="tcp" to-port="8080""#).is_none());
    }

    #[test]
    fn exposure_comes_from_the_default_zone_not_dockers() {
        let zones = parse_zones(FEDORA);
        let infos = parse_service_infos("@@svc ssh\n  ports: 22/tcp\n@@svc dhcpv6-client\n  ports: 546/udp\n@@svc samba-client\n  ports: 138/udp\n");
        let state = FirewallOperationalState::Active(zones_to_summary(&zones, &infos, FEDORA.into(), false));
        assert!(matches!(correlate_port_firewall(Some(&state), "22", "TCP"), PortFirewallMatch::Allowed { .. }));
        assert!(matches!(correlate_port_firewall(Some(&state), "8080", "TCP"), PortFirewallMatch::Allowed { .. }), "Fedora Workstation opens 1025-65535");
        // docker0 accepts everything, but that says nothing about wlo1.
        assert_eq!(correlate_port_firewall(Some(&state), "443", "TCP"), PortFirewallMatch::NoRule);
        let FirewallOperationalState::Active(s) = state else { unreachable!() };
        assert_eq!(s.default_incoming, RuleAction::Reject);
        assert!(s.rules.iter().any(|r| r.comment.as_deref().is_some_and(|c| c.contains("zone docker accepts all traffic (on docker0)"))));
    }

    #[test]
    fn interface_bindings_are_not_runtime_only_changes() {
        let runtime = parse_zones(FEDORA);
        // What `--permanent --list-all-zones` printed on the same machine:
        // no "(active)" and no interfaces, as NetworkManager binds them.
        let permanent = FEDORA.replace(" (default, active)", " (default)").replace("docker (active)", "docker").replace("interfaces: wlo1", "interfaces: ").replace("interfaces: docker0", "interfaces: ");
        assert!(!runtime_only_changes(&runtime, &parse_zones(&permanent)));
        let opened = FEDORA.replace("services: dhcpv6-client samba-client ssh", "services: dhcpv6-client samba-client ssh http");
        assert!(runtime_only_changes(&parse_zones(&opened), &parse_zones(&permanent)), "a service added at runtime only");
    }

    #[test]
    fn runtime_only_changes_are_called_out() {
        let s = zones_to_summary(&[], &HashMap::new(), String::new(), true);
        assert!(s.notice.unwrap().contains("reload or reboot"));
    }
}
