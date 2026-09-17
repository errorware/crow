use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

pub fn identity_bar(app: Entity<CrowApp>) -> impl IntoElement {
    let app_clone = app.clone();
    div()
        .h(px(52.0))
        .flex_none()
        .flex()
        .items_center()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        // 1. Server identity
        .child(
            div()
                .flex()
                .items_baseline()
                .gap(px(10.0))
                .px(px(16.0))
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("edge-01"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.5))
                        .text_color(TEXT_DIM)
                        .child("root@159.223.84.17:22"),
                ),
        )
        // 2. Connection state
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .pr(px(16.0))
                .h_full()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .size(px(7.0))
                        .rounded_full()
                        .bg(OK)
                        .flex_none(),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(OK)
                        .child("CONNECTED"),
                ),
        )
        // 3. Badge cluster
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(16.0))
                .h_full()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .bg(BG_CHIP)
                        .text_color(TEXT_SECONDARY)
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child("UBUNTU 24.04 LTS"),
                )
                .child(
                    div()
                        .bg(CRIT)
                        .text_color(hex_rgb(0x0a0a0c))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child("PROD"),
                )
                .child(
                    div()
                        .bg(WARN)
                        .text_color(hex_rgb(0x0a0a0c))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child("3 BEHIND"),
                )
                .child(
                    div()
                        .bg(BG_CHIP)
                        .text_color(TEXT_TERTIARY)
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child("KERNEL 6.8.0-45"),
                ),
        )
        // 4. Spacer
        .child(div().flex_1())
        // 5. Command trigger
        .child(
            div()
                .id("cmd-palette-trigger")
                .flex()
                .items_center()
                .gap(px(10.0))
                .h(px(26.0))
                .w(px(300.0))
                .px(px(10.0))
                .mr(px(12.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .bg(hex_rgb(0x0e0f13))
                .cursor_pointer()
                .on_click(move |_ev, _window, cx| {
                    app_clone.update(cx, |this, cx| {
                        this.toggle_palette(cx);
                    });
                })
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .text_color(TEXT_FAINTER)
                        .child("⌕"),
                )
                .child(
                    div()
                        .flex_1()
                        .font_family("Inter")
                        .text_size(px(11.5))
                        .text_color(TEXT_FAINT)
                        .child("Search or run command…"),
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
                        .child("⌘K"),
                ),
        )
        // 6. Action group
        .child(
            div()
                .flex()
                .items_center()
                .mr(px(12.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(5.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .bg(BG_CONTROL)
                        .border_r_1()
                        .border_color(BORDER_DEFAULT)
                        .child("SNAPSHOT"),
                )
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(5.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .child("RECONNECT"),
                ),
        )
}
