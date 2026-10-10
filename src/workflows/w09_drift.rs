//! W09: config drift. Pin a server's /etc/hosts as the baseline, change it
//! behind Crow's back, let Crow notice; bringing it back needs the content
//! (left out, saying why, when only a hash was kept); accepting the
//! server's copy clears it.

use std::time::Duration;

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, isolate, open_crow, root, server, settle, ssh, step, targets};

#[gpui_kit::test]
fn w09_drift(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let _ = ssh(&t["APP1"], "sed -i '/crow-wf-drift/d' /etc/hosts");
    let (app, w) = open_crow(cx);
    settle(cx, w);
    let app1 = server(cx, &app, "wf-app1");
    crow_exec(&app1, "true").unwrap();

    step("wf-app1 → Config → hosts: pin the current version as the baseline");
    act(cx, &app, w, |this, _, cx| {
        this.switch_tab(&app1.id, cx);
        this.set_view("config", cx);
    });
    let (version, scope) = app.read_with(cx, |this, _| {
        let st = &this.configs.states["hosts"];
        (st.revisions.iter().filter(|r| !r.id.is_empty()).map(|r| r.version).max(), this.configs.baseline_scope.clone())
    });
    let version = version.expect("a recorded revision of /etc/hosts");
    act(cx, &app, w, |this, _, cx| {
        this.configs.selected_file = "hosts".into();
        this.pin_config_baseline("hosts", version, cx);
    });
    let err = app.read_with(cx, |this, _| this.configs.history_error.clone());
    assert_eq!(err, None);
    step(&format!("  pinned v{version} for scope {scope:?}"));

    step("Outside Crow: someone edits /etc/hosts");
    assert!(ssh(&t["APP1"], "echo '10.9.9.9 crow-wf-drift' >> /etc/hosts").0);

    step("Drift check: Crow reads the file again and flags it");
    act(cx, &app, w, |this, _, cx| this.check_drift(true, cx));
    let drifted = app.read_with(cx, |this, _| this.fleet.drift.iter().flatten().any(|e| e.server_id == app1.id && e.path == "/etc/hosts" && e.drift.is_drift()));
    assert!(drifted, "{:?}", app.read_with(cx, |this, _| this.fleet.drift.clone()));
    let alert = app.read_with(cx, |this, _| this.vault.db().lock().unwrap().list_alerts(i64::MAX).unwrap().into_iter().any(|a| a.server_id == app1.id && a.resolved_at.is_none() && a.detail.to_lowercase().contains("hosts")));
    assert!(alert, "a drift alert is open");

    // gpui's test platform has a stand-in keyring: Crow makes a data key per
    // run, so content kept by an earlier run can't be opened (hash only).
    step("BRING BACK: planned when the baseline's content was kept, else left out saying why");
    act(cx, &app, w, |this, window, cx| this.plan_bring_back("/etc/hosts", Some(&app1.id), window, cx));
    let (steps, excluded) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.steps.len(), r.run.excluded.clone())).unwrap());
    step(&format!("  planned {steps}; left out {excluded:?}"));
    assert!(steps == 1 || excluded.iter().any(|(n, why)| n == "wf-app1" && why.contains("only its hash")), "planned, or left out saying why: {excluded:?}");
    act(cx, &app, w, |this, _, cx| this.close_fleet_run(cx));

    step("ACCEPT the server's copy: the drift and its alert clear");
    act(cx, &app, w, |this, _, cx| this.accept_drift(&app1.id, "/etc/hosts", cx));
    act(cx, &app, w, |this, _, cx| this.check_drift(true, cx));
    cx.executor().advance_clock(Duration::from_millis(100));
    settle(cx, w);
    let still = app.read_with(cx, |this, _| this.fleet.drift.iter().flatten().any(|e| e.server_id == app1.id && e.path == "/etc/hosts" && e.drift.is_drift()));
    assert!(!still, "accepted");
    let open = app.read_with(cx, |this, _| this.vault.db().lock().unwrap().list_alerts(i64::MAX).unwrap().into_iter().any(|a| a.server_id == app1.id && a.resolved_at.is_none() && a.detail.to_lowercase().contains("hosts")));
    assert!(!open, "the alert resolved");

    step("Cleanup: the line goes, the baseline is unpinned");
    let _ = ssh(&t["APP1"], "sed -i '/crow-wf-drift/d' /etc/hosts");
    act(cx, &app, w, |this, _, cx| this.unpin_config_baseline("/etc/hosts", &scope, cx));
}
