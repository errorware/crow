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
    /// What the active server's provider can do, when the server is linked
    /// to a configured provider account.
    pub fn active_provider_actions(&self) -> Option<crate::components::danger_zone::ProviderActions> {
        use crow_provider_core::capabilities::{INSTANCES_POWER, SNAPSHOTS};
        let srv = self.fleet.active_server()?;
        let account = self.providers.accounts.iter().find(|a| a.id == srv.provider_account && !srv.provider_instance.is_empty())?;
        let manifest = (crate::providers::factory(&account.plugin)?.manifest)();
        Some(crate::components::danger_zone::ProviderActions {
            name: manifest.display_name().to_string(),
            power: manifest.has_capability(INSTANCES_POWER),
            snapshots: manifest.has_capability(SNAPSHOTS),
        })
    }

    /// Power or snapshot at the provider, in the background, with a change
    /// record and a journal marker like host actions have.
    fn run_provider_action(&mut self, action: crate::providers::ProviderAction, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let Some(account) = self.providers.accounts.iter().find(|a| a.id == srv.provider_account).cloned() else { return };
        let settings = match (self.vault.key(), self.vault.db().lock()) {
            (Some(key), Ok(db)) => crate::providers::load_settings(&db, key, &account),
            _ => Err(self.vault.secrets_blocker().unwrap_or_else(|| "the vault is busy".into())),
        };
        let settings = match settings {
            Ok(s) => s,
            Err(e) => {
                self.danger.error = Some(e);
                cx.notify();
                return;
            }
        };
        let record_id = format!("chg_{}", chrono::Local::now().timestamp_micros());
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.insert_change_record(&crate::vault::ChangeRecord {
                id: record_id.clone(),
                server_id: srv.id.clone(),
                server_name: srv.name.clone(),
                action_kind: format!("provider.{}", action.verb()),
                target: format!("{}:{}", account.id, srv.provider_instance),
                before_state: format!("{} via {}", action.verb(), account.label),
                after_state: None,
                blast_radius: None,
                outcome: "pending".into(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }
        self.danger.last_result = Some(format!("asking {} to {}…", account.label, action.verb()));
        cx.notify();
        let (db, instance) = (self.vault.db(), srv.provider_instance.clone());
        cx.spawn(async move |entity, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let provider = crate::providers::connect(&account, settings).map_err(|e| e.to_string())?;
                    crate::providers::run_action(provider.as_ref(), &instance, action)
                })
                .await;
            if let Ok(db) = db.lock() {
                let (outcome, after) = match &result {
                    Ok(m) => ("success", m.clone()),
                    Err(e) => ("failed", e.clone()),
                };
                let _ = db.update_change_record_outcome(&record_id, outcome, Some(&after), &chrono::Utc::now().to_rfc3339());
            }
            let _ = entity.update(cx, |this, cx| {
                match result {
                    Ok(msg) => {
                        this.push_journal_action_marker(format!("crow: {msg}"));
                        this.danger.last_result = Some(msg);
                    }
                    Err(e) => {
                        this.danger.last_result = None;
                        this.danger.error = Some(e);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

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

        // Linked to a provider that can do it: the real thing, there.
        if let Some(pa) = crate::providers::ProviderAction::from_danger(&action) {
            let caps = self.active_provider_actions();
            let supported = caps.is_some_and(|c| if pa == crate::providers::ProviderAction::Snapshot { c.snapshots } else { c.power });
            if supported {
                self.run_provider_action(pa, cx);
                return;
            }
        }

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
