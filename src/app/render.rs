use gpui_kit::*;

use super::CrowApp;
use crate::theme::*;
use crate::app::Screen;
use crate::vault::VaultStatus;
use crate::views::config::cron_editor;
use crate::views::onboard::onboard_view;
use crate::views::lock::vault_lock_view;
use crate::components::sidebar::sidebar;
use crate::views::lock::vault_setup_view;
use crate::views::fleet::fleet_setup_view;
use crate::views::settings::settings_view;
use crate::views::firewall::firewall_view;
use crate::components::titlebar::titlebar;
use crate::views::logs::logs_explorer_view;
use crate::views::files::file_browser_view;
use crate::views::config::raw_config_editor;
use crate::views::fleet::fleet_overview_view;
use crate::views::users::user_management_view;
use crate::components::stat_strip::stat_strip;
use crate::views::overview::log_tail::log_tail;
use crate::components::danger_zone::danger_zone;
use crate::components::palette::palette_overlay;
use crate::components::identity_bar::identity_bar;
use crate::components::titlebar::burger_menu_overlay;
use crate::views::config::rules_editor::rules_editor;
use crate::views::overview::log_tail::socket_log_drawer;
use crate::views::overview::services_table::services_table;
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::overview::service_inspector::service_inspector_rail;

impl Render for CrowApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_overview = self.active_view == "overview";
        let is_config = self.active_view == "config";
        let palette_open = self.palette_open;
        let menu_open = self.menu_open;
        let screen = self.screen;
        let app_view = cx.entity();
        let vault_status = self.vault.status();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                this.handle_key_down(ev, window, cx);
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _ev: &MouseUpEvent, _window, cx| {
                if this.input_drag_anchor.is_some() {
                    this.input_drag_anchor = None;
                    cx.notify();
                }
            }))
            .size_full()
            .bg(BG_WINDOW)
            .text_color(TEXT_PRIMARY)
            .font_family(FONT_MONO)
            .text_size(px(12.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .border_1()
            .border_color(BORDER_DEFAULT)
            .relative()
            // If locked, show full lock screen
            .children(if vault_status == VaultStatus::Locked {
                Some(
                    div().size_full().child(vault_lock_view(app_view.clone(), self))
                )
            } else {
                None
            })
            // If not locked (Disabled or Unlocked), show normal app
            .children(if vault_status != VaultStatus::Locked {
                Some(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        // 1. Frameless Titlebar
                        .child(titlebar(
                            &self.tabs,
                            &self.active_tab_id,
                            self.screen,
                            self.menu_open,
                            self.servers.len(),
                            self.servers.iter().filter(|s| s.agent_installed).count(),
                            app_view.clone(),
                        ))
                        // 2. Main Screen Area
                        .child(
                            div()
                                .flex_1()
                                .min_h(px(0.0))
                                .flex()
                                .flex_col()
                                .w_full()
                                .children(match screen {
                                    Screen::Server => {
                                        let active_srv = self.servers.iter().find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id);
                                        let active_mtr = self.metrics_store.get(&self.active_tab_id).or_else(|| active_srv.and_then(|s| self.metrics_store.get(&s.id)));
                                        if self.servers.is_empty() || active_srv.is_none() {
                                            let app_fleet = app_view.clone();
                                            let app_add = app_view.clone();
                                            Some(
                                                div()
                                                    .size_full()
                                                    .bg(BG_APP)
                                                    .flex()
                                                    .flex_col()
                                                    .items_center()
                                                    .justify_center()
                                                    .gap(px(16.0))
                                                    .p(px(32.0))
                                                    .child(
                                                        div()
                                                            .size(px(48.0))
                                                            .border_1()
                                                            .border_color(BORDER_STRONG)
                                                            .bg(BG_PANEL)
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                crate::components::icons::tabler_icon(crate::components::icons::TablerIcon::Server)
                                                                    .size(px(24.0))
                                                                    .text_color(TEXT_MUTED),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_size(px(13.0))
                                                            .text_color(TEXT_MAX)
                                                            .child("NO ACTIVE SERVER SELECTED"),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .text_color(TEXT_DIM)
                                                            .child("Select a server tab or enroll a new Linux host to inspect services and config."),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(12.0))
                                                            .mt(px(8.0))
                                                            .child(
                                                                div()
                                                                    .id("empty-server-goto-fleet")
                                                                    .px(px(14.0))
                                                                    .py(px(7.0))
                                                                    .bg(BG_PANEL)
                                                                    .border_1()
                                                                    .border_color(BORDER_STRONG)
                                                                    .text_color(TEXT_PRIMARY)
                                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                                    .cursor_pointer()
                                                                    .font_family(FONT_MONO)
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_size(px(11.0))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        app_fleet.update(cx, |this, cx| {
                                                                            this.set_screen(Screen::Fleet, cx);
                                                                        });
                                                                    })
                                                                    .child("⬢ GO TO FLEET OVERVIEW"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("empty-server-enroll-btn")
                                                                    .px(px(14.0))
                                                                    .py(px(7.0))
                                                                    .bg(OK_BG)
                                                                    .border_1()
                                                                    .border_color(OK)
                                                                    .text_color(OK)
                                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                                    .cursor_pointer()
                                                                    .font_family(FONT_MONO)
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_size(px(11.0))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        app_add.update(cx, |this, cx| {
                                                                            this.set_screen(Screen::Onboard, cx);
                                                                        });
                                                                    })
                                                                    .child("+ ENROLL NEW SERVER"),
                                                            ),
                                                    ),
                                            )
                                        } else {
                                            Some(
                                                div()
                                                    .size_full()
                                                    .flex()
                                                    .flex_col()
                                                    // Server Identity Bar
                                                    .child(identity_bar(active_srv, app_view.clone()))
                                                    // Server Stat Strip
                                                    .child(stat_strip(active_mtr, self.metrics_lag_secs, self.active_surge_alert.as_ref()))
                                                // Main Server Body: Sidebar + Content
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_h(px(0.0))
                                                        .flex()
                                                        .w_full()
                                                        // Sidebar
                                                        .child(sidebar(&self.active_view, self.sidebar_collapsed, app_view.clone()))
                                                        // Content Area (Overview or Config or other server view)
                                                        .child(
                                                            div()
                                                                .flex_1()
                                                                .min_w(px(0.0))
                                                                .h_full()
                                                            .flex()
                                                            .children(if is_overview {
                                                                Some(
                                                                    if self.overview.active_tab == "sockets" {
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .flex_col()
                                                                            .child(
                                                                                div()
                                                                                    .flex_1()
                                                                                    .min_h(px(0.0))
                                                                                    .child(services_table(self, app_view.clone()))
                                                                            )
                                                                            .children(if self.overview.socket_drawer_open {
                                                                                Some(socket_log_drawer(self, app_view.clone()).into_any_element())
                                                                            } else {
                                                                                None
                                                                            })
                                                                    } else {
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .child(services_table(self, app_view.clone()))
                                                                            .child(if self.overview.active_tab == "services" {
                                                                                service_inspector_rail(self, app_view.clone()).into_any_element()
                                                                            } else {
                                                                                log_tail(&self.journal.entries, app_view.clone()).into_any_element()
                                                                            })
                                                                    }
                                                                )
                                                            } else if is_config {
                                                                let editor_view = if self.configs.selected_file == "journald.conf" {
                                                                    crate::views::config::journald_editor::journald_editor(
                                                                        &self.journal.retention,
                                                                        &self.journal.telemetry,
                                                                        self,
                                                                        app_view.clone(),
                                                                    ).into_any_element()
                                                                } else if self.configs.selected_file == "pg_hba.conf" {
                                                                    rules_editor(&self.configs.hba_rules, self, app_view.clone()).into_any_element()
                                                                } else if self.configs.selected_file == "crontab" || self.configs.selected_file.contains("cron") {
                                                                    cron_editor(&self.configs.cron_jobs, self, app_view.clone()).into_any_element()
                                                                } else if let Some(st) = self.configs.states.get(&self.configs.selected_file) {
                                                                    raw_config_editor(st, app_view.clone()).into_any_element()
                                                                } else {
                                                                    rules_editor(&self.configs.hba_rules, self, app_view.clone()).into_any_element()
                                                                };

                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(managed_files_rail(&self.configs.selected_file, self, app_view.clone()))
                                                                        .child(editor_view)
                                                                        .child(pending_diff_rail(self, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "logs" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(logs_explorer_view(app_view.clone(), self))
                                                                )
                                                            } else if self.active_view == "cron" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(cron_editor(&self.configs.cron_jobs, self, app_view.clone()))
                                                                        .child(pending_diff_rail(self, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "users" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(user_management_view(app_view.clone(), &self.users, &self.keys.enrolled))
                                                                )
                                                            } else if self.active_view == "files" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(file_browser_view(app_view.clone(), &self.files))
                                                                )
                                                            } else if self.active_view == "firewall" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(
                                                                            div()
                                                                                .flex_1()
                                                                                .min_w(px(0.0))
                                                                                .h_full()
                                                                                .child(firewall_view(app_view.clone(), &self.firewall, &self.configs.states))
                                                                        )
                                                                        .children(if self.firewall.show_audit_rail {
                                                                            Some(pending_diff_rail(self, app_view.clone()))
                                                                        } else {
                                                                            None
                                                                        })
                                                                )
                                                            } else {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .flex_col()
                                                                        .bg(BG_APP)
                                                                        .child(
                                                                            div()
                                                                                .h(px(34.0))
                                                                                .flex_none()
                                                                                .flex()
                                                                                .items_center()
                                                                                .px(px(12.0))
                                                                                .bg(BG_PANEL)
                                                                                .border_b_1()
                                                                                .border_color(BORDER_PANEL)
                                                                                .child(
                                                                                    div()
                                                                                        .font_family(FONT_MONO)
                                                                                        .text_size(px(11.0))
                                                                                        .font_weight(FontWeight::SEMIBOLD)
                                                                                        .text_color(TEXT_PRIMARY)
                                                                                        .child(self.active_view.to_uppercase()),
                                                                                ),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .flex_1()
                                                                                .flex()
                                                                                .items_center()
                                                                                .justify_center()
                                                                                .gap(px(8.0))
                                                                                .font_family(FONT_MONO)
                                                                                .text_size(px(11.5))
                                                                                .text_color(TEXT_FAINT)
                                                                                .child(div().size(px(6.0)).rounded_full().bg(OK))
                                                                                .child(format!("{} · agent discovery stream pending", self.active_view)),
                                                                        ),
                                                                )
                                                            })
                                                    ),
                                            )
                                            // Persistent Danger Zone Strip
                                            .child(danger_zone(&self.danger, app_view.clone())),
                                        )
                                    }
                                },
                                    Screen::Fleet => Some(
                                        div()
                                            .size_full()
                                            .child(fleet_overview_view(app_view.clone(), self)),
                                    ),
                                    Screen::Settings => Some(
                                        div()
                                            .size_full()
                                            .child(settings_view(
                                                app_view.clone(),
                                                self,
                                                self.settings_section,
                                            )),
                                    ),
                                    Screen::Onboard => Some(
                                        div()
                                            .size_full()
                                            .child(onboard_view(app_view.clone(), self)),
                                    ),
                                    Screen::FleetSetup => Some(
                                        div()
                                            .size_full()
                                            .child(fleet_setup_view(app_view.clone())),
                                    ),
                                    Screen::VaultSetup => Some(
                                        div()
                                            .size_full()
                                            .child(vault_setup_view(app_view.clone(), self)),
                                    ),
                                }),
                        )
                        // 3. Burger Menu Overlay
                        .children(if menu_open {
                            let active_name = self.servers.iter()
                                .find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id)
                                .map(|s| s.name.clone());
                            Some(burger_menu_overlay(app_view.clone(), self.screen, &self.active_view, active_name))
                        } else {
                            None
                        })
                        // 4. Command Palette Overlay (⌘K)
                        .children(if palette_open {
                            let scope = self.servers.iter()
                                .find(|s| s.id == self.active_tab_id || s.name == self.active_tab_id)
                                .map(|s| s.name.as_str())
                                .unwrap_or("Fleet");
                            Some(palette_overlay(app_view.clone(), scope))
                        } else {
                            None
                        })
                        // 5. About Crow Modal
                        .children(if self.show_about_modal {
                            Some(crate::components::about::about_modal(app_view.clone(), self.about_copied_toast))
                        } else {
                            None
                        })
                )
            } else {
                None
            })
    }
}
