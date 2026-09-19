pub mod crypto;
pub mod db;

use std::sync::{Arc, Mutex};
use std::time::Instant;

pub use crypto::{
    generate_salt, generate_totp_secret, totp_auth_url, verify_totp_code, MasterKey,
};
pub use db::{
    ClankerProviderConfig, ServerRecord, SshKeyGroup, SshKeyRecord, SshScanPath, VaultDb,
    VaultEntryMeta, VaultError, VaultMeta,
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

pub struct Vault {
    db: Arc<Mutex<VaultDb>>,
    session: Option<VaultSession>,
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

    pub fn key(&self) -> Option<&MasterKey> {
        self.session.as_ref().map(|s| &s.key)
    }

    /// Initializes password and mandatory 2FA TOTP protection.
    pub fn initialize(&mut self, password: &str, totp_secret: &str) -> Result<(), VaultError> {
        let mut db = self.db.lock().map_err(|_| VaultError::Crypto("DB lock poisoned".into()))?;
        let key = db.init_vault(password, Some(totp_secret))?;
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
