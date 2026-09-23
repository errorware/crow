use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::config::{highlight_config_line, ConfigFileState};

pub fn raw_config_editor(
    state: &ConfigFileState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_revert = app.clone();
    let app_stage = app.clone();
    let app_edit = app.clone();
    let app_hist = app.clone();

    let is_modified = state.is_modified();
    let (add_count, del_count) = state.diff_stats();
    let lines: Vec<&str> = state.current_content.lines().collect();
    let line_count = lines.len().max(1);

    div()
        .id("raw-config-editor")
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Editor Header Toolbar
        .child(
            div()
                .h(px(36.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(state.filename.clone()),
                )
                .child(
                    div()
                        .bg(hex_rgba(0xffffff, 0.06))
                        .text_color(TEXT_MUTED)
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .child("TRADITIONAL EDITOR"),
                )
                .child(
                    div()
                        .bg(if is_modified { WARN_BG } else { OK_BG })
                        .text_color(if is_modified { WARN } else { OK })
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .child(if is_modified {
                            format!("EDITED (+{} -{})", add_count, del_count)
                        } else {
                            format!("v{} (BASELINE)", state.active_revision)
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .child("Author:")
                        .child(
                            div()
                                .text_color(TEXT_SECONDARY)
                                .font_weight(FontWeight::MEDIUM)
                                .child(state.last_author().to_string())
                        )
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-raw-history")
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_TERTIARY)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .px(px(8.0))
                        .py(px(3.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_hist.update(cx, |this, cx| {
                                this.toggle_config_history(cx);
                            });
                        })
                        .child(format!("HISTORY · {}", state.revisions.len()))
                )
                // Quick Line Tweak Action (convenience shortcut for editing in GUI)
                .child({
                    let fn_clone = state.filename.clone();
                    div()
                        .id("btn-raw-edit-tweak")
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_SECONDARY)
                        .on_click(move |_ev, _window, cx| {
                            let f = fn_clone.clone();
                            app_edit.update(cx, |this, cx| {
                                this.configs.toggle_sample_edit(&f); cx.notify();
                            });
                        })
                        .child(if is_modified { "SIMULATE EDIT ↺" } else { "+ TEST EDIT" })
                })
                // Revert button
                .children(if is_modified {
                    let fn_clone = state.filename.clone();
                    Some(
                        div()
                            .id("btn-raw-revert")
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(hex_rgba(0xef4444, 0.15))
                            .border_1()
                            .border_color(hex_rgba(0xef4444, 0.4))
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgba(0xef4444, 0.25)))
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(CRIT)
                            .on_click(move |_ev, _window, cx| {
                                let f = fn_clone.clone();
                                app_revert.update(cx, |this, cx| {
                                    this.revert_managed_config(&f, cx);
                                });
                            })
                            .child("REVERT BASELINE"),
                    )
                } else {
                    None
                })
                // Stage Revision button
                .children(if is_modified {
                    let fn_clone = state.filename.clone();
                    Some(
                        div()
                            .id("btn-raw-stage")
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(hex_rgba(0x10b981, 0.18))
                            .border_1()
                            .border_color(hex_rgba(0x10b981, 0.4))
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgba(0x10b981, 0.3)))
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(OK)
                            .on_click(move |_ev, _window, cx| {
                                let f = fn_clone.clone();
                                app_stage.update(cx, |this, cx| {
                                    this.stage_config_version(&f, "Updated via traditional editor", cx);
                                });
                            })
                            .child("STAGE VERSION"),
                    )
                } else {
                    None
                }),
        )
        // 2. Editor Gutter + Code Body
        .child(
            div()
                .id("raw-editor-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .bg(BG_APP)
                // Line Number Gutter
                .child(
                    div()
                        .w(px(44.0))
                        .flex_none()
                        .py(px(8.0))
                        .border_r_1()
                        .border_color(BORDER_ROW)
                        .bg(hex_rgba(0x000000, 0.15))
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_FAINTER)
                        .flex()
                        .flex_col()
                        .children((1..=line_count).map(|num| {
                            div()
                                .h(px(20.0))
                                .pr(px(10.0))
                                .text_right()
                                .child(num.to_string())
                        })),
                )
                // Code Lines View
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .py(px(8.0))
                        .px(px(12.0))
                        .font_family(FONT_MONO)
                        .text_size(px(11.5))
                        .flex()
                        .flex_col()
                        .children(lines.iter().enumerate().map(|(idx, line)| {
                            let tokens = highlight_config_line(line, &state.filename);

                            div()
                                .id(ElementId::NamedInteger("editor-line".into(), idx as u64))
                                .h(px(20.0))
                                .flex()
                                .items_center()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .children(tokens.into_iter().map(|tok| {
                                    let mut el = div()
                                        .text_color(tok.color)
                                        .child(tok.text);
                                    if tok.is_bold {
                                        el = el.font_weight(FontWeight::BOLD);
                                    }
                                    el
                                }))
                        })),
                ),
        )
        // 3. Status Bar
        .child(
            div()
                .h(px(24.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_FAINT)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(format!("{} lines", line_count))
                        .child(format!("{} bytes", state.current_content.len()))
                        .child("UTF-8")
                        .child("UNIX (LF)"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(format!("Revisions: {}", state.revisions.len()))
                        .child(if is_modified {
                            div().text_color(WARN).child("● Unsaved staged changes")
                        } else {
                            div().text_color(OK).child("✓ Clean baseline")
                        }),
                ),
        )
}
