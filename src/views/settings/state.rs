use crate::app::SettingsSection;

/// Settings screen navigation and the inline dropdown/custom-value editor.
pub struct SettingsState {
    pub section: SettingsSection,
    /// Id of the setting whose dropdown is open.
    pub dropdown_open: Option<String>,
    pub custom_input: String,
    /// Why the last edit was refused (out of range, not a number, ...).
    pub edit_error: Option<String>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self { section: SettingsSection::General, dropdown_open: None, custom_input: String::new(), edit_error: None }
    }
}
