//! The lines that explain one journal line (ERR-198): the same process
//! around that moment, else the same unit, else the whole system for a
//! moment either side. What the wand on a log line sends to the AI.

use super::{JournalBootScope, JournalEntry, JournalQuery, JournalTimeRange};

/// Where the related lines come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    Process(u32),
    Unit(String),
    Moment,
}

/// Lines read around the picked one.
const LIMIT: usize = 300;

/// How far either side of the picked line to look, in seconds: a process's
/// story can be long, the whole system's gets noisy fast.
fn span(scope: &Scope) -> i64 {
    match scope {
        Scope::Process(_) => 600,
        Scope::Unit(_) => 300,
        Scope::Moment => 60,
    }
}

pub fn scope_of(e: &JournalEntry) -> Scope {
    match e.pid {
        Some(pid) if pid > 1 => Scope::Process(pid),
        // A real systemd unit (it has a type suffix), which `-u` can read.
        _ if e.unit.contains('.') => Scope::Unit(e.unit.clone()),
        _ => Scope::Moment,
    }
}

/// The journal read for the lines related to `e`.
pub fn query_for(e: &JournalEntry, boot: JournalBootScope) -> (JournalQuery, Scope) {
    let scope = scope_of(e);
    let secs = (e.timestamp_usec / 1_000_000) as i64;
    let window = (secs > 0).then(|| (secs - span(&scope), secs + span(&scope)));
    let q = JournalQuery {
        limit: LIMIT,
        unit: match &scope {
            Scope::Unit(u) => Some(u.clone()),
            _ => None,
        },
        pid: match scope {
            Scope::Process(p) => Some(p),
            _ => None,
        },
        window,
        // No time on the line: the last day of its boot.
        time_range: JournalTimeRange::Last24h,
        boot,
        ..Default::default()
    };
    (q, scope)
}

/// One line as the AI reads it.
pub fn line_text(e: &JournalEntry) -> String {
    format!("{} [{}] {}{}: {}", e.timestamp_formatted, e.priority.label(), e.unit, e.pid.map(|p| format!("[{p}]")).unwrap_or_default(), e.message)
}

/// The lines nearest in time to `picked` (always included, marked "→ "),
/// up to `max_chars`, oldest first.
pub fn around(read: &[JournalEntry], picked: &JournalEntry, max_chars: usize) -> Vec<String> {
    let same = |e: &JournalEntry| e.id == picked.id || (e.cursor.is_some() && e.cursor == picked.cursor);
    let mut others: Vec<&JournalEntry> = read.iter().filter(|e| !same(e)).collect();
    others.sort_by_key(|e| e.timestamp_usec.abs_diff(picked.timestamp_usec));
    let mut chosen: Vec<(&JournalEntry, bool)> = vec![(picked, true)];
    let mut size = line_text(picked).len() + 3;
    for e in others {
        size += line_text(e).len() + 3;
        if size > max_chars {
            break;
        }
        chosen.push((e, false));
    }
    chosen.sort_by_key(|(e, _)| e.timestamp_usec);
    chosen.into_iter().map(|(e, mine)| format!("{}{}", if mine { "→ " } else { "  " }, line_text(e))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::JournalPriority;

    fn entry(id: &str, secs: u64, unit: &str, pid: Option<u32>, msg: &str) -> JournalEntry {
        JournalEntry {
            id: id.into(),
            cursor: Some(format!("c-{id}")),
            timestamp_usec: secs * 1_000_000,
            timestamp_formatted: format!("t{secs}"),
            time_relative: String::new(),
            priority: JournalPriority::Info,
            unit: unit.into(),
            syslog_identifier: String::new(),
            pid,
            message: msg.into(),
            fields: Vec::new(),
            is_expanded: false,
        }
    }

    #[test]
    fn a_process_line_reads_that_process_around_it() {
        let (q, scope) = query_for(&entry("a", 10_000, "ssh.service", Some(812), "x"), JournalBootScope::Current);
        assert_eq!(scope, Scope::Process(812));
        assert_eq!((q.pid, q.unit.as_deref()), (Some(812), None));
        assert_eq!(q.window, Some((9_400, 10_600)));
    }

    #[test]
    fn without_a_process_the_unit_then_the_moment() {
        assert_eq!(scope_of(&entry("a", 1, "cron.service", None, "x")), Scope::Unit("cron.service".into()));
        assert_eq!(scope_of(&entry("a", 1, "cron.service", Some(1), "x")), Scope::Unit("cron.service".into()));
        let (q, scope) = query_for(&entry("a", 1_000, "kernel", None, "x"), JournalBootScope::Current);
        assert_eq!(scope, Scope::Moment);
        assert_eq!((q.pid, q.unit), (None, None));
        assert_eq!(q.window, Some((940, 1_060)));
    }

    #[test]
    fn the_picked_line_is_kept_marked_with_its_nearest_neighbours() {
        let picked = entry("p", 100, "u.service", Some(5), "picked");
        let read = vec![entry("far", 10, "u.service", Some(5), "far away"), entry("near", 99, "u.service", Some(5), "just before"), entry("after", 101, "u.service", Some(5), "just after"), entry("p", 100, "u.service", Some(5), "picked")];
        let all = around(&read, &picked, 10_000);
        assert_eq!(all.len(), 4);
        assert!(all[0].contains("far away") && all[2].starts_with("→ ") && all[2].contains("picked"));
        // Too little room: the far one goes first, the picked one never.
        let one = line_text(&picked).len() + 3;
        let tight = around(&read, &picked, one * 3);
        assert_eq!(tight.iter().filter(|l| l.contains("far away")).count(), 0);
        assert!(tight.iter().any(|l| l.starts_with("→ ")));
        assert_eq!(around(&[], &picked, 0), vec![format!("→ {}", line_text(&picked))]);
    }
}
