use std::process::Command;
use gpui_kit::*;
use gpui_kit::component::input::Input;
use crate::theme::*;
use crate::app::CrowApp;
use crate::lab::{stop_local_node, LocalTestNode};
use crate::os_detect::{classify_distro_family, detect_local_os_release, DistroFamily};
use crate::vault::ServerRecord;

fn is_local_record(server: &ServerRecord) -> bool {
    server.host == "127.0.0.1"
        || server.host == "localhost"
        || server.host == "::1"
        || server.name.to_lowercase() == "localhost"
        || server.tags.iter().any(|t| t == "localhost" || t == "local")
}

/// Power actions never actually execute — there is no reliable way to tell
/// a lab container apart from the literal machine Crow itself runs on (both
/// share host 127.0.0.1 in every collector in this app), and real remote
/// execution has no transport yet (crow-ssh isn't built). Rather than risk
/// shutting down the wrong thing, this stays a named, honest simulation on
/// every host — a safety boundary, not a shortcut we forgot to finish.
pub fn send_power_action(server: &ServerRecord, action: &str) -> Result<String, String> {
    Ok(format!(
        "Simulated {} signal sent to {} — Crow does not execute real power actions yet",
        action, server.name
    ))
}

pub fn flush_firewall(server: &ServerRecord) -> Result<String, String> {
    if !is_local_record(server) {
        return Ok(format!("Simulated firewall flush sent to {}", server.name));
    }
    let family = detect_local_os_release()
        .map(|d| classify_distro_family(&d))
        .unwrap_or(DistroFamily::Unknown);
    if family != DistroFamily::Debian {
        return Err("This host isn't running ufw (Debian/Ubuntu-family firewall) — nothing to flush.".to_string());
    }
    let out = Command::new("ufw").args(["--force", "reset"]).output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok("ufw rules flushed".to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Stops every Crow-enrolled lab node — bounded to what Crow itself manages,
/// never a blind "kill every container on the host."
pub fn kill_all_lab_containers(nodes: &[LocalTestNode]) -> (usize, usize) {
    let mut ok = 0;
    let mut failed = 0;
    for node in nodes {
        match stop_local_node(node) {
            Ok(()) => ok += 1,
            Err(_) => failed += 1,
        }
    }
    (ok, failed)
}

pub fn danger_action_keyword(action: &str) -> &'static str {
    match action {
        "poweroff" => "POWEROFF",
        "reboot" => "REBOOT",
        "flush_firewall" => "FLUSH",
        "kill_containers" => "KILL",
        _ => "CONFIRM",
    }
}

pub fn danger_triangle() -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let mut path = PathBuilder::fill();
            let mid_x = bounds.origin.x + bounds.size.width / 2.0;
            path.move_to(point(mid_x, bounds.origin.y));
            path.line_to(point(bounds.origin.x + bounds.size.width, bounds.origin.y + bounds.size.height));
            path.line_to(point(bounds.origin.x, bounds.origin.y + bounds.size.height));
            path.close();
            if let Ok(built) = path.build() {
                window.paint_path(built, CRIT);
            }
        },
    )
    .w(px(12.0))
    .h(px(12.0))
    .flex_none()
}

fn action_btn(id: &'static str, label: &'static str, app: Entity<CrowApp>, action: &'static str) -> impl IntoElement {
    div()
        .id(id)
        .font_family("JetBrains Mono")
        .text_size(px(11.0))
        .text_color(CRIT_INK_DIM)
        .border_1()
        .border_color(BORDER_DANGER_BTN)
        .px(px(10.0))
        .py(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(CRIT_ROW_BG).text_color(CRIT))
        .on_click(move |_ev, window, cx| {
            app.update(cx, move |this, cx| {
                this.arm_danger_zone_action(action, window, cx);
            });
        })
        .child(label)
}

pub fn danger_zone(app_data: &CrowApp, app: Entity<CrowApp>) -> impl IntoElement {
    div()
        .h(px(46.0))
        .flex_none()
        .flex()
        .items_stretch()
        .bg(CRIT_STRIP_BG)
        .border_t_1()
        .border_color(BORDER_DANGER)
        // 1. Label
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(9.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_DANGER)
                .child(danger_triangle())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(CRIT)
                        .child("DANGER ZONE"),
                ),
        )
        // 2. Actions, or the typed-confirmation prompt for an armed action
        .children(if let Some(action) = app_data.danger_zone_pending_action.clone() {
            Some(render_confirm_prompt(&action, app_data, app.clone()).into_any_element())
        } else {
            None
        })
        .children(if app_data.danger_zone_pending_action.is_none() {
            Some(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(14.0))
                    .child(action_btn("btn-danger-poweroff", "POWER OFF", app.clone(), "poweroff"))
                    .child(action_btn("btn-danger-reboot", "REBOOT · 45s DOWNTIME", app.clone(), "reboot"))
                    .child(action_btn("btn-danger-flush-fw", "FLUSH FIREWALL (UFW)", app.clone(), "flush_firewall"))
                    .child(
                        div()
                            .id("btn-danger-rotate-keys")
                            .font_family("JetBrains Mono")
                            .text_size(px(11.0))
                            .text_color(TEXT_FAINTER)
                            .border_1()
                            .border_color(BORDER_DEFAULT)
                            .px(px(10.0))
                            .py(px(4.0))
                            .child("ROTATE HOST KEYS · NOT YET BUILT"),
                    )
                    .child(action_btn("btn-danger-kill-containers", "KILL ALL LAB CONTAINERS", app.clone(), "kill_containers"))
                    .into_any_element(),
            )
        } else {
            None
        })
        // 3. Policy note / last result
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .border_l_1()
                .border_color(BORDER_DANGER)
                .children(app_data.danger_zone_last_result.as_ref().map(|msg| {
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child(msg.clone())
                }))
                .children(if app_data.danger_zone_last_result.is_none() {
                    Some(
                        div()
                            .font_family("JetBrains Mono")
                            .text_size(px(10.0))
                            .text_color(TEXT_DIMMER)
                            .child("type the action's keyword to confirm — no single-click destructive actions"),
                    )
                } else {
                    None
                }),
        )
}

fn render_confirm_prompt(action: &str, app_data: &CrowApp, app: Entity<CrowApp>) -> impl IntoElement {
    let keyword = danger_action_keyword(action);
    let app_execute = app.clone();
    let app_cancel = app.clone();

    div()
        .flex_1()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(14.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(CRIT_INK)
                .child(format!("Type {} to confirm:", keyword)),
        )
        .children(app_data.danger_zone_confirm_state.as_ref().map(|state| {
            div()
                .w(px(160.0))
                .child(
                    Input::new(state)
                        .id("input-danger-confirm")
                        .font_family(FONT_MONO)
                        .bg(BG_APP)
                        .border_color(BORDER_DANGER_BTN)
                        .rounded(px(2.0)),
                )
        }))
        .child(
            div()
                .id("btn-danger-confirm-execute")
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .text_color(hex_rgb(0x0a0a0c))
                .bg(CRIT)
                .px(px(9.0))
                .py(px(4.0))
                .cursor_pointer()
                .hover(|s| s.bg(hex_rgb(0xef4444)))
                .on_click(move |_ev, _window, cx| {
                    app_execute.update(cx, |this, cx| {
                        this.execute_danger_zone_action(cx);
                    });
                })
                .child("EXECUTE ⏎"),
        )
        .child(
            div()
                .id("btn-danger-confirm-cancel")
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_TERTIARY)
                .cursor_pointer()
                .hover(|s| s.text_color(TEXT_PRIMARY))
                .on_click(move |_ev, _window, cx| {
                    app_cancel.update(cx, |this, cx| {
                        this.cancel_danger_zone_action(cx);
                    });
                })
                .child("cancel"),
        )
        .children(app_data.danger_zone_error.as_ref().map(|err| {
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(CRIT)
                .child(err.clone())
        }))
}
