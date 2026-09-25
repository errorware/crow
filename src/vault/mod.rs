pub mod crypto;
pub mod db;

use std::sync::{Arc, Mutex};
use std::time::Instant;

pub use crypto::{
    generate_data_key, generate_salt, generate_totp_secret, key_from_bytes, totp_auth_url, verify_totp_code, MasterKey,
};
pub use db::{
    clanker_secret_id, provider_secret_id, ChangeRecord, CLANKER_SECRET_CATEGORY, ClankerProviderConfig, ProviderAccount, PurgeOutcome, PROVIDER_SECRET_CATEGORY, ServerRecord, SshKeyGroup, SshKeyRecord,
    SshScanPath, VaultDb, VaultEntryMeta, VaultError, VaultMeta, AUDIT_SERVER_ID,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VaultStatus {
    /// Password logon is not enabled (default on fresh install).
    Disabled,
    /// Password and mandatory 2FA are enabled and the vault is locked.
    Locked,
    /// Password and mandatory 2FA are enabled and authenticated.
    Unlocked,
}

pub struct VaultSession {
    pub key: MasterKey,
    pub unlocked_at: Instant,
    pub auto_lock_minutes: i64,
}

/// Where the data key comes from while the vault password is off (ERR-56).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyringState {
    /// Still asking the OS keyring.
    Loading,
    /// The keyring holds Crow's data key.
    Ready,
    /// The keyring works but holds no key yet (created on first secret).
    Empty,
    /// No usable keyring (no Secret Service, access denied, ...).
    Unavailable(String),
}

pub struct Vault {
    db: Arc<Mutex<VaultDb>>,
    session: Option<VaultSession>,
    /// The data key from the OS keyring, used while the password is off.
    keyring_key: Option<MasterKey>,
    pub keyring_state: KeyringState,
    totp_enabled: bool,
    initialized: bool,
}

impl Vault {
    pub fn open_default() -> Result<Self, VaultError> {
        let db = VaultDb::open_or_create(None)?;
        let initialized = db.is_initialized()?;
        let totp_enabled = if initialized {
            db.is_totp_enabled()?
        } else {
            false
        };

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            session: None,
            keyring_key: None,
            keyring_state: KeyringState::Loading,
            totp_enabled,
            initialized,
        })
    }

    pub fn status(&self) -> VaultStatus {
        if !self.initialized {
            VaultStatus::Disabled
        } else if self.session.is_some() {
            VaultStatus::Unlocked
        } else {
            VaultStatus::Locked
        }
    }

    pub fn is_password_auth_enabled(&self) -> bool {
        self.initialized
    }

    pub fn is_unlocked(&self) -> bool {
        !self.initialized || self.session.is_some()
    }

    /// The data key every secret is encrypted with: from the unlocked
    /// session when the vault password is on, else from the OS keyring.
    /// `None` while locked, while the keyring is still being read, or when
    /// no keyring is available.
    pub fn key(&self) -> Option<&MasterKey> {
        if self.initialized {
            self.session.as_ref().map(|s| &s.key)
        } else {
            self.keyring_key.as_ref()
        }
    }

    /// Records the data key read from (or just written to) the OS keyring.
    pub fn set_keyring_key(&mut self, key: Option<MasterKey>) {
        self.keyring_state = if key.is_some() { KeyringState::Ready } else { KeyringState::Empty };
        self.keyring_key = key;
    }

    /// Whether a secret can be stored right now, and if not, why.
    pub fn secrets_blocker(&self) -> Option<String> {
        if self.initialized {
            return self.session.is_none().then(|| "the vault is locked".to_string());
        }
        match &self.keyring_state {
            KeyringState::Ready | KeyringState::Empty => None,
            KeyringState::Loading => Some("still reading the OS keyring; try again in a moment".into()),
            KeyringState::Unavailable(why) => Some(format!("no OS keyring is available to hold the encryption key ({why}); turn on the vault password in Vault & Security instead")),
        }
    }

    /// Initializes password and mandatory 2FA TOTP protection.
    pub fn initialize(&mut self, password: &str, totp_secret: &str) -> Result<(), VaultError> {
        let mut db = self.db.lock().map_err(|_| VaultError::Crypto("DB lock poisoned".into()))?;
        // Keep the data key secrets already use; it moves from the keyring
        // into the vault, wrapped by the password.
        let key = db.init_vault(password, Some(totp_secret), self.keyring_key.as_ref())?;
        self.keyring_key = None;
        self.initialized = true;
        self.totp_enabled = true;
        self.session = Some(VaultSession {
            key,
            unlocked_at: Instant::now(),
            auto_lock_minutes: 15,
        });
        Ok(())
    }

    /// Authenticates with master password and mandatory 2FA code.
    pub fn unlock(&mut self, password: &str, totp_code: &str) -> Result<(), VaultError> {
        let db = self.db.lock().map_err(|_| VaultError::Crypto("DB lock poisoned".into()))?;
        let key = db.unlock(password, Some(totp_code))?;
        let meta = db.get_meta().ok();
        let auto_lock_minutes = meta.map(|m| m.auto_lock_minutes).unwrap_or(15);

        self.session = Some(VaultSession {
            key,
            unlocked_at: Instant::now(),
            auto_lock_minutes,
        });
        Ok(())
    }

    pub fn lock(&mut self) {
        if self.initialized {
            // MasterKey inside session will be dropped and automatically zeroized
            self.session = None;
        }
    }

    pub fn db(&self) -> Arc<Mutex<VaultDb>> {
        Arc::clone(&self.db)
    }
}
