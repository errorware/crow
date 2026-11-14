use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::components::titlebar::ServerTab;
use crate::vault::ServerRecord;

/// app_flags key: the open tabs' server ids, in order, comma separated.
pub const OPEN_TABS: &str = "fleet.open_tabs";

/// The tabs to start with: the ones open last time, in their order (those
/// still enrolled); the first few servers when nothing was saved yet.
pub fn initial_tabs(servers: &[ServerRecord], saved: Option<&str>) -> Vec<ServerTab> {
    let tab = |s: &ServerRecord| ServerTab { id: s.id.clone(), name: s.name.clone(), is_active: false };
    match saved {
        Some(ids) => ids.split(',').filter_map(|id| servers.iter().find(|s| s.id == id.trim())).map(tab).collect(),
        None => servers.iter().take(5).map(tab).collect(),
    }
}

impl CrowApp {
    /// Saves which tabs are open, for the next start.
    pub fn remember_tabs(&self) {
        let ids = self.fleet.tabs.iter().map(|t| t.id.as_str()).collect::<Vec<_>>().join(",");
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(OPEN_TABS, &ids);
        }
    }

    /// Hands the SSH transport the current servers (for jump hosts) and key
    /// file paths. Call after either list changes.
    pub fn sync_ssh_directory(&self) {
        crate::host::update_directory(&self.fleet.servers, &self.keys.enrolled);
    }

    pub fn switch_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(srv) = self.fleet.servers.iter().find(|s| s.id == tab_id || s.name == tab_id).cloned() {
            if !self.fleet.tabs.iter().any(|t| t.id == srv.id || t.name == srv.name) {
                self.fleet.tabs.push(ServerTab {
                    id: srv.id.clone(),
                    name: srv.name.clone(),
                    is_active: true,
                });
                self.remember_tabs();
            }
            // Remember where the tab we're leaving was (also when leaving
            // from the Fleet screen, which keeps the last server view).
            let leaving = std::mem::replace(&mut self.fleet.active_tab_id, srv.id.clone());
            if self.fleet.tabs.iter().any(|t| t.id == leaving) {
                self.tab_views.insert(leaving, self.active_view.clone());
            }
            for tab in &mut self.fleet.tabs {
                tab.is_active = tab.id == self.fleet.active_tab_id;
            }
            self.screen = Screen::Server;
            let view = self.tab_views.get(&srv.id).cloned().unwrap_or_else(|| "overview".to_string());
            if view == "files" {
                self.files.current_path = "/".to_string();
                self.files.pending_delete = None;
            }
            self.set_view(&view, cx);
        } else {
            self.fleet.active_tab_id = tab_id.to_string();
            self.screen = Screen::Server;
        }
        self.reload_configs_for_active_server(cx);
        self.refresh_firewall_for_active_server(cx);
        cx.notify();
    }

    pub fn close_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(pos) = self.fleet.tabs.iter().position(|t| t.id == tab_id || t.name == tab_id) {
            let removed = self.fleet.tabs.remove(pos);
            self.remember_tabs();
            self.tab_views.remove(&removed.id);
            let was_active = self.fleet.active_tab_id == tab_id || self.fleet.active_tab_id == removed.id || self.fleet.active_tab_id == removed.name;
            if was_active {
                if let Some(next_tab) = self.fleet.tabs.get(pos).or_else(|| self.fleet.tabs.last()) {
                    let next_id = next_tab.id.clone();
                    self.switch_tab(&next_id, cx);
                } else {
                    self.fleet.active_tab_id.clear();
                    self.screen = Screen::Fleet;
                }
            }
            cx.notify();
        }
    }

    /// Reorders a server tab from source_id to target_id and persists to SQLite.
    pub fn reorder_tab(&mut self, source_id: &str, target_id: &str, cx: &mut Context<Self>) {
        if reorder_tab_list(&mut self.fleet.tabs, source_id, target_id) {
            self.remember_tabs();
            cx.notify();
        }
    }
}

/// Reorders tabs in a list. Returns true if the order changed.
pub fn reorder_tab_list(tabs: &mut Vec<ServerTab>, source_id: &str, target_id: &str) -> bool {
    if source_id == target_id {
        return false;
    }
    let from_opt = tabs.iter().position(|t| t.id == source_id || t.name == source_id);
    let to_opt = tabs.iter().position(|t| t.id == target_id || t.name == target_id);
    if let (Some(from_idx), Some(to_idx)) = (from_opt, to_opt) {
        let tab = tabs.remove(from_idx);
        tabs.insert(to_idx, tab);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{initial_tabs, reorder_tab_list};
    use crate::components::titlebar::ServerTab;
    use crate::vault::ServerRecord;

    fn srv(id: &str) -> ServerRecord {
        ServerRecord { id: id.into(), name: format!("n-{id}"), ..Default::default() }
    }

    #[test]
    fn open_tabs_come_back_as_they_were_left() {
        let servers: Vec<ServerRecord> = ["a", "b", "c", "d", "e", "f"].iter().map(|i| srv(i)).collect();
        let ids = |t: Vec<ServerTab>| t.into_iter().map(|t| t.id).collect::<Vec<_>>();
        // Nothing saved yet: the first five.
        assert_eq!(ids(initial_tabs(&servers, None)), ["a", "b", "c", "d", "e"]);
        // Saved: those, in that order; a server since removed is dropped.
        assert_eq!(ids(initial_tabs(&servers, Some("f,gone,b"))), ["f", "b"]);
        // Every tab closed stays closed.
        assert!(initial_tabs(&servers, Some("")).is_empty());
    }

    #[test]
    fn reorder_tab_list_moves_and_clamps() {
        let mut tabs = vec![
            ServerTab { id: "a".into(), name: "A".into(), is_active: false },
            ServerTab { id: "b".into(), name: "B".into(), is_active: false },
            ServerTab { id: "c".into(), name: "C".into(), is_active: false },
        ];

        // Move "c" to "a" position -> [c, a, b]
        assert!(reorder_tab_list(&mut tabs, "c", "a"));
        let ids: Vec<_> = tabs.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["c", "a", "b"]);

        // Move "c" to "b" position -> [a, b, c]
        assert!(reorder_tab_list(&mut tabs, "c", "b"));
        let ids: Vec<_> = tabs.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);

        // Same source and target is no-op
        assert!(!reorder_tab_list(&mut tabs, "a", "a"));

        // Unknown tab is no-op
        assert!(!reorder_tab_list(&mut tabs, "nonexistent", "b"));
    }
}
