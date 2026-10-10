//! W02: make a server a bastion, then add a server only it can reach,
//! logging in with a password once (Crow installs its own key through the
//! bastion).

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, isolate, open_crow, root, server, settle, ssh, ssh_via, step, targets};
use crate::app::bastions::BASTION_TAG;

/// The root password app2 gets for this workflow (a throwaway server).
pub const APP2_PASSWORD: &str = "wf-Throwaway-7731-pass";

#[gpui_kit::test]
fn w02_attach_a_server_behind_a_bastion(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let (bastion_ip, app2_ip) = (t["BASTION"].clone(), t["APP2"].clone());

    step("Setup, outside Crow: app2 lets root in only from the bastion, with a password");
    let (ok, out) = ssh_via(&bastion_ip, &app2_ip, &format!(
        "echo 'root:{APP2_PASSWORD}' | chpasswd && printf 'AllowUsers root@{bastion_ip}\\n' > /etc/ssh/sshd_config.d/10-crow-wf.conf && sshd -t && systemctl reload sshd && echo done"
    ));
    assert!(ok && out.contains("done"), "{out}");
    let (direct, _) = ssh(&app2_ip, "true");
    assert!(!direct, "app2 refuses direct logins now");
    let (via, out) = ssh_via(&bastion_ip, &app2_ip, "hostname");
    assert!(via && out.contains("wf-app2"), "{out}");

    let (app, w) = open_crow(cx);
    settle(cx, w);

    step("Server → identity bar: designate wf-bastion a bastion");
    let bastion = server(cx, &app, "wf-bastion");
    act(cx, &app, w, |this, _, cx| this.set_bastion(&bastion.id, true, cx));
    let bastion = server(cx, &app, "wf-bastion");
    assert!(bastion.tags.iter().any(|t| t == BASTION_TAG), "{:?}", bastion.tags);
    let offered: Vec<String> = app.read_with(cx, |this, _| crate::app::bastions::choices(None, &this.fleet.servers).iter().map(|s| s.name.clone()).collect());
    assert_eq!(offered, ["wf-bastion"], "Add Server offers it as a route");

    step("Add Server: app2's address, root + password, route via wf-bastion");
    act(cx, &app, w, |this, _, cx| {
        this.start_onboarding(cx);
        this.onboard_state.host = app2_ip.clone();
        this.onboard_state.user = "root".into();
        this.onboard_set_auth("password", cx);
        this.onboard_state.password = crate::secret_string::SecretString::new(APP2_PASSWORD);
        this.onboard_state.jump_host_id = Some(bastion.id.clone());
        this.onboard_inputs = None;
        this.onboard_next_step(cx);
    });
    let (reachable, known, err, logs) = app.read_with(cx, |this, _| {
        let o = &this.onboard_state;
        (o.probe_result.as_ref().map(|p| p.is_reachable), o.probe_result.as_ref().map(|p| p.is_known_host), o.error_message.clone(), o.probe_logs.iter().map(|l| format!("{l:?}")).collect::<Vec<_>>())
    });
    assert_eq!(reachable, Some(true), "probed through the bastion: {err:?} {logs:#?}");
    assert_eq!(known, Some(false));

    step("Verify: trust app2's host key; Crow installs its key with the password, through the bastion");
    act(cx, &app, w, |this, _, cx| this.onboard_accept_host_key(cx));
    let (auth, distro, password_left, logs) = app.read_with(cx, |this, _| {
        let o = &this.onboard_state;
        (o.auth_method.clone(), o.facts.distro.clone(), !o.password.is_empty(), o.probe_logs.iter().map(|l| format!("{l:?}")).collect::<Vec<_>>())
    });
    assert_eq!(auth, "publickey", "switched to Crow's key: {logs:#?}");
    assert!(!password_left, "the password is wiped once the key is in");
    assert!(distro.contains("Rocky"), "{distro} {logs:#?}");

    step("Name and enrol");
    act(cx, &app, w, |this, _, cx| {
        this.onboard_next_step(cx);
        this.onboard_state.label = "wf-app2".into();
        this.onboard_set_env("PROD", cx);
        this.onboard_set_group("app", cx);
        this.onboard_inputs = None;
        this.onboard_next_step(cx);
    });
    let app2 = server(cx, &app, "wf-app2");
    assert_eq!(app2.jump_host_id.as_deref(), Some(bastion.id.as_str()));
    assert_eq!(app2.auth_method, "publickey");
    let crow_key = app.read_with(cx, |this, _| this.keys.enrolled.iter().find(|k| Some(&k.id) == app2.key_id.as_ref()).cloned()).expect("app2's key is enrolled");
    assert_eq!(crow_key.name, "crow", "Crow's own key");

    step("Crow reaches app2 through the bastion; the same login without it fails");
    assert_eq!(crow_exec(&app2, "hostname").map(|s| s.trim().to_string()), Ok("wf-app2".into()));
    let direct = crate::vault::ServerRecord { jump_host_id: None, ..app2.clone() };
    assert!(crow_exec(&direct, "hostname").is_err(), "direct login is refused");

    step("Outside Crow: app2's authorized_keys holds Crow's key once");
    let pubkey = crow_key.public_key.split_whitespace().nth(1).unwrap().to_string();
    let (_, out) = ssh_via(&bastion_ip, &app2_ip, &format!("grep -c '{pubkey}' /root/.ssh/authorized_keys"));
    assert_eq!(out.trim(), "1");

    step("The fleet shows the route; the bastion can't stop being one while app2 uses it");
    let route = app.read_with(cx, |this, _| crate::app::bastions::route(&bastion.id, &this.fleet.servers));
    assert_eq!(route, "wf-bastion");
    act(cx, &app, w, |this, _, cx| this.set_bastion(&bastion.id, false, cx));
    let still = server(cx, &app, "wf-bastion");
    assert!(still.tags.iter().any(|t| t == BASTION_TAG), "refused");
    let toast = app.read_with(cx, |this, _| this.keys.toast.clone()).unwrap_or_default();
    assert!(toast.contains("wf-app2"), "names who still uses it: {toast}");
}
