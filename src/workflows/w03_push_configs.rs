//! W03: push configs. An sshd_config edit on a server behind the bastion,
//! saved through Crow's validated write; then one file rolled out to a
//! group (a direct server and the one behind the bastion), canary first,
//! and rolled back.

use std::time::Duration;

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, isolate, open_crow, root, server, settle, ssh, ssh_via, step, targets};
use crate::app::patching::Target;
use crate::app::rollouts::Kind;
use crate::views::fleet::run::RunPhase;

const FILE: &str = "/etc/crow-wf/app.conf";

#[gpui_kit::test]
fn w03_push_configs(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let (bastion_ip, app1_ip, app2_ip) = (t["BASTION"].clone(), t["APP1"].clone(), t["APP2"].clone());
    let _ = ssh(&app1_ip, "rm -rf /etc/crow-wf");
    let _ = ssh_via(&bastion_ip, &app2_ip, "rm -rf /etc/crow-wf; sed -i '/^ClientAliveInterval 120$/d' /etc/ssh/sshd_config");

    let (app, w) = open_crow(cx);
    settle(cx, w);
    let app2 = server(cx, &app, "wf-app2");

    step("wf-app2 → Config: its files are read through the bastion");
    act(cx, &app, w, |this, _, cx| {
        this.switch_tab(&app2.id, cx);
        this.set_view("config", cx);
    });
    let (loaded_for, has_sshd) = app.read_with(cx, |this, _| (this.configs.server_id.clone(), this.configs.states.contains_key("sshd_config")));
    assert_eq!(loaded_for.as_deref(), Some(app2.id.as_str()));
    assert!(has_sshd, "sshd_config is listed");

    step("sshd_config: add ClientAliveInterval 120, then save");
    act(cx, &app, w, |this, _, cx| {
        this.configs.selected_file = "sshd_config".into();
        this.insert_structured_row("sshd_config", Some(("ClientAliveInterval".into(), "120".into())), cx);
    });
    let pending = app.read_with(cx, |this, _| this.configs.states["sshd_config"].is_modified());
    assert!(pending, "the edit is pending");
    act(cx, &app, w, |this, _, cx| this.stage_config_version("sshd_config", "wf: keepalive", cx));
    let (err, modified, revisions) = app.read_with(cx, |this, _| {
        let st = &this.configs.states["sshd_config"];
        (this.configs.save_error.clone(), st.is_modified(), st.revisions.len())
    });
    assert_eq!(err, None);
    assert!(!modified, "saved: nothing pending");
    assert!(revisions >= 2, "the save is a revision ({revisions})");

    step("On app2: the line is there and sshd accepts the file");
    let (_, out) = ssh_via(&bastion_ip, &app2_ip, "grep -c '^ClientAliveInterval 120$' /etc/ssh/sshd_config; sshd -t && echo valid; sshd -T | grep -i '^clientaliveinterval'");
    assert!(out.contains("1\nvalid") && out.to_lowercase().contains("clientaliveinterval 120"), "{out}");

    step("The audit log has the write");
    let changes = app.read_with(cx, |this, _| this.vault.db().lock().unwrap().list_change_records(&app2.id, 20).unwrap());
    assert!(changes.iter().any(|c| c.action_kind == "config.write" && c.target.contains("sshd_config") && c.outcome == "applied"), "{changes:#?}");

    step("Fleet Setup → Rollouts: a config file to group app (app1 direct, app2 via the bastion)");
    // The fleet poll would have seen both connected.
    for name in ["wf-app1", "wf-app2"] {
        let srv = server(cx, &app, name);
        crow_exec(&srv, "true").unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    let content = "# rolled out by Crow (workflow W03)\nmode = canary-tested\n";
    act(cx, &app, w, |this, window, cx| {
        this.ensure_rollout_inputs(window, cx);
        this.set_rollout_kind(Kind::Config, cx);
        this.set_rollout_target(Target::Group("app".into()), cx);
        let inputs = this.rollout_inputs.as_ref().unwrap();
        inputs.path.update(cx, |i, cx| i.set_value(FILE.to_string(), window, cx));
        inputs.content.update(cx, |e, cx| e.set_value(content.to_string(), window, cx));
        this.read_rollout_preview(cx);
    });
    act(cx, &app, w, |this, window, cx| this.plan_rollout(window, cx));
    let (steps, excluded, msg) = app.read_with(cx, |this, _| {
        let r = this.fleet_runner.as_ref();
        (r.map(|r| r.run.steps.iter().map(|s| format!("{} · {}", s.server, s.what)).collect::<Vec<_>>()).unwrap_or_default(), r.map(|r| r.run.excluded.clone()).unwrap_or_default(), this.rollouts.message.clone())
    });
    assert_eq!(steps.len(), 2, "both app servers: {steps:?} excluded {excluded:?} {msg:?}");
    assert!(steps.iter().all(|s| s.contains(&format!("create {FILE}"))), "{steps:?}");

    step("Confirm with the keyword; the canary runs and the run pauses");
    act(cx, &app, w, |this, window, cx| {
        let r = this.fleet_runner.as_ref().unwrap();
        let keyword = r.run.keyword;
        r.input.update(cx, |i, cx| i.set_value(keyword.to_string(), window, cx));
        this.confirm_fleet_run(cx);
    });
    let (phase, summary) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, format!("{} {:#?}", r.run.summary(), r.run.steps))).unwrap());
    assert_eq!(phase, RunPhase::Paused, "paused after the canary: {summary}");

    step("Continue: the rest runs");
    act(cx, &app, w, |this, _, cx| this.continue_fleet_run(cx));
    cx.executor().advance_clock(Duration::from_millis(300));
    settle(cx, w);
    let (phase, summary) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, r.run.summary())).unwrap());
    assert_eq!(phase, RunPhase::Finished, "{summary}");
    let (_, a1) = ssh(&app1_ip, &format!("cat {FILE}"));
    let (_, a2) = ssh_via(&bastion_ip, &app2_ip, &format!("cat {FILE}"));
    assert_eq!((a1.as_str(), a2.as_str()), (content, content));

    step("ROLL BACK: both servers lose the file they didn't have");
    act(cx, &app, w, |this, window, cx| this.run_fleet_after(window, cx));
    act(cx, &app, w, |this, window, cx| {
        let r = this.fleet_runner.as_ref().unwrap();
        let keyword = r.run.keyword;
        r.input.update(cx, |i, cx| i.set_value(keyword.to_string(), window, cx));
        this.confirm_fleet_run(cx);
    });
    for _ in 0..3 {
        if app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| r.run.phase)) == Some(RunPhase::Paused) {
            act(cx, &app, w, |this, _, cx| this.continue_fleet_run(cx));
        }
        cx.executor().advance_clock(Duration::from_millis(300));
        settle(cx, w);
    }
    let (phase, summary) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, r.run.summary())).unwrap());
    assert_eq!(phase, RunPhase::Finished, "{summary}");
    let (gone1, _) = ssh(&app1_ip, &format!("test ! -e {FILE}"));
    let (gone2, _) = ssh_via(&bastion_ip, &app2_ip, &format!("test ! -e {FILE}"));
    assert!(gone1 && gone2, "rolled back");
}
