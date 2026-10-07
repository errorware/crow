//! Server groups (ERR-97): kept in the vault, so a group can exist before
//! any server is in it. "default" (or empty) means no group.

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use crate::app::CrowApp;

/// The longest group name Crow keeps.
const MAX_GROUP_NAME: usize = 40;

/// A typed group name, tidied: trimmed, inner whitespace collapsed, capped.
/// `None` when nothing usable is left, or it's the reserved "default".
pub fn clean_group_name(raw: &str) -> Option<String> {
    let name: String = raw.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(MAX_GROUP_NAME).collect();
    (!name.is_empty() && !name.eq_ignore_ascii_case("default")).then_some(name)
}

/// Whether a server's `group_name` means "no group".
pub fn is_ungrouped(group: &str) -> bool {
    let g = group.trim();
    g.is_empty() || g == "default"
}

impl CrowApp {
    /// Toggles the group chips bar and the grouped Fleet list.
    pub fn toggle_fleet_group_bar(&mut self, cx: &mut Context<Self>) {
        self.fleet.group_bar_open = !self.fleet.group_bar_open;
        if !self.fleet.group_bar_open {
            self.fleet.group_filter = None;
            self.fleet.new_group_input = None;
        } else {
            self.reload_server_groups();
        }
        cx.notify();
    }

    /// Shows one group only (`None` for all).
    pub fn set_fleet_group_filter(&mut self, group: Option<String>, cx: &mut Context<Self>) {
        self.fleet.group_filter = group;
        cx.notify();
    }

    /// Opens the new-group box in the group bar (Enter creates, Esc cancels),
    /// or closes it.
    pub fn toggle_new_group_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.fleet.new_group_input.take().is_none() {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("group name (e.g. workers, edge, data)"));
            input.update(cx, |i, cx| i.focus(window, cx));
            cx.subscribe(&input, |this, _, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::PressEnter { .. }) {
                    this.create_new_group(cx);
                }
            })
            .detach();
            self.fleet.new_group_input = Some(input);
        }
        cx.notify();
    }

    /// Saves the typed group to the vault and shows it.
    pub fn create_new_group(&mut self, cx: &mut Context<Self>) {
        let Some(input) = self.fleet.new_group_input.as_ref() else { return };
        let Some(name) = clean_group_name(&input.read(cx).value()) else { return };
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.create_server_group(&name);
        }
        self.reload_server_groups();
        self.fleet.new_group_input = None;
        self.fleet.group_filter = Some(name);
        cx.notify();
    }

    /// Deletes an empty group.
    pub fn delete_server_group(&mut self, name: &str, cx: &mut Context<Self>) {
        if self.fleet.servers.iter().any(|s| s.group_name.trim() == name) {
            return;
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.delete_server_group(name);
        }
        if self.fleet.group_filter.as_deref() == Some(name) {
            self.fleet.group_filter = None;
        }
        self.reload_server_groups();
        cx.notify();
    }

    /// Moves a server to `group` ("default" for no group).
    pub fn assign_server_group(&mut self, server_id: &str, group: &str, cx: &mut Context<Self>) {
        let group = clean_group_name(group).unwrap_or_else(|| "default".to_string());
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.update_server_group(server_id, &group);
        }
        if let Some(s) = self.fleet.servers.iter_mut().find(|s| s.id == server_id) {
            s.group_name = group;
        }
        self.reload_server_groups();
        self.fleet.group_assign_modal_server = None;
        cx.notify();
    }

    /// Opens (or, with `None`, closes) the move-to-group dialog for a server.
    pub fn set_group_assign_target(&mut self, server_id: Option<String>, cx: &mut Context<Self>) {
        self.reload_server_groups();
        self.fleet.group_assign_modal_server = server_id;
        cx.notify();
    }

    /// Reloads the groups: the vault's, plus any a server is in.
    pub fn reload_server_groups(&mut self) {
        if let Ok(groups) = self.vault.db().lock().map_err(|_| ()).and_then(|db| db.list_server_groups().map_err(|_| ())) {
            self.fleet.groups = groups;
            return;
        }
        let set: std::collections::BTreeSet<String> = self.fleet.servers.iter().filter(|s| !is_ungrouped(&s.group_name)).map(|s| s.group_name.trim().to_string()).collect();
        self.fleet.groups = set.into_iter().collect();
    }
}

#[cfg(test)]
mod tests {
    use super::{clean_group_name, is_ungrouped, MAX_GROUP_NAME};

    #[test]
    fn group_names_are_tidied() {
        assert_eq!(clean_group_name("  edge   eu  "), Some("edge eu".to_string()));
        assert_eq!(clean_group_name("   "), None);
        assert_eq!(clean_group_name("Default"), None, "reserved for no group");
        assert_eq!(clean_group_name(&"x".repeat(60)).map(|n| n.len()), Some(MAX_GROUP_NAME));
        assert!(is_ungrouped("") && is_ungrouped("default") && !is_ungrouped("edge"));
    }
}
