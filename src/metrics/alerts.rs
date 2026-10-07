//! Alert rules over history samples (ERR-85). An alert opens when its
//! condition first holds, stays open (updating level and detail) while it
//! holds, and resolves on the first sample that shows it no longer does.
//! A sample that can't tell (unreachable host, systemd not readable) leaves
//! an alert as it is.

use std::collections::{HashMap, HashSet};

use super::history::HistoryRow;

pub const DISK_WARN: f32 = 90.0;
pub const DISK_CRIT: f32 = 95.0;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Alert {
    pub id: String,
    pub server_id: String,
    /// "unreachable", "disk", "service:<unit>", or "host_key_age".
    pub kind: String,
    /// "WARN" or "CRIT".
    pub level: String,
    pub detail: String,
    pub opened_at: i64,
    pub last_seen: i64,
    pub resolved_at: Option<i64>,
    pub acknowledged_at: Option<i64>,
}

/// What one round of samples changes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AlertChanges {
    pub opened: Vec<Alert>,
    /// (id, level, detail, last_seen) of alerts still holding.
    pub updated: Vec<(String, String, String, i64)>,
    pub resolved: Vec<String>,
}

/// Conditions that hold in `row`, as (kind, level, detail), and the kinds
/// the row can vouch are clear.
fn conditions(row: &HistoryRow, was_down: bool) -> (Vec<(String, &'static str, String)>, Vec<String>) {
    let (mut hold, mut clear) = (Vec::new(), Vec::new());
    if !row.reachable {
        // One missed sample is a blip; two in a row is an outage.
        if was_down {
            hold.push(("unreachable".into(), "CRIT", row.reason.clone().unwrap_or_else(|| "unreachable".into())));
        }
        return (hold, clear);
    }
    clear.push("unreachable".into());
    // Rounded up, as df reports it.
    let mount = row.disk_mount.as_deref().unwrap_or("/");
    match row.disk {
        Some(d) if d >= DISK_CRIT => hold.push(("disk".into(), "CRIT", format!("disk {mount} at {:.0}%", d.ceil()))),
        Some(d) if d >= DISK_WARN => hold.push(("disk".into(), "WARN", format!("disk {mount} at {:.0}%", d.ceil()))),
        Some(_) => clear.push("disk".into()),
        None => {}
    }
    if let Some(failed) = &row.failed_services {
        for unit in failed {
            hold.push((format!("service:{unit}"), "WARN", format!("{unit} failed")));
        }
        clear.push("service:*".into());
    }
    (hold, clear)
}

/// Applies a round of samples to the open alerts. `down_before` holds the
/// servers whose previous sample was unreachable; the return includes the
/// set for next time.
pub fn evaluate(open: &[Alert], rows: &[HistoryRow], down_before: &HashSet<String>, now: i64) -> (AlertChanges, HashSet<String>) {
    let mut changes = AlertChanges::default();
    let mut down_now = HashSet::new();
    let by_key: HashMap<(&str, &str), &Alert> = open.iter().map(|a| ((a.server_id.as_str(), a.kind.as_str()), a)).collect();
    for row in rows {
        if !row.reachable {
            down_now.insert(row.server_id.clone());
        }
        let (hold, clear) = conditions(row, down_before.contains(&row.server_id));
        for (kind, level, detail) in &hold {
            match by_key.get(&(row.server_id.as_str(), kind.as_str())) {
                Some(a) => changes.updated.push((a.id.clone(), level.to_string(), detail.clone(), now)),
                None => changes.opened.push(Alert {
                    id: format!("alert-{}-{}-{now}", row.server_id, kind),
                    server_id: row.server_id.clone(),
                    kind: kind.clone(),
                    level: level.to_string(),
                    detail: detail.clone(),
                    opened_at: now,
                    last_seen: now,
                    resolved_at: None,
                    acknowledged_at: None,
                }),
            }
        }
        for a in open.iter().filter(|a| a.server_id == row.server_id) {
            let still = hold.iter().any(|(k, _, _)| *k == a.kind);
            let vouched = clear.iter().any(|c| *c == a.kind || (c == "service:*" && a.kind.starts_with("service:")));
            if !still && vouched {
                changes.resolved.push(a.id.clone());
            }
        }
    }
    (changes, down_now)
}

/// The host-key policy (ERR-98): a server whose host key is older than
/// `max_days` holds a WARN alert, resolved once its key is younger (it was
/// rotated) or the policy is off (`None`). `servers` are (id, the key
/// file's time); a key Crow hasn't read leaves its alert as it is.
pub fn key_age_changes(open: &[Alert], servers: &[(String, Option<i64>)], max_days: Option<i64>, now: i64) -> AlertChanges {
    const KIND: &str = "host_key_age";
    let mut changes = AlertChanges::default();
    for (server, mtime) in servers {
        let existing = open.iter().find(|a| a.server_id == *server && a.kind == KIND && a.resolved_at.is_none());
        let overdue = match (max_days, mtime) {
            (None, _) => None,
            (Some(_), None) => continue,
            (Some(max), Some(t)) => {
                let days = (now - t) / 86_400;
                (days > max).then(|| format!("host key {days} days old; the policy is {max}"))
            }
        };
        match (overdue, existing) {
            (Some(detail), Some(a)) => changes.updated.push((a.id.clone(), "WARN".into(), detail, now)),
            (Some(detail), None) => changes.opened.push(Alert {
                id: format!("alert-{server}-{KIND}-{now}"),
                server_id: server.clone(),
                kind: KIND.into(),
                level: "WARN".into(),
                detail,
                opened_at: now,
                last_seen: now,
                resolved_at: None,
                acknowledged_at: None,
            }),
            (None, Some(a)) => changes.resolved.push(a.id.clone()),
            (None, None) => {}
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(server: &str, reachable: bool, disk: Option<f32>, failed: Option<Vec<&str>>) -> HistoryRow {
        HistoryRow {
            server_id: server.into(),
            ts: 0,
            reachable,
            reason: (!reachable).then(|| "unreachable · Connection refused".into()),
            cpu: None,
            mem: None,
            disk,
            disk_mount: None,
            load1: None,
            failed_services: failed.map(|f| f.into_iter().map(String::from).collect()),
        }
    }

    fn apply(open: &mut Vec<Alert>, c: &AlertChanges) {
        open.retain(|a| !c.resolved.contains(&a.id));
        for (id, level, detail, seen) in &c.updated {
            if let Some(a) = open.iter_mut().find(|a| a.id == *id) {
                (a.level, a.detail, a.last_seen) = (level.clone(), detail.clone(), *seen);
            }
        }
        open.extend(c.opened.iter().cloned());
    }

    #[test]
    fn unreachable_needs_two_samples_in_a_row() {
        let (c, down) = evaluate(&[], &[row("a", false, None, None)], &HashSet::new(), 60);
        assert!(c.opened.is_empty(), "one miss is a blip");
        let (c, _) = evaluate(&[], &[row("a", false, None, None)], &down, 120);
        assert_eq!((c.opened[0].kind.as_str(), c.opened[0].level.as_str()), ("unreachable", "CRIT"));
        assert_eq!(c.opened[0].detail, "unreachable · Connection refused");
    }

    #[test]
    fn disk_escalates_then_resolves() {
        let mut open = Vec::new();
        let (c, _) = evaluate(&open, &[row("a", true, Some(91.0), None)], &HashSet::new(), 60);
        apply(&mut open, &c);
        assert_eq!(open[0].level, "WARN");
        let (c, _) = evaluate(&open, &[row("a", true, Some(96.0), None)], &HashSet::new(), 120);
        apply(&mut open, &c);
        assert_eq!((open.len(), open[0].level.as_str(), open[0].last_seen), (1, "CRIT", 120), "same alert, escalated");
        // Unreachable: can't tell about the disk, so it stays open.
        let (c, _) = evaluate(&open, &[row("a", false, None, None)], &HashSet::new(), 180);
        assert!(c.resolved.is_empty());
        let (c, _) = evaluate(&open, &[row("a", true, Some(40.0), None)], &HashSet::new(), 240);
        assert_eq!(c.resolved, vec![open[0].id.clone()]);
    }

    #[test]
    fn failed_services_open_per_unit_and_resolve_when_gone() {
        let mut open = Vec::new();
        let (c, _) = evaluate(&open, &[row("a", true, None, Some(vec!["backup.service", "cron.service"]))], &HashSet::new(), 60);
        apply(&mut open, &c);
        assert_eq!(open.len(), 2);
        let (c, _) = evaluate(&open, &[row("a", true, None, Some(vec!["cron.service"]))], &HashSet::new(), 120);
        assert_eq!(c.resolved.len(), 1);
        assert!(c.resolved[0].contains("backup.service"));
        // systemd unreadable this time: nothing resolves.
        let (c, _) = evaluate(&open, &[row("a", true, None, None)], &HashSet::new(), 180);
        assert!(c.resolved.is_empty());
    }

    #[test]
    fn host_keys_past_the_policy_warn_until_rotated() {
        let day = 86_400;
        let now = 1_000 * day;
        let servers = vec![("old".to_string(), Some(now - 400 * day)), ("new".to_string(), Some(now - 10 * day)), ("unread".to_string(), None)];
        let c = key_age_changes(&[], &servers, Some(365), now);
        assert_eq!(c.opened.len(), 1);
        assert_eq!((c.opened[0].server_id.as_str(), c.opened[0].level.as_str()), ("old", "WARN"));
        assert_eq!(c.opened[0].detail, "host key 400 days old; the policy is 365");
        // Still overdue a day later: updated, not opened again.
        let c2 = key_age_changes(&c.opened, &servers, Some(365), now + day);
        assert!(c2.opened.is_empty() && c2.updated.len() == 1);
        // Rotated: the key file is new, the alert resolves.
        let rotated = vec![("old".to_string(), Some(now))];
        assert_eq!(key_age_changes(&c.opened, &rotated, Some(365), now + day).resolved, vec![c.opened[0].id.clone()]);
        // Policy off: resolves too; an unread key changes nothing.
        assert_eq!(key_age_changes(&c.opened, &servers, None, now).resolved.len(), 1);
        assert_eq!(key_age_changes(&c.opened, &[("old".to_string(), None)], Some(365), now), AlertChanges::default());
    }
}
