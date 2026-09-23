use gpui_kit::Entity;
use gpui_kit::component::input::InputState;

/// Danger Zone state: the armed destructive action and its typed confirmation.
#[derive(Default)]
pub struct DangerZoneState {
    pub pending_action: Option<String>,
    pub confirm_input: Option<Entity<InputState>>,
    pub error: Option<String>,
    pub last_result: Option<String>,
}

impl DangerZoneState {
    pub fn disarm(&mut self) {
        self.pending_action = None;
        self.confirm_input = None;
        self.error = None;
    }
}
