use gpui_kit::*;

use super::{ClankerEditModalState, ClankerModalFocus, CrowApp};
use crate::vault::ClankerProviderConfig;

// ==========================================
// Clankers AI Hub & Usability Methods
// ==========================================

impl CrowApp {
    pub fn refresh_clankers(&mut self, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            if let Ok(providers) = db_guard.list_clanker_providers() {
                self.clankers.providers = providers;
            }
        }
        cx.notify();
    }

    pub fn open_edit_clanker_modal(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        if let Some(p) = self.clankers.providers.iter().find(|p| p.id == provider_id) {
            let key_len = p.api_key.chars().count();
            self.clankers.editing = Some(ClankerEditModalState {
                provider_id: p.id.clone(),
                display_name: p.display_name.clone(),
                api_key_input: p.api_key.clone(),
                model_input: p.model.clone(),
                base_url_input: p.base_url.clone(),
                focus: ClankerModalFocus::ApiKey,
                error_message: None,
            });
            self.caret.place(key_len);
            self.caret.blink = true;
            cx.notify();
        }
    }

    pub fn close_edit_clanker_modal(&mut self, cx: &mut Context<Self>) {
        self.clankers.editing = None;
        cx.notify();
    }

    pub fn submit_edit_clanker(&mut self, cx: &mut Context<Self>) {
        if let Some(ref state) = self.clankers.editing.clone() {
            let p_id = state.provider_id.clone();
            let key = state.api_key_input.trim().to_string();
            let model = state.model_input.trim().to_string();
            let base_url = state.base_url_input.trim().to_string();

            if model.is_empty() {
                if let Some(ref mut st) = self.clankers.editing {
                    st.error_message = Some("Model cannot be empty".to_string());
                }
                cx.notify();
                return;
            }

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                if let Ok(Some(mut provider)) = db_guard.get_clanker_provider(&p_id) {
                    provider.api_key = key;
                    provider.model = model;
                    if !base_url.is_empty() {
                        provider.base_url = base_url;
                    }
                    let _ = db_guard.upsert_clanker_provider(&provider);
                }
            }

            self.clankers.editing = None;
            self.copy_text_with_toast("", &format!("Config updated for provider"), cx);
            self.refresh_clankers(cx);
        }
    }

    pub fn set_default_clanker(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.set_default_clanker_provider(provider_id);
        }
        self.copy_text_with_toast("", &format!("Default Clanker set to {}", provider_id), cx);
        self.refresh_clankers(cx);
    }

    pub fn reset_clanker_stats(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.reset_clanker_usage(provider_id);
        }
        self.copy_text_with_toast("", "Provider call stats reset", cx);
        self.refresh_clankers(cx);
    }

    pub fn simulate_clanker_call(&mut self, provider_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.record_clanker_usage(provider_id);
        }
        self.copy_text_with_toast("", &format!("Test call simulated (+1 call)"), cx);
        self.refresh_clankers(cx);
    }

    pub fn run_clanker_eli5(&mut self, cx: &mut Context<Self>) {
        let default_prov = self.clankers.providers.iter()
            .find(|p| p.is_default)
            .cloned()
            .unwrap_or_else(|| {
                self.clankers.providers.first().cloned().unwrap_or(ClankerProviderConfig {
                    id: "openai".into(),
                    display_name: "OpenAI".into(),
                    api_key: "".into(),
                    model: "gpt-4o-mini".into(),
                    base_url: "https://api.openai.com/v1".into(),
                    is_default: true,
                    total_calls: 0,
                    calls_30d: 0,
                    last_used_at: None,
                    daily_history: vec![],
                })
            });

        // Record call to default provider
        let prov_id = default_prov.id.clone();
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.record_clanker_usage(&prov_id);
        }
        self.refresh_clankers(cx);

        let query = self.clankers.demo_log.to_lowercase();
        let response = if query.contains("out of memory") || query.contains("oom") || query.contains("sacrifice child") {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** The server completely ran out of available RAM and swap. The Linux kernel's emergency survival reflex (\"OOM Killer\") triggered to prevent the entire host from locking up.\n\n\
                **The victim:** The kernel targeted process `mysqld` (PID 28419) because it had the highest memory badness score (`812`) and immediately sent `SIGKILL` (`-9`).\n\n\
                **What you should do next:**\n\
                1. Check memory consumption: `free -h` or `vmstat -s -S M`\n\
                2. If running MySQL, tune `innodb_buffer_pool_size` down to ~50% of total host RAM.\n\
                3. Add or increase swap space: `fallocate -l 4G /swapfile && mkswap /swapfile && swapon /swapfile`.",
                default_prov.display_name, default_prov.model
            )
        } else if query.contains("segfault") || query.contains("segmentation fault") {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** A program attempted to read or write memory that wasn't assigned to it (a null pointer or buffer overflow), so the CPU halted the program immediately.\n\n\
                **What you should do next:**\n\
                1. Inspect the stack trace: `coredumpctl info`\n\
                2. Restart the crashed daemon or check for updated package releases.",
                default_prov.display_name, default_prov.model
            )
        } else if query.contains("failed to start") || query.contains("exit-code") {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** A systemd service crashed on startup or returned a non-zero exit status during its initialization phase.\n\n\
                **What you should do next:**\n\
                1. Check exact logs for that unit: `journalctl -u <unit> -n 50 --no-pager`\n\
                2. Test manual config validity before restarting: e.g. `nginx -t` or `sshd -t`.",
                default_prov.display_name, default_prov.model
            )
        } else {
            format!(
                "💡 **ELI5 Translation** (via {} / `{}`):\n\n\
                **What happened:** The system logged an operational notification or warning event. Everything is still functioning, but an underlying component is reporting non-standard behavior.\n\n\
                **Suggested action:** Monitor logs for recurring occurrences or check journal filtering for PID details.",
                default_prov.display_name, default_prov.model
            )
        };

        self.clankers.demo_output = Some(response);
        cx.notify();
    }

    // --- Server Enrollment Subsystem ---
}
