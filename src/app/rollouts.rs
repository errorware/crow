//! Fleet Setup → ROLLOUTS (ERR-142): one command or one config file across
//! a group, a canary first, then batches; a config rollout can be rolled
//! back afterwards.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui_kit::component::input::{EditorState, InputEvent, InputState};
use gpui_kit::*;

use super::fleet_run::{record_step, RunJob};
use super::patching::Target;
use super::CrowApp;
use crate::host::{host_for, transport_kind, TransportKind};
use crate::rollout;
use crate::vault::ServerRecord;
use crate::views::fleet::run::FleetRun;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Command,
    Config,
}

/// What a server's file says now, for the preview.
#[derive(Clone, Debug, PartialEq)]
pub enum Current {
    Reading,
    Read(String),
    /// No such file there: the rollout creates it.
    Missing,
    Failed(String),
}

pub struct RolloutInputs {
    pub script: Entity<EditorState>,
    pub path: Entity<InputState>,
    pub content: Entity<EditorState>,
    _subs: Vec<Subscription>,
}

pub struct RolloutState {
    pub kind: Kind,
    pub as_root: bool,
    pub target: Target,
    pub use_canary: bool,
    /// The canary; `None`: the target's first server.
    pub canary: Option<String>,
    pub batch: usize,
    /// The path the preview was read for, and each server's current file.
    pub preview_path: String,
    pub current: HashMap<String, Current>,
    /// Whose diff is shown.
    pub selected: Option<String>,
    pub message: Option<String>,
}

impl Default for RolloutState {
    fn default() -> Self {
        Self { kind: Kind::Command, as_root: true, target: Target::All, use_canary: true, canary: None, batch: 1, preview_path: String::new(), current: HashMap::new(), selected: None, message: None }
    }
}

/// The servers a rollout reaches, and those left out with why.
pub fn candidates(servers: &[ServerRecord], target: &Target, healthy: impl Fn(&ServerRecord) -> bool) -> (Vec<ServerRecord>, Vec<(String, String)>) {
    let mut ok = Vec::new();
    let mut out = Vec::new();
    for s in servers.iter().filter(|s| target.includes(s)) {
        if transport_kind(s) == TransportKind::Local {
            out.push((s.name.clone(), "the machine Crow runs on: run it yourself".into()));
        } else if !healthy(s) {
            out.push((s.name.clone(), "not reachable right now".into()));
        } else {
            ok.push(s.clone());
        }
    }
    ok.sort_by(|a, b| a.name.cmp(&b.name));
    (ok, out)
}

impl CrowApp {
    pub fn ensure_rollout_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.rollout_inputs.is_some() {
            return;
        }
        let script = cx.new(|cx| EditorState::new(window, cx).placeholder("# runs with sh on each server, e.g.\nsystemctl reload nginx"));
        let path = cx.new(|cx| InputState::new(window, cx).placeholder("/etc/ssh/sshd_config.d/50-crow.conf"));
        let content = cx.new(|cx| EditorState::new(window, cx).placeholder("the file's new content"));
        let sub = cx.subscribe(&path, |this, input, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                let path = input.read(cx).value().trim().to_string();
                if path != this.rollouts.preview_path {
                    this.read_rollout_preview(cx);
                }
            }
        });
        let sub2 = cx.subscribe(&content, |_this, _e, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change) {
                cx.notify();
            }
        });
        self.rollout_inputs = Some(RolloutInputs { script, path, content, _subs: vec![sub, sub2] });
    }

    pub fn set_rollout_kind(&mut self, kind: Kind, cx: &mut Context<Self>) {
        self.rollouts.kind = kind;
        self.rollouts.message = None;
        cx.notify();
    }

    pub fn set_rollout_target(&mut self, target: Target, cx: &mut Context<Self>) {
        self.rollouts.target = target;
        self.rollouts.canary = None;
        if self.rollouts.kind == Kind::Config && !self.rollouts.preview_path.is_empty() {
            self.read_rollout_preview(cx);
        }
        cx.notify();
    }

    pub fn set_rollout_canary(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        match id {
            Some(id) => {
                self.rollouts.use_canary = true;
                self.rollouts.canary = Some(id);
            }
            None => self.rollouts.use_canary = false,
        }
        cx.notify();
    }

    pub fn set_rollout_batch(&mut self, batch: usize, cx: &mut Context<Self>) {
        self.rollouts.batch = batch.max(1);
        cx.notify();
    }

    pub fn toggle_rollout_root(&mut self, cx: &mut Context<Self>) {
        self.rollouts.as_root = !self.rollouts.as_root;
        cx.notify();
    }

    pub fn select_rollout_preview(&mut self, id: &str, cx: &mut Context<Self>) {
        self.rollouts.selected = Some(id.to_string());
        cx.notify();
    }

    /// Reads the file at the path on every server the rollout reaches.
    pub fn read_rollout_preview(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = &self.rollout_inputs else { return };
        let path = inputs.path.read(cx).value().trim().to_string();
        self.rollouts.preview_path = path.clone();
        self.rollouts.current.clear();
        if !path.starts_with('/') {
            cx.notify();
            return;
        }
        let (servers, _) = candidates(&self.fleet.servers, &self.rollouts.target, |s| self.fleet.health(s).is_ok());
        if self.rollouts.selected.as_ref().is_none_or(|s| !servers.iter().any(|x| &x.id == s)) {
            self.rollouts.selected = servers.first().map(|s| s.id.clone());
        }
        for srv in servers {
            self.rollouts.current.insert(srv.id.clone(), Current::Reading);
            let (id, path) = (srv.id.clone(), path.clone());
            cx.spawn(async move |entity, cx| {
                let job_path = path.clone();
                let read = cx
                    .background_executor()
                    .spawn(async move {
                        let path = job_path;
                        let host = host_for(&srv);
                        let quoted = crate::host::ssh::shell_quote(&path);
                        let script = format!("if [ -e {quoted} ]; then cat -- {quoted}; else echo @@crow-missing; fi");
                        match host.exec(&["sh", "-c", &script], crate::host::DEFAULT_TIMEOUT).or_else(|_| host.exec_privileged(&["sh", "-c", &script], &[], crate::host::DEFAULT_TIMEOUT)) {
                            Ok(o) if o.stdout.trim_end() == "@@crow-missing" => Current::Missing,
                            Ok(o) => Current::Read(o.stdout),
                            Err(e) => Current::Failed(e.to_string()),
                        }
                    })
                    .await;
                let _ = entity.update(cx, |this, cx| {
                    if this.rollouts.preview_path == path {
                        this.rollouts.current.insert(id, read);
                        cx.notify();
                    }
                });
            })
            .detach();
        }
        cx.notify();
    }

    /// Starts the new content from what a server (or a baseline) has.
    pub fn rollout_content_from(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(inputs) = &self.rollout_inputs {
            inputs.content.update(cx, |e, cx| e.set_value(text, window, cx));
        }
        cx.notify();
    }

    /// Fills path and content from a pinned baseline.
    pub fn rollout_from_baseline(&mut self, path: &str, revision_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let key = self.vault.key().cloned();
        let content = self.vault.db().lock().ok().and_then(|db| db.get_config_revision(revision_id).ok().flatten()).and_then(|rev| crate::config::history::open_content(&rev, key.as_ref()));
        let Some(content) = content else {
            self.rollouts.message = Some("That baseline's content wasn't kept (only its hash), so it can't be rolled out.".into());
            cx.notify();
            return;
        };
        if let Some(inputs) = &self.rollout_inputs {
            inputs.path.update(cx, |i, cx| i.set_value(path.to_string(), window, cx));
        }
        self.rollout_content_from(content, window, cx);
        self.read_rollout_preview(cx);
    }

    /// Plans the rollout on the fleet runner: canary, then batches.
    pub fn plan_rollout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inputs) = &self.rollout_inputs else { return };
        let st = &self.rollouts;
        let (mut servers, mut excluded) = candidates(&self.fleet.servers, &st.target, |s| self.fleet.health(s).is_ok());
        // The canary goes first.
        if st.use_canary {
            let canary = st.canary.clone().filter(|c| servers.iter().any(|s| &s.id == c)).or_else(|| servers.first().map(|s| s.id.clone()));
            if let Some(c) = canary {
                servers.sort_by_key(|s| s.id != c);
            }
        }
        let author = self.default_author();
        let (mut steps, mut jobs): (Vec<_>, Vec<RunJob>) = (Vec::new(), Vec::new());
        let previous: Arc<Mutex<HashMap<String, (ServerRecord, crate::config::push::Before)>>> = Arc::default();
        let (title, keyword, path) = match st.kind {
            Kind::Command => {
                let script = inputs.script.read(cx).value().to_string();
                if script.trim().is_empty() {
                    self.rollouts.message = Some("Write the command to run first.".into());
                    cx.notify();
                    return;
                }
                let first = script.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#')).unwrap_or("script").to_string();
                let what = format!("{}{first}{}", if st.as_root { "as root: " } else { "" }, if script.trim().lines().count() > 1 { " …" } else { "" });
                for srv in &servers {
                    steps.push(FleetRun::step(&srv.id, &srv.name, what.clone()));
                    let (db, srv, script, as_root, what) = (self.vault.db(), srv.clone(), script.clone(), st.as_root, what.clone());
                    jobs.push(Box::new(move || {
                        let result = rollout::run_command(host_for(&srv).as_ref(), &script, as_root);
                        record_step(&db, &srv, "rollout.command", &format!("{what}\n{script}"), &result);
                        result
                    }));
                }
                ("ROLLOUT · COMMAND".to_string(), "RUN", None)
            }
            Kind::Config => {
                let path = inputs.path.read(cx).value().trim().to_string();
                let content = inputs.content.read(cx).value().to_string();
                if !path.starts_with('/') {
                    self.rollouts.message = Some("The file's path must be absolute, like /etc/nginx/conf.d/app.conf.".into());
                    cx.notify();
                    return;
                }
                if content.trim().is_empty() {
                    self.rollouts.message = Some("Write the file's new content first (or start from a server's copy or a baseline).".into());
                    cx.notify();
                    return;
                }
                if path != st.preview_path {
                    self.read_rollout_preview(cx);
                    self.rollouts.message = Some("Reading each server's current file for the preview; plan again once it's read.".into());
                    cx.notify();
                    return;
                }
                let mut kept = Vec::new();
                for srv in servers {
                    let what = match st.current.get(&srv.id) {
                        Some(Current::Read(now)) => {
                            let (add, del) = rollout::diff_stats(&rollout::line_diff(now, &content));
                            if add == 0 && del == 0 {
                                excluded.push((srv.name.clone(), "already identical".into()));
                                continue;
                            }
                            format!("write {path} (+{add} −{del})")
                        }
                        Some(Current::Missing) => format!("create {path}"),
                        Some(Current::Failed(e)) => {
                            excluded.push((srv.name.clone(), format!("couldn't read its copy: {e}")));
                            continue;
                        }
                        Some(Current::Reading) | None => {
                            excluded.push((srv.name.clone(), "its copy hasn't been read yet".into()));
                            continue;
                        }
                    };
                    kept.push((srv, what));
                }
                for (srv, what) in kept {
                    steps.push(FleetRun::step(&srv.id, &srv.name, what.clone()));
                    let (db, key, author, path, content, previous) = (self.vault.db(), self.vault.key().cloned(), author.clone(), path.clone(), content.clone(), Arc::clone(&previous));
                    jobs.push(Box::new(move || {
                        let host = host_for(&srv);
                        let target = crate::config::push::PushTarget { server_id: &srv.id, server_name: &srv.name, path: &path, baseline: &content, author: &author, login_user: &srv.login_user, message: "Rolled out", context: "rollout" };
                        let (result, before) = crate::config::push::push_file(host.as_ref(), &db, key.as_ref(), &target);
                        if let Some(before) = before {
                            if let Ok(mut p) = previous.lock() {
                                p.insert(srv.id.clone(), (srv.clone(), before));
                            }
                        }
                        record_step(&db, &srv, "rollout.config", &what, &result);
                        result
                    }));
                }
                (format!("ROLLOUT · {path}"), "ROLLOUT", Some(path))
            }
        };
        let n = steps.len();
        let use_canary = st.use_canary && n > 1;
        let stages = rollout::stages(n, use_canary, st.batch);
        let title = format!("{title} · {n} SERVER{}", if n == 1 { "" } else { "S" });
        self.rollouts.message = None;
        let run = FleetRun::new(title, keyword, steps, excluded).staged(stages, use_canary.then_some(0));
        self.open_fleet_run(run, jobs, window, cx);
        // A config rollout can be undone where it wrote.
        if let (Some(path), Some(runner)) = (path, self.fleet_runner.as_mut()) {
            let author = author.clone();
            runner.after = Some((
                "ROLL BACK".into(),
                Box::new(move |this: &mut CrowApp, window: &mut Window, cx: &mut Context<CrowApp>| this.plan_rollback(&path, &previous, &author, window, cx)),
            ));
        }
    }

    /// Writes back what each server had before the rollout.
    fn plan_rollback(&mut self, path: &str, previous: &Arc<Mutex<HashMap<String, (ServerRecord, crate::config::push::Before)>>>, author: &str, window: &mut Window, cx: &mut Context<Self>) {
        let mut entries: Vec<(ServerRecord, crate::config::push::Before)> = previous.lock().map(|p| p.values().cloned().collect()).unwrap_or_default();
        entries.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        let (mut steps, mut jobs): (Vec<_>, Vec<RunJob>) = (Vec::new(), Vec::new());
        for (srv, before) in entries {
            let what = match &before {
                crate::config::push::Before::Missing => format!("remove {path} (it didn't exist)"),
                crate::config::push::Before::Content(_) => format!("put back {path} as it was"),
            };
            steps.push(FleetRun::step(&srv.id, &srv.name, what.clone()));
            let (db, key, author, path) = (self.vault.db(), self.vault.key().cloned(), author.to_string(), path.to_string());
            jobs.push(Box::new(move || {
                let host = host_for(&srv);
                let content = match &before {
                    crate::config::push::Before::Content(c) => c.as_str(),
                    crate::config::push::Before::Missing => "",
                };
                let target = crate::config::push::PushTarget { server_id: &srv.id, server_name: &srv.name, path: &path, baseline: content, author: &author, login_user: &srv.login_user, message: "Rolled back", context: "rollout rollback" };
                let result = crate::config::push::restore(host.as_ref(), &db, key.as_ref(), &target, &before);
                record_step(&db, &srv, "rollout.rollback", &what, &result);
                result
            }));
        }
        let n = steps.len();
        self.open_fleet_run(FleetRun::new(format!("ROLL BACK {path} · {n} SERVER{}", if n == 1 { "" } else { "S" }), "ROLLBACK", steps, Vec::new()), jobs, window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::candidates;
    use crate::app::patching::Target;
    use crate::vault::ServerRecord;

    #[test]
    fn a_rollout_reaches_the_targets_group_and_leaves_out_the_unreachable() {
        let s = |id: &str, g: &str| ServerRecord { id: id.into(), name: id.into(), host: format!("{id}.lan"), group_name: g.into(), ..Default::default() };
        let servers = vec![s("b", "web"), s("a", "web"), s("db", "data"), s("down", "web")];
        let (ok, out) = candidates(&servers, &Target::Group("web".into()), |x| x.id != "down");
        assert_eq!(ok.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(out, vec![("down".to_string(), "not reachable right now".to_string())]);
    }
}
