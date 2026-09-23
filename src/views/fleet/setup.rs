use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};

/// Fleet groups, dependencies and baseline policy. Crow doesn't model any of
/// these yet, so the screen says so instead of showing an example fleet.
pub fn fleet_setup_view(app: Entity<CrowApp>) -> impl IntoElement {
    let app_close = app.clone();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Top Header
        .child(
            div()
                .h(px(52.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child("FLEET SETUP"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .text_color(TEXT_DIM)
                                .child("groups, dependencies & baseline policy"),
                        ),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-close-fleet-setup")
                        .px(px(10.0))
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_close.update(cx, |this, cx| {
                                this.set_screen(Screen::Fleet, cx);
                            });
                        })
                        .child("CLOSE esc"),
                ),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .font_family(FONT_MONO)
                .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("NOT BUILT YET"))
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(TEXT_DIM)
                        .child("Crow doesn't track fleet groups, service dependencies or baseline policies yet."),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(TEXT_DIM)
                        .child("Nothing here is enforced on your servers."),
                ),
        )
}
