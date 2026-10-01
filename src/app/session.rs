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
        self.config.saved_int("security.auto_lock_minutes").unwrap_or(15).max(0)
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
            // Terminal sessions end with the lock, like SSH connections.
            self.close_all_terminals();
            self.lock_inputs = None;
            self.lock_state = Default::default();
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

    /// Creates the lock screen's inputs (masked password, 6-digit code),
    /// focused on the password; Enter in either unlocks.
    pub fn ensure_lock_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use gpui_kit::component::input::{InputEvent, InputState};
        if self.lock_inputs.is_some() {
            return;
        }
        let masked = !self.lock_state.show_password;
        let password = cx.new(|cx| InputState::new(window, cx).placeholder("Enter master password…").masked(masked));
        let code = cx.new(|cx| InputState::new(window, cx).placeholder("000000"));
        let events = [&password, &code]
            .into_iter()
            .map(|input| {
                cx.subscribe(input, |this, _input, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.submit_unlock(cx);
                    }
                })
            })
            .collect();
        password.update(cx, |i, cx| i.focus(window, cx));
        self.lock_inputs = Some(super::LockInputs { password, code, _events: events });
    }

    pub fn toggle_lock_show_password(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.lock_state.show_password = !self.lock_state.show_password;
        let masked = !self.lock_state.show_password;
        if let Some(i) = self.lock_inputs.as_ref() {
            i.password.update(cx, |i, cx| i.set_masked(masked, window, cx));
        }
        cx.notify();
    }

    pub fn submit_unlock(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.lock_inputs.as_ref() else { return };
        let pwd = zeroize::Zeroizing::new(inputs.password.read(cx).value().to_string());
        let code: String = inputs.code.read(cx).value().chars().filter(|c| c.is_ascii_digit()).collect();
        if pwd.is_empty() {
            self.lock_state.error_message = Some("Password cannot be empty".into());
            cx.notify();
            return;
        }
        let totp = code.as_str();
        if totp.is_empty() {
            self.lock_state.error_message = Some("6-digit 2FA code is required".into());
            cx.notify();
            return;
        }

        match self.vault.unlock(&pwd, totp) {
            Ok(_) => {
                self.lock_inputs = None;
                self.lock_state.error_message = None;
                self.on_data_key_ready(cx);
                cx.notify();
            }
            Err(e) => {
                self.lock_state.error_message = Some(e.to_string());
                cx.notify();
            }
        }
    }

    /// Vault setup, step 2: a fresh TOTP secret (once) and its QR code.
    pub fn enter_setup_credentials(&mut self, cx: &mut Context<Self>) {
        use crate::views::lock::SetupStep;
        self.setup_state.step = SetupStep::ConfigureCredentials;
        if self.setup_state.totp_secret.is_empty() {
            self.setup_state.totp_secret = crate::vault::generate_totp_secret();
        }
        if self.setup_state.totp_qr.is_none() {
            let account = format!(
                "{}@{}",
                std::env::var("USER").unwrap_or_else(|_| "crow".into()),
                std::fs::read_to_string("/etc/hostname").map(|h| h.trim().to_string()).unwrap_or_else(|_| "this computer".into())
            );
            self.setup_state.totp_qr = crate::vault::totp_qr_png(&self.setup_state.totp_secret, &account)
                .ok()
                .map(|png| std::sync::Arc::new(Image::from_bytes(ImageFormat::Png, png)));
        }
        self.setup_inputs = None;
        cx.notify();
    }

    /// Creates the setup inputs (masked passwords, the 6-digit code);
    /// Enter in any of them activates.
    pub fn ensure_setup_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use gpui_kit::component::input::{InputEvent, InputState};
        if self.setup_inputs.is_some() {
            return;
        }
        let masked = !self.setup_state.show_password;
        let password = cx.new(|cx| InputState::new(window, cx).placeholder("Enter master password (min 8 characters)…").masked(masked));
        let confirm = cx.new(|cx| InputState::new(window, cx).placeholder("Re-type password…").masked(masked));
        let code = cx.new(|cx| InputState::new(window, cx).placeholder("6-digit code (e.g. 123456)"));
        let events = [&password, &confirm, &code]
            .into_iter()
            .map(|input| {
                cx.subscribe(input, |this, _input, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.submit_setup(cx);
                    }
                })
            })
            .collect();
        password.update(cx, |i, cx| i.focus(window, cx));
        self.setup_inputs = Some(super::SetupInputs { password, confirm, code, _events: events });
    }

    pub fn toggle_setup_show_password(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.setup_state.show_password = !self.setup_state.show_password;
        let masked = !self.setup_state.show_password;
        if let Some(inputs) = self.setup_inputs.as_ref() {
            for i in [&inputs.password, &inputs.confirm] {
                i.update(cx, |i, cx| i.set_masked(masked, window, cx));
            }
        }
        cx.notify();
    }

    /// Copies the TOTP secret, for authenticators that take it pasted.
    pub fn copy_setup_secret(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.setup_state.totp_secret.clone()));
        self.setup_state.secret_copied = true;
        cx.notify();
    }

    pub fn submit_setup(&mut self, cx: &mut Context<Self>) {
        let Some(inputs) = self.setup_inputs.as_ref() else { return };
        let pwd = zeroize::Zeroizing::new(inputs.password.read(cx).value().to_string());
        let confirm = zeroize::Zeroizing::new(inputs.confirm.read(cx).value().to_string());
        let code: String = inputs.code.read(cx).value().chars().filter(|c| c.is_ascii_digit()).collect();

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

        let code = code.as_str();
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
                self.setup_inputs = None;
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
        }
        self.screen = screen;
        self.menu_open = false;
        self.palette_open = false;
        cx.notify();
    }
}
