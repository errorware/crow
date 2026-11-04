//! W10: a fresh Ubuntu box to a running Rails app with Dokku, the way the
//! spec's operator journey goes: enable the plugin, bootstrap, a Postgres,
//! an app linked to it, its secrets, a deploy key, `git push`, then check
//! it serves. Needs `DOKKU=<address>` and `RAILS_APP=<repo dir>` in
//! targets.env; the repo's config/master.key is sent, never printed.

use gpui_kit::{Entity, TestAppContext};

use super::harness::{act, isolate, open_crow, root, settle, ssh, step, targets};
use crate::app::dokku::{DokkuSelection, PendingDokkuAction};
use crate::app::CrowApp;
use crate::dokku::{Scan, ServiceKind};
use crate::views::onboard::OnboardStep;

const APP: &str = "quorum";
const DB: &str = "quorum-db";

fn scan(cx: &mut TestAppContext, app: &Entity<CrowApp>) -> Scan {
    app.read_with(cx, |this, _| match &this.dokku.scan {
        Some(Ok(s)) => s.clone(),
        other => panic!("no Dokku scan: {other:?}"),
    })
}

/// The last action's outcome; fails the test on an error.
fn done(cx: &mut TestAppContext, app: &Entity<CrowApp>, what: &str) {
    let result = app.read_with(cx, |this, _| this.dokku.result.clone());
    match result {
        Some((true, msg)) => step(&format!("  {msg}")),
        other => panic!("{what}: {other:?}"),
    }
}

#[gpui_kit::test]
fn w10_rails_app_on_dokku(cx: &mut TestAppContext) {
    if root().is_none() {
        return;
    }
    let root = isolate();
    let t = targets();
    let (addr, repo) = (t["DOKKU"].clone(), std::path::PathBuf::from(&t["RAILS_APP"]));
    let (app, w) = open_crow(cx);
    settle(cx, w);

    step("Keys: import the test key; Add Server: crow-dokku-1, key login");
    let fp = act(cx, &app, w, |this, _, cx| {
        this.refresh_keys(cx);
        this.keys.discovered.iter().find(|k| k.file_path.ends_with("id_ed25519")).map(|k| k.fingerprint.clone())
    })
    .expect("the test key is discovered");
    act(cx, &app, w, |this, _, cx| this.import_discovered_key(&fp, Some("fleet"), cx));
    let key_id = app.read_with(cx, |this, _| this.keys.enrolled.iter().find(|k| k.fingerprint == fp).map(|k| k.id.clone())).unwrap();
    act(cx, &app, w, |this, _, cx| {
        this.start_onboarding(cx);
        this.onboard_state.host = addr.clone();
        this.onboard_state.user = "root".into();
        this.onboard_set_auth("publickey", cx);
        this.onboard_state.selected_key_id = Some(key_id.clone());
        this.onboard_inputs = None;
        this.onboard_next_step(cx);
    });
    let step_now = app.read_with(cx, |this, _| this.onboard_state.step);
    assert_eq!(step_now, OnboardStep::Verify);
    act(cx, &app, w, |this, _, cx| this.onboard_accept_host_key(cx));
    act(cx, &app, w, |this, _, cx| {
        this.onboard_next_step(cx);
        this.onboard_state.label = "crow-dokku-1".into();
        this.onboard_set_env("PROD", cx);
        this.onboard_set_group("paas", cx);
        this.onboard_inputs = None;
        this.onboard_next_step(cx);
    });
    let srv_id = app.read_with(cx, |this, _| this.fleet.servers.iter().find(|s| s.name == "crow-dokku-1").map(|s| s.id.clone())).expect("enrolled");

    step("Settings → Plugins: Dokku on; Server → Dokku: not installed, can bootstrap");
    act(cx, &app, w, |this, _, cx| {
        this.set_plugin_enabled("dokku", true, cx);
        this.screen = crate::app::Screen::Server;
        this.fleet.active_tab_id = srv_id.clone();
        this.set_view("dokku", cx);
    });
    let s = scan(cx, &app);
    if !s.installed {
        assert!(s.can_bootstrap(), "a fresh Ubuntu box can take Dokku: {s:?}");
        step(&format!("  {} · {} MB · docker {}", s.os_id, s.mem_mb, s.docker_ok));

        step("BOOTSTRAP DOKKU (confirm): Docker + Dokku, a few minutes");
        act(cx, &app, w, |this, _, cx| {
            this.request_dokku_action(PendingDokkuAction::BootstrapDokku, cx);
            this.confirm_dokku_pending(cx);
        });
        done(cx, &app, "bootstrap");
    }
    let s = scan(cx, &app);
    assert!(s.installed && !s.version.is_empty(), "{s:?}");
    step(&format!("  Dokku {}", s.version));

    let global = format!("{}.sslip.io", addr);
    step(&format!("Global setup: domain {global}"));
    act(cx, &app, w, |this, window, cx| {
        this.ensure_dokku_inputs(window, cx);
        let input = this.dokku_inputs.as_ref().unwrap().global_domain.clone();
        input.update(cx, |s, cx| s.set_value(global.clone(), window, cx));
        this.save_dokku_global_domain(cx);
    });
    done(cx, &app, "global domain");
    assert!(scan(cx, &app).global_domains.contains(&global));

    if scan(cx, &app).find_service(ServiceKind::Postgres, DB).is_none() {
        step(&format!("New service: Postgres {DB} (installs the plugin first)"));
        act(cx, &app, w, |this, window, cx| {
            this.ensure_dokku_inputs(window, cx);
            this.set_dokku_new_service_kind(ServiceKind::Postgres, cx);
            let input = this.dokku_inputs.as_ref().unwrap().new_service_name.clone();
            input.update(cx, |s, cx| s.set_value(DB, window, cx));
            this.create_dokku_service(cx);
        });
        done(cx, &app, "create postgres");
    }
    let s = scan(cx, &app);
    assert!(s.has_plugin("postgres"), "{:?}", s.plugins);
    let db = s.find_service(ServiceKind::Postgres, DB).expect("the database is listed").clone();
    step(&format!("  {DB}: {} {}", db.status, db.version));

    if scan(cx, &app).find_app(APP).is_none() {
        step(&format!("New app: {APP}"));
        act(cx, &app, w, |this, window, cx| {
            this.ensure_dokku_inputs(window, cx);
            let input = this.dokku_inputs.as_ref().unwrap().new_app_name.clone();
            input.update(cx, |s, cx| s.set_value(APP, window, cx));
            this.create_dokku_app(cx);
        });
        done(cx, &app, "create app");
        let sel = app.read_with(cx, |this, _| this.dokku.selected.clone());
        assert_eq!(sel, Some(DokkuSelection::App(APP.into())), "the new app opens on its Deploy tab");
    }

    if !scan(cx, &app).find_app(APP).unwrap().linked_services.contains(&(ServiceKind::Postgres, DB.into())) {
        step(&format!("Services: link {DB} → {APP} (confirm)"));
        act(cx, &app, w, |this, _, cx| {
            this.request_dokku_action(PendingDokkuAction::Link { kind: ServiceKind::Postgres, service: DB.into(), app: APP.into(), link: true }, cx);
            this.confirm_dokku_pending(cx);
        });
        done(cx, &app, "link");
    }
    let s = scan(cx, &app);
    assert!(s.find_app(APP).unwrap().linked_services.contains(&(ServiceKind::Postgres, DB.into())));
    assert!(s.find_service(ServiceKind::Postgres, DB).unwrap().linked_apps.contains(&APP.to_string()));

    step("Env & Secrets: the link set DATABASE_URL; paste the rest as .env, apply without a restart");
    act(cx, &app, w, |this, _, cx| {
        this.select_dokku_app(APP, cx);
        this.set_dokku_tab(crate::app::dokku::DokkuDrawerTab::Env, cx);
        this.load_dokku_app_env(APP, true, cx);
    });
    let env = app.read_with(cx, |this, _| this.dokku.env_cache.get(APP).cloned()).expect("env read").expect("env readable");
    let url = env.get("DATABASE_URL").expect("linking set DATABASE_URL").clone();
    assert!(url.starts_with("postgres://"), "a Postgres URL");
    let master = std::fs::read_to_string(repo.join("config/master.key")).expect("config/master.key").trim().to_string();
    let blob = format!(
        "RAILS_MASTER_KEY={master}\nCACHE_DATABASE_URL={url}\nQUEUE_DATABASE_URL={url}\nCABLE_DATABASE_URL={url}\nSOLID_QUEUE_IN_PUMA=true\nRAILS_LOG_TO_STDOUT=1\n"
    );
    act(cx, &app, w, |this, window, cx| {
        this.ensure_dokku_inputs(window, cx);
        if this.dokku.restart_on_env_apply {
            this.toggle_dokku_restart_on_env(cx);
        }
        this.toggle_dokku_dotenv_paste(cx);
        let input = this.dokku_inputs.as_ref().unwrap().dotenv_blob.clone();
        input.update(cx, |s, cx| s.set_value(blob.clone(), window, cx));
        this.stage_dokku_dotenv_blob(APP, window, cx);
    });
    let staged = app.read_with(cx, |this, _| this.dokku.staged_env_set.get(APP).map(|m| m.keys().cloned().collect::<Vec<_>>()));
    step(&format!("  staged: {}", staged.clone().unwrap_or_default().join(", ")));
    assert_eq!(staged.map(|k| k.len()), Some(6));
    act(cx, &app, w, |this, _, cx| this.apply_dokku_staged_env(APP, cx));
    done(cx, &app, "apply env");
    let summary = app.read_with(cx, |this, _| this.dokku.result.clone().unwrap().1);
    assert!(!summary.contains(&master), "the master key stays out of the summary");
    let env = app.read_with(cx, |this, _| this.dokku.env_cache.get(APP).cloned()).unwrap().unwrap();
    assert_eq!(env.get("RAILS_MASTER_KEY").map(String::as_str), Some(master.as_str()), "the secret round-trips");

    step("Deploy keys: authorize the test key for git push");
    let public = std::fs::read_to_string(root.join("home/.ssh/id_ed25519.pub")).unwrap();
    if !scan(cx, &app).deploy_keys.iter().any(|k| k.name == "crow-wf") {
        act(cx, &app, w, |this, _, cx| this.authorize_dokku_ssh_key("crow-wf", &public, cx));
        done(cx, &app, "deploy key");
    }
    assert!(scan(cx, &app).deploy_keys.iter().any(|k| k.name == "crow-wf"));

    step(&format!("Outside Crow: git push dokku@{addr}:{APP} (builds from the Dockerfile; several minutes)"));
    let key = root.join("home/.ssh/id_ed25519");
    let push = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["push", "--force", &format!("dokku@{addr}:{APP}"), "HEAD:refs/heads/master"])
        .env("GIT_SSH_COMMAND", format!("/usr/bin/ssh -F /dev/null -o IdentitiesOnly=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i {}", key.display()))
        .output()
        .expect("git");
    let log = format!("{}{}", String::from_utf8_lossy(&push.stdout), String::from_utf8_lossy(&push.stderr));
    std::fs::write(root.join("w10-push.log"), &log).unwrap();
    assert!(push.status.success(), "push failed; tail:\n{}", log.lines().rev().take(40).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n"));

    step("REFRESH: the app is deployed and its web process runs");
    act(cx, &app, w, |this, _, cx| this.refresh_dokku(cx));
    let s = scan(cx, &app);
    let a = s.find_app(APP).unwrap();
    step(&format!("  {} · {} · ports {:?} · domains {:?} · git {}", a.process_summary(), if a.running { "running" } else { "stopped" }, a.ports.iter().map(|p| p.spec()).collect::<Vec<_>>(), a.domains, a.git_sha));
    assert!(a.deployed && a.running, "{a:?}");
    assert!(a.processes.iter().any(|p| p.proc_type == "web" && p.running >= 1), "{:?}", a.processes);
    assert!(a.ports.iter().any(|p| p.scheme == "http" && p.host_port == 80), "the port Dokku detected from EXPOSE: {:?}", a.ports);

    let domain = format!("{APP}.{global}");
    // On the box itself; once TLS is on (a rerun), plain http redirects.
    let (scheme, port) = if a.ssl_enabled { ("https", 443) } else { ("http", 80) };
    step(&format!("It serves: {scheme}://{domain}/up"));
    let (ok, out) = ssh(&addr, &format!("curl -sk -o /dev/null -w '%{{http_code}}' --resolve {domain}:{port}:127.0.0.1 -H 'X-Forwarded-Proto: https' {scheme}://{domain}/up"));
    step(&format!("  /up → {out}"));
    assert!(ok && out.trim() == "200", "{out}");

    match t.get("LE_EMAIL") {
        None => step("TLS: skipped (no LE_EMAIL in targets.env)"),
        Some(email) => {
            step("Domains & TLS: no contact email yet, so the card asks for one; ENABLE LET'S ENCRYPT");
            act(cx, &app, w, |this, window, cx| {
                this.set_dokku_tab(crate::app::dokku::DokkuDrawerTab::Domains, cx);
                this.ensure_dokku_inputs(window, cx);
                let input = this.dokku_inputs.as_ref().unwrap().letsencrypt_email.clone();
                input.update(cx, |s, cx| s.set_value(email.clone(), window, cx));
                this.enable_dokku_letsencrypt(APP, cx);
            });
            done(cx, &app, "letsencrypt");
            let s = scan(cx, &app);
            let a = s.find_app(APP).unwrap();
            step(&format!("  ssl {} · expires in {:?} days · server email set: {}", a.ssl_enabled, a.letsencrypt_expiry_days, !s.letsencrypt_email.is_empty()));
            assert!(a.ssl_enabled && s.letsencrypt_cron, "{a:?}");
            assert_eq!(&s.letsencrypt_email, email);
            // From here, with the system's trust store: a real certificate.
            let out = std::process::Command::new("curl").args(["-s", "-o", "/dev/null", "-w", "%{http_code}", &format!("https://{domain}/up")]).output().expect("curl");
            let code = String::from_utf8_lossy(&out.stdout).to_string();
            step(&format!("  https://{domain}/up → {code}"));
            assert_eq!(code, "200");
        }
    }

    step("Wire: the app's secrets never reach the ledger, in memory or in crow.db");
    let db_password = url.split_once("://").and_then(|(_, r)| r.split_once('@')).and_then(|(auth, _)| auth.split_once(':')).map(|(_, p)| p.to_string()).expect("a password in DATABASE_URL");
    let secrets = [master.clone(), crate::dokku::base64_encode(master.as_bytes()), db_password];
    let events = crate::wire::recent_events();
    let env_reads = events.iter().filter(|e| e.request_payload.contains("config:export")).count();
    assert!(env_reads > 0, "the Env tab's reads are on the wire");
    for e in &events {
        for (i, secret) in secrets.iter().enumerate() {
            for (field, text) in [("summary", &e.summary), ("request", &e.request_payload), ("response", &e.response_payload)] {
                assert!(!text.contains(secret.as_str()), "secret #{i} in event {} {field}: {}", e.seq, e.summary);
            }
        }
    }
    let conn = rusqlite::Connection::open_with_flags(root.join("xdg/config/crow/crow.db"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let mut stmt = conn.prepare("SELECT summary || request_payload || response_payload || coalesce(error, '') FROM wire_log").unwrap();
    let rows: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap().map(Result::unwrap).collect();
    assert!(!rows.is_empty(), "wire_log persisted");
    for (i, secret) in secrets.iter().enumerate() {
        assert!(!rows.iter().any(|r| r.contains(secret.as_str())), "secret #{i} in wire_log");
    }
    let exported = events.iter().find(|e| e.request_payload.contains("config:export")).unwrap();
    assert!(exported.response_payload.contains("\"RAILS_MASTER_KEY\":\"[REDACTED]\""), "keys kept, values blanked: {}", exported.response_payload);
    step(&format!("  {} events, {env_reads} env reads, {} rows on disk: no secret in any", events.len(), rows.len()));

    step("Containers: Dokku's containers show up with what they use");
    act(cx, &app, w, |this, _, cx| this.set_view("containers", cx));
    let names: Vec<String> = app.read_with(cx, |this, _| this.containers.by_server.get(&srv_id).map(|s| s.containers.iter().map(|c| c.name.clone()).collect()).unwrap_or_default());
    step(&format!("  {names:?}"));
    assert!(names.iter().any(|n| n.starts_with(&format!("{APP}.web"))), "{names:?}");
}
