//! Fleet Setup → PEOPLE (ERR-144): who has access where, find a key
//! everywhere, offboard and onboard across servers.

use std::collections::{HashMap, HashSet};

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::fleet_run::{record_step, RunJob};
use super::patching::Target;
use super::CrowApp;
use crate::host::{host_for, transport_kind, TransportKind};
use crate::people::{self, Offboard, ServerAccounts};
use crate::views::fleet::run::{FleetRun, StepOutcome};
use crate::views::users::host_data::Argv;

#[derive(Default)]
pub struct PeopleState {
    pub scans: HashMap<String, Result<ServerAccounts, String>>,
    pub reading: HashSet<String>,
    pub last_read: i64,
    pub key_query: Option<Entity<InputState>>,
    pub new_user: Option<Entity<InputState>>,
    pub new_key: Option<Entity<InputState>>,
    pub new_sudo: bool,
    pub target: Target,
    pub selected: Option<String>,
    pub message: Option<(bool, String)>,
}

impl PeopleState {
    pub fn read(&self) -> Vec<ServerAccounts> {
        let mut v: Vec<ServerAccounts> = self.scans.values().filter_map(|r| r.as_ref().ok()).cloned().collect();
        v.sort_by(|a, b| a.server.cmp(&b.server));
        v
    }
}

/// Runs each command as root, in order; the first failure stops it.
fn run_all(host: &dyn crate::host::Host, cmds: &[Argv]) -> Result<(), String> {
    for c in cmds {
        let argv: Vec<&str> = c.iter().map(String::as_str).collect();
        host.exec_privileged(&argv, &[], crate::host::DEFAULT_TIMEOUT).map_err(|e| match e {
            crate::host::HostError::Failed { stderr, status } => format!("`{}` failed (exit {status}): {}", c.first().cloned().unwrap_or_default(), stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim()),
            other => other.to_string(),
        })?;
    }
    Ok(())
}

impl CrowApp {
    pub fn ensure_people_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let p = &mut self.people;
        if p.key_query.is_none() {
            p.key_query = Some(cx.new(|cx| InputState::new(window, cx).placeholder("a key's fingerprint (SHA256:…) or comment")));
            p.new_user = Some(cx.new(|cx| InputState::new(window, cx).placeholder("username, e.g. alice")));
            p.new_key = Some(cx.new(|cx| InputState::new(window, cx).placeholder("their public key: ssh-ed25519 AAAA… alice@laptop")));
        }
    }

    /// Reads every reachable server's accounts and keys in the background.
    pub fn refresh_people(&mut self, cx: &mut Context<Self>) {
        self.sync_ssh_directory();
        let blob = |srv: &crate::vault::ServerRecord| srv.key_id.as_ref().and_then(|id| self.keys.enrolled.iter().find(|k| &k.id == id)).and_then(|k| crate::keys::deploy::key_blob(&k.public_key));
        let servers: Vec<_> = self.fleet.servers.iter().filter(|s| self.fleet.health(s).is_ok() && transport_kind(s) != TransportKind::Local && !self.people.reading.contains(&s.id)).map(|s| (s.clone(), blob(s))).collect();
        self.people.last_read = chrono::Utc::now().timestamp();
        for (srv, crow_key_blob) in servers {
            self.people.reading.insert(srv.id.clone());
            let id = srv.id.clone();
            cx.spawn(async move |entity, cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        crate::views::users::host_data::read_accounts(host_for(&srv).as_ref()).map(|(users, groups)| ServerAccounts {
                            server_id: srv.id.clone(),
                            server: srv.name.clone(),
                            crow_user: srv.login_user.clone(),
                            crow_key_blob,
                            users,
                            groups: groups.into_iter().map(|(g, _)| g).collect(),
                        })
                    })
                    .await;
                let _ = entity.update(cx, |this, cx| {
                    this.people.reading.remove(&id);
                    this.people.scans.insert(id, result);
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub fn select_person(&mut self, who: Option<String>, cx: &mut Context<Self>) {
        self.people.selected = who;
        cx.notify();
    }

    pub fn set_people_target(&mut self, target: Target, cx: &mut Context<Self>) {
        self.people.target = target;
        cx.notify();
    }

    pub fn toggle_people_sudo(&mut self, cx: &mut Context<Self>) {
        self.people.new_sudo = !self.people.new_sudo;
        cx.notify();
    }

    fn people_run(&mut self, title: String, keyword: &'static str, kind: &'static str, plan: Vec<(ServerAccounts, Result<Vec<Argv>, String>)>, what: impl Fn(&[Argv]) -> String, window: &mut Window, cx: &mut Context<Self>) {
        let (mut steps, mut jobs, mut excluded): (Vec<_>, Vec<RunJob>, Vec<(String, String)>) = (Vec::new(), Vec::new(), Vec::new());
        for (s, cmds) in plan {
            let Some(srv) = self.fleet.servers.iter().find(|x| x.id == s.server_id).cloned() else { continue };
            match cmds {
                Err(why) => excluded.push((s.server.clone(), why)),
                Ok(cmds) => {
                    let w = what(&cmds);
                    steps.push(FleetRun::step(&srv.id, &srv.name, w.clone()));
                    let db = self.vault.db();
                    jobs.push(Box::new(move || {
                        let result = run_all(host_for(&srv).as_ref(), &cmds).map(|()| StepOutcome::Done("done".into()));
                        record_step(&db, &srv, kind, &w, &result);
                        result
                    }));
                }
            }
        }
        self.open_fleet_run(FleetRun::new(title, keyword, steps, excluded), jobs, window, cx);
        // Read accounts again afterwards.
        self.people.last_read = 0;
    }

    /// Offboards `user` on every server it's on.
    pub fn plan_offboard(&mut self, user: &str, mode: Offboard, window: &mut Window, cx: &mut Context<Self>) {
        let plan: Vec<_> = self.people.read().into_iter().filter(|s| s.users.iter().any(|u| u.username == user)).map(|s| {
            let c = people::offboard_commands(&s, user, mode);
            (s, c)
        }).collect();
        let verb = match mode {
            Offboard::Lock => "revoke keys, lock and expire",
            Offboard::Delete => "revoke keys, delete the account (home kept)",
        };
        let n = plan.iter().filter(|(_, c)| c.is_ok()).count();
        self.people_run(format!("OFFBOARD {user} · {n} SERVER{}", if n == 1 { "" } else { "S" }), "OFFBOARD", "people.offboard", plan, |c| format!("{user}: {verb} ({} step{})", c.len(), if c.len() == 1 { "" } else { "s" }), window, cx);
    }

    /// Removes a key from every account on every server it opens.
    pub fn plan_revoke_key_everywhere(&mut self, blob: &str, label: &str, window: &mut Window, cx: &mut Context<Self>) {
        let plan: Vec<_> = self.people.read().into_iter().filter(|s| s.users.iter().any(|u| u.authorized_keys.iter().any(|k| k.key_preview.contains(blob)))).map(|s| {
            let c = people::revoke_key_commands(&s, blob);
            (s, c)
        }).collect();
        let n = plan.iter().filter(|(_, c)| c.is_ok()).count();
        self.people_run(format!("REVOKE KEY {label} · {n} SERVER{}", if n == 1 { "" } else { "S" }), "REVOKE", "people.revoke_key", plan, |c| format!("remove the key from {} account{}", c.len(), if c.len() == 1 { "" } else { "s" }), window, cx);
    }

    /// Gives a user a login with a key on the target's servers.
    pub fn plan_onboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let user = self.people.new_user.as_ref().map(|i| i.read(cx).value().trim().to_lowercase()).unwrap_or_default();
        let key = self.people.new_key.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        if user.is_empty() || key.is_empty() {
            self.people.message = Some((false, "Fill in the username and their public key first.".into()));
            cx.notify();
            return;
        }
        let target = self.people.target.clone();
        let sudo = self.people.new_sudo;
        let in_target: HashSet<String> = self.fleet.servers.iter().filter(|s| target.includes(s)).map(|s| s.id.clone()).collect();
        let plan: Vec<_> = self.people.read().into_iter().filter(|s| in_target.contains(&s.server_id)).map(|s| {
            let c = people::onboard_commands(&s, &user, &key, sudo);
            (s, c)
        }).collect();
        if plan.is_empty() {
            self.people.message = Some((false, format!("No server in {} has been read yet: READ AGAIN first.", target.label())));
            cx.notify();
            return;
        }
        self.people.message = None;
        let n = plan.iter().filter(|(_, c)| c.is_ok()).count();
        self.people_run(format!("ONBOARD {user} · {} · {n} SERVER{}", target.label().to_uppercase(), if n == 1 { "" } else { "S" }), "ONBOARD", "people.onboard", plan, |c| if c.len() > 1 { format!("create {user}{}, add their key", if sudo { " with sudo" } else { "" }) } else { format!("add the key to {user}") }, window, cx);
    }

    pub fn people_tick(&mut self, cx: &mut Context<Self>) {
        // Read again after a run; otherwise only when the page opens.
        if self.people.last_read == 0 && self.fleet_runner.as_ref().is_none_or(|r| r.run.phase == crate::views::fleet::run::RunPhase::Finished) && self.screen == super::Screen::FleetSetup && self.fleet.setup_page == crate::views::fleet::state::SetupPage::People {
            self.refresh_people(cx);
        }
    }
}
