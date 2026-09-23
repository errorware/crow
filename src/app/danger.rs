use gpui_kit::*;
use gpui_kit::component::input::InputState;

use super::CrowApp;
use crate::components::danger_zone::{
    danger_action_keyword,
    flush_firewall,
    kill_all_lab_containers,
    send_power_action,
};

// ==========================================
// Danger Zone — typed-confirmation destructive host actions
// ==========================================

impl CrowApp {
    pub fn arm_danger_zone_action(&mut self, action: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.danger.pending_action = Some(action.to_string());
        self.danger.error = None;
        self.danger.confirm_input = Some(cx.new(|cx| {
            InputState::new(window, cx).placeholder(danger_action_keyword(action))
        }));
        cx.notify();
    }

    pub fn cancel_danger_zone_action(&mut self, cx: &mut Context<Self>) {
        self.danger.disarm();
        cx.notify();
    }

    pub fn execute_danger_zone_action(&mut self, cx: &mut Context<Self>) {
        let Some(action) = self.danger.pending_action.clone() else {
            return;
        };
        let keyword = danger_action_keyword(&action);
        let typed = self.danger.confirm_input.as_ref()
            .map(|s| s.read(cx).value().trim().to_string())
            .unwrap_or_default();
        if typed != keyword {
            self.danger.error = Some(format!("Type {} exactly to confirm.", keyword));
            cx.notify();
            return;
        }

        let Some(srv) = self.fleet.active_server() else {
            self.danger.error = Some("No active server".to_string());
            cx.notify();
            return;
        };

        self.danger.disarm();

        let result = match action.as_str() {
            "poweroff" => send_power_action(&srv, "power-off"),
            "reboot" => send_power_action(&srv, "reboot"),
            "flush_firewall" => flush_firewall(&srv),
            "kill_containers" => {
                let (ok, failed) = kill_all_lab_containers(&self.local_lab.nodes);
                Ok(format!("Stopped {} lab container(s), {} failed", ok, failed))
            }
            _ => Err("Unknown action".to_string()),
        };

        match result {
            Ok(msg) => {
                self.push_journal_action_marker(format!("crow: {}", msg));
                self.danger.last_result = Some(msg);
            }
            Err(e) => self.danger.error = Some(e),
        }
        cx.notify();
    }
}
