use gpui_kit::*;

use super::CrowApp;
use crate::theme::OK;
use crate::app::Screen;
use crate::theme::CRIT;
use crate::theme::WARN;
use crate::theme::TEXT_FAINTER;
use crate::components::titlebar::ServerTab;
use crate::views::overview::collector::collect_sockets_for_server;
use crate::views::overview::collector::collect_services_for_server;
use crate::views::overview::collector::collect_processes_for_server;

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
                let tab_owned = self.overview.active_tab.clone();
                let srv_clone = srv.clone();
                cx.spawn(async move |entity, cx| {
                    match tab_owned.as_str() {
                        "processes" => {
                            let procs = cx.background_executor().spawn(async move {
                                collect_processes_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.overview.processes = procs;
                                cx.notify();
                            });
                        }
                        "sockets" => {
                            let socks = cx.background_executor().spawn(async move {
                                collect_sockets_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.overview.sockets = socks;
                                cx.notify();
                            });
                        }
                        _ => {
                            let svcs = cx.background_executor().spawn(async move {
                                collect_services_for_server(&srv_clone)
                            }).await;
                            let _ = entity.update(cx, |this, cx| {
                                this.overview.services = svcs;
                                cx.notify();
                            });
                        }
                    }
                }).detach();
            } else if self.active_view == "files" {
                self.files.current_path = "/".to_string();
                self.files.pending_delete = None;
                self.load_file_listing(cx);
            }
        } else {
            self.fleet.active_tab_id = tab_id.to_string();
            self.screen = Screen::Server;
        }
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
