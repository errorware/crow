use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::components::danger_zone::danger_zone;
use crate::components::identity_bar::{connection_banner, identity_bar};
use crate::components::window_frame::window_frame;
use crate::components::palette::palette_overlay;
use crate::components::sidebar::sidebar;
use crate::components::stat_strip::stat_strip;
use crate::components::titlebar::{burger_menu_overlay, titlebar};
use crate::theme::*;
use crate::vault::VaultStatus;
use crate::views::config::cron_editor;
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::files::file_browser_view;
use crate::views::firewall::firewall_view;
use crate::views::fleet::{fleet_overview_view, fleet_setup_view};
use crate::views::lock::{vault_lock_view, vault_setup_view};
use crate::views::logs::logs_explorer_view;
use crate::views::onboard::onboard_view;
use crate::views::overview::dashboard::overview_dashboard;
use crate::views::overview::log_tail::{log_tail, socket_log_drawer};
use crate::views::overview::service_inspector::service_inspector_rail;
use crate::views::overview::services_table::services_table;
use crate::views::settings::settings_view;
use crate::views::users::user_management_view;

impl Render for CrowApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.screen == Screen::Onboard {
            self.ensure_onboard_inputs(window, cx);
        }
        if self.screen == Screen::Fleet || (self.screen == Screen::Settings && self.settings.section == super::SettingsSection::Personalisation) {
            self.ensure_fleet_background(cx);
        }
        if self.users.show_new_user_modal {
            self.ensure_new_user_inputs(window, cx);
        }
        if self.screen == Screen::Settings && self.settings.section == super::SettingsSection::Providers && self.providers.editing.is_some() {
            self.ensure_provider_inputs(window, cx);
        }
        if self.screen == Screen::Settings && self.settings.section == super::SettingsSection::Security && self.vault_form.open.is_some() {
            self.ensure_vault_form_inputs(window, cx);
        }
        if self.screen == Screen::Settings && self.clankers.editing.is_some() {
            self.ensure_clanker_inputs(window, cx);
        }
        if self.vault.status() == crate::vault::VaultStatus::Locked {
            self.ensure_lock_inputs(window, cx);
        }
        if self.screen == Screen::VaultSetup && self.setup_state.step == crate::views::lock::SetupStep::ConfigureCredentials {
            self.ensure_setup_inputs(window, cx);
        }
        if self.users.password_for.is_some() && self.active_view == "users" {
            self.ensure_password_inputs(window, cx);
        }
        if self.screen == Screen::Server {
            if let Some(page) = super::overview::TablePage::for_view(&self.active_view) {
                self.ensure_table_search(page, window, cx);
            }
        }
        self.ensure_key_inputs(window, cx);
        self.ensure_settings_custom_input(window, cx);
        self.ensure_config_search(window, cx);

        // The Fleet screen's Archived tab needs the purge window and Crow's
        // own purge audit trail (ERR-32).
        let fleet_purge_days = self.archive_purge_days();
        let fleet_purge_audit = self.recent_purge_audit();
        let (alert_lines, watch_gap) = if self.screen == Screen::Fleet { self.fleet_alert_panel() } else { (Vec::new(), None) };
        let audit = match self.screen {
            Screen::Fleet => self.audit_items(30),
            Screen::Audit => self.audit_items(1000),
            _ => Vec::new(),
        };
        let archive_purge_due = crate::app::archive::purge_due_text(fleet_purge_days);
        let is_overview = self.active_view == "overview";
        let is_table_page = super::is_table_page(&self.active_view);
        let is_config = self.active_view == "config";
        let palette_open = self.palette_open;
        let menu_open = self.menu_open;
        let screen = self.screen;
        let app_view = cx.entity();
        let vault_status = self.vault.status();
        let stance = self.stance_report();

        let app_root = div()
            .track_focus(&self.focus_handle)
            .capture_key_down(cx.listener(|this, _ev: &KeyDownEvent, _window, _cx| {
                this.last_activity = std::time::Instant::now();
            }))
            .capture_any_mouse_down(cx.listener(|this, _ev: &MouseDownEvent, _window, _cx| {
                this.last_activity = std::time::Instant::now();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                this.handle_key_down(ev, window, cx);
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
                    div().size_full().child(vault_lock_view(app_view.clone(), &self.lock_state, self.lock_inputs.as_ref()))
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
                            &self.fleet.tabs_with_health(),
                            &self.fleet.active_tab_id,
                            self.screen,
                            self.menu_open,
                            self.fleet.servers.len(),
                            self.session_label(),
                            &stance,
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
                                        let active_srv = self.fleet.servers.iter().find(|s| s.id == self.fleet.active_tab_id || s.name == self.fleet.active_tab_id);
                                        // Its jump host, and whether that's down (ERR-91).
                                        let jump = active_srv
                                            .and_then(|s| s.jump_host_id.as_ref())
                                            .and_then(|id| self.fleet.servers.iter().find(|j| &j.id == id))
                                            .map(|j| (j.name.clone(), matches!(self.fleet.health(j), crate::views::fleet::state::FleetHealth::Down { .. })));
                                        let active_mtr = self.fleet.metrics_store.get(&self.fleet.active_tab_id).or_else(|| active_srv.and_then(|s| self.fleet.metrics_store.get(&s.id)));
                                        if self.fleet.servers.is_empty() || active_srv.is_none() {
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
                                                    .child(identity_bar(active_srv, jump.clone(), self.region_picker_open, app_view.clone()))
                                                    .children(connection_banner(active_srv, jump.clone(), app_view.clone()))
                                                    // Server Stat Strip
                                                    .child(stat_strip(active_mtr, self.fleet.metrics_lag_secs, self.fleet.active_surge_alert.as_ref()))
                                                // Main Server Body: Sidebar + Content
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_h(px(0.0))
                                                        .flex()
                                                        .w_full()
                                                        // Sidebar
                                                        .child(sidebar(&self.active_view, self.sidebar_collapsed, &self.nav_badges(), app_view.clone()))
                                                        // Content Area (Overview or Config or other server view)
                                                        .child(
                                                            div()
                                                                .flex_1()
                                                                .min_w(px(0.0))
                                                                .h_full()
                                                            .flex()
                                                            .children(if is_overview {
                                                                Some(div().size_full().flex().child(overview_dashboard(&self.overview, app_view.clone())))
                                                            } else if is_table_page {
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
                                                                                    .child(services_table(&self.overview, match self.active_view.as_str() {
                                                                        "processes" => self.processes_search.as_ref().map(|s| &s.input),
                                                                        _ => self.services_search.as_ref().map(|s| &s.input),
                                                                    }, Some(&self.firewall.status), app_view.clone()))
                                                                            )
                                                                            .children(if self.overview.socket_drawer_open {
                                                                                Some(socket_log_drawer(&self.fleet, &self.overview, &self.journal, app_view.clone()).into_any_element())
                                                                            } else {
                                                                                None
                                                                            })
                                                                    } else if self.overview.active_tab == "processes" {
                                                                        // Table above, live log panel below (drag the divider).
                                                                        let app_resize = app_view.clone();
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .flex_col()
                                                                            .on_drag_move::<crate::components::resize::BottomPanelResize>(move |ev, _window, cx| {
                                                                                let height = crate::components::resize::bottom_panel_height(ev, 120.0, 180.0);
                                                                                app_resize.update(cx, |this, cx| {
                                                                                    this.overview.process_log_height = height;
                                                                                    cx.notify();
                                                                                });
                                                                            })
                                                                            .child(div().flex_1().min_h(px(0.0)).flex().child(services_table(&self.overview, self.processes_search.as_ref().map(|s| &s.input), Some(&self.firewall.status), app_view.clone())))
                                                                            .child(crate::components::resize::resize_handle("process-log-resize"))
                                                                            .child(div().h(px(self.overview.process_log_height)).flex_none().child(log_tail(&self.journal.entries, app_view.clone())))
                                                                    } else {
                                                                        div()
                                                                            .size_full()
                                                                            .flex()
                                                                            .child(services_table(&self.overview, self.services_search.as_ref().map(|s| &s.input), Some(&self.firewall.status), app_view.clone()))
                                                                            .child(service_inspector_rail(&self.vault, &self.fleet, &self.overview, app_view.clone()).into_any_element())
                                                                    }
                                                                )
                                                            } else if is_config {
                                                                let editor_view = self.config_editor_view(window, cx);

                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(managed_files_rail(&self.configs.selected_file, self.config_search.as_ref().map(|s| &s.input), &self.fleet, &self.configs, app_view.clone()))
                                                                        .child(editor_view)
                                                                        .child(pending_diff_rail(&self.configs, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "logs" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(logs_explorer_view(app_view.clone(), self.logs_search.as_ref().map(|s| &s.input), &self.journal, self.ai_provider_name()))
                                                                )
                                                            } else if self.active_view == "history" {
                                                                let hover = self.history_hover;
                                                                self.history_cache().map(|c| {
                                                                    div()
                                                                        .size_full()
                                                                        .on_mouse_move({
                                                                            // Leaving the charts clears the crosshair.
                                                                            let app = app_view.clone();
                                                                            move |_ev, _window, cx| app.update(cx, |this, cx| this.set_history_hover(None, cx))
                                                                        })
                                                                        .child(crate::views::history::history_view(c, hover, app_view.clone()))
                                                                })
                                                            } else if self.active_view == "cron" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .flex()
                                                                        .child(cron_editor(&self.configs.cron_jobs, &self.configs, app_view.clone()))
                                                                        .child(pending_diff_rail(&self.configs, app_view.clone()))
                                                                )
                                                            } else if self.active_view == "users" {
                                                                Some(
                                                                    div()
                                                                        .size_full()
                                                                        .child(user_management_view(app_view.clone(), &self.users, &self.keys.enrolled, self.users_search.as_ref().map(|s| &s.input), self.new_user_inputs.as_ref(), self.password_inputs.as_ref()))
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
                                                                            Some(pending_diff_rail(&self.configs, app_view.clone()))
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
                                                                                .child(format!("{} · not built yet", self.active_view)),
                                                                        ),
                                                                )
                                                            })
                                                    ),
                                            )
                                            // Persistent Danger Zone Strip
                                            .child(danger_zone(&self.danger, self.active_provider_actions().as_ref(), app_view.clone()))
                                            // Archive confirmation for this server (ERR-32).
                                            .children(self.fleet.pending_archive.as_ref().and_then(|id| self.fleet.servers.iter().find(|s| &s.id == id)).map(|srv| {
                                                crate::views::fleet::archived::archive_confirm_overlay(srv, &archive_purge_due, app_view.clone())
                                            })),
                                        )
                                    }
                                },
                                    Screen::Fleet => Some(
                                        div()
                                            .size_full()
                                            .relative()
                                            .child(fleet_overview_view(app_view.clone(), &self.fleet, &self.local_lab, self.fleet_background.rendered.clone().filter(|_| self.fleet_background_source().is_some()).map(|p| (p, self.fleet_background_opacity())), fleet_purge_days, &fleet_purge_audit, &audit, &alert_lines, watch_gap))

                                    ),
                                    Screen::Settings => Some(
                                        div()
                                            .size_full()
                                            .child(settings_view(
                                                app_view.clone(),
                                                &self.vault,
                                                &self.config,
                                                self.key_inputs.as_ref(),
                                                &self.fleet,
                                                &self.keys,
                                                &self.clankers,
                                                &self.settings,
                                                &self.lab_state,
                                                self.settings.section,
                                                (&self.fleet_background, self.fleet_background_source()),
                                                &self.providers,
                                                self.provider_inputs.as_ref(),
                                                self.secrets_notice.as_deref(),
                                                (&self.vault_form, self.vault_form_inputs.as_ref()),
                                                self.clanker_inputs.as_ref(),
                                                self.custom_setting_input.as_ref().map(|s| &s.input),
                                            )),
                                    ),
                                    Screen::Onboard => Some(
                                        div()
                                            .size_full()
                                            .child(onboard_view(app_view.clone(), self.onboard_inputs.as_ref(), &self.fleet, &self.onboard_state, &self.keys, &self.local_lab)),
                                    ),
                                    Screen::Audit => Some(
                                        div()
                                            .size_full()
                                            .child(crate::views::audit::audit_view(&audit, &self.audit_filter, app_view.clone())),
                                    ),
                                    Screen::FleetSetup => Some(
                                        div()
                                            .size_full()
                                            .child(fleet_setup_view(&self.fleet, app_view.clone())),
                                    ),
                                    Screen::VaultSetup => Some(
                                        div()
                                            .size_full()
                                            .child(vault_setup_view(app_view.clone(), &self.setup_state, self.setup_inputs.as_ref())),
                                    ),
                                }),
                        )
                        // 3. Burger Menu Overlay
                        .children(if menu_open {
                            let active_name = self.fleet.servers.iter()
                                .find(|s| s.id == self.fleet.active_tab_id || s.name == self.fleet.active_tab_id)
                                .map(|s| s.name.clone());
                            Some(burger_menu_overlay(app_view.clone(), self.screen, &self.active_view, active_name))
                        } else {
                            None
                        })
                        // 4. Command Palette Overlay (⌘K)
                        .children(if palette_open {
                            let scope = self.fleet.servers.iter()
                                .find(|s| s.id == self.fleet.active_tab_id || s.name == self.fleet.active_tab_id)
                                .map(|s| s.name.as_str())
                                .unwrap_or("Fleet");
                            Some(palette_overlay(app_view.clone(), scope))
                        } else {
                            None
                        })
                        // Danger Zone → SNAPSHOTS (ERR-47)
                        .children(self.snapshots_panel.as_ref().map(|p| crate::components::snapshots_panel::snapshots_panel(p, app_view.clone())))
                        .children(self.fleet_runner.as_ref().map(|r| crate::views::fleet::run_panel::fleet_run_panel(r, app_view.clone())))
                        .children(self.recovery.as_ref().map(|p| {
                            let name = self.fleet.servers.iter().find(|s| s.id == p.server_id).map(|s| s.name.clone()).unwrap_or_default();
                            crate::components::recovery_panel::recovery_panel(p, &name, app_view.clone())
                        }))
                        // Turn off SSH password login (ERR-34)
                        .children(self.password_login.as_ref().map(|f| crate::components::password_login::password_login_dialog(f, app_view.clone())))
                        // Snapshot first? before a lockout-risk change (ERR-48)
                        .children(self.snapshot_offer.as_ref().map(|o| crate::components::snapshot_offer::snapshot_offer(o, app_view.clone())))
                        // Fleet → IMPORT FROM PROVIDERS (ERR-46)
                        .children(self.import.open.then(|| crate::views::fleet::import::import_panel(&self.import, app_view.clone())))
                        // Security stance panel (titlebar badge, ERR-60)
                        .children(self.stance_panel_open.then(|| crate::components::stance::stance_panel(&stance, app_view.clone())))
                        // First secret while Open: choose a stance on purpose (ERR-60)
                        .children(self.pending_secret.is_some().then(|| crate::components::stance::stance_choice_modal(self.stance_choice_ack, app_view.clone())))
                        // 5. About Crow Modal
                        .children(if self.show_about_modal {
                            Some(crate::components::about::about_modal(app_view.clone(), self.about_copied_toast))
                        } else {
                            None
                        })
                )
            } else {
                None
            });
        window_frame(window, app_root)
    }
}

impl CrowApp {
    /// How Crow is talking to the active server, for the titlebar.
    fn session_label(&self) -> String {
        use crate::host::{connection_state, transport_kind, ConnectionState, TransportKind};
        let Some(srv) = self.fleet.active_server() else { return "local".into() };
        match transport_kind(&srv) {
            TransportKind::Local => "local".into(),
            TransportKind::Container => "lab container".into(),
            TransportKind::Ssh => match connection_state(&srv.id) {
                None => "ssh · connecting".into(),
                Some(ConnectionState::Connected) => "ssh · connected".into(),
                Some(state) => format!("ssh · {}", state.label().to_lowercase()),
            },
        }
    }
}
