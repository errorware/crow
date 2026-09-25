//! Turning off SSH password login, without locking anyone out (ERR-34).
//!
//! Crow writes one drop-in, `/etc/ssh/sshd_config.d/00-crow-no-password.conf`.
//! sshd keeps the first value it reads for a keyword and reads the
//! `sshd_config.d` drop-ins in name order, so `00-` wins over, say,
//! a cloud image's `50-cloud-init.conf` that turns passwords on. Every step
//! is checked, and anything that fails is undone:
//!
//! 1. a fresh key login works (a new connection, not the one Crow holds);
//! 2. the drop-in is written and `sshd -t` accepts it;
//! 3. `sshd -T` confirms password login is now off;
//! 4. sshd reloads, and a fresh key login still works.
//!
//! Undoing is deleting the drop-in and reloading. Existing SSH sessions
//! (including Crow's own) survive a reload.

use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

pub const DROPIN: &str = "/etc/ssh/sshd_config.d/00-crow-no-password.conf";
pub const DROPIN_CONTENT: &str = "# Written by Crow: SSH password login is off.\n# Delete this file and reload sshd to turn it back on.\nPasswordAuthentication no\nKbdInteractiveAuthentication no\n";

const RELOAD: &str = "systemctl reload ssh 2>/dev/null || systemctl reload sshd";
/// Accounts that have a usable password (not locked, not empty).
const PASSWORD_USERS: &str = r#"awk -F: '$2 != "" && $2 !~ /^[!*]/ {print $1}' /etc/shadow"#;

/// What Crow learns before offering to turn passwords off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preflight {
    /// Password login is on right now (`sshd -T`).
    pub password_login_on: bool,
    /// The main config reads `sshd_config.d`, so a drop-in can take effect.
    pub reads_dropins: bool,
    /// Accounts that could still log in with a password, and would lose it.
    pub password_users: Vec<String>,
}

/// `sshd -T` prints effective settings lowercased, one per line.
pub fn effective_password_login(sshd_t: &str) -> Option<bool> {
    sshd_t.lines().find_map(|l| {
        let mut parts = l.split_whitespace();
        (parts.next()? == "passwordauthentication").then(|| parts.next() == Some("yes"))
    })
}

/// Whether the main sshd_config includes the drop-in directory.
pub fn reads_dropins(main_config: &str) -> bool {
    main_config.lines().map(str::trim).filter(|l| !l.starts_with('#')).any(|l| {
        let mut parts = l.split_whitespace();
        parts.next().is_some_and(|k| k.eq_ignore_ascii_case("include")) && parts.any(|p| p.contains("sshd_config.d"))
    })
}

fn root(host: &dyn Host, argv: &[&str]) -> Result<String, String> {
    host.exec_privileged(argv, &[], DEFAULT_TIMEOUT).map(|o| o.stdout).map_err(|e| match e {
        HostError::Failed { stderr, .. } => stderr.trim().to_string(),
        other => other.to_string(),
    })
}

pub fn preflight(host: &dyn Host) -> Result<Preflight, String> {
    let effective = root(host, &["sshd", "-T"])?;
    let main = root(host, &["cat", "/etc/ssh/sshd_config"])?;
    let users = root(host, &["sh", "-c", PASSWORD_USERS]).unwrap_or_default();
    Ok(Preflight {
        password_login_on: effective_password_login(&effective).unwrap_or(true),
        reads_dropins: reads_dropins(&main),
        password_users: users.lines().map(str::trim).filter(|u| !u.is_empty()).map(str::to_string).collect(),
    })
}

/// Turns password login off, step by step; each line of the returned log
/// says what happened. On failure the log ends with what was undone.
/// `fresh_login` makes a brand-new key login and runs a no-op.
pub fn turn_off(host: &dyn Host, fresh_login: &dyn Fn() -> Result<(), String>) -> Result<Vec<String>, Vec<String>> {
    let mut log = Vec::new();
    macro_rules! fail {
        ($($arg:tt)*) => {{
            log.push(format!($($arg)*));
            return Err(log);
        }};
    }
    if let Err(e) = fresh_login() {
        fail!("✕ A fresh key login doesn't work ({e}). Nothing was changed: turning passwords off now could lock you out.");
    }
    log.push("✓ A fresh key login works.".into());

    let pre = match preflight(host) {
        Ok(p) => p,
        Err(e) => fail!("✕ Couldn't read sshd's settings: {e}. Nothing was changed."),
    };
    if !pre.password_login_on {
        log.push("✓ Password login is already off. Nothing to do.".into());
        return Ok(log);
    }
    if !pre.reads_dropins {
        fail!("✕ This sshd_config doesn't include sshd_config.d, so a drop-in wouldn't apply. Nothing was changed; set PasswordAuthentication to no in the sshd_config sheet instead.");
    }

    let undo = |log: &mut Vec<String>, reload: bool| {
        let removed = root(host, &["rm", "-f", "--", DROPIN]).is_ok();
        let reloaded = !reload || root(host, &["sh", "-c", RELOAD]).is_ok();
        log.push(if removed && reloaded { format!("↺ Undone: removed {DROPIN}{}.", if reload { " and reloaded sshd" } else { "" }) } else { format!("⚠ Undo didn't finish: remove {DROPIN} by hand and reload sshd.") });
    };

    if let Err(e) = root(host, &["mkdir", "-p", "/etc/ssh/sshd_config.d"]) {
        fail!("✕ Couldn't create /etc/ssh/sshd_config.d: {e}. Nothing was changed.");
    }
    if let Err(e) = host.write_file_privileged(DROPIN, DROPIN_CONTENT) {
        fail!("✕ Couldn't write {DROPIN}: {e}. Nothing was changed.");
    }
    log.push(format!("✓ Wrote {DROPIN}."));

    if let Err(e) = root(host, &["sshd", "-t"]) {
        log.push(format!("✕ sshd rejected the change: {e}"));
        undo(&mut log, false);
        return Err(log);
    }
    log.push("✓ sshd -t accepts it.".into());

    match root(host, &["sshd", "-T"]).map(|out| effective_password_login(&out)) {
        Ok(Some(false)) => log.push("✓ sshd will refuse passwords.".into()),
        Ok(_) => {
            let who = root(host, &["sh", "-c", "grep -ril '^ *passwordauthentication' /etc/ssh/sshd_config.d/ /etc/ssh/sshd_config 2>/dev/null"]).unwrap_or_default();
            log.push(format!("✕ Something else still turns password login on{}.", if who.trim().is_empty() { String::new() } else { format!(" (set in: {})", who.split_whitespace().collect::<Vec<_>>().join(", ")) }));
            undo(&mut log, false);
            return Err(log);
        }
        Err(e) => {
            log.push(format!("✕ Couldn't check the result: {e}"));
            undo(&mut log, false);
            return Err(log);
        }
    }

    if let Err(e) = root(host, &["sh", "-c", RELOAD]) {
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
    log.push("✓ A fresh key login still works. Password login is off.".into());
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::ExecOutput;
    use std::sync::Mutex;
    use std::time::Duration;

    /// A server simulated by command: records what ran, answers by rule.
    struct FakeSshd {
        ran: Mutex<Vec<String>>,
        dropin_written: Mutex<bool>,
        /// What `sshd -T` says once the drop-in is in place.
        effective_after: &'static str,
    }

    impl FakeSshd {
        fn new(effective_after: &'static str) -> Self {
            Self { ran: Mutex::new(Vec::new()), dropin_written: Mutex::new(false), effective_after }
        }
        fn ran(&self, needle: &str) -> bool {
            self.ran.lock().unwrap().iter().any(|c| c.contains(needle))
        }
    }

    impl Host for FakeSshd {
        fn label(&self) -> String {
            "fake".into()
        }
        fn exec_stdin(&self, argv: &[&str], _stdin: &[u8], _t: Duration) -> Result<ExecOutput, HostError> {
            let cmd = argv.join(" ");
            self.ran.lock().unwrap().push(cmd.clone());
            let out = |s: &str| Ok(ExecOutput { stdout: s.into(), stderr: String::new() });
            if cmd.contains("crow-write") {
                *self.dropin_written.lock().unwrap() = true;
                return out("");
            }
            if cmd.ends_with("sshd -T") {
                return out(if *self.dropin_written.lock().unwrap() { self.effective_after } else { "port 22\npasswordauthentication yes\n" });
            }
            if cmd.ends_with("cat /etc/ssh/sshd_config") {
                return out("Include /etc/ssh/sshd_config.d/*.conf\nPort 22\n");
            }
            if cmd.contains("/etc/shadow") {
                return out("deploy\n");
            }
            if cmd.contains(&format!("rm -f -- {DROPIN}")) {
                *self.dropin_written.lock().unwrap() = false;
            }
            out("")
        }
        fn write_file_atomic(&self, _path: &str, _content: &str) -> Result<(), HostError> {
            Err(HostError::Failed { status: 1, stderr: "permission denied".into() })
        }
    }

    #[test]
    fn parses_sshd_output_and_includes() {
        assert_eq!(effective_password_login("port 22\npasswordauthentication no\n"), Some(false));
        assert_eq!(effective_password_login("passwordauthentication yes"), Some(true));
        assert!(reads_dropins("# comment\nInclude /etc/ssh/sshd_config.d/*.conf\n"));
        assert!(!reads_dropins("#Include /etc/ssh/sshd_config.d/*.conf\nPort 22\n"));
    }

    #[test]
    fn preflight_lists_accounts_that_would_lose_passwords() {
        let host = FakeSshd::new("passwordauthentication no");
        let p = preflight(&host).unwrap();
        assert_eq!(p, Preflight { password_login_on: true, reads_dropins: true, password_users: vec!["deploy".into()] });
    }

    #[test]
    fn turns_passwords_off_when_every_check_passes() {
        let host = FakeSshd::new("passwordauthentication no");
        let log = turn_off(&host, &|| Ok(())).unwrap();
        assert!(log.last().unwrap().contains("Password login is off"));
        assert!(host.ran("systemctl reload ssh") && *host.dropin_written.lock().unwrap());
    }

    #[test]
    fn nothing_changes_without_a_working_key_login() {
        let host = FakeSshd::new("passwordauthentication no");
        let log = turn_off(&host, &|| Err("Permission denied (publickey)".into())).unwrap_err();
        assert!(log[0].contains("Nothing was changed"));
        assert!(host.ran.lock().unwrap().is_empty(), "not a single command ran");
    }

    #[test]
    fn an_override_elsewhere_is_reported_and_undone() {
        let host = FakeSshd::new("passwordauthentication yes");
        let log = turn_off(&host, &|| Ok(())).unwrap_err();
        assert!(log.iter().any(|l| l.contains("Something else still turns password login on")));
        assert!(log.last().unwrap().starts_with("↺ Undone"));
        assert!(!*host.dropin_written.lock().unwrap() && !host.ran("systemctl reload"), "removed, and sshd never reloaded");
    }

    #[test]
    fn a_failed_login_after_reload_rolls_back() {
        let host = FakeSshd::new("passwordauthentication no");
        let calls = std::cell::Cell::new(0);
        let log = turn_off(&host, &|| {
            calls.set(calls.get() + 1);
            if calls.get() == 1 { Ok(()) } else { Err("timeout".into()) }
        })
        .unwrap_err();
        assert!(log.last().unwrap().contains("removed") && log.last().unwrap().contains("reloaded sshd"));
        assert!(!*host.dropin_written.lock().unwrap());
    }
}
