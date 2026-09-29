use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use gpui_kit::Rgba;
use crate::theme::*;
use crate::views::overview::{ProcessUnit, ServiceUnit, SocketUnit};

pub mod collector;
pub mod history;
pub mod alerts;

pub const MAX_HISTORY_POINTS: usize = 20;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveServiceStatus {
    pub name: String,
    pub status: String,
    pub status_color_hex: u32,
    pub pid: String,
    pub cpu: String,
    pub mem: String,
    pub rss: String,
    pub uptime: String,
}

impl LiveServiceStatus {
    pub fn color(&self) -> Rgba {
        match self.status.to_uppercase().as_str() {
            "ACTIVE" | "RUNNING" => OK,
            "DEGRADED" | "PENDING" => WARN,
            "FAILED" | "INACTIVE" | "DEAD" => CRIT,
            _ => TEXT_MUTED,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerMetrics {
    // 1. CPU
    pub cpu_pct: f32,
    pub vcpu_count: usize,
    pub cpu_peak: f32,
    pub cpu_history: Vec<f32>,

    // 2. Memory
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub mem_pct: f32,
    pub mem_history: Vec<f32>,

    // 3. Disk
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub disk_pct: f32,
    pub disk_mount: String,
    pub inodes_pct: Option<f32>,
    pub iowait_pct: Option<f32>,

    // 4. Load Average
    pub load_1m: f32,
    pub load_5m: f32,
    pub load_15m: f32,
    pub load_history: Vec<f32>,

    // 5. Uptime
    pub uptime_seconds: u64,
    pub uptime_formatted: String,

    // 6. Network
    pub net_rx_bps: u64,
    pub net_tx_bps: u64,
    pub net_history: Vec<f32>,

    // 7. Services & timestamps
    pub services: Vec<LiveServiceStatus>,
    pub last_sample_ts: String,
    /// When the oldest SSH host key file was written (Unix seconds), if
    /// Crow could read it (ERR-81).
    pub host_key_oldest: Option<i64>,
    /// `cpu_pct` is a measurement (the first sample has no rate to take).
    pub cpu_known: bool,
    /// Whether the last probe reached the host. When it didn't, the numbers
    /// above are stale (or defaults) and must not be shown as readings.
    pub reachable: bool,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            cpu_pct: 0.0,
            vcpu_count: 1,
            cpu_peak: 0.0,
            cpu_history: vec![0.0; MAX_HISTORY_POINTS],
            mem_used_bytes: 0,
            mem_total_bytes: 1,
            mem_pct: 0.0,
            mem_history: vec![0.0; MAX_HISTORY_POINTS],
            disk_used_bytes: 0,
            disk_total_bytes: 1,
            disk_pct: 0.0,
            disk_mount: "/".to_string(),
            inodes_pct: None,
            iowait_pct: None,
            load_1m: 0.0,
            load_5m: 0.0,
            load_15m: 0.0,
            load_history: vec![0.0; MAX_HISTORY_POINTS],
            uptime_seconds: 0,
            uptime_formatted: "0m".to_string(),
            net_rx_bps: 0,
            net_tx_bps: 0,
            net_history: vec![0.0; MAX_HISTORY_POINTS],
            services: Vec::new(),
            last_sample_ts: chrono::Local::now().format("%H:%M:%S").to_string(),
            reachable: false,
            host_key_oldest: None,
            cpu_known: false,
        }
    }
}

impl ServerMetrics {
    /// Pushes a new CPU percentage sample and updates the peak and capped history buffer.
    pub fn push_cpu_sample(&mut self, pct: f32) {
        self.cpu_known = true;
        self.cpu_pct = pct.clamp(0.0, 100.0);
        if self.cpu_pct > self.cpu_peak {
            self.cpu_peak = self.cpu_pct;
        }
        push_capped_history(&mut self.cpu_history, self.cpu_pct, MAX_HISTORY_POINTS);
    }

    /// Pushes a new Memory sample.
    pub fn push_mem_sample(&mut self, used: u64, total: u64) {
        self.mem_used_bytes = used;
        self.mem_total_bytes = total.max(1);
        self.mem_pct = ((used as f64 / self.mem_total_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32;
        push_capped_history(&mut self.mem_history, self.mem_pct, MAX_HISTORY_POINTS);
    }

    /// Pushes a new Load average sample.
    pub fn push_load_sample(&mut self, l1: f32, l5: f32, l15: f32) {
        self.load_1m = l1;
        self.load_5m = l5;
        self.load_15m = l15;
        push_capped_history(&mut self.load_history, l1, MAX_HISTORY_POINTS);
    }

    /// Pushes a new Network throughput sample (in KB/s).
    pub fn push_net_sample(&mut self, rx_bps: u64, tx_bps: u64) {
        self.net_rx_bps = rx_bps;
        self.net_tx_bps = tx_bps;
        let total_kbps = ((rx_bps + tx_bps) / 1024) as f32;
        push_capped_history(&mut self.net_history, total_kbps, MAX_HISTORY_POINTS);
    }

    /// Helper to format used/total memory in GB or MB.
    pub fn mem_formatted(&self) -> (String, String) {
        let used_gb = self.mem_used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.mem_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        (format!("{:.1}", used_gb), format!("/{} GB", total_gb.round() as u64))
    }

    /// Helper to format used/total disk in GB.
    pub fn disk_formatted(&self) -> (String, String) {
        let used_gb = self.disk_used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.disk_total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        (format!("{:.0}", used_gb), format!("/{} GB", total_gb.round() as u64))
    }

    /// Helper to format uptime into (days, hours, minutes).
    pub fn uptime_parts(&self) -> (u64, u64, u64) {
        let days = self.uptime_seconds / 86400;
        let hours = (self.uptime_seconds % 86400) / 3600;
        let mins = (self.uptime_seconds % 3600) / 60;
        (days, hours, mins)
    }
}

pub fn push_capped_history(history: &mut Vec<f32>, new_val: f32, max_len: usize) {
    history.push(new_val);
    if history.len() > max_len {
        let excess = history.len() - max_len;
        history.drain(0..excess);
    }
}

pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {:02}h", days, hours)
    } else if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// A fully hydrated local telemetry snapshot stored in the time-series ring buffer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricSample {
    pub timestamp_secs: u64,
    pub metrics: ServerMetrics,
    pub services: Vec<ServiceUnit>,
    pub processes: Vec<ProcessUnit>,
    pub sockets: Vec<SocketUnit>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SurgeType {
    CpuSpike { pct: f32, delta: f32 },
    MemSurge { pct: f32 },
    ServiceFailure { unit_name: String },
    LoadSpike { load_1m: f32 },
}

#[derive(Clone, Debug)]
pub struct SurgeAlert {
    pub surge_type: SurgeType,
    pub lead_seconds: u64,
    pub description: String,
    pub is_critical: bool,
}

/// CPU or memory use, in percent, that the turbo buffer warns about.
pub const CPU_ALERT_PCT: f32 = 90.0;
pub const MEM_ALERT_PCT: f32 = 90.0;
/// Load average per core that the turbo buffer warns about.
pub const LOAD_ALERT_PER_CORE: f32 = 2.0;
/// How many samples in a row a level must hold before it counts: a single
/// busy second isn't worth anyone's attention.
pub const SUSTAINED_SAMPLES: usize = 3;

/// A decoupled local in-memory sliding buffer of telemetry samples.
/// Ingestion pushes at T_head, while the UI queries at (T_head - lag_secs),
/// providing buttery smooth rendering and lookahead foreknowledge.
#[derive(Clone, Debug)]
pub struct ServerTimeSeriesBuffer {
    pub samples: VecDeque<MetricSample>,
    pub max_duration_secs: u64,
}

impl Default for ServerTimeSeriesBuffer {
    fn default() -> Self {
        Self::new(180) // 3-minute sliding window default
    }
}

impl ServerTimeSeriesBuffer {
    pub fn new(max_duration_secs: u64) -> Self {
        Self {
            samples: VecDeque::new(),
            max_duration_secs: max_duration_secs.max(60),
        }
    }

    pub fn push_sample(&mut self, sample: MetricSample) {
        let current_ts = sample.timestamp_secs;
        self.samples.push_back(sample);

        // Trim samples older than max_duration_secs relative to head
        while let Some(front) = self.samples.front() {
            if current_ts.saturating_sub(front.timestamp_secs) > self.max_duration_secs {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Queries the sample closest to (T_head - lag_secs).
    /// If the buffer has not yet accumulated lag_secs of history, returns the oldest available sample.
    pub fn query_lagged(&self, lag_secs: u64) -> Option<&MetricSample> {
        let newest = self.samples.back()?;
        let target_ts = newest.timestamp_secs.saturating_sub(lag_secs);

        let mut best: Option<&MetricSample> = None;
        let mut min_diff = u64::MAX;

        for s in &self.samples {
            let diff = if s.timestamp_secs >= target_ts {
                s.timestamp_secs - target_ts
            } else {
                target_ts - s.timestamp_secs
            };
            if diff < min_diff {
                min_diff = diff;
                best = Some(s);
            }
        }

        best
    }

    /// Returns the newest (head) sample currently in the buffer.
    pub fn head(&self) -> Option<&MetricSample> {
        self.samples.back()
    }

    /// Scans the lookahead window (T_playhead, T_head] for trouble worth
    /// interrupting someone for: a service failing, or CPU, memory or load
    /// staying high for several samples in a row. One-sample blips and jumps
    /// that end at a harmless level don't count. Returns the earliest, with
    /// its lead time.
    pub fn detect_upcoming_surge(&self, lag_secs: u64) -> Option<SurgeAlert> {
        let newest = self.samples.back()?;
        let target_ts = newest.timestamp_secs.saturating_sub(lag_secs);
        let baseline = self.query_lagged(lag_secs)?;
        let ahead: Vec<&MetricSample> = self.samples.iter().filter(|s| s.timestamp_secs > target_ts).collect();
        let mut found: Vec<SurgeAlert> = Vec::new();

        // A service that fails ahead, and wasn't failed at the playhead.
        'services: for s in &ahead {
            for svc in s.services.iter().filter(|svc| svc.status == "FAILED") {
                if !baseline.services.iter().any(|b| b.name == svc.name && b.status == "FAILED") {
                    found.push(SurgeAlert {
                        surge_type: SurgeType::ServiceFailure { unit_name: svc.name.clone() },
                        lead_seconds: s.timestamp_secs.saturating_sub(target_ts),
                        description: format!("{} fails", svc.name),
                        is_critical: true,
                    });
                    break 'services;
                }
            }
        }

        // A level that holds for SUSTAINED_SAMPLES in a row, and isn't
        // already there at the playhead (that's shown as current, not ahead).
        let sustained = |over: &dyn Fn(&MetricSample) -> bool| -> Option<(usize, usize)> {
            if over(baseline) {
                return None;
            }
            let mut i = 0;
            while i < ahead.len() {
                let run = ahead[i..].iter().take_while(|s| over(s)).count();
                if run >= SUSTAINED_SAMPLES {
                    return Some((i, i + run - 1));
                }
                i += run.max(1);
            }
            None
        };
        let lead = |i: usize| ahead[i].timestamp_secs.saturating_sub(target_ts);
        let held = |i: usize, j: usize| ahead[j].timestamp_secs.saturating_sub(ahead[i].timestamp_secs);

        if let Some((i, j)) = sustained(&|s| s.metrics.cpu_pct >= CPU_ALERT_PCT) {
            let peak = ahead[i..=j].iter().map(|s| s.metrics.cpu_pct).fold(0.0, f32::max);
            found.push(SurgeAlert {
                surge_type: SurgeType::CpuSpike { pct: peak, delta: peak - baseline.metrics.cpu_pct },
                lead_seconds: lead(i),
                description: format!("CPU at {peak:.0}% for {}s", held(i, j)),
                is_critical: true,
            });
        }
        if let Some((i, j)) = sustained(&|s| s.metrics.mem_pct >= MEM_ALERT_PCT) {
            let peak = ahead[i..=j].iter().map(|s| s.metrics.mem_pct).fold(0.0, f32::max);
            found.push(SurgeAlert {
                surge_type: SurgeType::MemSurge { pct: peak },
                lead_seconds: lead(i),
                description: format!("memory at {peak:.0}% for {}s", held(i, j)),
                is_critical: true,
            });
        }
        let overloaded = |s: &MetricSample| s.metrics.vcpu_count > 0 && s.metrics.load_1m >= s.metrics.vcpu_count as f32 * LOAD_ALERT_PER_CORE;
        if let Some((i, j)) = sustained(&overloaded) {
            let peak = ahead[i..=j].iter().map(|s| s.metrics.load_1m).fold(0.0, f32::max);
            found.push(SurgeAlert {
                surge_type: SurgeType::LoadSpike { load_1m: peak },
                lead_seconds: lead(i),
                description: format!("load {peak:.1} on {} cores for {}s", ahead[i].metrics.vcpu_count, held(i, j)),
                is_critical: true,
            });
        }

        found.into_iter().min_by_key(|a| a.lead_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_capped_history() {
        let mut history = vec![1.0, 2.0, 3.0];
        push_capped_history(&mut history, 4.0, 3);
        assert_eq!(history, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(3600 * 24 * 3 + 3600 * 5), "3d 05h");
        assert_eq!(format_uptime(3600 * 2 + 60 * 14), "2h 14m");
        assert_eq!(format_uptime(120), "2m");
    }

    #[test]
    fn test_server_metrics_peak() {
        let mut m = ServerMetrics::default();
        m.push_cpu_sample(25.0);
        assert_eq!(m.cpu_peak, 25.0);
        m.push_cpu_sample(55.0);
        assert_eq!(m.cpu_peak, 55.0);
        m.push_cpu_sample(30.0);
        assert_eq!(m.cpu_peak, 55.0);
    }

    #[test]
    fn test_buffer_push_and_query_lagged() {
        let mut buffer = ServerTimeSeriesBuffer::new(120);
        let make_sample = |ts: u64, cpu: f32| MetricSample {
            timestamp_secs: ts,
            metrics: ServerMetrics { cpu_pct: cpu, ..Default::default() },
            services: Vec::new(),
            processes: Vec::new(),
            sockets: Vec::new(),
        };

        buffer.push_sample(make_sample(100, 20.0));
        buffer.push_sample(make_sample(110, 25.0));
        buffer.push_sample(make_sample(124, 30.0)); // Head is at 124

        // Query with lag of 24s: Target is 124 - 24 = 100
        let lagged = buffer.query_lagged(24).expect("sample expected");
        assert_eq!(lagged.timestamp_secs, 100);
        assert_eq!(lagged.metrics.cpu_pct, 20.0);
    }

    #[test]
    fn test_buffer_retention_trim() {
        let mut buffer = ServerTimeSeriesBuffer::new(60); // 60s window
        let make_sample = |ts: u64| MetricSample {
            timestamp_secs: ts,
            metrics: ServerMetrics::default(),
            services: Vec::new(),
            processes: Vec::new(),
            sockets: Vec::new(),
        };

        buffer.push_sample(make_sample(10));
        buffer.push_sample(make_sample(50));
        buffer.push_sample(make_sample(80)); // 80 - 10 = 70 > 60, so sample 10 must be trimmed

        assert_eq!(buffer.samples.len(), 2);
        assert_eq!(buffer.samples.front().unwrap().timestamp_secs, 50);
        assert_eq!(buffer.samples.back().unwrap().timestamp_secs, 80);
    }

    #[test]
    fn test_detect_upcoming_surge_cpu() {
        let mut buffer = ServerTimeSeriesBuffer::new(180);
        let make_sample = |ts: u64, cpu: f32| MetricSample {
            timestamp_secs: ts,
            metrics: ServerMetrics { cpu_pct: cpu, ..Default::default() },
            services: Vec::new(),
            processes: Vec::new(),
            sockets: Vec::new(),
        };

        buffer.push_sample(make_sample(100, 25.0)); // T_playhead (lag = 24, Head = 124)
        buffer.push_sample(make_sample(110, 40.0));
        buffer.push_sample(make_sample(114, 91.0)); // high from T+14s...
        buffer.push_sample(make_sample(118, 95.0));
        buffer.push_sample(make_sample(124, 92.0)); // ...and it holds

        let alert = buffer.detect_upcoming_surge(24).expect("surge alert expected");
        assert_eq!(alert.lead_seconds, 14);
        assert!(alert.is_critical);
        assert_eq!(alert.description, "CPU at 95% for 10s");
    }

    fn cpu_buffer(points: &[(u64, f32)]) -> ServerTimeSeriesBuffer {
        let mut buffer = ServerTimeSeriesBuffer::new(180);
        for &(ts, cpu) in points {
            buffer.push_sample(MetricSample { timestamp_secs: ts, metrics: ServerMetrics { cpu_pct: cpu, ..Default::default() }, services: Vec::new(), processes: Vec::new(), sockets: Vec::new() });
        }
        buffer
    }

    #[test]
    fn small_changes_and_harmless_jumps_are_not_alerts() {
        // 29% → 30%: nothing to see.
        assert!(cpu_buffer(&[(100, 29.0), (110, 29.5), (116, 30.0), (124, 30.0)]).detect_upcoming_surge(24).is_none());
        // 5% → 30% in one tick: a jump, but to a harmless level.
        assert!(cpu_buffer(&[(100, 5.0), (110, 30.0), (116, 31.0), (124, 30.0)]).detect_upcoming_surge(24).is_none());
    }

    #[test]
    fn a_single_busy_sample_is_not_an_alert() {
        assert!(cpu_buffer(&[(100, 20.0), (110, 99.0), (116, 25.0), (124, 22.0)]).detect_upcoming_surge(24).is_none());
    }

    #[test]
    fn already_high_at_the_playhead_is_not_news() {
        assert!(cpu_buffer(&[(100, 95.0), (110, 96.0), (116, 97.0), (124, 95.0)]).detect_upcoming_surge(24).is_none());
    }
}
