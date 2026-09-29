//! Running fleet-wide actions (ERR-79): plan, confirm, then one host at a
//! time on a background thread, stopping at the first failure.

use std::sync::Arc;

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::config::{history, push};
use crate::host::host_for;
use crate::views::fleet::run::{FleetRun, RunPhase, StepResult};

/// One host's work, run on a background thread.
pub type RunJob = Box<dyn FnOnce() -> StepResult + Send>;

/// The run on screen, the jobs behind its steps (until it starts), and the
/// keyword box.
pub struct FleetRunner {
    pub run: FleetRun,
    jobs: Vec<RunJob>,
    pub input: Entity<InputState>,
}

impl CrowApp {
    /// Plans pushing each baseline to every server whose last-read copy
    /// drifted from it. Nothing runs until the keyword is typed.
    pub fn plan_push_baselines(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh_drift();
        let key = self.vault.key().cloned();
        let db = self.vault.db();
        let author = self.default_author();
        let mut steps = Vec::new();
        let mut jobs: Vec<RunJob> = Vec::new();
        let mut excluded = Vec::new();
        {
            let Ok(guard) = db.lock() else { return };
            let baselines = guard.list_config_baselines().unwrap_or_default();
            for e in self.fleet.drift.iter().flatten().filter(|e| e.drift.is_drift()) {
                let Some(srv) = self.fleet.servers.iter().find(|s| s.id == e.server_id).cloned() else { continue };
                let content = baselines
                    .iter()
                    .find(|b| b.path == e.path && b.scope == e.scope)
                    .and_then(|b| guard.get_config_revision(&b.revision_id).ok().flatten())
                    .and_then(|rev| history::open_content(&rev, key.as_ref()));
                let Some(baseline) = content else {
                    excluded.push((srv.name.clone(), format!("{}: the baseline's content wasn't kept (hash only), so there's nothing to push", e.path)));
                    continue;
                };
                steps.push(FleetRun::step(&srv.id, &srv.name, format!("push baseline {}", e.path)));
                let (db, key, author, path) = (Arc::clone(&db), key.clone(), author.clone(), e.path.clone());
                jobs.push(Box::new(move || {
                    let host = host_for(&srv);
                    let target = push::PushTarget { server_id: &srv.id, server_name: &srv.name, path: &path, baseline: &baseline, author: &author };
                    push::push_baseline(host.as_ref(), &db, key.as_ref(), &target)
                }));
            }
        }
        let title = if steps.is_empty() && excluded.is_empty() { "PUSH BASELINE · nothing has drifted" } else { "PUSH BASELINE TO ALL" };
        self.open_fleet_run(FleetRun::new(title, "PUSH", steps, excluded), jobs, window, cx);
    }

    fn open_fleet_run(&mut self, run: FleetRun, jobs: Vec<RunJob>, window: &mut Window, cx: &mut Context<Self>) {
        let keyword = run.keyword;
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(keyword));
        self.fleet_runner = Some(FleetRunner { run, jobs, input });
        cx.notify();
    }

    /// Starts the planned run if the keyword was typed.
    pub fn confirm_fleet_run(&mut self, cx: &mut Context<Self>) {
        let Some(runner) = self.fleet_runner.as_mut() else { return };
        if runner.run.phase != RunPhase::Confirming || runner.run.steps.is_empty() {
            return;
        }
        let typed = runner.input.read(cx).value().trim().to_string();
        if typed != runner.run.keyword {
            runner.run.error = Some(format!("Type {} exactly to start.", runner.run.keyword));
            cx.notify();
            return;
        }
        runner.run.error = None;
        runner.run.phase = RunPhase::Running;
        let jobs = std::mem::take(&mut runner.jobs);
        cx.notify();
        cx.spawn(async move |entity, cx| {
            for (i, job) in jobs.into_iter().enumerate() {
                let go = entity
                    .update(cx, |this, cx| {
                        let run = &mut this.fleet_runner.as_mut()?.run;
                        if run.stop_requested() {
                            run.finish();
                            cx.notify();
                            return None;
                        }
                        run.start(i);
                        cx.notify();
                        Some(())
                    })
                    .ok()
                    .flatten();
                if go.is_none() {
                    break;
                }
                let result = cx.background_executor().spawn(async move { job() }).await;
                let more = entity
                    .update(cx, |this, cx| {
                        let more = this.fleet_runner.as_mut().is_some_and(|r| r.run.record(i, result));
                        cx.notify();
                        more
                    })
                    .unwrap_or(false);
                if !more {
                    break;
                }
            }
            let _ = entity.update(cx, |this, cx| {
                let marker = this.fleet_runner.as_mut().map(|r| {
                    if r.run.phase != RunPhase::Finished {
                        r.run.finish();
                    }
                    format!("crow: {} — {}", r.run.title.to_lowercase(), r.run.summary())
                });
                if let Some(m) = marker {
                    this.push_journal_action_marker(m);
                }
                // Files changed on servers: drift, and the configs on screen.
                this.refresh_drift();
                if !this.configs.has_unsaved_changes() {
                    this.configs.server_id = None;
                    this.reload_configs_for_active_server(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Lets the host that's running finish, then stops.
    pub fn stop_fleet_run(&mut self, cx: &mut Context<Self>) {
        if let Some(r) = &self.fleet_runner {
            r.run.request_stop();
        }
        cx.notify();
    }

    /// Closes the panel; a running run can't be closed, only stopped.
    pub fn close_fleet_run(&mut self, cx: &mut Context<Self>) {
        if self.fleet_runner.as_ref().is_some_and(|r| r.run.phase == RunPhase::Running) {
            return;
        }
        self.fleet_runner = None;
        cx.notify();
    }
}
