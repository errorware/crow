use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::journal::JournalEntry;

pub fn log_tail(
    entries: &[JournalEntry],
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_expand = app.clone();

    let err_count = entries.iter().filter(|e| e.priority.is_error()).count();
    let warn_count = entries.iter().filter(|e| e.priority.is_warn()).count();
    let info_count = entries.len().saturating_sub(err_count + warn_count);

    div()
        .w(px(400.0))
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
                        .child("LOG TAIL"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("journalctl -f"),
                )
                .child(div().flex_1())
                // Expand to full view button
                .child(
                    div()
                        .id("log-tail-expand-btn")
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_MUTED)
                        .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .on_click(move |_ev, _window, cx| {
                            app_expand.update(cx, |this, cx| {
                                this.set_active_view("logs", cx);
                            });
                        })
                        .child("EXPAND ↗"),
                )
                .child(
                    div()
                        .size(px(6.0))
                        .rounded_full()
                        .bg(OK)
                        .flex_none(),
                ),
        )
        // Filter bar
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(12.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family("JetBrains Mono")
                .text_size(px(10.0))
                .child(
                    div()
                        .bg(if err_count > 0 { CRIT } else { BG_CONTROL })
                        .text_color(if err_count > 0 { rgb(0x0a0a0c) } else { TEXT_DIMMER })
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child(format!("ERROR {}", err_count)),
                )
                .child(
                    div()
                        .bg(if warn_count > 0 { WARN } else { BG_CONTROL })
                        .text_color(if warn_count > 0 { rgb(0x0a0a0c) } else { TEXT_DIMMER })
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child(format!("WARN {}", warn_count)),
                )
                .child(
                    div()
                        .bg(BG_CHIP)
                        .text_color(TEXT_TERTIARY)
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child(format!("INFO {}", info_count)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_color(TEXT_FAINT)
                        .child("stream active"),
                ),
        )
        // Log stream
        .child(
            div()
                .id("log-stream-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .py(px(6.0))
                .children(entries.iter().map(|entry| {
                    let is_err = entry.priority.is_error();
                    let is_warn = entry.priority.is_warn();

                    let (lvl_color, msg_color, row_bg) = if is_err {
                        (CRIT, CRIT_INK, CRIT_LOG_BG)
                    } else if is_warn {
                        (WARN, WARN_INK, rgb(0x00000000))
                    } else {
                        (TEXT_FAINT, TEXT_TERTIARY, rgb(0x00000000))
                    };

                    div()
                        .flex()
                        .gap(px(8.0))
                        .px(px(12.0))
                        .py(px(2.0))
                        .bg(row_bg)
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(
                            div()
                                .text_color(TEXT_FAINTER)
                                .flex_none()
                                .child(entry.timestamp_formatted.clone()),
                        )
                        .child(
                            div()
                                .w(px(40.0))
                                .flex_none()
                                .font_weight(FontWeight::BOLD)
                                .text_color(lvl_color)
                                .child(entry.priority.label()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .text_color(msg_color)
                                .child(entry.message.clone()),
                        )
                })),
        )
        // Inline terminal prompt hatch
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .font_family("JetBrains Mono")
                .text_size(px(11.0))
                .child(
                    div()
                        .text_color(OK)
                        .child("root@edge-01"),
                )
                .child(
                    div()
                        .text_color(TEXT_FAINTER)
                        .child(":~#"),
                )
                .child(
                    div()
                        .text_color(TEXT_SECONDARY)
                        .child("journalctl -f"),
                )
                .child(
                    div()
                        .w(px(7.0))
                        .h(px(14.0))
                        .bg(TEXT_PRIMARY),
                ),
        )
}
