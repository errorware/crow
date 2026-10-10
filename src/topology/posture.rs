//! A server's security posture as the topology map shows it (ERR-128):
//! what sshd really allows, whether a firewall filters incoming traffic,
//! and what listens on every interface. One read-only script, run as root
//! where Crow can (sshd -T and most firewalls need it), every few hours.

use crate::host::{Host, DEFAULT_TIMEOUT};

/// How often the fleet's posture is read again.
pub const POSTURE_EVERY_SECS: i64 = 6 * 3600;

/// Reads `nft list ruleset` on stdin; succeeds when an input-hook chain
/// filters by itself: policy drop, a drop/reject rule, or a jump to a chain
/// other than ufw's (ufw-, ufw6-: they do nothing while ufw is off). A chain that only
/// exists, as iptables-nft and a disabled ufw leave behind, isn't a firewall.
const NFT_FILTERS: &str = r#"/hook input/{i=1} i&&/policy drop/{f=1} i&&/(^|[ \t])(drop|reject)([ \t;]|$)/{f=1} i&&/jump /&&!/jump ufw6?-/{f=1} /^[ \t]*}/{i=0} END{exit !f}"#;

/// The same for `iptables -S INPUT`.
const IPT_FILTERS: &str = r#"/^-P INPUT (DROP|REJECT)/{f=1} /^-A INPUT/&&/-j (DROP|REJECT)/{f=1} /^-A INPUT/&&/-j /&&!/-j (ufw6?-|ACCEPT|RETURN|LOG)/{f=1} END{exit !f}"#;

const POSTURE: &str = r#"PATH="$PATH:/usr/sbin:/sbin"
echo "@@sshd"; sshd -T 2>/dev/null | grep -Ei '^(permitrootlogin|passwordauthentication|kbdinteractiveauthentication|port) '
echo "@@firewall"
if command -v ufw >/dev/null 2>&1 && ufw status 2>/dev/null | grep -q '^Status: active'; then echo ufw
elif command -v firewall-cmd >/dev/null 2>&1 && firewall-cmd --state >/dev/null 2>&1; then echo firewalld
elif command -v nft >/dev/null 2>&1 && nft list ruleset 2>/dev/null | awk "$1"; then echo nftables
elif command -v iptables >/dev/null 2>&1 && iptables -S INPUT 2>/dev/null | awk "$2"; then echo iptables
elif [ "$(id -u)" = 0 ]; then echo none
else echo unknown; fi
echo "@@tools"; for t in ufw firewall-cmd; do command -v "$t" >/dev/null 2>&1 && echo "$t"; done
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
    /// Firewall tools Crow can turn on here (ufw, firewall-cmd), installed
    /// whether or not they're running.
    pub firewall_tools: Vec<String>,
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
            section = match s {
                "sshd" => "sshd",
                "firewall" => "firewall",
                "tools" => "tools",
                _ => "listen",
            };
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
            "tools" => p.firewall_tools.push(line.to_string()),
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
    let argv = ["sh", "-c", POSTURE, "crow-posture", NFT_FILTERS, IPT_FILTERS];
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
        assert!(p.firewall_tools.is_empty(), "no @@tools section: none known");
        let with_tools = parse("@@firewall\nnone\n@@tools\nufw\n@@listen\n0.0.0.0:22\n", 1);
        assert_eq!((with_tools.firewall_tools, with_tools.exposed_ports), (vec!["ufw".to_string()], vec![22]));
    }

    /// Runs a filter program over `input` the way the posture script does.
    fn filters(program: &str, input: &str) -> bool {
        use crate::host::Host;
        crate::host::LocalHost.exec_stdin(&["sh", "-c", "awk \"$1\"", "t", program], input.as_bytes(), DEFAULT_TIMEOUT).is_ok()
    }

    #[test]
    fn a_disabled_ufw_or_an_empty_chain_is_not_a_firewall() {
        // Ubuntu 24.04 after `ufw disable` (Linode, 2026-10-10): the input
        // chain is still there, policy accept, jumping only to ufw's chains.
        let disabled = "table ip filter {\n\tchain ufw-reject-input {\n\t\tcounter reject\n\t}\n\tchain INPUT {\n\t\ttype filter hook input priority filter; policy accept;\n\t\tcounter packets 6841 bytes 840787 jump ufw-before-logging-input\n\t\tcounter packets 3909 bytes 495741 jump ufw-reject-input\n\t}\n}\n";
        assert!(!filters(NFT_FILTERS, disabled));
        let disabled6 = "table ip6 filter {\n\tchain INPUT {\n\t\ttype filter hook input priority filter; policy accept;\n\t\tcounter packets 100 bytes 2022810 jump ufw6-before-input\n\t}\n}\n";
        assert!(!filters(NFT_FILTERS, disabled6), "and its IPv6 chains");
        let enabled = "table ip filter {\n\tchain INPUT {\n\t\ttype filter hook input priority filter; policy drop;\n\t\tjump ufw-before-input\n\t}\n}\n";
        assert!(filters(NFT_FILTERS, enabled));
        let own_rules = "table inet filter {\n\tchain input {\n\t\ttype filter hook input priority 0; policy accept;\n\t\ttcp dport 22 accept\n\t\tct state invalid drop\n\t}\n}\n";
        assert!(filters(NFT_FILTERS, own_rules));
        let custom_chain = "table inet filter {\n\tchain input {\n\t\ttype filter hook input priority 0; policy accept;\n\t\tjump my-rules\n\t}\n}\n";
        assert!(filters(NFT_FILTERS, custom_chain), "someone else's chain may filter: give it the benefit");
        let output_only = "table inet filter {\n\tchain output {\n\t\ttype filter hook output priority 0; policy drop;\n\t}\n}\n";
        assert!(!filters(NFT_FILTERS, output_only));

        assert!(!filters(IPT_FILTERS, "-P INPUT ACCEPT\n-A INPUT -j ufw-before-input\n-A INPUT -j ufw-reject-input\n"));
        assert!(filters(IPT_FILTERS, "-P INPUT DROP\n"));
        assert!(filters(IPT_FILTERS, "-P INPUT ACCEPT\n-A INPUT -p tcp --dport 23 -j REJECT\n"));
        assert!(!filters(IPT_FILTERS, "-P INPUT ACCEPT\n"));
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
