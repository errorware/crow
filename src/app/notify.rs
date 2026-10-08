//! Email notifications in the app (ERR-139): the Email plugin's form, its
//! test button, and the tick that turns alerts into mail.

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;
use zeroize::Zeroize;

use super::CrowApp;
use crate::notify::email::{self, Delivery, DispatchState, EmailSettings, Security};

/// How often the open alerts are looked at for new mail.
const TICK_SECS: i64 = 30;

#[derive(Default)]
pub struct EmailState {
    /// What's saved; `None` until the form is saved once.
    pub saved: Option<EmailSettings>,
    pub has_password: bool,
    /// The form's choices that aren't text.
    pub security: Security,
    pub crit: Option<Delivery>,
    pub warn: Option<Delivery>,
    /// The form's last complaint, or what SAVE did.
    pub form_message: Option<(bool, String)>,
    pub testing: bool,
    /// What SEND TEST did: (it worked, what happened).
    pub test_result: Option<(bool, String)>,
    pub sending: bool,
    pub last_tick: i64,
    /// A copy of the dispatcher's state, for the card.
    pub dispatch: DispatchState,
}

pub struct EmailInputs {
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub from: Entity<InputState>,
    pub to: Entity<InputState>,
    pub digest: Entity<InputState>,
    pub quiet: Entity<InputState>,
    _events: Vec<Subscription>,
}

impl CrowApp {
    /// Reads the saved settings and the dispatcher's state.
    pub fn load_email(&mut self) {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let saved: Option<EmailSettings> = db.flag(email::SETTINGS_FLAG).and_then(|s| serde_json::from_str(&s).ok());
        let has_password = db.list_entries(Some(email::PASSWORD_CATEGORY)).is_ok_and(|e| e.iter().any(|m| m.id == email::PASSWORD_ENTRY));
        let dispatch = db.flag(email::STATE_FLAG).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        let st = &mut self.email;
        st.security = saved.as_ref().map(|s| s.security).unwrap_or_default();
        st.crit = saved.as_ref().map(|s| s.crit);
        st.warn = saved.as_ref().map(|s| s.warn);
        st.saved = saved;
        st.has_password = has_password;
        st.dispatch = dispatch;
    }

    /// Creates the Email card's inputs, filled from what's saved.
    pub fn ensure_email_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.email_inputs.is_some() {
            return;
        }
        let s = self.email.saved.clone().unwrap_or_default();
        let input = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str, value: String| {
            let placeholder = placeholder.to_string();
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).default_value(value))
        };
        let host = input(window, cx, "smtp.example.com", s.host.clone());
        let port = input(window, cx, "465", s.port.to_string());
        let username = input(window, cx, "blank if the server needs no login", s.username.clone());
        let pw_placeholder = if self.email.has_password { "set · leave blank to keep it" } else { "the SMTP password or app password" };
        let password = cx.new(|cx| InputState::new(window, cx).placeholder(pw_placeholder).masked(true));
        let from = input(window, cx, "crow@example.com", s.from.clone());
        let to = input(window, cx, "you@example.com, oncall@example.com", s.to.join(", "));
        let digest = input(window, cx, "60", s.digest_minutes.to_string());
        let quiet = input(window, cx, "e.g. 22-07, or blank", email::format_quiet_hours(s.quiet_hours));
        let events = [&host, &port, &username, &password, &from, &to, &digest, &quiet]
            .into_iter()
            .map(|i| {
                cx.subscribe(i, |this, _i, ev: &InputEvent, cx| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        this.save_email(cx);
                    }
                })
            })
            .collect();
        self.email_inputs = Some(EmailInputs { host, port, username, password, from, to, digest, quiet, _events: events });
    }

    /// Picks the security, moving the port along if it was the old default.
    pub fn set_email_security(&mut self, security: Security, window: &mut Window, cx: &mut Context<Self>) {
        let old = self.email.security;
        self.email.security = security;
        if let Some(inputs) = &self.email_inputs {
            let port = inputs.port.read(cx).value().trim().to_string();
            if port.is_empty() || port == old.default_port().to_string() {
                inputs.port.update(cx, |i, cx| i.set_value(security.default_port().to_string(), window, cx));
            }
        }
        cx.notify();
    }

    pub fn set_email_delivery(&mut self, crit: bool, delivery: Delivery, cx: &mut Context<Self>) {
        if crit {
            self.email.crit = Some(delivery);
        } else {
            self.email.warn = Some(delivery);
        }
        cx.notify();
    }

    /// The form as settings, and the password typed (blank: keep the saved one).
    fn email_form(&self, cx: &App) -> Result<(EmailSettings, String), String> {
        let inputs = self.email_inputs.as_ref().ok_or("the form isn't open")?;
        let text = |i: &Entity<InputState>| i.read(cx).value().trim().to_string();
        let port: u16 = text(&inputs.port).parse().map_err(|_| "the port is a number, like 465 or 587".to_string())?;
        let digest_minutes: u32 = text(&inputs.digest).parse().map_err(|_| "the digest interval is a number of minutes".to_string())?;
        let defaults = EmailSettings::default();
        let settings = EmailSettings {
            host: text(&inputs.host),
            port,
            security: self.email.security,
            username: text(&inputs.username),
            from: text(&inputs.from),
            to: email::parse_recipients(&text(&inputs.to)),
            crit: self.email.crit.unwrap_or(defaults.crit),
            warn: self.email.warn.unwrap_or(defaults.warn),
            digest_minutes,
            quiet_hours: email::parse_quiet_hours(&text(&inputs.quiet))?,
        };
        settings.validate()?;
        if !settings.username.is_empty() && !self.email.has_password && text(&inputs.password).is_empty() {
            return Err("there's a username but no password".into());
        }
        Ok((settings, inputs.password.read(cx).value().to_string()))
    }

    pub fn save_email(&mut self, cx: &mut Context<Self>) {
        let (settings, password) = match self.email_form(cx) {
            Ok(f) => f,
            Err(e) => {
                self.email.form_message = Some((false, e));
                cx.notify();
                return;
            }
        };
        let store = move |this: &mut Self, ready: Result<(), String>, cx: &mut Context<Self>| {
            let mut password = password;
            let result = ready.and_then(|()| {
                let db = this.vault.db();
                let db = db.lock().map_err(|_| "the vault is busy; try again".to_string())?;
                if !password.is_empty() {
                    let key = this.vault.key().ok_or("the vault is locked")?;
                    db.store_entry(key, email::PASSWORD_ENTRY, email::PASSWORD_CATEGORY, "SMTP password", password.as_bytes()).map_err(|e| e.to_string())?;
                } else if settings.username.is_empty() {
                    // No login: no password to keep.
                    let _ = db.delete_entry(email::PASSWORD_ENTRY);
                }
                db.set_flag(email::SETTINGS_FLAG, &serde_json::to_string(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
            });
            password.zeroize();
            this.email.form_message = Some(match result {
                Ok(()) => (true, "Saved. Alerts are mailed while Crow is running.".into()),
                Err(e) => (false, format!("Not saved: {e}")),
            });
            this.load_email();
            this.check_plugin("email", cx);
            // Rebuilt from what's saved: the password field comes back empty.
            this.email_inputs = None;
            cx.notify();
        };
        if self.email_form(cx).is_ok_and(|(_, p)| p.is_empty()) {
            store(self, Ok(()), cx);
        } else {
            self.with_data_key(cx, store);
        }
    }

    /// The saved password, if the vault is open.
    fn email_password(&self) -> Option<String> {
        let key = self.vault.key()?;
        let db = self.vault.db();
        let db = db.lock().ok()?;
        db.load_entry(key, email::PASSWORD_ENTRY).ok().and_then(|b| String::from_utf8(b).ok())
    }

    /// Sends one test mail with what's in the form (saved or not).
    pub fn send_test_email(&mut self, cx: &mut Context<Self>) {
        if self.email.testing {
            return;
        }
        let (settings, typed) = match self.email_form(cx) {
            Ok(f) => f,
            Err(e) => {
                self.email.test_result = Some((false, e));
                cx.notify();
                return;
            }
        };
        let password = if typed.is_empty() { self.email_password() } else { Some(typed) };
        if !settings.username.is_empty() && password.is_none() {
            self.email.test_result = Some((false, "the saved password can't be read while the vault is locked".into()));
            cx.notify();
            return;
        }
        self.email.testing = true;
        self.email.test_result = None;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let to = settings.to.join(", ");
            let (sent, err) = cx
                .background_executor()
                .spawn(async move {
                    let mut password = password;
                    let r = email::send(&settings, password.as_deref(), &[email::test_mail(chrono::Utc::now().timestamp())]);
                    password.zeroize();
                    r
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.email.testing = false;
                this.email.test_result = Some(match err {
                    None if sent == 1 => (true, format!("Sent to {to}. Check the inbox (and spam).")),
                    Some(e) => (false, e),
                    None => (false, "nothing was sent".into()),
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Every poll: with the Email plugin on and saved, turns new alerts into
    /// mail and sends what's waiting, in the background.
    pub fn notify_tick(&mut self, cx: &mut Context<Self>) {
        let now = chrono::Utc::now().timestamp();
        if now - self.email.last_tick < TICK_SECS || !self.plugin_enabled("email") || self.vault.status() == crate::vault::VaultStatus::Locked {
            return;
        }
        let Some(settings) = self.email.saved.clone() else { return };
        self.email.last_tick = now;
        let names: email::ServerNames = self.fleet.servers.iter().chain(self.fleet.archived.iter()).map(|s| (s.id.clone(), (s.name.clone(), s.host.clone()))).collect();
        let hour = chrono::Timelike::hour(&chrono::Local::now()) as u8;
        let mut state = {
            let db = self.vault.db();
            let Ok(db) = db.lock() else { return };
            let alerts = db.list_alerts(i64::MAX).unwrap_or_default();
            let mut state: DispatchState = db.flag(email::STATE_FLAG).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
            email::plan(&mut state, &alerts, &names, &settings, now, hour);
            let _ = db.set_flag(email::STATE_FLAG, &serde_json::to_string(&state).unwrap_or_default());
            state
        };
        if self.email.sending || !email::send_due(&state, now) {
            self.email.dispatch = state;
            return;
        }
        let password = if settings.username.is_empty() { None } else { self.email_password() };
        if !settings.username.is_empty() && password.is_none() {
            state.last_error = Some("the SMTP password can't be read (is the vault locked?)".into());
            state.last_attempt = now;
            self.save_dispatch(&state);
            self.email.dispatch = state;
            return;
        }
        let batch = state.outbox.clone();
        self.email.dispatch = state;
        self.email.sending = true;
        cx.spawn(async move |entity, cx| {
            let to_send = batch.clone();
            let (sent, err) = cx
                .background_executor()
                .spawn(async move {
                    let mut password = password;
                    let r = email::send(&settings, password.as_deref(), &to_send);
                    password.zeroize();
                    r
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.email.sending = false;
                let db = this.vault.db();
                let Ok(db) = db.lock() else { return };
                let mut state: DispatchState = db.flag(email::STATE_FLAG).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                drop(db);
                let now = chrono::Utc::now().timestamp();
                // Only what went out leaves the outbox (more may have joined).
                for m in &batch[..sent] {
                    if let Some(i) = state.outbox.iter().position(|o| o == m) {
                        state.outbox.remove(i);
                    }
                }
                state.last_attempt = now;
                if sent > 0 {
                    state.last_sent = Some(now);
                }
                state.last_error = err;
                this.save_dispatch(&state);
                this.email.dispatch = state;
                cx.notify();
            });
        })
        .detach();
    }

    fn save_dispatch(&self, state: &DispatchState) {
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(email::STATE_FLAG, &serde_json::to_string(state).unwrap_or_default());
        }
    }
}
