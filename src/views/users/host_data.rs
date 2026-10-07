//! Real accounts on a server, and the exact commands behind each Users
//! screen action (run as root through `Host::exec_privileged`).

use std::collections::HashMap;

use ssh_key::{HashAlg, PublicKey};

use super::{NewUserState, SystemUserRecord, UserSshKeySummary};
use crate::host::{Host, DEFAULT_TIMEOUT};

pub type Argv = Vec<String>;

/// Separates homes in the batched authorized_keys read.
const HOME_MARKER: &str = "@@crow-home@@ ";

/// Reads /etc/passwd, /etc/group, lock state and authorized_keys from `host`.
/// /etc/shadow is read as root, but only the first character of each
/// password field leaves the server — enough to tell a locked account.
/// Every group on the host (name, gid), from /etc/group.
pub fn parse_group_names(group: &str) -> Vec<(String, u32)> {
    group
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let mut f = l.split(':');
            let name = f.next()?.to_string();
            let gid = f.nth(1)?.parse().ok()?;
            Some((name, gid))
        })
        .collect()
}

/// Accounts plus every group that exists (a group with no members still
/// counts, e.g. `sudo` on a fresh server where only root exists).
pub fn read_accounts(host: &dyn Host) -> Result<(Vec<SystemUserRecord>, Vec<(String, u32)>), String> {
    let users = read_users(host)?;
    let groups = parse_group_names(&host.read_file("/etc/group").unwrap_or_default());
    Ok((users, groups))
}

pub fn read_users(host: &dyn Host) -> Result<Vec<SystemUserRecord>, String> {
    let passwd = host.read_file("/etc/passwd").map_err(|e| format!("can't read /etc/passwd: {e}"))?;
    let group = host.read_file("/etc/group").unwrap_or_default();
    let locks = host
        .exec_privileged(&["awk", "-F:", "{print $1\":\"substr($2,1,1)}", "/etc/shadow"], &[], DEFAULT_TIMEOUT)
        .map(|o| o.stdout)
        .unwrap_or_default();
    let mut users = parse_accounts(&passwd, &group, &locks);

    let homes: Vec<String> = users.iter().filter(|u| has_login_shell(u)).map(|u| u.home_dir.clone()).collect();
    if !homes.is_empty() {
        // Read as root, so a symlinked ~/.ssh or authorized_keys (which could
        // point at any file) is skipped rather than followed.
        let script = format!(r#"for h in "$@"; do echo "{HOME_MARKER}$h"; [ -L "$h/.ssh" ] || [ -L "$h/.ssh/authorized_keys" ] || cat "$h/.ssh/authorized_keys" 2>/dev/null; done; true"#);
        let mut argv = vec!["sh".to_string(), "-c".to_string(), script, "crow-keys".to_string()];
        argv.extend(homes);
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        if let Ok(out) = host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT) {
            let keys = parse_authorized_keys_by_home(&out.stdout);
            for user in &mut users {
                if let Some(k) = keys.get(&user.home_dir) {
                    user.authorized_keys = k.clone();
                }
            }
        }
    }
    Ok(users)
}

fn has_login_shell(u: &SystemUserRecord) -> bool {
    !(u.shell.ends_with("nologin") || u.shell.ends_with("/false") || u.shell.is_empty())
}

/// Builds account records from passwd, group and `user:<first char of hash>` lines.
pub fn parse_accounts(passwd: &str, group: &str, locks: &str) -> Vec<SystemUserRecord> {
    // gid -> name, and user -> supplementary groups
    let mut group_names: HashMap<u32, String> = HashMap::new();
    let mut memberships: HashMap<String, Vec<String>> = HashMap::new();
    for line in group.lines() {
        let f: Vec<&str> = line.split(':').collect();
        if f.len() < 3 {
            continue;
        }
        if let Ok(gid) = f[2].parse() {
            group_names.insert(gid, f[0].to_string());
        }
        for member in f.get(3).unwrap_or(&"").split(',').filter(|m| !m.is_empty()) {
            memberships.entry(member.to_string()).or_default().push(f[0].to_string());
        }
    }
    let locked: HashMap<&str, bool> = locks.lines().filter_map(|l| l.split_once(':')).map(|(u, c)| (u, c == "!")).collect();

    passwd
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split(':').collect();
            if f.len() < 7 {
                return None;
            }
            let (username, uid, gid) = (f[0].to_string(), f[2].parse::<u32>().ok()?, f[3].parse::<u32>().ok()?);
            let primary_group = group_names.get(&gid).cloned().unwrap_or_else(|| gid.to_string());
            let mut groups = vec![primary_group.clone()];
            for g in memberships.get(&username).into_iter().flatten() {
                if !groups.contains(g) {
                    groups.push(g.clone());
                }
            }
            let shell = f[6].to_string();
            let is_system_user = uid != 0 && (uid < 1000 || shell.ends_with("nologin") || shell.ends_with("/false"));
            Some(SystemUserRecord {
                is_locked: locked.get(f[0]).copied().unwrap_or(false),
                username,
                uid,
                gid,
                gecos: f[4].split(',').next().unwrap_or_default().to_string(),
                home_dir: f[5].to_string(),
                shell,
                primary_group,
                groups,
                is_system_user,
                authorized_keys: Vec::new(),
                last_login: None,
            })
        })
        .collect()
}

/// Splits the batched authorized_keys read into keys per home directory.
pub fn parse_authorized_keys_by_home(stdout: &str) -> HashMap<String, Vec<UserSshKeySummary>> {
    let mut out: HashMap<String, Vec<UserSshKeySummary>> = HashMap::new();
    let mut current: Option<String> = None;
    for line in stdout.lines() {
        if let Some(home) = line.strip_prefix(HOME_MARKER) {
            current = Some(home.to_string());
            out.entry(home.to_string()).or_default();
        } else if let (Some(home), Some(key)) = (&current, key_summary(line)) {
            out.entry(home.clone()).or_default().push(key);
        }
    }
    out
}

/// One authorized_keys line as a summary; the full line is kept in
/// `key_preview` so it can be revoked exactly. Options before the key type
/// (e.g. `from="..."`) are tolerated.
pub fn key_summary(line: &str) -> Option<UserSshKeySummary> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let start = line.find("ssh-").or_else(|| line.find("ecdsa-")).or_else(|| line.find("sk-"))?;
    let key = PublicKey::from_openssh(&line[start..]).ok()?;
    let fingerprint = key.fingerprint(HashAlg::Sha256).to_string();
    let comment = (!key.comment().is_empty()).then(|| key.comment().to_string());
    Some(UserSshKeySummary {
        id: fingerprint.clone(),
        name: comment.clone().unwrap_or_else(|| "unnamed key".into()),
        algorithm: key.algorithm().as_str().to_uppercase(),
        fingerprint,
        comment,
        key_preview: line.to_string(),
        added_at: String::new(),
    })
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        && name.len() <= 32
}

fn check_name(kind: &str, name: &str) -> Result<(), String> {
    if valid_name(name) { Ok(()) } else { Err(format!("{name:?} is not a valid {kind} name")) }
}

fn argv(parts: &[&str]) -> Argv {
    parts.iter().map(|s| s.to_string()).collect()
}

/// `chpasswd` with `user:password` on stdin: the password never appears in
/// argv (so not in `ps`, change records or the journal). Setting a password
/// also clears a password lock.
pub fn set_password(user: &str, password: &str) -> Result<(Argv, Vec<u8>), String> {
    check_name("user", user)?;
    if password.is_empty() {
        return Err("the password is empty".into());
    }
    if password.contains('\n') || password.contains('\r') {
        return Err("the password can't contain line breaks".into());
    }
    if password.len() > 256 {
        return Err("the password is longer than 256 characters".into());
    }
    Ok((argv(&["chpasswd"]), format!("{user}:{password}\n").into_bytes()))
}

pub fn add_to_group(user: &str, group: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    check_name("group", group)?;
    Ok(argv(&["usermod", "-aG", group, user]))
}

pub fn remove_from_group(user: &str, group: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    check_name("group", group)?;
    Ok(argv(&["gpasswd", "-d", user, group]))
}

pub fn set_locked(user: &str, locked: bool) -> Result<Argv, String> {
    check_name("user", user)?;
    Ok(argv(&["passwd", if locked { "-l" } else { "-u" }, user]))
}

pub fn set_shell(user: &str, shell: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    if !shell.starts_with('/') || shell.contains(char::is_whitespace) {
        return Err(format!("{shell:?} is not an absolute shell path"));
    }
    Ok(argv(&["usermod", "-s", shell, user]))
}

/// `useradd` for the New User form. `sudo_group` is the admin group present
/// on this host (`sudo` on Debian-likes, `wheel` on Red Hat-likes).
pub fn create_user(form: &NewUserState, sudo_group: Option<&str>) -> Result<Argv, String> {
    let user = form.username.trim().to_lowercase();
    check_name("user", &user)?;
    let gecos = form.gecos.trim();
    if gecos.contains(':') || gecos.contains('\n') {
        return Err("the full name can't contain ':' or line breaks".into());
    }
    let mut groups: Vec<String> = Vec::new();
    for g in &form.selected_groups {
        check_name("group", g)?;
        if !groups.contains(g) && *g != user {
            groups.push(g.clone());
        }
    }
    if form.grant_sudo {
        let g = sudo_group.ok_or("this host has neither a sudo nor a wheel group")?;
        if !groups.iter().any(|x| x == g) {
            groups.push(g.to_string());
        }
    }
    let mut cmd = vec!["useradd".to_string(), if form.create_home { "-m" } else { "-M" }.to_string()];
    if !form.shell.trim().is_empty() {
        let shell = form.shell.trim();
        if !shell.starts_with('/') || shell.contains(char::is_whitespace) {
            return Err(format!("{shell:?} is not an absolute shell path"));
        }
        cmd.extend(["-s".into(), shell.into()]);
    }
    if !gecos.is_empty() {
        cmd.extend(["-c".into(), gecos.into()]);
    }
    if !groups.is_empty() {
        cmd.extend(["-G".into(), groups.join(",")]);
    }
    cmd.push(user);
    Ok(cmd)
}

/// Deletes the account but keeps its home directory.
pub fn delete_user(user: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    if user == "root" {
        return Err("the root account can't be deleted".into());
    }
    Ok(argv(&["userdel", user]))
}

/// Runs `script` as `user`, never as root (ERR-117): the user's ~/.ssh is
/// theirs to fill with symlinks, so anything root wrote there could be
/// redirected anywhere. As the user, the kernel only lets the script touch
/// what the user could touch anyway. Uses runuser, else su; when Crow is
/// already that user, runs it directly. `args` become $1, $2, …
fn as_user(user: &str, script: &str, args: &[&str]) -> Argv {
    const SWITCH: &str = r#"u="$1"; s="$2"; shift 2
if [ "$(id -u)" = "$(id -u "$u" 2>/dev/null)" ]; then exec sh -c "$s" crow-as-user "$@"; fi
if command -v runuser >/dev/null 2>&1; then exec runuser -u "$u" -- sh -c "$s" crow-as-user "$@"; fi
exec su -s /bin/sh "$u" -c "$s" crow-as-user "$@""#;
    let mut argv: Argv = vec!["sh".into(), "-c".into(), SWITCH.into(), "crow-as-user".into(), user.into(), script.into()];
    argv.extend(args.iter().map(|a| a.to_string()));
    argv
}

/// Appends `key` to the user's authorized_keys (creating ~/.ssh with the
/// right modes) unless it's already there, as the user (see `as_user`).
pub fn authorize_key(user: &str, home: &str, key: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    let key = key.trim();
    PublicKey::from_openssh(key).map_err(|_| "not a valid OpenSSH public key".to_string())?;
    let script = r#"set -e; umask 077; d="$1/.ssh"; f="$d/authorized_keys"; mkdir -p "$d"; chmod 700 "$d"; [ -e "$f" ] || : > "$f"; chmod 600 "$f"; grep -qxF -- "$2" "$f" || printf '%s\n' "$2" >> "$f""#;
    Ok(as_user(user, script, &[home, key]))
}

/// Removes the exact `line` from the user's authorized_keys, as the user.
pub fn revoke_key(user: &str, home: &str, line: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    let script = r#"f="$1/.ssh/authorized_keys"; [ -f "$f" ] || exit 0; t=$(mktemp "$1/.ssh/.crow-ak.XXXXXX") || exit 1; grep -vxF -- "$2" "$f" > "$t"; cat "$t" > "$f"; r=$?; rm -f "$t"; exit $r"#;
    Ok(as_user(user, script, &[home, line]))
}

/// Removes every authorized_keys line carrying the key `blob` (the base64
/// part), whatever its options or comment, as the user.
pub fn revoke_key_blob(user: &str, home: &str, blob: &str) -> Result<Argv, String> {
    check_name("user", user)?;
    let script = r#"f="$1/.ssh/authorized_keys"; [ -f "$f" ] || exit 0; t=$(mktemp "$1/.ssh/.crow-ak.XXXXXX") || exit 1; awk -v b="$2" '{k=1; for(i=1;i<=NF;i++) if($i==b) k=0} k' "$f" > "$t" || { rm -f "$t"; exit 1; }; cat "$t" > "$f"; r=$?; rm -f "$t"; exit $r"#;
    Ok(as_user(user, script, &[home, blob]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_go_on_stdin_never_argv() {
        let (argv, stdin) = set_password("alice", "s3cret: pass").unwrap();
        assert_eq!(argv, ["chpasswd"]);
        assert_eq!(stdin, b"alice:s3cret: pass\n");
        assert!(set_password("alice", "").is_err());
        assert!(set_password("alice", "a\nroot:x").is_err(), "no injecting a second line");
        assert!(set_password("-alice", "x").is_err());
    }

    #[test]
    fn group_names_include_empty_groups() {
        let g = parse_group_names("root:x:0:\nsudo:x:27:\ndocker:x:998:nhc\n");
        assert_eq!(g, [("root".to_string(), 0), ("sudo".to_string(), 27), ("docker".to_string(), 998)]);
    }

    const PASSWD: &str = "root:x:0:0:root:/root:/bin/bash\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\nnhc:x:1000:1000:Nelson,,,:/home/nhc:/bin/zsh\n";
    const GROUP: &str = "root:x:0:\nsudo:x:27:nhc\nnhc:x:1000:\ndocker:x:998:nhc,other\n";
    const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl nhc@laptop";

    #[test]
    fn accounts_come_from_passwd_group_and_shadow() {
        let users = parse_accounts(PASSWD, GROUP, "root:*\ndaemon:*\nnhc:!\n");
        assert_eq!(users.len(), 3);
        let nhc = users.iter().find(|u| u.username == "nhc").unwrap();
        assert_eq!(nhc.gecos, "Nelson");
        assert_eq!(nhc.groups, vec!["nhc", "sudo", "docker"]);
        assert!(nhc.is_locked);
        assert!(!nhc.is_system_user);
        assert!(users.iter().find(|u| u.username == "daemon").unwrap().is_system_user);
        assert!(!users.iter().find(|u| u.username == "root").unwrap().is_locked);
    }

    #[test]
    fn authorized_keys_are_split_by_home_and_kept_verbatim() {
        let out = format!("{HOME_MARKER}/root\n{HOME_MARKER}/home/nhc\n# comment\nfrom=\"10.0.0.0/8\" {KEY}\ngarbage line\n");
        let keys = parse_authorized_keys_by_home(&out);
        assert!(keys["/root"].is_empty());
        let k = &keys["/home/nhc"][0];
        assert_eq!(keys["/home/nhc"].len(), 1);
        assert_eq!(k.algorithm, "SSH-ED25519");
        assert!(k.fingerprint.starts_with("SHA256:"));
        assert_eq!(k.key_preview, format!("from=\"10.0.0.0/8\" {KEY}"));
    }

    #[test]
    fn commands_are_exact_and_validated() {
        assert_eq!(add_to_group("nhc", "docker").unwrap(), argv(&["usermod", "-aG", "docker", "nhc"]));
        assert_eq!(set_locked("nhc", true).unwrap(), argv(&["passwd", "-l", "nhc"]));
        assert!(add_to_group("-rf", "docker").is_err());
        assert!(set_shell("nhc", "/bin/bash; reboot").is_err());
        assert!(delete_user("root").is_err());
        assert!(authorize_key("nhc", "/home/nhc", "not a key").is_err());
        assert_eq!(authorize_key("nhc", "/home/nhc", KEY).unwrap().last().unwrap(), KEY);
    }

    #[test]
    fn create_user_builds_useradd() {
        let form = NewUserState {
            username: " Deploy ".into(),
            gecos: "Deploy Bot".into(),
            shell: "/bin/bash".into(),
            grant_sudo: true,
            create_home: true,
            selected_groups: vec!["docker".into()],
            ..NewUserState::default()
        };
        assert_eq!(
            create_user(&form, Some("sudo")).unwrap(),
            argv(&["useradd", "-m", "-s", "/bin/bash", "-c", "Deploy Bot", "-G", "docker,sudo", "deploy"])
        );
        assert!(create_user(&NewUserState { gecos: "a:b".into(), ..form.clone() }, Some("sudo")).is_err());
    }

    /// The authorize/revoke scripts, run for real against a temp home.
    #[test]
    fn authorize_and_revoke_round_trip_on_a_real_shell() {
        use crate::host::LocalHost;
        let home = std::env::temp_dir().join(format!("crow-home-{}", std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        let h = home.to_str().unwrap();
        let me = std::env::var("USER").unwrap_or_else(|_| "root".into());
        let run = |cmd: Argv| {
            let a: Vec<&str> = cmd.iter().map(String::as_str).collect();
            LocalHost.exec(&a, DEFAULT_TIMEOUT).unwrap();
        };
        run(authorize_key(&me, h, KEY).unwrap());
        run(authorize_key(&me, h, KEY).unwrap()); // idempotent
        let f = home.join(".ssh/authorized_keys");
        assert_eq!(std::fs::read_to_string(&f).unwrap(), format!("{KEY}\n"));
        run(revoke_key(&me, h, KEY).unwrap());
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "");
        std::fs::remove_dir_all(&home).unwrap();
    }
}
