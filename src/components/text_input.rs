use gpui_kit::*;
use crate::theme::*;

/// Render a terminal-style text input box adhering to the Obsidian Edge aesthetic.
/// Features:
/// - I-beam cursor styling on hover (`cursor_text()`)
/// - High-contrast focus border (`OK` emerald) and subtle background elevation on focus
/// - Dedicated blinking terminal insertion caret (`OK` emerald)
/// - Evident text selection highlighting with translucent cyan background
/// - Ghost placeholder styling (`TEXT_FAINTER`) when value is empty
pub fn terminal_text_input(
    id: impl Into<ElementId>,
    value: &str,
    placeholder: &str,
    is_focused: bool,
    is_password: bool,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    cursor_visible: bool,
) -> Stateful<Div> {
    terminal_text_input_styled(
        id,
        value,
        placeholder,
        is_focused,
        is_password,
        32.0,
        12.0,
        cursor_pos,
        selection,
        cursor_visible,
    )
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
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    cursor_visible: bool,
) -> Stateful<Div> {
    let caret_height = (height_px * 0.48).round().max(12.0);

    let chars: Vec<char> = if is_password {
        "•".repeat(value.len()).chars().collect()
    } else {
        value.chars().collect()
    };
    let len = chars.len();

    let inner_content = if !is_focused {
        if len == 0 {
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
        } else {
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_MAX)
                        .child(chars.iter().collect::<String>()),
                )
        }
    } else if len == 0 {
        // Focused and empty: blinking cursor at col 0, followed by placeholder
        div()
            .flex()
            .items_center()
            .child(
                div()
                    .w(px(2.0))
                    .h(px(caret_height))
                    .bg(if cursor_visible { OK } else { hex_rgba(0, 0.0) })
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
        // Focused with text: render selection or cursor
        let active_sel = selection.filter(|(s, e)| s < e && *s < len);
        if let Some((s_raw, e_raw)) = active_sel {
            let s = s_raw.min(len);
            let e = e_raw.min(len);
            let before: String = chars[..s].iter().collect();
            let selected: String = chars[s..e].iter().collect();
            let after: String = chars[e..].iter().collect();

            let mut row = div().flex().items_center();
            if !before.is_empty() {
                row = row.child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_MAX)
                        .child(before),
                );
            }
            row = row.child(
                div()
                    .bg(hex_rgba(0x38bdf8, 0.35)) // Translucent vibrant cyan highlight
                    .rounded(px(2.0))
                    .px(px(2.0))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(text_size_px))
                            .font_weight(FontWeight::BOLD)
                            .text_color(TEXT_MAX)
                            .child(selected),
                    ),
            );
            if !after.is_empty() {
                row = row.child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_MAX)
                        .child(after),
                );
            }
            if cursor_visible {
                row = row.child(
                    div()
                        .w(px(2.0))
                        .h(px(caret_height))
                        .bg(OK)
                        .ml(px(1.0)),
                );
            }
            row
        } else {
            let cur = cursor_pos.min(len);
            let before: String = chars[..cur].iter().collect();
            let after: String = chars[cur..].iter().collect();

            let mut row = div().flex().items_center();
            if !before.is_empty() {
                row = row.child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_MAX)
                        .child(before),
                );
            }
            row = row.child(
                div()
                    .w(px(2.0))
                    .h(px(caret_height))
                    .bg(if cursor_visible { OK } else { hex_rgba(0, 0.0) })
                    .mx(px(0.5)),
            );
            if !after.is_empty() {
                row = row.child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(text_size_px))
                        .text_color(TEXT_MAX)
                        .child(after),
                );
            }
            row
        }
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

/// Centralized keyboard handler for terminal text inputs.
/// Handles:
/// - Cmd+A / Ctrl+A: Select All
/// - Cmd+C / Ctrl+C: Copy selection or full text to clipboard
/// - Cmd+X / Ctrl+X: Cut selection or full text to clipboard
/// - Cmd+V / Ctrl+V: Paste from clipboard replacing selection or inserting at cursor
/// - Left / Right Arrow (with Shift for range expansion, Cmd for line start/end)
/// - Home / End
/// - Backspace / Delete
/// - Printable character typing (replacing selection if active)
pub fn handle_text_key_event(
    text: &mut String,
    cursor: &mut usize,
    selection: &mut Option<(usize, usize)>,
    ev: &KeyDownEvent,
) -> bool {
    let is_mod = ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control;
    let is_shift = ev.keystroke.modifiers.shift;
    let key = ev.keystroke.key.to_lowercase();
    let mut chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    // 1. Select All (Cmd+A / Ctrl+A)
    if is_mod && key == "a" {
        if len > 0 {
            *selection = Some((0, len));
            *cursor = len;
        }
        return true;
    }

    // 2. Copy (Cmd+C / Ctrl+C)
    if is_mod && key == "c" {
        let copy_str = if let Some((s, e)) = *selection {
            let s = s.min(len);
            let e = e.min(len);
            chars[s..e].iter().collect::<String>()
        } else {
            text.clone()
        };
        if !copy_str.is_empty() {
            crate::keys::copy_to_clipboard_system(&copy_str);
        }
        return true;
    }

    // 3. Cut (Cmd+X / Ctrl+X)
    if is_mod && key == "x" {
        if let Some((s, e)) = *selection {
            let s = s.min(len);
            let e = e.min(len);
            let cut_str = chars[s..e].iter().collect::<String>();
            crate::keys::copy_to_clipboard_system(&cut_str);
            chars.drain(s..e);
            *text = chars.into_iter().collect();
            *cursor = s;
            *selection = None;
        } else if len > 0 {
            crate::keys::copy_to_clipboard_system(text);
            text.clear();
            *cursor = 0;
            *selection = None;
        }
        return true;
    }

    // 4. Paste (Cmd+V / Ctrl+V)
    if is_mod && key == "v" {
        let clip_text = crate::keys::read_clipboard_system();
        if let Some(paste) = clip_text {
            let sanitized: String = paste.chars().filter(|c| *c != '\n' && *c != '\r').collect();
            if !sanitized.is_empty() {
                if let Some((s, e)) = *selection {
                    let s = s.min(len);
                    let e = e.min(len);
                    chars.drain(s..e);
                    for (idx, ch) in sanitized.chars().enumerate() {
                        chars.insert(s + idx, ch);
                    }
                    *cursor = s + sanitized.chars().count();
                } else {
                    let cur = (*cursor).min(len);
                    for (idx, ch) in sanitized.chars().enumerate() {
                        chars.insert(cur + idx, ch);
                    }
                    *cursor = cur + sanitized.chars().count();
                }
                *text = chars.into_iter().collect();
                *selection = None;
            }
        }
        return true;
    }

    // 5. Left Arrow
    if key == "left" {
        let cur = (*cursor).min(len);
        if is_shift {
            let anchor = match *selection {
                Some((s, e)) => if cur == s { e } else { s },
                None => cur,
            };
            let new_cur = if is_mod { 0 } else { cur.saturating_sub(1) };
            let (start, end) = if new_cur <= anchor { (new_cur, anchor) } else { (anchor, new_cur) };
            if start == end {
                *selection = None;
            } else {
                *selection = Some((start, end));
            }
            *cursor = new_cur;
        } else {
            if is_mod {
                *cursor = 0;
            } else if let Some((s, _e)) = *selection {
                *cursor = s;
            } else {
                *cursor = cur.saturating_sub(1);
            }
            *selection = None;
        }
        return true;
    }

    // 6. Right Arrow
    if key == "right" {
        let cur = (*cursor).min(len);
        if is_shift {
            let anchor = match *selection {
                Some((s, e)) => if cur == e { s } else { e },
                None => cur,
            };
            let new_cur = if is_mod { len } else { (cur + 1).min(len) };
            let (start, end) = if new_cur <= anchor { (new_cur, anchor) } else { (anchor, new_cur) };
            if start == end {
                *selection = None;
            } else {
                *selection = Some((start, end));
            }
            *cursor = new_cur;
        } else {
            if is_mod {
                *cursor = len;
            } else if let Some((_s, e)) = *selection {
                *cursor = e;
            } else {
                *cursor = (cur + 1).min(len);
            }
            *selection = None;
        }
        return true;
    }

    // 7. Home / End
    if key == "home" {
        *cursor = 0;
        *selection = None;
        return true;
    }
    if key == "end" {
        *cursor = len;
        *selection = None;
        return true;
    }

    // 8. Escape: clear selection if active
    if key == "escape" && selection.is_some() {
        *selection = None;
        return true;
    }

    // 9. Backspace
    if key == "backspace" {
        if let Some((s, e)) = *selection {
            let s = s.min(len);
            let e = e.min(len);
            chars.drain(s..e);
            *text = chars.into_iter().collect();
            *cursor = s;
            *selection = None;
        } else if *cursor > 0 && len > 0 {
            let cur = (*cursor).min(len);
            chars.remove(cur - 1);
            *text = chars.into_iter().collect();
            *cursor = cur - 1;
        }
        return true;
    }

    // 10. Delete (Forward delete)
    if key == "delete" {
        if let Some((s, e)) = *selection {
            let s = s.min(len);
            let e = e.min(len);
            chars.drain(s..e);
            *text = chars.into_iter().collect();
            *cursor = s;
            *selection = None;
        } else if *cursor < len {
            chars.remove(*cursor);
            *text = chars.into_iter().collect();
        }
        return true;
    }

    // 11. Character Typing (when not command/ctrl)
    if !is_mod {
        let char_to_insert = ev.keystroke.key_char.as_deref().or(if ev.keystroke.key.chars().count() == 1 {
            Some(ev.keystroke.key.as_str())
        } else {
            None
        });
        if let Some(c_str) = char_to_insert {
            if let Some((s, e)) = *selection {
                let s = s.min(len);
                let e = e.min(len);
                chars.drain(s..e);
                for (idx, ch) in c_str.chars().enumerate() {
                    chars.insert(s + idx, ch);
                }
                *cursor = s + c_str.chars().count();
            } else {
                let cur = (*cursor).min(len);
                for (idx, ch) in c_str.chars().enumerate() {
                    chars.insert(cur + idx, ch);
                }
                *cursor = cur + c_str.chars().count();
            }
            *text = chars.into_iter().collect();
            *selection = None;
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{KeyDownEvent, Keystroke, Modifiers};

    fn make_event(key: &str, is_mod: bool, is_shift: bool) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke {
                modifiers: Modifiers {
                    platform: is_mod,
                    control: false,
                    alt: false,
                    shift: is_shift,
                    function: false,
                },
                key: key.to_string(),
                key_char: if key.chars().count() == 1 { Some(key.to_string()) } else { None },
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[test]
    fn test_typing_and_cursor() {
        let mut text = String::new();
        let mut cursor = 0;
        let mut selection = None;

        let ev = make_event("a", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "a");
        assert_eq!(cursor, 1);
        assert_eq!(selection, None);

        let ev = make_event("b", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "ab");
        assert_eq!(cursor, 2);
    }

    #[test]
    fn test_select_all_and_replace() {
        let mut text = "hello".to_string();
        let mut cursor = 5;
        let mut selection = None;

        // Cmd+A
        let ev = make_event("a", true, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(selection, Some((0, 5)));

        // Type 'x' replaces selection
        let ev = make_event("x", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "x");
        assert_eq!(cursor, 1);
        assert_eq!(selection, None);
    }

    #[test]
    fn test_backspace_with_and_without_selection() {
        let mut text = "hello".to_string();
        let mut cursor = 5;
        let mut selection = None;

        // Backspace without selection
        let ev = make_event("backspace", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "hell");
        assert_eq!(cursor, 4);

        // Select "el"
        selection = Some((1, 3));
        let ev = make_event("backspace", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "hl");
        assert_eq!(cursor, 1);
        assert_eq!(selection, None);
    }

    #[test]
    fn test_delete_forward_with_and_without_selection() {
        let mut text = "crow".to_string();
        let mut cursor = 0;
        let mut selection = None;

        // Delete 'c'
        let ev = make_event("delete", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "row");
        assert_eq!(cursor, 0);

        // Select "ow"
        selection = Some((1, 3));
        let ev = make_event("delete", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(text, "r");
        assert_eq!(cursor, 1);
        assert_eq!(selection, None);
    }

    #[test]
    fn test_arrow_keys_and_shift_selection() {
        let mut text = "fleet".to_string();
        let mut cursor = 5;
        let mut selection = None;

        // Left arrow moves cursor left
        let ev = make_event("left", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(cursor, 4);
        assert_eq!(selection, None);

        // Shift+Left expands selection leftwards
        let ev = make_event("left", false, true);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(cursor, 3);
        assert_eq!(selection, Some((3, 4)));

        // Shift+Left again
        let ev = make_event("left", false, true);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(cursor, 2);
        assert_eq!(selection, Some((2, 4)));

        // Right arrow clears selection and sets cursor
        let ev = make_event("right", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(cursor, 4);
        assert_eq!(selection, None);
    }

    #[test]
    fn test_home_end_escape() {
        let mut text = "server".to_string();
        let mut cursor = 3;
        let mut selection = Some((1, 4));

        let ev = make_event("escape", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(selection, None);

        let ev = make_event("home", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(cursor, 0);

        let ev = make_event("end", false, false);
        assert!(handle_text_key_event(&mut text, &mut cursor, &mut selection, &ev));
        assert_eq!(cursor, 6);
    }
}
