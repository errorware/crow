use crate::app::ClankerEditModalState;
use crate::vault::ClankerProviderConfig;

/// Clankers (AI provider) settings: stored providers, the edit modal and the
/// ELI5 log sandbox.
pub struct ClankersState {
    pub providers: Vec<ClankerProviderConfig>,
    pub editing: Option<ClankerEditModalState>,
    pub demo_log: String,
    /// The real answer (or error) for `demo_log`.
    pub demo_output: Option<Result<String, String>>,
    pub demo_loading: bool,
    /// Provider id → result of its last key check, and checks running.
    pub key_checks: std::collections::HashMap<String, Result<String, String>>,
    pub key_checking: std::collections::HashSet<String>,
    /// The edit dialog's LOAD MODELS result, and whether it's running.
    pub models: Option<Result<Vec<String>, String>>,
    pub models_loading: bool,
}

impl ClankersState {
    pub fn new(providers: Vec<ClankerProviderConfig>) -> Self {
        Self {
            providers,
            editing: None,
            demo_log: "kernel: [  129.412033] Out of memory: Kill process 28419 (mysqld) score 812 or sacrifice child".to_string(),
            demo_output: None,
            demo_loading: false,
            key_checks: Default::default(),
            key_checking: Default::default(),
            models: None,
            models_loading: false,
        }
    }
}
