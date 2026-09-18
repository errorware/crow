use std::path::{Path, PathBuf};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::crypto::{
    decrypt_data, derive_master_key, encrypt_data, generate_salt, hash_password_verifier,
    verify_password, verify_totp_code, MasterKey, NONCE_LEN, SALT_LEN,
};

#[derive(Debug)]
pub enum VaultError {
    Database(rusqlite::Error),
    Crypto(String),
    Io(std::io::Error),
    InvalidPassword,
    TotpRequired,
    InvalidTotpCode,
    AlreadyInitialized,
    NotInitialized,
    EntryNotFound(String),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Database(e) => write!(f, "Database error: {}", e),
            VaultError::Crypto(e) => write!(f, "Cryptographic error: {}", e),
            VaultError::Io(e) => write!(f, "IO error: {}", e),
            VaultError::InvalidPassword => write!(f, "Invalid master password"),
            VaultError::TotpRequired => write!(f, "Two-factor authentication code required"),
            VaultError::InvalidTotpCode => write!(f, "Invalid 2FA authentication code"),
            VaultError::AlreadyInitialized => write!(f, "Vault is already initialized"),
            VaultError::NotInitialized => write!(f, "Vault is not initialized"),
            VaultError::EntryNotFound(id) => write!(f, "Vault entry '{}' not found", id),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<rusqlite::Error> for VaultError {
    fn from(e: rusqlite::Error) -> Self {
        VaultError::Database(e)
    }
}

impl From<std::io::Error> for VaultError {
    fn from(e: std::io::Error) -> Self {
        VaultError::Io(e)
    }
}

impl From<super::crypto::CryptoError> for VaultError {
    fn from(e: super::crypto::CryptoError) -> Self {
        VaultError::Crypto(e.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultEntryMeta {
    pub id: String,
    pub category: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct VaultMeta {
    pub totp_enabled: bool,
    pub auto_lock_minutes: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshScanPath {
    pub id: i64,
    pub path: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshKeyGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshKeyRecord {
    pub id: String,
    pub name: String,
    pub group_id: String,
    pub public_key: String,
    pub fingerprint: String,
    pub algorithm: String,
    pub comment: Option<String>,
    pub private_key_path: Option<String>,
    pub attached_servers: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct VaultDb {
    conn: Connection,
    path: PathBuf,
}

impl VaultDb {
    /// Returns the default path for Crow's SQLite database (~/.config/crow/crow.db).
    pub fn default_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("crow").join("crow.db")
        } else {
            PathBuf::from("crow.db")
        }
    }

    /// Opens or creates the SQLite vault at the specified path (or default path if None).
    pub fn open_or_create(custom_path: Option<&Path>) -> Result<Self, VaultError> {
        let path = custom_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;

        // Pragmas for durability and security
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;

        let db = Self { conn, path };
        db.create_tables()?;
        Ok(db)
    }

    /// In-memory vault connection for unit tests.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, VaultError> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn,
            path: PathBuf::from(":memory:"),
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<(), VaultError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS vault_meta (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                salt BLOB NOT NULL,
                verifier_hash TEXT NOT NULL,
                totp_enabled INTEGER NOT NULL DEFAULT 0,
                encrypted_totp_secret BLOB,
                totp_nonce BLOB,
                auto_lock_minutes INTEGER NOT NULL DEFAULT 15,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vault_entries (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                nonce BLOB NOT NULL,
                ciphertext BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_entries_category ON vault_entries (category);

            CREATE TABLE IF NOT EXISTS ssh_scan_paths (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ssh_key_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#60a5fa',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ssh_keys (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                group_id TEXT NOT NULL DEFAULT 'default',
                public_key TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                algorithm TEXT NOT NULL,
                comment TEXT,
                private_key_path TEXT,
                attached_servers TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_ssh_keys_group ON ssh_keys (group_id);
            CREATE INDEX IF NOT EXISTS idx_ssh_keys_fingerprint ON ssh_keys (fingerprint);",
        )?;

        // Seed default scan path if empty
        let scan_path_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM ssh_scan_paths",
            [],
            |r| r.get(0),
        )?;
        if scan_path_count == 0 {
            let now = Utc::now().to_rfc3339();
            let default_scan = dirs::home_dir()
                .map(|h| h.join(".ssh").display().to_string())
                .unwrap_or_else(|| "~/.ssh".to_string());
            let _ = self.conn.execute(
                "INSERT OR IGNORE INTO ssh_scan_paths (path, created_at) VALUES (?1, ?2)",
                params![default_scan, now],
            );
        }

        // Seed default groups if empty
        let group_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM ssh_key_groups",
            [],
            |r| r.get(0),
        )?;
        if group_count == 0 {
            let now = Utc::now().to_rfc3339();
            let defaults = [
                ("default", "Default", "#94a3b8"),
                ("fleet", "Fleet", "#4ade80"),
                ("bastions", "Bastions", "#60a5fa"),
                ("production", "Production", "#f59e0b"),
                ("legacy", "Legacy", "#f87171"),
            ];
            for (id, name, color) in defaults {
                let _ = self.conn.execute(
                    "INSERT OR IGNORE INTO ssh_key_groups (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
                    params![id, name, color, now],
                );
            }
        }

        Ok(())
    }

    /// Checks whether the vault has been initialized with a master password.
    pub fn is_initialized(&self) -> Result<bool, VaultError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM vault_meta WHERE id = 1",
            [],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    /// Returns whether TOTP 2FA is currently enabled for this vault.
    pub fn is_totp_enabled(&self) -> Result<bool, VaultError> {
        let res: Option<i64> = self.conn.query_row(
            "SELECT totp_enabled FROM vault_meta WHERE id = 1",
            [],
            |r| r.get(0),
        ).optional()?;
        Ok(res.map(|v| v > 0).unwrap_or(false))
    }

    /// First-time setup: initializes the vault with a master password and optional TOTP secret.
    pub fn init_vault(&mut self, password: &str, totp_secret: Option<&str>) -> Result<MasterKey, VaultError> {
        if self.is_initialized()? {
            return Err(VaultError::AlreadyInitialized);
        }

        let salt = generate_salt();
        let master_key = derive_master_key(password, &salt)?;
        let verifier_hash = hash_password_verifier(password)?;
        let now = Utc::now().to_rfc3339();

        let (totp_enabled, enc_secret, totp_nonce) = if let Some(secret) = totp_secret {
            let (ciphertext, nonce) = encrypt_data(&master_key, secret.as_bytes())?;
            (1, Some(ciphertext), Some(nonce.to_vec()))
        } else {
            (0, None, None)
        };

        self.conn.execute(
            "INSERT INTO vault_meta (id, salt, verifier_hash, totp_enabled, encrypted_totp_secret, totp_nonce, auto_lock_minutes, created_at, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, 15, ?6, ?6)",
            params![
                salt.as_slice(),
                verifier_hash,
                totp_enabled,
                enc_secret,
                totp_nonce,
                now,
            ],
        )?;

        Ok(master_key)
    }

    /// Authenticates with password and optional TOTP code, returning the derived MasterKey on success.
    pub fn unlock(&self, password: &str, totp_code: Option<&str>) -> Result<MasterKey, VaultError> {
        let row = self.conn.query_row(
            "SELECT salt, verifier_hash, totp_enabled, encrypted_totp_secret, totp_nonce FROM vault_meta WHERE id = 1",
            [],
            |r| {
                let salt: Vec<u8> = r.get(0)?;
                let verifier_hash: String = r.get(1)?;
                let totp_enabled: i64 = r.get(2)?;
                let encrypted_totp: Option<Vec<u8>> = r.get(3)?;
                let totp_nonce: Option<Vec<u8>> = r.get(4)?;
                Ok((salt, verifier_hash, totp_enabled > 0, encrypted_totp, totp_nonce))
            },
        ).optional()?;

        let (salt_vec, verifier_hash, totp_enabled, encrypted_totp, totp_nonce) = match row {
            Some(data) => data,
            None => return Err(VaultError::NotInitialized),
        };

        if salt_vec.len() != SALT_LEN {
            return Err(VaultError::Crypto("Corrupted salt in database".into()));
        }
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&salt_vec);

        // 1. Verify password against hash verifier
        let valid_password = verify_password(password, &verifier_hash)
            .map_err(|e| VaultError::Crypto(e.to_string()))?;
        if !valid_password {
            return Err(VaultError::InvalidPassword);
        }

        // 2. Derive MasterKey
        let master_key = derive_master_key(password, &salt)?;

        // 3. If TOTP is enabled, decrypt secret and verify the submitted code
        if totp_enabled {
            let code = totp_code.ok_or(VaultError::TotpRequired)?;
            let enc_secret = encrypted_totp.ok_or(VaultError::Crypto("Missing encrypted TOTP secret".into()))?;
            let nonce_vec = totp_nonce.ok_or(VaultError::Crypto("Missing TOTP nonce".into()))?;

            if nonce_vec.len() != NONCE_LEN {
                return Err(VaultError::Crypto("Invalid TOTP nonce length".into()));
            }
            let mut nonce = [0u8; NONCE_LEN];
            nonce.copy_from_slice(&nonce_vec);

            let decrypted_secret_bytes = decrypt_data(&master_key, &enc_secret, &nonce)?;
            let secret_base32 = String::from_utf8(decrypted_secret_bytes)
                .map_err(|_| VaultError::Crypto("Corrupt TOTP secret encoding".into()))?;

            if !verify_totp_code(&secret_base32, code) {
                return Err(VaultError::InvalidTotpCode);
            }
        }

        Ok(master_key)
    }

    /// Stores an encrypted entry in the vault.
    pub fn store_entry(
        &self,
        key: &MasterKey,
        id: &str,
        category: &str,
        name: &str,
        plaintext: &[u8],
    ) -> Result<(), VaultError> {
        let (ciphertext, nonce) = encrypt_data(key, plaintext)?;
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO vault_entries (id, category, name, nonce, ciphertext, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 category = excluded.category,
                 name = excluded.name,
                 nonce = excluded.nonce,
                 ciphertext = excluded.ciphertext,
                 updated_at = excluded.updated_at",
            params![
                id,
                category,
                name,
                nonce.as_slice(),
                ciphertext,
                now,
            ],
        )?;

        Ok(())
    }

    /// Decrypts and loads an entry from the vault.
    pub fn load_entry(&self, key: &MasterKey, id: &str) -> Result<Vec<u8>, VaultError> {
        let row = self.conn.query_row(
            "SELECT nonce, ciphertext FROM vault_entries WHERE id = ?1",
            params![id],
            |r| {
                let nonce: Vec<u8> = r.get(0)?;
                let ciphertext: Vec<u8> = r.get(1)?;
                Ok((nonce, ciphertext))
            },
        ).optional()?;

        let (nonce_vec, ciphertext) = match row {
            Some(data) => data,
            None => return Err(VaultError::EntryNotFound(id.to_string())),
        };

        if nonce_vec.len() != NONCE_LEN {
            return Err(VaultError::Crypto("Invalid nonce stored in database".into()));
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&nonce_vec);

        let plaintext = decrypt_data(key, &ciphertext, &nonce)?;
        Ok(plaintext)
    }

    /// Lists metadata for stored entries, optionally filtered by category.
    pub fn list_entries(&self, category: Option<&str>) -> Result<Vec<VaultEntryMeta>, VaultError> {
        let mut results = Vec::new();

        if let Some(cat) = category {
            let mut stmt = self.conn.prepare(
                "SELECT id, category, name, created_at, updated_at FROM vault_entries WHERE category = ?1 ORDER BY name ASC",
            )?;
            let rows = stmt.query_map(params![cat], |r| {
                Ok(VaultEntryMeta {
                    id: r.get(0)?,
                    category: r.get(1)?,
                    name: r.get(2)?,
                    created_at: r.get(3)?,
                    updated_at: r.get(4)?,
                })
            })?;
            for row in rows {
                results.push(row?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, category, name, created_at, updated_at FROM vault_entries ORDER BY category ASC, name ASC",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok(VaultEntryMeta {
                    id: r.get(0)?,
                    category: r.get(1)?,
                    name: r.get(2)?,
                    created_at: r.get(3)?,
                    updated_at: r.get(4)?,
                })
            })?;
            for row in rows {
                results.push(row?);
            }
        }

        Ok(results)
    }

    /// Deletes an entry from the vault. Returns true if an entry was deleted.
    pub fn delete_entry(&self, id: &str) -> Result<bool, VaultError> {
        let rows_affected = self.conn.execute(
            "DELETE FROM vault_entries WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    /// Fetches vault configuration metadata.
    pub fn get_meta(&self) -> Result<VaultMeta, VaultError> {
        let meta = self.conn.query_row(
            "SELECT totp_enabled, auto_lock_minutes, created_at, updated_at FROM vault_meta WHERE id = 1",
            [],
            |r| {
                let totp: i64 = r.get(0)?;
                let auto_lock: i64 = r.get(1)?;
                let created: String = r.get(2)?;
                let updated: String = r.get(3)?;
                Ok(VaultMeta {
                    totp_enabled: totp > 0,
                    auto_lock_minutes: auto_lock,
                    created_at: created,
                    updated_at: updated,
                })
            },
        ).optional()?;

        meta.ok_or(VaultError::NotInitialized)
    }

    // --- SSH Scan Paths ---
    pub fn list_scan_paths(&self) -> Result<Vec<SshScanPath>, VaultError> {
        let mut stmt = self.conn.prepare("SELECT id, path, created_at FROM ssh_scan_paths ORDER BY id ASC")?;
        let rows = stmt.query_map([], |r| {
            Ok(SshScanPath {
                id: r.get(0)?,
                path: r.get(1)?,
                created_at: r.get(2)?,
            })
        })?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn add_scan_path(&self, path: &str) -> Result<SshScanPath, VaultError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO ssh_scan_paths (path, created_at) VALUES (?1, ?2)",
            params![path, now],
        )?;
        let row = self.conn.query_row(
            "SELECT id, path, created_at FROM ssh_scan_paths WHERE path = ?1",
            params![path],
            |r| {
                Ok(SshScanPath {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    created_at: r.get(2)?,
                })
            },
        )?;
        Ok(row)
    }

    pub fn remove_scan_path(&self, id: i64) -> Result<bool, VaultError> {
        let rows = self.conn.execute("DELETE FROM ssh_scan_paths WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    // --- SSH Key Groups ---
    pub fn list_key_groups(&self) -> Result<Vec<SshKeyGroup>, VaultError> {
        let mut stmt = self.conn.prepare("SELECT id, name, color, created_at FROM ssh_key_groups ORDER BY rowid ASC")?;
        let rows = stmt.query_map([], |r| {
            Ok(SshKeyGroup {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                created_at: r.get(3)?,
            })
        })?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn add_key_group(&self, id: &str, name: &str, color: &str) -> Result<SshKeyGroup, VaultError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO ssh_key_groups (id, name, color, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, color = excluded.color",
            params![id, name, color, now],
        )?;
        Ok(SshKeyGroup {
            id: id.to_string(),
            name: name.to_string(),
            color: color.to_string(),
            created_at: now,
        })
    }

    pub fn delete_key_group(&self, id: &str) -> Result<bool, VaultError> {
        if id == "default" {
            return Ok(false); // Protect default group
        }
        self.conn.execute("UPDATE ssh_keys SET group_id = 'default' WHERE group_id = ?1", params![id])?;
        let rows = self.conn.execute("DELETE FROM ssh_key_groups WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    // --- SSH Keys ---
    pub fn list_ssh_keys(&self) -> Result<Vec<SshKeyRecord>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, group_id, public_key, fingerprint, algorithm, comment, private_key_path, attached_servers, created_at, updated_at
             FROM ssh_keys ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |r| {
            let servers_json: String = r.get(8)?;
            let servers: Vec<String> = serde_json::from_str(&servers_json).unwrap_or_default();
            Ok(SshKeyRecord {
                id: r.get(0)?,
                name: r.get(1)?,
                group_id: r.get(2)?,
                public_key: r.get(3)?,
                fingerprint: r.get(4)?,
                algorithm: r.get(5)?,
                comment: r.get(6)?,
                private_key_path: r.get(7)?,
                attached_servers: servers,
                created_at: r.get(9)?,
                updated_at: r.get(10)?,
            })
        })?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn get_ssh_key(&self, id: &str) -> Result<Option<SshKeyRecord>, VaultError> {
        let res = self.conn.query_row(
            "SELECT id, name, group_id, public_key, fingerprint, algorithm, comment, private_key_path, attached_servers, created_at, updated_at
             FROM ssh_keys WHERE id = ?1",
            params![id],
            |r| {
                let servers_json: String = r.get(8)?;
                let servers: Vec<String> = serde_json::from_str(&servers_json).unwrap_or_default();
                Ok(SshKeyRecord {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    group_id: r.get(2)?,
                    public_key: r.get(3)?,
                    fingerprint: r.get(4)?,
                    algorithm: r.get(5)?,
                    comment: r.get(6)?,
                    private_key_path: r.get(7)?,
                    attached_servers: servers,
                    created_at: r.get(9)?,
                    updated_at: r.get(10)?,
                })
            }
        ).optional()?;
        Ok(res)
    }

    pub fn get_ssh_key_by_fingerprint(&self, fingerprint: &str) -> Result<Option<SshKeyRecord>, VaultError> {
        let res = self.conn.query_row(
            "SELECT id, name, group_id, public_key, fingerprint, algorithm, comment, private_key_path, attached_servers, created_at, updated_at
             FROM ssh_keys WHERE fingerprint = ?1",
            params![fingerprint],
            |r| {
                let servers_json: String = r.get(8)?;
                let servers: Vec<String> = serde_json::from_str(&servers_json).unwrap_or_default();
                Ok(SshKeyRecord {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    group_id: r.get(2)?,
                    public_key: r.get(3)?,
                    fingerprint: r.get(4)?,
                    algorithm: r.get(5)?,
                    comment: r.get(6)?,
                    private_key_path: r.get(7)?,
                    attached_servers: servers,
                    created_at: r.get(9)?,
                    updated_at: r.get(10)?,
                })
            }
        ).optional()?;
        Ok(res)
    }

    pub fn upsert_ssh_key(&self, key: &SshKeyRecord) -> Result<(), VaultError> {
        let servers_json = serde_json::to_string(&key.attached_servers).unwrap_or_else(|_| "[]".to_string());
        let now = Utc::now().to_rfc3339();
        let created = if key.created_at.is_empty() { &now } else { &key.created_at };
        self.conn.execute(
            "INSERT INTO ssh_keys (id, name, group_id, public_key, fingerprint, algorithm, comment, private_key_path, attached_servers, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 group_id = excluded.group_id,
                 public_key = excluded.public_key,
                 fingerprint = excluded.fingerprint,
                 algorithm = excluded.algorithm,
                 comment = excluded.comment,
                 private_key_path = excluded.private_key_path,
                 attached_servers = excluded.attached_servers,
                 updated_at = excluded.updated_at",
            params![
                key.id,
                key.name,
                key.group_id,
                key.public_key,
                key.fingerprint,
                key.algorithm,
                key.comment,
                key.private_key_path,
                servers_json,
                created,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn delete_ssh_key(&self, id: &str) -> Result<bool, VaultError> {
        let rows = self.conn.execute("DELETE FROM ssh_keys WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    pub fn update_ssh_key_name_and_group(&self, id: &str, name: &str, group_id: &str) -> Result<(), VaultError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE ssh_keys SET name = ?1, group_id = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, group_id, now, id],
        )?;
        Ok(())
    }

    pub fn update_ssh_key_attached_servers(&self, id: &str, servers: &[String]) -> Result<(), VaultError> {
        let now = Utc::now().to_rfc3339();
        let servers_json = serde_json::to_string(servers).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "UPDATE ssh_keys SET attached_servers = ?1, updated_at = ?2 WHERE id = ?3",
            params![servers_json, now, id],
        )?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::crypto::generate_totp_secret;

    #[test]
    fn test_vault_init_and_unlock_lifecycle() {
        let mut db = VaultDb::open_in_memory().unwrap();
        assert!(!db.is_initialized().unwrap());

        let password = "CorrectHorseBatteryStaple!";
        let key = db.init_vault(password, None).unwrap();
        assert!(db.is_initialized().unwrap());
        assert!(!db.is_totp_enabled().unwrap());

        // Storing and retrieving encrypted entry
        let secret_payload = b"Host edge-01 private ssh key rsa-ed25519";
        db.store_entry(&key, "server:edge-01", "ssh_key", "edge-01-key", secret_payload).unwrap();

        let loaded = db.load_entry(&key, "server:edge-01").unwrap();
        assert_eq!(loaded, secret_payload);

        let list = db.list_entries(None).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "server:edge-01");

        // Test unlock with correct and wrong password
        let unlocked_key = db.unlock(password, None).unwrap();
        assert_eq!(key.as_bytes(), unlocked_key.as_bytes());

        let wrong = db.unlock("wrongpassword", None);
        assert!(matches!(wrong, Err(VaultError::InvalidPassword)));
    }

    #[test]
    fn test_vault_totp_protection() {
        let mut db = VaultDb::open_in_memory().unwrap();
        let password = "MyMasterPassword456";
        let totp_secret = generate_totp_secret();

        let _ = db.init_vault(password, Some(&totp_secret)).unwrap();
        assert!(db.is_totp_enabled().unwrap());

        // Unlock without code should fail with TotpRequired
        let res_no_totp = db.unlock(password, None);
        assert!(matches!(res_no_totp, Err(VaultError::TotpRequired)));

        // Unlock with wrong code should fail
        let res_bad_code = db.unlock(password, Some("000000"));
        assert!(matches!(res_bad_code, Err(VaultError::InvalidTotpCode)));

        // Unlock with generated valid code
        let secret = totp_rs::Secret::try_from_base32(&totp_secret).unwrap();
        let totp = totp_rs::Builder::new()
            .with_secret(secret)
            .build()
            .unwrap();
        let valid_code = totp.generate_current().to_string();

        let res_valid = db.unlock(password, Some(&valid_code));
        assert!(res_valid.is_ok());
    }

    #[test]
    fn test_ssh_scan_paths_and_groups_lifecycle() {
        let db = VaultDb::open_in_memory().unwrap();

        // Check seeded scan path
        let paths = db.list_scan_paths().unwrap();
        assert!(!paths.is_empty());

        // Add custom scan path
        let added = db.add_scan_path("/custom/ssh/keys").unwrap();
        assert_eq!(added.path, "/custom/ssh/keys");

        let updated_paths = db.list_scan_paths().unwrap();
        assert_eq!(updated_paths.len(), paths.len() + 1);

        // Remove custom scan path
        let removed = db.remove_scan_path(added.id).unwrap();
        assert!(removed);
        let final_paths = db.list_scan_paths().unwrap();
        assert_eq!(final_paths.len(), paths.len());

        // Check seeded groups
        let groups = db.list_key_groups().unwrap();
        assert!(groups.iter().any(|g| g.id == "fleet"));
        assert!(groups.iter().any(|g| g.id == "bastions"));

        // Add custom group
        let custom_grp = db.add_key_group("custom-zone", "Custom Zone", "#10b981").unwrap();
        assert_eq!(custom_grp.name, "Custom Zone");

        let grp_list = db.list_key_groups().unwrap();
        assert!(grp_list.iter().any(|g| g.id == "custom-zone"));

        // Delete group
        let del = db.delete_key_group("custom-zone").unwrap();
        assert!(del);
    }

    #[test]
    fn test_ssh_keys_crud_lifecycle() {
        let db = VaultDb::open_in_memory().unwrap();

        let key = SshKeyRecord {
            id: "key-1".into(),
            name: "Fleet Ed25519".into(),
            group_id: "fleet".into(),
            public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI... nelson@crow".into(),
            fingerprint: "SHA256:49c83fbc7d9...".into(),
            algorithm: "Ed25519".into(),
            comment: Some("nelson@crow".into()),
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            attached_servers: vec!["edge-01".into(), "edge-02".into()],
            created_at: String::new(),
            updated_at: String::new(),
        };

        db.upsert_ssh_key(&key).unwrap();

        let fetched = db.get_ssh_key("key-1").unwrap().unwrap();
        assert_eq!(fetched.name, "Fleet Ed25519");
        assert_eq!(fetched.attached_servers.len(), 2);

        let by_fp = db.get_ssh_key_by_fingerprint("SHA256:49c83fbc7d9...").unwrap().unwrap();
        assert_eq!(by_fp.id, "key-1");

        // Update name and group
        db.update_ssh_key_name_and_group("key-1", "Fleet Primary Master", "bastions").unwrap();
        let updated = db.get_ssh_key("key-1").unwrap().unwrap();
        assert_eq!(updated.name, "Fleet Primary Master");
        assert_eq!(updated.group_id, "bastions");

        // Update attached servers
        db.update_ssh_key_attached_servers("key-1", &["edge-01".into(), "bastion".into(), "db-primary".into()]).unwrap();
        let updated_servers = db.get_ssh_key("key-1").unwrap().unwrap();
        assert_eq!(updated_servers.attached_servers.len(), 3);

        // Delete key
        let deleted = db.delete_ssh_key("key-1").unwrap();
        assert!(deleted);
        assert!(db.get_ssh_key("key-1").unwrap().is_none());
    }
}
