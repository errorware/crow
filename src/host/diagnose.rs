//! Why a server can't be reached, and what to do about it (ERR-89).
//! GPUI-free: built from the transport's state, a fresh probe of the host,
//! and which key Crow tried.

use super::ConnectionState;
use crate::vault::ServerRecord;
use crate::views::onboard::probe::ProbeResult;

/// The key Crow logs in with, as far as this machine can tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyTried {
    /// A private key file; whether it exists here.
    File { path: String, exists: bool },
    /// The SSH agent and ~/.ssh/config decide.
    Agent,
}

/// What the recovery panel offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Log in once with the password to install Crow's key (onboarding's
    /// password bootstrap).
    InstallKeyWithPassword,
    /// Replace the known_hosts entry with the key the server presents now.
    TrustNewHostKey,
    /// Drop the held connection and try again.
    Retry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnosis {
    pub title: String,
    /// What happened, in a sentence or two.
    pub what: String,
    /// (label, value) facts behind it.
    pub facts: Vec<(String, String)>,
    /// What to check or do, in order.
    pub steps: Vec<String>,
    pub actions: Vec<RecoveryAction>,
}

/// Auth methods from "Permission denied (publickey,password)."
pub fn offered_methods(detail: &str) -> Vec<String> {
    let Some(start) = detail.rfind('(') else { return Vec::new() };
    let Some(end) = detail[start..].find(')') else { return Vec::new() };
    detail[start + 1..start + end].split(',').map(|m| m.trim().to_string()).filter(|m| !m.is_empty()).collect()
}

/// `probe` is a fresh direct probe of the host; `None` when there's a jump
/// host in the way (it can't be probed directly) or it hasn't run yet.
pub fn diagnose(server: &ServerRecord, state: &ConnectionState, probe: Option<&ProbeResult>, key: &KeyTried, jump: Option<&str>) -> Diagnosis {
    let detail = state.detail().unwrap_or_default().to_string();
    let addr = format!("{}@{}:{}", server.login_user, server.host, if server.port == 0 { 22 } else { server.port });
    let mut facts = vec![("ssh said".to_string(), if detail.is_empty() { "—".into() } else { detail.clone() })];
    if let Some(j) = jump {
        facts.push(("through".into(), format!("jump host {j}")));
    }
    let key_fact = match key {
        KeyTried::File { path, exists: true } => path.clone(),
        KeyTried::File { path, exists: false } => format!("{path} (missing on this machine)"),
        KeyTried::Agent => "your SSH agent / ~/.ssh/config".into(),
    };

    match state {
        ConnectionState::Connected => Diagnosis {
            title: "CONNECTED".into(),
            what: format!("Crow reaches {addr}."),
            facts,
            steps: Vec::new(),
            actions: Vec::new(),
        },
        ConnectionState::AuthFailed(_) => {
            let offered = offered_methods(&detail);
            facts.push(("key tried".into(), key_fact));
            facts.push(("server accepts".into(), if offered.is_empty() { "—".into() } else { offered.join(", ") }));
            let takes_password = offered.iter().any(|m| m == "password" || m == "keyboard-interactive");
            let mut steps = Vec::new();
            if let KeyTried::File { path, exists: false } = key {
                steps.push(format!("The key file {path} isn't on this machine any more: restore it, or switch this server to another key."));
            } else {
                steps.push(format!("The server doesn't accept that key for {}: it isn't in ~{}/.ssh/authorized_keys there, or sshd ignores the file (permissions, AuthorizedKeysFile).", server.login_user, server.login_user));
            }
            if !offered.iter().any(|m| m == "publickey") && !offered.is_empty() {
                steps.push("The server doesn't offer key login at all (PubkeyAuthentication no): Crow only logs in with keys.".into());
            }
            if takes_password {
                steps.push("It accepts a password: log in once with it and Crow installs its key, without storing the password.".into());
            }
            let mut actions = Vec::new();
            if takes_password {
                actions.push(RecoveryAction::InstallKeyWithPassword);
            }
            actions.push(RecoveryAction::Retry);
            Diagnosis { title: "LOGIN REFUSED".into(), what: format!("{addr} answered, but refused Crow's login."), facts, steps, actions }
        }
        ConnectionState::HostKeyRejected(_) => {
            let pinned = server.host_key_fingerprint.clone().filter(|f| !f.trim().is_empty()).unwrap_or_else(|| "—".into());
            facts.push(("pinned".into(), pinned));
            let now = probe.map(|p| p.host_key_fingerprint.clone()).filter(|f| !f.is_empty());
            facts.push(("presents now".into(), now.clone().unwrap_or_else(|| if jump.is_some() { "can't fetch through a jump host".into() } else { "—".into() })));
            let changed = probe.is_some_and(|p| p.host_key_mismatch);
            let (title, what, mut steps) = if changed {
                (
                    "HOST KEY CHANGED",
                    format!("{addr} presents a different host key than the one Crow trusted. Either the server was reinstalled or its keys regenerated, or something between you and it is pretending to be it."),
                    vec![
                        "Don't trust the new key until you know why it changed.".to_string(),
                        format!("Check it on the server itself (console, provider's web shell): ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub, and compare with \"presents now\"."),
                    ],
                )
            } else {
                (
                    "HOST KEY NOT TRUSTED",
                    format!("~/.ssh/known_hosts has no key for {addr}, so ssh refuses to connect (strict checking)."),
                    vec!["Compare the fingerprint with the one on the server's console before trusting it.".to_string()],
                )
            };
            if jump.is_some() {
                steps.push("This server is behind a jump host, so Crow can't fetch its key directly: verify on the server, then add it to known_hosts yourself.".into());
            }
            let mut actions = Vec::new();
            if now.is_some() && jump.is_none() {
                steps.push("Once it checks out: trust the new key (you type the server's name to confirm).".into());
                actions.push(RecoveryAction::TrustNewHostKey);
            }
            actions.push(RecoveryAction::Retry);
            Diagnosis { title: title.into(), what, facts, steps, actions }
        }
        ConnectionState::Unreachable(_) => {
            let lower = detail.to_lowercase();
            let (title, what, steps): (&str, String, Vec<String>) = if let Some(j) = jump.filter(|_| lower.contains("jump") || lower.contains("proxy") || lower.contains("kex_exchange") || lower.contains("closed by")) {
                ("JUMP HOST FAILED", format!("The connection through {j} failed before reaching {addr}."), vec![format!("Open {j} in Crow: if it's unreachable too, fix that first."), "Check the jump host can reach this server's address and port.".into()])
            } else if lower.contains("could not resolve") || probe.and_then(|p| p.error.as_deref()).is_some_and(|e| e.contains("could not resolve")) {
                ("NAME NOT FOUND", format!("{} doesn't resolve to an address from this machine.", server.host), vec!["Check the spelling, or use the IP address.".into(), "A VPN or private DNS may be needed for internal names.".into()])
            } else if lower.contains("refused") {
                ("CONNECTION REFUSED", format!("{} answered, but nothing accepts connections on port {}.", server.host, if server.port == 0 { 22 } else { server.port }), vec!["sshd may be stopped, or listening on another port.".into(), "A firewall rejecting the port looks the same.".into()])
            } else if lower.contains("timed out") || lower.contains("timeout") {
                ("TIMED OUT", format!("{} didn't answer at all.", server.host), vec!["The server may be off or rebooting; check it at its provider.".into(), "A firewall dropping the port, or a wrong address, looks the same.".into()])
            } else if lower.contains("no route") || lower.contains("unreachable") {
                ("NO ROUTE", format!("This machine has no route to {}.", server.host), vec!["Check your network, VPN, or the address.".into()])
            } else {
                ("UNREACHABLE", format!("Crow couldn't open a connection to {addr}."), vec!["Try again; if it persists, ssh to it from a terminal for the full error.".into()])
            };
            if let Some(p) = probe.filter(|_| jump.is_none()) {
                facts.push(("direct probe".into(), if p.is_reachable { format!("port open ({}ms) · {}", p.latency_ms.unwrap_or(0), p.ssh_banner.clone().unwrap_or_else(|| "no ssh banner".into())) } else { p.error.clone().unwrap_or_else(|| "no answer".into()) }));
            }
            Diagnosis { title: title.into(), what, facts, steps, actions: vec![RecoveryAction::Retry] }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> ServerRecord {
        ServerRecord { name: "rails".into(), host: "127.0.0.1".into(), port: 2222, login_user: "root".into(), host_key_fingerprint: Some("SHA256:old (ED25519)".into()), ..Default::default() }
    }

    fn probe(mismatch: bool, fp: &str) -> ProbeResult {
        ProbeResult { is_reachable: true, latency_ms: Some(1), ssh_banner: Some("SSH-2.0-OpenSSH_9.6".into()), host_key_fingerprint: fp.into(), is_known_host: !mismatch, host_key_mismatch: mismatch, scanned_keys: vec![], error: None }
    }

    #[test]
    fn methods_from_permission_denied() {
        assert_eq!(offered_methods("root@127.0.0.1: Permission denied (publickey,password)."), vec!["publickey", "password"]);
        assert!(offered_methods("Permission denied").is_empty());
    }

    #[test]
    fn login_refused_offers_the_password_route_only_when_accepted() {
        let key = KeyTried::File { path: "~/.ssh/id_ed25519".into(), exists: true };
        let d = diagnose(&server(), &ConnectionState::AuthFailed("root@127.0.0.1: Permission denied (publickey,password).".into()), None, &key, None);
        assert_eq!(d.title, "LOGIN REFUSED");
        assert_eq!(d.actions, vec![RecoveryAction::InstallKeyWithPassword, RecoveryAction::Retry]);
        assert!(d.facts.contains(&("server accepts".into(), "publickey, password".into())));
        let d = diagnose(&server(), &ConnectionState::AuthFailed("Permission denied (publickey).".into()), None, &key, None);
        assert_eq!(d.actions, vec![RecoveryAction::Retry]);
    }

    #[test]
    fn a_missing_key_file_is_named() {
        let key = KeyTried::File { path: "~/.ssh/crow_old".into(), exists: false };
        let d = diagnose(&server(), &ConnectionState::AuthFailed("Permission denied (publickey).".into()), None, &key, None);
        assert!(d.steps[0].contains("isn't on this machine"));
    }

    #[test]
    fn changed_host_key_shows_both_and_never_offers_trust_without_a_fetch() {
        let st = ConnectionState::HostKeyRejected("Host key verification failed.".into());
        let d = diagnose(&server(), &st, Some(&probe(true, "SHA256:new (ED25519)")), &KeyTried::Agent, None);
        assert_eq!(d.title, "HOST KEY CHANGED");
        assert!(d.facts.contains(&("pinned".into(), "SHA256:old (ED25519)".into())));
        assert!(d.facts.contains(&("presents now".into(), "SHA256:new (ED25519)".into())));
        assert_eq!(d.actions, vec![RecoveryAction::TrustNewHostKey, RecoveryAction::Retry]);
        // Behind a jump host: nothing fetched, nothing to trust from here.
        let d = diagnose(&server(), &st, None, &KeyTried::Agent, Some("bastion"));
        assert_eq!(d.actions, vec![RecoveryAction::Retry]);
    }

    #[test]
    fn unreachable_kinds_are_told_apart() {
        let title = |detail: &str| diagnose(&server(), &ConnectionState::Unreachable(detail.into()), None, &KeyTried::Agent, None).title;
        assert_eq!(title("ssh: connect to host 10.0.0.9 port 22: Connection refused"), "CONNECTION REFUSED");
        assert_eq!(title("ssh: connect to host 10.0.0.9 port 22: Connection timed out"), "TIMED OUT");
        assert_eq!(title("ssh: Could not resolve hostname db-1: Name or service not known"), "NAME NOT FOUND");
        assert_eq!(title("ssh: connect to host 10.9.9.9 port 22: No route to host"), "NO ROUTE");
        let d = diagnose(&server(), &ConnectionState::Unreachable("kex_exchange_identification: Connection closed by remote host".into()), None, &KeyTried::Agent, Some("bastion"));
        assert_eq!(d.title, "JUMP HOST FAILED");
    }
}
