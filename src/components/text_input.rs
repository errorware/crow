use gpui_kit::*;
use crate::theme::*;

/// Render a terminal-style text input box adhering to the Obsidian Edge aesthetic.
/// Features:
/// - I-beam cursor styling on hover (`cursor_text()`)
/// - High-contrast focus border (`OK` emerald) and subtle background elevation on focus
/// - Dedicated terminal insertion caret bar (`OK` emerald)
/// - Ghost placeholder styling (`TEXT_FAINTER`) when value is empty
pub fn terminal_text_input(
    id: impl Into<ElementId>,
    value: &str,
    placeholder: &str,
    is_focused: bool,
    is_password: bool,
) -> Stateful<Div> {
    terminal_text_input_styled(id, value, placeholder, is_focused, is_password, 32.0, 12.0)
}

/// Render a terminal text input with custom height and text size.
pub fn terminal_text_input_styled(
    id: impl Into<ElementId>,
    value: &str,
    placeholder: &str,
    is_focused: bool,
    is_password: bool,
    height_px: f32,
    text_size_px: f32,
) -> Stateful<Div> {
    let caret_height = (height_px * 0.48).round().max(12.0);

    let display_text = if is_password {
        "•".repeat(value.len())
    } else {
        value.to_string()
    };

    let inner_content = if display_text.is_empty() {
        if is_focused {
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .w(px(2.0))
                        .h(px(caret_height))
                        .bg(OK)
                        .mr(px(3.0)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_FAINTER)
                        .child(placeholder.to_string()),
                )
        } else {
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_FAINTER)
                        .child(placeholder.to_string()),
                )
        }
    } else {
        div()
            .flex()
            .items_center()
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(text_size_px))
                    .text_color(TEXT_MAX)
                    .child(display_text),
            )
            .children(if is_focused {
                Some(
                    div()
                        .w(px(2.0))
                        .h(px(caret_height))
                        .bg(OK)
                        .ml(px(2.0)),
                )
            } else {
                None
            })
    };

    div()
        .id(id)
        .h(px(height_px))
        .px(px(10.0))
        .bg(if is_focused { BG_ROW_HOVER } else { BG_APP })
        .border_1()
        .border_color(if is_focused { OK } else { BORDER_DEFAULT })
        .cursor_text()
        .flex()
        .items_center()
        .child(inner_content)
}
