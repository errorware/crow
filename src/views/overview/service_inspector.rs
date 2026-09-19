use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::ServiceUnit;

/// Maps a systemd unit name to the crow-config managed file that governs it,
/// if one is known. None means there's nothing to hand off to the Config screen yet.
fn config_file_for_service(unit: &str) -> Option<&'static str> {
    let u = unit.to_lowercase();
    if u.contains("postgres") {
        Some("pg_hba.conf")
    } else if u.contains("sshd") || u == "ssh.service" {
        Some("sshd_config")
    } else if u.contains("nginx") {
        Some("nginx.conf")
    } else if u.contains("ufw") {
        Some("ufw/user.rules")
    } else if u.contains("fail2ban") {
        Some("fail2ban/jail.local")
    } else if u.contains("journald") {
        Some("journald.conf")
    } else if u.contains("docker") {
        Some("docker/daemon.json")
    } else if u.contains("cron") {
        Some("crontab")
    } else {
        None
    }
}

pub fn service_inspector_rail(app_data: &CrowApp, app: Entity<CrowApp>) -> impl IntoElement {
    let focused = app_data.services.iter().find(|s| s.is_focused);

    div()
        .w(px(360.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        // Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("SERVICE MANAGER"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("systemctl"),
                ),
        )
        .children(match focused {
            None => Some(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(6.0))
                    .p(px(24.0))
                    .child(
                        tabler_icon(TablerIcon::Server)
                            .size(px(22.0))
                            .text_color(TEXT_FAINTER),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(TEXT_DIMMER)
                            .text_align(TextAlign::Center)
                            .child("Select a service on the left to manage it"),
                    )
                    .into_any_element(),
            ),
            Some(svc) => Some(
                render_focused_panel(svc, app_data.service_panel_pending_action.as_deref(), app)
                    .into_any_element(),
            ),
        })
}

fn info_cell(label: &str, value: String, value_color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(label.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(12.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(value_color)
                .child(value),
        )
}

fn render_focused_panel(svc: &ServiceUnit, pending_action: Option<&str>, app: Entity<CrowApp>) -> impl IntoElement {
    let is_active = svc.status == "ACTIVE";
    let pill_bg = match svc.status.as_str() {
        "ACTIVE" => OK_BG,
        "DEGRADED" | "PENDING" => WARN_BG,
        _ => CRIT_BG,
    };
    let config_file = config_file_for_service(&svc.name);

    div()
        .id("service-inspector-scroll")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .p(px(14.0))
        // Identity
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(svc.name.clone()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .bg(pill_bg)
                                .text_color(svc.status_color())
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(5.0))
                                .py(px(2.0))
                                .child(svc.status.clone()),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .child(svc.description.clone()),
                        ),
                ),
        )
        // Info grid
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(16.0))
                .px(px(2.0))
                .child(info_cell("PID", svc.pid.clone(), TEXT_SECONDARY))
                .child(info_cell("CPU", svc.cpu.clone(), TEXT_SECONDARY))
                .child(info_cell("MEM", svc.mem.clone(), TEXT_SECONDARY))
                .child(info_cell("RSS", svc.rss.clone(), TEXT_SECONDARY))
                .child(info_cell("UPTIME", svc.uptime.clone(), TEXT_SECONDARY)),
        )
        // Lifecycle actions
        .child(section_divider("LIFECYCLE"))
        .child(render_actions_row(is_active, app.clone()))
        // Confirm bar for disruptive actions
        .children(pending_action.map(|action| render_pending_confirm(&svc.name, action, app.clone())))
        // Cross-nav: jump to this unit's log stream
        .child({
            let app_logs = app.clone();
            let unit_name = svc.name.clone();
            div()
                .id("btn-view-service-logs")
                .flex()
                .items_center()
                .justify_between()
                .p(px(10.0))
                .bg(BG_CONTROL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    app_logs.update(cx, |this, cx| {
                        this.jump_to_service_logs(&unit_name, cx);
                    });
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("VIEW LOGS FOR THIS UNIT"),
                )
                .child(
                    tabler_icon(TablerIcon::ChevronRight)
                        .size(px(14.0))
                        .text_color(TEXT_DIM),
                )
        })
        // Config handoff
        .children(config_file.map(|file| {
            let app_cfg = app.clone();
            let file_owned = file.to_string();
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(section_divider("CONFIGURATION"))
                .child(
                    div()
                        .id("btn-edit-service-config")
                        .flex()
                        .items_center()
                        .justify_between()
                        .p(px(10.0))
                        .bg(hex_rgba(0x4ade80, 0.08))
                        .border_1()
                        .border_color(OK)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgba(0x4ade80, 0.16)))
                        .on_click(move |_ev, _window, cx| {
                            app_cfg.update(cx, |this, cx| {
                                this.open_config_for_service(&file_owned, cx);
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .child(format!("EDIT {}", file)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_DIM)
                                        .child("open in crow-config — structured, validated editing"),
                                ),
                        )
                        .child(
                            tabler_icon(TablerIcon::ChevronRight)
                                .size(px(14.0))
                                .text_color(OK),
                        ),
                )
        }))
}

fn section_divider(label: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(label.to_string()),
        )
        .child(div().flex_1().h(px(1.0)).bg(BORDER_PANEL))
}

fn action_button(
    id: &'static str,
    label: &'static str,
    fg: Rgba,
    bg: Rgba,
    border: Rgba,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .py(px(6.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(fg)
        .on_click(on_click)
        .child(label)
}

fn render_actions_row(is_active: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let app_start = app.clone();
    let app_reload = app.clone();
    let app_restart = app.clone();
    let app_stop = app.clone();

    div()
        .flex()
        .gap(px(6.0))
        .children(if !is_active {
            Some(action_button(
                "btn-svc-start",
                "▶ START",
                OK,
                OK_BG,
                OK,
                move |_ev, _window, cx| {
                    app_start.update(cx, |this, cx| {
                        this.run_service_panel_action_now("start", cx);
                    });
                },
            ))
        } else {
            None
        })
        .children(if is_active {
            Some(action_button(
                "btn-svc-reload",
                "⟳ RELOAD",
                TEXT_SECONDARY,
                BG_CONTROL,
                BORDER_DEFAULT,
                move |_ev, _window, cx| {
                    app_reload.update(cx, |this, cx| {
                        this.run_service_panel_action_now("reload", cx);
                    });
                },
            ))
        } else {
            None
        })
        .child(action_button(
            "btn-svc-restart",
            "↻ RESTART",
            WARN,
            WARN_BG,
            WARN,
            move |_ev, _window, cx| {
                app_restart.update(cx, |this, cx| {
                    this.arm_service_panel_action("restart", cx);
                });
            },
        ))
        .children(if is_active {
            Some(action_button(
                "btn-svc-stop",
                "■ STOP",
                CRIT,
                CRIT_BG,
                CRIT,
                move |_ev, _window, cx| {
                    app_stop.update(cx, |this, cx| {
                        this.arm_service_panel_action("stop", cx);
                    });
                },
            ))
        } else {
            None
        })
}

fn render_pending_confirm(unit_name: &str, action: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let app_confirm = app.clone();
    let app_cancel = app.clone();
    let verb = action.to_uppercase();

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(10.0))
        .bg(CRIT_ROW_BG)
        .border_1()
        .border_color(CRIT)
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(CRIT_INK)
                .child(format!(
                    "{} {}? Active connections may be dropped.",
                    verb, unit_name
                )),
        )
        .child(
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    div()
                        .id("btn-svc-confirm-cancel")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_KEY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .on_click(move |_ev, _window, cx| {
                            app_cancel.update(cx, |this, cx| {
                                this.cancel_service_panel_action(cx);
                            });
                        })
                        .child("Cancel"),
                )
                .child(
                    div()
                        .id("btn-svc-confirm-execute")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(5.0))
                        .bg(CRIT)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgb(0xef4444)))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hex_rgb(0x0a0a0c))
                        .on_click(move |_ev, _window, cx| {
                            app_confirm.update(cx, |this, cx| {
                                this.execute_service_panel_action(cx);
                            });
                        })
                        .child(format!("{} ⏎", verb)),
                ),
        )
}
