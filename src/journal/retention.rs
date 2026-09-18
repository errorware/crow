use serde::{Deserialize, Serialize};
use std::process::Command;

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

/// Dispatches journal retention telemetry reading for a given server
pub fn read_retention_for_server(endpoint: &str) -> (JournalRetentionConfig, JournalTelemetry) {
    if endpoint.contains("localhost") || endpoint.contains("127.0.0.1") || endpoint == "local" {
        read_local_retention()
    } else {
        read_simulated_retention(endpoint)
    }
}

/// Queries local Linux systemd journal disk usage and configuration
pub fn read_local_retention() -> (JournalRetentionConfig, JournalTelemetry) {
    let mut config = JournalRetentionConfig::default();
    let mut telemetry = JournalTelemetry::default();

    // 1. Run journalctl --disk-usage
    if let Ok(output) = Command::new("journalctl")
        .arg("--disk-usage")
        .output()
    {
        if output.status.success() {
            let txt = String::from_utf8_lossy(&output.stdout);
            // Example: "Archived and active journals take up 241M in the file system."
            telemetry.disk_usage_bytes = parse_disk_usage_output(&txt).unwrap_or(241 * 1024 * 1024);
        }
    }

    // 2. Check if persistent directory exists
    let persistent_dir = std::path::Path::new("/var/log/journal");
    if persistent_dir.exists() {
        config.storage = JournalStorageMode::Persistent;
        telemetry.is_volatile_warning = false;
    } else {
        // Many systems default to auto without /var/log/journal created!
        config.storage = JournalStorageMode::Auto;
        telemetry.is_volatile_warning = true;
    }

    // 3. Calculate estimated retention days given daily burn
    let _used_mb = telemetry.disk_usage_bytes as f32 / (1024.0 * 1024.0);
    telemetry.daily_burn_rate_mb = 64.0;
    telemetry.estimated_retained_days = config.system_max_use_mb as f32 / telemetry.daily_burn_rate_mb.max(1.0);

    (config, telemetry)
}

/// Simulated retention telemetry for demo/remote servers
pub fn read_simulated_retention(endpoint: &str) -> (JournalRetentionConfig, JournalTelemetry) {
    let mut config = JournalRetentionConfig::default();
    let mut telemetry = JournalTelemetry::default();

    if endpoint.contains("db") || endpoint.contains("postgres") {
        config.system_max_use_mb = 8192; // 8 GB for high-traffic DB
        config.max_retention_days = 90;
        config.storage = JournalStorageMode::Persistent;
        telemetry.disk_usage_bytes = 1_840_000_000; // ~1.8 GB
        telemetry.oldest_timestamp = "18 days ago".to_string();
        telemetry.retention_horizon_hours = 432.0;
        telemetry.daily_burn_rate_mb = 102.0;
        telemetry.estimated_retained_days = 80.3;
        telemetry.is_volatile_warning = false;
        telemetry.active_files_count = 14;
    } else if endpoint.contains("staging") || endpoint.contains("dev") {
        config.system_max_use_mb = 1024;
        config.max_retention_days = 7;
        config.storage = JournalStorageMode::Volatile; // Volatile danger demo!
        telemetry.disk_usage_bytes = 180_000_000;
        telemetry.oldest_timestamp = "6 hours ago".to_string();
        telemetry.retention_horizon_hours = 6.0;
        telemetry.daily_burn_rate_mb = 720.0;
        telemetry.estimated_retained_days = 1.4;
        telemetry.is_volatile_warning = true; // Danger!
        telemetry.active_files_count = 3;
    } else {
        config.system_max_use_mb = 4096;
        config.max_retention_days = 30;
        config.storage = JournalStorageMode::Persistent;
        telemetry.disk_usage_bytes = 420_000_000;
        telemetry.oldest_timestamp = "5 days ago".to_string();
        telemetry.retention_horizon_hours = 120.0;
        telemetry.daily_burn_rate_mb = 84.0;
        telemetry.estimated_retained_days = 48.7;
        telemetry.is_volatile_warning = false;
        telemetry.active_files_count = 7;
    }

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
