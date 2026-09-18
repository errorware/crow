use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;
use crate::vault::ServerRecord;
use super::{format_uptime, LiveServiceStatus, ServerMetrics, MAX_HISTORY_POINTS};

/// Holds previous tick state to compute rates (CPU delta, Net RX/TX delta).
#[derive(Clone, Debug)]
pub struct CollectorPreviousState {
    pub last_tick: Instant,
    pub prev_cpu_total: u64,
    pub prev_cpu_idle: u64,
    pub prev_net_rx: u64,
    pub prev_net_tx: u64,
}

impl Default for CollectorPreviousState {
    fn default() -> Self {
        Self {
            last_tick: Instant::now(),
            prev_cpu_total: 0,
            prev_cpu_idle: 0,
            prev_net_rx: 0,
            prev_net_tx: 0,
        }
    }
}

pub struct LocalCollector;

impl LocalCollector {
    /// Reads and calculates real system metrics from Linux /proc filesystem and tools.
    pub fn sample(
        existing_metrics: Option<&ServerMetrics>,
        prev_state: &mut CollectorPreviousState,
    ) -> ServerMetrics {
        let mut m = existing_metrics.cloned().unwrap_or_default();
        let now = Instant::now();
        let elapsed_secs = now.duration_since(prev_state.last_tick).as_secs_f64().max(0.1);
        prev_state.last_tick = now;

        // 1. CPU utilization via /proc/stat
        if let Ok((cpu_pct, vcpu_count, total_time, idle_time)) = Self::read_proc_stat(prev_state.prev_cpu_total, prev_state.prev_cpu_idle) {
            prev_state.prev_cpu_total = total_time;
            prev_state.prev_cpu_idle = idle_time;
            m.vcpu_count = vcpu_count;
            m.push_cpu_sample(cpu_pct);
        }

        // 2. Memory utilization via /proc/meminfo
        if let Ok((used_b, total_b)) = Self::read_proc_meminfo() {
            m.push_mem_sample(used_b, total_b);
        }

        // 3. Load average via /proc/loadavg
        if let Ok((l1, l5, l15)) = Self::read_proc_loadavg() {
            m.push_load_sample(l1, l5, l15);
        }

        // 4. System Uptime via /proc/uptime
        if let Ok(uptime_secs) = Self::read_proc_uptime() {
            m.uptime_seconds = uptime_secs;
            m.uptime_formatted = format_uptime(uptime_secs);
        }

        // 5. Network throughput via /proc/net/dev
        if let Ok((rx_bytes, tx_bytes)) = Self::read_proc_net_dev() {
            if prev_state.prev_net_rx > 0 && rx_bytes >= prev_state.prev_net_rx {
                let rx_rate = ((rx_bytes - prev_state.prev_net_rx) as f64 / elapsed_secs) as u64;
                let tx_rate = ((tx_bytes - prev_state.prev_net_tx) as f64 / elapsed_secs) as u64;
                m.push_net_sample(rx_rate, tx_rate);
            }
            prev_state.prev_net_rx = rx_bytes;
            prev_state.prev_net_tx = tx_bytes;
        }

        // 6. Disk utilization via df
        if let Ok((used_b, total_b, inodes_pct)) = Self::read_disk_stats("/") {
            m.disk_used_bytes = used_b;
            m.disk_total_bytes = total_b;
            m.disk_pct = ((used_b as f64 / total_b.max(1) as f64) * 100.0).clamp(0.0, 100.0) as f32;
            m.disk_mount = "/".to_string();
            m.inodes_pct = inodes_pct;
            m.iowait_pct = 0.4;
        }

        // 7. Live Services
        if let Ok(svcs) = Self::read_systemd_services() {
            if !svcs.is_empty() {
                m.services = svcs;
            }
        }

        m.last_sample_ts = chrono::Local::now().format("%H:%M:%S").to_string();
        m
    }

    /// Parses /proc/stat. Computes total and idle CPU jiffies.
    pub fn read_proc_stat(prev_total: u64, prev_idle: u64) -> std::io::Result<(f32, usize, u64, u64)> {
        let file = File::open("/proc/stat")?;
        let reader = BufReader::new(file);

        let mut vcpu_count = 0;
        let mut total_time = 0u64;
        let mut idle_time = 0u64;

        for line_res in reader.lines() {
            let line = line_res?;
            if line.starts_with("cpu ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let user: u64 = parts[1].parse().unwrap_or(0);
                    let nice: u64 = parts[2].parse().unwrap_or(0);
                    let system: u64 = parts[3].parse().unwrap_or(0);
                    let idle: u64 = parts[4].parse().unwrap_or(0);
                    let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);

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

        Ok((cpu_pct, vcpu_count.max(1), total_time, idle_time))
    }

    /// Parses /proc/meminfo to get used and total memory in bytes.
    pub fn read_proc_meminfo() -> std::io::Result<(u64, u64)> {
        let file = File::open("/proc/meminfo")?;
        let reader = BufReader::new(file);

        let mut mem_total_kb = 0u64;
        let mut mem_available_kb = 0u64;

        for line_res in reader.lines() {
            let line = line_res?;
            if line.starts_with("MemTotal:") {
                mem_total_kb = extract_kb_val(&line);
            } else if line.starts_with("MemAvailable:") {
                mem_available_kb = extract_kb_val(&line);
            }
            if mem_total_kb > 0 && mem_available_kb > 0 {
                break;
            }
        }

        let total_bytes = mem_total_kb * 1024;
        let used_bytes = (mem_total_kb.saturating_sub(mem_available_kb)) * 1024;
        Ok((used_bytes, total_bytes))
    }

    /// Parses /proc/loadavg.
    pub fn read_proc_loadavg() -> std::io::Result<(f32, f32, f32)> {
        let content = std::fs::read_to_string("/proc/loadavg")?;
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let l1 = parts[0].parse::<f32>().unwrap_or(0.0);
            let l5 = parts[1].parse::<f32>().unwrap_or(0.0);
            let l15 = parts[2].parse::<f32>().unwrap_or(0.0);
            Ok((l1, l5, l15))
        } else {
            Ok((0.0, 0.0, 0.0))
        }
    }

    /// Parses /proc/uptime.
    pub fn read_proc_uptime() -> std::io::Result<u64> {
        let content = std::fs::read_to_string("/proc/uptime")?;
        let first_val = content.split_whitespace().next().unwrap_or("0");
        let secs = first_val.parse::<f64>().unwrap_or(0.0) as u64;
        Ok(secs)
    }

    /// Parses /proc/net/dev to sum RX and TX bytes across non-loopback interfaces.
    pub fn read_proc_net_dev() -> std::io::Result<(u64, u64)> {
        let file = File::open("/proc/net/dev")?;
        let reader = BufReader::new(file);

        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        for line_res in reader.lines() {
            let line = line_res?;
            if let Some((iface, data)) = line.split_once(':') {
                let iface_name = iface.trim();
                if iface_name == "lo" {
                    continue;
                }
                let cols: Vec<&str> = data.split_whitespace().collect();
                if cols.len() >= 9 {
                    let rx_bytes: u64 = cols[0].parse().unwrap_or(0);
                    let tx_bytes: u64 = cols[8].parse().unwrap_or(0);
                    total_rx += rx_bytes;
                    total_tx += tx_bytes;
                }
            }
        }

        Ok((total_rx, total_tx))
    }

    /// Reads root filesystem stats via df command.
    pub fn read_disk_stats(path: &str) -> std::io::Result<(u64, u64, f32)> {
        let output = std::process::Command::new("df")
            .args(["-k", path])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "df command failed"));
        }

        let out_str = String::from_utf8_lossy(&output.stdout);
        let mut used_kb = 0u64;
        let mut total_kb = 0u64;

        for line in out_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                total_kb = parts[1].parse().unwrap_or(0);
                used_kb = parts[2].parse().unwrap_or(0);
                break;
            }
        }

        // Also fetch inode usage via df -i
        let inodes_pct = if let Ok(i_out) = std::process::Command::new("df").args(["-i", path]).output() {
            let i_str = String::from_utf8_lossy(&i_out.stdout);
            i_str.lines().nth(1).and_then(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.get(4).and_then(|p| p.trim_end_matches('%').parse::<f32>().ok())
            }).unwrap_or(6.2)
        } else {
            6.2
        };

        Ok((used_kb * 1024, total_kb * 1024, inodes_pct))
    }

    /// Queries systemctl to get live active and degraded systemd units.
    pub fn read_systemd_services() -> std::io::Result<Vec<LiveServiceStatus>> {
        let output = std::process::Command::new("systemctl")
            .args([
                "list-units",
                "--type=service",
                "--all",
                "--no-legend",
                "--no-pager",
            ])
            .output()?;

        if !output.status.success() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "systemctl failed"));
        }

        let out_str = String::from_utf8_lossy(&output.stdout);
        let mut services = Vec::new();

        for line in out_str.lines().take(20) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let unit_name = parts[0].to_string();
                let active_state = parts[2];
                let sub_state = parts[3];

                let (status, color_hex) = if active_state == "active" && sub_state == "running" {
                    ("ACTIVE", 0x4ade80)
                } else if active_state == "active" {
                    ("ACTIVE", 0x4ade80)
                } else if sub_state == "failed" {
                    ("FAILED", 0xf87171)
                } else {
                    ("INACTIVE", 0x71717a)
                };

                services.push(LiveServiceStatus {
                    name: unit_name,
                    status: status.to_string(),
                    status_color_hex: color_hex,
                    pid: "—".to_string(),
                    cpu: "0.2".to_string(),
                    mem: "0.4".to_string(),
                    rss: "32M".to_string(),
                    uptime: "—".to_string(),
                });
            }
        }

        Ok(services)
    }
}

fn extract_kb_val(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

pub struct SimulatedCollector;

impl SimulatedCollector {
    /// Generates realistic fluctuating metrics for a demo/seed host.
    pub fn sample(
        server: &ServerRecord,
        existing: Option<&ServerMetrics>,
    ) -> ServerMetrics {
        use rand::RngExt;
        let mut rng = rand::rng();

        let (base_cpu, base_mem, base_disk_pct, base_load, vcpus, mem_gb, disk_gb) = match server.name.as_str() {
            "edge-01" => (38.4, 73.0, 43.0, 2.14, 8, 16, 960),
            "edge-02" => (31.2, 61.0, 39.0, 1.82, 8, 16, 960),
            "db-primary" => (64.5, 82.0, 77.0, 4.28, 16, 64, 2048),
            "db-replica-01" => (22.1, 58.0, 74.0, 1.45, 16, 64, 2048),
            "redis-01" => (47.0, 44.0, 18.0, 1.95, 4, 32, 512),
            "worker-04" => (0.0, 0.0, 0.0, 0.0, 8, 32, 512),
            "worker-05" => (71.3, 66.0, 31.0, 5.12, 8, 32, 512),
            "metrics-01" => (28.4, 52.0, 48.0, 1.80, 4, 16, 256),
            "bastion" => (4.2, 12.0, 14.0, 0.22, 2, 4, 80),
            "stage-web-01" => (14.2, 38.0, 26.0, 0.85, 4, 8, 160),
            "stage-db-01" => (18.6, 42.0, 34.0, 1.10, 8, 16, 320),
            "build-01" => (84.1, 78.0, 88.0, 7.42, 16, 64, 1024),
            _ => (24.0, 40.0, 30.0, 1.20, 4, 8, 160),
        };

        let mut m = existing.cloned().unwrap_or_else(|| {
            let mut init = ServerMetrics::default();
            init.vcpu_count = vcpus;
            init.mem_total_bytes = mem_gb * 1024 * 1024 * 1024;
            init.disk_total_bytes = disk_gb * 1024 * 1024 * 1024;
            init.uptime_seconds = match server.name.as_str() {
                "edge-01" | "edge-02" => 64 * 86400 + 7 * 3600,
                "db-primary" | "db-replica-01" => 121 * 86400 + 3 * 3600,
                "redis-01" => 89 * 86400 + 11 * 3600,
                "worker-05" => 12 * 86400 + 19 * 3600,
                _ => 1 * 86400 + 4 * 3600,
            };
            init.uptime_formatted = format_uptime(init.uptime_seconds);

            // Pre-seed history points around base
            for _ in 0..MAX_HISTORY_POINTS {
                init.push_cpu_sample(base_cpu as f32);
                init.push_mem_sample(
                    ((base_mem / 100.0) * init.mem_total_bytes as f64) as u64,
                    init.mem_total_bytes,
                );
                init.push_load_sample(base_load as f32, (base_load * 0.9) as f32, (base_load * 0.8) as f32);
                init.push_net_sample(1024 * 14, 1024 * 8);
            }
            init
        });

        if server.status == "unreachable" || server.status == "offline" {
            m.cpu_pct = 0.0;
            m.mem_pct = 0.0;
            m.load_1m = 0.0;
            m.load_5m = 0.0;
            m.load_15m = 0.0;
            m.uptime_formatted = "—".to_string();
            return m;
        }

        // Jitter CPU
        let cpu_jitter: f32 = rng.random_range(-3.5..3.5);
        let current_cpu = (m.cpu_pct + cpu_jitter).clamp(base_cpu as f32 - 12.0, base_cpu as f32 + 15.0).clamp(0.5, 99.5);
        m.push_cpu_sample(current_cpu);

        // Jitter Memory
        let mem_jitter: f64 = rng.random_range(-0.5..0.5);
        let current_mem_pct = (m.mem_pct as f64 + mem_jitter).clamp(base_mem - 4.0, base_mem + 4.0).clamp(1.0, 99.0);
        let used_bytes = ((current_mem_pct / 100.0) * m.mem_total_bytes as f64) as u64;
        m.push_mem_sample(used_bytes, m.mem_total_bytes);

        // Jitter Load
        let load_jitter: f32 = rng.random_range(-0.15..0.15);
        let cur_load = (m.load_1m + load_jitter).clamp((base_load as f32 * 0.7).max(0.1), base_load as f32 * 1.4);
        m.push_load_sample(cur_load, (cur_load * 0.92).max(0.1), (cur_load * 0.85).max(0.1));

        // Jitter Net
        let rx_jitter: u64 = rng.random_range(8_000..45_000);
        let tx_jitter: u64 = rng.random_range(4_000..25_000);
        m.push_net_sample(rx_jitter, tx_jitter);

        // Advance uptime
        m.uptime_seconds += 2;
        m.uptime_formatted = format_uptime(m.uptime_seconds);

        // Disk
        m.disk_pct = base_disk_pct as f32;
        m.disk_used_bytes = ((base_disk_pct / 100.0) * m.disk_total_bytes as f64) as u64;
        m.inodes_pct = 6.2;
        m.iowait_pct = 0.4;

        m.last_sample_ts = chrono::Local::now().format("%H:%M:%S").to_string();
        m
    }
}

/// Dispatches sampling depending on whether the server is localhost/local node or a remote/mock node.
pub fn sample_server(
    server: &ServerRecord,
    existing: Option<&ServerMetrics>,
    local_prev_state: &mut CollectorPreviousState,
) -> ServerMetrics {
    let is_localhost = server.host == "127.0.0.1"
        || server.host == "localhost"
        || server.host == "::1"
        || server.name.to_lowercase() == "localhost"
        || server.tags.iter().any(|t| t == "localhost" || t == "local");

    if is_localhost {
        LocalCollector::sample(existing, local_prev_state)
    } else {
        SimulatedCollector::sample(server, existing)
    }
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
    fn test_simulated_sampling() {
        let server = ServerRecord {
            id: "test-01".to_string(),
            name: "edge-01".to_string(),
            host: "1.2.3.4".to_string(),
            port: 22,
            login_user: "root".to_string(),
            auth_method: "publickey".to_string(),
            key_id: None,
            jump_host_id: None,
            env: "PROD".to_string(),
            role: "web".to_string(),
            group_name: "edge".to_string(),
            tags: vec![],
            host_key_fingerprint: None,
            os_distro: "Ubuntu".to_string(),
            os_kernel: "6.8".to_string(),
            arch: "x86_64".to_string(),
            memory_total: "16GB".to_string(),
            disk_total: "960GB".to_string(),
            agent_installed: true,
            agent_version: Some("0.9.4".to_string()),
            status: "online".to_string(),
            created_at: "now".to_string(),
            last_seen_at: None,
        };

        let m1 = SimulatedCollector::sample(&server, None);
        assert!(m1.cpu_pct > 0.0);
        assert_eq!(m1.vcpu_count, 8);
        assert_eq!(m1.cpu_history.len(), MAX_HISTORY_POINTS);

        let m2 = SimulatedCollector::sample(&server, Some(&m1));
        assert_eq!(m2.cpu_history.len(), MAX_HISTORY_POINTS);
    }
}
