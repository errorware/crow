//! Performance bottleneck analyzer over rolling `HistoryRow` windows and live
//! process snapshots (ERR-205).
//!
//! Evaluates four classes of sustained bottlenecks:
//! 1. `bottleneck:cpu` — Sustained CPU saturation (>= 85% WARN, >= 95% CRIT across >= 3 samples)
//! 2. `bottleneck:load` — Sustained run-queue contention (`load1` >= 1.5x / 2.5x vCPUs)
//! 3. `bottleneck:mem` — Sustained RAM pressure (>= 88% WARN, >= 95% CRIT) or upward memory leak creep
//! 4. `bottleneck:disk_rate` — Linear regression disk burn-rate projecting full disk within <= 7d / <= 48h

use crate::config::CrowConfigManager;
use crate::views::overview::ProcessUnit;

use super::alerts::{Alert, AlertChanges};
use super::history::HistoryRow;

pub const KIND_CPU: &str = "bottleneck:cpu";
pub const KIND_LOAD: &str = "bottleneck:load";
pub const KIND_MEM: &str = "bottleneck:mem";
pub const KIND_DISK_RATE: &str = "bottleneck:disk_rate";

pub const SUSTAINED_SAMPLES: usize = 3;
pub const CPU_WARN_PCT: f32 = 85.0;
pub const CPU_CRIT_PCT: f32 = 95.0;
pub const LOAD_WARN_MULT: f32 = 1.5;
pub const LOAD_CRIT_MULT: f32 = 2.5;
pub const MEM_WARN_PCT: f32 = 88.0;
pub const MEM_CRIT_PCT: f32 = 95.0;
pub const MEM_CREEP_DELTA_PCT: f32 = 15.0;
pub const DISK_WARN_HOURS: f64 = 7.0 * 24.0;
pub const DISK_CRIT_HOURS: f64 = 48.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BottleneckConfig {
    pub cpu_enabled: bool,
    pub load_enabled: bool,
    pub mem_enabled: bool,
    pub disk_rate_enabled: bool,
}

impl Default for BottleneckConfig {
    fn default() -> Self {
        Self {
            cpu_enabled: true,
            load_enabled: true,
            mem_enabled: true,
            disk_rate_enabled: true,
        }
    }
}

pub const FLAG_CPU: &str = "agent.bottleneck_cpu";
pub const FLAG_LOAD: &str = "agent.bottleneck_load";
pub const FLAG_MEM: &str = "agent.bottleneck_mem";
pub const FLAG_DISK_RATE: &str = "agent.bottleneck_disk_rate";

impl BottleneckConfig {
    pub fn from_config(cfg: &CrowConfigManager) -> Self {
        Self {
            cpu_enabled: cfg.saved_bool(FLAG_CPU).unwrap_or(true),
            load_enabled: cfg.saved_bool(FLAG_LOAD).unwrap_or(true),
            mem_enabled: cfg.saved_bool(FLAG_MEM).unwrap_or(true),
            disk_rate_enabled: cfg.saved_bool(FLAG_DISK_RATE).unwrap_or(true),
        }
    }

    pub fn from_db(db: &crate::vault::VaultDb) -> Self {
        let parse_flag = |k: &str| -> bool {
            db.flag(k)
                .map(|v| !matches!(v.trim(), "false" | "0" | "off"))
                .unwrap_or(true)
        };
        Self {
            cpu_enabled: parse_flag(FLAG_CPU),
            load_enabled: parse_flag(FLAG_LOAD),
            mem_enabled: parse_flag(FLAG_MEM),
            disk_rate_enabled: parse_flag(FLAG_DISK_RATE),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BottleneckContext {
    pub vcpu_count: usize,
    pub disk_mount: String,
    /// `(process_name, cpu_pct_str)` of the top CPU consumer when sampled.
    pub top_cpu_proc: Option<(String, String)>,
    /// `(process_name, mem_pct_str)` of the top RAM consumer when sampled.
    pub top_mem_proc: Option<(String, String)>,
}

impl BottleneckContext {
    pub fn from_metrics_and_processes(
        vcpu_count: usize,
        disk_mount: &str,
        processes: &[ProcessUnit],
    ) -> Self {
        let proc_label = |cmd: &str| -> String {
            cmd.split_whitespace().next().unwrap_or(cmd).to_string()
        };
        let top_cpu_proc = processes
            .iter()
            .max_by(|a, b| a.cpu.partial_cmp(&b.cpu).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|p| p.cpu >= 5.0)
            .map(|p| (proc_label(&p.command), format!("{:.1}", p.cpu)));
        let top_mem_proc = processes
            .iter()
            .max_by(|a, b| a.mem.partial_cmp(&b.mem).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|p| p.mem >= 5.0)
            .map(|p| (proc_label(&p.command), format!("{:.1}", p.mem)));

        Self {
            vcpu_count: vcpu_count.max(1),
            disk_mount: if disk_mount.is_empty() { "/".to_string() } else { disk_mount.to_string() },
            top_cpu_proc,
            top_mem_proc,
        }
    }
}

pub fn is_bottleneck_kind(kind: &str) -> bool {
    kind.starts_with("bottleneck:")
}

/// Evaluates all four bottleneck detectors for `server_id` given its recent
/// `history` (ordered oldest-first) and returns the `AlertChanges` to apply.
pub fn evaluate_server_bottlenecks(
    open: &[Alert],
    server_id: &str,
    history: &[HistoryRow],
    ctx: &BottleneckContext,
    cfg: &BottleneckConfig,
    now: i64,
) -> AlertChanges {
    let mut changes = AlertChanges::default();
    let reachable: Vec<&HistoryRow> = history
        .iter()
        .filter(|r| r.server_id == server_id && r.reachable)
        .collect();

    // If the most recent sample for this server was unreachable, leave existing
    // bottleneck alerts untouched (same rule as `alerts::evaluate`).
    if history.iter().rev().find(|r| r.server_id == server_id).is_some_and(|r| !r.reachable) {
        return changes;
    }

    let detectors: [(&str, bool, Option<(&'static str, String)>); 4] = [
        (KIND_CPU, cfg.cpu_enabled, cfg.cpu_enabled.then(|| detect_cpu_saturation(&reachable, ctx)).flatten()),
        (KIND_LOAD, cfg.load_enabled, cfg.load_enabled.then(|| detect_load_contention(&reachable, ctx)).flatten()),
        (KIND_MEM, cfg.mem_enabled, cfg.mem_enabled.then(|| detect_mem_bottleneck(&reachable, ctx)).flatten()),
        (KIND_DISK_RATE, cfg.disk_rate_enabled, cfg.disk_rate_enabled.then(|| detect_disk_burn_rate(&reachable, ctx)).flatten()),
    ];

    for (kind, enabled, result) in detectors {
        let existing = open
            .iter()
            .find(|a| a.server_id == server_id && a.kind == kind && a.resolved_at.is_none());

        if !enabled {
            if let Some(a) = existing {
                changes.resolved.push(a.id.clone());
            }
            continue;
        }

        match (existing, result) {
            (Some(a), Some((level, detail))) => {
                changes.updated.push((a.id.clone(), level.to_string(), detail, now));
            }
            (None, Some((level, detail))) => {
                changes.opened.push(Alert {
                    id: format!("alert-{server_id}-{kind}-{now}"),
                    server_id: server_id.to_string(),
                    kind: kind.to_string(),
                    level: level.to_string(),
                    detail,
                    opened_at: now,
                    last_seen: now,
                    resolved_at: None,
                    acknowledged_at: None,
                });
            }
            (Some(a), None) => {
                changes.resolved.push(a.id.clone());
            }
            (None, None) => {}
        }
    }

    changes
}

fn detect_cpu_saturation(
    reachable: &[&HistoryRow],
    ctx: &BottleneckContext,
) -> Option<(&'static str, String)> {
    let cpu_samples: Vec<f32> = reachable
        .iter()
        .rev()
        .filter_map(|r| r.cpu)
        .take(SUSTAINED_SAMPLES)
        .collect();
    if cpu_samples.len() < SUSTAINED_SAMPLES {
        return None;
    }

    let avg = cpu_samples.iter().copied().sum::<f32>() / cpu_samples.len() as f32;
    let level = if cpu_samples.iter().all(|&c| c >= CPU_CRIT_PCT) {
        "CRIT"
    } else if cpu_samples.iter().all(|&c| c >= CPU_WARN_PCT) {
        "WARN"
    } else {
        return None;
    };

    let proc_suffix = match &ctx.top_cpu_proc {
        Some((name, pct)) => format!(" · top: {name} ({pct})"),
        None => String::new(),
    };
    Some((
        level,
        format!("sustained CPU saturation ({avg:.0}% avg){proc_suffix}"),
    ))
}

fn detect_load_contention(
    reachable: &[&HistoryRow],
    ctx: &BottleneckContext,
) -> Option<(&'static str, String)> {
    let load_samples: Vec<f32> = reachable
        .iter()
        .rev()
        .filter_map(|r| r.load1)
        .take(SUSTAINED_SAMPLES)
        .collect();
    if load_samples.len() < SUSTAINED_SAMPLES {
        return None;
    }

    let cores = ctx.vcpu_count.max(1) as f32;
    let avg_load = load_samples.iter().copied().sum::<f32>() / load_samples.len() as f32;
    let level = if load_samples.iter().all(|&l| l >= cores * LOAD_CRIT_MULT) {
        "CRIT"
    } else if load_samples.iter().all(|&l| l >= cores * LOAD_WARN_MULT) {
        "WARN"
    } else {
        return None;
    };

    Some((
        level,
        format!(
            "sustained load contention (load1 {avg_load:.2} across {} vCPU{})",
            ctx.vcpu_count.max(1),
            if ctx.vcpu_count.max(1) == 1 { "" } else { "s" }
        ),
    ))
}

fn detect_mem_bottleneck(
    reachable: &[&HistoryRow],
    ctx: &BottleneckContext,
) -> Option<(&'static str, String)> {
    let proc_suffix = match &ctx.top_mem_proc {
        Some((name, pct)) => format!(" · top: {name} ({pct})"),
        None => String::new(),
    };

    let recent_mem: Vec<f32> = reachable
        .iter()
        .rev()
        .filter_map(|r| r.mem)
        .take(SUSTAINED_SAMPLES)
        .collect();

    if recent_mem.len() >= SUSTAINED_SAMPLES {
        let avg = recent_mem.iter().copied().sum::<f32>() / recent_mem.len() as f32;
        if recent_mem.iter().all(|&m| m >= MEM_CRIT_PCT) {
            return Some(("CRIT", format!("sustained memory pressure ({avg:.0}% used){proc_suffix}")));
        }
        if recent_mem.iter().all(|&m| m >= MEM_WARN_PCT) {
            return Some(("WARN", format!("sustained memory pressure ({avg:.0}% used){proc_suffix}")));
        }
    }

    // Memory creep / leak detection: >= 6 samples spanning >= 30 minutes where
    // memory climbed by >= 15 percentage points without dropping below previous quartiles,
    // and current usage is >= 70%.
    let timed_mem: Vec<(i64, f32)> = reachable
        .iter()
        .filter_map(|r| r.mem.map(|m| (r.ts, m)))
        .collect();
    if timed_mem.len() >= 6 {
        let window = &timed_mem[timed_mem.len().saturating_sub(36)..];
        if let (Some(&(first_ts, first_mem)), Some(&(last_ts, last_mem))) = (window.first(), window.last()) {
            let span_secs = last_ts - first_ts;
            let delta = last_mem - first_mem;
            let monotonic_enough = window
                .windows(2)
                .all(|w| w[1].1 + 1.5 >= w[0].1);
            if span_secs >= 1800 && last_mem >= 70.0 && delta >= MEM_CREEP_DELTA_PCT && monotonic_enough {
                return Some((
                    "WARN",
                    format!("steady memory creep (+{delta:.0}% to {last_mem:.0}%){proc_suffix}"),
                ));
            }
        }
    }

    None
}

fn detect_disk_burn_rate(
    reachable: &[&HistoryRow],
    ctx: &BottleneckContext,
) -> Option<(&'static str, String)> {
    let points: Vec<(f64, f64)> = reachable
        .iter()
        .filter_map(|r| r.disk.map(|d| (r.ts as f64, d as f64)))
        .collect();
    if points.len() < 4 {
        return None;
    }

    let window = &points[points.len().saturating_sub(120)..];
    let (t0, _) = window.first().copied()?;
    let (t_last, d_last) = window.last().copied()?;
    let span_secs = t_last - t0;
    // Need at least 15 minutes of span to project a meaningful burn rate.
    if span_secs < 900.0 || d_last >= 100.0 {
        return None;
    }

    let n = window.len() as f64;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;
    for &(t, y) in window {
        let x_hours = (t - t0) / 3600.0;
        sum_x += x_hours;
        sum_y += y;
        sum_xy += x_hours * y;
        sum_xx += x_hours * x_hours;
    }
    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-9 {
        return None;
    }

    // Slope in percentage points per hour.
    let slope_pct_per_hour = (n * sum_xy - sum_x * sum_y) / denom;
    if slope_pct_per_hour <= 0.05 {
        return None;
    }

    let remaining_pct = (100.0 - d_last).max(0.0);
    let hours_to_full = remaining_pct / slope_pct_per_hour;
    let pct_per_day = slope_pct_per_hour * 24.0;
    let mount = if ctx.disk_mount.is_empty() { "/" } else { ctx.disk_mount.as_str() };

    if hours_to_full <= DISK_CRIT_HOURS {
        Some((
            "CRIT",
            format!(
                "disk {mount} burn rate +{pct_per_day:.1}%/day · projected full in ~{hours_to_full:.0}h"
            ),
        ))
    } else if hours_to_full <= DISK_WARN_HOURS {
        let days = hours_to_full / 24.0;
        Some((
            "WARN",
            format!(
                "disk {mount} burn rate +{pct_per_day:.1}%/day · projected full in ~{days:.1}d"
            ),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(ts: i64, cpu: Option<f32>, mem: Option<f32>, disk: Option<f32>, load1: Option<f32>) -> HistoryRow {
        HistoryRow {
            server_id: "srv-1".into(),
            ts,
            reachable: true,
            reason: None,
            cpu,
            mem,
            disk,
            disk_mount: Some("/".into()),
            load1,
            failed_services: None,
        }
    }

    #[test]
    fn detects_sustained_cpu_saturation_and_resolves_when_cooled() {
        let ctx = BottleneckContext {
            vcpu_count: 4,
            disk_mount: "/".into(),
            top_cpu_proc: Some(("postgres".into(), "81.2%".into())),
            top_mem_proc: None,
        };
        let cfg = BottleneckConfig::default();
        let history = vec![
            row(60, Some(88.0), Some(40.0), Some(50.0), Some(1.0)),
            row(120, Some(91.0), Some(40.0), Some(50.0), Some(1.0)),
            row(180, Some(94.0), Some(40.0), Some(50.0), Some(1.0)),
        ];
        let c1 = evaluate_server_bottlenecks(&[], "srv-1", &history, &ctx, &cfg, 180);
        assert_eq!(c1.opened.len(), 1);
        assert_eq!(c1.opened[0].kind, KIND_CPU);
        assert_eq!(c1.opened[0].level, "WARN");
        assert!(c1.opened[0].detail.contains("postgres"));

        let mut cooled = history.clone();
        cooled.push(row(240, Some(30.0), Some(40.0), Some(50.0), Some(1.0)));
        let c2 = evaluate_server_bottlenecks(&c1.opened, "srv-1", &cooled, &ctx, &cfg, 240);
        assert_eq!(c2.resolved, vec![c1.opened[0].id.clone()]);
    }

    #[test]
    fn detects_sustained_load_contention_relative_to_vcpus() {
        let ctx = BottleneckContext {
            vcpu_count: 2,
            disk_mount: "/".into(),
            top_cpu_proc: None,
            top_mem_proc: None,
        };
        let cfg = BottleneckConfig::default();
        let history = vec![
            row(60, Some(50.0), Some(40.0), Some(50.0), Some(5.2)),
            row(120, Some(50.0), Some(40.0), Some(50.0), Some(5.5)),
            row(180, Some(50.0), Some(40.0), Some(50.0), Some(6.0)),
        ];
        let c = evaluate_server_bottlenecks(&[], "srv-1", &history, &ctx, &cfg, 180);
        assert_eq!(c.opened.len(), 1);
        assert_eq!(c.opened[0].kind, KIND_LOAD);
        assert_eq!(c.opened[0].level, "CRIT");
    }

    #[test]
    fn detects_memory_creep_leak_signature() {
        let ctx = BottleneckContext {
            vcpu_count: 2,
            disk_mount: "/".into(),
            top_cpu_proc: None,
            top_mem_proc: Some(("sidekiq".into(), "64.0%".into())),
        };
        let cfg = BottleneckConfig::default();
        let history = vec![
            row(0, Some(20.0), Some(55.0), Some(40.0), Some(0.4)),
            row(600, Some(20.0), Some(60.0), Some(40.0), Some(0.4)),
            row(1200, Some(20.0), Some(65.0), Some(40.0), Some(0.4)),
            row(1800, Some(20.0), Some(70.0), Some(40.0), Some(0.4)),
            row(2400, Some(20.0), Some(74.0), Some(40.0), Some(0.4)),
            row(3000, Some(20.0), Some(78.0), Some(40.0), Some(0.4)),
        ];
        let c = evaluate_server_bottlenecks(&[], "srv-1", &history, &ctx, &cfg, 3000);
        assert_eq!(c.opened.len(), 1);
        assert_eq!(c.opened[0].kind, KIND_MEM);
        assert_eq!(c.opened[0].level, "WARN");
        assert!(c.opened[0].detail.contains("steady memory creep"));
        assert!(c.opened[0].detail.contains("sidekiq"));
    }

    #[test]
    fn detects_disk_burn_rate_before_static_90_pct_threshold() {
        let ctx = BottleneckContext {
            vcpu_count: 2,
            disk_mount: "/var".into(),
            top_cpu_proc: None,
            top_mem_proc: None,
        };
        let cfg = BottleneckConfig::default();
        // Rising by 2% every hour from 60% -> 68% over 4 hours (+48%/day -> full in ~16h -> CRIT)
        let history = vec![
            row(0, Some(10.0), Some(30.0), Some(60.0), Some(0.2)),
            row(3600, Some(10.0), Some(30.0), Some(62.0), Some(0.2)),
            row(7200, Some(10.0), Some(30.0), Some(64.0), Some(0.2)),
            row(10800, Some(10.0), Some(30.0), Some(66.0), Some(0.2)),
            row(14400, Some(10.0), Some(30.0), Some(68.0), Some(0.2)),
        ];
        let c = evaluate_server_bottlenecks(&[], "srv-1", &history, &ctx, &cfg, 14400);
        assert_eq!(c.opened.len(), 1);
        assert_eq!(c.opened[0].kind, KIND_DISK_RATE);
        assert_eq!(c.opened[0].level, "CRIT");
        assert!(c.opened[0].detail.contains("disk /var burn rate"));
    }
}
