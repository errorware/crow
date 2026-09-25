use gpui_kit::*;

use super::{CrowApp, Screen, SettingsSection};
use crate::views::lock::SetupState;
use crate::views::onboard::OnboardState;

impl CrowApp {
    /// Locking forgets every secret held in memory and closes the SSH
    /// master connections, so nothing running as you can reuse them to
    /// reach your servers while Crow is locked.
    fn wipe_session_secrets(&mut self, cx: &mut Context<Self>) {
        use zeroize::Zeroize;
        for p in &mut self.clankers.providers {
            p.api_key.zeroize();
        }
        self.clankers.editing = None;
        self.provider_inputs = None;
        self.providers.editing = None;
        let servers = self.fleet.servers.clone();
        cx.background_executor()
            .spawn(async move {
                for s in &servers {
                    crate::host::host_for(s).close_connection();
                }
            })
            .detach();
    }

    /// Minutes of inactivity before Crow locks (Settings → Vault & Security);
    /// 0 means never.
    pub fn auto_lock_minutes(&self) -> i64 {
        self.config
            .get_field("security.auto_lock_minutes")
            .and_then(|f| f.value.as_i64().or_else(|| f.value.as_str()?.trim().parse().ok()))
            .unwrap_or(15)
            .max(0)
    }

    /// Locks an unlocked vault once nobody has touched Crow for the
    /// auto-lock period.
    pub fn check_auto_lock(&mut self, cx: &mut Context<Self>) {
        let minutes = self.auto_lock_minutes();
        if minutes > 0 && self.vault.status() == crate::vault::VaultStatus::Unlocked && self.last_activity.elapsed() >= std::time::Duration::from_secs(minutes as u64 * 60) {
            self.lock(cx);
        }
    }

    pub fn lock(&mut self, cx: &mut Context<Self>) {
        if self.vault.is_password_auth_enabled() {
            self.vault.lock();
            self.wipe_session_secrets(cx);
            self.lock_state.password_input.clear();
            self.lock_state.totp_input.clear();
            self.lock_state.error_message = None;
            self.caret.place(0);
            self.menu_open = false;
            self.palette_open = false;
            cx.notify();
        } else {
            self.screen = Screen::Settings;
            self.settings.section = SettingsSection::Security;
            self.menu_open = false;
            self.palette_open = false;
            cx.notify();
        }
    }

    pub fn submit_unlock(&mut self, cx: &mut Context<Self>) {
        let pwd = self.lock_state.password_input.clone();
        if pwd.is_empty() {
            self.lock_state.error_message = Some("Password cannot be empty".into());
            cx.notify();
            return;
        }
        let totp = self.lock_state.totp_input.trim();
        if totp.is_empty() {
            self.lock_state.error_message = Some("6-digit 2FA code is required".into());
            cx.notify();
            return;
        }

        match self.vault.unlock(&pwd, totp) {
            Ok(_) => {
                self.lock_state.password_input.clear();
                self.lock_state.totp_input.clear();
                self.lock_state.error_message = None;
                self.caret.place(0);
                self.on_data_key_ready(cx);
                cx.notify();
            }
            Err(e) => {
                self.lock_state.error_message = Some(e.to_string());
                cx.notify();
            }
        }
    }

    pub fn submit_setup(&mut self, cx: &mut Context<Self>) {
        let pwd = self.setup_state.password_input.clone();
        let confirm = self.setup_state.confirm_input.clone();

        if pwd.len() < 8 {
            self.setup_state.error_message = Some("Password must be at least 8 characters".into());
            cx.notify();
            return;
        }
        if pwd != confirm {
            self.setup_state.error_message = Some("Passwords do not match".into());
            cx.notify();
            return;
        }

        let code = self.setup_state.totp_confirm_input.trim();
        if code.is_empty() {
            self.setup_state.error_message = Some("Please enter the 6-digit confirmation code from your authenticator".into());
            cx.notify();
            return;
        }
        if !crate::vault::verify_totp_code(&self.setup_state.totp_secret, code) {
            self.setup_state.error_message = Some("Invalid 6-digit code. Check your authenticator and try again".into());
            cx.notify();
            return;
        }

        match self.vault.initialize(&pwd, &self.setup_state.totp_secret) {
            Ok(_) => {
                // The data key now lives in the vault, wrapped by the password.
                self.forget_keyring_key(cx);
                self.on_data_key_ready(cx);
                self.setup_state = SetupState::default();
                self.caret.place(0);
                self.screen = Screen::Fleet;
                cx.notify();
            }
            Err(e) => {
                self.setup_state.error_message = Some(e.to_string());
                cx.notify();
            }
        }
    }

    pub fn set_screen(&mut self, screen: Screen, cx: &mut Context<Self>) {
        if screen == Screen::Onboard && self.screen != Screen::Onboard {
            // A fresh wizard: new state, and new inputs so the text boxes
            // don't keep the last server's values.
            self.onboard_state = OnboardState::new(&self.keys.enrolled);
            self.onboard_inputs = None;
            self.onboard_focus_pending = true;
        } else if screen == Screen::VaultSetup && self.screen != Screen::VaultSetup {
            self.setup_state = SetupState::default();
            self.caret.place(0);
        }
        self.screen = screen;
        self.menu_open = false;
        self.palette_open = false;
        cx.notify();
    }
}
