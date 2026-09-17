pub mod crypto;
pub mod db;

use std::sync::{Arc, Mutex};
use std::time::Instant;

pub use crypto::{
    generate_salt, generate_totp_secret, totp_auth_url, verify_totp_code, MasterKey,
};
pub use db::{VaultDb, VaultEntryMeta, VaultError, VaultMeta};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VaultStatus {
    Uninitialized,
    Locked { totp_enabled: bool },
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
            VaultStatus::Uninitialized
        } else if self.session.is_some() {
            VaultStatus::Unlocked
        } else {
            VaultStatus::Locked {
                totp_enabled: self.totp_enabled,
            }
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.is_some()
    }

    pub fn key(&self) -> Option<&MasterKey> {
        self.session.as_ref().map(|s| &s.key)
    }

    pub fn initialize(&mut self, password: &str, totp_secret: Option<&str>) -> Result<(), VaultError> {
        let mut db = self.db.lock().map_err(|_| VaultError::Crypto("DB lock poisoned".into()))?;
        let key = db.init_vault(password, totp_secret)?;
        self.initialized = true;
        self.totp_enabled = totp_secret.is_some();
        self.session = Some(VaultSession {
            key,
            unlocked_at: Instant::now(),
            auto_lock_minutes: 15,
        });
        Ok(())
    }

    pub fn unlock(&mut self, password: &str, totp_code: Option<&str>) -> Result<(), VaultError> {
        let db = self.db.lock().map_err(|_| VaultError::Crypto("DB lock poisoned".into()))?;
        let key = db.unlock(password, totp_code)?;
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
        // MasterKey inside session will be dropped and automatically zeroized
        self.session = None;
    }

    pub fn db(&self) -> Arc<Mutex<VaultDb>> {
        Arc::clone(&self.db)
    }
}
