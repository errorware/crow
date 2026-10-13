//! Refreshing plugins' badges (ERR-182) on the poll tick: each accepted
//! plugin's badge action, on each reachable server, at most every
//! `badge.every` (at least a minute). A server action runs per server; an
//! action on this machine runs once for all of them and reads `{results}`.
//! Badge reads stay out of the run history: they'd drown it.

use std::collections::{BTreeMap, HashMap};

use gpui_kit::*;

use super::CrowApp;
use crate::plugin_host::badges::{self, BadgeView};
use crate::plugin_host::local;
use crate::plugin_host::manifest::{Action, Runs};
use crate::plugin_host::run::{self, ServerValues, State, Status, Values};
use crate::vault::ServerRecord;

/// When each (plugin, server) badge was last read.
#[derive(Default)]
pub struct BadgeClock {
    pub read_at: HashMap<(String, String), i64>,
    pub busy: std::collections::HashSet<String>,
}

/// The status a local action's `{results}` line gives.
fn status_of_line(l: &serde_json::Value) -> Option<Status> {
    let state = match l.get("state").and_then(|s| s.as_str())? {
        "ok" => State::Ok,
        "warn" => State::Warn,
        "bad" => State::Bad,
        _ => State::Unknown,
    };
    Some(Status { state, label: l.get("label").and_then(|s| s.as_str()).unwrap_or_default().chars().filter(|c| !c.is_control()).take(80).collect() })
}

impl CrowApp {
    pub fn plugin_badges_tick(&mut self, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        let usable: Vec<_> = self.plugins.user.installed.iter().filter(|p| p.usable()).cloned().collect();
        badges::retain_plugins(&usable.iter().map(|p| p.id.clone()).collect::<Vec<_>>());
        for p in usable {
            let Some(m) = p.report.manifest.clone() else { continue };
            let Some(badge) = m.badge.clone() else { continue };
            let Some(action) = m.actions.iter().find(|a| a.id == badge.action).cloned() else { continue };
            if self.plugins.user.badge_clock.busy.contains(&p.id) {
                continue;
            }
            if action.on == Runs::Local && m.setup.is_some() && self.user_plugin_setup_hash(&p.id).as_deref() != Some(p.hash.as_str()) {
                continue;
            }
            let due: Vec<ServerRecord> = self
                .fleet
                .servers
                .iter()
                .filter(|s| self.fleet.health(s).is_ok())
                .filter(|s| action.on == Runs::Server || !s.tags.iter().any(|t| t == "test-node"))
                .filter(|s| crate::host::transport_kind(s) != crate::host::TransportKind::Local || action.on == Runs::Server)
                .filter(|s| self.plugins.user.badge_clock.read_at.get(&(p.id.clone(), s.id.clone())).is_none_or(|t| now - t >= badge.every.0 as i64))
                .cloned()
                .collect();
            if due.is_empty() {
                continue;
            }
            for s in &due {
                self.plugins.user.badge_clock.read_at.insert((p.id.clone(), s.id.clone()), now);
            }
            let stored = self.user_plugin_settings(&p.id, &m.settings);
            let (settings, secrets, missing) = run::settings_values(&m.settings, &stored);
            if !missing.is_empty() {
                continue;
            }
            let last_applied: HashMap<String, i64> = badge
                .last_run
                .as_ref()
                .map(|a| {
                    let db = self.vault.db();
                    let runs = db.lock().ok().and_then(|db| db.list_plugin_runs(&p.id, None, 500).ok()).unwrap_or_default();
                    let mut out = HashMap::new();
                    for r in runs.into_iter().filter(|r| &r.action_id == a && r.ok && !r.dry_run) {
                        out.entry(r.server_id).or_insert(r.finished_at);
                    }
                    out
                })
                .unwrap_or_default();
            self.plugins.user.badge_clock.busy.insert(p.id.clone());
            let (plugin_id, dir, name, requires) = (p.id.clone(), p.dir.clone(), p.name().to_string(), m.requires.clone());
            let job_action: Action = action.clone();
            cx.spawn(async move |entity, cx| {
                let found: Vec<(String, Result<Status, String>)> = cx
                    .background_executor()
                    .spawn(async move { read_badges(&plugin_id, &dir, &job_action, &requires, &due, settings, secrets) })
                    .await;
                let _ = entity.update(cx, |this, cx| {
                    for (server_id, status) in found {
                        // A badge that couldn't be read shows nothing rather than a guess.
                        let view = status.ok().map(|st| {
                            let when = last_applied.get(&server_id).and_then(|t| chrono::DateTime::from_timestamp(*t, 0)).map(|t| t.with_timezone(&chrono::Local).format(" · %b %d").to_string()).unwrap_or_default();
                            let label = if st.label.is_empty() { name.clone() } else { st.label };
                            BadgeView { plugin: p.id.clone(), text: format!("{label}{when}"), state: st.state }
                        });
                        badges::set(&server_id, &p.id, view);
                    }
                    this.plugins.user.badge_clock.busy.remove(&p.id);
                    cx.notify();
                });
            })
            .detach();
        }
    }
}

/// Reads one plugin's badge on `servers`: blocking, off the UI thread.
fn read_badges(plugin_id: &str, dir: &std::path::Path, action: &Action, requires: &crate::plugin_host::manifest::Requires, servers: &[ServerRecord], settings: BTreeMap<String, String>, secrets: BTreeMap<String, String>) -> Vec<(String, Result<Status, String>)> {
    let timeout = std::time::Duration::from_secs(action.timeout.0);
    match action.on {
        Runs::Server => servers
            .iter()
            .map(|s| {
                let values = Values { settings: settings.clone(), secrets: secrets.clone(), server: Some(ServerValues { name: s.name.clone(), host: s.host.clone(), user: s.login_user.clone(), port: s.port }), ..Default::default() };
                let nonce = run::nonce();
                let status = run::remote_program(action, &requires.server, dir, &values, &nonce).and_then(|program| {
                    let r = run::run_program(crate::host::host_for(s).as_ref(), &program, action.root, timeout, &nonce);
                    if r.ok { run::status(&r.stdout) } else { Err(r.error.unwrap_or_else(|| r.stderr.lines().last().unwrap_or_default().to_string())) }
                });
                (s.id.clone(), status)
            })
            .collect(),
        Runs::Local => {
            let fail = |e: String| servers.iter().map(|s| (s.id.clone(), Err(e.clone()))).collect::<Vec<_>>();
            if !local::missing_here(&requires.local).is_empty() {
                return fail("missing on this machine".into());
            }
            let nonce = run::nonce();
            let data_dir = local::data_dir(plugin_id);
            let prepared = local::run_dir(&data_dir, &nonce).and_then(|rd| {
                let (inventory, ssh_config) = local::write_targets(&rd, servers)?;
                let paths = local::LocalPaths { plugin_dir: dir.to_path_buf(), data_dir: data_dir.clone(), inventory, ssh_config, results: rd.join("results") };
                let values = Values { settings, secrets, local: Some(paths.clone()), ..Default::default() };
                let env = local::environment(&data_dir, &rd, &action.env, &values)?;
                let argv = local::argv(action.command.as_deref(), action.script.as_deref(), action.interpreter, &action.args, dir, &values)?;
                Ok((rd, paths, env, argv))
            });
            let (rd, paths, env, argv) = match prepared {
                Ok(p) => p,
                Err(e) => return fail(e),
            };
            let _ = local::run(&argv, &data_dir, &env, timeout, &mut |_, _| {}, &Default::default());
            let lines = local::read_results(&paths.results);
            let _ = std::fs::remove_dir_all(&rd);
            servers
                .iter()
                .map(|s| {
                    let alias = local::alias(s);
                    let st = lines.iter().find(|l| l["host"] == alias.as_str()).and_then(status_of_line).ok_or_else(|| "no status".to_string());
                    (s.id.clone(), st)
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{read_badges, status_of_line};
    use crate::plugin_host::manifest;
    use crate::plugin_host::run::State;
    use crate::vault::ServerRecord;

    #[test]
    fn results_lines_give_statuses() {
        let l: serde_json::Value = serde_json::from_str(r#"{"host":"web-1","state":"warn","label":"3 changes pending\u001b[2J"}"#).unwrap();
        let st = status_of_line(&l).unwrap();
        assert_eq!((st.state, st.label.as_str()), (State::Warn, "3 changes pending[2J"));
        assert!(status_of_line(&serde_json::json!({"host": "web-1"})).is_none());
    }

    /// Against the lab container; by hand: `cargo test badge_lab -- --ignored`.
    #[test]
    #[ignore]
    fn badge_lab() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugin_host/fixtures/server-facts");
        let m = manifest::parse(&std::fs::read_to_string(dir.join("plugin.toml")).unwrap()).unwrap();
        let action = m.actions.iter().find(|a| a.id == m.badge.as_ref().unwrap().action).unwrap();
        let srv = ServerRecord { id: "d1".into(), name: "crow-dr-1".into(), host: "crow-dr-1".into(), tags: vec!["test-node".into(), "podman".into()], ..Default::default() };
        let got = read_badges("server-facts", &dir, action, &m.requires, &[srv], BTreeMap::new(), BTreeMap::new());
        eprintln!("{got:?}");
        assert!(matches!(&got[0].1, Ok(s) if s.label.starts_with("fullest disk")));
    }
}
