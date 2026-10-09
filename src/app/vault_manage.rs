//! Vault & Security while Locked: change the password, or go back to Open
//! (ERR-60). Both need the current password and a 2FA code.

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;
use zeroize::Zeroize;

use super::secrets::KEYRING_URL;
use super::CrowApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultForm {
    ChangePassword,
    GoOpen,
}

impl VaultForm {
    /// (key, placeholder, masked) for each input.
    fn fields(self) -> &'static [(&'static str, &'static str, bool)] {
        match self {
            Self::ChangePassword => &[("current", "current password", true), ("code", "2FA code", false), ("new", "new password (8+ characters)", true), ("confirm", "new password again", true)],
            Self::GoOpen => &[("current", "current password", true), ("code", "2FA code", false)],
        }
    }
}

pub struct VaultFormInputs {
    pub form: VaultForm,
    pub inputs: Vec<(&'static str, Entity<InputState>)>,
    pub _events: Vec<Subscription>,
}

impl VaultFormInputs {
    fn value(&self, key: &str, cx: &App) -> String {
        self.inputs.iter().find(|(k, _)| *k == key).map(|(_, i)| i.read(cx).value().to_string()).unwrap_or_default()
    }
}

#[derive(Default)]
pub struct VaultFormState {
    pub open: Option<VaultForm>,
    pub error: Option<String>,
    pub done: Option<String>,
    pub busy: bool,
    /// GO OPEN's "I understand".
    pub ack: bool,
}

impl CrowApp {
    pub fn open_vault_form(&mut self, form: Option<VaultForm>, cx: &mut Context<Self>) {
        self.vault_form = VaultFormState { open: form, ..Default::default() };
        self.vault_form_inputs = None;
        cx.notify();
    }

    pub fn ensure_vault_form_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = self.vault_form.open else { return };
        if self.vault_form_inputs.as_ref().is_some_and(|i| i.form == form) {
            return;
        }
        let inputs: Vec<(&'static str, Entity<InputState>)> = form
            .fields()
            .iter()
            .map(|(key, placeholder, masked)| (*key, cx.new(|cx| InputState::new(window, cx).placeholder(*placeholder).masked(*masked))))
            .collect();
        let events = inputs
            .iter()
            .map(|(_, input)| {
                cx.subscribe(input, |this, _input, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.submit_vault_form(cx);
                    }
                })
            })
            .collect();
        if let Some((_, first)) = inputs.first() {
            first.update(cx, |i, cx| i.focus(window, cx));
        }
        self.vault_form_inputs = Some(VaultFormInputs { form, inputs, _events: events });
    }

    pub fn submit_vault_form(&mut self, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(inputs) = self.vault_form_inputs.as_ref() else { return };
        if self.vault_form.busy {
            return;
        }
        let mut current = inputs.value("current", cx);
        let code = inputs.value("code", cx).trim().to_string();
        match inputs.form {
            VaultForm::ChangePassword => {
                let (mut new, mut confirm) = (inputs.value("new", cx), inputs.value("confirm", cx));
                let result = if new != confirm {
                    Err("the new passwords don't match".to_string())
                } else {
                    self.vault.change_password(&current, &code, &new).map_err(|e| e.to_string())
                };
                current.zeroize();
                new.zeroize();
                confirm.zeroize();
                match result {
                    Ok(()) => {
                        self.vault_form = VaultFormState { done: Some("Password changed. Your secrets and 2FA are unchanged.".into()), ..Default::default() };
                        self.vault_form_inputs = None;
                    }
                    Err(e) => self.vault_form.error = Some(e),
                }
            }
            VaultForm::GoOpen => {
                if !self.vault_form.ack {
                    self.vault_form.error = Some("Tick \"I understand\" first.".into());
                    cx.notify();
                    return;
                }
                let key = self.vault.data_key_for_removal(&current, &code);
                current.zeroize();
                let key = match key {
                    Ok(k) => k,
                    Err(e) => {
                        self.vault_form.error = Some(e.to_string());
                        cx.notify();
                        return;
                    }
                };
                // The keyring must hold the key before the password goes;
                // otherwise every secret would become unreadable.
                self.vault_form.busy = true;
                let write = cx.write_credentials(KEYRING_URL, "crow", key.as_bytes());
                cx.spawn(async move |entity, cx| {
                    let result = write.await;
                    let _ = entity.update(cx, |this, cx| {
                        this.vault_form.busy = false;
                        match result.map_err(|e| format!("{e:#}")).and_then(|()| this.vault.finish_password_removal(key).map_err(|e| e.to_string())) {
                            Ok(()) => {
                                this.vault_form = VaultFormState { done: Some("Crow is OPEN now: no password, and the OS keyring holds the key.".into()), ..Default::default() };
                                this.vault_form_inputs = None;
                                this.on_data_key_ready(cx);
                            }
                            Err(e) => this.vault_form.error = Some(format!("Still LOCKED: couldn't hand the key to the OS keyring ({e}).")),
                        }
                        cx.notify();
                    });
                })
                .detach();
            }
        }
        cx.notify();
    }
}
