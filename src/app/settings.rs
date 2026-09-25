use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::{CrowApp, SettingsSection};

pub struct CustomSettingInput {
    pub row_id: String,
    pub input: Entity<InputState>,
    pub _events: Subscription,
}

impl CrowApp {
    pub fn set_settings_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.settings.section = section;
        cx.notify();
    }

    pub fn update_config_field(&mut self, row_id: &str, new_value: serde_json::Value, cx: &mut Context<Self>) {
        match self.config.update_field(row_id, new_value) {
            Ok(()) => self.settings.edit_error = None,
            Err(crow_config_core::edit::EditError::InvalidValue { message, .. }) => self.settings.edit_error = Some(format!("Not changed: {message}")),
            Err(e) => self.settings.edit_error = Some(format!("Not changed: {e}")),
        }
        // Personalisation is saved as soon as it changes; nothing to review.
        if row_id.starts_with("appearance.") {
            if let Err(e) = self.config.save() {
                eprintln!("Failed to save config: {:?}", e);
            }
        }
        cx.notify();
    }

    pub fn reset_config_field(&mut self, row_id: &str, cx: &mut Context<Self>) {
        if let Err(e) = self.config.reset_field(row_id) {
            eprintln!("Failed to reset config field {}: {:?}", row_id, e);
        }
        cx.notify();
    }

    pub fn reset_config_section(&mut self, sec_prefix: &str, cx: &mut Context<Self>) {
        if let Err(e) = self.config.reset_section(sec_prefix) {
            eprintln!("Failed to reset config section {}: {:?}", sec_prefix, e);
        }
        cx.notify();
    }

    pub fn save_config(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {:?}", e);
        }
        self.apply_settings();
        cx.notify();
    }

    /// Pushes saved settings to the parts of Crow that use them.
    pub fn apply_settings(&self) {
        let d = crate::host::ssh::SshSettings::default();
        crate::host::ssh::set_ssh_settings(crate::host::ssh::SshSettings {
            connect_timeout: self.config.saved_int("connection.connect_timeout").map_or(d.connect_timeout, |v| v.clamp(1, 300) as u32),
            keepalive_interval: self.config.saved_int("connection.keepalive_interval").map_or(d.keepalive_interval, |v| v.clamp(1, 3600) as u32),
            control_master: self.config.saved_bool("connection.control_master").unwrap_or(d.control_master),
        });
    }

    /// Seconds between live refreshes of the server on screen.
    pub fn refresh_interval_secs(&self) -> u64 {
        self.config.saved_int("general.refresh_interval").map_or(2, |v| v.clamp(1, 60) as u64)
    }

    pub fn open_config_file(&self) {
        let path = &self.config.path;
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(path).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/c", "start", ""]).arg(path).spawn();
    }

    pub fn toggle_settings_dropdown(&mut self, row_id: &str, initial_val: &str, cx: &mut Context<Self>) {
        if self.settings.dropdown_open.as_deref() == Some(row_id) {
            self.settings.dropdown_open = None;
            self.settings.custom_input.clear();
            self.custom_setting_input = None;
        } else {
            self.settings.dropdown_open = Some(row_id.to_string());
            self.settings.custom_input = initial_val.to_string();
            self.custom_setting_input = None;
        }
        cx.notify();
    }

    pub fn close_settings_dropdown(&mut self, cx: &mut Context<Self>) {
        self.settings.dropdown_open = None;
        self.settings.custom_input.clear();
        self.custom_setting_input = None;
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_settings_custom_input(&mut self, val: String, cx: &mut Context<Self>) {
        self.settings.custom_input = val;
        cx.notify();
    }

    pub fn apply_settings_custom_input(&mut self, row_id: &str, cx: &mut Context<Self>) {
        let trimmed = self.settings.custom_input.trim();
        let field_is_int = self
            .config
            .get_field(row_id)
            .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Integer))
            .unwrap_or(false);

        if field_is_int {
            if let Ok(n) = trimmed.parse::<i64>() {
                self.update_config_field(row_id, serde_json::Value::Number(serde_json::Number::from(n)), cx);
                self.settings.dropdown_open = None;
                self.settings.custom_input.clear();
                self.custom_setting_input = None;
            }
        } else {
            self.update_config_field(row_id, serde_json::Value::String(trimmed.to_string()), cx);
            self.settings.dropdown_open = None;
            self.settings.custom_input.clear();
            self.custom_setting_input = None;
        }
        cx.notify();
    }

    pub fn ensure_settings_custom_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row_id) = self.settings.dropdown_open.clone() else {
            self.custom_setting_input = None;
            return;
        };
        if self.custom_setting_input.as_ref().is_some_and(|s| s.row_id == row_id) {
            return;
        }
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("value…")
                .default_value(&self.settings.custom_input)
        });
        let rid = row_id.clone();
        let events = cx.subscribe(&input, move |this, input, ev: &InputEvent, cx| match ev {
            InputEvent::Change => {
                this.settings.custom_input = input.read(cx).value().to_string();
                let field_is_int = this
                    .config
                    .get_field(&rid)
                    .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Integer))
                    .unwrap_or(false);
                if field_is_int {
                    this.settings.custom_input.retain(|c| c.is_ascii_digit());
                }
                cx.notify();
            }
            InputEvent::PressEnter { .. } => {
                this.apply_settings_custom_input(&rid, cx);
            }
            _ => {}
        });
        self.custom_setting_input = Some(CustomSettingInput {
            row_id,
            input,
            _events: events,
        });
    }
}
