//! A user plugin's settings (ERR-182): a form under its review in Settings
//! → Plugins. Plain values are vault flags; secrets are sealed with the
//! vault's key like provider tokens, never shown again, and reach actions
//! only as environment variables. Whatever a run prints is cleared of them
//! before it's shown or kept.

use std::collections::{BTreeMap, HashMap};

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::plugin_host::manifest::{Field, FieldKind};
use crate::plugin_host::run;
use crate::vault::{ChangeRecord, AUDIT_SERVER_ID};

pub const SECRET_CATEGORY: &str = "user_plugin";

fn flag(id: &str, key: &str) -> String {
    format!("user_plugin.{id}.setting.{key}")
}

fn secret_id(id: &str, key: &str) -> String {
    format!("user_plugin:{id}:{key}")
}

/// The form's boxes and picks, per (plugin, key).
#[derive(Default)]
pub struct SettingsForm {
    pub inputs: HashMap<(String, String), Entity<InputState>>,
    pub picked: HashMap<(String, String), String>,
    /// Secrets that have a saved value (never the value itself).
    pub saved_secrets: HashMap<(String, String), bool>,
    pub message: Option<(bool, String)>,
}

/// `text` with every secret value replaced by •••.
pub fn redact(text: &str, secrets: &BTreeMap<String, String>) -> String {
    let mut out = text.to_string();
    // Longest first, so a secret containing another is covered whole.
    let mut values: Vec<&String> = secrets.values().filter(|v| v.len() >= 3).collect();
    values.sort_by_key(|v| std::cmp::Reverse(v.len()));
    for v in values {
        out = out.replace(v.as_str(), "•••");
    }
    out
}

impl CrowApp {
    /// What's stored for a plugin's settings, secrets included: the values
    /// actions are filled from.
    pub fn user_plugin_settings(&self, id: &str, fields: &[Field]) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return out };
        for f in fields {
            let value = if f.kind == FieldKind::Secret {
                self.vault.key().and_then(|k| db.load_entry(k, &secret_id(id, &f.key)).ok()).and_then(|b| String::from_utf8(b).ok())
            } else {
                db.flag(&flag(id, &f.key)).filter(|v| !v.is_empty())
            };
            if let Some(v) = value {
                out.insert(f.key.clone(), v);
            }
        }
        out
    }

    /// The form for an open review: text boxes for text, numbers and
    /// secrets; picks for the rest, from what's stored.
    pub fn ensure_plugin_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.plugins.user.reviewing.clone() else { return };
        let Some(fields) = self.plugins.user.get(&id).filter(|p| p.usable()).and_then(|p| p.report.manifest.as_ref()).map(|m| m.settings.clone()) else { return };
        if fields.is_empty() || fields.iter().all(|f| self.plugins.user.settings.inputs.contains_key(&(id.clone(), f.key.clone())) || self.plugins.user.settings.picked.contains_key(&(id.clone(), f.key.clone()))) {
            return;
        }
        let stored = self.user_plugin_settings(&id, &fields);
        for f in &fields {
            let k = (id.clone(), f.key.clone());
            match f.kind {
                FieldKind::Text | FieldKind::Number => {
                    let value = stored.get(&f.key).cloned().or_else(|| run::default_value(f)).unwrap_or_default();
                    let state = cx.new(|cx| InputState::new(window, cx).placeholder(f.label.clone()).default_value(value));
                    self.plugins.user.settings.inputs.insert(k, state);
                }
                FieldKind::Secret => {
                    let saved = stored.contains_key(&f.key);
                    let placeholder = if saved { t!("app-user-plugin-settings-secret-saved").to_string() } else { f.label.clone() };
                    let state = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).masked(true));
                    self.plugins.user.settings.inputs.insert(k.clone(), state);
                    self.plugins.user.settings.saved_secrets.insert(k, saved);
                }
                FieldKind::Bool | FieldKind::Choice | FieldKind::Choices => {
                    let value = stored.get(&f.key).cloned().or_else(|| run::default_value(f)).unwrap_or_default();
                    self.plugins.user.settings.picked.insert(k, value);
                }
            }
        }
    }

    pub fn pick_plugin_setting(&mut self, id: &str, key: &str, value: String, cx: &mut Context<Self>) {
        self.plugins.user.settings.picked.insert((id.to_string(), key.to_string()), value);
        cx.notify();
    }

    /// SAVE: checks every value, then stores them. An empty secret box
    /// keeps the saved secret.
    pub fn save_plugin_settings(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(p) = self.plugins.user.get(id).filter(|p| p.usable()).cloned() else { return };
        let fields = p.report.manifest.as_ref().map(|m| m.settings.clone()).unwrap_or_default();
        let mut plain = Vec::new();
        let mut secrets = Vec::new();
        for f in &fields {
            let k = (id.to_string(), f.key.clone());
            let raw = match self.plugins.user.settings.inputs.get(&k) {
                Some(input) => input.read(cx).value().to_string(),
                None => self.plugins.user.settings.picked.get(&k).cloned().unwrap_or_default(),
            };
            if f.kind == FieldKind::Secret {
                if raw.is_empty() {
                    if f.required && !self.plugins.user.settings.saved_secrets.get(&k).copied().unwrap_or(false) {
                        self.plugins.user.settings.message = Some((false, format!("{}: {}", f.label, t!("app-user-plugin-settings-needed"))));
                        cx.notify();
                        return;
                    }
                    continue;
                }
                secrets.push((f.key.clone(), f.label.clone(), raw));
                continue;
            }
            match run::check_value(f, &raw) {
                Ok(v) => plain.push((f.key.clone(), v)),
                Err(e) => {
                    self.plugins.user.settings.message = Some((false, e));
                    cx.notify();
                    return;
                }
            }
        }
        let Some(key) = self.vault.key().cloned() else {
            self.plugins.user.settings.message = Some((false, t!("app-user-plugin-settings-vault-locked").to_string()));
            cx.notify();
            return;
        };
        if let Ok(db) = self.vault.db().lock() {
            for (k, v) in &plain {
                let _ = db.set_flag(&flag(id, k), v);
            }
            for (k, label, v) in &secrets {
                let _ = db.store_entry(&key, &secret_id(id, k), SECRET_CATEGORY, &format!("{} · {label}", p.name()), v.as_bytes());
            }
            let _ = db.insert_change_record(&ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: AUDIT_SERVER_ID.to_string(),
                server_name: "Crow".to_string(),
                action_kind: "plugin.settings".into(),
                target: p.name().to_string(),
                // Which keys changed, never their values.
                before_state: plain.iter().map(|(k, _)| k.as_str()).chain(secrets.iter().map(|(k, _, _)| k.as_str())).collect::<Vec<_>>().join(", "),
                after_state: None,
                blast_radius: None,
                outcome: "success".into(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }
        for (k, _, _) in &secrets {
            self.plugins.user.settings.saved_secrets.insert((id.to_string(), k.clone()), true);
        }
        // Fresh boxes next time, so a typed secret doesn't stay on screen.
        self.plugins.user.settings.inputs.retain(|(pid, _), _| pid != id);
        self.plugins.user.settings.picked.retain(|(pid, _), _| pid != id);
        self.plugins.user.settings.message = Some((true, t!("app-user-plugin-settings-saved", name = p.name())));
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::redact;

    #[test]
    fn secrets_are_cleared_from_output() {
        let s = BTreeMap::from([("a".to_string(), "hunter2".to_string()), ("b".to_string(), "hunter2-long".to_string()), ("c".to_string(), "x".to_string())]);
        assert_eq!(redact("pw=hunter2-long then hunter2; x stays", &s), "pw=••• then •••; x stays");
    }
}
