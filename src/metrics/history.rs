//! Metrics history (ERR-84): one row per server per minute while Crow is
//! open, kept for a week. Only what was measured is stored; a value Crow
//! couldn't read is `None`, never zero.

use super::ServerMetrics;

/// Seconds between history samples.
pub const HISTORY_EVERY_SECS: u64 = 60;
/// How long rows are kept.
pub const HISTORY_KEEP_SECS: i64 = 7 * 86_400;

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryRow {
    pub server_id: String,
    /// Unix seconds.
    pub ts: i64,
    pub reachable: bool,
    /// Why not, when not reachable ("auth failed · Permission denied").
    pub reason: Option<String>,
    pub cpu: Option<f32>,
    pub mem: Option<f32>,
    pub disk: Option<f32>,
    /// Which mount `disk` is for ("/", or "/var" on a read-only root). Not
    /// stored; used to word this round's alerts.
    pub disk_mount: Option<String>,
    pub load1: Option<f32>,
    /// Failed systemd units; `None` where systemd couldn't be read.
    pub failed_services: Option<Vec<String>>,
}

/// A row from a server's newest sample. `down` is the transport's reason
/// when the server couldn't be reached.
pub fn row_from(server_id: &str, ts: i64, m: Option<&ServerMetrics>, down: Option<String>) -> HistoryRow {
    let m = m.filter(|m| m.reachable && down.is_none());
    HistoryRow {
        server_id: server_id.to_string(),
        ts,
        reachable: m.is_some(),
        reason: if m.is_some() { None } else { Some(down.unwrap_or_else(|| "no answer to the metrics probe".into())) },
        cpu: m.filter(|m| m.cpu_known).map(|m| m.cpu_pct),
        mem: m.filter(|m| m.mem_total_bytes > 1).map(|m| m.mem_pct),
        disk: m.filter(|m| m.disk_total_bytes > 1).map(|m| m.disk_pct),
        disk_mount: m.filter(|m| m.disk_total_bytes > 1).map(|m| m.disk_mount.clone()),
        load1: m.map(|m| m.load_1m),
        failed_services: m
            .filter(|m| !m.services.is_empty())
            .map(|m| m.services.iter().filter(|s| s.status == "FAILED").map(|s| s.name.clone()).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::LiveServiceStatus;

    fn svc(name: &str, status: &str) -> LiveServiceStatus {
        LiveServiceStatus { name: name.into(), status: status.into(), status_color_hex: 0, pid: String::new(), cpu: String::new(), mem: String::new(), rss: String::new(), uptime: String::new() }
    }

    #[test]
    fn measured_values_only() {
        let mut m = ServerMetrics { reachable: true, ..Default::default() };
        m.push_mem_sample(4, 8);
        let row = row_from("s1", 100, Some(&m), None);
        assert!(row.reachable);
        assert_eq!((row.cpu, row.mem, row.disk, row.failed_services.clone()), (None, Some(50.0), None, None), "no CPU rate yet, no df, no systemd: unknown, not 0");

        m.push_cpu_sample(12.5);
        m.services = vec![svc("nginx.service", "ACTIVE"), svc("backup.service", "FAILED")];
        let row = row_from("s1", 160, Some(&m), None);
        assert_eq!((row.cpu, row.failed_services), (Some(12.5), Some(vec!["backup.service".to_string()])));
    }

    #[test]
    fn unreachable_rows_carry_the_reason_and_no_numbers() {
        let m = ServerMetrics { reachable: true, cpu_known: true, cpu_pct: 40.0, ..Default::default() };
        let row = row_from("s1", 100, Some(&m), Some("unreachable · Connection refused".into()));
        assert!(!row.reachable);
        assert_eq!((row.cpu, row.reason.as_deref()), (None, Some("unreachable · Connection refused")), "stale numbers from before the outage aren't readings");
        assert_eq!(row_from("s1", 100, None, None).reason.as_deref(), Some("no answer to the metrics probe"));
    }
}
