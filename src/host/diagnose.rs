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
/// [`diagnose_state`], plus what an agent that asks for approval (1Password,
/// Secretive) means for it (ERR-87).
pub fn diagnose(server: &ServerRecord, state: &ConnectionState, probe: Option<&ProbeResult>, key: &KeyTried, jump: Option<&str>, approval_agent: Option<&str>) -> Diagnosis {
    let mut d = diagnose_state(server, state, probe, key, jump);
    if let (Some(agent), false) = (approval_agent, matches!(state, ConnectionState::Connected)) {
        d.facts.push(("agent".into(), t!("host-diagnose-asks-to-approve-each-use", agent = agent)));
        d.steps.insert(
            0,
            t!("host-diagnose-is-the-ssh-agent-here", agent = agent),
        );
        d.steps.push(t!("host-diagnose-for-no-prompts-at-all", agent = agent));
    }
    d
}

pub fn diagnose_state(server: &ServerRecord, state: &ConnectionState, probe: Option<&ProbeResult>, key: &KeyTried, jump: Option<&str>) -> Diagnosis {
    let detail = state.detail().unwrap_or_default().to_string();
    let addr = format!("{}@{}:{}", server.login_user, server.host, if server.port == 0 { 22 } else { server.port });
    let mut facts = vec![(t!("host-diagnose-ssh-said").to_string(), if detail.is_empty() { "—".into() } else { detail.clone() })];
    if let Some(j) = jump {
        facts.push(("through".into(), t!("host-diagnose-jump-host", j = j)));
    }
    let key_fact = match key {
        KeyTried::File { path, exists: true } => path.clone(),
        KeyTried::File { path, exists: false } => t!("host-diagnose-missing-on-this-machine", path = path),
        KeyTried::Agent => t!("host-diagnose-your-ssh-agent-ssh-config").into(),
    };

    match state {
        ConnectionState::Connected => Diagnosis {
            title: t!("host-diagnose-connected").into(),
            what: t!("host-diagnose-crow-reaches", addr = addr),
            facts,
            steps: Vec::new(),
            actions: Vec::new(),
        },
        ConnectionState::AuthFailed(_) => {
            let offered = offered_methods(&detail);
            facts.push((t!("host-diagnose-key-tried").into(), key_fact));
            facts.push((t!("host-diagnose-server-accepts").into(), if offered.is_empty() { "—".into() } else { offered.join(", ") }));
            let takes_password = offered.iter().any(|m| m == "password" || m == "keyboard-interactive");
            let mut steps = Vec::new();
            if let KeyTried::File { path, exists: false } = key {
                steps.push(t!("host-diagnose-the-key-file-isnt-on", path = path));
            } else {
                steps.push(t!("host-diagnose-the-server-doesnt-accept-that", login_user = server.login_user));
            }
            if !offered.iter().any(|m| m == "publickey") && !offered.is_empty() {
                steps.push(t!("host-diagnose-the-server-doesnt-offer-key").into());
            }
            if takes_password {
                steps.push(t!("host-diagnose-it-accepts-a-password-log").into());
            }
            let mut actions = Vec::new();
            if takes_password {
                actions.push(RecoveryAction::InstallKeyWithPassword);
            }
            actions.push(RecoveryAction::Retry);
            Diagnosis { title: t!("host-diagnose-login-refused").into(), what: t!("host-diagnose-answered-but-refused-crows-login", addr = addr), facts, steps, actions }
        }
        ConnectionState::HostKeyRejected(_) => {
            let pinned = server.host_key_fingerprint.clone().filter(|f| !f.trim().is_empty()).unwrap_or_else(|| "—".into());
            facts.push(("pinned".into(), pinned));
            let now = probe.map(|p| p.host_key_fingerprint.clone()).filter(|f| !f.is_empty());
            facts.push((t!("host-diagnose-presents-now").into(), now.clone().unwrap_or_else(|| if jump.is_some() { t!("host-diagnose-cant-fetch-through-a-jump").into() } else { "—".into() })));
            let changed = probe.is_some_and(|p| p.host_key_mismatch);
            let (title, what, mut steps) = if changed {
                (
                    t!("host-diagnose-host-key-changed"),
                    t!("host-diagnose-presents-a-different-host-key", addr = addr),
                    vec![
                        t!("host-diagnose-dont-trust-the-new-key").to_string(),
                        t!("host-diagnose-check-it-on-the-server").to_string(),
                    ],
                )
            } else {
                (
                    t!("host-diagnose-host-key-not-trusted"),
                    t!("host-diagnose-ssh-known-hosts-has-no", addr = addr),
                    vec![t!("host-diagnose-compare-the-fingerprint-with-the").to_string()],
                )
            };
            if jump.is_some() {
                steps.push(t!("host-diagnose-this-server-is-behind-a").into());
            }
            let mut actions = Vec::new();
            if now.is_some() && jump.is_none() {
                steps.push(t!("host-diagnose-once-it-checks-out-trust").into());
                actions.push(RecoveryAction::TrustNewHostKey);
            }
            actions.push(RecoveryAction::Retry);
            Diagnosis { title: title.into(), what, facts, steps, actions }
        }
        ConnectionState::Unreachable(_) => {
            let lower = detail.to_lowercase();
            let (title, what, steps): (&str, String, Vec<String>) = if let Some(j) = jump.filter(|_| lower.contains("jump") || lower.contains("proxy") || lower.contains("kex_exchange") || lower.contains("closed by")) {
                (t!("host-diagnose-jump-host-failed"), t!("host-diagnose-the-connection-through-failed-before", j = j, addr = addr), vec![t!("host-diagnose-open-in-crow-if-its", j = j), t!("host-diagnose-check-the-jump-host-can").into()])
            } else if lower.contains("could not resolve") || probe.and_then(|p| p.error.as_deref()).is_some_and(|e| e.contains("could not resolve")) {
                (t!("host-diagnose-name-not-found"), t!("host-diagnose-doesnt-resolve-to-an-address", host = server.host), vec![t!("host-diagnose-check-the-spelling-or-use").into(), t!("host-diagnose-a-vpn-or-private-dns").into()])
            } else if lower.contains("refused") {
                (t!("host-diagnose-connection-refused"), t!("host-diagnose-answered-but-nothing-accepts-connections", host = server.host, server = if server.port == 0 { 22 } else { server.port }), vec![t!("host-diagnose-sshd-may-be-stopped-or").into(), t!("host-diagnose-a-firewall-rejecting-the-port").into()])
            } else if lower.contains("timed out") || lower.contains("timeout") {
                (t!("host-diagnose-timed-out"), t!("host-diagnose-didnt-answer-at-all", host = server.host), vec![t!("host-diagnose-the-server-may-be-off").into(), t!("host-diagnose-a-firewall-dropping-the-port").into()])
            } else if lower.contains("no route") || lower.contains("unreachable") {
                (t!("host-diagnose-no-route"), t!("host-diagnose-this-machine-has-no-route", host = server.host), vec![t!("host-diagnose-check-your-network-vpn-or").into()])
            } else {
                (t!("host-diagnose-unreachable"), t!("host-diagnose-crow-couldnt-open-a-connection", addr = addr), vec![t!("host-diagnose-try-again-if-it-persists").into()])
            };
            if let Some(p) = probe.filter(|_| jump.is_none()) {
                facts.push((t!("host-diagnose-direct-probe").into(), if p.is_reachable { t!("host-diagnose-port-open-ms", p = p.latency_ms.unwrap_or(0), p2 = p.ssh_banner.clone().unwrap_or_else(|| t!("host-diagnose-no-ssh-banner").into())) } else { p.error.clone().unwrap_or_else(|| t!("host-diagnose-no-answer").into()) }));
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
        let d = diagnose_state(&server(), &ConnectionState::AuthFailed("root@127.0.0.1: Permission denied (publickey,password).".into()), None, &key, None);
        assert_eq!(d.title, "LOGIN REFUSED");
        assert_eq!(d.actions, vec![RecoveryAction::InstallKeyWithPassword, RecoveryAction::Retry]);
        assert!(d.facts.contains(&("server accepts".into(), "publickey, password".into())));
        let d = diagnose_state(&server(), &ConnectionState::AuthFailed("Permission denied (publickey).".into()), None, &key, None);
        assert_eq!(d.actions, vec![RecoveryAction::Retry]);
    }

    #[test]
    fn a_missing_key_file_is_named() {
        let key = KeyTried::File { path: "~/.ssh/crow_old".into(), exists: false };
        let d = diagnose_state(&server(), &ConnectionState::AuthFailed("Permission denied (publickey).".into()), None, &key, None);
        assert!(d.steps[0].contains("isn't on this machine"));
    }

    #[test]
    fn changed_host_key_shows_both_and_never_offers_trust_without_a_fetch() {
        let st = ConnectionState::HostKeyRejected("Host key verification failed.".into());
        let d = diagnose_state(&server(), &st, Some(&probe(true, "SHA256:new (ED25519)")), &KeyTried::Agent, None);
        assert_eq!(d.title, "HOST KEY CHANGED");
        assert!(d.facts.contains(&("pinned".into(), "SHA256:old (ED25519)".into())));
        assert!(d.facts.contains(&("presents now".into(), "SHA256:new (ED25519)".into())));
        assert_eq!(d.actions, vec![RecoveryAction::TrustNewHostKey, RecoveryAction::Retry]);
        // Behind a jump host: nothing fetched, nothing to trust from here.
        let d = diagnose_state(&server(), &st, None, &KeyTried::Agent, Some("bastion"));
        assert_eq!(d.actions, vec![RecoveryAction::Retry]);
    }

    #[test]
    fn unreachable_kinds_are_told_apart() {
        let title = |detail: &str| diagnose_state(&server(), &ConnectionState::Unreachable(detail.into()), None, &KeyTried::Agent, None).title;
        assert_eq!(title("ssh: connect to host 10.0.0.9 port 22: Connection refused"), "CONNECTION REFUSED");
        assert_eq!(title("ssh: connect to host 10.0.0.9 port 22: Connection timed out"), "TIMED OUT");
        assert_eq!(title("ssh: Could not resolve hostname db-1: Name or service not known"), "NAME NOT FOUND");
        assert_eq!(title("ssh: connect to host 10.9.9.9 port 22: No route to host"), "NO ROUTE");
        let d = diagnose_state(&server(), &ConnectionState::Unreachable("kex_exchange_identification: Connection closed by remote host".into()), None, &KeyTried::Agent, Some("bastion"));
        assert_eq!(d.title, "JUMP HOST FAILED");
    }

    #[test]
    fn an_approving_agent_is_explained_first_and_switching_keys_offered() {
        let st = ConnectionState::Unreachable("Connection timed out during banner exchange".into());
        let d = diagnose(&server(), &st, None, &KeyTried::Agent, None, Some("1Password"));
        assert!(d.steps[0].starts_with("1Password is the SSH agent here"));
        assert!(d.steps.last().unwrap().contains("switch this server to Crow's own key"));
        assert!(d.facts.iter().any(|(k, v)| k == "agent" && v.starts_with("1Password")));
        let plain = diagnose(&server(), &st, None, &KeyTried::Agent, None, None);
        assert!(!plain.steps.iter().any(|s| s.contains("approve")), "no agent, no mention");
        let ok = diagnose(&server(), &ConnectionState::Connected, None, &KeyTried::Agent, None, Some("1Password"));
        assert!(ok.steps.is_empty(), "nothing to explain when it works");
    }
}
