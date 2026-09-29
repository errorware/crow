//! Alert rules over history samples (ERR-85). An alert opens when its
//! condition first holds, stays open (updating level and detail) while it
//! holds, and resolves on the first sample that shows it no longer does.
//! A sample that can't tell (unreachable host, systemd not readable) leaves
//! an alert as it is.

use std::collections::{HashMap, HashSet};

use super::history::HistoryRow;

pub const DISK_WARN: f32 = 90.0;
pub const DISK_CRIT: f32 = 95.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alert {
    pub id: String,
    pub server_id: String,
    /// "unreachable", "disk", or "service:<unit>".
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
}
