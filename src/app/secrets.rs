//! The vault's data key while the vault password is off (ERR-56).
//!
//! Every secret (provider tokens, AI keys) is encrypted with one random data
//! key. With a vault password, the key is stored wrapped by the password.
//! Without one, the OS credential store holds it (Keychain, Secret Service,
//! Credential Manager, through gpui). If neither is available, secrets are
//! refused, never stored in plain text.

use gpui_kit::*;
use zeroize::Zeroize;

use super::CrowApp;
use crate::vault::{generate_data_key, key_from_bytes, KeyringState, CLANKER_SECRET_CATEGORY, PROVIDER_SECRET_CATEGORY};

/// Where the data key lives in the OS credential store.
pub const KEYRING_URL: &str = "crow.rs/vault-data-key";
const KEYRING_USER: &str = "crow";

/// Vault entry categories that hold secrets encrypted with the data key.
const SECRET_CATEGORIES: [&str; 2] = [PROVIDER_SECRET_CATEGORY, CLANKER_SECRET_CATEGORY];

impl CrowApp {
    /// Reads the data key from the OS keyring, in the background, when the
    /// vault password is off. Then moves any plain-text AI keys into the vault.
    pub fn load_keyring_key(&mut self, cx: &mut Context<Self>) {
        if self.vault.is_password_auth_enabled() {
            self.on_data_key_ready(cx);
            return;
        }
        let read = cx.read_credentials(KEYRING_URL);
        cx.spawn(async move |entity, cx| {
            let result = read.await;
            let _ = entity.update(cx, |this, cx| {
                match result {
                    Ok(Some((_, mut bytes))) => {
                        let key = key_from_bytes(&bytes);
                        bytes.zeroize();
                        match key {
                            Some(k) => this.vault.set_keyring_key(Some(k)),
                            None => this.vault.keyring_state = KeyringState::Unavailable("the key stored there is malformed".into()),
                        }
                    }
                    Ok(None) => this.vault.set_keyring_key(None),
                    Err(e) => this.vault.keyring_state = KeyringState::Unavailable(root_cause(&e)),
                }
                this.on_data_key_ready(cx);
            });
        })
        .detach();
    }

    /// Runs `then` once a data key exists, creating one in the OS keyring
    /// first if this is the first secret. `then` gets the reason when no
    /// key can be had.
    pub fn with_data_key(&mut self, cx: &mut Context<Self>, then: impl FnOnce(&mut Self, Result<(), String>, &mut Context<Self>) + 'static) {
        if self.vault.key().is_some() {
            return then(self, Ok(()), cx);
        }
        if let Some(why) = self.vault.secrets_blocker() {
            return then(self, Err(why), cx);
        }
        // Password off and the keyring is empty: make the key, keep it there.
        let key = generate_data_key();
        let write = cx.write_credentials(KEYRING_URL, KEYRING_USER, key.as_bytes());
        cx.spawn(async move |entity, cx| {
            let result = write.await;
            let _ = entity.update(cx, |this, cx| match result {
                Ok(()) => {
                    // Anything already in the vault was encrypted with a key
                    // the keyring no longer has, so it can't be read again.
                    let lost = this.purge_unreadable_secrets();
                    if lost > 0 {
                        this.secrets_notice = Some(format!("The OS keyring no longer had Crow's encryption key, so {lost} saved secret(s) couldn't be read and were cleared. Enter them again."));
                    }
                    this.vault.set_keyring_key(Some(key));
                    this.on_data_key_ready(cx);
                    then(this, Ok(()), cx);
                }
                Err(e) => {
                    let why = root_cause(&e);
                    this.vault.keyring_state = KeyringState::Unavailable(why.clone());
                    then(this, Err(this.vault.secrets_blocker().unwrap_or(why)), cx);
                }
            });
        })
        .detach();
    }

    /// Called whenever a data key becomes available (keyring read, vault
    /// unlocked or set up): moves plain-text AI keys into the vault and
    /// reloads what shows secrets.
    pub fn on_data_key_ready(&mut self, cx: &mut Context<Self>) {
        if let (Some(key), Ok(db)) = (self.vault.key(), self.vault.db().lock()) {
            match db.migrate_plaintext_clanker_keys(key) {
                Ok(0) => {}
                Ok(n) => self.secrets_notice = Some(format!("Moved {n} AI API key(s) out of plain text into the encrypted vault.")),
                Err(e) => self.secrets_notice = Some(format!("Couldn't move AI API keys into the vault: {e}")),
            }
        } else if self.vault.keyring_state == KeyringState::Empty && self.count_secret_entries() > 0 {
            self.secrets_notice = Some("The OS keyring no longer has Crow's encryption key, so saved provider tokens and AI keys can't be read. Enter them again.".into());
        }
        self.refresh_clankers(cx);
        self.refresh_providers();
        cx.notify();
    }

    /// After the vault password is turned on, the data key lives in the
    /// vault (wrapped by the password); remove the keyring copy.
    pub fn forget_keyring_key(&mut self, cx: &mut Context<Self>) {
        cx.delete_credentials(KEYRING_URL).detach();
    }

    fn count_secret_entries(&self) -> usize {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return 0 };
        SECRET_CATEGORIES.iter().map(|c| db.list_entries(Some(c)).map(|e| e.len()).unwrap_or(0)).sum()
    }

    fn purge_unreadable_secrets(&mut self) -> usize {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return 0 };
        let mut n = 0;
        for c in SECRET_CATEGORIES {
            for e in db.list_entries(Some(c)).unwrap_or_default() {
                if db.delete_entry(&e.id).unwrap_or(false) {
                    n += 1;
                }
            }
        }
        n
    }

    pub fn dismiss_secrets_notice(&mut self, cx: &mut Context<Self>) {
        self.secrets_notice = None;
        cx.notify();
    }
}

/// The error with its causes, e.g. "failed to connect: org.freedesktop.secrets was not provided".
fn root_cause(e: &impl std::fmt::Display) -> String {
    format!("{e:#}")
}
