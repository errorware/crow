//! W06: hardening. Password login off, the guarded way; then Fleet Setup →
//! Hardening: the posture scan, root password login fixed everywhere, and
//! a firewall turned on that still lets SSH in. Every fix must leave Crow
//! able to log in.

use std::time::Duration;

use gpui_kit::TestAppContext;

use super::harness::{act, crow_exec, isolate, open_crow, root, server, settle, ssh, step, targets};
use crate::app::password_login::PasswordLoginStage;
use crate::views::fleet::run::RunPhase;

#[gpui_kit::test]
fn w06_hardening(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    isolate();
    let t = targets();
    let (app, w) = open_crow(cx);
    settle(cx, w);
    let app2 = server(cx, &app, "wf-app2");

    step("Setup, outside Crow: password login back on for wf-app2 (a rerun)");
    let (ok, out) = ssh(&t["APP2"], &format!("rm -f {}; sshd -t && systemctl reload sshd && sshd -T | grep -i '^passwordauthentication'", crate::security::sshd_passwords::DROPIN));
    assert!(ok && out.contains("yes"), "{out}");

    step("wf-app2 → sshd_config → TURN OFF PASSWORD LOGIN SAFELY");
    act(cx, &app, w, |this, _, cx| {
        this.switch_tab(&app2.id, cx);
        this.open_password_login(cx);
    });
    let users = app.read_with(cx, |this, _| match this.password_login.as_ref().map(|f| &f.stage) {
        Some(PasswordLoginStage::Ready(p)) => Ok(p.password_users.clone()),
        Some(PasswordLoginStage::Unavailable(why)) => Err(why.clone()),
        Some(PasswordLoginStage::Done(r)) => Err(format!("done already: {r:?}")),
        _ => Err("still checking".into()),
    });
    let users = users.expect("ready to run");
    assert!(users.iter().any(|u| u == "root"), "root has a password (W02): {users:?}");

    step("It won't run until the password accounts are acknowledged");
    act(cx, &app, w, |this, _, cx| this.run_password_login_off(cx));
    assert!(app.read_with(cx, |this, _| matches!(this.password_login.as_ref().map(|f| &f.stage), Some(PasswordLoginStage::Ready(_)))), "still waiting");
    act(cx, &app, w, |this, _, cx| {
        this.toggle_password_login_ack(cx);
        this.run_password_login_off(cx);
    });
    let done = app.read_with(cx, |this, _| match this.password_login.as_ref().map(|f| &f.stage) {
        Some(PasswordLoginStage::Done(r)) => Some(r.clone()),
        _ => None,
    });
    let log = done.expect("finished").expect("succeeded");
    step(&format!("  {}", log.last().cloned().unwrap_or_default()));
    let (_, out) = ssh(&t["APP2"], "sshd -T | grep -i '^passwordauthentication'");
    assert_eq!(out.trim().to_lowercase(), "passwordauthentication no");
    assert!(crow_exec(&app2, "true").is_ok(), "Crow still logs in");
    let changes = app.read_with(cx, |this, _| this.vault.db().lock().unwrap().list_change_records(&app2.id, 20).unwrap());
    assert!(changes.iter().any(|c| c.action_kind == "sshd.password_login_off" && c.outcome == "success"));

    step("Fleet Setup → Hardening: the posture of every server");
    for name in ["wf-bastion", "wf-app1", "wf-app2"] {
        crow_exec(&server(cx, &app, name), "true").unwrap();
    }
    act(cx, &app, w, |this, _, cx| this.check_posture(cx));
    let findings: Vec<(String, Vec<&'static str>)> = app.read_with(cx, |this, _| {
        this.fleet.servers.iter().map(|s| (s.name.clone(), this.server_findings(&s.id).unwrap_or_default().iter().map(|(f, _)| f.code()).collect())).collect()
    });
    step(&format!("  {findings:?}"));
    let app2_codes = &findings.iter().find(|(n, _)| n == "wf-app2").unwrap().1;
    assert!(!app2_codes.contains(&"password-login"), "fixed above");
    for name in ["wf-bastion", "wf-app1", "wf-app2"] {
        let srv = server(cx, &app, name);
        let (_, out) = ssh(&srv.host, "sshd -T | grep -i '^permitrootlogin'");
        step(&format!("  {name}: {}", out.trim()));
    }

    for (code, keyword_checks) in [("root-password-login", "permitrootlogin prohibit-password"), ("no-firewall", "")] {
        let affected: Vec<String> = findings.iter().filter(|(_, c)| c.contains(&code)).map(|(n, _)| n.clone()).collect();
        if affected.is_empty() {
            step(&format!("  nothing has {code}"));
            continue;
        }
        step(&format!("FIX EVERYWHERE · {code} on {affected:?}"));
        act(cx, &app, w, |this, window, cx| this.plan_hardening_fix(code, None, window, cx));
        let (planned, excluded): (Vec<String>, Vec<(String, String)>) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.steps.iter().map(|s| s.server.clone()).collect(), r.run.excluded.clone())).unwrap());
        step(&format!("  planned {planned:?}, left out {excluded:?}"));
        // Without ufw or firewalld there's nothing to turn on: left out, saying so (ERR-167).
        for (name, why) in &excluded {
            assert!(code == "no-firewall" && why.contains("ufw nor firewalld"), "{name}: {why}");
            let (_, tools) = ssh(&server(cx, &app, name).host, "command -v ufw firewall-cmd || echo none");
            assert_eq!(tools.trim(), "none", "{name} really has neither");
        }
        assert_eq!(planned.len() + excluded.len(), affected.len());
        let affected = planned;
        if affected.is_empty() {
            act(cx, &app, w, |this, _, cx| this.close_fleet_run(cx));
            continue;
        }
        act(cx, &app, w, |this, window, cx| {
            let r = this.fleet_runner.as_ref().unwrap();
            let keyword = r.run.keyword;
            r.input.update(cx, |i, cx| i.set_value(keyword.to_string(), window, cx));
            this.confirm_fleet_run(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(300));
        settle(cx, w);
        let (phase, summary, detail) = app.read_with(cx, |this, _| this.fleet_runner.as_ref().map(|r| (r.run.phase, r.run.summary(), format!("{:#?}", r.run.steps))).unwrap());
        assert_eq!(phase, RunPhase::Finished);
        assert!(!summary.contains("stopped") && !summary.contains("failed"), "{summary} {detail}");
        act(cx, &app, w, |this, _, cx| this.close_fleet_run(cx));
        for name in &affected {
            let srv = server(cx, &app, name);
            assert!(crow_exec(&srv, "true").is_ok(), "Crow still logs in to {name}");
            let addr = srv.host.clone();
            if !keyword_checks.is_empty() {
                let (_, out) = ssh(&addr, "sshd -T | grep -i '^permitrootlogin'");
                // Older sshd prints prohibit-password's old name.
                let got = out.trim().to_lowercase().replace("without-password", "prohibit-password");
                assert_eq!(got, keyword_checks, "{name}");
            } else {
                let (_, out) = ssh(&addr, "(ufw status 2>/dev/null | head -1; firewall-cmd --state 2>/dev/null; nft list ruleset 2>/dev/null | grep -c 'dport 22') | tr '\\n' ' '");
                assert!(out.contains("active") || out.contains("running"), "{name}: a firewall is on: {out}");
            }
        }
    }

    step("The posture read again: those findings are gone");
    act(cx, &app, w, |this, _, cx| this.check_posture(cx));
    let left: Vec<(String, Vec<&'static str>)> = app.read_with(cx, |this, _| {
        this.fleet.servers.iter().map(|s| (s.name.clone(), this.server_findings(&s.id).unwrap_or_default().iter().map(|(f, _)| f.code()).collect())).collect()
    });
    step(&format!("  {left:?}"));
    for (name, codes) in &left {
        assert!(!codes.contains(&"root-password-login"), "{name}: {codes:?}");
        match name.as_str() {
            "wf-app1" => assert!(!codes.contains(&"no-firewall"), "ufw is on now: {codes:?}"),
            "wf-app2" => assert!(!codes.contains(&"password-login"), "{codes:?}"),
            _ => {}
        }
    }
    let (_, ufw) = ssh(&t["APP1"], "ufw status | head -1; ufw status | grep -c '22'");
    assert!(ufw.contains("Status: active"), "{ufw}");
}
