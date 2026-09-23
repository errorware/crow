use crate::app::SettingsSection;

/// Settings screen navigation and the inline dropdown/custom-value editor.
pub struct SettingsState {
    pub section: SettingsSection,
    /// Id of the setting whose dropdown is open.
    pub dropdown_open: Option<String>,
    pub custom_input: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self { section: SettingsSection::General, dropdown_open: None, custom_input: String::new() }
    }
}
