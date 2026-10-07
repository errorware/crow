//! Deploying a key to a server and switching the server to it (ERR-90).
//!
//! Order matters, so a failure never locks Crow out: install the new key,
//! prove it with a fresh key-only login (not over the connection Crow already
//! holds, which outlives its key), switch, and only then remove the old key.
//! If the new key doesn't log in, it's taken back out and nothing switches.

use crate::host::{Host, DEFAULT_TIMEOUT};
use crate::vault::{ServerRecord, SshKeyRecord};
use crate::views::users::host_data::{authorize_key, revoke_key_blob};

/// The base64 part of an OpenSSH public key line.
pub fn key_blob(public_key: &str) -> Option<String> {
    let mut parts = public_key.split_whitespace();
    let first = parts.next()?;
    // Options may come first in authorized_keys; the algorithm precedes the blob.
    if first.starts_with("ssh-") || first.starts_with("ecdsa-") || first.starts_with("sk-") {
        parts.next().map(str::to_string)
    } else {
        parts.nth(1).map(str::to_string)
    }
}

/// What one server ends up with, for the step's result line.
#[derive(Debug, PartialEq, Eq)]
pub struct Switched {
    pub server: ServerRecord,
    pub removed_old: bool,
}

/// Installs `new_key` for the server's login user, checks it with `verify`
/// (a fresh key-only login as the switched server), and removes `old_blob`
/// from authorized_keys once it does. Returns the server record to save.
pub fn deploy_and_switch(
    host: &dyn Host,
    server: &ServerRecord,
    new_key: &SshKeyRecord,
    old_blob: Option<&str>,
    verify: impl FnOnce(&ServerRecord) -> Result<(), String>,
) -> Result<Switched, String> {
    let user = server.login_user.as_str();
    let new_blob = key_blob(&new_key.public_key).ok_or("the new key isn't an OpenSSH public key")?;
    let home = host
        .exec(&["sh", "-c", "getent passwd \"$1\" | cut -d: -f6", "crow-home", user], DEFAULT_TIMEOUT)
        .map(|o| o.stdout.trim().to_string())
        .map_err(|e| format!("couldn't find {user}'s home: {e}"))?;
    if home.is_empty() {
        return Err(format!("{user} has no home directory on the server"));
    }
    let argv = authorize_key(user, &home, &new_key.public_key)?;
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT).map_err(|e| format!("couldn't install the new key: {e}"))?;

    let mut switched = server.clone();
    switched.key_id = Some(new_key.id.clone());
    switched.auth_method = "publickey".into();
    if let Err(e) = verify(&switched) {
        // Leave the server as it was.
        if let Ok(undo) = revoke_key_blob(user, &home, &new_blob) {
            let undo: Vec<&str> = undo.iter().map(String::as_str).collect();
            let _ = host.exec_privileged(&undo, &[], DEFAULT_TIMEOUT);
        }
        return Err(format!("the new key didn't log in ({e}); it was removed again and nothing switched"));
    }

    let removed_old = match old_blob.filter(|b| *b != new_blob) {
        Some(blob) => match revoke_key_blob(user, &home, blob) {
            Ok(argv) => {
                let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
                host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT).is_ok()
            }
            Err(_) => false,
        },
        None => false,
    };
    Ok(Switched { server: switched, removed_old })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_with_and_without_options() {
        assert_eq!(key_blob("ssh-ed25519 AAAAblob me@box").as_deref(), Some("AAAAblob"));
        assert_eq!(key_blob("no-pty,from=\"10.0.0.1\" ssh-ed25519 AAAAblob me@box").as_deref(), Some("AAAAblob"));
        assert_eq!(key_blob("").as_deref(), None);
    }

    /// Against a throwaway sshd: install, verify, switch, remove the old key;
    /// and a failing verification leaves authorized_keys as it was.
    ///   CROW_LIVE_SSHD="root@127.0.0.1:2299:/path/to/old_key" cargo test live_key_rotation -- --ignored
    #[test]
    #[ignore]
    fn live_key_rotation() {
        use crate::host::{update_directory, SshHost};
        let spec = std::env::var("CROW_LIVE_SSHD").expect("set CROW_LIVE_SSHD=user@host:port:old_key_path");
        let (user_host, rest) = spec.split_once(':').unwrap();
        let (port, old_path) = rest.split_once(':').unwrap();
        let (user, host_addr) = user_host.split_once('@').unwrap();
        let dir = std::env::temp_dir().join(format!("crow-rotate-{}", std::process::id()));
        let (new_key, new_pub, _, _) = crate::keys::generate_keypair("rotated", crate::keys::KeyAlgorithm::Ed25519, Some("crow-test"), "", &dir, None).unwrap();
        let old_pub = std::fs::read_to_string(format!("{old_path}.pub")).unwrap();
        let old_key = SshKeyRecord { id: "old".into(), public_key: old_pub.clone(), private_key_path: Some(old_path.into()), ..new_key.clone() };
        let server = ServerRecord { id: "live".into(), name: "live".into(), host: host_addr.into(), port: port.parse().unwrap(), login_user: user.into(), auth_method: "publickey".into(), key_id: Some("old".into()), ..Default::default() };
        update_directory(std::slice::from_ref(&server), &[old_key.clone(), new_key.clone()]);
        let host = SshHost::for_server(&server);

        // A verification that fails: the new key is installed, then taken out.
        let r = deploy_and_switch(&host, &server, &new_key, key_blob(&old_pub).as_deref(), |_| Err("simulated".into()));
        assert!(r.unwrap_err().contains("nothing switched"));
        let keys = host.exec(&["cat", ".ssh/authorized_keys"], DEFAULT_TIMEOUT).unwrap().stdout;
        assert!(!keys.contains(&key_blob(&new_pub).unwrap()), "new key removed after a failed check");
        assert!(keys.contains(&key_blob(&old_pub).unwrap()), "old key untouched");

        // The real thing.
        let s = deploy_and_switch(&host, &server, &new_key, key_blob(&old_pub).as_deref(), crate::host::ssh::fresh_key_login).unwrap();
        assert!(s.removed_old);
        host.close_connection();
        update_directory(std::slice::from_ref(&s.server), &[old_key, new_key]);
        crate::host::ssh::fresh_key_login(&s.server).expect("logs in with the new key");
        assert!(crate::host::ssh::fresh_key_login(&server).is_err(), "the old key no longer logs in");
        std::fs::remove_dir_all(&dir).ok();
    }
}
