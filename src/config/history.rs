//! Config history that outlives the app (ERR-72): every version of a file
//! Crow wrote, or found on the host, stored per server and path in the vault.
//!
//! Config files can hold credentials, so content is sealed with the vault's
//! data key like any secret. Without a key only the hash is kept: Crow can
//! still tell that a file changed, not what it said.

use std::collections::HashMap;

use chrono::{DateTime, Local, Utc};
use sha2::{Digest, Sha256};

use super::{ConfigFileState, ConfigRevision};
use crate::vault::crypto::{decrypt_data, encrypt_data, NONCE_LEN};
use crate::vault::{MasterKey, StoredConfigRevision, VaultDb, VaultError};

/// Written by Crow.
pub const SOURCE_CROW: &str = "crow";
/// Found on the host: first sight, or changed by something other than Crow.
pub const SOURCE_OBSERVED: &str = "observed";

/// History key for configs read from this machine when no server is enrolled.
pub const LOCAL_SERVER_ID: &str = "local";

pub fn sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Who is making a change: the local account and machine running Crow.
pub fn local_author() -> String {
    let user = std::env::var("USER").or_else(|_| std::env::var("LOGNAME")).unwrap_or_else(|_| "unknown".into());
    let host = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .or_else(|| std::process::Command::new("hostname").output().ok().and_then(|o| String::from_utf8(o.stdout).ok()))
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty());
    match host {
        Some(h) => format!("{user}@{h}"),
        None => user,
    }
}

/// Stores one version of `path` on `server_id`.
pub fn record(
    db: &VaultDb,
    key: Option<&MasterKey>,
    server_id: &str,
    path: &str,
    content: &str,
    author: &str,
    message: &str,
    source: &str,
) -> Result<StoredConfigRevision, VaultError> {
    let sealed = match key {
        Some(k) => {
            let (ciphertext, nonce) = encrypt_data(k, content.as_bytes())?;
            Some((nonce.to_vec(), ciphertext))
        }
        None => None,
    };
    let rev = StoredConfigRevision {
        id: format!("cfgrev-{}", uuid_like()),
        server_id: server_id.to_string(),
        path: path.to_string(),
        sha256: sha256_hex(content),
        author: author.to_string(),
        message: message.to_string(),
        source: source.to_string(),
        created_at: Utc::now().to_rfc3339(),
        sealed,
    };
    db.insert_config_revision(&rev)?;
    Ok(rev)
}

/// Brings each file read from the host up to date with its recorded
/// history, then loads that history into the file's state.
///
/// A file Crow hasn't seen before gets a "first seen" revision; one whose
/// content differs from its last recorded revision was changed by something
/// other than Crow and gets a "changed outside Crow" revision. Files that
/// weren't read from the host (unreadable, or content Crow generated) are
/// left alone.
pub fn sync(db: &VaultDb, key: Option<&MasterKey>, server_id: &str, states: &mut HashMap<String, ConfigFileState>) -> Result<(), VaultError> {
    let latest = db.latest_config_hashes(server_id)?;
    for st in states.values_mut().filter(|st| st.read_from_host) {
        let path = st.path.to_string_lossy().into_owned();
        // The baseline is what's on the host; current may carry unsaved edits.
        let on_host = sha256_hex(&st.baseline_content);
        let message = match latest.get(&path) {
            Some(h) if *h == on_host => None,
            Some(_) => Some("Changed outside Crow"),
            None => Some("First seen by Crow"),
        };
        if let Some(message) = message {
            record(db, key, server_id, &path, &st.baseline_content, "on the host", message, SOURCE_OBSERVED)?;
        }
        load_into(db, key, server_id, st)?;
    }
    Ok(())
}

/// Replaces `st`'s revisions with those recorded for it, oldest first, and
/// marks the one matching what's on the host as active.
pub fn load_into(db: &VaultDb, key: Option<&MasterKey>, server_id: &str, st: &mut ConfigFileState) -> Result<(), VaultError> {
    let stored = db.list_config_revisions(server_id, &st.path.to_string_lossy())?;
    if stored.is_empty() {
        return Ok(());
    }
    st.revisions = stored.iter().enumerate().map(|(i, r)| to_revision(i + 1, r, key)).collect();
    let on_host = sha256_hex(&st.baseline_content);
    st.active_revision = st.revisions.iter().rev().find(|r| r.sha256 == on_host).map(|r| r.version).unwrap_or(0);
    Ok(())
}

fn to_revision(version: usize, r: &StoredConfigRevision, key: Option<&MasterKey>) -> ConfigRevision {
    let content = match (&r.sealed, key) {
        (Some((nonce, ciphertext)), Some(k)) => <[u8; NONCE_LEN]>::try_from(nonce.as_slice())
            .ok()
            .and_then(|n| decrypt_data(k, ciphertext, &n).ok())
            .and_then(|b| String::from_utf8(b).ok()),
        _ => None,
    };
    ConfigRevision {
        version,
        timestamp: DateTime::parse_from_rfc3339(&r.created_at)
            .map(|t| t.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|_| r.created_at.clone()),
        author: r.author.clone(),
        message: r.message.clone(),
        content,
        sha256: r.sha256.clone(),
        source: r.source.clone(),
    }
}

/// Why a revision has no content, for the UI.
pub const MISSING_CONTENT: &str = "content not kept (no encryption key when recorded, or the vault is locked)";

fn uuid_like() -> String {
    let bytes: [u8; 16] = rand::random();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::generate_data_key;
    use std::path::PathBuf;

    fn state(path: &str, content: &str) -> ConfigFileState {
        let mut st = ConfigFileState::new(PathBuf::from(path), path.rsplit('/').next().unwrap().into(), content.into());
        st.read_from_host = true;
        st
    }

    #[test]
    fn history_survives_a_reload_and_notices_outside_changes() {
        let db = VaultDb::open_in_memory().unwrap();
        let key = generate_data_key();
        let mut states = HashMap::from([("sshd_config".to_string(), state("/etc/ssh/sshd_config", "Port 22\n"))]);

        sync(&db, Some(&key), "srv1", &mut states).unwrap();
        let st = &states["sshd_config"];
        assert_eq!(st.revisions.len(), 1);
        assert_eq!(st.revisions[0].message, "First seen by Crow");
        assert_eq!(st.active_revision, 1);

        // Crow saves a change.
        record(&db, Some(&key), "srv1", "/etc/ssh/sshd_config", "Port 2222\n", "me@box", "Move SSH", SOURCE_CROW).unwrap();

        // Next launch: same content as Crow wrote, nothing new recorded.
        let mut states = HashMap::from([("sshd_config".to_string(), state("/etc/ssh/sshd_config", "Port 2222\n"))]);
        sync(&db, Some(&key), "srv1", &mut states).unwrap();
        assert_eq!(states["sshd_config"].revisions.len(), 2);
        assert_eq!(states["sshd_config"].active_revision, 2);

        // Someone edits it over plain SSH; the next load records that.
        let mut states = HashMap::from([("sshd_config".to_string(), state("/etc/ssh/sshd_config", "Port 2200\n"))]);
        sync(&db, Some(&key), "srv1", &mut states).unwrap();
        let revs = &states["sshd_config"].revisions;
        assert_eq!(revs.len(), 3);
        assert_eq!((revs[2].message.as_str(), revs[2].source.as_str()), ("Changed outside Crow", SOURCE_OBSERVED));
        assert_eq!(revs[1].content.as_deref(), Some("Port 2222\n"), "earlier revisions are restorable");
    }

    #[test]
    fn content_is_sealed_and_hash_only_without_a_key() {
        let db = VaultDb::open_in_memory().unwrap();
        let key = generate_data_key();
        let secret = "password = hunter2\n";
        let rev = record(&db, Some(&key), "srv1", "/etc/app.conf", secret, "me", "save", SOURCE_CROW).unwrap();
        let (_, ciphertext) = rev.sealed.as_ref().unwrap();
        assert!(!ciphertext.windows(7).any(|w| w == b"hunter2"), "no plaintext in the vault");

        record(&db, None, "srv1", "/etc/app.conf", "x\n", "me", "save", SOURCE_CROW).unwrap();
        let mut st = state("/etc/app.conf", "x\n");
        load_into(&db, Some(&key), "srv1", &mut st).unwrap();
        assert_eq!(st.revisions[0].content.as_deref(), Some(secret));
        assert_eq!(st.revisions[1].content, None);
        assert_eq!(st.revisions[1].sha256, sha256_hex("x\n"));

        // Locked vault: nothing decrypts, hashes still match.
        load_into(&db, None, "srv1", &mut st).unwrap();
        assert!(st.revisions.iter().all(|r| r.content.is_none()));
        assert_eq!(st.active_revision, 2);
    }

    #[test]
    fn files_not_read_from_the_host_are_not_recorded() {
        let db = VaultDb::open_in_memory().unwrap();
        let mut unread = state("/etc/shadow", "");
        unread.read_from_host = false;
        let mut states = HashMap::from([("shadow".to_string(), unread)]);
        sync(&db, None, "srv1", &mut states).unwrap();
        assert!(db.list_config_revisions("srv1", "/etc/shadow").unwrap().is_empty());
    }
}
