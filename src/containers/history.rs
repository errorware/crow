//! Container usage over time (ERR-201): one sample per container per
//! minute while Crow is open, kept as long as server history. Only what
//! was measured is stored; a stopped container has a state and no numbers.
//! Turned into chart lines here, GPUI-free.

use super::{Container, Scan};
use crate::metrics::chart::{self, Series, Span};
use crate::metrics::history::HistoryRow;

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerSample {
    pub server_id: String,
    /// The container's name: it outlives the id, which changes every time
    /// compose recreates the container.
    pub name: String,
    pub ts: i64,
    pub state: String,
    pub cpu: Option<f32>,
    pub mem: Option<u64>,
    pub mem_limit: Option<u64>,
    /// Totals since the container started.
    pub net_rx: Option<u64>,
    pub net_tx: Option<u64>,
    pub restarts: Option<u32>,
}

fn sample(server_id: &str, ts: i64, c: &Container) -> ContainerSample {
    let u = c.usage.filter(|_| c.is_running());
    ContainerSample {
        server_id: server_id.to_string(),
        name: c.name.clone(),
        ts,
        state: c.state.clone(),
        cpu: u.map(|u| u.cpu_pct),
        mem: u.map(|u| u.mem_used),
        mem_limit: u.map(|u| u.mem_limit),
        net_rx: u.map(|u| u.net_rx),
        net_tx: u.map(|u| u.net_tx),
        restarts: c.restarts,
    }
}

/// One sample per container in the scan.
pub fn samples_from(server_id: &str, ts: i64, scan: &Scan) -> Vec<ContainerSample> {
    scan.containers.iter().filter(|c| !c.name.is_empty()).map(|c| sample(server_id, ts, c)).collect()
}

/// A container's charts for one range.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContainerCharts {
    pub from: i64,
    pub to: i64,
    pub bucket: i64,
    /// CPU %.
    pub cpu: Series,
    /// Memory in MB.
    pub mem: Series,
    /// The newest memory limit, MB, for the memory chart's scale.
    pub mem_limit_mb: Option<f32>,
    /// Network in and out together, kB/s.
    pub net: Series,
    /// Where it restarted or came back up, in 0..=1 of the range.
    pub restarts: Vec<f32>,
    /// Crow closed / server unreachable, from the server's own history.
    pub gaps: Vec<Span>,
}

/// Samples of one container (oldest first), the server's rows and Crow's
/// watch sessions, as charts over [from, to).
pub fn charts(samples: &[ContainerSample], server_rows: &[HistoryRow], sessions: &[(i64, i64)], from: i64, to: i64, bucket: i64) -> ContainerCharts {
    let frac = |t: i64| (t.clamp(from, to) - from) as f32 / (to - from) as f32;
    // Network is a running total: the rate is the step between two
    // samples, unless the counter went back (a restart) or Crow wasn't
    // watching in between.
    let mut net: Vec<(i64, Option<f32>)> = Vec::new();
    let mut restarts = Vec::new();
    for pair in samples.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let dt = b.ts - a.ts;
        let rate = match (a.net_rx, a.net_tx, b.net_rx, b.net_tx) {
            (Some(ar), Some(at), Some(br), Some(bt)) if dt > 0 && dt <= 3 * 60 && br >= ar && bt >= at => Some(((br - ar) + (bt - at)) as f32 / dt as f32 / 1000.0),
            _ => None,
        };
        net.push((b.ts, rate));
        let restarted = matches!((a.restarts, b.restarts), (Some(x), Some(y)) if y > x);
        let came_up = a.state != "running" && b.state == "running";
        if restarted || came_up {
            restarts.push(frac(b.ts));
        }
    }
    ContainerCharts {
        from,
        to,
        bucket,
        cpu: chart::series_of(samples.iter().map(|s| (s.ts, s.cpu)), from, to, bucket),
        mem: chart::series_of(samples.iter().map(|s| (s.ts, s.mem.map(|m| m as f32 / 1e6))), from, to, bucket),
        mem_limit_mb: samples.iter().rev().find_map(|s| s.mem_limit).map(|l| l as f32 / 1e6),
        net: chart::series_of(net, from, to, bucket),
        restarts,
        gaps: chart::gaps(server_rows, sessions, from, to, bucket),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::containers::parse_scan;

    fn s(ts: i64, state: &str, cpu: Option<f32>, rx: Option<u64>, restarts: u32) -> ContainerSample {
        ContainerSample { server_id: "s".into(), name: "web".into(), ts, state: state.into(), cpu, mem: cpu.map(|_| 50_000_000), mem_limit: cpu.map(|_| 2_000_000_000), net_rx: rx, net_tx: rx.map(|_| 0), restarts: Some(restarts) }
    }

    #[test]
    fn a_scan_gives_a_sample_per_container_with_numbers_only_when_running() {
        let out = "@@uid 0\n@@begin docker root\n@@version 29\n@@ps\n{\"ID\":\"aaa\",\"Names\":\"web\",\"State\":\"running\",\"Status\":\"Up\"}\n{\"ID\":\"bbb\",\"Names\":\"job\",\"State\":\"exited\",\"Status\":\"Exited (1)\"}\n@@stats\naaa\t2.5%\t10MB / 1GB\t1%\t1kB / 2kB\t0B / 0B\t3\n@@end\n@@done\n";
        let got = samples_from("s1", 600, &parse_scan(out));
        assert_eq!(got.len(), 2);
        let web = got.iter().find(|x| x.name == "web").unwrap();
        assert_eq!((web.cpu, web.mem, web.net_rx), (Some(2.5), Some(10_000_000), Some(1000)));
        let job = got.iter().find(|x| x.name == "job").unwrap();
        assert_eq!((job.state.as_str(), job.cpu), ("exited", None), "stopped: a state, no numbers");
    }

    #[test]
    fn network_is_a_rate_and_restarts_are_marked() {
        let samples = vec![
            s(0, "running", Some(1.0), Some(0), 0),
            s(60, "running", Some(3.0), Some(600_000), 0),
            // Restarted: the counter starts over, so no rate here.
            s(120, "running", Some(2.0), Some(1_000), 1),
            s(180, "exited", None, None, 1),
            s(240, "running", Some(1.0), Some(0), 1),
        ];
        let c = charts(&samples, &[], &[(0, 300)], 0, 300, 60);
        assert_eq!(c.net.segments.iter().flatten().map(|p| p.1).collect::<Vec<_>>(), vec![10.0], "600 kB over 60 s is 10 kB/s; the rest aren't rates");
        assert_eq!(c.restarts, vec![0.4, 0.8], "the restart count going up, and coming back from exited");
        assert_eq!(c.cpu.segments.len(), 2, "the line breaks while it was stopped");
        assert_eq!((c.mem.max, c.mem_limit_mb), (50.0, Some(2000.0)));
        assert!(c.gaps.is_empty(), "Crow was watching the whole time");
    }

    #[test]
    fn a_long_unwatched_stretch_is_not_a_rate() {
        let samples = vec![s(0, "running", Some(1.0), Some(0), 0), s(3_600, "running", Some(1.0), Some(36_000_000), 0)];
        let c = charts(&samples, &[], &[(0, 60), (3_600, 3_660)], 0, 7_200, 60);
        assert!(c.net.segments.is_empty());
        assert!(!c.gaps.is_empty(), "Crow was closed in between");
    }
}
