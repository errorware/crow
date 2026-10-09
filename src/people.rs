//! People across the fleet (ERR-144): which accounts and keys can log in
//! where (and with sudo), offboarding someone everywhere, onboarding them
//! to a group, never touching the login or key Crow itself uses.

use std::collections::BTreeMap;

use crate::views::users::host_data::{self, Argv};
use crate::views::users::models::SystemUserRecord;

/// One server's accounts as last read, and how Crow logs in there.
#[derive(Clone, Debug)]
pub struct ServerAccounts {
    pub server_id: String,
    pub server: String,
    /// The account Crow logs in as.
    pub crow_user: String,
    /// The key blob Crow logs in with, when it's one of its own.
    pub crow_key_blob: Option<String>,
    pub users: Vec<SystemUserRecord>,
    /// Group names on the server (to find sudo or wheel).
    pub groups: Vec<String>,
}

impl ServerAccounts {
    pub fn sudo_group(&self) -> Option<&str> {
        ["sudo", "wheel"].into_iter().find(|g| self.groups.iter().any(|x| x == g))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Presence {
    pub server_id: String,
    pub server: String,
    pub sudo: bool,
    pub locked: bool,
    pub keys: usize,
    /// Crow logs in as this account here.
    pub crow: bool,
}

/// A human account and where it exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountRow {
    pub username: String,
    pub on: Vec<Presence>,
}

/// An authorized key and every account it opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRow {
    pub fingerprint: String,
    /// The base64 blob (how it's matched and revoked).
    pub blob: String,
    /// Crow's name for it when enrolled, else its comment.
    pub label: String,
    pub enrolled: bool,
    /// (server id, server, account, Crow's own login).
    pub opens: Vec<(String, String, String, bool)>,
}

fn blob_of(line: &str) -> Option<String> {
    line.split_whitespace().find(|w| w.len() > 40 && w.starts_with("AAAA")).map(String::from)
}

fn has_login(u: &SystemUserRecord) -> bool {
    !(u.shell.ends_with("nologin") || u.shell.ends_with("/false") || u.shell.is_empty())
}

/// The human accounts (a login shell, or root), each with where it is.
pub fn accounts(scans: &[ServerAccounts]) -> Vec<AccountRow> {
    let mut rows: BTreeMap<String, Vec<Presence>> = BTreeMap::new();
    for s in scans {
        for u in s.users.iter().filter(|u| u.is_human() && has_login(u)) {
            rows.entry(u.username.clone()).or_default().push(Presence {
                server_id: s.server_id.clone(),
                server: s.server.clone(),
                sudo: u.is_sudoer(),
                locked: u.is_locked,
                keys: u.authorized_keys.len(),
                crow: u.username == s.crow_user,
            });
        }
    }
    rows.into_iter().map(|(username, on)| AccountRow { username, on }).collect()
}

/// Every key in any authorized_keys, named from Crow's enrolled keys
/// (`known`: (blob, name)) where it can be.
pub fn keys(scans: &[ServerAccounts], known: &[(String, String)]) -> Vec<KeyRow> {
    let mut rows: BTreeMap<String, KeyRow> = BTreeMap::new();
    for s in scans {
        for u in &s.users {
            for k in &u.authorized_keys {
                let Some(blob) = blob_of(&k.key_preview) else { continue };
                let name = known.iter().find(|(b, _)| *b == blob).map(|(_, n)| n.clone());
                let row = rows.entry(k.fingerprint.clone()).or_insert_with(|| KeyRow {
                    fingerprint: k.fingerprint.clone(),
                    blob: blob.clone(),
                    enrolled: name.is_some(),
                    label: name.unwrap_or_else(|| k.comment.clone().unwrap_or_else(|| "no comment".into())),
                    opens: Vec::new(),
                });
                let crow = u.username == s.crow_user && s.crow_key_blob.as_deref() == Some(blob.as_str());
                row.opens.push((s.server_id.clone(), s.server.clone(), u.username.clone(), crow));
            }
        }
    }
    let mut out: Vec<KeyRow> = rows.into_values().collect();
    out.sort_by(|a, b| b.opens.len().cmp(&a.opens.len()).then(a.label.cmp(&b.label)));
    out
}

/// Keys matching a fingerprint (or its start) or a name/comment.
pub fn find_keys<'a>(rows: &'a [KeyRow], query: &str) -> Vec<&'a KeyRow> {
    let q = query.trim().to_lowercase();
    let q = q.strip_prefix("sha256:").unwrap_or(&q).to_string();
    if q.len() < 3 {
        return rows.iter().collect();
    }
    rows.iter().filter(|r| r.fingerprint.to_lowercase().trim_start_matches("sha256:").starts_with(&q) || r.label.to_lowercase().contains(&q)).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offboard {
    /// Revoke their keys, lock the password and expire the account (an
    /// expired account can't log in with a key either). Reversible.
    Lock,
    /// Revoke their keys and delete the account (the home is kept).
    Delete,
}

/// The commands that offboard `user` on one server, or why it's left out.
pub fn offboard_commands(s: &ServerAccounts, user: &str, mode: Offboard) -> Result<Vec<Argv>, String> {
    host_data::check_name("user", user)?;
    if user == s.crow_user {
        return Err(format!("Crow logs in to {} as {user}: offboarding it would lock Crow out", s.server));
    }
    if user == "root" {
        return Err("root isn't offboarded: turn off root's SSH login in HARDENING instead".into());
    }
    let u = s.users.iter().find(|u| u.username == user).ok_or_else(|| format!("no {user} account on {}", s.server))?;
    // Keys first, while the account can still run things as itself.
    let mut cmds: Vec<Argv> = u.authorized_keys.iter().filter_map(|k| blob_of(&k.key_preview)).map(|b| host_data::revoke_key_blob(user, &u.home_dir, &b)).collect::<Result<_, _>>()?;
    match mode {
        Offboard::Lock => cmds.push(vec!["usermod".into(), "-L".into(), "-e".into(), "1".into(), user.into()]),
        Offboard::Delete => cmds.push(host_data::delete_user(user)?),
    }
    Ok(cmds)
}

/// The commands that remove a key from every account on one server, or
/// why not (it's the key Crow logs in with).
pub fn revoke_key_commands(s: &ServerAccounts, blob: &str) -> Result<Vec<Argv>, String> {
    let mut cmds = Vec::new();
    for u in s.users.iter().filter(|u| u.authorized_keys.iter().any(|k| blob_of(&k.key_preview).as_deref() == Some(blob))) {
        if u.username == s.crow_user && s.crow_key_blob.as_deref() == Some(blob) {
            return Err(format!("it's the key Crow logs in to {} with: switch Crow to another key first (Keys → switch)", s.server));
        }
        cmds.push(host_data::revoke_key_blob(&u.username, &u.home_dir, blob)?);
    }
    if cmds.is_empty() {
        return Err(format!("not on {}", s.server));
    }
    Ok(cmds)
}

/// The commands that give `user` a login with `key` on one server:
/// create the account where it's missing (with sudo if asked), then
/// authorize the key (as the user, its home looked up there).
pub fn onboard_commands(s: &ServerAccounts, user: &str, key: &str, sudo: bool) -> Result<Vec<Argv>, String> {
    host_data::check_name("user", user)?;
    ssh_key::PublicKey::from_openssh(key.trim()).map_err(|_| "that isn't an OpenSSH public key (ssh-ed25519 AAAA… comment)".to_string())?;
    let mut cmds = Vec::new();
    if !s.users.iter().any(|u| u.username == user) {
        let form = crate::views::users::new_user_modal::NewUserState { username: user.into(), shell: "/bin/bash".into(), create_home: true, grant_sudo: sudo, ..Default::default() };
        cmds.push(host_data::create_user(&form, s.sudo_group())?);
    } else if sudo {
        let g = s.sudo_group().ok_or_else(|| format!("{} has neither a sudo nor a wheel group", s.server))?;
        cmds.push(host_data::add_to_group(user, g)?);
    }
    let script = r#"set -e; umask 077; h=$(getent passwd "$(id -un)" | cut -d: -f6); d="$h/.ssh"; f="$d/authorized_keys"; mkdir -p "$d"; chmod 700 "$d"; [ -e "$f" ] || : > "$f"; chmod 600 "$f"; grep -qxF -- "$1" "$f" || printf '%s\n' "$1" >> "$f""#;
    cmds.push(host_data::as_user(user, script, &[key.trim()]));
    Ok(cmds)
}

#[cfg(test)]
mod tests {
    use super::{accounts, find_keys, keys, offboard_commands, onboard_commands, revoke_key_commands, Offboard, ServerAccounts};
    use crate::views::users::host_data::key_summary;
    use crate::views::users::models::SystemUserRecord;

    const ALICE: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID6IeWJtxICF/halpN1E+KtZi88x2yIZCTlt4CjfYRyr alice@laptop";
    const CROW: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG5swBLfTulSzMWiSSQsbNyQ1zO4e9qkW3P2Yb0RZ0aB crow@box";

    fn user(name: &str, groups: &[&str], keys: &[&str]) -> SystemUserRecord {
        SystemUserRecord {
            username: name.into(),
            uid: if name == "root" { 0 } else { 1000 },
            gid: 1000,
            gecos: String::new(),
            home_dir: if name == "root" { "/root".into() } else { format!("/home/{name}") },
            shell: "/bin/bash".into(),
            primary_group: name.into(),
            groups: groups.iter().map(|g| g.to_string()).collect(),
            is_system_user: false,
            is_locked: false,
            authorized_keys: keys.iter().filter_map(|k| key_summary(k)).collect(),
            last_login: None,
        }
    }

    fn crow_blob() -> String {
        CROW.split_whitespace().nth(1).unwrap().to_string()
    }

    fn fleet() -> Vec<ServerAccounts> {
        let s = |id: &str, users: Vec<SystemUserRecord>| ServerAccounts { server_id: id.into(), server: id.into(), crow_user: "ops".into(), crow_key_blob: Some(crow_blob()), users, groups: vec!["sudo".into()] };
        vec![
            s("web-1", vec![user("ops", &["sudo"], &[CROW]), user("alice", &["sudo"], &[ALICE])]),
            s("web-2", vec![user("ops", &["sudo"], &[CROW, ALICE]), user("alice", &[], &[ALICE])]),
            s("db-1", vec![user("ops", &["sudo"], &[CROW])]),
        ]
    }

    #[test]
    fn the_matrix_shows_who_and_which_key_gets_in_where() {
        let rows = accounts(&fleet());
        let alice = rows.iter().find(|r| r.username == "alice").unwrap();
        assert_eq!(alice.on.iter().map(|p| (p.server.as_str(), p.sudo)).collect::<Vec<_>>(), [("web-1", true), ("web-2", false)]);
        assert!(rows.iter().find(|r| r.username == "ops").unwrap().on.iter().all(|p| p.crow));
        let k = keys(&fleet(), &[(crow_blob(), "crow".into())]);
        let alice_key = k.iter().find(|r| r.label == "alice@laptop").unwrap();
        assert_eq!(alice_key.opens.len(), 3, "alice's key also opens ops on web-2");
        assert!(k.iter().find(|r| r.label == "crow").unwrap().enrolled);
        assert_eq!(find_keys(&k, &alice_key.fingerprint[7..20]).len(), 1, "found by the start of its fingerprint");
        assert_eq!(find_keys(&k, "laptop").len(), 1);
    }

    #[test]
    fn offboarding_revokes_keys_first_and_never_touches_crow() {
        let f = fleet();
        let cmds = offboard_commands(&f[0], "alice", Offboard::Lock).unwrap();
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].iter().any(|a| a.contains("authorized_keys")), "keys first");
        assert_eq!(cmds[1], ["usermod", "-L", "-e", "1", "alice"]);
        assert_eq!(offboard_commands(&f[0], "alice", Offboard::Delete).unwrap().last().unwrap(), &vec!["userdel".to_string(), "alice".to_string()]);
        assert!(offboard_commands(&f[0], "ops", Offboard::Lock).unwrap_err().contains("lock Crow out"));
        assert!(offboard_commands(&f[2], "alice", Offboard::Lock).unwrap_err().contains("no alice"));
        // Alice's key comes off ops on web-2 too; Crow's own key never.
        let alice_blob = super::blob_of(ALICE).unwrap();
        assert_eq!(revoke_key_commands(&f[1], &alice_blob).unwrap().len(), 2);
        assert!(revoke_key_commands(&f[1], &crow_blob()).unwrap_err().contains("Crow logs in"));
    }

    #[test]
    fn onboarding_creates_the_account_only_where_missing() {
        let f = fleet();
        let cmds = onboard_commands(&f[2], "alice", ALICE, true).unwrap();
        assert_eq!(cmds[0][0], "useradd");
        assert!(cmds[0].iter().any(|a| a == "sudo"));
        assert!(cmds[1].iter().any(|a| a == ALICE), "the key as an argument");
        let existing = onboard_commands(&f[0], "alice", ALICE, false).unwrap();
        assert_eq!(existing.len(), 1, "just the key");
        assert!(onboard_commands(&f[0], "alice", "not a key", false).is_err());
        assert!(onboard_commands(&f[0], "Bad Name", ALICE, false).is_err());
    }
}
