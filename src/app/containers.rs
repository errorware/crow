//! The server's Containers page (ERR-140): scans, the selected container's
//! logs and inspect, actions behind a confirmation, and the wand.

use std::collections::{HashMap, HashSet};

use gpui_kit::*;

use super::host_actions::HostCommand;
use super::CrowApp;
use crate::containers::history::{self, ContainerCharts};
use crate::containers::{self, Action, Scan, Scope, StackAction};
use crate::host::host_for;
use crate::metrics::chart::Range;
use crate::views::overview::state::ServiceEli5;

/// How often an open Containers page reads the server again.
const RESCAN_SECS: i64 = 15;
/// How often followed logs are read again.
const FOLLOW_SECS: i64 = 3;
/// How often every server's containers are sampled for history (ERR-201).
const SAMPLE_SECS: i64 = 60;
/// A server with neither Docker nor Podman is asked again after this long.
const NO_RUNTIME_RECHECK_SECS: i64 = 3_600;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerTab {
    #[default]
    Logs,
    Usage,
    Inspect,
}

/// An action waiting for its second click.
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    Container { key: String, action: Action },
    Stack { project: String, action: StackAction },
}

#[derive(Default)]
pub struct ContainersState {
    /// Which server `scan` belongs to.
    pub server_id: Option<String>,
    pub scan: Option<Result<Scan, String>>,
    pub loading: bool,
    pub scanned_at: i64,
    pub selected: Option<String>,
    pub tab: DrawerTab,
    /// (container key, its last lines).
    pub logs: Option<(String, Result<String, String>)>,
    pub logs_loading: bool,
    pub logs_at: i64,
    pub follow: bool,
    pub inspect: Option<(String, Result<String, String>)>,
    pub inspect_loading: bool,
    pub pending: Option<Pending>,
    /// What's running now ("docker restart web").
    pub busy: Option<String>,
    /// What the last action did.
    pub result: Option<(bool, String)>,
    pub eli5: HashMap<String, Result<ServiceEli5, String>>,
    pub eli5_loading: HashSet<String>,
    pub eli5_open: bool,
    /// The last scan of each server, for the fleet map.
    pub by_server: HashMap<String, Scan>,
    /// The minute sampler (ERR-201): when it last ran, whether it's out.
    pub sampled_at: i64,
    pub sampling: bool,
    /// Servers without a container runtime, left alone until then.
    pub no_runtime_until: HashMap<String, i64>,
    pub usage_range: Range,
    /// (container key, range, sampled_at it was built at, the charts).
    pub usage: Option<(String, Range, i64, ContainerCharts)>,
    pub usage_hover: Option<f32>,
}

impl ContainersState {
    pub fn current(&self) -> Option<&Scan> {
        self.scan.as_ref().and_then(|s| s.as_ref().ok())
    }
}

impl CrowApp {
    /// Reads the active server's containers in the background.
    pub fn refresh_containers(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        if self.containers.server_id.as_deref() != Some(srv.id.as_str()) {
            let old = std::mem::take(&mut self.containers);
            self.containers = ContainersState {
                server_id: Some(srv.id.clone()),
                follow: true,
                by_server: old.by_server,
                sampled_at: old.sampled_at,
                sampling: old.sampling,
                no_runtime_until: old.no_runtime_until,
                usage_range: old.usage_range,
                ..Default::default()
            };
        }
        if self.containers.loading {
            return;
        }
        self.containers.loading = true;
        self.containers.scanned_at = chrono::Utc::now().timestamp();
        cx.notify();
        let server_id = srv.id.clone();
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { containers::scan(host_for(&srv).as_ref()) }).await;
            let _ = entity.update(cx, |this, cx| {
                if this.containers.server_id.as_deref() != Some(server_id.as_str()) {
                    return;
                }
                this.containers.loading = false;
                if let Ok(scan) = &result {
                    this.containers.by_server.insert(server_id, scan.clone());
                    // The selected container went away (removed, recreated).
                    if let Some(key) = &this.containers.selected {
                        if scan.find(key).is_none() {
                            this.containers.selected = None;
                        }
                    }
                }
                this.containers.scan = Some(result);
                cx.notify();
            });
        })
        .detach();
    }

    /// Every poll: an open Containers page stays current, and followed
    /// logs keep coming.
    pub fn containers_tick(&mut self, cx: &mut Context<Self>) {
        self.sample_containers(cx);
        if self.screen != super::Screen::Server || self.active_view != "containers" {
            return;
        }
        let now = chrono::Utc::now().timestamp();
        if now - self.containers.scanned_at >= RESCAN_SECS && self.containers.busy.is_none() {
            self.refresh_containers(cx);
        }
        if self.containers.tab == DrawerTab::Usage {
            self.load_container_usage();
        }
        if self.containers.follow && self.containers.tab == DrawerTab::Logs && self.containers.selected.is_some() && now - self.containers.logs_at >= FOLLOW_SECS {
            self.load_container_logs(cx);
        }
    }

    pub fn select_container(&mut self, key: &str, cx: &mut Context<Self>) {
        if self.containers.selected.as_deref() == Some(key) {
            self.containers.selected = None;
            cx.notify();
            return;
        }
        self.containers.selected = Some(key.to_string());
        self.containers.pending = None;
        self.containers.eli5_open = false;
        self.containers.logs = None;
        self.containers.inspect = None;
        match self.containers.tab {
            DrawerTab::Logs => self.load_container_logs(cx),
            DrawerTab::Usage => self.load_container_usage(),
            DrawerTab::Inspect => self.load_container_inspect(cx),
        }
        cx.notify();
    }

    pub fn close_container_drawer(&mut self, cx: &mut Context<Self>) {
        self.containers.selected = None;
        self.containers.pending = None;
        cx.notify();
    }

    pub fn set_container_tab(&mut self, tab: DrawerTab, cx: &mut Context<Self>) {
        self.containers.tab = tab;
        match tab {
            DrawerTab::Logs if self.containers.logs.is_none() => self.load_container_logs(cx),
            DrawerTab::Inspect if self.containers.inspect.is_none() => self.load_container_inspect(cx),
            DrawerTab::Usage => self.load_container_usage(),
            _ => {}
        }
        cx.notify();
    }

    /// Every server Crow can reach, once a minute: its containers' usage
    /// into history, and its scan for the fleet map. Each server is read
    /// on its own thread, so one slow server doesn't hold up the rest.
    fn sample_containers(&mut self, cx: &mut Context<Self>) {
        use crate::views::fleet::state::FleetHealth;
        let now = chrono::Utc::now().timestamp();
        if self.containers.sampling || now - self.containers.sampled_at < SAMPLE_SECS {
            return;
        }
        self.containers.sampled_at = now;
        let servers: Vec<_> = self
            .fleet
            .servers
            .iter()
            .filter(|s| matches!(self.fleet.health(s), FleetHealth::Ok))
            .filter(|s| self.containers.no_runtime_until.get(&s.id).is_none_or(|&until| now >= until))
            .cloned()
            .collect();
        if servers.is_empty() {
            return;
        }
        self.containers.sampling = true;
        cx.spawn(async move |entity, cx| {
            let scans = cx
                .background_executor()
                .spawn(async move {
                    std::thread::scope(|sc| {
                        let jobs: Vec<_> = servers.iter().map(|s| sc.spawn(move || (s.id.clone(), containers::scan(host_for(s).as_ref())))).collect();
                        jobs.into_iter().filter_map(|j| j.join().ok()).collect::<Vec<_>>()
                    })
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.containers.sampling = false;
                let mut samples = Vec::new();
                for (server_id, scan) in scans {
                    let Ok(scan) = scan else { continue };
                    if scan.runtimes.is_empty() {
                        this.containers.no_runtime_until.insert(server_id.clone(), now + NO_RUNTIME_RECHECK_SECS);
                    }
                    samples.extend(history::samples_from(&server_id, now, &scan));
                    this.containers.by_server.insert(server_id, scan);
                }
                if let Ok(db) = this.vault.db().lock() {
                    let _ = db.insert_container_samples(&samples);
                }
                if this.containers.tab == DrawerTab::Usage {
                    this.load_container_usage();
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The selected container's charts, rebuilt when the range changes or a
    /// new sample came in.
    pub fn load_container_usage(&mut self) {
        let Some((srv, c)) = self.selected_container() else { return };
        let (key, range, at) = (c.key(), self.containers.usage_range, self.containers.sampled_at);
        if self.containers.usage.as_ref().is_some_and(|(k, r, t, _)| *k == key && *r == range && *t == at) {
            return;
        }
        let now = chrono::Utc::now().timestamp();
        let from = now - range.secs();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let samples = db.list_container_history(&srv.id, &c.name, from).unwrap_or_default();
        let rows = db.list_history(&srv.id, from).unwrap_or_default();
        let sessions = db.list_watch_sessions(from).unwrap_or_default();
        drop(db);
        self.containers.usage = Some((key, range, at, history::charts(&samples, &rows, &sessions, from, now, range.bucket_secs())));
    }

    pub fn set_container_usage_range(&mut self, range: Range, cx: &mut Context<Self>) {
        self.containers.usage_range = range;
        self.containers.usage_hover = None;
        self.load_container_usage();
        cx.notify();
    }

    pub fn set_container_usage_hover(&mut self, x: Option<f32>, cx: &mut Context<Self>) {
        if self.containers.usage_hover != x {
            self.containers.usage_hover = x;
            cx.notify();
        }
    }

    pub fn toggle_container_follow(&mut self, cx: &mut Context<Self>) {
        self.containers.follow = !self.containers.follow;
        if self.containers.follow {
            self.load_container_logs(cx);
        }
        cx.notify();
    }

    fn selected_container(&self) -> Option<(crate::vault::ServerRecord, containers::Container)> {
        let srv = self.fleet.active_server()?;
        let key = self.containers.selected.as_ref()?;
        let c = self.containers.current()?.find(key)?.clone();
        Some((srv, c))
    }

    pub fn load_container_logs(&mut self, cx: &mut Context<Self>) {
        if self.containers.logs_loading {
            return;
        }
        let Some((srv, c)) = self.selected_container() else { return };
        self.containers.logs_loading = true;
        self.containers.logs_at = chrono::Utc::now().timestamp();
        let key = c.key();
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { containers::logs(host_for(&srv).as_ref(), &c) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.containers.logs_loading = false;
                this.containers.logs_at = chrono::Utc::now().timestamp();
                if this.containers.selected.as_deref() == Some(key.as_str()) {
                    this.containers.logs = Some((key, result));
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn load_container_inspect(&mut self, cx: &mut Context<Self>) {
        if self.containers.inspect_loading {
            return;
        }
        let Some((srv, c)) = self.selected_container() else { return };
        self.containers.inspect_loading = true;
        let key = c.key();
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { containers::inspect(host_for(&srv).as_ref(), &c) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.containers.inspect_loading = false;
                if this.containers.selected.as_deref() == Some(key.as_str()) {
                    this.containers.inspect = Some((key, result));
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The first click asks; the second (`confirm_container_pending`) runs.
    pub fn request_container_action(&mut self, pending: Pending, cx: &mut Context<Self>) {
        self.containers.pending = if self.containers.pending.as_ref() == Some(&pending) { None } else { Some(pending) };
        self.containers.result = None;
        cx.notify();
    }

    pub fn cancel_container_pending(&mut self, cx: &mut Context<Self>) {
        self.containers.pending = None;
        cx.notify();
    }

    pub fn confirm_container_pending(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.containers.pending.take() else { return };
        let Some(scan) = self.containers.current().cloned() else { return };
        let (kind, target, argv, scope) = match &pending {
            Pending::Container { key, action } => {
                let Some(c) = scan.find(key) else { return };
                (format!("container.{}", action.verb()), c.name.clone(), containers::action_argv(c, *action), c.scope)
            }
            Pending::Stack { project, action } => {
                let Some(stack) = scan.stacks().into_iter().find(|s| s.project == *project) else { return };
                match containers::stack_argv(&stack, *action) {
                    Ok(argv) => (format!("compose.{}", action.verb()), project.clone(), argv, stack.scope),
                    Err(e) => {
                        self.containers.result = Some((false, e));
                        cx.notify();
                        return;
                    }
                }
            }
        };
        let mut cmd = HostCommand::new(argv);
        cmd.as_user = scope == Scope::User;
        self.containers.busy = Some(super::host_actions::describe_commands(std::slice::from_ref(&cmd)));
        self.containers.result = None;
        cx.notify();
        self.run_host_action(
            &kind,
            &target,
            vec![cmd],
            |srv| containers::scan(host_for(srv).as_ref()),
            move |this, rescan, result, summary, cx| {
                this.containers.busy = None;
                this.containers.result = Some(match result {
                    Ok(()) => (true, t!("app-containers-done", summary = summary)),
                    Err(e) => (false, t!("app-containers-failed", summary = summary, e = e)),
                });
                if let Ok(scan) = &rescan {
                    if let Some(id) = this.containers.server_id.clone() {
                        this.containers.by_server.insert(id, scan.clone());
                    }
                }
                this.containers.scan = Some(rescan);
                this.containers.scanned_at = chrono::Utc::now().timestamp();
                this.containers.logs = None;
                if this.containers.selected.is_some() {
                    this.load_container_logs(cx);
                }
            },
            cx,
        );
    }

    /// The wand: what this container is and how risky stopping it is.
    pub fn explain_container(&mut self, cx: &mut Context<Self>) {
        if self.containers.eli5_open {
            self.containers.eli5_open = false;
            cx.notify();
            return;
        }
        let Some((_, c)) = self.selected_container() else { return };
        self.containers.eli5_open = true;
        // Keyed by image and service: the same software, the same answer.
        let key = format!("{}|{}", c.image, c.service().unwrap_or_default());
        if matches!(self.containers.eli5.get(&key), Some(Ok(_))) || self.containers.eli5_loading.contains(&key) {
            cx.notify();
            return;
        }
        let Some((primary, backup)) = self.ai_providers() else {
            self.containers.eli5.insert(key, Err(t!("app-containers-no-ai-provider-has-a").into()));
            cx.notify();
            return;
        };
        let ports: Vec<String> = c
            .published()
            .iter()
            .map(|p| format!("{}->{} ({})", p.host_port, p.container, if p.exposed() { t!("app-containers-every-network") } else { t!("app-containers-this-machine-only") }))
            .collect();
        let (image, service, state) = (c.image.clone(), c.service().map(String::from), c.state.clone());
        self.containers.eli5.remove(&key);
        self.containers.eli5_loading.insert(key.clone());
        cx.notify();
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let answer = cx
                .background_executor()
                .spawn(async move { crate::ai::explain_container(&primary, backup.as_ref(), &crate::ai::ContainerFacts { image: &image, service: service.as_deref(), state: &state, ports: &ports }) })
                .await;
            if let Ok(a) = &answer {
                if let Ok(db) = db.lock() {
                    let _ = db.record_clanker_call(&a.provider_id);
                }
            }
            let _ = entity.update(cx, |this, cx| {
                this.containers.eli5_loading.remove(&key);
                let result = answer.map(|a| {
                    let (severity, body) = crate::ai::split_severity(&a.text);
                    ServiceEli5 { severity, body, via: a.provider_label, primary_failed: a.primary_failed }
                });
                this.containers.eli5.insert(key, result);
                this.refresh_clankers(cx);
            });
        })
        .detach();
    }
}
