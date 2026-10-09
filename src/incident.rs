//! Incident timeline (ERR-149): for a server and a window of time, what
//! happened (alerts, check failures, error logs) and what changed around
//! it (Crow's own actions), in one list.

use crate::journal::JournalEntry;
use crate::metrics::alerts::Alert;
use crate::vault::ChangeRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventKind {
    AlertOpened,
    AlertResolved,
    Change,
    CheckFailed,
    Log,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::AlertOpened => "ALERT",
            EventKind::AlertResolved => "RESOLVED",
            EventKind::Change => "CHANGE",
            EventKind::CheckFailed => "CHECK",
            EventKind::Log => "LOG",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub ts: i64,
    pub kind: EventKind,
    pub server: String,
    pub text: String,
    /// CRIT/WARN for alerts, the journal priority for logs.
    pub level: String,
}

/// What a timeline is built from, already narrowed to its servers.
pub struct Sources<'a> {
    pub from: i64,
    pub to: i64,
    /// server id → name.
    pub names: &'a std::collections::HashMap<String, String>,
    pub alerts: &'a [Alert],
    pub changes: &'a [ChangeRecord],
    /// (check name, server id, ts, detail) of failed results.
    pub check_failures: &'a [(String, String, i64, String)],
    /// (server id, lines).
    pub logs: &'a [(String, Vec<JournalEntry>)],
}

fn rfc3339_ts(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s).ok().map(|t| t.timestamp())
}

/// Everything in the window, oldest first.
pub fn build(src: &Sources) -> Vec<Event> {
    let name = |id: &str| src.names.get(id).cloned().unwrap_or_else(|| id.to_string());
    let inside = |ts: i64| ts >= src.from && ts <= src.to;
    let mut out = Vec::new();
    for a in src.alerts {
        if inside(a.opened_at) {
            out.push(Event { ts: a.opened_at, kind: EventKind::AlertOpened, server: name(&a.server_id), text: a.detail.clone(), level: a.level.clone() });
        }
        if let Some(r) = a.resolved_at.filter(|r| inside(*r)) {
            out.push(Event { ts: r, kind: EventKind::AlertResolved, server: name(&a.server_id), text: a.detail.clone(), level: a.level.clone() });
        }
    }
    for c in src.changes {
        if let Some(ts) = rfc3339_ts(&c.started_at).filter(|t| inside(*t)) {
            let after = c.after_state.as_deref().filter(|a| !a.is_empty()).map(|a| format!(" → {a}")).unwrap_or_default();
            out.push(Event { ts, kind: EventKind::Change, server: if c.server_name.is_empty() { name(&c.server_id) } else { c.server_name.clone() }, text: format!("{} {}: {}{after} ({})", c.action_kind, c.target, c.before_state, c.outcome), level: c.outcome.clone() });
        }
    }
    for (check, server, ts, detail) in src.check_failures {
        if inside(*ts) {
            out.push(Event { ts: *ts, kind: EventKind::CheckFailed, server: name(server), text: format!("{check}: {detail}"), level: "CRIT".into() });
        }
    }
    for (server, lines) in src.logs {
        for e in lines {
            let ts = (e.timestamp_usec / 1_000_000) as i64;
            if inside(ts) {
                out.push(Event { ts, kind: EventKind::Log, server: name(server), text: format!("{}: {}", if e.unit.is_empty() { &e.syslog_identifier } else { &e.unit }, e.message), level: e.priority.label().to_string() });
            }
        }
    }
    out.sort_by(|a, b| a.ts.cmp(&b.ts).then(a.kind.cmp(&b.kind)));
    out
}

fn when(ts: i64) -> String {
    use chrono::TimeZone;
    chrono::Local.timestamp_opt(ts, 0).single().map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default()
}

/// The timeline as plain lines (for the clanker): "time KIND server: text".
pub fn as_lines(events: &[Event], max: usize) -> String {
    let skip = events.len().saturating_sub(max);
    events.iter().skip(skip).map(|e| format!("{} {} {}: {}", when(e.ts), e.kind.label(), e.server, e.text)).collect::<Vec<_>>().join("\n")
}

/// The timeline as a Markdown post-mortem draft.
pub fn markdown(title: &str, from: i64, to: i64, events: &[Event], summary: Option<&str>) -> String {
    let mut md = format!("# Incident: {title}\n\nWindow: {} → {}\n\n", when(from), when(to));
    let count = |k: EventKind| events.iter().filter(|e| e.kind == k).count();
    md.push_str(&format!("- Alerts opened: {}\n- Alerts resolved: {}\n- Changes by Crow: {}\n- Check failures: {}\n- Error log lines: {}\n\n", count(EventKind::AlertOpened), count(EventKind::AlertResolved), count(EventKind::Change), count(EventKind::CheckFailed), count(EventKind::Log)));
    if let Some(s) = summary {
        md.push_str("## A likely sequence (an AI's reading: check it)\n\n");
        md.push_str(s.trim());
        md.push_str("\n\n");
    }
    md.push_str("## Timeline\n\n| Time | What | Server | Detail |\n|---|---|---|---|\n");
    for e in events {
        md.push_str(&format!("| {} | {} | {} | {} |\n", when(e.ts), e.kind.label(), e.server, e.text.replace('|', "/").replace('\n', " ")));
    }
    md.push_str("\n## Follow-ups\n\n- [ ] \n");
    md
}

#[cfg(test)]
mod tests {
    use super::{build, markdown, EventKind, Sources};
    use crate::metrics::alerts::Alert;
    use crate::vault::ChangeRecord;
    use std::collections::HashMap;

    #[test]
    fn a_timeline_merges_everything_in_the_window_in_order() {
        let names: HashMap<String, String> = [("s1".to_string(), "web-1".to_string())].into();
        let alerts = vec![Alert { id: "a".into(), server_id: "s1".into(), kind: "disk".into(), level: "CRIT".into(), detail: "disk / at 97%".into(), opened_at: 1_000, resolved_at: Some(1_600), ..Default::default() }];
        let at = |ts: i64| chrono::DateTime::from_timestamp(ts, 0).unwrap().to_rfc3339();
        let changes = vec![
            ChangeRecord { id: "c".into(), server_id: "s1".into(), server_name: "web-1".into(), action_kind: "service.restart".into(), target: "nginx".into(), before_state: "restart".into(), after_state: None, blast_radius: None, outcome: "success".into(), started_at: at(900), completed_at: None },
            ChangeRecord { id: "old".into(), server_id: "s1".into(), server_name: "web-1".into(), action_kind: "x".into(), target: "y".into(), before_state: "z".into(), after_state: None, blast_radius: None, outcome: "success".into(), started_at: at(10), completed_at: None },
        ];
        let checks = vec![("site".to_string(), "s1".to_string(), 1_100, "HTTP 502".to_string())];
        let src = Sources { from: 800, to: 2_000, names: &names, alerts: &alerts, changes: &changes, check_failures: &checks, logs: &[] };
        let ev = build(&src);
        assert_eq!(ev.iter().map(|e| e.kind).collect::<Vec<_>>(), [EventKind::Change, EventKind::AlertOpened, EventKind::CheckFailed, EventKind::AlertResolved], "the change before the alert, the old one left out");
        assert!(ev[0].text.contains("service.restart nginx"));
        let md = markdown("disk on web-1", 800, 2_000, &ev, Some("nginx was restarted, then the disk filled."));
        assert!(md.contains("Alerts opened: 1") && md.contains("an AI's reading") && md.contains("| CHECK | web-1 | site: HTTP 502 |"));
    }
}
