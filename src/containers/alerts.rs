//! Container alerts (ERR-202), from two scans of a server a minute apart:
//!
//! - `container:<name>:unhealthy` (CRIT) while it runs and its health
//!   check fails;
//! - `container:<name>:exited` (CRIT) when Crow sees it go from running
//!   to exited with a non-zero code: a container that was already down
//!   when Crow first looked isn't news. It holds until it runs again;
//! - `container:<name>:restarts` (WARN) when its restart count goes up:
//!   the runtime only counts the restarts its restart policy made, so each
//!   one is a crash. It resolves after ten quiet minutes.
//!
//! A container that's gone (removed) takes its alerts with it.

use super::{Container, Scan};
use crate::metrics::alerts::{Alert, AlertChanges};

/// How long a crash-restart alert stays open without another one.
const RESTARTS_QUIET_SECS: i64 = 600;

const PREFIX: &str = "container:";

/// "Exited (137) 2 minutes ago" → 137.
pub fn exit_code(status: &str) -> Option<i32> {
    status.strip_prefix("Exited (")?.split_once(')')?.0.parse().ok()
}

fn kind(name: &str, what: &str) -> String {
    format!("{PREFIX}{name}:{what}")
}

/// The container a `container:` alert kind is about.
pub fn container_of(kind: &str) -> Option<&str> {
    kind.strip_prefix(PREFIX)?.rsplit_once(':').map(|(name, _)| name)
}

/// What one scan changes. `prev` is the scan from a minute ago, `None` the
/// first time.
pub fn container_changes(open: &[Alert], server_id: &str, prev: Option<&Scan>, scan: &Scan, now: i64) -> AlertChanges {
    let mut changes = AlertChanges::default();
    let open: Vec<&Alert> = open.iter().filter(|a| a.server_id == server_id && a.resolved_at.is_none() && a.kind.starts_with(PREFIX)).collect();
    let existing = |k: &str| open.iter().find(|a| a.kind == k).copied();
    let was = |c: &Container| prev.and_then(|p| p.containers.iter().find(|x| x.name == c.name && x.runtime == c.runtime && x.scope == c.scope));
    let hold = |k: String, level: &str, detail: String, changes: &mut AlertChanges| match existing(&k) {
        Some(a) => changes.updated.push((a.id.clone(), level.into(), detail, now)),
        None => changes.opened.push(Alert {
            id: format!("alert-{server_id}-{k}-{now}"),
            server_id: server_id.into(),
            kind: k,
            level: level.into(),
            detail,
            opened_at: now,
            last_seen: now,
            resolved_at: None,
            acknowledged_at: None,
        }),
    };
    let resolve = |k: String, changes: &mut AlertChanges| {
        if let Some(a) = existing(&k) {
            changes.resolved.push(a.id.clone());
        }
    };
    for c in &scan.containers {
        let before = was(c);

        let unhealthy = kind(&c.name, "unhealthy");
        if c.is_running() && c.health.as_deref() == Some("unhealthy") {
            hold(unhealthy, "CRIT", t!("containers-alert-unhealthy", name = c.name.clone()), &mut changes);
        } else if c.is_running() || existing(&unhealthy).is_some() {
            resolve(unhealthy, &mut changes);
        }

        let exited = kind(&c.name, "exited");
        match exit_code(&c.status).filter(|code| *code != 0 && !c.is_running()) {
            Some(code) if existing(&exited).is_some() || before.is_some_and(|b| b.is_running()) => hold(exited, "CRIT", t!("containers-alert-exited", name = c.name.clone(), code = code), &mut changes),
            _ if c.is_running() => resolve(exited, &mut changes),
            _ => {}
        }

        let restarts = kind(&c.name, "restarts");
        let rose = matches!((before.and_then(|b| b.restarts), c.restarts), (Some(a), Some(b)) if b > a);
        if rose {
            hold(restarts, "WARN", t!("containers-alert-restarted", name = c.name.clone(), n = c.restarts.unwrap_or(0)), &mut changes);
        } else if existing(&restarts).is_some_and(|a| now - a.last_seen >= RESTARTS_QUIET_SECS) {
            resolve(restarts, &mut changes);
        }
    }
    // Removed containers.
    for a in &open {
        if container_of(&a.kind).is_some_and(|name| !scan.containers.iter().any(|c| c.name == name)) {
            changes.resolved.push(a.id.clone());
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::containers::{Runtime, Scope};

    fn c(name: &str, state: &str, status: &str, health: Option<&str>, restarts: u32) -> Container {
        Container {
            runtime: Runtime::Docker,
            scope: Scope::Root,
            id: format!("id-{name}"),
            name: name.into(),
            image: "img".into(),
            state: state.into(),
            status: status.into(),
            health: health.map(String::from),
            ports: Vec::new(),
            restarts: Some(restarts),
            labels: Default::default(),
            usage: None,
        }
    }

    fn scan(cs: Vec<Container>) -> Scan {
        Scan { containers: cs, ..Default::default() }
    }

    /// The open alerts after applying `changes` to `open`.
    fn apply(open: &[Alert], changes: &AlertChanges) -> Vec<Alert> {
        let mut out: Vec<Alert> = open.iter().filter(|a| !changes.resolved.contains(&a.id)).cloned().collect();
        for (id, level, detail, seen) in &changes.updated {
            if let Some(a) = out.iter_mut().find(|a| a.id == *id) {
                (a.level, a.detail, a.last_seen) = (level.clone(), detail.clone(), *seen);
            }
        }
        out.extend(changes.opened.iter().cloned());
        out
    }

    #[test]
    fn exit_codes_read_from_the_status() {
        assert_eq!(exit_code("Exited (137) 2 minutes ago"), Some(137));
        assert_eq!(exit_code("Exited (0) 1 second ago"), Some(0));
        assert_eq!(exit_code("Up 3 days"), None);
        assert_eq!(container_of("container:shop-web-1:exited"), Some("shop-web-1"));
        assert_eq!(container_of("disk"), None);
    }

    #[test]
    fn a_container_that_was_already_down_is_not_news() {
        let first = scan(vec![c("job", "exited", "Exited (1) 2 hours ago", None, 0)]);
        assert_eq!(container_changes(&[], "s", None, &first, 100), AlertChanges::default());
        assert!(container_changes(&[], "s", Some(&first), &first, 160).opened.is_empty());
    }

    #[test]
    fn dying_opens_an_alert_that_holds_until_it_runs_again() {
        let up = scan(vec![c("web", "running", "Up 3 days", None, 0)]);
        let died = scan(vec![c("web", "exited", "Exited (137) 5 seconds ago", None, 0)]);
        let ch = container_changes(&[], "s", Some(&up), &died, 100);
        assert_eq!(ch.opened.len(), 1);
        assert_eq!((ch.opened[0].kind.as_str(), ch.opened[0].level.as_str()), ("container:web:exited", "CRIT"));
        assert!(ch.opened[0].detail.contains("137"), "{}", ch.opened[0].detail);
        let open = apply(&[], &ch);
        // Still down a minute later: held, not opened twice.
        let ch = container_changes(&open, "s", Some(&died), &died, 160);
        assert_eq!((ch.opened.len(), ch.updated.len()), (0, 1));
        let ch = container_changes(&open, "s", Some(&died), &up, 220);
        assert_eq!(ch.resolved, vec![open[0].id.clone()]);
        // A clean stop (code 0) is somebody's decision, not an alert.
        let stopped = scan(vec![c("web", "exited", "Exited (0) 1 second ago", None, 0)]);
        assert!(container_changes(&[], "s", Some(&up), &stopped, 100).opened.is_empty());
    }

    #[test]
    fn unhealthy_and_crash_restarts() {
        let ok = scan(vec![c("api", "running", "Up 1 hour (healthy)", Some("healthy"), 2)]);
        let sick = scan(vec![c("api", "running", "Up 1 hour (unhealthy)", Some("unhealthy"), 2)]);
        let ch = container_changes(&[], "s", Some(&ok), &sick, 100);
        assert_eq!(ch.opened.iter().map(|a| a.kind.as_str()).collect::<Vec<_>>(), vec!["container:api:unhealthy"]);
        let open = apply(&[], &ch);
        assert_eq!(container_changes(&open, "s", Some(&sick), &ok, 160).resolved.len(), 1);

        let crashed = scan(vec![c("api", "running", "Up 2 seconds (healthy)", Some("healthy"), 3)]);
        let ch = container_changes(&[], "s", Some(&ok), &crashed, 100);
        assert_eq!((ch.opened[0].kind.as_str(), ch.opened[0].level.as_str()), ("container:api:restarts", "WARN"));
        let open = apply(&[], &ch);
        assert!(container_changes(&open, "s", Some(&crashed), &crashed, 160).resolved.is_empty(), "too soon to call it settled");
        assert_eq!(container_changes(&open, "s", Some(&crashed), &crashed, 100 + RESTARTS_QUIET_SECS).resolved.len(), 1);
    }

    #[test]
    fn a_removed_container_takes_its_alerts_with_it() {
        let up = scan(vec![c("web", "running", "Up", None, 0)]);
        let died = scan(vec![c("web", "exited", "Exited (1) now", None, 0)]);
        let open = apply(&[], &container_changes(&[], "s", Some(&up), &died, 100));
        let ch = container_changes(&open, "s", Some(&died), &scan(vec![]), 160);
        assert_eq!(ch.resolved, vec![open[0].id.clone()]);
        assert!(container_changes(&open, "other-server", None, &scan(vec![]), 160).resolved.is_empty(), "another server's alerts are left alone");
    }
}
