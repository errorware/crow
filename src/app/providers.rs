//! Settings → Providers: provider accounts, their settings form and TEST
//! CONNECTION (ERR-45). The form is built from each provider's manifest;
//! crow-config validates it and secrets go to the encrypted vault only.

use std::collections::{HashMap, HashSet};

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use crow_config_core::{FieldType, SecretValue, SettingsEdit};

use super::CrowApp;
use crate::providers;
use crate::vault::ProviderAccount;

#[derive(Default)]
pub struct ProvidersState {
    pub accounts: Vec<ProviderAccount>,
    /// Account id → keys of the secrets it has in the vault.
    pub secrets_set: HashMap<String, Vec<String>>,
    /// The provider (plugin name) whose form is open.
    pub editing: Option<String>,
    /// Account ids with a TEST CONNECTION running.
    pub checking: HashSet<String>,
    /// Plugin name → the last save error shown on its card.
    pub errors: HashMap<String, String>,
    /// Plugin name whose REMOVE is waiting for a second click.
    pub confirm_remove: Option<String>,
}

impl ProvidersState {
    /// The account for `plugin`. One per provider for now; the data model
    /// allows several.
    pub fn account(&self, plugin: &str) -> Option<&ProviderAccount> {
        self.accounts.iter().find(|a| a.plugin == plugin)
    }

    pub fn is_secret_set(&self, account: &str, key: &str) -> bool {
        self.secrets_set.get(account).is_some_and(|keys| keys.iter().any(|k| k == key))
    }
}

/// One text input per manifest field of the provider being edited.
pub struct ProviderFormInputs {
    pub plugin: String,
    /// (field key, is secret, input)
    pub fields: Vec<(String, bool, Entity<InputState>)>,
    pub _events: Vec<Subscription>,
}

impl CrowApp {
    pub fn refresh_providers(&mut self) {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        self.providers.accounts = db.list_provider_accounts().unwrap_or_default();
        self.providers.secrets_set = self.providers.accounts.iter().map(|a| (a.id.clone(), db.provider_secret_keys(&a.id).unwrap_or_default())).collect();
    }

    pub fn open_provider_form(&mut self, plugin: &str, cx: &mut Context<Self>) {
        self.providers.editing = Some(plugin.to_string());
        self.providers.errors.remove(plugin);
        self.providers.confirm_remove = None;
        self.provider_inputs = None;
        cx.notify();
    }

    pub fn close_provider_form(&mut self, cx: &mut Context<Self>) {
        self.providers.editing = None;
        self.provider_inputs = None;
        cx.notify();
    }

    /// Creates the open form's inputs (they need the window): plain fields
    /// start with their stored value, secrets start empty and say whether
    /// one is set. Enter saves.
    pub fn ensure_provider_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(plugin) = self.providers.editing.clone() else { return };
        if self.provider_inputs.as_ref().is_some_and(|i| i.plugin == plugin) {
            return;
        }
        let Some(factory) = providers::factory(&plugin) else { return };
        let account = self.providers.account(&plugin).cloned();
        let mut fields = Vec::new();
        for def in &(factory.manifest)().fields {
            let secret = def.field_type == FieldType::Secret;
            let current = account.as_ref().and_then(|a| a.settings.get(&def.name)).map(|v| v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string()));
            let placeholder = if secret {
                if account.as_ref().is_some_and(|a| self.providers.is_secret_set(&a.id, &def.name)) { "set · type to replace".to_string() } else { "not set".to_string() }
            } else {
                def.default.clone().map(|d| format!("default: {d}")).unwrap_or_default()
            };
            let input = cx.new(|cx| {
                let state = InputState::new(window, cx).placeholder(placeholder).masked(secret);
                match current {
                    Some(v) => state.default_value(v),
                    None => state,
                }
            });
            fields.push((def.name.clone(), secret, input));
        }
        let events = fields
            .iter()
            .map(|(_, _, input)| {
                cx.subscribe(input, |this, _input, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.save_provider_form(cx);
                    }
                })
            })
            .collect();
        if let Some((_, _, first)) = fields.first() {
            first.update(cx, |i, cx| i.focus(window, cx));
        }
        self.provider_inputs = Some(ProviderFormInputs { plugin, fields, _events: events });
    }

    /// Validates and saves the open form, then tests the connection.
    pub fn save_provider_form(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.provider_inputs.as_ref() else { return };
        let plugin = inputs.plugin.clone();
        let mut account = self.providers.account(&plugin).cloned().unwrap_or_else(|| ProviderAccount {
            id: plugin.clone(),
            plugin: plugin.clone(),
            label: display_name(&plugin),
            settings: Default::default(),
            last_check: None,
            last_check_ok: false,
            last_check_at: None,
        });
        let mut edits = Vec::new();
        for (key, secret, input) in &inputs.fields {
            let value = input.read(cx).value().trim().to_string();
            match (secret, value.is_empty()) {
                (true, false) => edits.push(SettingsEdit::SetSecret { key: key.clone(), value: SecretValue::new(value) }),
                (true, true) => {} // blank keeps the stored secret
                (false, false) => edits.push(SettingsEdit::Set { key: key.clone(), value: serde_json::Value::String(value) }),
                (false, true) if account.settings.contains_key(key) => edits.push(SettingsEdit::Unset { key: key.clone() }),
                (false, true) => {}
            }
        }
        // A new secret needs the data key; the first one creates it in the
        // OS keyring (when there's no vault password), then saving resumes.
        if self.vault.key().is_none() && edits.iter().any(|e| matches!(e, SettingsEdit::SetSecret { .. })) {
            drop(edits);
            self.with_data_key(cx, move |this, ready, cx| match ready {
                Ok(()) => this.save_provider_form(cx),
                Err(e) => {
                    this.providers.errors.insert(plugin, e);
                    cx.notify();
                }
            });
            return;
        }
        let result = match self.vault.db().lock() {
            Ok(db) => providers::save_form(&db, self.vault.key(), &mut account, edits),
            Err(_) => Err("the vault is busy; try again".into()),
        };
        match result {
            Ok(()) => {
                self.providers.editing = None;
                self.provider_inputs = None;
                self.providers.errors.remove(&plugin);
                self.refresh_providers();
                self.test_provider(&plugin, cx);
            }
            Err(e) => {
                self.providers.errors.insert(plugin, e);
            }
        }
        cx.notify();
    }

    /// Removes a stored secret that isn't required (e.g. switching UpCloud
    /// from a token to username and password).
    pub fn clear_provider_secret(&mut self, plugin: &str, key: &str, cx: &mut Context<Self>) {
        let Some(mut account) = self.providers.account(plugin).cloned() else { return };
        let result = match self.vault.db().lock() {
            Ok(db) => providers::save_form(&db, self.vault.key(), &mut account, vec![SettingsEdit::ClearSecret { key: key.to_string() }]),
            Err(_) => Err("the vault is busy; try again".into()),
        };
        if let Err(e) = result {
            self.providers.errors.insert(plugin.to_string(), e);
        }
        self.provider_inputs = None;
        self.refresh_providers();
        cx.notify();
    }

    /// First click arms, second removes the account and its vault secrets.
    pub fn remove_provider_clicked(&mut self, plugin: &str, cx: &mut Context<Self>) {
        if self.providers.confirm_remove.as_deref() != Some(plugin) {
            self.providers.confirm_remove = Some(plugin.to_string());
            cx.notify();
            return;
        }
        self.providers.confirm_remove = None;
        if let Some(account) = self.providers.account(plugin).cloned() {
            if let Ok(db) = self.vault.db().lock() {
                let _ = db.delete_provider_account(&account.id);
            }
        }
        if self.providers.editing.as_deref() == Some(plugin) {
            self.providers.editing = None;
            self.provider_inputs = None;
        }
        self.refresh_providers();
        cx.notify();
    }

    /// Makes a cheap authenticated call in the background and records the
    /// result on the account.
    pub fn test_provider(&mut self, plugin: &str, cx: &mut Context<Self>) {
        let Some(account) = self.providers.account(plugin).cloned() else { return };
        if self.providers.checking.contains(&account.id) {
            return;
        }
        let db = self.vault.db();
        let settings = match (self.vault.key(), db.lock()) {
            (Some(key), Ok(guard)) => providers::load_settings(&guard, key, &account),
            (None, _) => Err("the vault is locked or has no password, so the provider's secrets can't be read".into()),
            (_, Err(_)) => Err("the vault is busy; try again".into()),
        };
        let settings = match settings {
            Ok(s) => s,
            Err(e) => {
                if let Ok(guard) = db.lock() {
                    let _ = guard.record_provider_check(&account.id, false, &e);
                }
                self.refresh_providers();
                cx.notify();
                return;
            }
        };
        self.providers.checking.insert(account.id.clone());
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let acct = account.clone();
            let (ok, message) = cx
                .background_executor()
                .spawn(async move {
                    match providers::connect(&acct, settings).and_then(|p| p.check()) {
                        Ok(summary) => (true, summary),
                        Err(e) => (false, e.to_string()),
                    }
                })
                .await;
            if let Ok(guard) = db.lock() {
                let _ = guard.record_provider_check(&account.id, ok, &message);
            }
            let _ = entity.update(cx, |this, cx| {
                this.providers.checking.remove(&account.id);
                this.refresh_providers();
                cx.notify();
            });
        })
        .detach();
    }
}

/// "linode" → "Linode", "upcloud" → "UpCloud".
pub fn display_name(plugin: &str) -> String {
    match plugin {
        "upcloud" => "UpCloud".into(),
        other => {
            let mut c = other.chars();
            c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
        }
    }
}
