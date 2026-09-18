use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};
use gpui_kit::Rgba;
use crate::theme::*;

pub mod reader;

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
