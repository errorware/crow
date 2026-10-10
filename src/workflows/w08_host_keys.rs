//! W08: Danger Zone → rotate SSH host keys. New keys are made on each
//! server, read over the connection the old key authenticated, re-pinned
//! in known_hosts and proven with a fresh strict login.

use std::time::Duration;

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, isolate, open_crow, root, server, settle, ssh, step};
use crate::views::fleet::run::RunPhase;

#[gpui_kit::test]
fn w08_rotate_host_keys(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    let profile = isolate();
    let (app, w) = open_crow(cx);
    settle(cx, w);
    let names = ["wf-bastion", "wf-app1", "wf-app2"];
    let before: Vec<(String, Option<String>, String)> = names
        .iter()
        .map(|n| {
            let s = server(cx, &app, n);
            let (_, keys) = ssh(&s.host, "cat /etc/ssh/ssh_host_ed25519_key.pub");
            (s.host.clone(), s.host_key_fingerprint.clone(), keys)
        })
        .collect();
    for n in names {
        crow_exec(&server(cx, &app, n), "true").unwrap();
    }
    let kh_before = std::fs::read_to_string(profile.join("home/.ssh/known_hosts")).unwrap();

    step("Danger Zone → ROTATE SSH HOST KEYS → type ROTATE");
    act(cx, &app, w, |this, window, cx| this.plan_rotate_host_keys(window, cx));
    let planned: Vec<String> = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| r.run.steps.iter().map(|s| s.server.clone()).collect()).unwrap());
    assert_eq!(planned.len(), 3, "{planned:?}");
    act(cx, &app, w, |this, window, cx| {
        let r = this.fleet_runner.as_ref().unwrap();
        r.input.update(cx, |i, cx| i.set_value("ROTATE".to_string(), window, cx));
        this.confirm_fleet_run(cx);
    });
    cx.executor().advance_clock(Duration::from_millis(300));
    settle(cx, w);
    let (phase, summary, detail) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, r.run.summary(), format!("{:#?}", r.run.steps))).unwrap());
    assert_eq!(phase, RunPhase::Finished);
    assert!(!summary.contains("stopped"), "{summary} {detail}");
    act(cx, &app, w, |this, _, cx| this.close_fleet_run(cx));

    step("Each server serves new keys, Crow pinned them, and logs in strictly");
    let kh_after = std::fs::read_to_string(profile.join("home/.ssh/known_hosts")).unwrap();
    for (n, (host, old_fp, old_key)) in names.iter().zip(&before) {
        let srv = server(cx, &app, n);
        let (_, new_key) = ssh(host, "cat /etc/ssh/ssh_host_ed25519_key.pub");
        assert_ne!(new_key.trim(), old_key.trim(), "{n}: a new key");
        assert_ne!(&srv.host_key_fingerprint, old_fp, "{n}: the record's pin moved");
        let blob = new_key.split_whitespace().nth(1).unwrap();
        assert!(kh_after.lines().any(|l| l.starts_with(host.as_str()) && l.contains(blob)), "{n}: known_hosts holds the new key");
        let old_blob = old_key.split_whitespace().nth(1).unwrap();
        assert!(!kh_after.lines().any(|l| l.starts_with(&format!("{host} ")) && l.contains(old_blob)), "{n}: the old pin is gone");
        assert_eq!(crow_exec(&srv, "hostname").map(|s| s.trim().to_string()), Ok(n.to_string()), "{n}: a strict login works");
    }
    assert_ne!(kh_before, kh_after);
    let real = std::fs::read_to_string(dirs_real_known_hosts()).unwrap_or_default();
    for (host, ..) in &before {
        assert!(!real.contains(host.as_str()), "the user's own known_hosts is untouched ({host})");
    }
}

/// The passwd home's known_hosts (what ssh reads outside the test).
fn dirs_real_known_hosts() -> std::path::PathBuf {
    let home = std::process::Command::new("getent").args(["passwd", &whoami()]).output().map(|o| String::from_utf8_lossy(&o.stdout).split(':').nth(5).unwrap_or("").to_string()).unwrap_or_default();
    std::path::PathBuf::from(home).join(".ssh/known_hosts")
}

fn whoami() -> String {
    std::process::Command::new("id").arg("-un").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}
