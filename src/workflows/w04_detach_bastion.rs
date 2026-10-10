//! W04: take a server out from behind its bastion, then stop the bastion
//! being one. Along the way: a bastion still in use can't be archived.

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, isolate, open_crow, root, server, settle, ssh, ssh_via, step, targets};
use crate::app::bastions::BASTION_TAG;

#[gpui_kit::test]
fn w04_detach_a_server_from_its_bastion(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let (bastion_ip, app2_ip) = (t["BASTION"].clone(), t["APP2"].clone());
    let (app, w) = open_crow(cx);
    settle(cx, w);
    // A run before ERR-165 was fixed archived the bastion: put it back.
    let archived = app.read_with(cx, |this, _| this.fleet.archived.iter().find(|s| s.name == "wf-bastion").map(|s| s.id.clone()));
    if let Some(id) = archived {
        act(cx, &app, w, |this, _, cx| this.restore_archived_server(&id, cx));
    }
    let bastion = server(cx, &app, "wf-bastion");
    let app2 = server(cx, &app, "wf-app2");
    assert_eq!(app2.jump_host_id.as_deref(), Some(bastion.id.as_str()), "W02 left app2 behind the bastion");

    step("Archiving the bastion while wf-app2 goes through it is refused");
    act(cx, &app, w, |this, _, cx| this.archive_server(&bastion.id, cx));
    let (still_there, notice) = app.read_with(cx, |this, _| (this.fleet.servers.iter().any(|s| s.id == bastion.id), this.fleet.notice.clone().unwrap_or_default()));
    assert!(still_there, "the bastion stays in the fleet");
    assert!(notice.contains("wf-app2"), "says who still uses it: {notice}");
    assert!(crow_exec(&app2, "true").is_ok(), "wf-app2 still works");

    step("Setup, outside Crow: app2 accepts direct logins again");
    let (ok, out) = ssh_via(&bastion_ip, &app2_ip, "rm -f /etc/ssh/sshd_config.d/10-crow-wf.conf && sshd -t && systemctl reload sshd && echo done");
    assert!(ok && out.contains("done"), "{out}");
    assert!(ssh(&app2_ip, "true").0);

    step("wf-app2 → identity bar → Add Server again: Crow's key, route Directly");
    act(cx, &app, w, |this, _, cx| {
        this.start_onboarding_for(&app2, cx);
        this.onboard_set_auth("publickey", cx);
        this.onboard_state.selected_key_id = app2.key_id.clone();
        this.onboard_state.jump_host_id = None;
        this.onboard_inputs = None;
        this.onboard_next_step(cx);
    });
    let (known, err, logs) = app.read_with(cx, |this, _| {
        let o = &this.onboard_state;
        (o.probe_result.as_ref().map(|p| p.is_known_host), o.error_message.clone(), o.probe_logs.iter().map(|l| l.message.clone()).collect::<Vec<_>>())
    });
    assert_eq!(known, Some(true), "the host key is already trusted: {err:?} {logs:#?}");
    act(cx, &app, w, |this, _, cx| {
        this.onboard_next_step(cx);
        this.onboard_next_step(cx);
    });
    let moved = server(cx, &app, "wf-app2");
    let err = app.read_with(cx, |this, _| this.onboard_state.error_message.clone());
    assert_eq!(moved.jump_host_id, None, "direct now: {err:?}");
    assert_eq!((moved.id.as_str(), moved.key_id.as_ref()), (app2.id.as_str(), app2.key_id.as_ref()), "the same server, the same key");
    let count = app.read_with(cx, |this, _| this.fleet.servers.iter().filter(|s| s.host == app2_ip).count());
    assert_eq!(count, 1, "not enrolled twice");

    step("Crow reaches it directly, on a new connection (not the bastion's)");
    assert_eq!(crow_exec(&moved, "hostname").map(|s| s.trim().to_string()), Ok("wf-app2".into()));
    let (_, from) = ssh(&app2_ip, "journalctl -u sshd --since '-1min' --no-pager | grep 'Accepted publickey' | tail -1");
    assert!(!from.contains(&bastion_ip), "the last login didn't come through the bastion: {from}");

    step("wf-bastion stops being a bastion, then can be archived and restored");
    act(cx, &app, w, |this, _, cx| this.set_bastion(&bastion.id, false, cx));
    let b = server(cx, &app, "wf-bastion");
    assert!(!b.tags.iter().any(|t| t == BASTION_TAG), "{:?}", b.tags);
    act(cx, &app, w, |this, _, cx| this.archive_server(&bastion.id, cx));
    let archived = app.read_with(cx, |this, _| this.fleet.archived.iter().any(|s| s.id == bastion.id));
    assert!(archived);
    act(cx, &app, w, |this, _, cx| this.restore_archived_server(&bastion.id, cx));
    let back = server(cx, &app, "wf-bastion");
    assert_eq!(back.host, bastion_ip);
}
