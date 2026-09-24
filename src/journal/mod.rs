use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use gpui_kit::Rgba;
use crate::theme::*;

pub mod reader;
pub mod retention;

pub use retention::{
    generate_journald_conf, read_retention_for_server, JournalRetentionConfig,
    JournalStorageMode, JournalTelemetry,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JournalPriority {
    Emerg = 0,
    Alert = 1,
    Crit = 2,
    Err = 3,
    Warning = 4,
    Notice = 5,
    Info = 6,
    Debug = 7,
}

impl JournalPriority {
    pub fn from_num_or_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "0" | "emerg" | "emergency" => JournalPriority::Emerg,
            "1" | "alert" => JournalPriority::Alert,
            "2" | "crit" | "critical" => JournalPriority::Crit,
            "3" | "err" | "error" => JournalPriority::Err,
            "4" | "warning" | "warn" => JournalPriority::Warning,
            "5" | "notice" => JournalPriority::Notice,
            "6" | "info" => JournalPriority::Info,
            "7" | "debug" => JournalPriority::Debug,
            _ => JournalPriority::Info,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            JournalPriority::Emerg => "EMERG",
            JournalPriority::Alert => "ALERT",
            JournalPriority::Crit => "CRIT",
            JournalPriority::Err => "ERROR",
            JournalPriority::Warning => "WARN",
            JournalPriority::Notice => "NOTICE",
            JournalPriority::Info => "INFO",
            JournalPriority::Debug => "DEBUG",
        }
    }

    pub fn color(&self) -> Rgba {
        match self {
            JournalPriority::Emerg | JournalPriority::Alert | JournalPriority::Crit => CRIT,
            JournalPriority::Err => CRIT,
            JournalPriority::Warning => WARN,
            JournalPriority::Notice => hex_rgb(0x8ab4ff),
            JournalPriority::Info => TEXT_PRIMARY,
            JournalPriority::Debug => TEXT_FAINT,
        }
    }

    pub fn badge_bg(&self) -> Rgba {
        match self {
            JournalPriority::Emerg | JournalPriority::Alert | JournalPriority::Crit | JournalPriority::Err => CRIT_BG,
            JournalPriority::Warning => WARN_BG,
            JournalPriority::Notice => hex_rgba(0x8ab4ff, 0.12),
            JournalPriority::Info => BG_CHIP,
            JournalPriority::Debug => hex_rgba(0x27272a, 0.4),
        }
    }

    pub fn badge_fg(&self) -> Rgba {
        match self {
            JournalPriority::Emerg | JournalPriority::Alert | JournalPriority::Crit | JournalPriority::Err => CRIT,
            JournalPriority::Warning => WARN,
            JournalPriority::Notice => hex_rgb(0x8ab4ff),
            JournalPriority::Info => TEXT_SECONDARY,
            JournalPriority::Debug => TEXT_DIMMER,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, JournalPriority::Emerg | JournalPriority::Alert | JournalPriority::Crit | JournalPriority::Err)
    }

    pub fn is_warn(&self) -> bool {
        matches!(self, JournalPriority::Warning)
    }
}

/// How far back a query should reach, translated into a `journalctl --since` string.
/// `Live` means "no --since, just tail the most recent N lines" — the only mode
/// compatible with continuing to auto-poll every few seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalTimeRange {
    Live,
    Last15m,
    Last1h,
    Last6h,
    Last24h,
    Last7d,
    AllTime,
}

impl JournalTimeRange {
    pub fn label(&self) -> &'static str {
        match self {
            JournalTimeRange::Live => "LIVE",
            JournalTimeRange::Last15m => "15M",
            JournalTimeRange::Last1h => "1H",
            JournalTimeRange::Last6h => "6H",
            JournalTimeRange::Last24h => "24H",
            JournalTimeRange::Last7d => "7D",
            JournalTimeRange::AllTime => "ALL TIME",
        }
    }

    /// A `journalctl --since`-compatible relative time string, or None for no bound.
    pub fn since_str(&self) -> Option<&'static str> {
        match self {
            JournalTimeRange::Live => None,
            JournalTimeRange::Last15m => Some("15 minutes ago"),
            JournalTimeRange::Last1h => Some("1 hour ago"),
            JournalTimeRange::Last6h => Some("6 hours ago"),
            JournalTimeRange::Last24h => Some("24 hours ago"),
            JournalTimeRange::Last7d => Some("7 days ago"),
            JournalTimeRange::AllTime => None,
        }
    }
}

/// Which boot's journal to read — current or the one before it. A handful of ops
/// questions ("did this survive the last reboot") only make sense across this boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalBootScope {
    Current,
    Previous,
}

impl JournalBootScope {
    pub fn label(&self) -> &'static str {
        match self {
            JournalBootScope::Current => "THIS BOOT",
            JournalBootScope::Previous => "PREVIOUS BOOT",
        }
    }

    pub fn offset(&self) -> i32 {
        match self {
            JournalBootScope::Current => 0,
            JournalBootScope::Previous => -1,
        }
    }
}

/// Everything needed to run one journal lookup, local or simulated. Built fresh
/// from current filter/UI state each time a query runs — there is exactly one
/// query-building path, used by both the live-tail poll and an explicit search.
#[derive(Clone, Debug)]
pub struct JournalQuery {
    pub limit: usize,
    pub unit: Option<String>,
    pub priority: Option<JournalPriority>,
    pub pid: Option<u32>,
    pub grep: Option<String>,
    pub time_range: JournalTimeRange,
    pub boot: JournalBootScope,
}

impl Default for JournalQuery {
    fn default() -> Self {
        Self {
            limit: 200,
            unit: None,
            priority: None,
            pid: None,
            grep: None,
            time_range: JournalTimeRange::Live,
            boot: JournalBootScope::Current,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub cursor: Option<String>,
    pub timestamp_usec: u64,
    pub timestamp_formatted: String,
    pub time_relative: String,
    pub priority: JournalPriority,
    pub unit: String,
    pub syslog_identifier: String,
    pub pid: Option<u32>,
    pub message: String,
    pub fields: Vec<(String, String)>,
    pub is_expanded: bool,
}

impl JournalEntry {
    /// Formats the raw timestamp into local HH:MM:SS.mmm and relative note.
    pub fn format_time(usec: u64) -> (String, String) {
        if usec == 0 {
            return (Local::now().format("%H:%M:%S.000").to_string(), "now".to_string());
        }

        let secs = (usec / 1_000_000) as i64;
        let nsecs = ((usec % 1_000_000) * 1_000) as u32;

        let dt = match Local.timestamp_opt(secs, nsecs) {
            chrono::LocalResult::Single(t) => t,
            _ => Local::now(),
        };

        let formatted = dt.format("%H:%M:%S").to_string();
        let millis = nsecs / 1_000_000;
        let ts_str = format!("{}.{:03}", formatted, millis);

        let now_secs = Local::now().timestamp();
        let diff = (now_secs - secs).max(0);
        let relative = if diff < 5 {
            "just now".to_string()
        } else if diff < 60 {
            format!("{}s ago", diff)
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        };

        (ts_str, relative)
    }
}

/// Parses a single JSON line from `journalctl -o json`.
pub fn parse_journal_json(line: &str) -> Option<JournalEntry> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = v.as_object()?;

    // 1. Message
    let message = if let Some(m) = obj.get("MESSAGE") {
        if let Some(s) = m.as_str() {
            s.to_string()
        } else if let Some(arr) = m.as_array() {
            // Binary data / byte array message
            let bytes: Vec<u8> = arr.iter().filter_map(|b| b.as_u64().map(|x| x as u8)).collect();
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            m.to_string()
        }
    } else {
        return None;
    };

    // 2. Priority
    let priority_val = obj.get("PRIORITY")
        .and_then(|p| p.as_str())
        .unwrap_or("6");
    let priority = JournalPriority::from_num_or_str(priority_val);

    // 3. Timestamp
    let usec: u64 = obj.get("__REALTIME_TIMESTAMP")
        .and_then(|t| t.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0);
    let (timestamp_formatted, time_relative) = JournalEntry::format_time(usec);

    // 4. Unit / Identifier
    let unit = obj.get("_SYSTEMD_UNIT")
        .and_then(|u| u.as_str())
        .or_else(|| obj.get("_SYSTEMD_USER_UNIT").and_then(|u| u.as_str()))
        .unwrap_or_else(|| {
            obj.get("SYSLOG_IDENTIFIER")
                .and_then(|s| s.as_str())
                .or_else(|| obj.get("_COMM").and_then(|c| c.as_str()))
                .unwrap_or("system")
        })
        .to_string();

    let syslog_identifier = obj.get("SYSLOG_IDENTIFIER")
        .and_then(|s| s.as_str())
        .or_else(|| obj.get("_COMM").and_then(|c| c.as_str()))
        .unwrap_or(&unit)
        .to_string();

    // 5. PID
    let pid = obj.get("_PID")
        .and_then(|p| p.as_str().and_then(|s| s.parse::<u32>().ok()));

    let cursor = obj.get("__CURSOR").and_then(|c| c.as_str()).map(|s| s.to_string());
    let id = cursor.clone().unwrap_or_else(|| format!("{}_{}", usec, pid.unwrap_or(0)));

    // 6. Extract key systemd fields for inspector
    let mut fields = Vec::new();
    let priority_keys = [
        "_SYSTEMD_UNIT",
        "_COMM",
        "_PID",
        "_CMDLINE",
        "_EXE",
        "_UID",
        "_GID",
        "_HOSTNAME",
        "_BOOT_ID",
        "_SYSTEMD_CGROUP",
        "_SYSTEMD_SLICE",
        "_TRANSPORT",
        "SYSLOG_FACILITY",
        "__CURSOR",
    ];

    for &key in &priority_keys {
        if let Some(val) = obj.get(key).and_then(|x| x.as_str()) {
            fields.push((key.to_string(), val.to_string()));
        }
    }

    Some(JournalEntry {
        id,
        cursor,
        timestamp_usec: usec,
        timestamp_formatted,
        time_relative,
        priority,
        unit,
        syslog_identifier,
        pid,
        message: message.trim().to_string(),
        fields,
        is_expanded: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_journal_json() {
        let sample = r#"{"PRIORITY":"3","_HOSTNAME":"trgk","MESSAGE":"Reject scan trigger since one is already pending","_PID":"1534","_SYSTEMD_UNIT":"wpa_supplicant.service","SYSLOG_IDENTIFIER":"wpa_supplicant","__REALTIME_TIMESTAMP":"1789758237876213","_EXE":"/usr/bin/wpa_supplicant","_CMDLINE":"/usr/sbin/wpa_supplicant -c /etc/wpa_supplicant/wpa_supplicant.conf"}"#;

        let entry = parse_journal_json(sample).expect("Failed to parse sample json");
        assert_eq!(entry.priority, JournalPriority::Err);
        assert_eq!(entry.unit, "wpa_supplicant.service");
        assert_eq!(entry.pid, Some(1534));
        assert_eq!(entry.message, "Reject scan trigger since one is already pending");
        assert!(entry.fields.iter().any(|(k, v)| k == "_CMDLINE" && v.contains("wpa_supplicant")));
    }

    #[test]
    fn test_priority_levels() {
        assert_eq!(JournalPriority::from_num_or_str("2"), JournalPriority::Crit);
        assert_eq!(JournalPriority::from_num_or_str("warn"), JournalPriority::Warning);
        assert_eq!(JournalPriority::from_num_or_str("INFO"), JournalPriority::Info);
        assert!(JournalPriority::Err.is_error());
        assert!(JournalPriority::Warning.is_warn());
    }
}

#[cfg(test)]
impl JournalEntry {
    /// A bare entry for tests, with `unit` set.
    pub fn from_marker_for_test(unit: &str) -> Self {
        JournalEntry {
            id: String::new(),
            cursor: None,
            timestamp_usec: 0,
            timestamp_formatted: String::new(),
            time_relative: String::new(),
            priority: JournalPriority::Info,
            unit: unit.to_string(),
            syslog_identifier: String::new(),
            pid: None,
            message: String::new(),
            fields: Vec::new(),
            is_expanded: false,
        }
    }
}

/// The literal pieces of a journal search to highlight in results: the
/// alternatives of `a|b`, split at `.*`, with simple escapes removed.
/// Pieces that are still regex syntax (classes, groups, quantifiers) are
/// skipped rather than guessed at.
pub fn search_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for alt in query.split('|') {
        for piece in alt.split(".*") {
            let mut lit = String::new();
            let mut chars = piece.chars();
            let mut regexy = false;
            while let Some(c) = chars.next() {
                match c {
                    '\\' => lit.extend(chars.next()),
                    '[' | ']' | '(' | ')' | '{' | '}' | '^' | '$' | '+' | '?' | '*' => regexy = true,
                    c => lit.push(c),
                }
            }
            let lit = lit.trim().to_string();
            if !regexy && lit.chars().count() >= 2 && !terms.contains(&lit) {
                terms.push(lit);
            }
        }
    }
    terms
}

/// Byte ranges in `text` where any of `terms` occurs, ignoring ASCII case,
/// merged and in order.
pub fn match_ranges(text: &str, terms: &[String]) -> Vec<std::ops::Range<usize>> {
    let hay = text.to_ascii_lowercase();
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    for term in terms {
        let needle = term.to_ascii_lowercase();
        let mut from = 0;
        while let Some(i) = hay[from..].find(&needle) {
            let start = from + i;
            ranges.push(start..start + needle.len());
            from = start + needle.len().max(1);
        }
    }
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<std::ops::Range<usize>> = Vec::new();
    for r in ranges {
        match merged.last_mut() {
            Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
            _ => merged.push(r),
        }
    }
    merged.retain(|r| text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
    merged
}

#[cfg(test)]
mod search_highlight_tests {
    use super::*;

    #[test]
    fn regex_searches_become_literal_highlight_terms() {
        assert_eq!(search_terms("fail.*timeout"), ["fail", "timeout"]);
        assert_eq!(search_terms("out of memory|oom-kill"), ["out of memory", "oom-kill"]);
        assert_eq!(search_terms("\\[UFW BLOCK\\]"), ["[UFW BLOCK]"]);
        assert!(search_terms("err(or)?").is_empty(), "groups and quantifiers aren't guessed at");
    }

    #[test]
    fn ranges_ignore_case_and_merge() {
        let t = vec!["fail".to_string(), "failed".to_string()];
        assert_eq!(match_ranges("Unit FAILED; failed again", &t), [5..11, 13..19]);
        assert!(match_ranges("naïve Fail", &["fail".into()]) == [7..11]);
    }
}
