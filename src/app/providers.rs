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
                if account.as_ref().is_some_and(|a| self.providers.is_secret_set(&a.id, &def.name)) { t!("app-providers-set-type-to-replace").to_string() } else { t!("app-providers-not-set").to_string() }
            } else {
                def.default.clone().map(|d| t!("app-providers-default", d = d)).unwrap_or_default()
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
            label: providers::display_name(&plugin),
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
            Err(_) => Err(t!("app-providers-the-vault-is-busy-try").into()),
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
            Err(_) => Err(t!("app-providers-the-vault-is-busy-try").into()),
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
            (None, _) => Err(t!("app-providers-the-vault-is-locked-or").into()),
            (_, Err(_)) => Err(t!("app-providers-the-vault-is-busy-try").into()),
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

/// Settings → Providers → IMPORT INSTANCES (ERR-46).
#[derive(Default)]
pub struct ImportState {
    pub open: bool,
    /// Accounts still being asked.
    pub loading: usize,
    pub rows: Vec<providers::InstanceRow>,
    /// Per-account failures, in words.
    pub errors: Vec<String>,
    /// Linked servers their provider no longer lists.
    pub missing: Vec<String>,
    /// Servers linked to their instance (or given a region) this time.
    pub linked: usize,
}

impl CrowApp {
    pub fn close_import(&mut self, cx: &mut Context<Self>) {
        self.import.open = false;
        cx.notify();
    }

    /// Asks every provider account that can list instances for them, in the
    /// background, then links enrolled servers to their instances.
    pub fn open_import(&mut self, cx: &mut Context<Self>) {
        use crow_provider_core::capabilities::INSTANCES_LIST;
        self.import = ImportState { open: true, ..Default::default() };
        let accounts: Vec<ProviderAccount> = self
            .providers
            .accounts
            .iter()
            .filter(|a| providers::factory(&a.plugin).is_some_and(|f| (f.manifest)().has_capability(INSTANCES_LIST)))
            .cloned()
            .collect();
        if accounts.is_empty() {
            self.import.errors.push(t!("app-providers-no-provider-accounts-yet-set").into());
            cx.notify();
            return;
        }
        let Some(key) = self.vault.key().cloned() else {
            self.import.errors.push(self.vault.secrets_blocker().unwrap_or_else(|| t!("app-providers-no-encryption-key").into()));
            cx.notify();
            return;
        };
        let mut jobs = Vec::new();
        if let Ok(db) = self.vault.db().lock() {
            for a in accounts {
                match providers::load_settings(&db, &key, &a) {
                    Ok(settings) => jobs.push((a, settings)),
                    Err(e) => self.import.errors.push(format!("{}: {e}", a.label)),
                }
            }
        }
        self.import.loading = jobs.len();
        cx.notify();
        for (account, settings) in jobs {
            cx.spawn(async move |entity, cx| {
                let acct = account.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        let provider = providers::connect(&acct, settings).map_err(|e| e.to_string())?;
                        let list = provider.as_list_instances().ok_or(t!("app-providers-this-provider-cant-list-instances"))?;
                        list.instances().map_err(|e| e.to_string())
                    })
                    .await;
                let _ = entity.update(cx, |this, cx| {
                    this.import.loading = this.import.loading.saturating_sub(1);
                    match result {
                        Ok(instances) => this.apply_instances(&account, &instances),
                        Err(e) => this.import.errors.push(format!("{}: {e}", account.label)),
                    }
                    cx.notify();
                });
            })
            .detach();
        }
    }

    fn apply_instances(&mut self, account: &ProviderAccount, instances: &[crow_provider_core::hosts::Instance]) {
        let name = providers::display_name(&account.plugin);
        let r = providers::reconcile(&self.fleet.servers, &account.id, &name, instances);
        if let Ok(db) = self.vault.db().lock() {
            for s in &r.updates {
                let _ = db.upsert_server(s);
            }
        }
        self.import.linked += r.updates.len();
        self.import.missing.extend(r.missing.into_iter().map(|m| t!("app-providers-no-longer-listed-by", m = m, name = name)));
        self.import.missing.extend(r.address_changed.into_iter().map(|(m, addrs)| t!("app-providers-now-lists-for-it-crow", m = m, name = name, addrs = addrs.join(", "))));
        self.import.rows.extend(r.rows);
        if !r.updates.is_empty() {
            self.reload_servers();
        }
    }

    /// Opens Add Server filled in from an instance. The usual host-key check
    /// and login still happen; nothing is trusted just because a provider
    /// listed it.
    pub fn import_instance(&mut self, account: &str, instance_id: &str, cx: &mut Context<Self>) {
        let Some(row) = self.import.rows.iter().find(|r| r.account == account && r.instance.id.0 == instance_id).cloned() else { return };
        self.import.open = false;
        self.start_onboarding(cx);
        let inst = &row.instance;
        let o = &mut self.onboard_state;
        o.host = inst.ipv4.first().or(inst.ipv6.first()).map(|ip| ip.to_string()).unwrap_or_default();
        o.label = inst.label.clone();
        o.tags = inst.tags.join(", ");
        o.provider = Some(crate::views::onboard::ProviderLink {
            account: row.account.clone(),
            instance: inst.id.0.clone(),
            provider_name: row.provider_name.clone(),
            region_code: inst.region.clone().unwrap_or_default(),
        });
        self.onboard_inputs = None;
        cx.notify();
    }

    // --- Cloud Instance Provisioning (ERR-224) ---

    pub fn close_provision(&mut self, cx: &mut Context<Self>) {
        self.provision.open = false;
        self.provision_inputs = None;
        cx.notify();
    }

    pub fn toggle_provision_dropdown(&mut self, which: &str, cx: &mut Context<Self>) {
        if self.provision.open_dropdown.as_deref() == Some(which) {
            self.provision.open_dropdown = None;
        } else {
            self.provision.open_dropdown = Some(which.to_string());
        }
        cx.notify();
    }

    pub fn set_provision_region(&mut self, region: &str, cx: &mut Context<Self>) {
        self.provision.region = region.to_string();
        self.provision.open_dropdown = None;
        cx.notify();
    }

    pub fn set_provision_region_group(&mut self, group: &str, cx: &mut Context<Self>) {
        self.provision.region_group = group.to_string();
        if group != "all" {
            if let Some(cat) = &self.provision.catalog {
                let current_matches = cat
                    .regions
                    .iter()
                    .find(|o| o.id == self.provision.region)
                    .is_some_and(|o| classify_region(o) == group);
                if !current_matches {
                    if let Some(first) = cat.regions.iter().find(|o| classify_region(o) == group) {
                        self.provision.region = first.id.clone();
                    }
                }
            }
        }
        cx.notify();
    }

    pub fn set_provision_plan(&mut self, plan: &str, cx: &mut Context<Self>) {
        self.provision.plan = plan.to_string();
        self.provision.open_dropdown = None;
        cx.notify();
    }

    pub fn set_provision_plan_tier(&mut self, tier: &str, cx: &mut Context<Self>) {
        self.provision.plan_tier = tier.to_string();
        if tier != "all" {
            if let Some(cat) = &self.provision.catalog {
                let current_matches = cat
                    .plans
                    .iter()
                    .find(|o| o.id == self.provision.plan)
                    .is_some_and(|o| classify_plan(o) == tier);
                if !current_matches {
                    if let Some(first) = cat.plans.iter().find(|o| classify_plan(o) == tier) {
                        self.provision.plan = first.id.clone();
                    }
                }
            }
        }
        cx.notify();
    }

    pub fn set_provision_image(&mut self, image: &str, cx: &mut Context<Self>) {
        self.provision.image = image.to_string();
        self.provision.open_dropdown = None;
        cx.notify();
    }

    pub fn set_provision_image_family(&mut self, family: &str, cx: &mut Context<Self>) {
        self.provision.image_family = family.to_string();
        if family != "all" {
            if let Some(cat) = &self.provision.catalog {
                let current_matches = cat
                    .images
                    .iter()
                    .find(|o| o.id == self.provision.image)
                    .is_some_and(|o| classify_image(o) == family);
                if !current_matches {
                    if let Some(first) = cat.images.iter().find(|o| classify_image(o) == family) {
                        self.provision.image = first.id.clone();
                    }
                }
            }
        }
        cx.notify();
    }

    pub fn set_provision_ssh_key(&mut self, key_id: &str, cx: &mut Context<Self>) {
        self.provision.selected_ssh_key_id = key_id.to_string();
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn open_provision(&mut self, account_id: Option<&str>, cx: &mut Context<Self>) {
        self.init_provision(account_id, true, cx);
    }

    pub fn prepare_inline_provision(&mut self, cx: &mut Context<Self>) {
        if self.provision.selected_account_id.is_empty() || self.provision.catalog.is_none() {
            self.init_provision(None, false, cx);
        }
    }

    fn init_provision(&mut self, account_id: Option<&str>, open_modal: bool, cx: &mut Context<Self>) {
        use crow_provider_core::capabilities::INSTANCES_CREATE;
        let accounts: Vec<ProviderAccount> = self
            .providers
            .accounts
            .iter()
            .filter(|a| providers::factory(&a.plugin).is_some_and(|f| (f.manifest)().has_capability(INSTANCES_CREATE)))
            .cloned()
            .collect();

        if accounts.is_empty() {
            self.provision = ProvisionState {
                open: open_modal,
                error: if open_modal { Some(t!("fleet-provision-no-providers-configured").into()) } else { None },
                ..Default::default()
            };
            self.provision_inputs = None;
            cx.notify();
            return;
        }

        let selected = match account_id {
            Some(id) if accounts.iter().any(|a| a.id == id) => id.to_string(),
            _ => accounts[0].id.clone(),
        };

        let default_ssh_key = self
            .keys
            .enrolled
            .first()
            .map(|k| k.id.clone())
            .unwrap_or_default();

        let rand_suffix: u32 = rand::random::<u32>() % 900000 + 100000;
        let default_label = format!("srv-{rand_suffix}");

        self.provision = ProvisionState {
            open: open_modal,
            loading_catalog: false,
            provisioning: false,
            selected_account_id: selected.clone(),
            label: default_label,
            region: String::new(),
            plan: String::new(),
            image: String::new(),
            region_group: "americas".to_string(),
            plan_tier: "shared".to_string(),
            image_family: "ubuntu".to_string(),
            selected_ssh_key_id: default_ssh_key,
            tags: "crow".to_string(),
            catalog: None,
            open_dropdown: None,
            error: None,
        };
        self.provision_inputs = None;
        cx.notify();

        self.load_provision_catalog(&selected, cx);
    }

    pub fn select_provision_account(&mut self, account_id: &str, cx: &mut Context<Self>) {
        self.provision.selected_account_id = account_id.to_string();
        self.provision.region.clear();
        self.provision.plan.clear();
        self.provision.image.clear();
        self.provision.catalog = None;
        self.provision.error = None;
        cx.notify();

        self.load_provision_catalog(account_id, cx);
    }

    fn load_provision_catalog(&mut self, account_id: &str, cx: &mut Context<Self>) {
        let Some(account) = self.providers.accounts.iter().find(|a| a.id == account_id).cloned() else { return };
        let Some(key) = self.vault.key().cloned() else {
            self.provision.error = Some(self.vault.secrets_blocker().unwrap_or_else(|| t!("app-providers-no-encryption-key").into()));
            cx.notify();
            return;
        };

        let settings = match self.vault.db().lock() {
            Ok(db) => match providers::load_settings(&db, &key, &account) {
                Ok(s) => s,
                Err(e) => {
                    self.provision.error = Some(format!("{}: {e}", account.label));
                    cx.notify();
                    return;
                }
            },
            Err(_) => return,
        };

        self.provision.loading_catalog = true;
        self.provision.error = None;
        cx.notify();

        let acct = account.clone();
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let provider = providers::connect(&acct, settings).map_err(|e| e.to_string())?;
                    let prov = provider.as_provision_instance().ok_or_else(|| "Provider does not support provisioning".to_string())?;
                    Ok::<_, String>(prov.catalog())
                })
                .await;

            let _ = entity.update(cx, |this, cx| {
                this.provision.loading_catalog = false;
                match result {
                    Ok(catalog) => {
                        if this.provision.region.is_empty() {
                            if let Some(r) = catalog
                                .regions
                                .iter()
                                .find(|o| classify_region(o) == this.provision.region_group)
                                .or_else(|| catalog.regions.first())
                            {
                                this.provision.region_group = classify_region(r).to_string();
                                this.provision.region = r.id.clone();
                            }
                        }
                        if this.provision.plan.is_empty() {
                            if let Some(p) = catalog
                                .plans
                                .iter()
                                .find(|o| classify_plan(o) == this.provision.plan_tier)
                                .or_else(|| catalog.plans.first())
                            {
                                this.provision.plan_tier = classify_plan(p).to_string();
                                this.provision.plan = p.id.clone();
                            }
                        }
                        if this.provision.image.is_empty() {
                            if let Some(img) = catalog
                                .images
                                .iter()
                                .find(|o| classify_image(o) == this.provision.image_family)
                                .or_else(|| catalog.images.first())
                            {
                                this.provision.image_family = classify_image(img).to_string();
                                this.provision.image = img.id.clone();
                            }
                        }
                        this.provision.catalog = Some(catalog);
                    }
                    Err(e) => {
                        this.provision.error = Some(e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn ensure_provision_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active = self.provision.open
            || (self.screen == super::Screen::Onboard
                && self.onboard_state.mode == crate::views::onboard::OnboardMode::CloudProvision);
        if self.provision_inputs.is_none() && active {
            let label_val = self.provision.label.clone();
            let tags_val = self.provision.tags.clone();
            let mut events = Vec::new();

            let label_input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("fleet-provision-label-placeholder"))
                    .default_value(label_val)
            });
            events.push(cx.subscribe_in(&label_input, window, |this, input, ev: &InputEvent, _window, cx| {
                if let InputEvent::Change = ev {
                    this.provision.label = input.read(cx).value().to_string();
                    this.provision.error = None;
                    cx.notify();
                }
            }));

            let tags_input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("fleet-provision-tags-placeholder"))
                    .default_value(tags_val)
            });
            events.push(cx.subscribe_in(&tags_input, window, |this, input, ev: &InputEvent, _window, cx| {
                if let InputEvent::Change = ev {
                    this.provision.tags = input.read(cx).value().to_string();
                    cx.notify();
                }
            }));

            self.provision_inputs = Some(ProvisionInputs {
                label: label_input,
                tags: tags_input,
                _events: events,
            });
        }
    }

    pub fn submit_provision(&mut self, cx: &mut Context<Self>) {
        if self.provision.provisioning || self.provision.loading_catalog {
            return;
        }

        let label = self.provision.label.trim().to_string();
        if label.is_empty() {
            self.provision.error = Some(t!("fleet-provision-label-required").into());
            cx.notify();
            return;
        }

        let ssh_key = self
            .keys
            .enrolled
            .iter()
            .find(|k| k.id == self.provision.selected_ssh_key_id)
            .cloned();

        let Some(key_record) = ssh_key else {
            self.provision.error = Some(t!("fleet-provision-ssh-key-required").into());
            cx.notify();
            return;
        };

        let Some(account) = self.providers.accounts.iter().find(|a| a.id == self.provision.selected_account_id).cloned() else {
            self.provision.error = Some(t!("fleet-provision-select-provider-account").into());
            cx.notify();
            return;
        };

        let Some(key) = self.vault.key().cloned() else {
            self.provision.error = Some(self.vault.secrets_blocker().unwrap_or_else(|| t!("app-providers-no-encryption-key").into()));
            cx.notify();
            return;
        };

        let settings = match self.vault.db().lock() {
            Ok(db) => match providers::load_settings(&db, &key, &account) {
                Ok(s) => s,
                Err(e) => {
                    self.provision.error = Some(format!("{}: {e}", account.label));
                    cx.notify();
                    return;
                }
            },
            Err(_) => return,
        };

        let tags_vec: Vec<String> = self
            .provision
            .tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let spec = crow_provider_core::hosts::ProvisionSpec {
            label: label.clone(),
            region: self.provision.region.clone(),
            plan: self.provision.plan.clone(),
            image: self.provision.image.clone(),
            ssh_public_key: key_record.public_key.clone(),
            tags: tags_vec,
        };

        self.provision.provisioning = true;
        self.provision.error = None;
        cx.notify();

        let acct = account.clone();
        let selected_key_id = key_record.id.clone();

        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let provider = providers::connect(&acct, settings).map_err(|e| e.to_string())?;
                    let prov = provider.as_provision_instance().ok_or_else(|| "Provider does not support provisioning".to_string())?;
                    prov.provision(&spec).map_err(|e| e.to_string())
                })
                .await;

            let _ = entity.update(cx, |this, cx| {
                this.provision.provisioning = false;
                match result {
                    Ok(instance) => {
                        this.provision.open = false;
                        this.provision_inputs = None;
                        this.start_onboarding(cx);
                        let o = &mut this.onboard_state;
                        o.host = instance
                            .ipv4
                            .first()
                            .or(instance.ipv6.first())
                            .map(|ip| ip.to_string())
                            .unwrap_or_default();
                        o.label = instance.label.clone();
                        o.tags = instance.tags.join(", ");
                        o.selected_key_id = Some(selected_key_id);
                        o.auth_method = "publickey".to_string();
                        o.provider = Some(crate::views::onboard::ProviderLink {
                            account: account.id.clone(),
                            instance: instance.id.0.clone(),
                            provider_name: providers::display_name(&account.plugin),
                            region_code: instance.region.clone().unwrap_or_default(),
                        });
                        this.onboard_inputs = None;
                        cx.notify();
                    }
                    Err(e) => {
                        this.provision.error = Some(e);
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }
}

/// Cloud instance provisioning state (`ERR-224`, `ERR-222`).
#[derive(Clone, Default)]
pub struct ProvisionState {
    pub open: bool,
    pub loading_catalog: bool,
    pub provisioning: bool,
    pub selected_account_id: String,
    pub label: String,
    pub region: String,
    pub plan: String,
    pub image: String,
    pub region_group: String,
    pub plan_tier: String,
    pub image_family: String,
    pub selected_ssh_key_id: String,
    pub tags: String,
    pub catalog: Option<crow_provider_core::hosts::ProvisionCatalog>,
    pub open_dropdown: Option<String>,
    pub error: Option<String>,
}

pub struct ProvisionInputs {
    pub label: Entity<InputState>,
    pub tags: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

/// Classifies a provider region option into `"americas"`, `"europe"`, or `"apac"`.
pub fn classify_region(opt: &crow_provider_core::hosts::ProvisionOption) -> &'static str {
    let id = opt.id.to_ascii_lowercase();
    let pfx = id.split('-').next().unwrap_or("");
    match pfx {
        "us" | "ca" | "br" | "sa" | "mx" | "cl" | "co" | "ar" => "americas",
        "eu" | "de" | "fr" | "gb" | "uk" | "nl" | "se" | "es" | "it" | "fi" | "pl" | "ch"
        | "at" | "no" | "dk" | "ie" | "be" | "pt" | "cz" | "ro" => "europe",
        "ap" | "jp" | "sg" | "in" | "id" | "au" | "kr" | "hk" | "tw" | "nz" | "my" | "th"
        | "vn" | "ph" => "apac",
        _ => {
            let lbl = opt.label.to_ascii_lowercase();
            if lbl.contains("europe")
                || lbl.contains("frankfurt")
                || lbl.contains("london")
                || lbl.contains("amsterdam")
                || lbl.contains("paris")
                || lbl.contains("madrid")
                || lbl.contains("milan")
                || lbl.contains("stockholm")
                || lbl.contains("helsinki")
                || lbl.contains("warsaw")
            {
                "europe"
            } else if lbl.contains("tokyo")
                || lbl.contains("osaka")
                || lbl.contains("singapore")
                || lbl.contains("sydney")
                || lbl.contains("melbourne")
                || lbl.contains("mumbai")
                || lbl.contains("chennai")
                || lbl.contains("jakarta")
                || lbl.contains("seoul")
            {
                "apac"
            } else {
                "americas"
            }
        }
    }
}

/// Classifies a provider instance plan into `"shared"`, `"dedicated"`, `"highmem"`, or `"gpu"`.
pub fn classify_plan(opt: &crow_provider_core::hosts::ProvisionOption) -> &'static str {
    let id = opt.id.to_ascii_lowercase();
    let lbl = opt.label.to_ascii_lowercase();
    if id.contains("gpu")
        || id.contains("rtx")
        || id.contains("quadro")
        || id.contains("accelerated")
        || lbl.contains("gpu")
        || lbl.contains("rtx")
        || lbl.contains("quadro")
        || lbl.contains("accelerated")
        || id.starts_with("g4")
        || id.starts_with("g5")
        || id.starts_with("p3")
        || id.starts_with("p4")
    {
        "gpu"
    } else if id.contains("highmem")
        || id.contains("himem")
        || lbl.contains("highmem")
        || lbl.contains("high memory")
        || id.starts_with("r6")
        || id.starts_with("r7")
    {
        "highmem"
    } else if id.contains("dedicated")
        || id.contains("premium")
        || id.contains("hicpu")
        || id.contains("cloudnative")
        || lbl.contains("dedicated")
        || lbl.contains("premium")
        || id.starts_with("c6")
        || id.starts_with("c7")
        || id.starts_with("m6")
        || id.starts_with("m7")
    {
        "dedicated"
    } else {
        "shared"
    }
}

/// Classifies an OS image option into `"ubuntu"`, `"debian"`, `"enterprise"`, or `"other"`.
pub fn classify_image(opt: &crow_provider_core::hosts::ProvisionOption) -> &'static str {
    let id = opt.id.to_ascii_lowercase();
    let lbl = opt.label.to_ascii_lowercase();
    if id.contains("ubuntu") || lbl.contains("ubuntu") {
        "ubuntu"
    } else if id.contains("debian") || lbl.contains("debian") {
        "debian"
    } else if id.contains("alma")
        || id.contains("rocky")
        || id.contains("fedora")
        || id.contains("centos")
        || id.contains("rhel")
        || id.contains("oracle")
        || id.contains("al2023")
        || id.contains("opensuse")
        || id.contains("suse")
        || lbl.contains("alma")
        || lbl.contains("rocky")
        || lbl.contains("fedora")
        || lbl.contains("centos")
        || lbl.contains("red hat")
        || lbl.contains("rhel")
        || lbl.contains("oracle")
        || lbl.contains("amazon linux")
        || lbl.contains("suse")
    {
        "enterprise"
    } else {
        "other"
    }
}
