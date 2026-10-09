//! Team management, local edition (ERR-150): who uses this Crow, with
//! which role, on which servers. Members, roles and scopes live in the
//! vault; each member signs in with their own passphrase, and Crow
//! enforces their role at every action that changes a server.
//!
//! It's one machine's Crow: sharing it across machines needs the accounts
//! service (ERR-102); the model here is what that will sync.

use serde::{Deserialize, Serialize};

use crate::vault::ServerRecord;

pub const TEAM_FLAG: &str = "team.config";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Permission {
    /// Read everything (always granted).
    View,
    /// Start, stop, restart services; reboot.
    Restart,
    /// Write config files, pin baselines.
    EditConfig,
    /// Fleet runs: patching, rollouts, drift, hardening, certificates.
    RunFleet,
    /// Accounts and keys on servers (users, people, firewall).
    Users,
    /// Crow's own SSH keys.
    Keys,
    /// The vault, plugins and settings.
    Vault,
    /// Members and roles.
    Team,
}

impl Permission {
    pub const ALL: [Permission; 8] = [Permission::View, Permission::Restart, Permission::EditConfig, Permission::RunFleet, Permission::Users, Permission::Keys, Permission::Vault, Permission::Team];

    pub fn label(self) -> &'static str {
        match self {
            Permission::View => "view",
            Permission::Restart => "restart services",
            Permission::EditConfig => "edit configs",
            Permission::RunFleet => "fleet runs",
            Permission::Users => "users & firewall",
            Permission::Keys => "SSH keys",
            Permission::Vault => "vault & settings",
            Permission::Team => "team",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Viewer,
    Operator,
    Admin,
    Custom { name: String, perms: Vec<Permission> },
}

impl Role {
    pub fn name(&self) -> String {
        match self {
            Role::Viewer => "Viewer".into(),
            Role::Operator => "Operator".into(),
            Role::Admin => "Admin".into(),
            Role::Custom { name, .. } => name.clone(),
        }
    }

    pub fn allows(&self, p: Permission) -> bool {
        match self {
            Role::Admin => true,
            Role::Viewer => p == Permission::View,
            Role::Operator => matches!(p, Permission::View | Permission::Restart | Permission::EditConfig | Permission::RunFleet),
            Role::Custom { perms, .. } => p == Permission::View || perms.contains(&p),
        }
    }
}

/// Where a member's role applies.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    #[default]
    Fleet,
    Groups(Vec<String>),
    /// Environments, e.g. STAGE (matched case-insensitively).
    Envs(Vec<String>),
}

impl Scope {
    pub fn covers(&self, s: &ServerRecord) -> bool {
        match self {
            Scope::Fleet => true,
            Scope::Groups(g) => g.iter().any(|x| x == s.group_name.trim()),
            Scope::Envs(e) => e.iter().any(|x| x.eq_ignore_ascii_case(s.env.trim())),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Scope::Fleet => "the whole fleet".into(),
            Scope::Groups(g) => format!("group{} {}", if g.len() == 1 { "" } else { "s" }, g.join(", ")),
            Scope::Envs(e) => format!("env {}", e.join(", ")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub scope: Scope,
    /// Argon2id PHC string of their passphrase.
    pub verifier: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TeamConfig {
    pub members: Vec<Member>,
    /// Fleet runs need a second member's passphrase.
    pub require_approval: bool,
}

impl TeamConfig {
    /// Team mode: on once there's a member.
    pub fn enabled(&self) -> bool {
        !self.members.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Whether `member` may do `perm` (on `server`, when it's about one).
    pub fn may(&self, member: Option<&Member>, perm: Permission, server: Option<&ServerRecord>) -> Result<(), String> {
        if !self.enabled() {
            return Ok(());
        }
        let m = member.ok_or("sign in first: this Crow has a team")?;
        if !m.role.allows(perm) {
            return Err(format!("{} is a {}: {} needs a role that allows {}", m.name, m.role.name(), perm.label(), perm.label()));
        }
        if let Some(s) = server {
            if !m.scope.covers(s) {
                return Err(format!("{} isn't in {}'s scope ({})", s.name, m.name, m.scope.describe()));
            }
        }
        Ok(())
    }

    /// Whether removing or demoting `id` would leave no Admin.
    pub fn last_admin(&self, id: &str) -> bool {
        self.members.iter().filter(|m| m.role == Role::Admin && m.id != id).count() == 0 && self.find(id).is_some_and(|m| m.role == Role::Admin)
    }
}

pub fn hash_passphrase(p: &str) -> Result<String, String> {
    if p.chars().count() < 8 {
        return Err("a passphrase needs at least 8 characters".into());
    }
    crate::vault::crypto::hash_password_verifier(p).map_err(|e| e.to_string())
}

pub fn check_passphrase(m: &Member, p: &str) -> bool {
    crate::vault::crypto::verify_password(p, &m.verifier).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{check_passphrase, hash_passphrase, Member, Permission, Role, Scope, TeamConfig};
    use crate::vault::ServerRecord;

    fn member(name: &str, role: Role, scope: Scope) -> Member {
        Member { id: name.into(), name: name.into(), role, scope, verifier: String::new() }
    }

    #[test]
    fn roles_and_scopes_decide() {
        let prod = ServerRecord { name: "web-1".into(), env: "PROD".into(), group_name: "web".into(), ..Default::default() };
        let stage = ServerRecord { name: "web-s".into(), env: "STAGE".into(), group_name: "web".into(), ..Default::default() };
        let team = TeamConfig { members: vec![member("ana", Role::Admin, Scope::Fleet), member("bo", Role::Operator, Scope::Envs(vec!["stage".into()])), member("cy", Role::Viewer, Scope::Fleet)], require_approval: false };
        let (ana, bo, cy) = (team.find("ana"), team.find("bo"), team.find("cy"));
        assert!(team.may(ana, Permission::Keys, Some(&prod)).is_ok());
        assert!(team.may(bo, Permission::Restart, Some(&stage)).is_ok());
        assert!(team.may(bo, Permission::Restart, Some(&prod)).unwrap_err().contains("scope"));
        assert!(team.may(bo, Permission::Users, Some(&stage)).unwrap_err().contains("Operator"));
        assert!(team.may(cy, Permission::View, Some(&prod)).is_ok());
        assert!(team.may(cy, Permission::Restart, None).is_err());
        assert!(team.may(None, Permission::View, None).unwrap_err().contains("sign in"));
        assert!(TeamConfig::default().may(None, Permission::Vault, None).is_ok(), "no team: everything, as before");
        let custom = Role::Custom { name: "Patcher".into(), perms: vec![Permission::RunFleet] };
        assert!(custom.allows(Permission::RunFleet) && custom.allows(Permission::View) && !custom.allows(Permission::Keys));
        assert!(team.last_admin("ana") && !team.last_admin("bo"));
    }

    #[test]
    fn passphrases_are_hashed_and_checked() {
        assert!(hash_passphrase("short").is_err());
        let mut m = member("ana", Role::Admin, Scope::Fleet);
        m.verifier = hash_passphrase("correct horse").unwrap();
        assert!(check_passphrase(&m, "correct horse") && !check_passphrase(&m, "wrong horse"));
        assert!(!m.verifier.contains("correct"));
    }
}
