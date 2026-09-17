use gpui_kit::*;
use crate::theme::*;

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

pub fn danger_zone() -> impl IntoElement {
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
        // 2. Actions
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(14.0))
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .text_color(CRIT_INK_DIM)
                        .border_1()
                        .border_color(BORDER_DANGER_BTN)
                        .px(px(10.0))
                        .py(px(4.0))
                        .child("POWER OFF"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .text_color(CRIT_INK_DIM)
                        .border_1()
                        .border_color(BORDER_DANGER_BTN)
                        .px(px(10.0))
                        .py(px(4.0))
                        .child("REBOOT · 45s DOWNTIME"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .text_color(CRIT_INK_DIM)
                        .border_1()
                        .border_color(BORDER_DANGER_BTN)
                        .px(px(10.0))
                        .py(px(4.0))
                        .child("FLUSH FIREWALL (UFW)"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .text_color(CRIT_INK_DIM)
                        .border_1()
                        .border_color(BORDER_DANGER_BTN)
                        .px(px(10.0))
                        .py(px(4.0))
                        .child("ROTATE HOST KEYS"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .text_color(CRIT_INK_DIM)
                        .border_1()
                        .border_color(BORDER_DANGER_BTN)
                        .px(px(10.0))
                        .py(px(4.0))
                        .child("KILL ALL CONTAINERS"),
                ),
        )
        // 3. Policy note
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .border_l_1()
                .border_color(BORDER_DANGER)
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("every action requires typed confirmation"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .bg(BG_KEY)
                        .border_1()
                        .border_color(BORDER_KEY)
                        .px(px(5.0))
                        .py(px(1.0))
                        .child("⇧⌘D"),
                ),
        )
}
