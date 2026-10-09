//! Fleet Setup → DR PLANS (ERR-151): write a plan, check it against the
//! servers, run drills, export it.

use gpui_kit::component::input::{EditorState, InputState};
use gpui_kit::*;

use super::CrowApp;
use crate::dr::{self, Drill, Finding, Plan};
use crate::host::host_for;

pub struct DrInputs {
    pub name: Entity<InputState>,
    pub rto: Entity<InputState>,
    pub rpo: Entity<InputState>,
    pub notes: Entity<InputState>,
    pub matters: Entity<EditorState>,
    pub backups: Entity<EditorState>,
    pub runbook: Entity<EditorState>,
}

#[derive(Default)]
pub struct DrState {
    pub plans: Vec<Plan>,
    pub selected: Option<String>,
    pub target: String,
    pub findings: Vec<(String, Vec<Finding>)>,
    pub checking: usize,
    pub message: Option<(bool, String)>,
}

impl CrowApp {
    pub fn load_dr(&mut self) {
        self.dr.plans = self.vault.db().lock().ok().and_then(|db| db.flag(dr::PLANS_FLAG)).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
    }

    fn save_dr(&self) {
        if let (Ok(db), Ok(json)) = (self.vault.db().lock(), serde_json::to_string(&self.dr.plans)) {
            let _ = db.set_flag(dr::PLANS_FLAG, &json);
        }
    }

    pub fn ensure_dr_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dr_inputs.is_some() {
            return;
        }
        let input = |p: &str, window: &mut Window, cx: &mut Context<Self>| {
            let p = p.to_string();
            cx.new(|cx| InputState::new(window, cx).placeholder(p))
        };
        let name = input("plan name, e.g. orders database", window, cx);
        let rto = input("RTO, minutes", window, cx);
        let rpo = input("RPO, minutes", window, cx);
        let notes = input("drill notes, e.g. restored to a Multipass VM from last night's dump", window, cx);
        let matters = cx.new(|cx| EditorState::new(window, cx).placeholder("what matters, one per line:\n/var/lib/postgresql\n/etc/nginx\norders database"));
        let backups = cx.new(|cx| EditorState::new(window, cx).placeholder("backups, one per line (path or glob | timer):\n/var/backups/db/*.sql.gz | pg-dump.timer"));
        let runbook = cx.new(|cx| EditorState::new(window, cx).placeholder("## Rebuild\n1. Launch a fresh server (Add Server)\n2. Restore: zcat /var/backups/db/latest.sql.gz | psql orders\n3. Push the baselines (Drift → bring back)\n4. Check the site (Checks)"));
        self.dr_inputs = Some(DrInputs { name, rto, rpo, notes, matters, backups, runbook });
    }

    pub fn new_dr_plan(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dr.selected = None;
        self.dr.findings.clear();
        self.dr.target = self.fleet.servers.first().map(|s| s.id.clone()).unwrap_or_default();
        self.fill_dr_inputs(&Plan { rto_minutes: 60, rpo_minutes: 1440, ..Default::default() }, window, cx);
    }

    pub fn select_dr_plan(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(p) = self.dr.plans.iter().find(|p| p.id == id).cloned() else { return };
        self.dr.selected = Some(id.to_string());
        self.dr.target = p.target.clone();
        self.dr.findings.clear();
        self.fill_dr_inputs(&p, window, cx);
    }

    fn fill_dr_inputs(&mut self, p: &Plan, window: &mut Window, cx: &mut Context<Self>) {
        let Some(i) = &self.dr_inputs else { return };
        i.name.update(cx, |x, cx| x.set_value(p.name.clone(), window, cx));
        i.rto.update(cx, |x, cx| x.set_value(if p.rto_minutes > 0 { p.rto_minutes.to_string() } else { String::new() }, window, cx));
        i.rpo.update(cx, |x, cx| x.set_value(if p.rpo_minutes > 0 { p.rpo_minutes.to_string() } else { String::new() }, window, cx));
        i.matters.update(cx, |x, cx| x.set_value(p.matters.clone(), window, cx));
        i.backups.update(cx, |x, cx| x.set_value(dr::format_backups(&p.backups), window, cx));
        i.runbook.update(cx, |x, cx| x.set_value(p.runbook.clone(), window, cx));
        cx.notify();
    }

    pub fn set_dr_target(&mut self, target: String, cx: &mut Context<Self>) {
        self.dr.target = target;
        cx.notify();
    }

    /// Saves the form as the selected plan (or a new one).
    pub fn save_dr_plan(&mut self, cx: &mut Context<Self>) {
        let Some(i) = &self.dr_inputs else { return };
        let name = i.name.read(cx).value().trim().to_string();
        if name.is_empty() || self.dr.target.is_empty() {
            self.dr.message = Some((false, "Give the plan a name and pick what it covers.".into()));
            cx.notify();
            return;
        }
        let num = |e: &Entity<InputState>| e.read(cx).value().trim().parse::<i64>().unwrap_or(0);
        let (rto, rpo) = (num(&i.rto), num(&i.rpo));
        let fields = (i.matters.read(cx).value().to_string(), dr::parse_backups(&i.backups.read(cx).value()), i.runbook.read(cx).value().to_string());
        let id = self.dr.selected.clone().unwrap_or_else(|| format!("dr{}", chrono::Utc::now().timestamp_millis()));
        let existing = self.dr.plans.iter().find(|p| p.id == id).cloned().unwrap_or_default();
        let plan = Plan { id: id.clone(), name, target: self.dr.target.clone(), matters: fields.0, backups: fields.1, rto_minutes: rto, rpo_minutes: rpo, runbook: fields.2, ..existing };
        self.dr.plans.retain(|p| p.id != id);
        self.dr.plans.push(plan);
        self.dr.selected = Some(id);
        self.save_dr();
        self.dr.message = Some((true, "Saved.".into()));
        cx.notify();
    }

    pub fn delete_dr_plan(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.dr.selected.take() {
            self.dr.plans.retain(|p| p.id != id);
            self.save_dr();
            self.dr.findings.clear();
        }
        cx.notify();
    }

    fn dr_servers(&self, target: &str) -> Vec<crate::vault::ServerRecord> {
        match target.strip_prefix("group:") {
            Some(g) => self.fleet.servers.iter().filter(|s| s.group_name.trim() == g).cloned().collect(),
            None => self.fleet.servers.iter().filter(|s| s.id == target).cloned().collect(),
        }
    }

    /// Checks the selected plan on each of its servers.
    pub fn check_dr_plan(&mut self, cx: &mut Context<Self>) {
        let Some(plan) = self.dr.selected.as_ref().and_then(|id| self.dr.plans.iter().find(|p| &p.id == id)).cloned() else {
            self.dr.message = Some((false, "Save the plan first.".into()));
            cx.notify();
            return;
        };
        let baselines = self.vault.db().lock().ok().and_then(|db| db.list_config_baselines().ok()).unwrap_or_default();
        self.dr.findings.clear();
        let servers = self.dr_servers(&plan.target);
        self.dr.checking = servers.len();
        for srv in servers {
            let count = crate::config::drift::paths_for(&baselines, &srv.group_name).len();
            let snapshots = self.providers.accounts.iter().find(|a| a.id == srv.provider_account && !srv.provider_instance.is_empty()).map(|a| a.label.clone());
            let (plan, name) = (plan.clone(), srv.name.clone());
            let reachable = self.fleet.health(&srv).is_ok();
            cx.spawn(async move |entity, cx| {
                let found = if reachable {
                    cx.background_executor().spawn(async move { dr::check(host_for(&srv).as_ref(), &plan, count, snapshots.as_deref(), chrono::Utc::now().timestamp()) }).await
                } else {
                    vec![Finding { ok: false, what: "not reachable right now: nothing could be checked".into() }]
                };
                let _ = entity.update(cx, |this, cx| {
                    this.dr.checking = this.dr.checking.saturating_sub(1);
                    this.dr.findings.push((name, found));
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub fn start_drill(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.dr.selected.clone() else { return };
        if let Some(p) = self.dr.plans.iter_mut().find(|p| p.id == id) {
            p.drill_started = Some(chrono::Utc::now().timestamp());
        }
        self.save_dr();
        self.dr.message = Some((true, "Drill started: follow the runbook (restore to a Multipass VM or a scratch server), then FINISH DRILL.".into()));
        cx.notify();
    }

    pub fn finish_drill(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.dr.selected.clone() else { return };
        let notes = self.dr_inputs.as_ref().map(|i| i.notes.read(cx).value().trim().to_string()).unwrap_or_default();
        let now = chrono::Utc::now().timestamp();
        let mut msg = None;
        if let Some(p) = self.dr.plans.iter_mut().find(|p| p.id == id) {
            if let Some(start) = p.drill_started.take() {
                let minutes = ((now - start) as f64 / 60.0).ceil() as i64;
                let met = p.rto_minutes == 0 || minutes <= p.rto_minutes;
                msg = Some((met, format!("Drill recorded: {minutes} min, {} the RTO of {} min.", if met { "within" } else { "over" }, p.rto_minutes)));
                p.drills.push(Drill { started_at: start, minutes, met_rto: met, notes });
            }
        }
        self.save_dr();
        self.dr.message = msg;
        cx.notify();
    }

    pub fn export_dr_plan(&mut self, cx: &mut Context<Self>) {
        let Some(plan) = self.dr.selected.as_ref().and_then(|id| self.dr.plans.iter().find(|p| &p.id == id)).cloned() else { return };
        let target = match plan.target.strip_prefix("group:") {
            Some(g) => format!("group {g}"),
            None => self.fleet.servers.iter().find(|s| s.id == plan.target).map(|s| s.name.clone()).unwrap_or_else(|| plan.target.clone()),
        };
        let now = chrono::Local::now();
        let md = dr::markdown(&plan, &target, &self.dr.findings, &now.format("%Y-%m-%d %H:%M").to_string());
        let dir = dirs::download_dir().or_else(dirs::document_dir).or_else(dirs::home_dir).unwrap_or_else(std::env::temp_dir);
        let safe: String = plan.name.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
        let path = dir.join(format!("crow-dr-{safe}.md"));
        self.dr.message = Some(match std::fs::write(&path, md) {
            Ok(()) => (true, format!("Plan written to {}: keep a copy where you can read it when Crow is down.", path.display())),
            Err(e) => (false, format!("Couldn't write {}: {e}", path.display())),
        });
        cx.notify();
    }
}
