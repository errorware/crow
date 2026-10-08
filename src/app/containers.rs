//! The server's Containers page (ERR-140): scans, the selected container's
//! logs and inspect, actions behind a confirmation, and the wand.

use std::collections::{HashMap, HashSet};

use gpui_kit::*;

use super::host_actions::HostCommand;
use super::CrowApp;
use crate::containers::{self, Action, Scan, Scope, StackAction};
use crate::host::host_for;
use crate::views::overview::state::ServiceEli5;

/// How often an open Containers page reads the server again.
const RESCAN_SECS: i64 = 15;
/// How often followed logs are read again.
const FOLLOW_SECS: i64 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerTab {
    #[default]
    Logs,
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
            let by_server = std::mem::take(&mut self.containers.by_server);
            self.containers = ContainersState { server_id: Some(srv.id.clone()), follow: true, by_server, ..Default::default() };
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
        if self.screen != super::Screen::Server || self.active_view != "containers" {
            return;
        }
        let now = chrono::Utc::now().timestamp();
        if now - self.containers.scanned_at >= RESCAN_SECS && self.containers.busy.is_none() {
            self.refresh_containers(cx);
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
            _ => {}
        }
        cx.notify();
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
                    Ok(()) => (true, format!("Done: {summary}")),
                    Err(e) => (false, format!("{summary} failed: {e}")),
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
            self.containers.eli5.insert(key, Err("No AI provider has a key yet. Add one in Settings → Clankers (AI).".into()));
            cx.notify();
            return;
        };
        let ports: Vec<String> = c
            .published()
            .iter()
            .map(|p| format!("{}->{} ({})", p.host_port, p.container, if p.exposed() { "every network" } else { "this machine only" }))
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
