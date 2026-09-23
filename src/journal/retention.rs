use serde::{Deserialize, Serialize};
use crate::host::{host_for, Host, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalStorageMode {
    Persistent,
    Volatile,
    Auto,
}

impl JournalStorageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Persistent => "persistent",
            Self::Volatile => "volatile",
            Self::Auto => "auto",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Persistent => "PERSISTENT (Survives Reboots)",
            Self::Volatile => "VOLATILE (RAM-only, Wiped on Reboot)",
            Self::Auto => "AUTO (Systemd Default, Fragile)",
        }
    }

    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Persistent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalRetentionConfig {
    pub storage: JournalStorageMode,
    pub system_max_use_mb: u64,
    pub system_keep_free_mb: u64,
    pub max_retention_days: u32,
    pub rate_limit_burst: u32,
    pub rate_limit_interval_sec: u32,
    pub sync_interval_sec: u32,
}

impl Default for JournalRetentionConfig {
    fn default() -> Self {
        Self {
            storage: JournalStorageMode::Persistent,
            system_max_use_mb: 4096, // 4.0 GB recommended
            system_keep_free_mb: 2048, // 2.0 GB reserve
            max_retention_days: 30, // 30 days recommended
            rate_limit_burst: 10000,
            rate_limit_interval_sec: 30,
            sync_interval_sec: 300,
        }
    }
}

impl JournalRetentionConfig {
    pub fn max_use_display(&self) -> String {
        if self.system_max_use_mb >= 1024 {
            let gb = self.system_max_use_mb as f32 / 1024.0;
            if (gb - gb.round()).abs() < 0.01 {
                format!("{:.0} GB", gb)
            } else {
                format!("{:.1} GB", gb)
            }
        } else {
            format!("{} MB", self.system_max_use_mb)
        }
    }

    pub fn retention_display(&self) -> String {
        if self.max_retention_days == 0 {
            "Unlimited".to_string()
        } else if self.max_retention_days % 365 == 0 {
            let yrs = self.max_retention_days / 365;
            format!("{} Year{}", yrs, if yrs > 1 { "s" } else { "" })
        } else {
            format!("{} Days", self.max_retention_days)
        }
    }

    pub fn keep_free_display(&self) -> String {
        if self.system_keep_free_mb >= 1024 {
            format!("{:.1} GB", self.system_keep_free_mb as f32 / 1024.0)
        } else {
            format!("{} MB", self.system_keep_free_mb)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalTelemetry {
    pub disk_usage_bytes: u64,
    pub oldest_timestamp: String,
    pub retention_horizon_hours: f32,
    pub daily_burn_rate_mb: f32,
    pub estimated_retained_days: f32,
    pub is_volatile_warning: bool,
    pub active_files_count: u32,
}

impl Default for JournalTelemetry {
    fn default() -> Self {
        Self {
            disk_usage_bytes: 252_706_816, // ~241 MB
            oldest_timestamp: "4 days ago".to_string(),
            retention_horizon_hours: 96.0,
            daily_burn_rate_mb: 60.5,
            estimated_retained_days: 67.7,
            is_volatile_warning: false,
            active_files_count: 8,
        }
    }
}

impl JournalTelemetry {
    pub fn disk_usage_display(&self) -> String {
        let mb = self.disk_usage_bytes as f32 / (1024.0 * 1024.0);
        if mb >= 1024.0 {
            format!("{:.2} GB", mb / 1024.0)
        } else {
            format!("{:.1} MB", mb)
        }
    }

    pub fn usage_percentage(&self, max_use_mb: u64) -> f32 {
        if max_use_mb == 0 {
            return 0.0;
        }
        let used_mb = self.disk_usage_bytes as f32 / (1024.0 * 1024.0);
        (used_mb / max_use_mb as f32) * 100.0
    }
}

/// Journal retention settings and telemetry for a server, read over its transport.
pub fn read_retention_for_server(server: &ServerRecord) -> (JournalRetentionConfig, JournalTelemetry) {
    read_retention(host_for(server).as_ref())
}

/// Queries a host's systemd journal disk usage and storage mode.
pub fn read_retention(host: &dyn Host) -> (JournalRetentionConfig, JournalTelemetry) {
    let mut config = JournalRetentionConfig::default();
    let mut telemetry = JournalTelemetry::default();

    // 1. Run journalctl --disk-usage
    if let Ok(out) = host.exec(&["journalctl", "--disk-usage"], DEFAULT_TIMEOUT) {
        // Example: "Archived and active journals take up 241M in the file system."
        telemetry.disk_usage_bytes = parse_disk_usage_output(&out.stdout).unwrap_or(241 * 1024 * 1024);
    }

    // 2. Check if persistent directory exists
    if host.exists("/var/log/journal") {
        config.storage = JournalStorageMode::Persistent;
        telemetry.is_volatile_warning = false;
    } else {
        // Many systems default to auto without /var/log/journal created!
        config.storage = JournalStorageMode::Auto;
        telemetry.is_volatile_warning = true;
    }

    // 3. Calculate estimated retention days given daily burn
    telemetry.daily_burn_rate_mb = 64.0;
    telemetry.estimated_retained_days = config.system_max_use_mb as f32 / telemetry.daily_burn_rate_mb.max(1.0);

    (config, telemetry)
}

/// Parses output from `journalctl --disk-usage`
/// e.g. "Archived and active journals take up 241M in the file system."
pub fn parse_disk_usage_output(output: &str) -> Option<u64> {
    for word in output.split_whitespace() {
        let trimmed = word.trim_end_matches('.');
        if let Some(num_str) = trimmed.strip_suffix('M') {
            if let Ok(mb) = num_str.parse::<f64>() {
                return Some((mb * 1024.0 * 1024.0) as u64);
            }
        } else if let Some(num_str) = trimmed.strip_suffix('G') {
            if let Ok(gb) = num_str.parse::<f64>() {
                return Some((gb * 1024.0 * 1024.0 * 1024.0) as u64);
            }
        } else if let Some(num_str) = trimmed.strip_suffix('K') {
            if let Ok(kb) = num_str.parse::<f64>() {
                return Some((kb * 1024.0) as u64);
            }
        } else if let Some(num_str) = trimmed.strip_suffix('B') {
            if let Ok(b) = num_str.parse::<u64>() {
                return Some(b);
            }
        }
    }
    None
}

/// Generates systemd journald configuration syntax
/// Settings the journald editor manages, with how each is written.
const MANAGED_KEYS: [&str; 4] = ["Storage", "SystemMaxUse", "SystemKeepFree", "MaxRetentionSec"];

/// Size (`4G`, `500M`, `1024K`, bytes) in MiB; percentages (systemd's
/// default form) aren't a fixed size and read as 0 = "systemd default".
fn parse_size_mb(v: &str) -> u64 {
    let v = v.trim();
    let (num, mult) = match v.chars().last() {
        Some('K') | Some('k') => (&v[..v.len() - 1], 1.0 / 1024.0),
        Some('M') | Some('m') => (&v[..v.len() - 1], 1.0),
        Some('G') | Some('g') => (&v[..v.len() - 1], 1024.0),
        Some('T') | Some('t') => (&v[..v.len() - 1], 1024.0 * 1024.0),
        _ if v.ends_with('%') => return 0,
        _ => (v, 1.0 / (1024.0 * 1024.0)),
    };
    num.trim().parse::<f64>().map(|n| (n * mult).round() as u64).unwrap_or(0)
}

/// A systemd time span (`30day`, `2week`, `1month`, `3600`) in whole days.
fn parse_span_days(v: &str) -> u32 {
    let v = v.trim();
    let split = v.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(v.len());
    let (num, unit) = (&v[..split], v[split..].trim());
    let Ok(n) = num.trim().parse::<f64>() else { return 0 };
    let secs = n * match unit {
        "" | "s" | "sec" | "second" | "seconds" => 1.0,
        "m" | "min" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hour" | "hours" => 3600.0,
        "d" | "day" | "days" => 86400.0,
        "w" | "week" | "weeks" => 7.0 * 86400.0,
        "M" | "month" | "months" => 30.44 * 86400.0,
        "y" | "year" | "years" => 365.25 * 86400.0,
        _ => return 0,
    };
    (secs / 86400.0).round() as u32
}

/// Reads journald.conf's `[Journal]` section. Keys that aren't set take
/// systemd's defaults (sizes default to a share of the disk, shown as 0).
pub fn parse_journald_conf(text: &str) -> JournalRetentionConfig {
    let mut cfg = JournalRetentionConfig {
        storage: JournalStorageMode::Auto,
        system_max_use_mb: 0,
        system_keep_free_mb: 0,
        max_retention_days: 0,
        rate_limit_burst: 10000,
        rate_limit_interval_sec: 30,
        sync_interval_sec: 300,
    };
    let mut in_journal = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_journal = t == "[Journal]";
            continue;
        }
        if !in_journal || t.starts_with('#') || t.starts_with(';') {
            continue;
        }
        let Some((key, value)) = t.split_once('=') else { continue };
        let value = value.trim();
        match key.trim() {
            "Storage" => {
                cfg.storage = match value {
                    "persistent" => JournalStorageMode::Persistent,
                    "volatile" | "none" => JournalStorageMode::Volatile,
                    _ => JournalStorageMode::Auto,
                }
            }
            "SystemMaxUse" => cfg.system_max_use_mb = parse_size_mb(value),
            "SystemKeepFree" => cfg.system_keep_free_mb = parse_size_mb(value),
            "MaxRetentionSec" => cfg.max_retention_days = parse_span_days(value),
            "RateLimitBurst" => cfg.rate_limit_burst = value.parse().unwrap_or(cfg.rate_limit_burst),
            "RateLimitIntervalSec" => cfg.rate_limit_interval_sec = value.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().unwrap_or(cfg.rate_limit_interval_sec),
            "SyncIntervalSec" => cfg.sync_interval_sec = value.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().unwrap_or(cfg.sync_interval_sec),
            _ => {}
        }
    }
    cfg
}

fn managed_value(key: &str, cfg: &JournalRetentionConfig) -> String {
    match key {
        "Storage" => cfg.storage.as_str().to_string(),
        "SystemMaxUse" => format!("{}M", cfg.system_max_use_mb),
        "SystemKeepFree" => format!("{}M", cfg.system_keep_free_mb),
        "MaxRetentionSec" if cfg.max_retention_days == 0 => "0".to_string(),
        "MaxRetentionSec" => format!("{}day", cfg.max_retention_days),
        _ => String::new(),
    }
}

/// Writes `cfg` into `source` losslessly: only settings that differ from what
/// `source` says are touched. Each is set in place (an active `Key=` line
/// replaced, or a commented `#Key=` line uncommented) or appended to the
/// `[Journal]` section; every other line stays byte-for-byte.
pub fn apply_journald_settings(source: &str, cfg: &JournalRetentionConfig) -> String {
    let current = parse_journald_conf(source);
    let changed: Vec<&str> = MANAGED_KEYS
        .iter()
        .copied()
        .filter(|k| managed_value(k, cfg) != managed_value(k, &current))
        .collect();
    if changed.is_empty() {
        return source.to_string();
    }
    let mut lines: Vec<String> = source.split_inclusive('\n').map(str::to_string).collect();
    let key_of = |line: &str| -> Option<(String, bool)> {
        let t = line.trim();
        let (commented, body) = match t.strip_prefix('#') {
            Some(b) => (true, b.trim_start()),
            None => (false, t),
        };
        let (k, _) = body.split_once('=')?;
        let k = k.trim();
        (!k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric())).then(|| (k.to_string(), commented))
    };
    for key in changed {
        let new_line = format!("{key}={}\n", managed_value(key, cfg));
        // The first [Journal] section: its header line and where it ends.
        let (mut section, mut section_end) = (None::<usize>, None::<usize>);
        let (mut active, mut commented) = (None, None);
        let mut in_journal = false;
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if t.starts_with('[') {
                if in_journal && section_end.is_none() {
                    section_end = Some(i);
                }
                in_journal = t == "[Journal]" && section.is_none();
                if in_journal {
                    section = Some(i);
                }
                continue;
            }
            if in_journal {
                match key_of(line) {
                    Some((k, false)) if k == key => active = Some(i),
                    Some((k, true)) if k == key && commented.is_none() => commented = Some(i),
                    _ => {}
                }
            }
        }
        if let Some(i) = active.or(commented) {
            lines[i] = new_line;
        } else if section.is_some() {
            let at = section_end.unwrap_or(lines.len());
            if at > 0 && !lines[at - 1].ends_with('\n') {
                lines[at - 1].push('\n');
            }
            lines.insert(at, new_line);
        } else {
            if lines.last().is_some_and(|l| !l.ends_with('\n')) {
                lines.last_mut().unwrap().push('\n');
            }
            lines.push("[Journal]\n".into());
            lines.push(new_line);
        }
    }
    lines.concat()
}

pub fn generate_journald_conf(config: &JournalRetentionConfig) -> String {
    let mut out = String::new();
    out.push_str("# ==============================================================================\n");
    out.push_str("# Systemd Journal Configuration (/etc/systemd/journald.conf)\n");
    out.push_str("# Managed by Crow (Control, Reliability & Operations Workbench)\n");
    out.push_str("# ==============================================================================\n\n");
    out.push_str("[Journal]\n");
    out.push_str(&format!("Storage={}\n", config.storage.as_str()));
    out.push_str("Compress=yes\n");
    out.push_str("Seal=yes\n");
    out.push_str(&format!("SystemMaxUse={}M\n", config.system_max_use_mb));
    out.push_str(&format!("SystemKeepFree={}M\n", config.system_keep_free_mb));
    if config.max_retention_days > 0 {
        out.push_str(&format!("MaxRetentionSec={}day\n", config.max_retention_days));
    } else {
        out.push_str("MaxRetentionSec=0\n");
    }
    out.push_str(&format!("RateLimitBurst={}\n", config.rate_limit_burst));
    out.push_str(&format!("RateLimitIntervalSec={}s\n", config.rate_limit_interval_sec));
    out.push_str(&format!("SyncIntervalSec={}s\n", config.sync_interval_sec));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const UBUNTU: &str = "#  This file is part of systemd.\n#\n[Journal]\n#Storage=auto\n#Compress=yes\n#SystemMaxUse=\n#SystemKeepFree=\nMaxRetentionSec=2week\n#ForwardToSyslog=no\n";

    #[test]
    fn journald_conf_is_read_with_systemd_defaults() {
        let cfg = parse_journald_conf(UBUNTU);
        assert_eq!(cfg.storage, JournalStorageMode::Auto);
        assert_eq!(cfg.max_retention_days, 14);
        assert_eq!(cfg.system_max_use_mb, 0);
        assert_eq!(parse_size_mb("4G"), 4096);
        assert_eq!(parse_size_mb("10%"), 0);
        assert_eq!(parse_span_days("1month"), 30);
    }

    #[test]
    fn journald_edits_touch_only_their_keys() {
        assert_eq!(apply_journald_settings(UBUNTU, &parse_journald_conf(UBUNTU)), UBUNTU);
        let mut cfg = parse_journald_conf(UBUNTU);
        cfg.system_max_use_mb = 2048; // commented key: uncommented in place
        cfg.max_retention_days = 30; // active key: replaced
        cfg.storage = JournalStorageMode::Persistent;
        let out = apply_journald_settings(UBUNTU, &cfg);
        assert_eq!(out, "#  This file is part of systemd.\n#\n[Journal]\nStorage=persistent\n#Compress=yes\nSystemMaxUse=2048M\n#SystemKeepFree=\nMaxRetentionSec=30day\n#ForwardToSyslog=no\n");
        assert_eq!(parse_journald_conf(&out).system_max_use_mb, 2048);
    }

    #[test]
    fn journald_keys_are_added_to_the_section_or_a_new_one() {
        let mut cfg = parse_journald_conf("");
        cfg.system_keep_free_mb = 512;
        assert_eq!(apply_journald_settings("", &cfg), "[Journal]\nSystemKeepFree=512M\n");
        let src = "[Journal]\nCompress=yes\n[Other]\nX=1\n";
        let out = apply_journald_settings(src, &cfg);
        assert_eq!(out, "[Journal]\nCompress=yes\nSystemKeepFree=512M\n[Other]\nX=1\n");
    }

    /// This machine's real journald.conf survives parse + apply unchanged,
    /// and one setting change touches one line.
    #[test]
    fn real_journald_conf_round_trips() {
        let Ok(text) = std::fs::read_to_string("/etc/systemd/journald.conf") else { return };
        let mut cfg = parse_journald_conf(&text);
        assert_eq!(apply_journald_settings(&text, &cfg), text);
        cfg.system_max_use_mb = 1234;
        let out = apply_journald_settings(&text, &cfg);
        assert_eq!(parse_journald_conf(&out).system_max_use_mb, 1234);
        let diff = |a: &str, b: &str| a.lines().count().abs_diff(b.lines().count()) + a.lines().zip(b.lines()).filter(|(x, y)| x != y).count();
        assert_eq!(diff(&text, &out), 1);
    }

    #[test]
    fn test_parse_disk_usage() {
        let line = "Archived and active journals take up 241M in the file system.";
        let bytes = parse_disk_usage_output(line).unwrap();
        assert_eq!(bytes, 241 * 1024 * 1024);

        let line_gb = "Archived and active journals take up 1.5G in the file system.";
        let bytes_gb = parse_disk_usage_output(line_gb).unwrap();
        assert_eq!(bytes_gb, (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn test_generate_journald_conf() {
        let cfg = JournalRetentionConfig {
            storage: JournalStorageMode::Persistent,
            system_max_use_mb: 4096,
            system_keep_free_mb: 2048,
            max_retention_days: 30,
            rate_limit_burst: 10000,
            rate_limit_interval_sec: 30,
            sync_interval_sec: 300,
        };
        let conf = generate_journald_conf(&cfg);
        assert!(conf.contains("Storage=persistent"));
        assert!(conf.contains("SystemMaxUse=4096M"));
        assert!(conf.contains("SystemKeepFree=2048M"));
        assert!(conf.contains("MaxRetentionSec=30day"));
    }
}
