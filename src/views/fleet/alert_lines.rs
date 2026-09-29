//! FLEET ALERTS lines (ERR-85): stored alerts, servers down right now that
//! haven't been down long enough to alert, and gaps when Crow wasn't
//! running. GPUI-free.

use crate::metrics::alerts::Alert;
use crate::vault::ServerRecord;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertLine {
    /// Stored alert id; `None` for a live-only line.
    pub id: Option<String>,
    /// "CRIT", "WARN", or "OK" for a resolved one.
    pub level: &'static str,
    pub host: String,
    pub message: String,
    /// "12m", "3h", "2d".
    pub age: String,
    pub acknowledged: bool,
    pub resolved: bool,
}

fn ago(secs: i64) -> String {
    match secs.max(0) {
        s if s < 3600 => format!("{}m", s / 60),
        s if s < 86_400 => format!("{}h", s / 3600),
        s => format!("{}d", s / 86_400),
    }
}

/// Open alerts (critical first, newest first), then servers down right now
/// with no alert yet, then alerts resolved in the last day.
pub fn alert_lines(stored: &[Alert], down_now: &[(String, String)], servers: &[ServerRecord], now: i64) -> Vec<AlertLine> {
    let name = |id: &str| servers.iter().find(|s| s.id == id).map(|s| s.name.clone()).unwrap_or_else(|| id.to_string());
    let level = |l: &str| if l == "CRIT" { "CRIT" } else { "WARN" };
    let mut open: Vec<&Alert> = stored.iter().filter(|a| a.resolved_at.is_none()).collect();
    open.sort_by_key(|a| (a.level != "CRIT", -a.opened_at));
    let mut lines: Vec<AlertLine> = open
        .iter()
        .map(|a| AlertLine {
            id: Some(a.id.clone()),
            level: level(&a.level),
            host: name(&a.server_id),
            message: a.detail.clone(),
            age: ago(now - a.opened_at),
            acknowledged: a.acknowledged_at.is_some(),
            resolved: false,
        })
        .collect();
    for (server_id, reason) in down_now {
        if !open.iter().any(|a| a.server_id == *server_id && a.kind == "unreachable") {
            lines.push(AlertLine {
                id: None,
                level: "WARN",
                host: name(server_id),
                message: format!("{reason} · alerts if it's still down at the next check"),
                age: "now".into(),
                acknowledged: false,
                resolved: false,
            });
        }
    }
    let mut resolved: Vec<&Alert> = stored.iter().filter(|a| a.resolved_at.is_some_and(|r| now - r < 86_400)).collect();
    resolved.sort_by_key(|a| -a.resolved_at.unwrap_or(0));
    lines.extend(resolved.iter().map(|a| AlertLine {
        id: Some(a.id.clone()),
        level: "OK",
        host: name(&a.server_id),
        message: format!("resolved · {} (lasted {})", a.detail, ago(a.resolved_at.unwrap_or(now) - a.opened_at)),
        age: ago(now - a.resolved_at.unwrap_or(now)),
        acknowledged: a.acknowledged_at.is_some(),
        resolved: true,
    }));
    lines
}

/// The last time Crow wasn't running, if it's worth saying: the gap between
/// the previous watch session's end and this one's start, over five minutes.
pub fn watch_gap(sessions: &[(i64, i64)], current_start: i64) -> Option<(i64, i64)> {
    let prev_end = sessions.iter().filter(|(start, _)| *start < current_start).map(|(_, end)| *end).max()?;
    (current_start - prev_end > 300).then_some((prev_end, current_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert(id: &str, level: &str, opened: i64, resolved: Option<i64>) -> Alert {
        Alert {
            id: id.into(),
            server_id: "s1".into(),
            kind: if id == "u" { "unreachable".into() } else { "disk".into() },
            level: level.into(),
            detail: format!("{id} detail"),
            opened_at: opened,
            last_seen: opened,
            resolved_at: resolved,
            acknowledged_at: None,
        }
    }

    #[test]
    fn open_crit_first_then_live_then_resolved() {
        let servers = [ServerRecord { id: "s1".into(), name: "web-1".into(), ..Default::default() }, ServerRecord { id: "s2".into(), name: "db-1".into(), ..Default::default() }];
        let now = 100_000;
        let stored = [alert("w", "WARN", now - 600, None), alert("c", "CRIT", now - 7200, None), alert("r", "WARN", now - 5000, Some(now - 1000)), alert("old", "WARN", 0, Some(1))];
        let lines = alert_lines(&stored, &[("s2".into(), "unreachable · timeout".into())], &servers, now);
        let summary: Vec<_> = lines.iter().map(|l| (l.level, l.host.as_str(), l.age.as_str())).collect();
        assert_eq!(summary, vec![("CRIT", "web-1", "2h"), ("WARN", "web-1", "10m"), ("WARN", "db-1", "now"), ("OK", "web-1", "16m")]);
        assert_eq!(lines[3].message, "resolved · r detail (lasted 1h)");
    }

    #[test]
    fn a_down_server_with_an_open_alert_isnt_listed_twice() {
        let stored = [alert("u", "CRIT", 0, None)];
        assert_eq!(alert_lines(&stored, &[("s1".into(), "x".into())], &[], 60).len(), 1);
    }

    #[test]
    fn gaps_over_five_minutes() {
        assert_eq!(watch_gap(&[(0, 1000), (5000, 5000)], 5000), Some((1000, 5000)));
        assert_eq!(watch_gap(&[(0, 4900)], 5000), None);
        assert_eq!(watch_gap(&[], 5000), None);
    }
}
