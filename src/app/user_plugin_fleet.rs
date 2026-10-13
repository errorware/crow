//! A user plugin's action across many servers (ERR-181), through the same
//! fleet runner rollouts use: targets, a canary that runs alone and pauses,
//! then batches, stopping at the first failure. A server action runs once
//! per server; a local action runs once per batch, with an inventory of
//! that batch, and each server's step reads its line of `{results}`.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use gpui_kit::*;

use super::fleet_run::RunJob;
use super::patching::Target;
use super::rollouts::candidates;
use super::user_plugins::PendingRun;
use super::CrowApp;
use crate::plugin_host::folder::{self, Trust};
use crate::plugin_host::local;
use crate::plugin_host::manifest::{Action, Effect, FieldKind, Manifest, Runs};
use crate::plugin_host::run::{self, ServerValues, Values};
use crate::vault::{ChangeRecord, PluginRun, ServerRecord, VaultDb};
use crate::views::fleet::run::{FleetRun, StepOutcome, StepResult};

/// Where a plugin's fleet actions go: the same choices as Rollouts.
#[derive(Debug, Clone)]
pub struct PluginFleetState {
    pub target: Target,
    pub use_canary: bool,
    /// `None`: the first server of the target.
    pub canary: Option<String>,
    pub batch: usize,
}

impl Default for PluginFleetState {
    fn default() -> Self {
        Self { target: Target::All, use_canary: true, canary: None, batch: 1 }
    }
}

/// What a run of `action` on one server comes to, recorded like a single
/// run: the run history, and the audit log.
fn record(db: &Mutex<VaultDb>, plugin: (&str, &str, &str), srv: &ServerRecord, r: &run::RunResult, summary: &str, detail: &str) {
    let (plugin_id, action_id, hash) = plugin;
    let Ok(db) = db.lock() else { return };
    let _ = db.insert_plugin_run(&PluginRun {
        plugin_id: plugin_id.into(),
        action_id: action_id.into(),
        server_id: srv.id.clone(),
        started_at: r.started_at,
        finished_at: r.finished_at,
        ok: r.ok,
        exit_code: r.exit_code,
        summary: summary.into(),
        output_bytes: r.output_bytes(),
        plugin_hash: hash.into(),
        dry_run: false,
        stopped: r.stopped,
        log: String::new(),
    });
    let _ = db.insert_change_record(&ChangeRecord {
        id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
        server_id: srv.id.clone(),
        server_name: srv.name.clone(),
        action_kind: format!("plugin.{plugin_id}.{action_id}"),
        target: plugin_id.into(),
        before_state: [summary, detail].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · "),
        after_state: r.error.clone(),
        blast_radius: Some("fleet run".into()),
        outcome: if r.ok { "success" } else { "failed" }.into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
    });
}

fn step_result(r: &run::RunResult, summary: String) -> StepResult {
    if r.ok {
        Ok(StepOutcome::Done(if summary.is_empty() { t!("app-user-plugin-fleet-done").to_string() } else { summary }))
    } else {
        Err(if summary.is_empty() { r.error.clone().unwrap_or_else(|| t!("app-user-plugin-fleet-failed").to_string()) } else { summary })
    }
}

impl CrowApp {
    pub fn open_plugin_fleet_page(&mut self, plugin: &str, page: &str, cx: &mut Context<Self>) {
        self.fleet.plugin_page = Some((plugin.to_string(), page.to_string()));
        self.set_setup_page(crate::views::fleet::state::SetupPage::Plugin, cx);
    }

    pub fn set_plugin_fleet_target(&mut self, target: Target, cx: &mut Context<Self>) {
        self.plugins.user.fleet.target = target;
        self.plugins.user.fleet.canary = None;
        cx.notify();
    }

    pub fn set_plugin_fleet_canary(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        match id {
            Some(id) => {
                self.plugins.user.fleet.use_canary = true;
                self.plugins.user.fleet.canary = Some(id);
            }
            None => self.plugins.user.fleet.use_canary = false,
        }
        cx.notify();
    }

    pub fn set_plugin_fleet_batch(&mut self, n: usize, cx: &mut Context<Self>) {
        self.plugins.user.fleet.batch = n.max(1);
        cx.notify();
    }

    /// The servers a fleet action reaches, canary first, and those left out.
    pub fn plugin_fleet_candidates(&self, on: Runs) -> (Vec<ServerRecord>, Vec<(String, String)>) {
        let (ok, mut out) = candidates(&self.fleet.servers, &self.plugins.user.fleet.target, |s| self.fleet.health(s).is_ok());
        let (mut ok, labs): (Vec<_>, Vec<_>) = ok.into_iter().partition(|s| on == Runs::Server || !s.tags.iter().any(|t| t == "test-node"));
        out.extend(labs.into_iter().map(|s| (s.name.clone(), t!("app-user-plugin-fleet-no-ssh").to_string())));
        let st = &self.plugins.user.fleet;
        if st.use_canary {
            let canary = st.canary.clone().or_else(|| ok.first().map(|s| s.id.clone()));
            if let Some(i) = canary.and_then(|c| ok.iter().position(|s| s.id == c)) {
                let c = ok.remove(i);
                ok.insert(0, c);
            }
        }
        (ok, out)
    }

    /// A fleet page's button: a read plans its run at once; a change or
    /// danger action opens the confirm panel, its dry run on the canary.
    pub fn press_fleet_plugin_action(&mut self, plugin_id: &str, action_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(action) = self.plugins.user.get(plugin_id).and_then(|p| p.report.manifest.as_ref()).and_then(|m| m.actions.iter().find(|a| a.id == action_id)).cloned() else { return };
        if action.effect == Effect::Read && action.inputs.is_empty() {
            self.plan_plugin_fleet_run(plugin_id, action_id, BTreeMap::new(), window, cx);
            return;
        }
        let (servers, _) = self.plugin_fleet_candidates(action.on);
        let Some(first) = servers.first() else {
            self.plugins.user.message = Some((false, t!("app-user-plugin-fleet-no-servers").to_string()));
            cx.notify();
            return;
        };
        let picked = action.inputs.iter().filter(|f| !matches!(f.kind, FieldKind::Text | FieldKind::Number)).filter_map(|f| run::default_value(f).map(|d| (f.key.clone(), d))).collect();
        self.plugins.user.pending = Some(PendingRun { key: (plugin_id.to_string(), action_id.to_string(), first.id.clone()), picked, dry_ran_with: None, error: None, fleet: true });
        self.plugins.user.pending_inputs.clear();
        self.plugins.user.keyword_input = None;
        cx.notify();
    }

    /// Plans `action` across the fleet targets; the fleet runner shows the
    /// plan and waits for its keyword.
    pub fn plan_plugin_fleet_run(&mut self, plugin_id: &str, action_id: &str, inputs: BTreeMap<String, String>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(shown) = self.plugins.user.get(plugin_id).cloned() else { return };
        let now = folder::load(&shown.dir, &self.user_plugin_pin(plugin_id));
        let Some(m) = now.report.manifest.clone().filter(|_| now.trust == Trust::Accepted) else {
            self.plugins.user.message = Some((false, t!("app-user-plugins-not-accepted-now").to_string()));
            cx.notify();
            return;
        };
        let Some(action) = m.actions.iter().find(|a| a.id == action_id).cloned() else { return };
        if action.on == Runs::Local && m.setup.is_some() && self.user_plugin_setup_hash(plugin_id).as_deref() != Some(now.hash.as_str()) {
            self.plugins.user.message = Some((false, t!("app-user-plugins-needs-install").to_string()));
            cx.notify();
            return;
        }
        let (settings, secrets, missing) = run::settings_values(&m.settings, &BTreeMap::new());
        if let Some((_, why)) = missing.first() {
            self.plugins.user.message = Some((false, t!("app-user-plugins-needs-settings", names = why)));
            cx.notify();
            return;
        }
        let (servers, excluded) = self.plugin_fleet_candidates(action.on);
        let st = self.plugins.user.fleet.clone();
        let stages = crate::rollout::stages(servers.len(), st.use_canary && servers.len() > 1, st.batch);
        let db = self.vault.db();
        let detail = inputs.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ");
        let hash = now.hash.clone();
        let mut steps = Vec::new();
        let mut jobs: Vec<RunJob> = Vec::new();
        // A local action: one invocation per stage, shared by its steps.
        let mut batches: HashMap<usize, Arc<OnceLock<HashMap<String, StepResult>>>> = HashMap::new();
        for (i, srv) in servers.iter().enumerate() {
            steps.push(FleetRun::step(&srv.id, &srv.name, action.label.clone()));
            let ctx = JobCtx { plugin_id: plugin_id.to_string(), action: action.clone(), manifest: m.clone(), dir: now.dir.clone(), hash: hash.clone(), inputs: inputs.clone(), settings: settings.clone(), secrets: secrets.clone(), db: Arc::clone(&db), detail: detail.clone() };
            match action.on {
                Runs::Server => {
                    let srv = srv.clone();
                    jobs.push(Box::new(move || ctx.run_on(&srv)));
                }
                Runs::Local => {
                    let stage = stages.get(i).copied().unwrap_or(i);
                    let batch = Arc::clone(batches.entry(stage).or_default());
                    let members: Vec<ServerRecord> = servers.iter().enumerate().filter(|(j, _)| stages.get(*j).copied().unwrap_or(*j) == stage).map(|(_, s)| s.clone()).collect();
                    let srv = srv.clone();
                    jobs.push(Box::new(move || {
                        let results = batch.get_or_init(|| ctx.run_batch(&members));
                        results.get(&srv.id).cloned().unwrap_or_else(|| Err(t!("app-user-plugin-fleet-no-result").to_string()))
                    }));
                }
            }
        }
        let keyword: &'static str = match action.effect {
            Effect::Danger => Box::leak(action.keyword.clone().unwrap_or_else(|| "CONFIRM".into()).into_boxed_str()),
            _ => "RUN",
        };
        let name = now.name().to_string();
        let run = FleetRun::new(t!("app-user-plugin-fleet-title", label = action.label, plugin = name), keyword, steps, excluded)
            .english(t_beside!("app-user-plugin-fleet-title", label = action.label, plugin = name))
            .staged(stages, (st.use_canary && servers.len() > 1).then_some(0));
        self.open_fleet_run(run, jobs, window, cx);
    }
}

/// What a fleet step needs, moved onto its background thread.
#[derive(Clone)]
struct JobCtx {
    plugin_id: String,
    action: Action,
    manifest: Manifest,
    dir: std::path::PathBuf,
    hash: String,
    inputs: BTreeMap<String, String>,
    settings: BTreeMap<String, String>,
    secrets: BTreeMap<String, String>,
    db: Arc<Mutex<VaultDb>>,
    detail: String,
}

impl JobCtx {
    /// The plugin's files must still be the ones planned with.
    fn unchanged(&self) -> Result<(), String> {
        let r = crate::plugin_host::check::check_dir(&self.dir);
        if folder::hash(&r) == self.hash && r.ok() { Ok(()) } else { Err(t!("app-user-plugins-not-accepted-now").to_string()) }
    }

    fn run_on(&self, srv: &ServerRecord) -> StepResult {
        self.unchanged()?;
        let values = Values { inputs: self.inputs.clone(), settings: self.settings.clone(), secrets: self.secrets.clone(), server: Some(ServerValues { name: srv.name.clone(), host: srv.host.clone(), user: srv.login_user.clone(), port: srv.port }), local: None };
        let nonce = run::nonce();
        let program = run::remote_program(&self.action, &self.manifest.requires.server, &self.dir, &values, &nonce)?;
        let host = crate::host::host_for(srv);
        let r = run::run_program(host.as_ref(), &program, self.action.root, std::time::Duration::from_secs(self.action.timeout.0), &nonce);
        let summary = run::summary(&self.action.output, &r);
        record(&self.db, (&self.plugin_id, &self.action.id, &self.hash), srv, &r, &summary, &self.detail);
        step_result(&r, summary)
    }

    /// One local run for a whole batch: each server's result from its line
    /// of `{results}`, else the run's own outcome.
    fn run_batch(&self, members: &[ServerRecord]) -> HashMap<String, StepResult> {
        let all = |res: StepResult| members.iter().map(|s| (s.id.clone(), res.clone())).collect::<HashMap<_, _>>();
        if let Err(e) = self.unchanged() {
            return all(Err(e));
        }
        let missing = local::missing_here(&self.manifest.requires.local);
        if !missing.is_empty() {
            return all(Err(t!("app-user-plugins-missing-here", names = missing.join(", "))));
        }
        let nonce = run::nonce();
        let data_dir = local::data_dir(&self.plugin_id);
        let prepared = local::run_dir(&data_dir, &nonce).and_then(|rd| {
            let (inventory, ssh_config) = local::write_targets(&rd, members)?;
            let paths = local::LocalPaths { plugin_dir: self.dir.clone(), data_dir: data_dir.clone(), inventory, ssh_config, results: rd.join("results") };
            let values = Values { inputs: self.inputs.clone(), settings: self.settings.clone(), secrets: self.secrets.clone(), server: None, local: Some(paths.clone()) };
            let env = local::environment(&data_dir, &rd, &self.action.env, &values)?;
            let argv = local::argv(self.action.command.as_deref(), self.action.script.as_deref(), self.action.interpreter, &self.action.args, &self.dir, &values)?;
            Ok((rd, paths, env, argv))
        });
        let (rd, paths, env, argv) = match prepared {
            Ok(p) => p,
            Err(e) => return all(Err(e)),
        };
        let r = local::run(&argv, &data_dir, &env, std::time::Duration::from_secs(self.action.timeout.0), &mut |_, _| {}, &Default::default());
        let lines = local::read_results(&paths.results);
        let _ = std::fs::remove_dir_all(&rd);
        let overall = run::summary(&self.action.output, &r);
        members
            .iter()
            .map(|s| {
                let alias = local::alias(s);
                let mine = lines.iter().find(|l| l["host"] == alias.as_str());
                let (ok, summary) = match mine {
                    Some(l) => (l.get("ok").and_then(|o| o.as_bool()).unwrap_or(r.ok), l.get("summary").and_then(|v| v.as_str()).unwrap_or(&overall).to_string()),
                    None => (r.ok, overall.clone()),
                };
                let mut own = r.clone();
                own.ok = ok;
                record(&self.db, (&self.plugin_id, &self.action.id, &self.hash), s, &own, &summary, &self.detail);
                (s.id.clone(), step_result(&own, summary))
            })
            .collect()
    }
}

#[cfg(test)]
mod lab {
    //! Against throwaway lab containers; run by hand with
    //! `CROW_PLUGIN_LAB=<scratch dir> cargo test plugin_fleet_lab -- --ignored`.
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use super::JobCtx;
    use crate::plugin_host::manifest;
    use crate::vault::{ServerRecord, SshKeyRecord, VaultDb};

    fn server(id: &str, name: &str, host: &str, port: u16, jump: Option<&str>, tags: &[&str]) -> ServerRecord {
        ServerRecord { id: id.into(), name: name.into(), host: host.into(), port, login_user: "root".into(), auth_method: if tags.is_empty() { "publickey".into() } else { "agent".into() }, key_id: tags.is_empty().then(|| "hk".to_string()), jump_host_id: jump.map(str::to_string), tags: tags.iter().map(|t| t.to_string()).collect(), ..Default::default() }
    }

    #[test]
    #[ignore]
    fn plugin_fleet_lab() {
        let lab = std::path::PathBuf::from(std::env::var("CROW_PLUGIN_LAB").expect("CROW_PLUGIN_LAB"));
        // The scratch ssh wrapper keeps the test's known_hosts.
        std::env::set_var("PATH", format!("{}:{}", lab.join("bin").display(), std::env::var("PATH").unwrap()));
        let servers = vec![
            server("bast", "crow-bastion", "127.0.0.1", 2224, None, &[]),
            server("d1", "crow-dr-1", "crow-dr-1", 22, None, &["test-node", "podman"]),
            server("ssh1", "crow-ssh-1", "127.0.0.1", 2223, None, &[]),
            server("ssh2", "crow-ssh-2", "10.89.0.2", 22, Some("bast"), &[]),
        ];
        let key = SshKeyRecord { id: "hk".into(), private_key_path: Some(lab.join("harden-lab/key").display().to_string()), ..Default::default() };
        crate::host::update_directory(&servers, &[key]);
        let db = Arc::new(Mutex::new(VaultDb::open_in_memory().unwrap()));
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugin_host/fixtures");

        // A server action: the write fails where /etc/motd.d is a file.
        let dir = fixtures.join("motd-note");
        let m = manifest::parse(&std::fs::read_to_string(dir.join("plugin.toml")).unwrap()).unwrap();
        let hash = crate::plugin_host::folder::hash(&crate::plugin_host::check::check_dir(&dir));
        let ctx = |action: &str| JobCtx {
            plugin_id: "motd-note".into(),
            action: m.actions.iter().find(|a| a.id == action).unwrap().clone(),
            manifest: m.clone(),
            dir: dir.clone(),
            hash: hash.clone(),
            inputs: BTreeMap::from([("note".to_string(), "Patch night Friday".to_string())]),
            settings: BTreeMap::new(),
            secrets: BTreeMap::new(),
            db: Arc::clone(&db),
            detail: "note=Patch night Friday".into(),
        };
        let bad = ctx("write").run_on(&servers[1]);
        eprintln!("crow-dr-1: {bad:?}");
        assert!(bad.is_err());
        let good = ctx("write").run_on(&servers[2]);
        eprintln!("crow-ssh-1: {good:?}");
        assert!(good.is_ok());
        let runs = db.lock().unwrap().list_plugin_runs("motd-note", None, 10).unwrap();
        assert_eq!(runs.len(), 2);
        assert!(runs.iter().any(|r| r.server_id == "ssh1" && r.ok) && runs.iter().any(|r| r.server_id == "d1" && !r.ok));

        // A local action, once for a batch of two (one behind the bastion).
        let dir = fixtures.join("local-reach");
        let m = manifest::parse(&std::fs::read_to_string(dir.join("plugin.toml")).unwrap()).unwrap();
        let data = crate::plugin_host::local::data_dir("local-reach");
        if !data.join("venv/bin/python").exists() {
            eprintln!("local-reach isn't installed under {}; skipping its batch", data.display());
            return;
        }
        let ctx = JobCtx { plugin_id: "local-reach".into(), action: m.actions[0].clone(), manifest: m.clone(), dir: dir.clone(), hash: crate::plugin_host::folder::hash(&crate::plugin_host::check::check_dir(&dir)), inputs: BTreeMap::new(), settings: BTreeMap::new(), secrets: BTreeMap::new(), db: Arc::clone(&db), detail: String::new() };
        let results = ctx.run_batch(&[servers[2].clone(), servers[3].clone()]);
        eprintln!("batch: {results:?}");
        assert!(results["ssh1"].is_ok() && results["ssh2"].is_ok());
    }
}
