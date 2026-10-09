//! Fleet log search (ERR-147): one journal query on every server of a
//! group at once, merged by time; saved searches that can raise alerts.

use serde::{Deserialize, Serialize};

use crate::journal::{JournalEntry, JournalPriority, JournalQuery, JournalTimeRange};

/// The vault flag that holds saved searches.
pub const SAVED_FLAG: &str = "logsearch.saved";
/// How often alerting searches run (over the last 15 minutes).
pub const ALERT_EVERY_SECS: i64 = 10 * 60;

/// One server's line, for the merged list.
#[derive(Clone, Debug)]
pub struct FleetHit {
    pub server_id: String,
    pub server: String,
    pub entry: JournalEntry,
}

/// One server's results: its lines, or why it couldn't be searched.
pub type ServerResult = (String, String, Result<Vec<JournalEntry>, String>);

/// All servers' lines, newest first.
pub fn merge(results: &[ServerResult]) -> Vec<FleetHit> {
    let mut hits: Vec<FleetHit> = results
        .iter()
        .filter_map(|(id, name, r)| r.as_ref().ok().map(|lines| (id, name, lines)))
        .flat_map(|(id, name, lines)| lines.iter().map(move |e| FleetHit { server_id: id.clone(), server: name.clone(), entry: e.clone() }))
        .collect();
    hits.sort_by(|a, b| b.entry.timestamp_usec.cmp(&a.entry.timestamp_usec));
    hits
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    pub text: String,
    pub unit: String,
    /// syslog priority (0 emerg … 7 debug) and worse; `None`: any.
    pub priority: Option<u8>,
    /// A group name, or empty for every server.
    pub group: String,
    /// Raise an alert when it matches.
    pub alert: bool,
    /// "WARN" or "CRIT".
    pub level: String,
}

impl SavedSearch {
    /// The journal query it runs, over `range`, at most `limit` lines a server.
    pub fn query(&self, range: JournalTimeRange, limit: usize) -> JournalQuery {
        JournalQuery {
            limit,
            unit: (!self.unit.trim().is_empty()).then(|| self.unit.trim().to_string()),
            priority: self.priority.map(|p| JournalPriority::from_num_or_str(&p.to_string())),
            pid: None,
            grep: (!self.text.trim().is_empty()).then(|| self.text.trim().to_string()),
            time_range: range,
            boot: crate::journal::JournalBootScope::Current,
            window: None,
        }
    }

    pub fn alert_kind(&self) -> String {
        format!("logsearch:{}", self.id)
    }
}

/// Alert changes for one saved search on one server: open (or update)
/// when it matched, resolve when it didn't.
pub fn alert_changes(open: &[crate::metrics::alerts::Alert], search: &SavedSearch, server_id: &str, matches: usize, sample: Option<&str>, now: i64) -> crate::metrics::alerts::AlertChanges {
    use crate::metrics::alerts::{Alert, AlertChanges};
    let kind = search.alert_kind();
    let existing = open.iter().find(|a| a.server_id == server_id && a.kind == kind && a.resolved_at.is_none());
    let mut c = AlertChanges::default();
    if matches > 0 {
        let detail = format!("\"{}\": {matches} line{} in the last 15 min{}", search.name, if matches == 1 { "" } else { "s" }, sample.map(|s| format!(" · {}", s.chars().take(120).collect::<String>())).unwrap_or_default());
        match existing {
            Some(a) => c.updated.push((a.id.clone(), search.level.clone(), detail, now)),
            None => c.opened.push(Alert { id: format!("alert-{server_id}-{kind}-{now}"), server_id: server_id.into(), kind, level: search.level.clone(), detail, opened_at: now, last_seen: now, resolved_at: None, acknowledged_at: None }),
        }
    } else if let Some(a) = existing {
        c.resolved.push(a.id.clone());
    }
    c
}

/// The lines sent to the clanker: "server unit: message", newest last,
/// capped. Nothing else about the servers is sent.
pub fn summary_input(hits: &[FleetHit], max: usize) -> String {
    hits.iter().take(max).rev().map(|h| format!("{} {} {}: {}", h.entry.timestamp_formatted, h.server, if h.entry.unit.is_empty() { &h.entry.syslog_identifier } else { &h.entry.unit }, h.entry.message)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::{alert_changes, merge, summary_input, SavedSearch};
    use crate::journal::JournalEntry;

    fn entry(ts: u64, msg: &str) -> JournalEntry {
        JournalEntry { id: ts.to_string(), cursor: None, timestamp_usec: ts, timestamp_formatted: format!("t{ts}"), time_relative: String::new(), priority: crate::journal::JournalPriority::Err, unit: "app.service".into(), syslog_identifier: "app".into(), pid: None, message: msg.into(), fields: vec![], is_expanded: false }
    }

    #[test]
    fn results_merge_newest_first_and_skip_failed_servers() {
        let results = vec![("a".to_string(), "web-1".to_string(), Ok(vec![entry(10, "x"), entry(30, "y")])), ("b".to_string(), "web-2".to_string(), Ok(vec![entry(20, "z")])), ("c".to_string(), "db".to_string(), Err("unreachable".to_string()))];
        let hits = merge(&results);
        assert_eq!(hits.iter().map(|h| (h.server.as_str(), h.entry.timestamp_usec)).collect::<Vec<_>>(), [("web-1", 30), ("web-2", 20), ("web-1", 10)]);
        assert_eq!(summary_input(&hits, 2), "t20 web-2 app.service: z\nt30 web-1 app.service: y");
    }

    #[test]
    fn a_saved_search_alerts_while_it_matches() {
        let s = SavedSearch { id: "oom".into(), name: "OOM in prod".into(), text: "Out of memory".into(), unit: String::new(), priority: Some(3), group: "prod".into(), alert: true, level: "CRIT".into() };
        let q = s.query(crate::journal::JournalTimeRange::Last15m, 50);
        assert_eq!((q.grep.as_deref(), q.limit, q.unit.is_none()), (Some("Out of memory"), 50, true));
        let c = alert_changes(&[], &s, "web-1", 3, Some("Out of memory: Killed process 42 (java)"), 1);
        assert_eq!((c.opened.len(), c.opened[0].level.as_str()), (1, "CRIT"));
        assert!(c.opened[0].detail.contains("3 lines") && c.opened[0].detail.contains("Killed process"));
        let resolved = alert_changes(&c.opened, &s, "web-1", 0, None, 2);
        assert_eq!(resolved.resolved.len(), 1);
    }
}
