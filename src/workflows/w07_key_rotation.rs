//! W07: rotate the SSH key two servers log in with. Crow makes a new key,
//! installs it, proves a fresh login with it, switches each server over
//! and removes the old key from it.

use std::time::Duration;

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, ensure_outside_key, isolate, open_crow, root, server, settle, ssh, step, targets};
use crate::views::fleet::run::RunPhase;

#[gpui_kit::test]
fn w07_rotate_a_key(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    ensure_outside_key(&[&t["BASTION"], &t["APP1"], &t["APP2"]]);
    let (app, w) = open_crow(cx);
    settle(cx, w);

    let bastion = server(cx, &app, "wf-bastion");
    let app1 = server(cx, &app, "wf-app1");
    let old_id = bastion.key_id.clone().expect("key login");
    assert_eq!(app1.key_id.as_ref(), Some(&old_id), "both use the key W01 imported (or the last rotation's)");
    let old = app.read_with(cx, |this, _| this.keys.enrolled.iter().find(|k| k.id == old_id).cloned()).unwrap();
    let old_blob = crate::keys::deploy::key_blob(&old.public_key).unwrap();
    for srv in [&bastion, &app1] {
        crow_exec(srv, "true").unwrap();
    }

    step(&format!("Settings → Keys → {} → ROTATE", old.name));
    act(cx, &app, w, |this, window, cx| this.plan_key_rotation(&old_id, window, cx));
    let (steps, title) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.steps.iter().map(|s| s.server.clone()).collect::<Vec<_>>(), r.run.title.clone())).unwrap());
    step(&format!("  {title}: {steps:?}"));
    let mut expected = vec!["wf-bastion".to_string(), "wf-app1".to_string()];
    expected.sort();
    let mut got = steps.clone();
    got.sort();
    assert_eq!(got, expected);
    let new = app.read_with(cx, |this, _| this.keys.enrolled.iter().find(|k| k.id != old_id && k.name.starts_with(&old.name)).cloned()).expect("the new key is enrolled");
    let new_path = crate::keys::expand_tilde(new.private_key_path.as_deref().unwrap());
    assert!(new_path.exists(), "{}", new_path.display());

    step("Type SWITCH; every server switches");
    act(cx, &app, w, |this, window, cx| {
        let r = this.fleet_runner.as_ref().unwrap();
        r.input.update(cx, |i, cx| i.set_value("SWITCH".to_string(), window, cx));
        this.confirm_fleet_run(cx);
    });
    cx.executor().advance_clock(Duration::from_millis(300));
    settle(cx, w);
    let (phase, summary, detail) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, r.run.summary(), format!("{:#?}", r.run.steps))).unwrap());
    assert_eq!(phase, RunPhase::Finished);
    assert!(!summary.contains("stopped"), "{summary} {detail}");
    act(cx, &app, w, |this, _, cx| this.close_fleet_run(cx));

    step("Both servers log in with the new key; the old one is gone from them");
    let new_blob = crate::keys::deploy::key_blob(&new.public_key).unwrap();
    for name in ["wf-bastion", "wf-app1"] {
        let srv = server(cx, &app, name);
        assert_eq!(srv.key_id.as_ref(), Some(&new.id), "{name} switched");
        assert_eq!(crow_exec(&srv, "hostname").map(|s| s.trim().to_string()), Ok(name.to_string()));
        let (_, keys) = ssh(&srv.host, "cat /root/.ssh/authorized_keys");
        assert!(keys.contains(&new_blob), "{name} has the new key");
        assert!(!keys.contains(&old_blob), "{name} lost the old key");
        let out = std::process::Command::new("/usr/bin/ssh")
            .args(["-F", "/dev/null", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null", "-o", "IdentitiesOnly=yes", "-i"])
            .arg(crate::keys::expand_tilde(old.private_key_path.as_deref().unwrap()))
            .arg(format!("root@{}", srv.host))
            .arg("true")
            .output()
            .unwrap();
        assert!(!out.status.success(), "{name} refuses the old key");
    }

    step("wf-app2 (another key) is untouched");
    let app2 = server(cx, &app, "wf-app2");
    assert_ne!(app2.key_id.as_ref(), Some(&new.id));
    assert!(crow_exec(&app2, "true").is_ok());

    step("The audit log has a switch per server");
    for srv in [&bastion, &app1] {
        let changes = app.read_with(cx, |this, _| this.vault.db().lock().unwrap().list_change_records(&srv.id, 10).unwrap());
        assert!(changes.iter().any(|c| c.action_kind == "ssh.key_switch"), "{}: {changes:#?}", srv.name);
    }
}
