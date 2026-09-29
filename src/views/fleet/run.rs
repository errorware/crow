//! The fleet runner's state (ERR-79): a plan of per-host steps, run one
//! host at a time, stopped by the first failure or on request. GPUI-free.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Waiting,
    Running,
    Done(String),
    /// Nothing to do on this host (e.g. it already matches).
    Skipped(String),
    Failed(String),
    /// Never started: an earlier host failed, or the run was stopped.
    NotRun,
}

/// What a step reports when it didn't fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    Done(String),
    Skipped(String),
}

pub type StepResult = Result<StepOutcome, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunStep {
    pub server_id: String,
    pub server: String,
    /// What this step does on that host, e.g. "push /etc/ssh/sshd_config".
    pub what: String,
    pub status: StepStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunPhase {
    /// The plan is shown; nothing has run.
    Confirming,
    Running,
    Finished,
}

#[derive(Clone, Debug)]
pub struct FleetRun {
    pub title: String,
    /// Typed to start the run.
    pub keyword: &'static str,
    pub steps: Vec<RunStep>,
    /// Hosts left out of the plan, and why.
    pub excluded: Vec<(String, String)>,
    pub phase: RunPhase,
    /// Set from the UI; the runner checks it before each host.
    pub stop: Arc<AtomicBool>,
    /// Shown under the plan, e.g. a wrong keyword.
    pub error: Option<String>,
}

impl FleetRun {
    pub fn new(title: impl Into<String>, keyword: &'static str, steps: Vec<RunStep>, excluded: Vec<(String, String)>) -> Self {
        Self { title: title.into(), keyword, steps, excluded, phase: RunPhase::Confirming, stop: Arc::new(AtomicBool::new(false)), error: None }
    }

    pub fn step(server_id: &str, server: &str, what: impl Into<String>) -> RunStep {
        RunStep { server_id: server_id.into(), server: server.into(), what: what.into(), status: StepStatus::Waiting }
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn stop_requested(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub fn start(&mut self, i: usize) {
        self.phase = RunPhase::Running;
        if let Some(s) = self.steps.get_mut(i) {
            s.status = StepStatus::Running;
        }
    }

    /// Records step `i`'s result. A failure ends the run: every step still
    /// waiting is marked not run. Returns whether the run may continue.
    pub fn record(&mut self, i: usize, result: StepResult) -> bool {
        let failed = result.is_err();
        if let Some(s) = self.steps.get_mut(i) {
            s.status = match result {
                Ok(StepOutcome::Done(m)) => StepStatus::Done(m),
                Ok(StepOutcome::Skipped(m)) => StepStatus::Skipped(m),
                Err(e) => StepStatus::Failed(e),
            };
        }
        if failed || self.stop_requested() || !self.steps.iter().any(|s| s.status == StepStatus::Waiting) {
            self.finish();
            return false;
        }
        true
    }

    /// Ends the run: anything that hadn't started is marked not run.
    pub fn finish(&mut self) {
        for s in &mut self.steps {
            if matches!(s.status, StepStatus::Waiting | StepStatus::Running) {
                s.status = StepStatus::NotRun;
            }
        }
        self.phase = RunPhase::Finished;
    }

    /// "3 done · 1 skipped · stopped at web-2 · 2 not run".
    pub fn summary(&self) -> String {
        let count = |f: fn(&StepStatus) -> bool| self.steps.iter().filter(|s| f(&s.status)).count();
        let mut parts = vec![format!("{} done", count(|s| matches!(s, StepStatus::Done(_))))];
        let skipped = count(|s| matches!(s, StepStatus::Skipped(_)));
        if skipped > 0 {
            parts.push(format!("{skipped} already fine"));
        }
        if let Some(f) = self.steps.iter().find(|s| matches!(s.status, StepStatus::Failed(_))) {
            parts.push(format!("stopped at {}", f.server));
        } else if self.stop_requested() && self.phase == RunPhase::Finished {
            parts.push("stopped on request".into());
        }
        let not_run = count(|s| matches!(s, StepStatus::NotRun));
        if not_run > 0 {
            parts.push(format!("{not_run} not run"));
        }
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> FleetRun {
        FleetRun::new("push", "PUSH", vec![FleetRun::step("a", "a", "x"), FleetRun::step("b", "b", "x"), FleetRun::step("c", "c", "x")], vec![])
    }

    #[test]
    fn first_failure_stops_the_rest() {
        let mut r = run();
        r.start(0);
        assert!(r.record(0, Ok(StepOutcome::Done("ok".into()))));
        r.start(1);
        assert!(!r.record(1, Err("sshd -t: bad".into())));
        assert_eq!(r.steps[2].status, StepStatus::NotRun);
        assert_eq!(r.phase, RunPhase::Finished);
        assert_eq!(r.summary(), "1 done · stopped at b · 1 not run");
    }

    #[test]
    fn stop_request_takes_effect_after_the_current_host() {
        let mut r = run();
        r.start(0);
        r.request_stop();
        assert!(!r.record(0, Ok(StepOutcome::Skipped("matches".into()))));
        assert_eq!(r.steps[0].status, StepStatus::Skipped("matches".into()), "the host that was running finishes");
        assert_eq!(r.summary(), "0 done · 1 already fine · stopped on request · 2 not run");
    }

    #[test]
    fn completes() {
        let mut r = run();
        for i in 0..3 {
            r.start(i);
            let more = r.record(i, Ok(StepOutcome::Done("ok".into())));
            assert_eq!(more, i < 2);
        }
        assert_eq!(r.phase, RunPhase::Finished);
        assert_eq!(r.summary(), "3 done");
    }
}
