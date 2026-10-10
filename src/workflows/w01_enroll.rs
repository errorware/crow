//! W01: enrol a server that trusts a key Crow found on disk.

use gpui_kit::TestAppContext;

use super::harness::{act, isolate, open_crow, root, settle, ssh, step, targets};
use crate::views::onboard::OnboardStep;

#[gpui_kit::test]
fn w01_enroll_a_server_with_a_key(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let (app, w) = open_crow(cx);
    settle(cx, w);

    step("Settings → Keys: the test key is discovered in ~/.ssh and imported");
    let fp = act(cx, &app, w, |this, _, cx| {
        this.refresh_keys(cx);
        this.keys.discovered.iter().find(|k| k.file_path.ends_with("id_ed25519")).map(|k| k.fingerprint.clone())
    })
    .expect("the test key is discovered");
    act(cx, &app, w, |this, _, cx| this.import_discovered_key(&fp, Some("fleet"), cx));
    let key_id = app.read_with(cx, |this, _| this.keys.enrolled.iter().find(|k| k.fingerprint == fp).map(|k| k.id.clone())).expect("the key is enrolled");

    for (name, addr, env, group) in [("wf-bastion", &t["BASTION"], "PROD", "edge"), ("wf-app1", &t["APP1"], "STAGE", "app")] {
        step(&format!("Add Server: {name} at {addr}, key login"));
        act(cx, &app, w, |this, _, cx| {
            this.start_onboarding(cx);
            this.onboard_state.host = addr.clone();
            this.onboard_state.user = "root".into();
            this.onboard_set_auth("publickey", cx);
            this.onboard_state.selected_key_id = Some(key_id.clone());
            this.onboard_inputs = None;
            this.onboard_next_step(cx);
        });
        let (step_now, reachable, known, err) = app.read_with(cx, |this, _| {
            let o = &this.onboard_state;
            (o.step, o.probe_result.as_ref().map(|p| p.is_reachable), o.probe_result.as_ref().map(|p| p.is_known_host), o.error_message.clone())
        });
        assert_eq!(step_now, OnboardStep::Verify, "{err:?}");
        assert_eq!(reachable, Some(true), "the probe reaches {addr}");
        assert_eq!(known, Some(false), "a new server's key isn't trusted yet");

        step("Verify: trust the host key; Crow logs in and reads the facts");
        act(cx, &app, w, |this, _, cx| this.onboard_accept_host_key(cx));
        let (distro, logs) = app.read_with(cx, |this, _| (this.onboard_state.facts.distro.clone(), this.onboard_state.probe_logs.iter().map(|l| format!("{l:?}")).collect::<Vec<_>>()));
        assert!(!distro.is_empty() && distro != "—", "facts read; logs: {logs:#?}");
        step(&format!("  facts: {distro}"));

        step("Name: label, environment, group; Enroll");
        act(cx, &app, w, |this, _, cx| {
            this.onboard_next_step(cx);
            this.onboard_state.label = name.into();
            this.onboard_set_env(env, cx);
            this.onboard_set_group(group, cx);
            this.onboard_inputs = None;
            this.onboard_next_step(cx);
        });
        let srv = app.read_with(cx, |this, _| this.fleet.servers.iter().find(|s| s.name == name).cloned()).expect("enrolled");
        assert_eq!((srv.host.as_str(), srv.env.as_str(), srv.group_name.as_str(), srv.auth_method.as_str()), (addr.as_str(), env, group, "publickey"));
        assert_eq!(srv.key_id.as_deref(), Some(key_id.as_str()));
        assert!(srv.host_key_fingerprint.as_deref().is_some_and(|f| f.starts_with("SHA256:")), "host key pinned: {:?}", srv.host_key_fingerprint);
        let active = app.read_with(cx, |this, _| this.fleet.active_tab_id.clone());
        assert_eq!(active, srv.id, "the new server's tab is open");
    }

    step("The vault keeps them across a restart");
    let (app2, w2) = open_crow(cx);
    settle(cx, w2);
    let names: Vec<String> = app2.read_with(cx, |this, _| this.fleet.servers.iter().map(|s| s.name.clone()).collect());
    assert!(names.contains(&"wf-bastion".to_string()) && names.contains(&"wf-app1".to_string()), "{names:?}");
    let _ = (app, w);

    step("Outside Crow: the servers are untouched (no agent, no new users)");
    let (ok, out) = ssh(&t["APP1"], "getent passwd | awk -F: '$3>=1000 && $3<65534' | wc -l");
    assert!(ok && out.trim() == "0", "{out}");
}
