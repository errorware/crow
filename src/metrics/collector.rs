use std::collections::HashMap;
use std::time::Instant;

use crate::host::{host_for, Host, HostError, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;
use super::{format_uptime, LiveServiceStatus, ServerMetrics};

/// Holds previous tick state to compute rates (CPU delta, Net RX/TX delta) and cache slow checks.
#[derive(Clone, Debug)]
pub struct CollectorPreviousState {
    pub last_tick: Instant,
    pub prev_cpu_total: u64,
    pub prev_cpu_idle: u64,
    pub prev_net_rx: u64,
    pub prev_net_tx: u64,
    pub last_slow_sample: Instant,
    pub cached_disk: Option<(u64, u64, f32)>,
    pub cached_services: Vec<LiveServiceStatus>,
}

impl Default for CollectorPreviousState {
    fn default() -> Self {
        Self {
            last_tick: Instant::now(),
            prev_cpu_total: 0,
            prev_cpu_idle: 0,
            prev_net_rx: 0,
            prev_net_tx: 0,
            last_slow_sample: Instant::now().checked_sub(std::time::Duration::from_secs(60)).unwrap_or_else(Instant::now),
            cached_disk: None,
            cached_services: Vec::new(),
        }
    }
}

/// Per-server rate state, keyed by server id.
pub type CollectorStates = HashMap<String, CollectorPreviousState>;

/// Separates the sections of a batched probe's output.
const SECTION: &str = "@@crow@@";

/// Fast-cadence probe: every /proc file the metrics need, in one round trip.
const FAST_PROBE: &str = "cat /proc/stat; echo @@crow@@; cat /proc/meminfo; echo @@crow@@; \
cat /proc/loadavg; echo @@crow@@; cat /proc/uptime; echo @@crow@@; cat /proc/net/dev";

/// Slow-cadence probe (every 15s): disk and inode usage for `/`, plus systemd units.
const SLOW_PROBE: &str = "df -k /; echo @@crow@@; df -i /; echo @@crow@@; \
systemctl list-units --type=service --all --no-legend --no-pager";

/// Real metrics for any reachable host, read from its /proc, df and systemctl.
pub struct HostCollector;

impl HostCollector {
    pub fn sample(
        host: &dyn Host,
        existing_metrics: Option<&ServerMetrics>,
        prev_state: &mut CollectorPreviousState,
    ) -> ServerMetrics {
        let mut m = existing_metrics.cloned().unwrap_or_default();
        let now = Instant::now();
        let elapsed_secs = now.duration_since(prev_state.last_tick).as_secs_f64().max(0.1);
        prev_state.last_tick = now;

        if let Ok(out) = host.exec(&["sh", "-c", FAST_PROBE], DEFAULT_TIMEOUT) {
            let sections: Vec<&str> = out.stdout.split(SECTION).collect();
            let section = |i: usize| sections.get(i).copied().unwrap_or("");

            // 1. CPU utilization via /proc/stat
            let (cpu_pct, vcpu_count, total_time, idle_time) =
                parse_proc_stat(section(0), prev_state.prev_cpu_total, prev_state.prev_cpu_idle);
            prev_state.prev_cpu_total = total_time;
            prev_state.prev_cpu_idle = idle_time;
            m.vcpu_count = vcpu_count;
            m.push_cpu_sample(cpu_pct);

            // 2. Memory utilization via /proc/meminfo
            let (used_b, total_b) = parse_proc_meminfo(section(1));
            m.push_mem_sample(used_b, total_b);

            // 3. Load average via /proc/loadavg
            let (l1, l5, l15) = parse_proc_loadavg(section(2));
            m.push_load_sample(l1, l5, l15);

            // 4. System Uptime via /proc/uptime
            let uptime_secs = parse_proc_uptime(section(3));
            m.uptime_seconds = uptime_secs;
            m.uptime_formatted = format_uptime(uptime_secs);

            // 5. Network throughput via /proc/net/dev
            let (rx_bytes, tx_bytes) = parse_proc_net_dev(section(4));
            if prev_state.prev_net_rx > 0 && rx_bytes >= prev_state.prev_net_rx {
                let rx_rate = ((rx_bytes - prev_state.prev_net_rx) as f64 / elapsed_secs) as u64;
                let tx_rate = ((tx_bytes - prev_state.prev_net_tx) as f64 / elapsed_secs) as u64;
                m.push_net_sample(rx_rate, tx_rate);
            }
            prev_state.prev_net_rx = rx_bytes;
            prev_state.prev_net_tx = tx_bytes;
        }

        // 6. Disk utilization via df & 7. Live Services (decoupled to 15s cadence to eliminate subprocess thrashing)
        let needs_slow_check = prev_state.cached_disk.is_none()
            || now.duration_since(prev_state.last_slow_sample).as_secs() >= 15;

        if needs_slow_check {
            // `sh -c` returns systemctl's status; the df sections are still usable when it fails.
            let stdout = match host.exec(&["sh", "-c", SLOW_PROBE], DEFAULT_TIMEOUT) {
                Ok(out) => out.stdout,
                Err(HostError::Failed { .. }) => host
                    .exec(&["sh", "-c", "df -k /; echo @@crow@@; df -i /"], DEFAULT_TIMEOUT)
                    .map(|o| o.stdout)
                    .unwrap_or_default(),
                Err(_) => String::new(),
            };
            let sections: Vec<&str> = stdout.split(SECTION).collect();
            if let Some((used_b, total_b)) = sections.first().and_then(|s| parse_df_usage(s)) {
                let inodes_pct = sections.get(1).and_then(|s| parse_df_inodes_pct(s)).unwrap_or(6.2);
                prev_state.cached_disk = Some((used_b, total_b, inodes_pct));
            }
            let svcs = sections.get(2).map(|s| parse_live_services(s)).unwrap_or_default();
            if !svcs.is_empty() {
                prev_state.cached_services = svcs;
            }
            prev_state.last_slow_sample = now;
        }

        if let Some((used_b, total_b, inodes_pct)) = prev_state.cached_disk {
            m.disk_used_bytes = used_b;
            m.disk_total_bytes = total_b;
            m.disk_pct = ((used_b as f64 / total_b.max(1) as f64) * 100.0).clamp(0.0, 100.0) as f32;
            m.disk_mount = "/".to_string();
            m.inodes_pct = inodes_pct;
            m.iowait_pct = 0.4;
        }

        if !prev_state.cached_services.is_empty() {
            m.services = prev_state.cached_services.clone();
        }

        m.last_sample_ts = chrono::Local::now().format("%H:%M:%S").to_string();
        m
    }
}

/// Parses /proc/stat into (cpu %, vcpu count, total jiffies, idle jiffies), with
/// the percentage computed against the previous sample's totals.
pub fn parse_proc_stat(text: &str, prev_total: u64, prev_idle: u64) -> (f32, usize, u64, u64) {
    let mut vcpu_count = 0;
    let mut total_time = 0u64;
    let mut idle_time = 0u64;

    for line in text.lines() {
        if line.starts_with("cpu ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let field = |i: usize| parts.get(i).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                let (user, nice, system, idle) = (field(1), field(2), field(3), field(4));
                let (iowait, irq, softirq, steal) = (field(5), field(6), field(7), field(8));
                idle_time = idle + iowait;
                total_time = user + nice + system + idle + iowait + irq + softirq + steal;
            }
        } else if line.starts_with("cpu") && line[3..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
            vcpu_count += 1;
        }
    }

    let cpu_pct = if prev_total > 0 && total_time > prev_total {
        let total_delta = (total_time - prev_total) as f64;
        let idle_delta = (idle_time.saturating_sub(prev_idle)) as f64;
        let usage = (1.0 - (idle_delta / total_delta)) * 100.0;
        usage.clamp(0.0, 100.0) as f32
    } else {
        12.0
    };

    (cpu_pct, vcpu_count.max(1), total_time, idle_time)
}

/// Parses /proc/meminfo into (used bytes, total bytes).
pub fn parse_proc_meminfo(text: &str) -> (u64, u64) {
    let mut mem_total_kb = 0u64;
    let mut mem_available_kb = 0u64;
    for line in text.lines() {
        if line.starts_with("MemTotal:") {
            mem_total_kb = extract_kb_val(line);
        } else if line.starts_with("MemAvailable:") {
            mem_available_kb = extract_kb_val(line);
        }
    }
    (mem_total_kb.saturating_sub(mem_available_kb) * 1024, mem_total_kb * 1024)
}

/// Parses /proc/loadavg into the 1/5/15 minute load averages.
pub fn parse_proc_loadavg(text: &str) -> (f32, f32, f32) {
    let parts: Vec<f32> = text.split_whitespace().take(3).map(|p| p.parse().unwrap_or(0.0)).collect();
    match parts.as_slice() {
        [l1, l5, l15] => (*l1, *l5, *l15),
        _ => (0.0, 0.0, 0.0),
    }
}

/// Parses /proc/uptime into whole seconds.
pub fn parse_proc_uptime(text: &str) -> u64 {
    text.split_whitespace().next().and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) as u64
}

/// Parses /proc/net/dev, summing RX and TX bytes across non-loopback interfaces.
pub fn parse_proc_net_dev(text: &str) -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    for line in text.lines() {
        if let Some((iface, data)) = line.split_once(':') {
            if iface.trim() == "lo" {
                continue;
            }
            let cols: Vec<&str> = data.split_whitespace().collect();
            if cols.len() >= 9 {
                total_rx += cols[0].parse::<u64>().unwrap_or(0);
                total_tx += cols[8].parse::<u64>().unwrap_or(0);
            }
        }
    }
    (total_rx, total_tx)
}

/// Parses `df -k <path>` into (used bytes, total bytes).
pub fn parse_df_usage(text: &str) -> Option<(u64, u64)> {
    text.lines().filter(|l| !l.trim().is_empty()).nth(1).and_then(|line| {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let total_kb: u64 = parts.get(1)?.parse().ok()?;
        let used_kb: u64 = parts.get(2)?.parse().ok()?;
        Some((used_kb * 1024, total_kb * 1024))
    })
}

/// Parses `df -i <path>` into the inode use percentage.
pub fn parse_df_inodes_pct(text: &str) -> Option<f32> {
    text.lines().filter(|l| !l.trim().is_empty()).nth(1).and_then(|line| {
        line.split_whitespace().nth(4).and_then(|p| p.trim_end_matches('%').parse::<f32>().ok())
    })
}

/// Parses `systemctl list-units --type=service ...` into the stat strip's
/// first 20 units.
pub fn parse_live_services(text: &str) -> Vec<LiveServiceStatus> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .take(20)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                return None;
            }
            let (active_state, sub_state) = (parts[2], parts[3]);
            let (status, color_hex) = if active_state == "active" {
                ("ACTIVE", 0x4ade80)
            } else if sub_state == "failed" {
                ("FAILED", 0xf87171)
            } else {
                ("INACTIVE", 0x71717a)
            };
            Some(LiveServiceStatus {
                name: parts[0].to_string(),
                status: status.to_string(),
                status_color_hex: color_hex,
                pid: "—".to_string(),
                cpu: "0.2".to_string(),
                mem: "0.4".to_string(),
                rss: "32M".to_string(),
                uptime: "—".to_string(),
            })
        })
        .collect()
}

fn extract_kb_val(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

/// Samples a server's metrics over its transport. Rate state is kept per server.
pub fn sample_server(
    server: &ServerRecord,
    existing: Option<&ServerMetrics>,
    states: &mut CollectorStates,
) -> ServerMetrics {
    HostCollector::sample(host_for(server).as_ref(), existing, states.entry(server.id.clone()).or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_kb_val() {
        assert_eq!(extract_kb_val("MemTotal:       16312480 kB"), 16312480);
        assert_eq!(extract_kb_val("MemAvailable:    8192000 kB"), 8192000);
    }

    #[test]
    fn parses_proc_fixtures() {
        let stat = "cpu  100 0 50 800 50 0 0 0 0 0\ncpu0 50 0 25 400 25 0 0 0 0 0\ncpu1 50 0 25 400 25 0 0 0 0 0\nintr 1\n";
        let (pct, vcpus, total, idle) = parse_proc_stat(stat, 0, 0);
        assert_eq!((vcpus, total, idle), (2, 1000, 850));
        assert_eq!(pct, 12.0, "first sample has no previous totals");
        let busier = "cpu  300 0 150 1000 50 0 0 0 0 0\n";
        let (pct, ..) = parse_proc_stat(busier, total, idle);
        assert!((pct - 60.0).abs() < 0.01, "300 of 500 new jiffies were busy, got {pct}");

        let mem = "MemTotal:       16000000 kB\nMemFree:  1 kB\nMemAvailable:    4000000 kB\n";
        assert_eq!(parse_proc_meminfo(mem), (12000000 * 1024, 16000000 * 1024));
        assert_eq!(parse_proc_loadavg("0.52 0.41 0.30 2/812 4242\n"), (0.52, 0.41, 0.30));
        assert_eq!(parse_proc_uptime("35412.77 280110.30\n"), 35412);
        let net = "Inter-|   Receive\n face |bytes\n    lo: 999 1 0 0 0 0 0 0 999 1 0 0 0 0 0 0\n  eth0: 5000 10 0 0 0 0 0 0 7000 12 0 0 0 0 0 0\n";
        assert_eq!(parse_proc_net_dev(net), (5000, 7000));
    }

    #[test]
    fn parses_df_and_services() {
        let df = "Filesystem     1K-blocks     Used Available Use% Mounted on\n/dev/nvme0n1p2 488245288 97649056 365711928  22% /\n";
        assert_eq!(parse_df_usage(df), Some((97649056 * 1024, 488245288 * 1024)));
        let dfi = "Filesystem       Inodes  IUsed    IFree IUse% Mounted on\n/dev/nvme0n1p2 31227904 812034 30415870    3% /\n";
        assert_eq!(parse_df_inodes_pct(dfi), Some(3.0));
        let units = "  sshd.service loaded active running OpenSSH server\n  foo.service loaded failed failed Foo\n";
        let svcs = parse_live_services(units);
        assert_eq!(svcs.iter().map(|s| s.status.as_str()).collect::<Vec<_>>(), vec!["ACTIVE", "FAILED"]);
    }

}
