use serde::{Deserialize, Serialize};
use gpui_kit::Rgba;
use crate::theme::*;

pub mod collector;

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
    pub inodes_pct: f32,
    pub iowait_pct: f32,

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
            inodes_pct: 0.0,
            iowait_pct: 0.0,
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
        }
    }
}

impl ServerMetrics {
    /// Pushes a new CPU percentage sample and updates the peak and capped history buffer.
    pub fn push_cpu_sample(&mut self, pct: f32) {
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
}
