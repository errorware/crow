use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use gpui_kit::component::input::{Editor, EditorState};
use crate::config::ConfigFileState;

/// Text editor for config files without a crow-config plugin. `editor` holds
/// the live text; every change is mirrored into `state` by the app, so the
/// diff rail, staging and write-back work off `state` as usual.
pub fn raw_config_editor(
    state: &ConfigFileState,
    editor: &Entity<EditorState>,
    can_structure: bool,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_structured = app.clone();
    let app_revert = app.clone();
    let app_stage = app.clone();
    let app_hist = app.clone();

    let is_modified = state.is_modified();
    let (add_count, del_count) = state.diff_stats();
    let line_count = state.current_content.lines().count().max(1);
    let read_only = state.write_blocked.is_some();

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
                        .child(t!("config-raw-editor-traditional-editor")),
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
                            t!("config-raw-editor-edited", add_count = add_count, del_count = del_count)
                        } else {
                            t!("config-raw-editor-v-baseline", active_revision = state.active_revision)
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
                        .child(t!("config-raw-editor-author"))
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
                        .child(t!("config-raw-editor-history", count = state.revisions.len()))
                )
                // Back to the plugin's structured editor
                .children(can_structure.then(|| {
                    let f = state.filename.clone();
                    div()
                        .id("btn-raw-structured")
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(hex_rgb(0x8ab4ff))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .px(px(8.0))
                        .py(px(3.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            let f = f.clone();
                            app_structured.update(cx, |this, cx| {
                                this.configs.text_mode.remove(&f);
                                cx.notify();
                            });
                        })
                        .child(t!("config-raw-editor-structured-view"))
                }))
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
                            .child(t!("config-raw-editor-revert-baseline")),
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
                            .child(t!("config-raw-editor-stage-version")),
                    )
                } else {
                    None
                }),
        )
        // 2. Editor body: real editing, line numbers, search and scrollbars.
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .bg(BG_APP)
                .child(
                    Editor::new(editor)
                        .h_full()
                        .appearance(false)
                        .bordered(false)
                        .readonly(read_only)
                        .font_family(FONT_MONO)
                        .text_size(px(11.5)),
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
                        .child(t!("config-raw-editor-lines", line_count = line_count))
                        .child(t!("config-raw-editor-bytes", count = state.current_content.len()))
                        .child(t!("config-raw-editor-utf-8"))
                        .child(t!("config-raw-editor-unix-lf")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(t!("config-raw-editor-revisions", count = state.revisions.len()))
                        .children(read_only.then(|| div().text_color(WARN).child(t!("config-raw-editor-read-only"))))
                        .child(if is_modified {
                            div().text_color(WARN).child(t!("config-raw-editor-unsaved-staged-changes"))
                        } else {
                            div().text_color(OK).child(t!("config-raw-editor-clean-baseline"))
                        }),
                ),
        )
}
