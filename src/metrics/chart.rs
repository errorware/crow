//! History charts' data (ERR-86): minute rows bucketed into lines that
//! break where nothing was measured, plus the spans to shade (Crow closed,
//! host unreachable) and alert periods. GPUI-free.

use super::alerts::Alert;
use super::history::HistoryRow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range {
    Day,
    Week,
}

impl Range {
    pub fn secs(self) -> i64 {
        match self {
            Range::Day => 86_400,
            Range::Week => 7 * 86_400,
        }
    }

    /// Bucket width: ~700 points either way.
    pub fn bucket_secs(self) -> i64 {
        match self {
            Range::Day => 120,
            Range::Week => 900,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Range::Day => "24H",
            Range::Week => "7D",
        }
    }
}

/// One chart's line: runs of (x in 0..=1, value), broken where a bucket had
/// no measurement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Series {
    pub segments: Vec<Vec<(f32, f32)>>,
    pub max: f32,
    pub last: Option<f32>,
}

impl Series {
    /// The value at x (nearest measured point within one bucket), for the
    /// hover readout.
    pub fn value_at(&self, x: f32, bucket_frac: f32) -> Option<f32> {
        self.segments
            .iter()
            .flatten()
            .map(|&(px, v)| ((px - x).abs(), v))
            .filter(|(d, _)| *d <= bucket_frac)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, v)| v)
    }
}

/// Buckets `rows` over [from, to) and averages `pick` in each bucket.
pub fn series(rows: &[HistoryRow], pick: fn(&HistoryRow) -> Option<f32>, from: i64, to: i64, bucket: i64) -> Series {
    let n = ((to - from) / bucket).max(1) as usize;
    let mut sums = vec![(0.0f32, 0u32); n];
    for r in rows {
        let (Some(v), true) = (pick(r), r.ts >= from && r.ts < to) else { continue };
        let i = (((r.ts - from) / bucket) as usize).min(n - 1);
        sums[i].0 += v;
        sums[i].1 += 1;
    }
    let mut out = Series::default();
    let mut run: Vec<(f32, f32)> = Vec::new();
    for (i, (sum, count)) in sums.iter().enumerate() {
        if *count == 0 {
            if !run.is_empty() {
                out.segments.push(std::mem::take(&mut run));
            }
            continue;
        }
        let v = sum / *count as f32;
        out.max = out.max.max(v);
        out.last = Some(v);
        run.push(((i as f32 + 0.5) / n as f32, v));
    }
    if !run.is_empty() {
        out.segments.push(run);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapKind {
    /// Crow wasn't running: nothing was watched.
    CrowClosed,
    /// Crow was running but the host didn't answer.
    Unreachable,
}

/// A shaded span, in 0..=1 of the range.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub from: f32,
    pub to: f32,
    pub kind: GapKind,
}

/// Where Crow wasn't running (outside every watch session) and where the
/// host was unreachable (runs of unreachable rows), within [from, to).
pub fn gaps(rows: &[HistoryRow], sessions: &[(i64, i64)], from: i64, to: i64, bucket: i64) -> Vec<Span> {
    let frac = |t: i64| (t.clamp(from, to) - from) as f32 / (to - from) as f32;
    let mut spans = Vec::new();
    // Uncovered time, allowing one bucket of slack at each edge.
    let mut covered: Vec<(i64, i64)> = sessions.iter().map(|&(s, e)| (s - bucket, e + bucket)).filter(|&(s, e)| e > from && s < to).collect();
    covered.sort();
    let mut cursor = from;
    for (s, e) in covered {
        if s > cursor {
            spans.push(Span { from: frac(cursor), to: frac(s), kind: GapKind::CrowClosed });
        }
        cursor = cursor.max(e);
    }
    if cursor < to {
        spans.push(Span { from: frac(cursor), to: frac(to), kind: GapKind::CrowClosed });
    }
    let mut start: Option<i64> = None;
    for r in rows.iter().filter(|r| r.ts >= from && r.ts < to) {
        match (r.reachable, start) {
            (false, None) => start = Some(r.ts),
            (true, Some(s)) => {
                spans.push(Span { from: frac(s), to: frac(r.ts), kind: GapKind::Unreachable });
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        let end = rows.iter().rev().find(|r| r.ts < to).map(|r| r.ts + bucket).unwrap_or(to);
        spans.push(Span { from: frac(s), to: frac(end), kind: GapKind::Unreachable });
    }
    spans
}

/// An alert's time on the chart: (from, to, level, label).
pub fn alert_spans(alerts: &[Alert], server_id: &str, from: i64, to: i64, now: i64) -> Vec<(f32, f32, String, String)> {
    let frac = |t: i64| (t.clamp(from, to) - from) as f32 / (to - from) as f32;
    alerts
        .iter()
        .filter(|a| a.server_id == server_id && a.resolved_at.unwrap_or(now) >= from && a.opened_at < to)
        .map(|a| (frac(a.opened_at), frac(a.resolved_at.unwrap_or(now)), a.level.clone(), a.detail.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(ts: i64, reachable: bool, cpu: Option<f32>) -> HistoryRow {
        HistoryRow { server_id: "s".into(), ts, reachable, reason: None, cpu, mem: None, disk: None, disk_mount: None, load1: None, failed_services: None }
    }

    #[test]
    fn lines_break_where_nothing_was_measured() {
        // 10 buckets of 60s; data in buckets 0-2 and 6-7.
        let rows = [row(0, true, Some(10.0)), row(60, true, Some(20.0)), row(130, true, Some(30.0)), row(370, true, Some(50.0)), row(420, true, Some(70.0))];
        let s = series(&rows, |r| r.cpu, 0, 600, 60);
        assert_eq!(s.segments.len(), 2);
        assert_eq!(s.segments[0].len(), 3);
        assert_eq!((s.max, s.last), (70.0, Some(70.0)));
        assert_eq!(s.value_at(0.05, 0.1), Some(10.0));
        assert_eq!(s.value_at(0.45, 0.05), None, "nothing measured there");
    }

    #[test]
    fn unmeasured_values_are_not_zeros() {
        let rows = [row(0, true, None), row(60, true, Some(5.0))];
        let s = series(&rows, |r| r.cpu, 0, 120, 60);
        assert_eq!(s.segments, vec![vec![(0.75, 5.0)]]);
    }

    #[test]
    fn closed_and_unreachable_spans() {
        let rows = [row(100, true, None), row(160, false, None), row(220, false, None), row(280, true, None)];
        let spans = gaps(&rows, &[(100, 300)], 0, 1000, 10);
        let kinds: Vec<_> = spans.iter().map(|s| (s.kind, (s.from * 1000.0).round(), (s.to * 1000.0).round())).collect();
        assert_eq!(kinds, vec![(GapKind::CrowClosed, 0.0, 90.0), (GapKind::CrowClosed, 310.0, 1000.0), (GapKind::Unreachable, 160.0, 280.0)]);
    }
}
