use gpui_kit::*;
use gpui_kit::component::input::Input;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::danger_zone_state::DangerZoneState;
use crate::lab::{stop_local_node, LocalTestNode};
use crate::host::{host_for, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;

/// Power actions never actually execute. host_for() can now tell a lab
/// container from the machine Crow runs on, but powering off or rebooting
/// "localhost" would take Crow down with it, and remote hosts have no
/// transport yet. Until power actions get their own design, this stays a
/// named, honest simulation on every host — a safety boundary, not a
/// shortcut we forgot to finish.
pub fn send_power_action(server: &ServerRecord, action: &str) -> Result<String, String> {
    Ok(format!(
        "Simulated {} signal sent to {} — Crow does not execute real power actions yet",
        action, server.name
    ))
}

pub fn flush_firewall(server: &ServerRecord) -> Result<String, String> {
    let host = host_for(server);
    if host.exec(&["which", "ufw"], DEFAULT_TIMEOUT).is_err() {
        return Err("This host isn't running ufw — nothing to flush.".to_string());
    }
    host.exec_privileged(&["ufw", "--force", "reset"], &[], DEFAULT_TIMEOUT).map_err(|e| e.to_string())?;
    Ok(format!("ufw rules flushed on {}", host.label()))
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

pub fn danger_zone(danger: &DangerZoneState, app: Entity<CrowApp>) -> impl IntoElement {
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
        .children(if let Some(action) = danger.pending_action.clone() {
            Some(render_confirm_prompt(&action, danger, app.clone()).into_any_element())
        } else {
            None
        })
        .children(if danger.pending_action.is_none() {
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
                .children(danger.last_result.as_ref().map(|msg| {
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child(msg.clone())
                }))
                .children(if danger.last_result.is_none() {
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
        // 4. Archive (ERR-32). Everything left of this acts on the host; this
        //    acts on Crow's own record of it, so it sits apart at the far right
        //    and opens an explanation rather than the keyword prompt.
        .children(if danger.pending_action.is_none() {
            let app_archive = app.clone();
            Some(
                div()
                    .flex()
                    .items_center()
                    .px(px(14.0))
                    .border_l_1()
                    .border_color(BORDER_DANGER)
                    .child(
                        div()
                            .id("btn-danger-archive")
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(CRIT_INK)
                            .border_1()
                            .border_color(BORDER_DANGER_BTN)
                            .bg(CRIT_BG)
                            .px(px(10.0))
                            .py(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(CRIT_ROW_BG).text_color(CRIT))
                            .on_click(move |_ev, _window, cx| {
                                app_archive.update(cx, |this, cx| {
                                    this.request_archive_active_server(cx);
                                });
                            })
                            .child("ARCHIVE SERVER"),
                    )
                    .into_any_element(),
            )
        } else {
            None
        })
}

fn render_confirm_prompt(action: &str, danger: &DangerZoneState, app: Entity<CrowApp>) -> impl IntoElement {
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
        .children(danger.confirm_input.as_ref().map(|state| {
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
        .children(danger.error.as_ref().map(|err| {
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(CRIT)
                .child(err.clone())
        }))
}
