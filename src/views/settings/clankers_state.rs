use crate::app::ClankerEditModalState;
use crate::vault::ClankerProviderConfig;

/// Clankers (AI provider) settings: stored providers, the edit modal and the
/// ELI5 log sandbox.
pub struct ClankersState {
    pub providers: Vec<ClankerProviderConfig>,
    pub editing: Option<ClankerEditModalState>,
    pub demo_log: String,
    pub demo_output: Option<String>,
}

impl ClankersState {
    pub fn new(providers: Vec<ClankerProviderConfig>) -> Self {
        Self {
            providers,
            editing: None,
            demo_log: "kernel: [  129.412033] Out of memory: Kill process 28419 (mysqld) score 812 or sacrifice child".to_string(),
            demo_output: None,
        }
    }
}
