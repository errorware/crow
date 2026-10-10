use gpui_kit::*;
use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
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
    Ok(t!("danger-zone-simulated-signal-sent-to-crow", action = action, name = server.name))
}

pub fn flush_firewall(server: &ServerRecord) -> Result<String, String> {
    let host = host_for(server);
    if host.exec(&["which", "ufw"], DEFAULT_TIMEOUT).is_err() {
        return Err(t!("danger-zone-this-host-isnt-running-ufw").to_string());
    }
    host.exec_privileged(&["ufw", "--force", "reset"], &[], DEFAULT_TIMEOUT).map_err(|e| e.to_string())?;
    Ok(t!("danger-zone-ufw-rules-flushed-on", label = host.label()))
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
        "boot" => "BOOT",
        "snapshot" => "SNAPSHOT",
        "flush_firewall" => "FLUSH",
        "kill_containers" => "KILL",
        _ => "CONFIRM",
    }
}

/// What the action about to be confirmed does, in words.
fn danger_action_title(action: &str) -> &'static str {
    match action {
        "poweroff" => t!("danger-zone-what-poweroff"),
        "reboot" => t!("danger-zone-what-reboot"),
        "boot" => t!("danger-zone-what-boot"),
        "snapshot" => t!("danger-zone-what-snapshot"),
        "flush_firewall" => t!("danger-zone-what-flush"),
        "kill_containers" => t!("danger-zone-what-kill"),
        _ => t!("danger-zone-what-other"),
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

/// What the active server's provider can do from outside SSH (ERR-47).
#[derive(Clone, Debug)]
pub struct ProviderActions {
    pub name: String,
    pub power: bool,
    pub snapshots: bool,
}

fn action_btn(id: &'static str, label: impl Into<SharedString>, app: Entity<CrowApp>, action: &'static str) -> impl IntoElement {
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
        .child(label.into())
}

/// One action on the Danger Zone page: what it does, and its button.
pub fn action_row(title: &'static str, detail: impl Into<SharedString>, buttons: Vec<AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(16.0))
        .px(px(16.0))
        .py(px(12.0))
        .border_b_1()
        .border_color(BORDER_DANGER)
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(3.0))
                .child(div().text_size(px(12.0)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_PRIMARY).child(title))
                .children(crate::components::english_beside::english_beside(crate::i18n::english_of(title)))
                .child(div().text_size(px(11.0)).text_color(TEXT_DIM).child(detail.into())),
        )
        .child(div().flex().flex_none().gap(px(8.0)).children(buttons))
}

/// The server's Danger Zone, a page of its own: destructive actions, each
/// confirmed by typing its keyword, and archiving Crow's record of it.
pub fn danger_zone(danger: &DangerZoneState, provider: Option<&ProviderActions>, app: Entity<CrowApp>) -> impl IntoElement {
    let power = provider.filter(|p| p.power);
    let snapshots = provider.filter(|p| p.snapshots);
    let btn = |id, label: String, action| action_btn(id, label, app.clone(), action).into_any_element();
    let mut rows: Vec<AnyElement> = Vec::new();
    rows.push(match power {
        Some(p) => action_row(
            t!("danger-zone-power"),
            t!("danger-zone-through-works-even-when-ssh", name = p.name),
            vec![btn("btn-danger-poweroff", t!("danger-zone-power-off").into(), "poweroff"), btn("btn-danger-reboot", "REBOOT".into(), "reboot"), btn("btn-danger-boot", "BOOT".into(), "boot")],
        )
        .into_any_element(),
        None => action_row(
            t!("danger-zone-power"),
            t!("danger-zone-simulated-crow-doesnt-send-real"),
            vec![btn("btn-danger-poweroff", t!("danger-zone-power-off").into(), "poweroff"), btn("btn-danger-reboot", t!("danger-zone-reboot-45s-downtime").into(), "reboot")],
        )
        .into_any_element(),
    });
    if let Some(p) = snapshots {
        let app = app.clone();
        let list = div()
            .id("btn-danger-snapshots")
            .text_size(px(11.0))
            .text_color(TEXT_SECONDARY)
            .border_1()
            .border_color(BORDER_DEFAULT)
            .px(px(10.0))
            .py(px(4.0))
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
            .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.open_snapshots_panel(cx)))
            .child(t!("danger-zone-snapshots"))
            .into_any_element();
        rows.push(action_row(t!("danger-zone-snapshots-2"), t!("danger-zone-taken-and-listed-at", name = p.name), vec![list, btn("btn-danger-snapshot", "SNAPSHOT".into(), "snapshot")]).into_any_element());
    }
    rows.push(action_row(t!("danger-zone-flush-firewall"), t!("danger-zone-resets-ufw-every-rule-is"), vec![btn("btn-danger-flush-fw", t!("danger-zone-flush-firewall-ufw").into(), "flush_firewall")]).into_any_element());
    rows.push(
        action_row(
            t!("danger-zone-rotate-host-keys"),
            t!("danger-zone-not-built-yet"),
            vec![div().id("btn-danger-rotate-keys").text_size(px(11.0)).text_color(TEXT_FAINTER).border_1().border_color(BORDER_DEFAULT).px(px(10.0)).py(px(4.0)).child(t!("danger-zone-not-yet-built")).into_any_element()],
        )
        .into_any_element(),
    );
    rows.push(action_row(t!("danger-zone-lab-containers"), t!("danger-zone-stops-every-lab-node-crow"), vec![btn("btn-danger-kill-containers", t!("danger-zone-kill-all-lab-containers").into(), "kill_containers")]).into_any_element());
    // Archive acts on Crow's own record, not the host: last, and it opens an
    // explanation rather than the keyword prompt.
    let app_archive = app.clone();
    rows.push(
        action_row(
            t!("danger-zone-archive-server"),
            t!("danger-zone-removes-it-from-the-fleet"),
            vec![div()
                .id("btn-danger-archive")
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
                .on_click(move |_ev, _window, cx| app_archive.update(cx, |this, cx| this.request_archive_active_server(cx)))
                .child(t!("danger-zone-archive-server-2"))
                .into_any_element()],
        )
        .into_any_element(),
    );

    div()
        .id("danger-zone-page")
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .font_family(FONT_MONO)
        .overflow_y_scrollbar()
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(9.0))
                .px(px(16.0))
                .bg(CRIT_STRIP_BG)
                .border_b_1()
                .border_color(BORDER_DANGER)
                .child(danger_triangle())
                .child(div().text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(CRIT).child(t!("danger-zone-danger-zone")))
                .child(div().flex_1())
                .child(div().text_size(px(10.0)).text_color(TEXT_DIMMER).child(t!("danger-zone-type-the-actions-keyword-to"))),
        )
        .children(danger.pending_action.clone().map(|action| {
            div().flex().items_center().min_h(px(46.0)).bg(CRIT_STRIP_BG).border_b_1().border_color(BORDER_DANGER).child(render_confirm_prompt(&action, danger, app.clone()))
        }))
        .children(danger.last_result.as_ref().map(|msg| div().px(px(16.0)).py(px(8.0)).border_b_1().border_color(BORDER_DANGER).text_size(px(10.5)).text_color(TEXT_SECONDARY).child(msg.clone())))
        .child(div().flex().flex_col().max_w(px(920.0)).children(rows))
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
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(div().font_family(FONT_MONO).text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(CRIT_INK).child(danger_action_title(action)))
                .children(crate::components::english_beside::english_beside(crate::i18n::english_of(danger_action_title(action)))),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(CRIT_INK)
                .child(t!("danger-zone-type-to-confirm", keyword = keyword)),
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
                .child(t!("danger-zone-execute")),
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
                .child(t!("danger-zone-cancel")),
        )
        .children(danger.error.as_ref().map(|err| {
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(CRIT)
                .child(err.clone())
        }))
}
