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

    /// Plans rebooting every reachable server, one at a time, each waiting
    /// for the server to come back from a fresh boot before the next (ERR-80).
    pub fn plan_rolling_reboot(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::host::{reboot, transport_kind, TransportKind};
        use crow_provider_core::capabilities::INSTANCES_POWER;
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);
        const POLL: std::time::Duration = std::time::Duration::from_secs(5);
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<_>) = (Vec::new(), Vec::new(), Vec::new());
        for srv in self.fleet.servers.clone() {
            let kind = transport_kind(&srv);
            if kind == TransportKind::Local {
                excluded.push((srv.name.clone(), "the machine Crow runs on".to_string()));
                continue;
            }
            if !self.fleet.health(&srv).is_ok() {
                excluded.push((srv.name.clone(), "not reachable right now, so Crow couldn't watch it come back".to_string()));
                continue;
            }
            // Through the provider when it's linked to one that can power it.
            let provider = self.providers.accounts.iter().find(|a| a.id == srv.provider_account && !srv.provider_instance.is_empty()).cloned().filter(|a| {
                crate::providers::factory(&a.plugin).is_some_and(|f| (f.manifest)().has_capability(INSTANCES_POWER))
            });
            let provider = provider.and_then(|a| {
                let settings = match (self.vault.key(), self.vault.db().lock()) {
                    (Some(key), Ok(db)) => crate::providers::load_settings(&db, key, &a).ok(),
                    _ => None,
                };
                settings.map(|s| (a, s))
            });
            let how = match (&provider, kind) {
                (Some((a, _)), _) => format!("reboot via {}", a.label),
                (None, TransportKind::Container) => "restart the lab container".to_string(),
                _ => "systemctl reboot over SSH".to_string(),
            };
            steps.push(FleetRun::step(&srv.id, &srv.name, how.clone()));
            let db = self.vault.db();
            jobs.push(Box::new(move || {
                let host = host_for(&srv);
                let trigger = || -> Result<(), String> {
                    match provider {
                        Some((account, settings)) => {
                            let p = crate::providers::connect(&account, settings).map_err(|e| e.to_string())?;
                            crate::providers::run_action(p.as_ref(), &srv.provider_instance, crate::providers::ProviderAction::Reboot).map(|_| ())
                        }
                        None if kind == TransportKind::Container => {
                            let engine = if srv.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
                            let out = std::process::Command::new(engine).args(["restart", &srv.name]).output().map_err(|e| e.to_string())?;
                            if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
                        }
                        None => host.exec_privileged(&["systemctl", "reboot"], &[], crate::host::DEFAULT_TIMEOUT).map(|_| ()).map_err(|e| e.to_string()),
                    }
                };
                let result = reboot::reboot_and_wait(host.as_ref(), trigger, TIMEOUT, POLL);
                record_step(&db, &srv, "fleet.reboot", &how, &result);
                result
            }));
        }
        let n = steps.len();
        self.open_fleet_run(FleetRun::new(format!("ROLLING REBOOT · {n} SERVER{}", if n == 1 { "" } else { "S" }), "REBOOT", steps, excluded), jobs, window, cx);
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

/// A change record for one host's step, once it has an outcome.
fn record_step(db: &std::sync::Mutex<crate::vault::VaultDb>, srv: &crate::vault::ServerRecord, kind: &str, what: &str, result: &StepResult) {
    use crate::views::fleet::run::StepOutcome;
    let Ok(db) = db.lock() else { return };
    let now = chrono::Utc::now().to_rfc3339();
    let (outcome, after) = match result {
        Ok(StepOutcome::Done(m)) | Ok(StepOutcome::Skipped(m)) => ("success", m.clone()),
        Err(e) => ("failed", e.clone()),
    };
    let _ = db.insert_change_record(&crate::vault::ChangeRecord {
        id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
        server_id: srv.id.clone(),
        server_name: srv.name.clone(),
        action_kind: kind.into(),
        target: srv.name.clone(),
        before_state: what.into(),
        after_state: Some(after),
        blast_radius: Some("fleet run".into()),
        outcome: outcome.into(),
        started_at: now.clone(),
        completed_at: Some(now),
    });
}
