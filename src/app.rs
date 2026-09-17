use gpui_kit::*;
use crate::theme::*;
use crate::components::danger_zone::danger_zone;
use crate::components::identity_bar::identity_bar;
use crate::components::palette::palette_overlay;
use crate::components::sidebar::sidebar;
use crate::components::stat_strip::stat_strip;
use crate::components::titlebar::{titlebar, ServerTab};
use crate::views::config::managed_files::managed_files_rail;
use crate::views::config::pending_diff_rail::pending_diff_rail;
use crate::views::config::rules_editor::{default_hba_rules, rules_editor, HbaRuleDef};
use crate::views::overview::log_tail::log_tail;
use crate::views::overview::services_table::{default_services, services_table, ServiceUnit};

use crow_config_core::edit::ConfigDocument;
use crow_config_core::ConfigPlugin;
use crow_config_schemas::PgHbaPlugin;

pub struct CrowApp {
    focus_handle: FocusHandle,
    pub active_tab_id: String,
    pub active_view: String,
    pub active_services_tab: String,
    pub tabs: Vec<ServerTab>,
    pub services: Vec<ServiceUnit>,
    pub hba_rules: Vec<HbaRuleDef>,
    pub palette_open: bool,
    pub sidebar_collapsed: bool,
}

impl CrowApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Demonstrate integration with crow-config-core & crow-config-schemas
        let plugin = PgHbaPlugin::new();
        let sample_pg_hba = r#"
# PostgreSQL Client Authentication Configuration File
local   all             postgres                                peer
host    all             all             127.0.0.1/32            scram-sha-256
host    all             all             ::1/128                 scram-sha-256
host    acme_prod       acme_app        10.0.4.19/32            scram-sha-256
host    all             all             0.0.0.0/0               md5
host    all             all             10.0.4.0/24             scram-sha-256
"#;
        if let Ok(doc) = ConfigDocument::parse(&plugin, sample_pg_hba) {
            if let Ok(ir) = doc.to_ir() {
                println!(
                    "Crow Core: parsed {} pg_hba rules from schema plugin: {}",
                    ir.rows.len(),
                    plugin.manifest().plugin.name
                );
            }
        }

        Self {
            focus_handle: cx.focus_handle(),
            active_tab_id: "edge-01".to_string(),
            active_view: "overview".to_string(),
            active_services_tab: "services".to_string(),
            tabs: vec![
                ServerTab { id: "edge-01", name: "edge-01", status_color: OK, is_active: true },
                ServerTab { id: "edge-02", name: "edge-02", status_color: OK, is_active: false },
                ServerTab { id: "db-primary", name: "db-primary", status_color: WARN, is_active: false },
                ServerTab { id: "worker-04", name: "worker-04", status_color: CRIT, is_active: false },
                ServerTab { id: "bastion", name: "bastion", status_color: TEXT_FAINTER, is_active: false },
            ],
            services: default_services(),
            hba_rules: default_hba_rules(),
            palette_open: false,
            sidebar_collapsed: false,
        }
    }

    pub fn toggle_palette(&mut self, cx: &mut Context<Self>) {
        self.palette_open = !self.palette_open;
        cx.notify();
    }

    pub fn close_palette(&mut self, cx: &mut Context<Self>) {
        self.palette_open = false;
        cx.notify();
    }

    pub fn set_view(&mut self, view: &str, cx: &mut Context<Self>) {
        if view == "services" {
            self.active_view = "overview".to_string();
            self.active_services_tab = "services".to_string();
        } else if view == "processes" {
            self.active_view = "overview".to_string();
            self.active_services_tab = "processes".to_string();
        } else {
            self.active_view = view.to_string();
        }
        cx.notify();
    }

    pub fn set_services_tab(&mut self, tab: &str, cx: &mut Context<Self>) {
        self.active_services_tab = tab.to_string();
        cx.notify();
    }

    pub fn switch_tab(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        self.active_tab_id = tab_id.to_string();
        cx.notify();
    }

    pub fn focus_service(&mut self, name: &str, cx: &mut Context<Self>) {
        for svc in &mut self.services {
            svc.is_focused = svc.name == name;
        }
        cx.notify();
    }

    pub fn toggle_service_confirm(&mut self, name: &str, cx: &mut Context<Self>) {
        for svc in &mut self.services {
            if svc.name == name {
                svc.show_confirm = !svc.show_confirm;
            } else {
                svc.show_confirm = false;
            }
        }
        cx.notify();
    }

    pub fn toggle_rule_expand(&mut self, num: &str, cx: &mut Context<Self>) {
        for r in &mut self.hba_rules {
            if r.num == num {
                r.is_expanded = !r.is_expanded;
            }
        }
        cx.notify();
    }

    pub fn set_rule_method(&mut self, rule_num: &str, method: &'static str, cx: &mut Context<Self>) {
        for r in &mut self.hba_rules {
            if r.num == rule_num {
                r.method = method;
                r.risk = if method == "scram-sha-256" || method == "cert" {
                    "OK"
                } else if method == "trust" {
                    "CRITICAL"
                } else {
                    "REVIEW"
                };
                r.risk_color = if r.risk == "OK" {
                    OK
                } else if r.risk == "CRITICAL" {
                    CRIT
                } else {
                    WARN
                };
            }
        }
        cx.notify();
    }
}

impl Render for CrowApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_overview = self.active_view == "overview";
        let is_config = self.active_view == "config";
        let palette_open = self.palette_open;
        let app_view = cx.entity();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _window, cx| {
                if ev.keystroke.key == "escape" {
                    if this.palette_open {
                        this.palette_open = false;
                        cx.notify();
                    }
                } else if ev.keystroke.key.to_lowercase() == "k"
                    && (ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control)
                {
                    this.palette_open = !this.palette_open;
                    cx.notify();
                } else if ev.keystroke.key == "\\"
                    && (ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control)
                {
                    this.sidebar_collapsed = !this.sidebar_collapsed;
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
            // 1. Frameless Titlebar
            .child(titlebar(&self.tabs, &self.active_tab_id, app_view.clone()))
            // 2. Identity Bar
            .child(identity_bar(app_view.clone()))
            // 3. Stat Strip
            .child(stat_strip())
            // 4. Main Body Grid: Sidebar + Content
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .w_full()
                    // Sidebar
                    .child(sidebar(&self.active_view, self.sidebar_collapsed, app_view.clone()))
                    // Content Area (Overview or Config)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .h_full()
                            .flex()
                            .children(if is_overview {
                                Some(
                                    div()
                                        .size_full()
                                        .flex()
                                        .child(services_table(&self.services, &self.active_services_tab, app_view.clone()))
                                        .child(log_tail())
                                )
                            } else if is_config {
                                Some(
                                    div()
                                        .size_full()
                                        .flex()
                                        .child(managed_files_rail())
                                        .child(rules_editor(&self.hba_rules, app_view.clone()))
                                        .child(pending_diff_rail())
                                )
                            } else if self.active_view == "logs" {
                                Some(
                                    div()
                                        .size_full()
                                        .flex()
                                        .child(log_tail())
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
            // 5. Persistent Danger Zone Strip
            .child(danger_zone())
            // 6. Command Palette Overlay (⌘K)
            .children(if palette_open {
                Some(palette_overlay(app_view.clone()))
            } else {
                None
            })
    }
}
