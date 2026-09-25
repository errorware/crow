use gpui_kit::*;

use gpui_kit::component::input::{InputEvent, InputState};

use super::{ClankerEditModalState, ClankerInputs, CrowApp};
use crate::vault::ClankerProviderConfig;
use zeroize::Zeroize;

// ==========================================
// Clankers AI Hub & Usability Methods
// ==========================================

impl CrowApp {
    pub fn refresh_clankers(&mut self, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            if let Ok(mut providers) = db_guard.list_clanker_providers() {
                // Keys come from the vault; the table's column is blank (or,
                // before migration, an older plain-text key).
                if let Some(key) = self.vault.key() {
                    db_guard.load_clanker_api_keys(key, &mut providers);
                }
                self.clankers.providers = providers;
            }
        }
        cx.notify();
    }

    pub fn open_edit_clanker_modal(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        if let Some(p) = self.clankers.providers.iter().find(|p| p.id == provider_id) {
            // The stored key is never shown back: its field starts empty and
            // a blank field keeps it.
            self.clankers.editing = Some(ClankerEditModalState {
                provider_id: p.id.clone(),
                display_name: p.display_name.clone(),
                has_key: !p.api_key.trim().is_empty(),
                model: p.model.clone(),
                base_url: p.base_url.clone(),
                error_message: None,
            });
            self.clanker_inputs = None;
            cx.notify();
        }
    }

    /// Creates the open dialog's inputs; Enter in any of them saves.
    pub fn ensure_clanker_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(st) = self.clankers.editing.clone() else { return };
        if self.clanker_inputs.as_ref().is_some_and(|i| i.provider_id == st.provider_id) {
            return;
        }
        let placeholder = if st.has_key { "set · leave blank to keep it, or paste a new key" } else { "e.g. sk-proj-..." };
        let key = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).masked(true));
        let model = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. gpt-4o-mini").default_value(st.model.clone()));
        let base_url = cx.new(|cx| InputState::new(window, cx).placeholder("https://api.openai.com/v1").default_value(st.base_url.clone()));
        let events = [&key, &model, &base_url]
            .into_iter()
            .map(|input| {
                cx.subscribe(input, |this, _input, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.submit_edit_clanker(cx);
                    }
                })
            })
            .collect();
        key.update(cx, |i, cx| i.focus(window, cx));
        self.clanker_inputs = Some(ClankerInputs { provider_id: st.provider_id, key, model, base_url, _events: events });
    }

    pub fn close_edit_clanker_modal(&mut self, cx: &mut Context<Self>) {
        self.clankers.editing = None;
        self.clanker_inputs = None;
        cx.notify();
    }

    pub fn submit_edit_clanker(&mut self, cx: &mut Context<Self>) {
        let (Some(state), Some(inputs)) = (self.clankers.editing.clone(), self.clanker_inputs.as_ref()) else { return };
        let p_id = state.provider_id.clone();
        let key = inputs.key.read(cx).value().trim().to_string();
        let model = inputs.model.read(cx).value().trim().to_string();
        let base_url = inputs.base_url.read(cx).value().trim().to_string();

        if model.is_empty() {
            if let Some(ref mut st) = self.clankers.editing {
                st.error_message = Some("Model cannot be empty".to_string());
            }
            cx.notify();
            return;
        }

        let key_is_empty = key.is_empty();
        let save = move |this: &mut Self, ready: Result<(), String>, cx: &mut Context<Self>| {
            let mut key = key;
            let result = ready.and_then(|()| {
                let db = this.vault.db();
                let db_guard = db.lock().map_err(|_| "the vault is busy; try again".to_string())?;
                let mut provider = db_guard.get_clanker_provider(&p_id).ok().flatten().ok_or("unknown provider")?;
                provider.model = model;
                if !base_url.is_empty() {
                    provider.base_url = base_url;
                }
                db_guard.upsert_clanker_provider(&provider).map_err(|e| e.to_string())?;
                if !key.is_empty() {
                    let data_key = this.vault.key().ok_or("no encryption key")?;
                    db_guard.set_clanker_api_key(data_key, &p_id, Some(&key)).map_err(|e| e.to_string())?;
                }
                Ok(())
            });
            key.zeroize();
            match result {
                Ok(()) => {
                    this.clankers.editing = None;
                    this.clanker_inputs = None;
                }
                Err(e) => {
                    if let Some(ref mut st) = this.clankers.editing {
                        st.error_message = Some(format!("Not saved: {e}"));
                    }
                }
            }
            this.refresh_clankers(cx);
        };
        if key_is_empty {
            save(self, Ok(()), cx);
        } else {
            self.with_data_key(cx, save);
        }
    }

    pub fn set_default_clanker(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.set_default_clanker_provider(provider_id);
        }
        self.keys.toast = Some(format!("Default Clanker set to {provider_id}"));
        self.refresh_clankers(cx);
    }

    pub fn reset_clanker_stats(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.reset_clanker_usage(provider_id);
        }
        self.keys.toast = Some("Provider call stats reset".into());
        self.refresh_clankers(cx);
    }

    /// Checks a provider's key with a real, minimal request (a few tokens).
    pub fn test_clanker_key(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let Some(provider) = self.clankers.providers.iter().find(|p| p.id == provider_id).cloned() else { return };
        if !self.clankers.key_checking.insert(provider.id.clone()) {
            return;
        }
        self.clankers.key_checks.remove(&provider.id);
        cx.notify();
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let id = provider.id.clone();
            let result = cx.background_executor().spawn(async move { crate::ai::check_key(&provider) }).await;
            if result.is_ok() {
                if let Ok(db) = db.lock() {
                    let _ = db.record_clanker_call(&id);
                }
            }
            let _ = entity.update(cx, |this, cx| {
                this.clankers.key_checking.remove(&id);
                this.clankers.key_checks.insert(id, result.map(|_| "key works".to_string()));
                this.refresh_clankers(cx);
            });
        })
        .detach();
    }

    /// Sends the sandbox's log line to the default provider (or the first
    /// with a key) and shows its real answer.
    pub fn run_clanker_eli5(&mut self, cx: &mut Context<Self>) {
        let keyed = |p: &&ClankerProviderConfig| !p.api_key.trim().is_empty();
        let provider = self.clankers.providers.iter().filter(keyed).find(|p| p.is_default).or_else(|| self.clankers.providers.iter().find(keyed)).cloned();
        let Some(provider) = provider else {
            self.clankers.demo_output = Some(Err("No AI provider has an API key yet. Add one above with ⚙ Edit Key.".into()));
            cx.notify();
            return;
        };
        if self.clankers.demo_loading {
            return;
        }
        let log = self.clankers.demo_log.clone();
        self.clankers.demo_loading = true;
        self.clankers.demo_output = None;
        cx.notify();
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let id = provider.id.clone();
            let label = format!("{} · {}", provider.display_name, provider.model);
            let answer = cx
                .background_executor()
                .spawn(async move { crate::ai::explain_logs(&provider, "A single journal line pasted into Crow's ELI5 sandbox.", &log) })
                .await;
            if answer.is_ok() {
                if let Ok(db) = db.lock() {
                    let _ = db.record_clanker_call(&id);
                }
            }
            let _ = entity.update(cx, |this, cx| {
                this.clankers.demo_loading = false;
                this.clankers.demo_output = Some(answer.map(|a| format!("via {label}\n\n{a}")));
                this.refresh_clankers(cx);
            });
        })
        .detach();
    }

    // --- Server Enrollment Subsystem ---
}
