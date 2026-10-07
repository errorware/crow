//! A server's security posture as the topology map shows it (ERR-128):
//! what sshd really allows, whether a firewall filters incoming traffic,
//! and what listens on every interface. One read-only script, run as root
//! where Crow can (sshd -T and most firewalls need it), every few hours.

use crate::host::{Host, DEFAULT_TIMEOUT};

/// How often the fleet's posture is read again.
pub const POSTURE_EVERY_SECS: i64 = 6 * 3600;

const POSTURE: &str = r#"PATH="$PATH:/usr/sbin:/sbin"
echo "@@sshd"; sshd -T 2>/dev/null | grep -Ei '^(permitrootlogin|passwordauthentication|kbdinteractiveauthentication|port) '
echo "@@firewall"
if command -v ufw >/dev/null 2>&1 && ufw status 2>/dev/null | grep -q '^Status: active'; then echo ufw
elif command -v firewall-cmd >/dev/null 2>&1 && firewall-cmd --state >/dev/null 2>&1; then echo firewalld
elif command -v nft >/dev/null 2>&1 && nft list ruleset 2>/dev/null | grep -q 'hook input'; then echo nftables
elif command -v iptables >/dev/null 2>&1 && iptables -S INPUT 2>/dev/null | grep -qvx -- '-P INPUT ACCEPT'; then echo iptables
elif [ "$(id -u)" = 0 ]; then echo none
else echo unknown; fi
echo "@@listen"; ss -Hltn 2>/dev/null | awk '{print $4}'
true"#;

/// What a posture check found. `None` fields weren't readable.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Posture {
    pub checked_at: i64,
    /// sshd's effective `permitrootlogin` (yes, no, prohibit-password, …).
    pub root_login: Option<String>,
    pub password_auth: Option<bool>,
    pub ssh_ports: Vec<u16>,
    /// The active firewall (ufw, firewalld, nftables, iptables), `Some("")`
    /// when Crow (as root) found none, `None` when it couldn't tell.
    pub firewall: Option<String>,
    /// TCP ports listening on every interface (0.0.0.0, [::], *).
    pub exposed_ports: Vec<u16>,
}

impl Posture {
    /// Root can log in with a password (`yes`), the worst setting.
    pub fn root_password_login(&self) -> bool {
        self.root_login.as_deref() == Some("yes")
    }

    /// Root can log in at all over SSH (with a key or a password).
    pub fn root_login_allowed(&self) -> bool {
        self.root_login.as_deref().is_some_and(|v| v != "no")
    }

    pub fn no_firewall(&self) -> bool {
        self.firewall.as_deref() == Some("")
    }

    /// Exposed ports other than SSH, when nothing filters them.
    pub fn unfiltered_ports(&self) -> Vec<u16> {
        if !self.no_firewall() {
            return Vec::new();
        }
        self.exposed_ports.iter().copied().filter(|p| !self.ssh_ports.contains(p)).collect()
    }
}

/// The local-address column of `ss -Hltn` → port, when it listens on every
/// interface (`0.0.0.0:22`, `[::]:22`, `*:22`).
fn exposed_port(local: &str) -> Option<u16> {
    let (addr, port) = local.rsplit_once(':')?;
    let addr = addr.split('%').next().unwrap_or(addr);
    matches!(addr, "0.0.0.0" | "[::]" | "*" | "::").then(|| port.parse().ok()).flatten()
}

pub fn parse(stdout: &str, checked_at: i64) -> Posture {
    let mut p = Posture { checked_at, ..Default::default() };
    let mut section = "";
    for line in stdout.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(s) = line.strip_prefix("@@") {
            section = if s == "sshd" { "sshd" } else if s == "firewall" { "firewall" } else { "listen" };
            continue;
        }
        match section {
            "sshd" => {
                let (key, value) = line.split_once(' ').unwrap_or((line, ""));
                match key.to_lowercase().as_str() {
                    "permitrootlogin" => p.root_login = Some(value.trim().to_lowercase()),
                    "passwordauthentication" => p.password_auth = Some(value.trim().eq_ignore_ascii_case("yes")),
                    "port" => p.ssh_ports.extend(value.trim().parse::<u16>().ok()),
                    _ => {}
                }
            }
            "firewall" => {
                p.firewall = match line {
                    "none" => Some(String::new()),
                    "unknown" => None,
                    fw => Some(fw.to_string()),
                }
            }
            _ => p.exposed_ports.extend(exposed_port(line)),
        }
    }
    p.exposed_ports.sort_unstable();
    p.exposed_ports.dedup();
    p
}

/// Reads `host`'s posture, as root when Crow can. `None` when it couldn't
/// run at all.
pub fn read_posture(host: &dyn Host, now: i64) -> Option<Posture> {
    let argv = ["sh", "-c", POSTURE];
    host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT).or_else(|_| host.exec(&argv, DEFAULT_TIMEOUT)).ok().map(|o| parse(&o.stdout, now))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sshd_firewall_and_what_listens_everywhere() {
        let out = "@@sshd\nport 22\npermitrootlogin yes\npasswordauthentication yes\nkbdinteractiveauthentication no\n@@firewall\nnone\n@@listen\n0.0.0.0:22\n[::]:22\n127.0.0.1:5432\n0.0.0.0:80\n*:9100\n[::1]:631\n";
        let p = parse(out, 1);
        assert_eq!(p.root_login.as_deref(), Some("yes"));
        assert!(p.root_password_login() && p.root_login_allowed());
        assert_eq!(p.password_auth, Some(true));
        assert_eq!(p.ssh_ports, [22]);
        assert!(p.no_firewall());
        assert_eq!(p.exposed_ports, [22, 80, 9100], "loopback-only ports aren't exposed");
        assert_eq!(p.unfiltered_ports(), [80, 9100], "ssh itself is expected");
    }

    #[test]
    fn real_ubuntu_26_04_output() {
        // A fresh Multipass VM, read over SSH with sudo, 2026-10-07.
        let out = "@@sshd\nport 22\npermitrootlogin prohibit-password\npasswordauthentication no\nkbdinteractiveauthentication no\n@@firewall\nnone\n@@listen\n127.0.0.53%lo:53\n127.0.0.54:53\n0.0.0.0:22\n[::]:22\n";
        let p = parse(out, 1);
        assert_eq!(p.exposed_ports, [22], "resolved's loopback listeners aren't exposed");
        assert!(p.no_firewall() && p.unfiltered_ports().is_empty());
        assert!(p.root_login_allowed() && !p.root_password_login());
        assert_eq!(p.password_auth, Some(false));
    }

    #[test]
    fn a_firewall_or_an_unknown_one_raises_nothing() {
        let p = parse("@@sshd\npermitrootlogin prohibit-password\n@@firewall\nufw\n@@listen\n0.0.0.0:80\n", 1);
        assert!(p.root_login_allowed() && !p.root_password_login());
        assert!(p.unfiltered_ports().is_empty());
        let unknown = parse("@@sshd\n@@firewall\nunknown\n@@listen\n0.0.0.0:80\n", 1);
        assert_eq!(unknown.firewall, None);
        assert!(unknown.unfiltered_ports().is_empty(), "not root: can't say there's no firewall");
        assert_eq!(unknown.root_login, None, "sshd -T needs root");
    }
}
