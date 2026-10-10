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
/// Something to offer once a run has finished (a rollout's ROLL BACK).
pub type AfterRun = Box<dyn FnOnce(&mut CrowApp, &mut Window, &mut Context<CrowApp>)>;

pub struct FleetRunner {
    pub run: FleetRun,
    jobs: Vec<RunJob>,
    pub input: Entity<InputState>,
    /// (button label, what it does), offered when the run is finished.
    pub after: Option<(String, AfterRun)>,
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
                    let target = push::PushTarget { server_id: &srv.id, server_name: &srv.name, path: &path, baseline: &baseline, author: &author, login_user: &srv.login_user, message: "Pushed the baseline", context: "fleet run: push baseline" };
                    push::push_baseline(host.as_ref(), &db, key.as_ref(), &target)
                }));
            }
        }
        let title = if steps.is_empty() && excluded.is_empty() { "PUSH BASELINE · nothing has drifted" } else { "PUSH BASELINE TO ALL" };
        self.open_fleet_run(FleetRun::new(title, "PUSH", steps, excluded), jobs, window, cx);
    }

    /// Plans rebooting every reachable server, one at a time, each waiting
    /// for the server to come back from a fresh boot, and its services to
    /// settle, before the next (ERR-80, ERR-141). Servers behind bastions go
    /// before their bastions.
    pub fn plan_rolling_reboot(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<_>) = (Vec::new(), Vec::new(), Vec::new());
        let ordered = crate::patching::reboot_order(self.fleet.servers.clone(), &self.fleet.servers);
        for srv in ordered {
            match self.reboot_work(&srv) {
                Err(why) => excluded.push((srv.name.clone(), why)),
                Ok((how, work)) => {
                    steps.push(FleetRun::step(&srv.id, &srv.name, how.clone()));
                    let db = self.vault.db();
                    jobs.push(Box::new(move || {
                        let result = work();
                        record_step(&db, &srv, "fleet.reboot", &how, &result);
                        result
                    }));
                }
            }
        }
        let n = steps.len();
        self.open_fleet_run(FleetRun::new(format!("ROLLING REBOOT · {n} SERVER{}", if n == 1 { "" } else { "S" }), "REBOOT", steps, excluded), jobs, window, cx);
    }

    /// How `srv` would be rebooted, and the work: reboot, wait for a fresh
    /// boot, then for systemd to settle with no unit failing that wasn't
    /// failing before. `Err` says why it can't be rebooted from Crow now.
    pub(crate) fn reboot_work(&self, srv: &crate::vault::ServerRecord) -> Result<(String, RunJob), String> {
        use crate::host::{reboot, transport_kind, TransportKind};
        use crow_provider_core::capabilities::INSTANCES_POWER;
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);
        const POLL: std::time::Duration = std::time::Duration::from_secs(5);
        let kind = transport_kind(srv);
        if kind == TransportKind::Local {
            return Err("the machine Crow runs on: reboot it yourself".to_string());
        }
        if !self.fleet.health(srv).is_ok() {
            return Err("not reachable right now, so Crow couldn't watch it come back".to_string());
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
        let srv = srv.clone();
        let work: RunJob = Box::new(move || {
            let host = host_for(&srv);
            let failed_before = crate::patching::system_state(host.as_ref()).ok().flatten().map(|(_, f)| f).unwrap_or_default();
            let trigger = || -> Result<(), String> {
                match provider {
                    Some((account, settings)) => {
                        let p = crate::providers::connect(&account, settings).map_err(|e| e.to_string())?;
                        crate::providers::run_action(p.as_ref(), &srv.provider_instance, crate::providers::ProviderAction::Reboot).map(|_| ())
                    }
                    None if kind == TransportKind::Container => {
                        let engine = if srv.tags.iter().any(|t| t == "docker") { "docker" } else { "podman" };
                        let out = std::process::Command::new(engine).args(["restart", "--", &srv.name]).output().map_err(|e| e.to_string())?;
                        if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
                    }
                    None => host.exec_privileged(&["systemctl", "reboot"], &[], crate::host::DEFAULT_TIMEOUT).map(|_| ()).map_err(|e| e.to_string()),
                }
            };
            let back = reboot::reboot_and_wait(host.as_ref(), trigger, TIMEOUT, POLL)?;
            let back = match back {
                crate::views::fleet::run::StepOutcome::Done(m) | crate::views::fleet::run::StepOutcome::Skipped(m) => m,
            };
            let services = crate::patching::wait_for_services(host.as_ref(), &failed_before, TIMEOUT, POLL)?;
            Ok(crate::views::fleet::run::StepOutcome::Done(format!("{back}; {services}")))
        });
        Ok((how, work))
    }

    /// Plans rotating every reachable SSH server's host keys and re-pinning
    /// them, one at a time (ERR-82).
    pub fn plan_rotate_host_keys(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::host::{transport_kind, TransportKind};
        use crate::security::host_keys;
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<_>) = (Vec::new(), Vec::new(), Vec::new());
        for srv in self.fleet.servers.clone() {
            match transport_kind(&srv) {
                TransportKind::Local => excluded.push((srv.name.clone(), "the machine Crow runs on".to_string())),
                TransportKind::Container => excluded.push((srv.name.clone(), "a lab container, reached without SSH".to_string())),
                TransportKind::Ssh if !self.fleet.health(&srv).is_ok() => {
                    excluded.push((srv.name.clone(), "not reachable right now: the new keys are fetched over the current connection".to_string()))
                }
                TransportKind::Ssh => {
                    let what = "new host keys, re-pin, verify a fresh login";
                    steps.push(FleetRun::step(&srv.id, &srv.name, what));
                    let db = self.vault.db();
                    jobs.push(Box::new(move || {
                        let host = host_for(&srv);
                        let port = if srv.port == 0 { 22 } else { srv.port };
                        let before = srv.host_key_fingerprint.clone().unwrap_or_default();
                        let result = host_keys::rotate(host.as_ref(), &srv.host, port, &host_keys::UserKnownHosts, &|| crate::host::ssh::fresh_key_login(&srv));
                        let step: StepResult = match result {
                            Ok(r) => {
                                if let Ok(db) = db.lock() {
                                    let mut updated = srv.clone();
                                    updated.host_key_fingerprint = Some(r.fingerprint.clone());
                                    let _ = db.upsert_server(&updated);
                                }
                                // The held connection was made with the old key.
                                crate::host::Host::close_connection(&crate::host::SshHost::for_server(&srv));
                                Ok(crate::views::fleet::run::StepOutcome::Done(format!("pinned {} (was {})", r.keys.join(", "), if before.is_empty() { "not pinned" } else { &before })))
                            }
                            Err(e) => Err(e),
                        };
                        record_step(&db, &srv, "ssh.host_key_rotate", what, &step);
                        step
                    }));
                }
            }
        }
        let n = steps.len();
        if n > 0 {
            excluded.push(("other machines".into(), "anyone else who SSHes to these servers will see a changed host key and must re-verify it".into()));
        }
        self.open_fleet_run(FleetRun::new(format!("ROTATE HOST KEYS · {n} SERVER{}", if n == 1 { "" } else { "S" }), "ROTATE", steps, excluded), jobs, window, cx);
    }

    /// Plans ending every other SSH login on every reachable SSH server,
    /// keeping Crow's own (ERR-83).
    pub fn plan_revoke_sessions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::host::{transport_kind, TransportKind};
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<_>) = (Vec::new(), Vec::new(), Vec::new());
        for srv in self.fleet.servers.clone() {
            match transport_kind(&srv) {
                TransportKind::Local => excluded.push((srv.name.clone(), "the machine Crow runs on (your own logins)".to_string())),
                TransportKind::Container => excluded.push((srv.name.clone(), "a lab container, reached without SSH".to_string())),
                TransportKind::Ssh if !self.fleet.health(&srv).is_ok() => excluded.push((srv.name.clone(), "not reachable right now".to_string())),
                TransportKind::Ssh => {
                    let what = "end every SSH login but Crow's";
                    steps.push(FleetRun::step(&srv.id, &srv.name, what));
                    let db = self.vault.db();
                    jobs.push(Box::new(move || {
                        let host = host_for(&srv);
                        let step: StepResult = crate::security::sessions::end_other_sessions(host.as_ref())
                            .map(|ended| crate::views::fleet::run::StepOutcome::Done(crate::security::sessions::summary(&ended)));
                        record_step(&db, &srv, "ssh.revoke_sessions", what, &step);
                        step
                    }));
                }
            }
        }
        if !steps.is_empty() {
            excluded.push(("keys".into(), "no key is removed: anyone whose key still works can log back in (rotate or remove keys for that)".into()));
        }
        let n = steps.len();
        self.open_fleet_run(FleetRun::new(format!("REVOKE SESSIONS · {n} SERVER{}", if n == 1 { "" } else { "S" }), "REVOKE", steps, excluded), jobs, window, cx);
    }

    /// Plans switching every server attached to `key_id` (that doesn't use
    /// it yet) to that key: install, prove with a fresh login, switch. The
    /// servers' previous keys stay in authorized_keys: you may use them
    /// yourself (ERR-90).
    pub fn plan_key_deploy(&mut self, key_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = self.keys.enrolled.iter().find(|k| k.id == key_id).cloned() else { return };
        let mut excluded = Vec::new();
        let targets: Vec<_> = self
            .fleet
            .servers
            .iter()
            .filter(|s| key.attached_servers.iter().any(|a| *a == s.id || *a == s.name))
            .filter(|s| s.key_id.as_deref() != Some(key_id))
            .cloned()
            .collect();
        if key.private_key_path.is_none() {
            excluded.push((key.name.clone(), "Crow has only the public half of this key, so it can't log in with it".into()));
        }
        let targets = if key.private_key_path.is_some() { targets } else { Vec::new() };
        let title = format!("SWITCH TO KEY {}", key.name);
        self.plan_key_switch(title, key, targets, false, excluded, window, cx);
    }

    /// Generates a new key next to `key_id`'s, then for every server using
    /// the old key: install the new one, prove it, switch, and remove the old
    /// one from authorized_keys (ERR-90).
    pub fn plan_key_rotation(&mut self, key_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(old) = self.keys.enrolled.iter().find(|k| k.id == key_id).cloned() else { return };
        let targets: Vec<_> = self.fleet.servers.iter().filter(|s| s.key_id.as_deref() == Some(key_id)).cloned().collect();
        if targets.is_empty() {
            self.keys.toast = Some(format!("No server logs in with {}; nothing to rotate.", old.name));
            cx.notify();
            return;
        }
        let dir = old
            .private_key_path
            .as_deref()
            .map(crate::keys::expand_tilde)
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| crate::keys::expand_tilde("~/.ssh"));
        // A second rotation the same day (say, after cancelling one, whose
        // new key stays on disk) gets the next free name.
        let base = format!("{}-{}", old.name, chrono::Local::now().format("%Y%m%d"));
        let taken = |n: &str| {
            let file = crate::keys::default_key_file(n, crate::keys::KeyAlgorithm::Ed25519);
            dir.join(&file).exists() || dir.join(format!("{file}.pub")).exists() || self.keys.enrolled.iter().any(|k| k.name == n)
        };
        let name = std::iter::once(base.clone()).chain((2..100).map(|i| format!("{base}-{i}"))).find(|n| !taken(n)).unwrap_or(base);
        let comment = format!("crow rotation of {}", old.name);
        let new_key = match crate::keys::generate_keypair(&name, crate::keys::KeyAlgorithm::Ed25519, Some(&comment), &old.group_id, &dir, None) {
            Ok((record, ..)) => record,
            Err(e) => {
                self.keys.toast = Some(format!("Couldn't generate the new key: {e}"));
                cx.notify();
                return;
            }
        };
        if let Ok(db) = self.vault.db().lock() {
            let mut record = new_key.clone();
            record.attached_servers = old.attached_servers.clone();
            let _ = db.upsert_ssh_key(&record);
        }
        self.refresh_keys(cx);
        let excluded = vec![(new_key.name.clone(), format!("new key written to {}; if you cancel it stays there, unused", dir.display()))];
        let title = format!("ROTATE {} → {}", old.name, new_key.name);
        self.plan_key_switch(title, new_key, targets, true, excluded, window, cx);
    }

    #[allow(clippy::too_many_arguments)]
    fn plan_key_switch(&mut self, title: String, key: crate::vault::SshKeyRecord, targets: Vec<crate::vault::ServerRecord>, revoke_old: bool, excluded: Vec<(String, String)>, window: &mut Window, cx: &mut Context<Self>) {
        // Verification resolves key paths through the SSH directory.
        self.sync_ssh_directory();
        let (mut steps, mut jobs): (Vec<_>, Vec<RunJob>) = (Vec::new(), Vec::new());
        for srv in targets {
            let old_blob = if revoke_old {
                srv.key_id.as_ref().and_then(|id| self.keys.enrolled.iter().find(|k| &k.id == id)).and_then(|k| crate::keys::deploy::key_blob(&k.public_key))
            } else {
                None
            };
            let what = if revoke_old { "install new key, verify, switch, remove old key" } else { "install key, verify, switch" };
            steps.push(FleetRun::step(&srv.id, &srv.name, what));
            let (db, key) = (self.vault.db(), key.clone());
            jobs.push(Box::new(move || {
                let host = host_for(&srv);
                let result = crate::keys::deploy::deploy_and_switch(host.as_ref(), &srv, &key, old_blob.as_deref(), crate::host::ssh::fresh_key_login);
                let step: StepResult = match result {
                    Ok(s) => {
                        if let Ok(db) = db.lock() {
                            let _ = db.upsert_server(&s.server);
                        }
                        // The held connection logged in with the old key.
                        crate::host::Host::close_connection(&crate::host::SshHost::for_server(&srv));
                        Ok(crate::views::fleet::run::StepOutcome::Done(if s.removed_old { "switched; old key removed".into() } else { "switched".into() }))
                    }
                    Err(e) => Err(e),
                };
                record_step(&db, &srv, "ssh.key_switch", &format!("{what} ({})", key.name), &step);
                step
            }));
        }
        self.open_fleet_run(FleetRun::new(title, "SWITCH", steps, excluded), jobs, window, cx);
    }

    pub(crate) fn open_fleet_run(&mut self, mut run: FleetRun, jobs: Vec<RunJob>, window: &mut Window, cx: &mut Context<Self>) {
        // Fleet runs need the role, and only reach the member's scope (ERR-150).
        if !self.allowed(crate::team::Permission::RunFleet, None, cx) {
            return;
        }
        let mut jobs = jobs;
        if self.team.config.enabled() {
            let mut keep = Vec::new();
            let (mut steps, mut kept_jobs, mut stages) = (Vec::new(), Vec::new(), Vec::new());
            for (i, (step, job)) in std::mem::take(&mut run.steps).into_iter().zip(jobs).enumerate() {
                let srv = self.fleet.servers.iter().find(|s| s.id == step.server_id).cloned();
                match srv.as_ref().map(|s| self.may(crate::team::Permission::RunFleet, Some(s))) {
                    Some(Err(why)) => run.excluded.push((step.server.clone(), why)),
                    _ => {
                        keep.push(i);
                        steps.push(step);
                        kept_jobs.push(job);
                        if let Some(st) = run.stages.get(i) {
                            stages.push(*st);
                        }
                    }
                }
            }
            run.steps = steps;
            run.stages = if run.stages.is_empty() { Vec::new() } else { stages };
            jobs = kept_jobs;
        }
        let keyword = run.keyword;
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(keyword));
        self.fleet_runner = Some(FleetRunner { run, jobs, input, after: None });
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
        // A second member's approval, when the team requires it (ERR-150).
        if self.team.config.enabled() && self.team.config.require_approval {
            match self.check_approval(cx) {
                Ok(by) => {
                    self.team.clear_approval = true;
                    self.team_notice(true, format!("Approved by {by}."), cx);
                }
                Err(e) => {
                    self.team.clear_approval = true;
                    if let Some(r) = self.fleet_runner.as_mut() {
                        r.run.error = Some(e);
                    }
                    cx.notify();
                    return;
                }
            }
        }
        let Some(runner) = self.fleet_runner.as_mut() else { return };
        runner.run.error = None;
        runner.run.phase = RunPhase::Running;
        let mut jobs: Vec<Option<RunJob>> = std::mem::take(&mut runner.jobs).into_iter().map(Some).collect();
        let groups = runner.run.stage_groups();
        let pause_after = runner.run.pause_after;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            for (stage, group) in groups.into_iter().enumerate() {
                let go = entity
                    .update(cx, |this, cx| {
                        let run = &mut this.fleet_runner.as_mut()?.run;
                        if run.stop_requested() {
                            run.finish();
                            cx.notify();
                            return None;
                        }
                        for &i in &group {
                            run.start(i);
                        }
                        cx.notify();
                        Some(())
                    })
                    .ok()
                    .flatten();
                if go.is_none() {
                    break;
                }
                // A stage's hosts run at the same time.
                let tasks: Vec<_> = group
                    .iter()
                    .filter_map(|&i| jobs.get_mut(i).and_then(Option::take).map(|job| (i, cx.background_executor().spawn(async move { job() }))))
                    .collect();
                let mut results = Vec::new();
                for (i, task) in tasks {
                    results.push((i, task.await));
                }
                let more = entity
                    .update(cx, |this, cx| {
                        let more = this.fleet_runner.as_mut().is_some_and(|r| r.run.record_stage(results));
                        if more && pause_after == Some(stage) {
                            if let Some(r) = this.fleet_runner.as_mut() {
                                r.run.phase = RunPhase::Paused;
                            }
                        }
                        cx.notify();
                        more
                    })
                    .unwrap_or(false);
                if !more {
                    break;
                }
                // Paused after the canary: wait for CONTINUE (or STOP).
                loop {
                    let phase = entity.update(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, r.run.stop_requested()))).ok().flatten();
                    match phase {
                        Some((RunPhase::Paused, false)) => cx.background_executor().timer(std::time::Duration::from_millis(250)).await,
                        _ => break,
                    }
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
                // Server records may have changed (key switches).
                this.reload_servers();
                // Packages may have changed (patching): read them again.
                this.after_patch_run(cx);
                // A hardening fix ran: read the posture again (ERR-145).
                if this.topology.posture_checked_at == 0 && !this.fleet.servers.is_empty() {
                    this.check_posture(cx);
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
        if let Some(r) = self.fleet_runner.as_mut() {
            r.run.request_stop();
            // Paused: nothing is running, so it's over now.
            if r.run.phase == RunPhase::Paused {
                r.run.finish();
            }
        }
        cx.notify();
    }

    /// The finished run's follow-up (e.g. ROLL BACK).
    pub fn run_fleet_after(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((_, action)) = self.fleet_runner.as_mut().filter(|r| r.run.phase == RunPhase::Finished).and_then(|r| r.after.take()) {
            action(self, window, cx);
        }
    }

    /// After the canary: go on with the rest.
    pub fn continue_fleet_run(&mut self, cx: &mut Context<Self>) {
        if let Some(r) = self.fleet_runner.as_mut().filter(|r| r.run.phase == RunPhase::Paused) {
            r.run.phase = RunPhase::Running;
        }
        cx.notify();
    }

    /// Closes the panel; a running run can't be closed, only stopped.
    pub fn close_fleet_run(&mut self, cx: &mut Context<Self>) {
        if self.fleet_runner.as_ref().is_some_and(|r| matches!(r.run.phase, RunPhase::Running | RunPhase::Paused)) {
            return;
        }
        self.fleet_runner = None;
        cx.notify();
    }
}

/// A change record for one host's step, once it has an outcome.
pub(crate) fn record_step(db: &std::sync::Mutex<crate::vault::VaultDb>, srv: &crate::vault::ServerRecord, kind: &str, what: &str, result: &StepResult) {
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
