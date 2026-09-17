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

            CREATE INDEX IF NOT EXISTS idx_entries_category ON vault_entries (category);",
        )?;
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
}
