use std::path::{Path, PathBuf};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use super::crypto::{
    decrypt_data, derive_master_key, encrypt_data, generate_data_key, generate_salt, hash_password_verifier, unwrap_key, wrap_key,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerRecord {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login_user: String,
    pub auth_method: String,
    pub key_id: Option<String>,
    pub jump_host_id: Option<String>,
    pub env: String,
    pub role: String,
    pub group_name: String,
    pub tags: Vec<String>,
    pub host_key_fingerprint: Option<String>,
    pub os_distro: String,
    pub os_kernel: String,
    pub arch: String,
    pub memory_total: String,
    pub disk_total: String,
    /// Unused: Crow is agentless. Kept so existing vaults keep loading.
    pub agent_installed: bool,
    /// Unused, see `agent_installed`.
    pub agent_version: Option<String>,
    pub status: String,
    pub created_at: String,
    pub last_seen_at: Option<String>,
    /// When the server left the active fleet (ERR-32). `None` = active.
    pub archived_at: Option<String>,
    /// When Crow deleted the data it stored about this server, if the archive
    /// window has closed. The server record itself is never deleted.
    pub purged_at: Option<String>,
    /// Where the server lives (ERR-36): ISO 3166 country code ("DE"), city
    /// ("Frankfurt"), cloud provider ("Linode"), the provider's region code
    /// ("de-fra-2") and where it came from ("metadata" or "manual").
    /// Empty when unknown.
    #[serde(default)]
    pub region_country: String,
    #[serde(default)]
    pub region_city: String,
    #[serde(default)]
    pub region_provider: String,
    #[serde(default)]
    pub region_code: String,
    #[serde(default)]
    pub region_source: String,
    /// The provider account and instance this server is (ERR-46), e.g.
    /// ("linode", "123"); empty when it isn't linked to a provider.
    #[serde(default)]
    pub provider_account: String,
    #[serde(default)]
    pub provider_instance: String,
}

/// The audit-log scope for actions Crow takes on its own behalf, such as
/// purging an archived server's data. Records under this id are not tied to a
/// server, so purging a server cannot delete the record of its own purge.
pub const AUDIT_SERVER_ID: &str = "crow";

/// What one purge removed, for the audit record and the Archived tab (ERR-32).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PurgeOutcome {
    pub change_records: usize,
    pub key_attachments: usize,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ClankerProviderConfig {
    pub id: String,
    pub display_name: String,
    /// Loaded from the vault into memory only; never serialized or printed.
    #[serde(skip)]
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub is_default: bool,
    pub total_calls: u64,
    pub calls_30d: u64,
    pub last_used_at: Option<String>,
    pub daily_history: Vec<f32>,
}

/// A configured account at a provider plugin (Linode, UpCloud, ...). Plain
/// settings live here as JSON; its secrets (API tokens, passwords) are
/// encrypted vault entries under [`provider_secret_id`], never in this row.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderAccount {
    pub id: String,
    /// The plugin's manifest name, e.g. "linode".
    pub plugin: String,
    pub label: String,
    pub settings: serde_json::Map<String, serde_json::Value>,
    /// The last TEST CONNECTION: its summary or error, whether it worked, when.
    pub last_check: Option<String>,
    pub last_check_ok: bool,
    pub last_check_at: Option<String>,
}

/// Vault entry category for AI (Clankers) API keys.
pub const CLANKER_SECRET_CATEGORY: &str = "clanker_secret";

/// The vault entry id holding AI provider `id`'s API key.
pub fn clanker_secret_id(id: &str) -> String {
    format!("clanker:{id}:api_key")
}

/// Vault entry category for provider secrets.
pub const PROVIDER_SECRET_CATEGORY: &str = "provider_secret";

/// The vault entry id holding secret `key` of provider account `account`.
pub fn provider_secret_id(account: &str, key: &str) -> String {
    format!("provider:{account}:{key}")
}

impl std::fmt::Debug for ClankerProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClankerProviderConfig")
            .field("id", &self.id)
            .field("api_key", &if self.api_key.is_empty() { "" } else { "[secret]" })
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field("is_default", &self.is_default)
            .finish_non_exhaustive()
    }
}

/// A durable record of one run through the Apply Pipeline — the audit trail
/// `CROW.md` describes: what changed, on which server, whether it worked.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChangeRecord {
    pub id: String,
    pub server_id: String,
    pub server_name: String,
    pub action_kind: String,
    pub target: String,
    pub before_state: String,
    pub after_state: Option<String>,
    pub blast_radius: Option<String>,
    pub outcome: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// One recorded version of a config file on a server (ERR-72). The content
/// is sealed with the vault's data key; `sealed` is `None` when no key was
/// available, and then only the hash is known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredConfigRevision {
    pub id: String,
    pub server_id: String,
    pub path: String,
    pub sha256: String,
    pub author: String,
    pub message: String,
    /// "crow" (written by Crow) or "observed" (found on the host).
    pub source: String,
    pub created_at: String,
    /// (nonce, ciphertext)
    pub sealed: Option<(Vec<u8>, Vec<u8>)>,
}

/// Scope of a baseline that applies to every server.
pub const BASELINE_FLEET: &str = "*";

/// A known-good version of a file for a server group or the whole fleet (ERR-74).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigBaseline {
    pub path: String,
    /// A server group name, or [`BASELINE_FLEET`].
    pub scope: String,
    pub revision_id: String,
    pub set_by: String,
    pub set_at: String,
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
             PRAGMA foreign_keys = ON;
             PRAGMA secure_delete = ON;",
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
            CREATE INDEX IF NOT EXISTS idx_ssh_keys_fingerprint ON ssh_keys (fingerprint);

            CREATE TABLE IF NOT EXISTS servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                login_user TEXT NOT NULL,
                auth_method TEXT NOT NULL DEFAULT 'publickey',
                key_id TEXT,
                jump_host_id TEXT,
                env TEXT NOT NULL DEFAULT 'PROD',
                role TEXT NOT NULL DEFAULT 'generic',
                group_name TEXT NOT NULL DEFAULT 'default',
                tags TEXT NOT NULL DEFAULT '[]',
                host_key_fingerprint TEXT,
                os_distro TEXT NOT NULL DEFAULT 'Ubuntu 24.04.1 LTS',
                os_kernel TEXT NOT NULL DEFAULT '6.8.0-45-generic',
                arch TEXT NOT NULL DEFAULT 'x86_64 · 4 vCPU',
                memory_total TEXT NOT NULL DEFAULT '8.0 GB',
                disk_total TEXT NOT NULL DEFAULT '160 GB nvme',
                agent_installed INTEGER NOT NULL DEFAULT 0,
                agent_version TEXT,
                status TEXT NOT NULL DEFAULT 'online',
                created_at TEXT NOT NULL,
                last_seen_at TEXT,
                archived_at TEXT,
                purged_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_servers_env ON servers (env);
            CREATE INDEX IF NOT EXISTS idx_servers_group ON servers (group_name);

            CREATE TABLE IF NOT EXISTS clanker_providers (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                api_key TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL,
                base_url TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                total_calls INTEGER NOT NULL DEFAULT 0,
                calls_30d INTEGER NOT NULL DEFAULT 0,
                last_used_at TEXT,
                daily_history TEXT NOT NULL DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS app_flags (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS provider_accounts (
                id TEXT PRIMARY KEY,
                plugin TEXT NOT NULL,
                label TEXT NOT NULL,
                settings TEXT NOT NULL DEFAULT '{}',
                last_check TEXT,
                last_check_ok INTEGER NOT NULL DEFAULT 0,
                last_check_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS change_records (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                server_name TEXT NOT NULL,
                action_kind TEXT NOT NULL,
                target TEXT NOT NULL,
                before_state TEXT NOT NULL,
                after_state TEXT,
                blast_radius TEXT,
                outcome TEXT NOT NULL DEFAULT 'pending',
                started_at TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS config_revisions (
                seq INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT NOT NULL UNIQUE,
                server_id TEXT NOT NULL,
                path TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                author TEXT NOT NULL,
                message TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL,
                nonce BLOB,
                ciphertext BLOB
            );
            CREATE INDEX IF NOT EXISTS config_revisions_by_file ON config_revisions (server_id, path, seq);

            CREATE TABLE IF NOT EXISTS config_baselines (
                path TEXT NOT NULL,
                scope TEXT NOT NULL,
                revision_id TEXT NOT NULL,
                set_by TEXT NOT NULL,
                set_at TEXT NOT NULL,
                PRIMARY KEY (path, scope)
            );",
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

        // Seed default Clanker providers if empty
        let clanker_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM clanker_providers",
            [],
            |r| r.get(0),
        )?;
        if clanker_count == 0 {
            // No usage until something is actually sent.
            let zero = "[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]";
            let default_clankers = [
                ("openai", "OpenAI", "gpt-4o-mini", "https://api.openai.com/v1", true, 0, 0, zero),
                ("anthropic", "Anthropic", "claude-haiku-4-5-20251001", "https://api.anthropic.com/v1", false, 0, 0, zero),
                ("mistral", "Mistral AI", "mistral-small-latest", "https://api.mistral.ai/v1", false, 0, 0, zero),
                ("deepseek", "DeepSeek", "deepseek-chat", "https://api.deepseek.com/v1", false, 0, 0, zero),
                ("qwen", "Qwen (Alibaba)", "qwen-2.5-coder-32b", "https://dashscope.aliyuncs.com/compatible-mode/v1", false, 0, 0, zero),
            ];

            for (id, display_name, model, base_url, is_def, total_calls, calls_30d, daily_hist) in default_clankers {
                let _ = self.conn.execute(
                    "INSERT OR IGNORE INTO clanker_providers (
                        id, display_name, api_key, model, base_url, is_default,
                        total_calls, calls_30d, last_used_at, daily_history
                    ) VALUES (?1, ?2, '', ?3, ?4, ?5, ?6, ?7, NULL, ?8)",
                    params![
                        id,
                        display_name,
                        model,
                        base_url,
                        if is_def { 1 } else { 0 },
                        total_calls,
                        calls_30d,
                        daily_hist
                    ],
                );
            }
        }

        // ERR-32: vaults created before archiving existed have neither column,
        // and CREATE TABLE IF NOT EXISTS won't add them. A fresh vault already
        // has both, so these alters fail harmlessly there.
        // ERR-56: the vault's data key, wrapped by the password-derived key.
        // Vaults created before it have neither column; for them the
        // password-derived key is the data key.
        let _ = self.conn.execute("ALTER TABLE vault_meta ADD COLUMN wrapped_data_key BLOB", []);
        let _ = self.conn.execute("ALTER TABLE vault_meta ADD COLUMN data_key_nonce BLOB", []);
        let _ = self.conn.execute("ALTER TABLE servers ADD COLUMN archived_at TEXT", []);
        let _ = self.conn.execute("ALTER TABLE servers ADD COLUMN purged_at TEXT", []);
        // ERR-36: where each server lives.
        for col in ["region_country", "region_city", "region_provider", "region_code", "region_source", "provider_account", "provider_instance"] {
            let _ = self.conn.execute(&format!("ALTER TABLE servers ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"), []);
        }

        // Older vaults were seeded with invented usage numbers and a
        // placeholder provider. Usage with no last-used time was never real.
        let _ = self.conn.execute(
            "UPDATE clanker_providers SET total_calls = 0, calls_30d = 0, daily_history = '[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]' WHERE last_used_at IS NULL",
            [],
        );
        let _ = self.conn.execute("DELETE FROM clanker_providers WHERE id = 'xiaomi' AND api_key = ''", []);
        let _ = self.conn.execute(
            "UPDATE clanker_providers SET model = 'claude-haiku-4-5-20251001' WHERE id = 'anthropic' AND model = 'claude-3-5-sonnet-20241022' AND last_used_at IS NULL",
            [],
        );

        Ok(())
    }

    /// Counts one real request to a provider (today's bucket is the last).
    pub fn record_clanker_call(&self, id: &str) -> Result<(), VaultError> {
        let hist: String = self.conn.query_row("SELECT daily_history FROM clanker_providers WHERE id = ?1", params![id], |r| r.get(0))?;
        let mut days: Vec<f32> = serde_json::from_str(&hist).unwrap_or_default();
        match days.last_mut() {
            Some(today) => *today += 1.0,
            None => days.push(1.0),
        }
        self.conn.execute(
            "UPDATE clanker_providers SET total_calls = total_calls + 1, calls_30d = calls_30d + 1, last_used_at = ?2, daily_history = ?3 WHERE id = ?1",
            params![id, chrono::Utc::now().to_rfc3339(), serde_json::to_string(&days).unwrap_or_default()],
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
    /// Turns on the vault password. Secrets keep their data key: pass the one
    /// already in use (held by the OS keyring until now), or `None` to start
    /// a new one. The data key is stored wrapped by the password-derived key
    /// and returned for the session.
    pub fn init_vault(&mut self, password: &str, totp_secret: Option<&str>, data_key: Option<&MasterKey>) -> Result<MasterKey, VaultError> {
        if self.is_initialized()? {
            return Err(VaultError::AlreadyInitialized);
        }

        let salt = generate_salt();
        let master_key = derive_master_key(password, &salt)?;
        let data_key = data_key.cloned().unwrap_or_else(generate_data_key);
        let (wrapped_data_key, data_key_nonce) = wrap_key(&master_key, &data_key)?;
        let verifier_hash = hash_password_verifier(password)?;
        let now = Utc::now().to_rfc3339();

        let (totp_enabled, enc_secret, totp_nonce) = if let Some(secret) = totp_secret {
            let (ciphertext, nonce) = encrypt_data(&master_key, secret.as_bytes())?;
            (1, Some(ciphertext), Some(nonce.to_vec()))
        } else {
            (0, None, None)
        };

        self.conn.execute(
            "INSERT INTO vault_meta (id, salt, verifier_hash, totp_enabled, encrypted_totp_secret, totp_nonce, auto_lock_minutes, created_at, updated_at, wrapped_data_key, data_key_nonce)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, 15, ?6, ?6, ?7, ?8)",
            params![
                salt.as_slice(),
                verifier_hash,
                totp_enabled,
                enc_secret,
                totp_nonce,
                now,
                wrapped_data_key,
                data_key_nonce.as_slice(),
            ],
        )?;

        Ok(data_key)
    }

    /// Authenticates with password and optional TOTP code, returning the
    /// vault's data key on success.
    pub fn unlock(&self, password: &str, totp_code: Option<&str>) -> Result<MasterKey, VaultError> {
        let (password_key, _totp_secret) = self.authenticate(password, totp_code)?;
        self.data_key(&password_key)
    }

    /// Checks the password (and TOTP code, when enabled). Returns the
    /// password-derived key and the decrypted TOTP secret.
    fn authenticate(&self, password: &str, totp_code: Option<&str>) -> Result<(MasterKey, Option<zeroize::Zeroizing<String>>), VaultError> {
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
            let secret_base32 = zeroize::Zeroizing::new(String::from_utf8(decrypted_secret_bytes)
                .map_err(|_| VaultError::Crypto("Corrupt TOTP secret encoding".into()))?);

            if !verify_totp_code(&secret_base32, code) {
                return Err(VaultError::InvalidTotpCode);
            }
            return Ok((master_key, Some(secret_base32)));
        }
        Ok((master_key, None))
    }

    /// The data key, unwrapped with the password-derived key.
    fn data_key(&self, master_key: &MasterKey) -> Result<MasterKey, VaultError> {
        // Unwrap the data key. Vaults from before ERR-56 have none: their
        //    entries were encrypted with the password-derived key itself.
        let wrapped: (Option<Vec<u8>>, Option<Vec<u8>>) = self.conn.query_row(
            "SELECT wrapped_data_key, data_key_nonce FROM vault_meta WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        match wrapped {
            (Some(ct), Some(nonce_vec)) if nonce_vec.len() == NONCE_LEN => {
                let mut nonce = [0u8; NONCE_LEN];
                nonce.copy_from_slice(&nonce_vec);
                Ok(unwrap_key(master_key, &ct, &nonce)?)
            }
            _ => self.upgrade_to_data_key(master_key),
        }
    }

    /// Changes the vault password. Only the data key (and the TOTP secret)
    /// are re-encrypted, under a key from the new password; entries don't
    /// change. Needs the current password and 2FA code.
    pub fn change_password(&self, current: &str, totp_code: Option<&str>, new_password: &str) -> Result<MasterKey, VaultError> {
        if new_password.chars().count() < 8 {
            return Err(VaultError::Crypto("the new password must be at least 8 characters".into()));
        }
        let (old_key, totp_secret) = self.authenticate(current, totp_code)?;
        let data_key = self.data_key(&old_key)?;
        let salt = generate_salt();
        let new_key = derive_master_key(new_password, &salt)?;
        let verifier = hash_password_verifier(new_password)?;
        let (wrapped, nonce) = wrap_key(&new_key, &data_key)?;
        let totp = totp_secret.map(|t| encrypt_data(&new_key, t.as_bytes())).transpose()?;
        self.conn.execute(
            "UPDATE vault_meta SET salt = ?1, verifier_hash = ?2, wrapped_data_key = ?3, data_key_nonce = ?4,
                 encrypted_totp_secret = COALESCE(?5, encrypted_totp_secret), totp_nonce = COALESCE(?6, totp_nonce), updated_at = ?7
             WHERE id = 1",
            params![salt.as_slice(), verifier, wrapped, nonce.as_slice(), totp.as_ref().map(|(c, _)| c.clone()), totp.as_ref().map(|(_, n)| n.to_vec()), Utc::now().to_rfc3339()],
        )?;
        Ok(data_key)
    }

    /// Turning the password off, step one: checks the password and 2FA code
    /// and returns the data key, which the caller must put in the OS
    /// keyring before calling [`Self::remove_password`].
    pub fn data_key_for_removal(&self, password: &str, totp_code: Option<&str>) -> Result<MasterKey, VaultError> {
        let (key, _) = self.authenticate(password, totp_code)?;
        self.data_key(&key)
    }

    /// Turning the password off, step two: forgets the password (and the
    /// wrapped data key). Only call once the keyring holds the data key.
    pub fn remove_password(&self) -> Result<(), VaultError> {
        self.conn.execute("DELETE FROM vault_meta WHERE id = 1", [])?;
        Ok(())
    }

    /// Moves a vault from before ERR-56 (entries encrypted with the
    /// password-derived key) to a data key: re-encrypts every entry and
    /// stores the new key wrapped, in one transaction. Returns the data key.
    fn upgrade_to_data_key(&self, password_key: &MasterKey) -> Result<MasterKey, VaultError> {
        let data_key = generate_data_key();
        let tx = self.conn.unchecked_transaction()?;
        let entries: Vec<(String, Vec<u8>, Vec<u8>)> = {
            let mut stmt = tx.prepare("SELECT id, nonce, ciphertext FROM vault_entries")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        for (id, nonce_vec, ciphertext) in entries {
            let nonce: [u8; NONCE_LEN] = nonce_vec.try_into().map_err(|_| VaultError::Crypto(format!("entry {id} has a bad nonce")))?;
            let mut plain = decrypt_data(password_key, &ciphertext, &nonce)?;
            let sealed = encrypt_data(&data_key, &plain);
            plain.zeroize();
            let (ct, n) = sealed?;
            tx.execute("UPDATE vault_entries SET nonce = ?2, ciphertext = ?3 WHERE id = ?1", params![id, n.as_slice(), ct])?;
        }
        let (wrapped, nonce) = wrap_key(password_key, &data_key)?;
        tx.execute("UPDATE vault_meta SET wrapped_data_key = ?1, data_key_nonce = ?2 WHERE id = 1", params![wrapped, nonce.as_slice()])?;
        tx.commit()?;
        Ok(data_key)
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

    pub fn attach_server_to_key(&self, key_id: &str, server_name: &str) -> Result<(), VaultError> {
        if let Some(mut key) = self.get_ssh_key(key_id)? {
            if !key.attached_servers.iter().any(|s| s == server_name) {
                key.attached_servers.push(server_name.to_string());
                self.update_ssh_key_attached_servers(key_id, &key.attached_servers)?;
            }
        }
        Ok(())
    }

    /// The columns every server query selects, in the order [`Self::row_to_server`] reads.
    const SERVER_COLUMNS: &'static str = "id, name, host, port, login_user, auth_method, key_id, jump_host_id,
                env, role, group_name, tags, host_key_fingerprint, os_distro, os_kernel,
                arch, memory_total, disk_total, agent_installed, agent_version, status,
                created_at, last_seen_at, archived_at, purged_at,
                region_country, region_city, region_provider, region_code, region_source,
                provider_account, provider_instance";

    fn row_to_server(r: &rusqlite::Row<'_>) -> rusqlite::Result<ServerRecord> {
        let tags_json: String = r.get(11)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let agent_inst: i64 = r.get(18)?;
        Ok(ServerRecord {
            id: r.get(0)?,
            name: r.get(1)?,
            host: r.get(2)?,
            port: r.get::<_, u16>(3)?,
            login_user: r.get(4)?,
            auth_method: r.get(5)?,
            key_id: r.get(6)?,
            jump_host_id: r.get(7)?,
            env: r.get(8)?,
            role: r.get(9)?,
            group_name: r.get(10)?,
            tags,
            host_key_fingerprint: r.get(12)?,
            os_distro: r.get(13)?,
            os_kernel: r.get(14)?,
            arch: r.get(15)?,
            memory_total: r.get(16)?,
            disk_total: r.get(17)?,
            agent_installed: agent_inst != 0,
            agent_version: r.get(19)?,
            status: r.get(20)?,
            created_at: r.get(21)?,
            last_seen_at: r.get(22)?,
            archived_at: r.get(23)?,
            purged_at: r.get(24)?,
            region_country: r.get(25)?,
            region_city: r.get(26)?,
            region_provider: r.get(27)?,
            region_code: r.get(28)?,
            region_source: r.get(29)?,
            provider_account: r.get(30)?,
            provider_instance: r.get(31)?,
        })
    }

    /// Servers in the active fleet — archived ones are excluded, so every
    /// caller that lists the fleet gets that for free (ERR-32).
    pub fn list_servers(&self) -> Result<Vec<ServerRecord>, VaultError> {
        self.query_servers("WHERE archived_at IS NULL ORDER BY created_at ASC")
    }

    /// Archived servers, most recently archived first — the Archived tab.
    pub fn list_archived_servers(&self) -> Result<Vec<ServerRecord>, VaultError> {
        self.query_servers("WHERE archived_at IS NOT NULL ORDER BY archived_at DESC")
    }

    fn query_servers(&self, tail: &str) -> Result<Vec<ServerRecord>, VaultError> {
        let sql = format!("SELECT {} FROM servers {}", Self::SERVER_COLUMNS, tail);
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |r| Self::row_to_server(r))?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn get_server(&self, id: &str) -> Result<Option<ServerRecord>, VaultError> {
        let sql = format!("SELECT {} FROM servers WHERE id = ?1", Self::SERVER_COLUMNS);
        let mut stmt = self.conn.prepare(&sql)?;
        let res = stmt.query_row(params![id], |r| Self::row_to_server(r)).optional()?;
        Ok(res)
    }

    pub fn upsert_server(&self, srv: &ServerRecord) -> Result<(), VaultError> {
        let tags_json = serde_json::to_string(&srv.tags).unwrap_or_else(|_| "[]".to_string());
        let now = Utc::now().to_rfc3339();
        let created = if srv.created_at.is_empty() { &now } else { &srv.created_at };
        let agent_inst = if srv.agent_installed { 1 } else { 0 };

        self.conn.execute(
            "INSERT INTO servers (
                id, name, host, port, login_user, auth_method, key_id, jump_host_id,
                env, role, group_name, tags, host_key_fingerprint, os_distro, os_kernel,
                arch, memory_total, disk_total, agent_installed, agent_version, status,
                created_at, last_seen_at, archived_at, purged_at,
                region_country, region_city, region_provider, region_code, region_source,
                provider_account, provider_instance
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                host = excluded.host,
                port = excluded.port,
                login_user = excluded.login_user,
                auth_method = excluded.auth_method,
                key_id = excluded.key_id,
                jump_host_id = excluded.jump_host_id,
                env = excluded.env,
                role = excluded.role,
                group_name = excluded.group_name,
                tags = excluded.tags,
                host_key_fingerprint = excluded.host_key_fingerprint,
                os_distro = excluded.os_distro,
                os_kernel = excluded.os_kernel,
                arch = excluded.arch,
                memory_total = excluded.memory_total,
                disk_total = excluded.disk_total,
                agent_installed = excluded.agent_installed,
                agent_version = excluded.agent_version,
                status = excluded.status,
                last_seen_at = excluded.last_seen_at,
                archived_at = excluded.archived_at,
                purged_at = excluded.purged_at,
                region_country = excluded.region_country,
                region_city = excluded.region_city,
                region_provider = excluded.region_provider,
                region_code = excluded.region_code,
                region_source = excluded.region_source,
                provider_account = excluded.provider_account,
                provider_instance = excluded.provider_instance",
            params![
                srv.id, srv.name, srv.host, srv.port, srv.login_user, srv.auth_method,
                srv.key_id, srv.jump_host_id, srv.env, srv.role, srv.group_name,
                tags_json, srv.host_key_fingerprint, srv.os_distro, srv.os_kernel,
                srv.arch, srv.memory_total, srv.disk_total, agent_inst,
                srv.agent_version, srv.status, created, srv.last_seen_at,
                srv.archived_at, srv.purged_at,
                srv.region_country, srv.region_city, srv.region_provider, srv.region_code, srv.region_source,
                srv.provider_account, srv.provider_instance
            ],
        )?;
        Ok(())
    }

    /// Takes a server out of the active fleet (ERR-32). The record stays; only
    /// its `archived_at` changes. Idempotent.
    pub fn archive_server(&self, id: &str) -> Result<(), VaultError> {
        self.conn.execute(
            "UPDATE servers SET archived_at = ?1, status = 'archived' WHERE id = ?2 AND archived_at IS NULL",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// Puts an archived server back in the fleet with whatever history the
    /// purge has not taken yet. Its status is unknown until Crow next reaches
    /// it, so it is reported as such rather than as a stale "online".
    pub fn restore_server(&self, id: &str) -> Result<(), VaultError> {
        self.conn.execute(
            "UPDATE servers SET archived_at = NULL, status = 'unknown' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Deletes the data Crow stored *about* `id`: its change records and its
    /// attachments on enrolled keys. The server record itself is never
    /// deleted. `purged_at` is set last, so a failure part-way through leaves
    /// the purge pending and it is retried on the next run.
    ///
    /// Attachments are stored by server id from the Keys screen but by server
    /// name from onboarding (`attach_server_to_key` takes a name), so both
    /// spellings are removed.
    pub fn purge_server_data(&self, id: &str, name: &str) -> Result<PurgeOutcome, VaultError> {
        let change_records = self.conn.execute("DELETE FROM change_records WHERE server_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM config_revisions WHERE server_id = ?1", params![id])?;

        let keys: Vec<(String, Vec<String>)> = {
            let mut stmt = self.conn.prepare("SELECT id, attached_servers FROM ssh_keys")?;
            let rows = stmt.query_map([], |r| {
                let id: String = r.get(0)?;
                let json: String = r.get(1)?;
                Ok((id, serde_json::from_str::<Vec<String>>(&json).unwrap_or_default()))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let mut key_attachments = 0;
        for (key_id, mut attached) in keys {
            let before = attached.len();
            attached.retain(|s| s != id && s != name);
            if attached.len() != before {
                key_attachments += before - attached.len();
                self.update_ssh_key_attached_servers(&key_id, &attached)?;
            }
        }

        self.conn.execute(
            "UPDATE servers SET purged_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(PurgeOutcome { change_records, key_attachments })
    }

    /// Archived servers whose window has closed and whose data has not been
    /// purged yet. `cutoff` is an RFC 3339 instant; anything archived at or
    /// before it is due.
    pub fn servers_due_for_purge(&self, cutoff: &str) -> Result<Vec<ServerRecord>, VaultError> {
        let sql = format!(
            "SELECT {} FROM servers WHERE archived_at IS NOT NULL AND purged_at IS NULL AND archived_at <= ?1",
            Self::SERVER_COLUMNS
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![cutoff], |r| Self::row_to_server(r))?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    /// Deleting a server record outright is never a user-facing operation
    /// (ERR-32): servers are archived instead. This stays for tests that need
    /// a clean vault, and cannot be reached from the app.
    #[cfg(test)]
    pub fn delete_server(&self, id: &str) -> Result<(), VaultError> {
        self.conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_server_status(&self, id: &str, status: &str) -> Result<(), VaultError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE servers SET status = ?1, last_seen_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;
        Ok(())
    }

    // ==========================================
    // Clankers AI Providers Storage & Metrics
    // ==========================================

    pub fn list_clanker_providers(&self) -> Result<Vec<ClankerProviderConfig>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, display_name, api_key, model, base_url, is_default, total_calls, calls_30d, last_used_at, daily_history
             FROM clanker_providers ORDER BY CASE id
                WHEN 'openai' THEN 1
                WHEN 'anthropic' THEN 2
                WHEN 'mistral' THEN 3
                WHEN 'deepseek' THEN 4
                WHEN 'xiaomi' THEN 5
                WHEN 'qwen' THEN 6
                ELSE 7 END"
        )?;

        let rows = stmt.query_map([], |r| {
            let is_def_int: i64 = r.get(5)?;
            let total_calls: i64 = r.get(6)?;
            let calls_30d: i64 = r.get(7)?;
            let hist_json: String = r.get(9)?;
            let daily_history: Vec<f32> = serde_json::from_str(&hist_json).unwrap_or_default();

            Ok(ClankerProviderConfig {
                id: r.get(0)?,
                display_name: r.get(1)?,
                api_key: r.get(2)?,
                model: r.get(3)?,
                base_url: r.get(4)?,
                is_default: is_def_int > 0,
                total_calls: total_calls.max(0) as u64,
                calls_30d: calls_30d.max(0) as u64,
                last_used_at: r.get(8)?,
                daily_history,
            })
        })?;

        let mut out = Vec::new();
        for item in rows {
            out.push(item?);
        }
        Ok(out)
    }

    pub fn get_clanker_provider(&self, id: &str) -> Result<Option<ClankerProviderConfig>, VaultError> {
        let res = self.conn.query_row(
            "SELECT id, display_name, api_key, model, base_url, is_default, total_calls, calls_30d, last_used_at, daily_history
             FROM clanker_providers WHERE id = ?1",
            params![id],
            |r| {
                let is_def_int: i64 = r.get(5)?;
                let total_calls: i64 = r.get(6)?;
                let calls_30d: i64 = r.get(7)?;
                let hist_json: String = r.get(9)?;
                let daily_history: Vec<f32> = serde_json::from_str(&hist_json).unwrap_or_default();

                Ok(ClankerProviderConfig {
                    id: r.get(0)?,
                    display_name: r.get(1)?,
                    api_key: r.get(2)?,
                    model: r.get(3)?,
                    base_url: r.get(4)?,
                    is_default: is_def_int > 0,
                    total_calls: total_calls.max(0) as u64,
                    calls_30d: calls_30d.max(0) as u64,
                    last_used_at: r.get(8)?,
                    daily_history,
                })
            }
        ).optional()?;
        Ok(res)
    }

    pub fn upsert_clanker_provider(&self, config: &ClankerProviderConfig) -> Result<(), VaultError> {
        let hist_json = serde_json::to_string(&config.daily_history).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT INTO clanker_providers (
                id, display_name, api_key, model, base_url, is_default, total_calls, calls_30d, last_used_at, daily_history
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                model = excluded.model,
                base_url = excluded.base_url,
                is_default = excluded.is_default,
                total_calls = excluded.total_calls,
                calls_30d = excluded.calls_30d,
                last_used_at = excluded.last_used_at,
                daily_history = excluded.daily_history",
            params![
                config.id,
                config.display_name,
                // Keys live in the vault (set_clanker_api_key), never here.
                "",
                config.model,
                config.base_url,
                if config.is_default { 1 } else { 0 },
                config.total_calls as i64,
                config.calls_30d as i64,
                config.last_used_at,
                hist_json
            ],
        )?;
        Ok(())
    }

    pub fn set_default_clanker_provider(&self, provider_id: &str) -> Result<(), VaultError> {
        self.conn.execute("UPDATE clanker_providers SET is_default = 0", [])?;
        self.conn.execute("UPDATE clanker_providers SET is_default = 1 WHERE id = ?1", params![provider_id])?;
        Ok(())
    }

    pub fn record_clanker_usage(&self, provider_id: &str) -> Result<(), VaultError> {
        let now = Utc::now().to_rfc3339();
        if let Some(mut current) = self.get_clanker_provider(provider_id)? {
            current.total_calls += 1;
            current.calls_30d += 1;
            current.last_used_at = Some(now);
            if current.daily_history.is_empty() {
                current.daily_history = vec![0.0; 29];
                current.daily_history.push(1.0);
            } else {
                let last_idx = current.daily_history.len() - 1;
                current.daily_history[last_idx] += 1.0;
            }
            self.upsert_clanker_provider(&current)?;
        }
        Ok(())
    }

    pub fn reset_clanker_usage(&self, provider_id: &str) -> Result<(), VaultError> {
        self.conn.execute(
            "UPDATE clanker_providers SET total_calls = 0, calls_30d = 0, last_used_at = NULL, daily_history = ?1 WHERE id = ?2",
            params!["[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]", provider_id],
        )?;
        Ok(())
    }

    // ==========================================
    // Apply Pipeline Change Records
    // ==========================================

    pub fn insert_change_record(&self, rec: &ChangeRecord) -> Result<(), VaultError> {
        self.conn.execute(
            "INSERT INTO change_records
                (id, server_id, server_name, action_kind, target, before_state, after_state, blast_radius, outcome, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                rec.id,
                rec.server_id,
                rec.server_name,
                rec.action_kind,
                rec.target,
                rec.before_state,
                rec.after_state,
                rec.blast_radius,
                rec.outcome,
                rec.started_at,
                rec.completed_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_change_record_outcome(
        &self,
        id: &str,
        outcome: &str,
        after_state: Option<&str>,
        completed_at: &str,
    ) -> Result<(), VaultError> {
        self.conn.execute(
            "UPDATE change_records SET outcome = ?1, after_state = ?2, completed_at = ?3 WHERE id = ?4",
            params![outcome, after_state, completed_at, id],
        )?;
        Ok(())
    }

    pub fn list_change_records(&self, server_id: &str, limit: usize) -> Result<Vec<ChangeRecord>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, server_id, server_name, action_kind, target, before_state, after_state, blast_radius, outcome, started_at, completed_at
             FROM change_records WHERE server_id = ?1 ORDER BY started_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![server_id, limit as i64], |r| {
            Ok(ChangeRecord {
                id: r.get(0)?,
                server_id: r.get(1)?,
                server_name: r.get(2)?,
                action_kind: r.get(3)?,
                target: r.get(4)?,
                before_state: r.get(5)?,
                after_state: r.get(6)?,
                blast_radius: r.get(7)?,
                outcome: r.get(8)?,
                started_at: r.get(9)?,
                completed_at: r.get(10)?,
            })
        })?;

        let mut out = Vec::new();
        for item in rows {
            out.push(item?);
        }
        Ok(out)
    }

    // ==========================================
    // Config history (ERR-72)
    // ==========================================

    pub fn insert_config_revision(&self, rev: &StoredConfigRevision) -> Result<(), VaultError> {
        let (nonce, ciphertext) = match &rev.sealed {
            Some((n, c)) => (Some(n.as_slice()), Some(c.as_slice())),
            None => (None, None),
        };
        self.conn.execute(
            "INSERT INTO config_revisions (id, server_id, path, sha256, author, message, source, created_at, nonce, ciphertext)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![rev.id, rev.server_id, rev.path, rev.sha256, rev.author, rev.message, rev.source, rev.created_at, nonce, ciphertext],
        )?;
        Ok(())
    }

    /// A file's revisions on a server, oldest first.
    pub fn list_config_revisions(&self, server_id: &str, path: &str) -> Result<Vec<StoredConfigRevision>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, server_id, path, sha256, author, message, source, created_at, nonce, ciphertext
             FROM config_revisions WHERE server_id = ?1 AND path = ?2 ORDER BY seq",
        )?;
        let rows = stmt.query_map(params![server_id, path], |r| {
            let nonce: Option<Vec<u8>> = r.get(8)?;
            let ciphertext: Option<Vec<u8>> = r.get(9)?;
            Ok(StoredConfigRevision {
                id: r.get(0)?,
                server_id: r.get(1)?,
                path: r.get(2)?,
                sha256: r.get(3)?,
                author: r.get(4)?,
                message: r.get(5)?,
                source: r.get(6)?,
                created_at: r.get(7)?,
                sealed: nonce.zip(ciphertext),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// One revision by id.
    pub fn get_config_revision(&self, id: &str) -> Result<Option<StoredConfigRevision>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, server_id, path, sha256, author, message, source, created_at, nonce, ciphertext FROM config_revisions WHERE id = ?1",
        )?;
        Ok(stmt.query_row(params![id], Self::config_revision_row).optional()?)
    }

    /// The latest revision of `path` on every server that has one.
    pub fn latest_config_revisions_for_path(&self, path: &str) -> Result<Vec<StoredConfigRevision>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, server_id, path, sha256, author, message, source, created_at, nonce, ciphertext FROM config_revisions r
             WHERE path = ?1 AND seq = (SELECT MAX(seq) FROM config_revisions WHERE server_id = r.server_id AND path = r.path)",
        )?;
        let rows = stmt.query_map(params![path], Self::config_revision_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn config_revision_row(r: &rusqlite::Row) -> rusqlite::Result<StoredConfigRevision> {
        let nonce: Option<Vec<u8>> = r.get(8)?;
        let ciphertext: Option<Vec<u8>> = r.get(9)?;
        Ok(StoredConfigRevision {
            id: r.get(0)?,
            server_id: r.get(1)?,
            path: r.get(2)?,
            sha256: r.get(3)?,
            author: r.get(4)?,
            message: r.get(5)?,
            source: r.get(6)?,
            created_at: r.get(7)?,
            sealed: nonce.zip(ciphertext),
        })
    }

    /// Pins `revision_id` as the known-good `path` for `scope` (a server
    /// group, or [`BASELINE_FLEET`]) (ERR-74).
    pub fn set_config_baseline(&self, path: &str, scope: &str, revision_id: &str, set_by: &str) -> Result<(), VaultError> {
        self.conn.execute(
            "INSERT INTO config_baselines (path, scope, revision_id, set_by, set_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(path, scope) DO UPDATE SET revision_id = excluded.revision_id, set_by = excluded.set_by, set_at = excluded.set_at",
            params![path, scope, revision_id, set_by, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn clear_config_baseline(&self, path: &str, scope: &str) -> Result<(), VaultError> {
        self.conn.execute("DELETE FROM config_baselines WHERE path = ?1 AND scope = ?2", params![path, scope])?;
        Ok(())
    }

    pub fn list_config_baselines(&self) -> Result<Vec<ConfigBaseline>, VaultError> {
        let mut stmt = self.conn.prepare("SELECT path, scope, revision_id, set_by, set_at FROM config_baselines ORDER BY path, scope")?;
        let rows = stmt.query_map([], |r| {
            Ok(ConfigBaseline { path: r.get(0)?, scope: r.get(1)?, revision_id: r.get(2)?, set_by: r.get(3)?, set_at: r.get(4)? })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// The hash of each file's latest revision on a server, by path.
    pub fn latest_config_hashes(&self, server_id: &str) -> Result<std::collections::HashMap<String, String>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT path, sha256 FROM config_revisions r
             WHERE server_id = ?1 AND seq = (SELECT MAX(seq) FROM config_revisions WHERE server_id = r.server_id AND path = r.path)",
        )?;
        let rows = stmt.query_map(params![server_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Change records across every server (and Crow's own), newest first.
    pub fn list_all_change_records(&self, limit: usize) -> Result<Vec<ChangeRecord>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, server_id, server_name, action_kind, target, before_state, after_state, blast_radius, outcome, started_at, completed_at
             FROM change_records ORDER BY started_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok(ChangeRecord {
                id: r.get(0)?,
                server_id: r.get(1)?,
                server_name: r.get(2)?,
                action_kind: r.get(3)?,
                target: r.get(4)?,
                before_state: r.get(5)?,
                after_state: r.get(6)?,
                blast_radius: r.get(7)?,
                outcome: r.get(8)?,
                started_at: r.get(9)?,
                completed_at: r.get(10)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Stores (or with `None`, removes) AI provider `id`'s API key as an
    /// encrypted vault entry.
    pub fn set_clanker_api_key(&self, key: &MasterKey, id: &str, api_key: Option<&str>) -> Result<(), VaultError> {
        match api_key.map(str::trim).filter(|k| !k.is_empty()) {
            Some(k) => self.store_entry(key, &clanker_secret_id(id), CLANKER_SECRET_CATEGORY, &format!("{id} API key"), k.as_bytes()),
            None => self.delete_entry(&clanker_secret_id(id)).map(drop),
        }
    }

    /// Fills each provider's `api_key` from the vault (in memory only).
    pub fn load_clanker_api_keys(&self, key: &MasterKey, providers: &mut [ClankerProviderConfig]) {
        for p in providers {
            if let Ok(bytes) = self.load_entry(key, &clanker_secret_id(&p.id)) {
                p.api_key = String::from_utf8(bytes).unwrap_or_default();
            }
        }
    }

    pub fn flag(&self, key: &str) -> Option<String> {
        self.conn.query_row("SELECT value FROM app_flags WHERE key = ?1", params![key], |r| r.get(0)).optional().ok().flatten()
    }

    pub fn set_flag(&self, key: &str, value: &str) -> Result<(), VaultError> {
        self.conn.execute("INSERT INTO app_flags (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value", params![key, value])?;
        Ok(())
    }

    /// Whether older versions left any AI key in plain text.
    pub fn has_plaintext_clanker_keys(&self) -> bool {
        self.conn.query_row("SELECT EXISTS(SELECT 1 FROM clanker_providers WHERE api_key != '')", [], |r| r.get::<_, bool>(0)).unwrap_or(false)
    }

    /// Moves API keys left in plain text by older versions into the vault,
    /// blanks the column and rewrites the database file so the plain text
    /// doesn't survive in free pages or the WAL. Returns how many moved.
    pub fn migrate_plaintext_clanker_keys(&self, key: &MasterKey) -> Result<usize, VaultError> {
        let plain: Vec<(String, String)> = {
            let mut stmt = self.conn.prepare("SELECT id, api_key FROM clanker_providers WHERE api_key != ''")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        for (id, api_key) in &plain {
            self.set_clanker_api_key(key, id, Some(api_key))?;
            self.conn.execute("UPDATE clanker_providers SET api_key = '' WHERE id = ?1", params![id])?;
        }
        if !plain.is_empty() {
            self.conn.execute_batch("VACUUM; PRAGMA wal_checkpoint(TRUNCATE);")?;
        }
        Ok(plain.len())
    }

    pub fn list_provider_accounts(&self) -> Result<Vec<ProviderAccount>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plugin, label, settings, last_check, last_check_ok, last_check_at FROM provider_accounts ORDER BY plugin, created_at",
        )?;
        let rows = stmt.query_map([], |r| {
            let settings: String = r.get(3)?;
            let ok: i64 = r.get(5)?;
            Ok(ProviderAccount {
                id: r.get(0)?,
                plugin: r.get(1)?,
                label: r.get(2)?,
                settings: serde_json::from_str(&settings).unwrap_or_default(),
                last_check: r.get(4)?,
                last_check_ok: ok > 0,
                last_check_at: r.get(6)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Saves an account's label and plain settings (not its check result).
    pub fn upsert_provider_account(&self, account: &ProviderAccount) -> Result<(), VaultError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO provider_accounts (id, plugin, label, settings, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(id) DO UPDATE SET label = excluded.label, settings = excluded.settings, updated_at = excluded.updated_at",
            params![account.id, account.plugin, account.label, serde_json::Value::Object(account.settings.clone()).to_string(), now],
        )?;
        Ok(())
    }

    pub fn record_provider_check(&self, id: &str, ok: bool, message: &str) -> Result<(), VaultError> {
        self.conn.execute(
            "UPDATE provider_accounts SET last_check = ?2, last_check_ok = ?3, last_check_at = ?4 WHERE id = ?1",
            params![id, message, ok as i64, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Deletes an account and every secret it kept in the vault.
    pub fn delete_provider_account(&self, id: &str) -> Result<(), VaultError> {
        self.conn.execute("DELETE FROM provider_accounts WHERE id = ?1", params![id])?;
        self.conn.execute(
            "DELETE FROM vault_entries WHERE category = ?1 AND substr(id, 1, length(?2)) = ?2",
            params![PROVIDER_SECRET_CATEGORY, provider_secret_id(id, "")],
        )?;
        Ok(())
    }

    /// Keys of the secrets account `id` has in the vault (their values stay
    /// encrypted; this reads ids only).
    pub fn provider_secret_keys(&self, id: &str) -> Result<Vec<String>, VaultError> {
        let prefix = provider_secret_id(id, "");
        Ok(self
            .list_entries(Some(PROVIDER_SECRET_CATEGORY))?
            .into_iter()
            .filter_map(|e| e.id.strip_prefix(&prefix).map(str::to_string))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::crypto::generate_totp_secret;

    #[test]
    fn data_key_survives_the_password_and_wraps_under_it() {
        let mut db = VaultDb::open_in_memory().unwrap();
        // Secrets stored while the password was off (key from the keyring)...
        let keyring_key = crate::vault::generate_data_key();
        db.store_entry(&keyring_key, "provider:linode:api_token", PROVIDER_SECRET_CATEGORY, "t", b"tok").unwrap();
        // ...stay readable once a password is turned on: same data key.
        let session = db.init_vault("pw", None, Some(&keyring_key)).unwrap();
        assert_eq!(session.as_bytes(), keyring_key.as_bytes());
        let unlocked = db.unlock("pw", None).unwrap();
        assert_eq!(db.load_entry(&unlocked, "provider:linode:api_token").unwrap(), b"tok");
        // The data key isn't the password-derived key.
        let (salt, wrapped): (Vec<u8>, Vec<u8>) = db.conn.query_row("SELECT salt, wrapped_data_key FROM vault_meta", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        let kek = derive_master_key("pw", &salt.try_into().unwrap()).unwrap();
        assert_ne!(kek.as_bytes(), unlocked.as_bytes());
        assert!(!wrapped.windows(8).any(|w| unlocked.as_bytes().windows(8).any(|k| k == w)), "stored wrapped, not raw");
    }

    fn current_code(secret: &str) -> String {
        totp_rs::Builder::new().with_secret(totp_rs::Secret::try_from_base32(secret).unwrap()).build().unwrap().generate_current().to_string()
    }

    #[test]
    fn changing_the_password_keeps_the_secrets_and_the_2fa() {
        let mut db = VaultDb::open_in_memory().unwrap();
        let totp = generate_totp_secret();
        let key = db.init_vault("old-password", Some(&totp), None).unwrap();
        db.store_entry(&key, "provider:linode:api_token", PROVIDER_SECRET_CATEGORY, "t", b"tok").unwrap();

        assert!(matches!(db.change_password("wrong", Some(&current_code(&totp)), "new-password"), Err(VaultError::InvalidPassword)));
        assert!(matches!(db.change_password("old-password", Some("000000"), "new-password"), Err(VaultError::InvalidTotpCode)));
        assert!(db.change_password("old-password", Some(&current_code(&totp)), "short").is_err());

        db.change_password("old-password", Some(&current_code(&totp)), "new-password").unwrap();
        assert!(matches!(db.unlock("old-password", Some(&current_code(&totp))), Err(VaultError::InvalidPassword)));
        let after = db.unlock("new-password", Some(&current_code(&totp))).unwrap();
        assert_eq!(after.as_bytes(), key.as_bytes(), "same data key");
        assert_eq!(db.load_entry(&after, "provider:linode:api_token").unwrap(), b"tok");
    }

    #[test]
    fn turning_the_password_off_hands_back_the_data_key() {
        let mut db = VaultDb::open_in_memory().unwrap();
        let totp = generate_totp_secret();
        let key = db.init_vault("password", Some(&totp), None).unwrap();
        assert!(db.data_key_for_removal("password", Some("000000")).is_err());
        let handed = db.data_key_for_removal("password", Some(&current_code(&totp))).unwrap();
        assert_eq!(handed.as_bytes(), key.as_bytes());
        db.remove_password().unwrap();
        assert!(!db.is_initialized().unwrap());
    }

    #[test]
    fn legacy_vaults_move_to_a_data_key_on_unlock() {
        let mut db = VaultDb::open_in_memory().unwrap();
        db.init_vault("pw", None, None).unwrap();
        // As a pre-ERR-56 vault: no wrapped key, entries under the password key.
        db.conn.execute("UPDATE vault_meta SET wrapped_data_key = NULL, data_key_nonce = NULL", []).unwrap();
        let (salt,): (Vec<u8>,) = db.conn.query_row("SELECT salt FROM vault_meta", [], |r| Ok((r.get(0)?,))).unwrap();
        let kek = derive_master_key("pw", &salt.try_into().unwrap()).unwrap();
        db.store_entry(&kek, "server:edge-01", "ssh_key", "k", b"old secret").unwrap();

        let key = db.unlock("pw", None).unwrap();
        assert_ne!(key.as_bytes(), kek.as_bytes(), "a fresh data key, not the password key");
        assert_eq!(db.load_entry(&key, "server:edge-01").unwrap(), b"old secret", "entries re-encrypted");
        assert!(db.load_entry(&kek, "server:edge-01").is_err(), "the password key no longer opens them");
        assert_eq!(db.unlock("pw", None).unwrap().as_bytes(), key.as_bytes(), "later unlocks unwrap the same key");
    }

    #[test]
    fn plaintext_ai_keys_move_into_the_vault() {
        let db = VaultDb::open_in_memory().unwrap();
        let key = crate::vault::generate_data_key();
        // As an older Crow left it: the key in plain text in the row.
        db.conn.execute("UPDATE clanker_providers SET api_key = 'sk-ant-PLAIN' WHERE id = 'anthropic'", []).unwrap();
        assert_eq!(db.migrate_plaintext_clanker_keys(&key).unwrap(), 1);
        let mut providers = db.list_clanker_providers().unwrap();
        let p = providers.iter().find(|p| p.id == "anthropic").unwrap();
        assert_eq!(p.api_key, "", "the column is blank");
        db.load_clanker_api_keys(&key, &mut providers);
        assert_eq!(providers.iter().find(|p| p.id == "anthropic").unwrap().api_key, "sk-ant-PLAIN");
        assert_eq!(db.migrate_plaintext_clanker_keys(&key).unwrap(), 0, "runs once");
        // Saving a provider never writes a key back into the column.
        let mut p = providers.into_iter().find(|p| p.id == "anthropic").unwrap();
        p.api_key = "sk-ant-NEW".into();
        db.upsert_clanker_provider(&p).unwrap();
        let raw: String = db.conn.query_row("SELECT api_key FROM clanker_providers WHERE id = 'anthropic'", [], |r| r.get(0)).unwrap();
        assert_eq!(raw, "");
    }

    #[test]
    fn provider_accounts_keep_secrets_out_of_their_row() {
        let mut db = VaultDb::open_in_memory().unwrap();
        let key = db.init_vault("pw", None, None).unwrap();
        let mut acct = ProviderAccount {
            id: "linode".into(),
            plugin: "linode".into(),
            label: "Linode".into(),
            settings: serde_json::Map::new(),
            last_check: None,
            last_check_ok: false,
            last_check_at: None,
        };
        acct.settings.insert("page_size".into(), serde_json::json!(100));
        db.upsert_provider_account(&acct).unwrap();
        db.store_entry(&key, &provider_secret_id("linode", "api_token"), PROVIDER_SECRET_CATEGORY, "linode api_token", b"tok").unwrap();
        db.store_entry(&key, &provider_secret_id("linode-2", "api_token"), PROVIDER_SECRET_CATEGORY, "other", b"tok2").unwrap();
        db.record_provider_check("linode", true, "3 instances").unwrap();

        let listed = db.list_provider_accounts().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!((listed[0].settings["page_size"].clone(), listed[0].last_check.as_deref(), listed[0].last_check_ok), (serde_json::json!(100), Some("3 instances"), true));
        assert_eq!(db.provider_secret_keys("linode").unwrap(), ["api_token"]);

        db.delete_provider_account("linode").unwrap();
        assert!(db.list_provider_accounts().unwrap().is_empty());
        assert!(db.provider_secret_keys("linode").unwrap().is_empty(), "its secrets are gone");
        assert_eq!(db.provider_secret_keys("linode-2").unwrap(), ["api_token"], "another account's aren't");
    }

    #[test]
    fn test_vault_init_and_unlock_lifecycle() {
        let mut db = VaultDb::open_in_memory().unwrap();
        assert!(!db.is_initialized().unwrap());

        let password = "CorrectHorseBatteryStaple!";
        let key = db.init_vault(password, None, None).unwrap();
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

        let _ = db.init_vault(password, Some(&totp_secret), None).unwrap();
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

    #[test]
    fn test_default_servers_seeding() {
        let db = VaultDb::open_in_memory().unwrap();
        let servers = db.list_servers().unwrap();
        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_servers_crud_lifecycle() {
        let db = VaultDb::open_in_memory().unwrap();

        let srv = ServerRecord {
            id: "srv-custom-01".into(),
            name: "custom-01".into(),
            host: "10.0.9.99".into(),
            port: 22,
            login_user: "deploy".into(),
            auth_method: "publickey".into(),
            key_id: Some("key-1".into()),
            jump_host_id: Some("bastion".into()),
            env: "PROD".into(),
            role: "worker".into(),
            group_name: "workers".into(),
            tags: vec!["custom".into(), "queue".into()],
            host_key_fingerprint: Some("SHA256:abcd1234...".into()),
            os_distro: "Ubuntu 24.04 LTS".into(),
            os_kernel: "6.8.0".into(),
            arch: "x86_64".into(),
            memory_total: "16 GB".into(),
            disk_total: "500 GB".into(),
            agent_installed: true,
            agent_version: Some("0.9.4".into()),
            status: "online".into(),
            created_at: String::new(),
            last_seen_at: None,
            archived_at: None,
            purged_at: None,
            region_country: "DE".into(),
            region_city: "Frankfurt".into(),
            region_provider: "Linode".into(),
            region_code: "de-fra-2".into(),
            region_source: "metadata".into(),
            provider_account: "linode".into(),
            provider_instance: "123".into(),
        };

        db.upsert_server(&srv).unwrap();

        let fetched = db.get_server("srv-custom-01").unwrap().unwrap();
        assert_eq!(fetched.name, "custom-01");
        assert_eq!((fetched.region_country.as_str(), fetched.region_code.as_str(), fetched.region_source.as_str()), ("DE", "de-fra-2", "metadata"), "region round-trips");
        assert_eq!((fetched.provider_account.as_str(), fetched.provider_instance.as_str()), ("linode", "123"), "provider link round-trips");
        assert_eq!(fetched.host, "10.0.9.99");
        assert_eq!(fetched.port, 22);
        assert_eq!(fetched.tags.len(), 2);
        assert!(fetched.agent_installed);

        // Update status
        db.update_server_status("srv-custom-01", "degraded").unwrap();
        let updated = db.get_server("srv-custom-01").unwrap().unwrap();
        assert_eq!(updated.status, "degraded");
        assert!(updated.last_seen_at.is_some());

        // Delete is test-only now (ERR-32): servers are archived, not deleted.
        db.delete_server("srv-custom-01").unwrap();
        assert!(db.get_server("srv-custom-01").unwrap().is_none());
    }

    /// A server leaves the fleet without being deleted, comes back with its
    /// history, and its data is purged once the window closes (ERR-32).
    #[test]
    fn archiving_hides_a_server_and_purging_keeps_the_record() {
        let db = VaultDb::open_in_memory().unwrap();
        let srv = ServerRecord { id: "srv-arch".into(), name: "arch-01".into(), host: "10.0.0.9".into(), os_distro: "Ubuntu 24.04 LTS".into(), created_at: String::new(), ..ServerRecord::default() };
        db.upsert_server(&srv).unwrap();
        db.insert_change_record(&ChangeRecord {
            id: "chg-1".into(),
            server_id: "srv-arch".into(),
            server_name: "arch-01".into(),
            action_kind: "firewall".into(),
            target: "allow 22/tcp".into(),
            before_state: "none".into(),
            after_state: None,
            blast_radius: None,
            outcome: "success".into(),
            started_at: "2026-09-24T00:00:00Z".into(),
            completed_at: None,
        }).unwrap();
        // Attachments are stored by id by the Keys screen and by name by
        // onboarding, so a key carrying both must lose both.
        db.conn.execute(
            "INSERT INTO ssh_keys (id, name, group_id, public_key, fingerprint, algorithm, attached_servers, created_at, updated_at)
             VALUES ('key-1', 'k', 'default', 'ssh-ed25519 AAAA', 'SHA256:x', 'ed25519', '[\"srv-arch\",\"arch-01\",\"other\"]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        assert_eq!(db.list_servers().unwrap().len(), 1);
        db.archive_server("srv-arch").unwrap();

        // Gone from the active fleet, still in the vault.
        assert!(db.list_servers().unwrap().is_empty());
        let archived = db.list_archived_servers().unwrap();
        assert_eq!(archived.len(), 1);
        assert!(archived[0].archived_at.is_some());
        assert_eq!(archived[0].status, "archived");

        // Restore brings it back with its history intact.
        db.restore_server("srv-arch").unwrap();
        assert_eq!(db.list_servers().unwrap().len(), 1);
        assert_eq!(db.list_change_records("srv-arch", 10).unwrap().len(), 1);

        // Archive again and purge: history and attachments go, the record stays.
        db.archive_server("srv-arch").unwrap();
        let outcome = db.purge_server_data("srv-arch", "arch-01").unwrap();
        assert_eq!(outcome.change_records, 1);
        assert_eq!(outcome.key_attachments, 2);
        let after = db.get_server("srv-arch").unwrap().unwrap();
        assert!(after.purged_at.is_some(), "the purge is recorded");
        assert_eq!(after.os_distro, srv.os_distro, "the record keeps its facts");
        assert!(db.list_change_records("srv-arch", 10).unwrap().is_empty());
        let key = db.get_ssh_key("key-1").unwrap().unwrap();
        assert_eq!(key.attached_servers, ["other"], "only the archived server was detached");
        assert_eq!(db.list_archived_servers().unwrap().len(), 1, "the record itself is never deleted");
    }

    /// The daily sweep purges each server once, not every day forever.
    #[test]
    fn a_purged_server_is_not_due_for_purge_again() {
        let db = VaultDb::open_in_memory().unwrap();
        db.upsert_server(&ServerRecord { id: "s1".into(), name: "s1".into(), created_at: String::new(), ..ServerRecord::default() }).unwrap();
        db.archive_server("s1").unwrap();
        let far_future = "2999-01-01T00:00:00Z";
        assert_eq!(db.servers_due_for_purge(far_future).unwrap().len(), 1);
        db.purge_server_data("s1", "s1").unwrap();
        assert!(db.servers_due_for_purge(far_future).unwrap().is_empty(), "purging is once, not daily");

        // A server still inside its window is not due at all.
        db.upsert_server(&ServerRecord { id: "s2".into(), name: "s2".into(), created_at: String::new(), ..ServerRecord::default() }).unwrap();
        db.archive_server("s2").unwrap();
        assert!(db.servers_due_for_purge("1999-01-01T00:00:00Z").unwrap().is_empty());
        // Restoring clears the archive, so it stops being a purge candidate.
        db.restore_server("s2").unwrap();
        assert!(db.servers_due_for_purge(far_future).unwrap().is_empty());
    }

    /// A vault created before ERR-32 has no archived_at/purged_at columns.
    /// Opening it must add them and leave the servers already in it alone.
    #[test]
    fn opening_an_old_vault_adds_the_archive_columns() {
        let path = std::env::temp_dir().join(format!("crow-old-vault-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
            }
        };
        cleanup();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE servers (
                    id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, host TEXT NOT NULL,
                    port INTEGER NOT NULL DEFAULT 22, login_user TEXT NOT NULL,
                    auth_method TEXT NOT NULL DEFAULT 'publickey', key_id TEXT, jump_host_id TEXT,
                    env TEXT NOT NULL DEFAULT 'PROD', role TEXT NOT NULL DEFAULT 'generic',
                    group_name TEXT NOT NULL DEFAULT 'default', tags TEXT NOT NULL DEFAULT '[]',
                    host_key_fingerprint TEXT, os_distro TEXT NOT NULL DEFAULT 'Ubuntu 24.04.1 LTS',
                    os_kernel TEXT NOT NULL DEFAULT '6.8.0-45-generic',
                    arch TEXT NOT NULL DEFAULT 'x86_64 · 4 vCPU',
                    memory_total TEXT NOT NULL DEFAULT '8.0 GB',
                    disk_total TEXT NOT NULL DEFAULT '160 GB nvme',
                    agent_installed INTEGER NOT NULL DEFAULT 0, agent_version TEXT,
                    status TEXT NOT NULL DEFAULT 'online', created_at TEXT NOT NULL, last_seen_at TEXT
                );
                INSERT INTO servers (id, name, host, login_user, created_at)
                VALUES ('old-1', 'legacy-01', '10.0.0.5', 'root', '2026-01-01T00:00:00Z');",
            ).unwrap();
        }

        let db = VaultDb::open_or_create(Some(&path)).unwrap();
        let servers = db.list_servers().unwrap();
        assert_eq!(servers.len(), 1, "the server already in the vault survives");
        assert_eq!(servers[0].name, "legacy-01");
        assert_eq!(servers[0].archived_at, None, "and starts out active");

        // The added columns work straight away.
        db.archive_server("old-1").unwrap();
        assert!(db.list_servers().unwrap().is_empty());
        assert_eq!(db.list_archived_servers().unwrap().len(), 1);

        // Re-opening runs the ALTERs again; that must stay harmless.
        drop(db);
        let db = VaultDb::open_or_create(Some(&path)).unwrap();
        assert_eq!(db.list_archived_servers().unwrap().len(), 1);
        assert!(db.list_archived_servers().unwrap()[0].archived_at.is_some());
        drop(db);
        cleanup();
    }

    #[test]
    fn test_clanker_providers_seeding_and_crud() {
        let db = VaultDb::open_in_memory().unwrap();
        let providers = db.list_clanker_providers().unwrap();

        // 5 real providers seeded, with no invented usage
        assert_eq!(providers.len(), 5);
        for id in ["openai", "anthropic", "mistral", "deepseek", "qwen"] {
            assert!(providers.iter().any(|p| p.id == id), "{id}");
        }
        assert!(providers.iter().all(|p| p.total_calls == 0 && p.last_used_at.is_none()), "no usage until something is sent");
        db.record_clanker_call("anthropic").unwrap();
        let a = db.list_clanker_providers().unwrap().into_iter().find(|p| p.id == "anthropic").unwrap();
        assert_eq!((a.total_calls, a.calls_30d), (1, 1));
        assert!(a.last_used_at.is_some());

        let openai = db.get_clanker_provider("openai").unwrap().unwrap();
        assert!(openai.is_default);
        assert_eq!(openai.model, "gpt-4o-mini");

        // Set default to deepseek
        db.set_default_clanker_provider("deepseek").unwrap();
        let deepseek = db.get_clanker_provider("deepseek").unwrap().unwrap();
        assert!(deepseek.is_default);
        let openai_after = db.get_clanker_provider("openai").unwrap().unwrap();
        assert!(!openai_after.is_default);

        // Record usage
        let before_calls = deepseek.calls_30d;
        db.record_clanker_usage("deepseek").unwrap();
        let deepseek_after = db.get_clanker_provider("deepseek").unwrap().unwrap();
        assert_eq!(deepseek_after.calls_30d, before_calls + 1);
        assert!(deepseek_after.last_used_at.is_some());

        // API keys go to the vault, encrypted; the row never holds one.
        let key = crate::vault::generate_data_key();
        let mut custom = deepseek_after.clone();
        custom.api_key = "sk-deepseek-test-key-12345".into();
        db.upsert_clanker_provider(&custom).unwrap();
        db.set_clanker_api_key(&key, "deepseek", Some("sk-deepseek-test-key-12345")).unwrap();
        let mut listed = db.list_clanker_providers().unwrap();
        assert_eq!(listed.iter().find(|p| p.id == "deepseek").unwrap().api_key, "", "not in the row");
        db.load_clanker_api_keys(&key, &mut listed);
        assert_eq!(listed.iter().find(|p| p.id == "deepseek").unwrap().api_key, "sk-deepseek-test-key-12345");

        // Reset usage
        db.reset_clanker_usage("deepseek").unwrap();
        let deepseek_reset = db.get_clanker_provider("deepseek").unwrap().unwrap();
        assert_eq!(deepseek_reset.total_calls, 0);
        assert_eq!(deepseek_reset.calls_30d, 0);
        assert!(deepseek_reset.last_used_at.is_none());
    }
}

