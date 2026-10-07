//! Danger Zone → SNAPSHOTS: the active server's snapshots at its provider
//! (ERR-47). A plain centred box: providers keep only a handful.

use gpui_kit::*;
use crate::components::icon_button::icon_button;
use crate::components::icons::TablerIcon;

use crate::app::danger::SnapshotsPanel;
use crate::app::CrowApp;
use crate::theme::*;

pub fn snapshots_panel(panel: &SnapshotsPanel, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_scrim, a_close) = (app.clone(), app);
    let body: AnyElement = match &panel.result {
        None => div().text_color(TEXT_DIMMER).child(format!("asking {}…", panel.provider)).into_any_element(),
        Some(Err(e)) => div().text_color(CRIT).line_height(px(15.0)).child(e.clone()).into_any_element(),
        Some(Ok(list)) if list.is_empty() => div().text_color(TEXT_DIMMER).child(format!("{} has no snapshots of {} yet.", panel.provider, panel.server)).into_any_element(),
        Some(Ok(list)) => div()
            .flex()
            .flex_col()
            .children(list.iter().map(|s| {
                let color = match s.status.as_str() {
                    "successful" => OK,
                    "pending" | "running" | "needs post processing" => WARN,
                    _ => CRIT,
                };
                div()
                    .flex()
                    .gap(px(12.0))
                    .py(px(6.0))
                    .border_b_1()
                    .border_color(BORDER_ROW)
                    .child(div().w(px(180.0)).flex_none().text_color(TEXT_PRIMARY).child(s.label.clone().unwrap_or_else(|| "(automatic)".into())))
                    .child(div().w(px(110.0)).flex_none().text_color(color).child(s.status.clone()))
                    .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_DIMMER).child(s.created.clone().unwrap_or_default().replace('T', " ")))
                    .child(div().flex_none().text_color(TEXT_FAINT).child(format!("#{}", s.id.0)))
            }))
            .into_any_element(),
    };
    div()
        .id("snapshots-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| a_scrim.update(cx, |this, cx| this.close_snapshots_panel(cx)))
        .child(
            div()
                .id("snapshots-panel")
                .w(px(620.0))
                .p(px(16.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(format!("SNAPSHOTS · {}", panel.server)))
                        .child(div().text_color(TEXT_DIMMER).child(format!("at {}", panel.provider)))
                        .child(div().flex_1())
                        .child(
                            icon_button("snapshots-close", TablerIcon::X, false)
                                .on_click(move |_ev, _window, cx| a_close.update(cx, |this, cx| this.close_snapshots_panel(cx))),
                        ),
                )
                .child(body)
                .child(div().text_color(TEXT_FAINT).line_height(px(15.0)).child("Restore a snapshot from the provider's own console: restoring replaces the server's disks, and Crow doesn't do that for you.")),
        )
}
