//! Hardening (ERR-145): the posture check's findings, a guarded fix for
//! each, and the ones you accept on purpose.
//!
//! Every fix follows the password-login fix's rules (ERR-34): prove a
//! fresh key login works first, make the change, check it took, prove a
//! fresh login still works, and undo everything on any failure.

use serde::{Deserialize, Serialize};

use crate::host::{Host, HostError, DEFAULT_TIMEOUT};
use crate::topology::posture::Posture;

/// What the posture check found, by a stable code (accepting a risk is
/// stored against it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    RootPasswordLogin,
    RootKeyLogin(String),
    PasswordLogin,
    /// No firewall; the listening ports open to every network (not SSH).
    NoFirewall(Vec<u16>),
}

impl Finding {
    pub fn code(&self) -> &'static str {
        match self {
            Finding::RootPasswordLogin => "root-password-login",
            Finding::RootKeyLogin(_) => "root-key-login",
            Finding::PasswordLogin => "password-login",
            Finding::NoFirewall(_) => "no-firewall",
        }
    }

    pub fn critical(&self) -> bool {
        matches!(self, Finding::RootPasswordLogin) || matches!(self, Finding::NoFirewall(p) if !p.is_empty())
    }

    pub fn describe(&self) -> String {
        match self {
            Finding::RootPasswordLogin => "root can log in over SSH with a password (PermitRootLogin yes)".into(),
            Finding::RootKeyLogin(v) => format!("root can log in over SSH with a key (PermitRootLogin {v})"),
            Finding::PasswordLogin => "sshd accepts passwords (PasswordAuthentication yes)".into(),
            Finding::NoFirewall(p) if p.is_empty() => "no firewall filters incoming traffic".into(),
            Finding::NoFirewall(p) => format!("no firewall, and port{} {} open to every network", if p.len() == 1 { "" } else { "s" }, p.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")),
        }
    }

    /// What the fix does, in a line.
    pub fn fix(&self, login_user: &str) -> String {
        match self {
            Finding::RootPasswordLogin | Finding::RootKeyLogin(_) if login_user == "root" => "PermitRootLogin prohibit-password (Crow logs in as root with a key, so not \"no\")".into(),
            Finding::RootPasswordLogin => "PermitRootLogin prohibit-password".into(),
            Finding::RootKeyLogin(_) => "PermitRootLogin no".into(),
            Finding::PasswordLogin => "PasswordAuthentication no (once a key login is proven)".into(),
            Finding::NoFirewall(p) => format!("turn a firewall on that allows SSH{}", if p.is_empty() { String::new() } else { format!(" and the listening port{} {}", if p.len() == 1 { "" } else { "s" }, p.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")) }),
        }
    }

    /// The root-login fix leaves nothing to do when Crow logs in as root
    /// and only key logins are allowed already.
    pub fn fixable(&self, login_user: &str) -> bool {
        !matches!(self, Finding::RootKeyLogin(v) if login_user == "root" && (v == "prohibit-password" || v == "without-password"))
    }
}

pub fn findings(p: &Posture) -> Vec<Finding> {
    let mut out = Vec::new();
    if p.root_password_login() {
        out.push(Finding::RootPasswordLogin);
    } else if p.root_login_allowed() {
        out.push(Finding::RootKeyLogin(p.root_login.clone().unwrap_or_default()));
    }
    if p.password_auth == Some(true) {
        out.push(Finding::PasswordLogin);
    }
    if p.no_firewall() {
        out.push(Finding::NoFirewall(p.unfiltered_ports()));
    }
    out
}

/// A risk someone decided to live with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accepted {
    pub server_id: String,
    pub code: String,
    pub reason: String,
    pub by: String,
    pub at: String,
}

/// The vault flag that holds accepted risks.
pub const ACCEPTED_FLAG: &str = "posture.accepted";

pub fn is_accepted(accepted: &[Accepted], server_id: &str, f: &Finding) -> bool {
    accepted.iter().any(|a| a.server_id == server_id && a.code == f.code())
}

fn root(host: &dyn Host, argv: &[&str]) -> Result<String, String> {
    host.exec_privileged(argv, &[], DEFAULT_TIMEOUT).map(|o| o.stdout).map_err(|e| match e {
        HostError::Failed { stderr, .. } => stderr.trim().to_string(),
        other => other.to_string(),
    })
}

const RELOAD_SSHD: &str = "systemctl reload ssh 2>/dev/null || systemctl reload sshd 2>/dev/null || { [ -f /run/sshd.pid ] && kill -HUP \"$(cat /run/sshd.pid)\"; } || pkill -HUP -x sshd";

pub const ROOT_DROPIN: &str = "/etc/ssh/sshd_config.d/00-crow-root-login.conf";

/// `sshd -T`'s effective permitrootlogin.
pub fn effective_root_login(sshd_t: &str) -> Option<String> {
    sshd_t.lines().find_map(|l| {
        let mut p = l.split_whitespace();
        (p.next()? == "permitrootlogin").then(|| p.next().unwrap_or_default().to_string())
    })
}

/// Sets PermitRootLogin through a drop-in, step by step; the log says what
/// happened, and on failure what was undone. Refuses "no" when Crow itself
/// logs in as root.
pub fn set_root_login(host: &dyn Host, value: &str, login_user: &str, fresh_login: &dyn Fn() -> Result<(), String>) -> Result<Vec<String>, Vec<String>> {
    let mut log = Vec::new();
    macro_rules! fail {
        ($($arg:tt)*) => {{
            log.push(format!($($arg)*));
            return Err(log);
        }};
    }
    if !matches!(value, "no" | "prohibit-password") {
        fail!("✕ {value:?} isn't a setting Crow applies.");
    }
    if value == "no" && login_user == "root" {
        fail!("✕ Crow logs in to this server as root: PermitRootLogin no would lock Crow out. Nothing was changed.");
    }
    if let Err(e) = fresh_login() {
        fail!("✕ A fresh key login doesn't work ({e}). Nothing was changed.");
    }
    log.push("✓ A fresh key login works.".into());
    let main = match root(host, &["cat", "/etc/ssh/sshd_config"]) {
        Ok(m) => m,
        Err(e) => fail!("✕ Couldn't read sshd_config: {e}. Nothing was changed."),
    };
    if !super::sshd_passwords::reads_dropins(&main) {
        fail!("✕ This sshd_config doesn't include sshd_config.d, so a drop-in wouldn't apply. Nothing was changed; set PermitRootLogin in the sshd_config sheet instead.");
    }
    let undo = |log: &mut Vec<String>, reload: bool| {
        let removed = root(host, &["rm", "-f", "--", ROOT_DROPIN]).is_ok();
        let reloaded = !reload || root(host, &["sh", "-c", RELOAD_SSHD]).is_ok();
        log.push(if removed && reloaded { format!("↺ Undone: removed {ROOT_DROPIN}{}.", if reload { " and reloaded sshd" } else { "" }) } else { format!("⚠ Undo didn't finish: remove {ROOT_DROPIN} by hand and reload sshd.") });
    };
    let content = format!("# Written by Crow (hardening): who may log in as root.\n# Delete this file and reload sshd to undo.\nPermitRootLogin {value}\n");
    if let Err(e) = root(host, &["mkdir", "-p", "/etc/ssh/sshd_config.d"]).and_then(|_| host.write_file_privileged(ROOT_DROPIN, &content).map_err(|e| e.to_string())) {
        fail!("✕ Couldn't write {ROOT_DROPIN}: {e}. Nothing was changed.");
    }
    log.push(format!("✓ Wrote {ROOT_DROPIN}."));
    if let Err(e) = root(host, &["sshd", "-t"]) {
        log.push(format!("✕ sshd rejected the change: {e}"));
        undo(&mut log, false);
        return Err(log);
    }
    log.push("✓ sshd -t accepts it.".into());
    match root(host, &["sshd", "-T"]).map(|o| effective_root_login(&o)) {
        Ok(Some(v)) if v == value || (value == "prohibit-password" && v == "without-password") => log.push(format!("✓ sshd will use PermitRootLogin {value}.")),
        Ok(other) => {
            log.push(format!("✕ Something earlier in the config still sets PermitRootLogin ({}).", other.unwrap_or_else(|| "unreadable".into())));
            undo(&mut log, false);
            return Err(log);
        }
        Err(e) => {
            log.push(format!("✕ Couldn't check the result: {e}"));
            undo(&mut log, false);
            return Err(log);
        }
    }
    if let Err(e) = root(host, &["sh", "-c", RELOAD_SSHD]) {
        log.push(format!("✕ Couldn't reload sshd: {e}"));
        undo(&mut log, false);
        return Err(log);
    }
    log.push("✓ Reloaded sshd.".into());
    if let Err(e) = fresh_login() {
        log.push(format!("✕ A fresh key login failed after the reload ({e})."));
        undo(&mut log, true);
        return Err(log);
    }
    log.push(format!("✓ A fresh key login still works. PermitRootLogin is {value}."));
    Ok(log)
}

/// Which firewall Crow can turn on here.
fn firewall_tool(host: &dyn Host) -> Option<&'static str> {
    let has = |t: &str| host.exec(&["sh", "-c", &format!("PATH=\"$PATH:/usr/sbin:/sbin\" command -v {t}")], DEFAULT_TIMEOUT).is_ok();
    if has("ufw") {
        Some("ufw")
    } else if has("firewall-cmd") {
        Some("firewalld")
    } else {
        None
    }
}

/// The commands that turn `tool` on allowing `ports`, and the ones that
/// undo it.
pub fn firewall_commands(tool: &str, ports: &[u16]) -> (Vec<String>, Vec<String>) {
    match tool {
        "ufw" => {
            let mut on: Vec<String> = ports.iter().map(|p| format!("ufw allow {p}/tcp")).collect();
            on.push("ufw --force enable".into());
            (on, vec!["ufw --force disable".into()])
        }
        _ => {
            let mut on = vec!["systemctl enable --now firewalld".to_string()];
            on.extend(ports.iter().map(|p| format!("firewall-cmd --permanent --add-port={p}/tcp")));
            on.push("firewall-cmd --reload".into());
            (on, vec!["systemctl disable --now firewalld".into()])
        }
    }
}

/// Turns a firewall on that allows the SSH ports and `also`; undone if a
/// fresh login fails afterwards.
pub fn enable_firewall(host: &dyn Host, ssh_ports: &[u16], also: &[u16], fresh_login: &dyn Fn() -> Result<(), String>) -> Result<Vec<String>, Vec<String>> {
    let mut log = Vec::new();
    if let Err(e) = fresh_login() {
        log.push(format!("✕ A fresh key login doesn't work ({e}). Nothing was changed."));
        return Err(log);
    }
    log.push("✓ A fresh key login works.".into());
    let Some(tool) = firewall_tool(host) else {
        log.push("✕ Neither ufw nor firewalld is installed. Install one (apt install ufw, dnf install firewalld), then try again. Nothing was changed.".into());
        return Err(log);
    };
    let mut ports: Vec<u16> = if ssh_ports.is_empty() { vec![22] } else { ssh_ports.to_vec() };
    ports.extend(also.iter().copied().filter(|p| !ports.contains(p)).collect::<Vec<_>>());
    let (on, off) = firewall_commands(tool, &ports);
    let undo = |log: &mut Vec<String>| {
        let ok = off.iter().all(|c| root(host, &["sh", "-c", &format!("PATH=\"$PATH:/usr/sbin:/sbin\"; {c}")]).is_ok());
        log.push(if ok { format!("↺ Undone: {}.", off.join("; ")) } else { format!("⚠ Undo didn't finish: run {} by hand.", off.join("; ")) });
    };
    for c in &on {
        if let Err(e) = root(host, &["sh", "-c", &format!("PATH=\"$PATH:/usr/sbin:/sbin\"; {c}")]) {
            log.push(format!("✕ {c}: {e}"));
            undo(&mut log);
            return Err(log);
        }
        log.push(format!("✓ {c}"));
    }
    if let Err(e) = fresh_login() {
        log.push(format!("✕ A fresh key login failed with the firewall on ({e})."));
        undo(&mut log);
        return Err(log);
    }
    log.push(format!("✓ A fresh key login still works. {tool} is on, allowing {}.", ports.iter().map(|p| format!("{p}/tcp")).collect::<Vec<_>>().join(", ")));
    Ok(log)
}

/// The fleet's posture as Markdown: found, accepted, and what was fixed
/// (`fixed`: change records' (server, what, when)).
pub fn report_markdown(rows: &[(String, Option<Posture>, Vec<(Finding, Option<Accepted>)>)], fixed: &[(String, String, String)], now: &str) -> String {
    let mut md = format!("# Fleet security posture\n\nGenerated by Crow, {now}.\n\n");
    let open: usize = rows.iter().map(|(_, _, f)| f.iter().filter(|(_, a)| a.is_none()).count()).sum();
    let accepted: usize = rows.iter().map(|(_, _, f)| f.iter().filter(|(_, a)| a.is_some()).count()).sum();
    md.push_str(&format!("- Servers: {}\n- Open findings: {open}\n- Accepted risks: {accepted}\n- Fixes applied: {}\n\n", rows.len(), fixed.len()));
    md.push_str("## Findings\n\n| Server | Finding | Status |\n|---|---|---|\n");
    for (server, posture, found) in rows {
        if posture.is_none() {
            md.push_str(&format!("| {server} | not checked (unreachable, or not read yet) | — |\n"));
            continue;
        }
        if found.is_empty() {
            md.push_str(&format!("| {server} | nothing found | ok |\n"));
        }
        for (f, a) in found {
            let status = match a {
                Some(a) => format!("accepted by {} ({}): {}", a.by, a.at.get(..10).unwrap_or(&a.at), a.reason.replace('|', "/")),
                None if f.critical() => "**open, critical**".into(),
                None => "open".into(),
            };
            md.push_str(&format!("| {server} | {} | {status} |\n", f.describe()));
        }
    }
    md.push_str("\n## Fixes applied\n\n");
    if fixed.is_empty() {
        md.push_str("None yet.\n");
    }
    for (server, what, when) in fixed {
        md.push_str(&format!("- {when} · {server}: {what}\n"));
    }
    md.push_str("\nRoot login, password login and the firewall come from `sshd -T` and the firewall's own status, read as root where Crow could. A finding Crow couldn't read is left out, never assumed safe.\n");
    md
}

#[cfg(test)]
mod tests {
    use super::{effective_root_login, findings, firewall_commands, is_accepted, report_markdown, set_root_login, Accepted, Finding};
    use crate::topology::posture::Posture;

    fn posture(root: &str, pw: bool, fw: Option<&str>, exposed: Vec<u16>) -> Posture {
        Posture { checked_at: 1, root_login: Some(root.into()), password_auth: Some(pw), ssh_ports: vec![22], firewall: fw.map(String::from), exposed_ports: exposed }
    }

    #[test]
    fn findings_come_from_what_was_read() {
        let f = findings(&posture("yes", true, Some(""), vec![22, 5432]));
        assert_eq!(f, vec![Finding::RootPasswordLogin, Finding::PasswordLogin, Finding::NoFirewall(vec![5432])]);
        assert!(f[0].critical() && f[2].critical());
        assert_eq!(findings(&posture("no", false, Some("ufw"), vec![22])), vec![]);
        let unknown = Posture { root_login: None, password_auth: None, firewall: None, ..Default::default() };
        assert!(findings(&unknown).is_empty(), "unreadable is not a finding");
        assert!(Finding::RootKeyLogin("prohibit-password".into()).fix("root").contains("not \"no\""));
        assert!(!Finding::RootKeyLogin("prohibit-password".into()).fixable("root"));
        assert!(Finding::RootKeyLogin("prohibit-password".into()).fixable("ops"));
    }

    #[test]
    fn accepted_risks_match_by_server_and_code() {
        let a = vec![Accepted { server_id: "s1".into(), code: "password-login".into(), reason: "legacy app".into(), by: "me".into(), at: "2026-10-09T00:00:00Z".into() }];
        assert!(is_accepted(&a, "s1", &Finding::PasswordLogin));
        assert!(!is_accepted(&a, "s2", &Finding::PasswordLogin));
        let md = report_markdown(&[("s1".into(), Some(posture("no", true, Some("ufw"), vec![])), vec![(Finding::PasswordLogin, Some(a[0].clone()))]), ("s2".into(), None, vec![])], &[("s3".into(), "PermitRootLogin prohibit-password".into(), "2026-10-09".into())], "2026-10-09");
        assert!(md.contains("accepted by me (2026-10-09): legacy app") && md.contains("not checked") && md.contains("Fixes applied: 1"));
    }

    #[test]
    fn the_firewall_allows_ssh_before_it_turns_on() {
        let (on, off) = firewall_commands("ufw", &[2222, 443]);
        assert_eq!(on, ["ufw allow 2222/tcp", "ufw allow 443/tcp", "ufw --force enable"]);
        assert_eq!(off, ["ufw --force disable"]);
        let (on, _) = firewall_commands("firewalld", &[22]);
        assert!(on[1].contains("--add-port=22/tcp") && on.last().unwrap().contains("--reload"));
        assert_eq!(effective_root_login("port 22\npermitrootlogin without-password\n").as_deref(), Some("without-password"));
    }

    #[test]
    fn root_login_no_is_refused_when_crow_is_root() {
        let host = crate::host::LocalHost;
        let r = set_root_login(&host, "no", "root", &|| Ok(())).unwrap_err();
        assert!(r[0].contains("lock Crow out"));
        let r = set_root_login(&host, "no", "ops", &|| Err("refused".into())).unwrap_err();
        assert!(r[0].contains("Nothing was changed"));
    }
}

/// Against a real sshd (a throwaway alpine container on 127.0.0.1:22401
/// with the key in CROW_HARDEN_KEY):
///   cargo test live_root_login_fix -- --ignored --nocapture
#[cfg(test)]
#[test]
#[ignore]
fn live_root_login_fix() {
    let key = std::env::var("CROW_HARDEN_KEY").expect("set CROW_HARDEN_KEY");
    let host = crate::host::ContainerHost::new("podman", "crow-harden");
    let login = || -> Result<(), String> {
        let out = std::process::Command::new("ssh")
            .args(["-F", "/dev/null", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null", "-o", "LogLevel=ERROR", "-i", &key, "-p", "22401", "root@127.0.0.1", "true"])
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
    };
    let effective = || effective_root_login(&host.exec(&["sshd", "-T"], DEFAULT_TIMEOUT).unwrap().stdout);
    // Root with a password, from a later drop-in (the finding).
    host.exec(&["sh", "-c", "rm -f /etc/ssh/sshd_config.d/00-crow-root-login.conf; echo 'PermitRootLogin yes' > /etc/ssh/sshd_config.d/50-test.conf"], DEFAULT_TIMEOUT).unwrap();
    assert_eq!(effective().as_deref(), Some("yes"));
    let log = set_root_login(&host, "prohibit-password", "root", &login).expect("fixed");
    for l in &log {
        eprintln!("{l}");
    }
    assert!(matches!(effective().as_deref(), Some("prohibit-password" | "without-password")));
    // A lockout after the reload is undone: the drop-in goes, "yes" is back.
    let calls = std::sync::atomic::AtomicUsize::new(0);
    host.exec(&["rm", "-f", ROOT_DROPIN], DEFAULT_TIMEOUT).unwrap();
    let flaky = || if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 { Ok(()) } else { Err("Permission denied (publickey)".to_string()) };
    let log = set_root_login(&host, "prohibit-password", "root", &flaky).unwrap_err();
    eprintln!("{}", log.join("\n"));
    assert!(log.last().unwrap().contains("Undone"));
    assert_eq!(effective().as_deref(), Some("yes"));
    host.exec(&["rm", "-f", "/etc/ssh/sshd_config.d/50-test.conf"], DEFAULT_TIMEOUT).unwrap();
}
