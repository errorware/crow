use gpui_kit::*;

use super::{CrowApp, SettingsSection};

impl CrowApp {
    pub fn set_settings_section(&mut self, section: SettingsSection, cx: &mut Context<Self>) {
        self.settings.section = section;
        cx.notify();
    }

    pub fn update_config_field(&mut self, row_id: &str, new_value: serde_json::Value, cx: &mut Context<Self>) {
        if let Err(e) = self.config.update_field(row_id, new_value) {
            eprintln!("Failed to update config field {}: {:?}", row_id, e);
        }
        // Personalisation is saved as soon as it changes; nothing to review.
        if row_id.starts_with("appearance.") {
            if let Err(e) = self.config.save() {
                eprintln!("Failed to save config: {:?}", e);
            }
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
        cx.notify();
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
        } else {
            self.settings.dropdown_open = Some(row_id.to_string());
            self.settings.custom_input = initial_val.to_string();
            self.caret.place(self.settings.custom_input.chars().count());
            self.caret.blink = true;
        }
        cx.notify();
    }

    pub fn close_settings_dropdown(&mut self, cx: &mut Context<Self>) {
        self.settings.dropdown_open = None;
        self.settings.custom_input.clear();
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
            .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Other(cow) if cow == "integer"))
            .unwrap_or(false);

        if field_is_int {
            if let Ok(n) = trimmed.parse::<i64>() {
                self.update_config_field(row_id, serde_json::Value::Number(serde_json::Number::from(n)), cx);
                self.settings.dropdown_open = None;
                self.settings.custom_input.clear();
            }
        } else {
            self.update_config_field(row_id, serde_json::Value::String(trimmed.to_string()), cx);
            self.settings.dropdown_open = None;
            self.settings.custom_input.clear();
        }
        cx.notify();
    }
}
