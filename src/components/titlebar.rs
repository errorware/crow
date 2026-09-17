use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};

pub fn diamond_mark() -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let mut path = PathBuilder::fill();
            let mid_x = bounds.origin.x + bounds.size.width / 2.0;
            let mid_y = bounds.origin.y + bounds.size.height / 2.0;
            path.move_to(point(mid_x, bounds.origin.y));
            path.line_to(point(bounds.origin.x + bounds.size.width, mid_y));
            path.line_to(point(mid_x, bounds.origin.y + bounds.size.height));
            path.line_to(point(bounds.origin.x, mid_y));
            path.close();
            if let Ok(built) = path.build() {
                window.paint_path(built, TEXT_PRIMARY);
            }
        },
    )
    .w(px(14.0))
    .h(px(14.0))
    .flex_none()
}

#[derive(Clone, Debug)]
pub struct ServerTab {
    pub id: &'static str,
    pub name: &'static str,
    pub status_color: Rgba,
    #[allow(dead_code)]
    pub is_active: bool,
}

pub fn titlebar(tabs: &[ServerTab], active_tab_id: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let mut bar = div()
        .h(px(36.0))
        .flex_none()
        .flex()
        .items_stretch()
        .bg(BG_CHROME)
        .border_b_1()
        .border_color(BORDER_PANEL);

    // 1. macOS native traffic lights clearance (only on macOS)
    #[cfg(target_os = "macos")]
    {
        bar = bar.child(
            div()
                .w(px(76.0))
                .flex_none(),
        );
    }

    // 2. App mark & name
    bar = bar.child(
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(12.0))
            .border_r_1()
            .border_color(BORDER_PANEL)
            .child(diamond_mark())
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(TEXT_PRIMARY)
                    .child("CROW"),
            ),
    );

    // 3. Tab strip with window drag affordance
    bar = bar.child(
        div()
            .flex()
            .items_stretch()
            .flex_1()
            .min_w(px(0.0))
            .on_mouse_down(MouseButton::Left, |ev, window, _cx| {
                if ev.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            })
            .children(tabs.iter().enumerate().map(|(idx, tab)| {
                let is_active = tab.id == active_tab_id;
                let tab_id = tab.id;
                let app_tab = app.clone();

                div()
                    .id(ElementId::NamedInteger("server-tab".into(), idx as u64))
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .px(px(14.0))
                    .border_r_1()
                    .border_color(BORDER_PANEL)
                    .bg(if is_active { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                    .cursor_pointer()
                    .on_click(move |_ev, _window, cx| {
                        app_tab.update(cx, |this, cx| {
                            this.switch_tab(tab_id, cx);
                        });
                    })
                    .child(
                        div()
                            .size(px(6.0))
                            .rounded_full()
                            .bg(tab.status_color)
                            .flex_none(),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(if is_active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .child(tab.name),
                    )
                    .child(
                        tabler_icon(TablerIcon::X)
                            .size(px(11.0))
                            .text_color(rgb(0x6b7280)),
                    )
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .px(px(12.0))
                    .cursor_pointer()
                    .child(
                        tabler_icon(TablerIcon::Plus)
                            .size(px(13.0))
                            .text_color(TEXT_FAINT),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .h_full(),
            ),
    );

    // 4. Session meta
    bar = bar.child(
        div()
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(14.0))
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .text_color(TEXT_FAINT)
            .border_l_1()
            .border_color(BORDER_PANEL)
            .child(div().child("SSH ED25519"))
            .child(div().child("lat 12ms"))
            .child(div().child("03:41:22 UTC")),
    );

    // 5. Linux / Windows platform window controls
    #[cfg(not(target_os = "macos"))]
    {
        bar = bar.child(
            div()
                .flex()
                .items_stretch()
                .border_l_1()
                .border_color(BORDER_PANEL)
                // Minimize
                .child(
                    div()
                        .id("win-btn-minimize")
                        .w(px(38.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(|_ev, window, _cx| {
                            window.minimize_window();
                        })
                        .child(
                            tabler_icon(TablerIcon::Minus)
                                .size(px(13.0))
                                .text_color(TEXT_MUTED),
                        ),
                )
                // Maximize / Restore
                .child(
                    div()
                        .id("win-btn-maximize")
                        .w(px(38.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(|_ev, window, _cx| {
                            window.zoom_window();
                        })
                        .child(
                            tabler_icon(TablerIcon::Square)
                                .size(px(12.0))
                                .text_color(TEXT_MUTED),
                        ),
                )
                // Close
                .child(
                    div()
                        .id("win-btn-close")
                        .w(px(38.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(CRIT_BG).text_color(CRIT_INK))
                        .on_click(|_ev, _window, cx| {
                            cx.quit();
                        })
                        .child(
                            tabler_icon(TablerIcon::X)
                                .size(px(13.0))
                                .text_color(TEXT_MUTED),
                        ),
                ),
        );
    }

    bar
}
