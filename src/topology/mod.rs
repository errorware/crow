//! The fleet as a graph (ERR-120): Crow on this machine, the servers it
//! manages (and the jump hosts it reaches them through), the local VMs and
//! lab containers, and the SSH keys that unlock them, with the risks Crow
//! can show from what it has actually read. Nothing on the map is guessed:
//! a risk comes from a server record, a key, an open alert or a posture
//! check (see [`posture`]).

pub mod layout;
pub mod posture;

use std::collections::HashMap;

use crate::metrics::alerts::Alert;
use crate::vault::{ServerRecord, SshKeyRecord};
use posture::Posture;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// Crow itself, on this machine.
    Crow,
    Server,
    /// A Multipass VM on this machine.
    Vm,
    /// A Podman/Docker lab container on this machine.
    Container,
    Key,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeKind {
    /// Crow manages it over SSH directly.
    Ssh,
    /// Crow reaches it through a jump host (from the jump host).
    ViaJump,
    /// This key logs Crow in.
    Unlocks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warn,
    Crit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Risk {
    pub level: Level,
    pub text: String,
    /// Where it comes from: "alert", "posture", "key", "server", "fleet".
    pub source: &'static str,
}

fn risk(level: Level, source: &'static str, text: impl Into<String>) -> Risk {
    Risk { level, text: text.into(), source }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Ok,
    Down,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// `crow`, `srv:<server id>` or `key:<key id>`.
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    /// The address (`user@host:port`) or the key's algorithm.
    pub detail: String,
    /// What it's grouped by on the map: the server's env, or "KEYS".
    pub lane: String,
    pub health: Health,
    pub risks: Vec<Risk>,
    pub server_id: Option<String>,
}

impl Node {
    pub fn worst(&self) -> Option<Level> {
        self.risks.iter().map(|r| r.level).max()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn edges_of<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| e.from == id || e.to == id)
    }
}

/// Everything the graph is built from, as Crow has it now.
pub struct Inputs<'a> {
    pub servers: &'a [ServerRecord],
    pub keys: &'a [SshKeyRecord],
    /// Open (unresolved) alerts.
    pub alerts: &'a [Alert],
    pub health: &'a HashMap<String, Health>,
    pub posture: &'a HashMap<String, Posture>,
    pub now: i64,
}

/// Keys older than this are flagged for rotation.
pub const KEY_MAX_AGE_DAYS: i64 = 90;

pub fn server_node_id(id: &str) -> String {
    format!("srv:{id}")
}

pub fn key_node_id(id: &str) -> String {
    format!("key:{id}")
}

fn kind_of(s: &ServerRecord) -> NodeKind {
    if crate::lab::multipass::vm_of(s).is_some() {
        NodeKind::Vm
    } else if s.tags.iter().any(|t| t == "test-node") {
        NodeKind::Container
    } else {
        NodeKind::Server
    }
}

/// A key's strength problem, from its public key: DSA is broken, RSA under
/// 3072 bits is below current guidance.
pub fn key_strength_risk(public_key: &str) -> Option<Risk> {
    let key = ssh_key::PublicKey::from_openssh(public_key.trim()).ok()?;
    match key.key_data() {
        ssh_key::public::KeyData::Dsa(_) => Some(risk(Level::Crit, "key", "DSA key: broken, and refused by current OpenSSH")),
        ssh_key::public::KeyData::Rsa(rsa) => {
            let bits = rsa.n.as_positive_bytes().map_or(0, |b| b.len() * 8);
            (bits < 3072).then(|| risk(Level::Warn, "key", format!("RSA {bits}-bit: under the 3072 bits current guidance asks for")))
        }
        _ => None,
    }
}

fn age_days(rfc3339: &str, now: i64) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(rfc3339).ok().map(|t| (now - t.timestamp()) / 86_400)
}

fn posture_risks(p: &Posture) -> Vec<Risk> {
    let mut r = Vec::new();
    if p.root_password_login() {
        r.push(risk(Level::Crit, "posture", "root can log in over SSH with a password (PermitRootLogin yes)"));
    } else if p.root_login_allowed() {
        r.push(risk(Level::Warn, "posture", format!("root can log in over SSH with a key (PermitRootLogin {})", p.root_login.as_deref().unwrap_or_default())));
    }
    if p.password_auth == Some(true) {
        r.push(risk(Level::Warn, "posture", "sshd accepts passwords (PasswordAuthentication yes)"));
    }
    let open = p.unfiltered_ports();
    if !open.is_empty() {
        let ports: Vec<String> = open.iter().map(u16::to_string).collect();
        r.push(risk(Level::Crit, "posture", format!("no firewall, and port{} {} open to every network", if open.len() == 1 { "" } else { "s" }, ports.join(", "))));
    } else if p.no_firewall() {
        r.push(risk(Level::Warn, "posture", "no firewall filters incoming traffic"));
    }
    r
}

pub fn build(i: &Inputs) -> Graph {
    let mut g = Graph::default();
    g.nodes.push(Node {
        id: "crow".into(),
        kind: NodeKind::Crow,
        label: "Crow".into(),
        detail: "this machine".into(),
        lane: "CROW".into(),
        health: Health::Ok,
        risks: Vec::new(),
        server_id: None,
    });

    let by_id: HashMap<&str, &ServerRecord> = i.servers.iter().map(|s| (s.id.as_str(), s)).collect();
    // How many servers depend on each jump host.
    let mut through: HashMap<&str, usize> = HashMap::new();
    for s in i.servers {
        if let Some(j) = s.jump_host_id.as_deref().filter(|j| by_id.contains_key(j)) {
            *through.entry(j).or_default() += 1;
        }
    }

    for s in i.servers {
        let mut risks = Vec::new();
        for a in i.alerts.iter().filter(|a| a.server_id == s.id && a.resolved_at.is_none()) {
            let level = if a.level == "CRIT" { Level::Crit } else { Level::Warn };
            risks.push(risk(level, "alert", if a.detail.is_empty() { a.kind.clone() } else { a.detail.clone() }));
        }
        if s.host_key_fingerprint.is_none() && kind_of(s) != NodeKind::Container {
            risks.push(risk(Level::Warn, "server", "host key not pinned: Crow never recorded which key this server should have"));
        }
        if s.auth_method == "password" {
            risks.push(risk(Level::Warn, "server", "Crow logs in with a password, not a key"));
        }
        if let Some(p) = i.posture.get(&s.id) {
            risks.extend(posture_risks(p));
        }
        if let Some(&n) = through.get(s.id.as_str()).filter(|n| **n >= 2) {
            risks.push(risk(Level::Warn, "fleet", format!("jump host for {n} servers: if it's down, Crow can't reach them")));
        }
        let addr = if s.login_user.is_empty() { format!("{}:{}", s.host, s.port) } else { format!("{}@{}:{}", s.login_user, s.host, s.port) };
        g.nodes.push(Node {
            id: server_node_id(&s.id),
            kind: kind_of(s),
            label: s.name.clone(),
            detail: addr,
            lane: if s.env.is_empty() { "SERVERS".into() } else { s.env.to_uppercase() },
            health: i.health.get(&s.id).copied().unwrap_or(Health::Unknown),
            risks,
            server_id: Some(s.id.clone()),
        });
        match s.jump_host_id.as_deref().filter(|j| by_id.contains_key(j)) {
            Some(j) => g.edges.push(Edge { from: server_node_id(j), to: server_node_id(&s.id), kind: EdgeKind::ViaJump }),
            None => g.edges.push(Edge { from: "crow".into(), to: server_node_id(&s.id), kind: EdgeKind::Ssh }),
        }
    }

    for k in i.keys {
        let unlocks: Vec<&ServerRecord> = i.servers.iter().filter(|s| s.key_id.as_deref() == Some(k.id.as_str()) && s.auth_method == "publickey").collect();
        if unlocks.is_empty() {
            continue;
        }
        let mut risks: Vec<Risk> = key_strength_risk(&k.public_key).into_iter().collect();
        if let Some(days) = age_days(&k.created_at, i.now).filter(|d| *d > KEY_MAX_AGE_DAYS) {
            risks.push(risk(Level::Warn, "key", format!("{days} days old: past the {KEY_MAX_AGE_DAYS}-day rotation guidance")));
        }
        if k.private_key_path.is_none() {
            risks.push(risk(Level::Warn, "key", "no private key file recorded on this machine"));
        }
        g.nodes.push(Node {
            id: key_node_id(&k.id),
            kind: NodeKind::Key,
            label: k.name.clone(),
            detail: k.algorithm.clone(),
            lane: "KEYS".into(),
            health: Health::Ok,
            risks,
            server_id: None,
        });
        for s in unlocks {
            g.edges.push(Edge { from: key_node_id(&k.id), to: server_node_id(&s.id), kind: EdgeKind::Unlocks });
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    const ED: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID6IeWJtxICF/halpN1E+KtZi88x2yIZCTlt4CjfYRyr crow@x";

    fn server(id: &str) -> ServerRecord {
        ServerRecord { id: id.into(), name: id.into(), host: format!("{id}.lan"), port: 22, login_user: "ops".into(), auth_method: "publickey".into(), key_id: Some("k1".into()), env: "PROD".into(), host_key_fingerprint: Some("SHA256:x".into()), ..Default::default() }
    }

    fn inputs<'a>(servers: &'a [ServerRecord], keys: &'a [SshKeyRecord], alerts: &'a [Alert], posture: &'a HashMap<String, Posture>, health: &'a HashMap<String, Health>) -> Inputs<'a> {
        Inputs { servers, keys, alerts, health, posture, now: chrono::DateTime::parse_from_rfc3339("2026-10-07T00:00:00Z").unwrap().timestamp() }
    }

    #[test]
    fn crow_reaches_servers_directly_or_through_their_jump_host() {
        let bastion = server("bastion");
        let web = ServerRecord { jump_host_id: Some("bastion".into()), ..server("web") };
        let db = ServerRecord { jump_host_id: Some("bastion".into()), ..server("db") };
        let key = SshKeyRecord { id: "k1".into(), name: "crow".into(), public_key: ED.into(), created_at: "2026-09-01T00:00:00Z".into(), private_key_path: Some("~/.ssh/crow".into()), ..Default::default() };
        let (servers, keys) = (vec![bastion, web, db], vec![key]);
        let (p, h) = (HashMap::new(), HashMap::new());
        let g = build(&inputs(&servers, &keys, &[], &p, &h));
        assert!(g.edges.contains(&Edge { from: "crow".into(), to: "srv:bastion".into(), kind: EdgeKind::Ssh }));
        assert!(g.edges.contains(&Edge { from: "srv:bastion".into(), to: "srv:web".into(), kind: EdgeKind::ViaJump }));
        assert!(!g.edges.iter().any(|e| e.from == "crow" && e.to == "srv:web"), "web is reached through bastion only");
        assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Unlocks).count(), 3);
        let bastion = g.node("srv:bastion").unwrap();
        assert!(bastion.risks.iter().any(|r| r.text.starts_with("jump host for 2 servers")), "single point of failure");
        assert!(g.node("key:k1").unwrap().risks.is_empty(), "a fresh ed25519 key is fine");
    }

    #[test]
    fn risks_come_only_from_what_crow_has_read() {
        let plain = server("plain");
        let pw = ServerRecord { auth_method: "password".into(), key_id: None, host_key_fingerprint: None, ..server("pw") };
        let servers = vec![plain, pw];
        let alerts = vec![Alert { id: "a".into(), server_id: "plain".into(), kind: "disk".into(), level: "CRIT".into(), detail: "disk 97% full".into(), ..Default::default() }];
        let mut posture = HashMap::new();
        posture.insert("plain".into(), posture::parse("@@sshd\npermitrootlogin yes\npasswordauthentication no\nport 22\n@@firewall\nnone\n@@listen\n0.0.0.0:22\n0.0.0.0:6379\n", 1));
        let h = HashMap::new();
        let g = build(&inputs(&servers, &[], &alerts, &posture, &h));
        let plain = g.node("srv:plain").unwrap();
        let texts: Vec<&str> = plain.risks.iter().map(|r| r.text.as_str()).collect();
        assert!(texts.contains(&"disk 97% full"));
        assert!(texts.iter().any(|t| t.contains("PermitRootLogin yes")));
        assert!(texts.iter().any(|t| t.contains("port 6379 open to every network")));
        assert_eq!(plain.worst(), Some(Level::Crit));
        let pw = g.node("srv:pw").unwrap();
        assert!(pw.risks.iter().any(|r| r.text.contains("password, not a key")));
        assert!(pw.risks.iter().any(|r| r.text.starts_with("host key not pinned")));
        assert!(!pw.risks.iter().any(|r| r.source == "posture"), "no posture read: nothing claimed");
    }

    #[test]
    fn keys_are_judged_by_age_and_strength() {
        let rsa2048 = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDErGx/aFpGNS67ONOrJDrV0/fiX73zejG+Z7d2CH84RAltZWsBz1tDw2Da938s5UeBSjOe7fNzkXWQniGY9v1gzmo+dNhPWuiLp+ZwmwK5q1LRtlzAVTWgPkRFeH7M85TmSPGS4zyik29Hi7irRkUf5yBjSoAincb3ZwpPsgiPBNY/Mb/c6js1xI1Q6XdgwY6xyQfifJ3DfmC6fuOssXjbLDw2/YOZQPnTlQR6TYI48ZBfG2XvbwyhybFHdWj2PaW+8DP29LSgibT1m/lUC+dqN9LyJIk0RMQKVxA9kf3syW9i4jj8sLsyO/gfEoXYg5jluiHYQUoRsV09KQcIsXp1 old@x";
        let r = key_strength_risk(rsa2048).expect("a real 2048-bit RSA key is flagged");
        assert_eq!(r.level, Level::Warn);
        assert!(r.text.starts_with("RSA 2048-bit"), "{}", r.text);
        assert!(key_strength_risk(ED).is_none());
        let old = SshKeyRecord { id: "k1".into(), name: "old".into(), public_key: ED.into(), created_at: "2026-01-01T00:00:00Z".into(), private_key_path: Some("x".into()), ..Default::default() };
        let servers = vec![server("a")];
        let (p, h) = (HashMap::new(), HashMap::new());
        let keys = vec![old];
        let g = build(&inputs(&servers, &keys, &[], &p, &h));
        assert!(g.node("key:k1").unwrap().risks.iter().any(|r| r.text.contains("past the 90-day")));
    }
}
