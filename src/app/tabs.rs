use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::components::titlebar::ServerTab;
use crate::theme::{CRIT, OK, TEXT_FAINTER, WARN};
use crate::vault::ServerRecord;

/// Tabs for the first few enrolled servers, colored by last known status.
pub fn initial_tabs(servers: &[ServerRecord]) -> Vec<ServerTab> {
    servers
        .iter()
        .take(5)
        .map(|s| ServerTab {
            id: s.id.clone(),
            name: s.name.clone(),
            status_color: match s.status.as_str() {
                "online" => OK,
                "warn" => WARN,
                "crit" => CRIT,
                _ => TEXT_FAINTER,
            },
            is_active: false,
        })
        .collect()
}

impl CrowApp {
    pub fn switch_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(srv) = self.fleet.servers.iter().find(|s| s.id == tab_id || s.name == tab_id).cloned() {
            if !self.fleet.tabs.iter().any(|t| t.id == srv.id || t.name == srv.name) {
                let status_color = match srv.status.as_str() {
                    "online" => OK,
                    "warn" => WARN,
                    "crit" => CRIT,
                    _ => TEXT_FAINTER,
                };
                self.fleet.tabs.push(ServerTab {
                    id: srv.id.clone(),
                    name: srv.name.clone(),
                    status_color,
                    is_active: true,
                });
            }
            self.fleet.active_tab_id = srv.id.clone();
            for tab in &mut self.fleet.tabs {
                tab.is_active = tab.id == self.fleet.active_tab_id;
            }
            self.screen = Screen::Server;
            if self.active_view == "overview" {
                self.refresh_overview_tables(cx);
            } else if super::is_table_page(&self.active_view) {
                let page = self.active_view.clone();
                self.set_services_tab(&page, cx);
            } else if self.active_view == "files" {
                self.files.current_path = "/".to_string();
                self.files.pending_delete = None;
                self.load_file_listing(cx);
            }
        } else {
            self.fleet.active_tab_id = tab_id.to_string();
            self.screen = Screen::Server;
        }
        self.reload_configs_for_active_server(cx);
        cx.notify();
    }

    pub fn close_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(pos) = self.fleet.tabs.iter().position(|t| t.id == tab_id || t.name == tab_id) {
            let removed = self.fleet.tabs.remove(pos);
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
}
