use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::config::DiffKind;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::views::config::state::ConfigsState;

pub fn pending_diff_rail(configs: &ConfigsState, app: Entity<CrowApp>) -> impl IntoElement {
    let sel_file = &configs.selected_file;
    let file_state = configs.states.get(sel_file);
    let app_revert = app.clone();
    let app_apply = app.clone();
    let default_author = crate::app::configs::default_author();

    let (add_count, del_count, diff_lines, is_modified, revisions, active_rev) = if let Some(st) = file_state {
        let (a, d) = st.diff_stats();
        let diff = st.diff();
        let modified = st.is_modified();
        (a, d, diff, modified, st.revisions.clone(), st.active_revision)
    } else {
        (0, 0, Vec::new(), false, Vec::new(), 1)
    };

    // A failed save outranks the standing read-only reason.
    let notice = match (&configs.save_error, file_state.and_then(|st| st.write_blocked.as_ref())) {
        (Some(err), _) => Some((format!("SAVE FAILED · {err}"), CRIT, 0xef4444)),
        (None, Some(reason)) => Some((format!("READ-ONLY · {reason}"), WARN, 0xf59e0b)),
        (None, None) => None,
    };

    let header_stats = if is_modified {
        format!("+{} −{}", add_count, del_count)
    } else {
        "clean".to_string()
    };

    div()
        .w(px(380.0))
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
                        .child("PENDING DIFF"),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(if is_modified { WARN } else { OK })
                        .child(header_stats),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child(format!("unified · {}", sel_file)),
                ),
        )
        // Author Attribution Banner
        .child(
            div()
                .flex_none()
                .p(px(8.0))
                .px(px(12.0))
                .bg(BG_SUBHEAD)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    tabler_icon(TablerIcon::Users)
                        .size(px(12.0))
                        .text_color(OK),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .child("Author:"),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(default_author.clone()),
                ),
        )
        // Scrollable content body
        .child(
            div()
                .id("pending-diff-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                // Dynamic Diff Block if file is modified
                .child(
                    if is_modified && !diff_lines.is_empty() {
                        div()
                            .flex_none()
                            .border_b_1()
                            .border_color(BORDER_PANEL)
                            .py(px(8.0))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(1.0))
                                    .text_color(TEXT_FAINT)
                                    .child(format!("--- baseline/{}", sel_file)),
                            )
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(1.0))
                                    .text_color(TEXT_FAINT)
                                    .child(format!("+++ staged/{} (unsaved)", sel_file)),
                            )
                            .children(diff_lines.into_iter().map(|line| {
                                let (bg, fg) = match line.kind {
                                    DiffKind::Hunk => (DIFF_HUNK_BG, TEXT_DIMMER),
                                    DiffKind::Deletion => (DIFF_DEL_BG, CRIT_INK),
                                    DiffKind::Addition => (DIFF_ADD_BG, OK_INK),
                                    DiffKind::Context => (rgb(0x00000000), TEXT_SECONDARY),
                                };

                                div()
                                    .px(px(12.0))
                                    .py(px(1.5))
                                    .bg(bg)
                                    .text_color(fg)
                                    .child(line.text)
                            }))
                            .into_any_element()
                    } else {
                        // Clean file notice
                        div()
                            .flex_none()
                            .border_b_1()
                            .border_color(BORDER_PANEL)
                            .py(px(16.0))
                            .px(px(12.0))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(OK)
                            .child(tabler_icon(TablerIcon::Check).size(px(14.0)).text_color(OK))
                            .child(div().child("Working copy is in sync with baseline"))
                            .into_any_element()
                    }
                )
                // Apply Plan Section
                .child(
                    div()
                        .flex_none()
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .p(px(10.0))
                        .px(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(7.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_DIMMER)
                                .child("APPLY PLAN & INTEGRITY"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(9.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(tabler_icon(TablerIcon::Check).size(px(12.0)).text_color(OK))
                                .child(div().flex_1().min_w(px(0.0)).text_color(OK).child("Atomic rewrite with revision backup"))
                                .child(div().flex_none().text_color(TEXT_FAINT).child("staged")),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(9.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(tabler_icon(TablerIcon::Check).size(px(12.0)).text_color(OK))
                                .child(div().flex_1().min_w(px(0.0)).text_color(OK).child("Lossless CST trivia & formatting preserved"))
                                .child(div().flex_none().text_color(TEXT_FAINT).child("100%")),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(9.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(div().w(px(12.0)).flex_none().text_color(TEXT_DIM).child("○"))
                                .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_DIM).child("Syntax verification before disk flush"))
                                .child(div().flex_none().text_color(TEXT_FAINT).child("ready")),
                        ),
                )
                // Revision History / Version Audit Log Section
                .child(
                    div()
                        .flex_none()
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .p(px(10.0))
                        .px(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIMMER)
                                        .child(format!("VERSION HISTORY ({} REVISIONS)", revisions.len())),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(OK)
                                        .child(format!("Active: v{}", active_rev)),
                                ),
                        )
                        .children(revisions.into_iter().rev().map(|rev| {
                            let is_current = rev.version == active_rev;
                            let app_rollback = app.clone();
                            let fn_str = sel_file.clone();
                            let v = rev.version;

                            div()
                                .p(px(8.0))
                                .bg(if is_current { hex_rgba(0x3ecf6e, 0.05) } else { hex_rgba(0xffffff, 0.02) })
                                .border_1()
                                .border_color(if is_current { hex_rgba(0x3ecf6e, 0.3) } else { BORDER_DEFAULT })
                                .rounded_sm()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(if is_current { OK } else { TEXT_PRIMARY })
                                                        .child(format!("v{}", rev.version)),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .text_color(TEXT_FAINT)
                                                        .child(rev.timestamp.clone()),
                                                ),
                                        )
                                        .children(if !is_current {
                                            Some(
                                                div()
                                                    .id(ElementId::NamedInteger("btn-rollback".into(), v as u64))
                                                    .px(px(6.0))
                                                    .py(px(2.0))
                                                    .bg(BG_CONTROL)
                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.0))
                                                    .text_color(TEXT_SECONDARY)
                                                    .rounded_sm()
                                                    .cursor_pointer()
                                                    .on_click(move |_ev, _window, cx| {
                                                        let f = fn_str.clone();
                                                        app_rollback.update(cx, |this, cx| {
                                                            this.rollback_config_revision(&f, v, cx);
                                                        });
                                                    })
                                                    .child("RESTORE")
                                                    .into_any_element()
                                            )
                                        } else {
                                            Some(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(OK)
                                                    .child("CURRENT")
                                                    .into_any_element()
                                            )
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(4.0))
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_MUTED)
                                        .child("by")
                                        .child(
                                            div()
                                                .text_color(TEXT_SECONDARY)
                                                .font_weight(FontWeight::MEDIUM)
                                                .child(rev.author),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_TERTIARY)
                                        .child(rev.message),
                                )
                        })),
                ),
        )
        .children(notice.map(|(text, color, tint)| {
            div()
                .flex_none()
                .px(px(12.0))
                .py(px(7.0))
                .bg(hex_rgba(tint, 0.08))
                .border_t_1()
                .border_color(hex_rgba(tint, 0.3))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .text_color(color)
                .child(text)
        }))
        // Action footer
        .child(
            div()
                .h(px(42.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .child({
                    let sel = sel_file.clone();
                    div()
                        .id("btn-rail-revert")
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(if is_modified { CRIT } else { TEXT_MUTED })
                        .border_1()
                        .border_color(if is_modified { hex_rgba(0xef4444, 0.4) } else { BORDER_KEY })
                        .bg(if is_modified { hex_rgba(0xef4444, 0.1) } else { hex_rgba(0, 0.0) })
                        .px(px(9.0))
                        .py(px(5.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            let s = sel.clone();
                            app_revert.update(cx, |this, cx| {
                                this.revert_managed_config(&s, cx);
                            });
                        })
                        .child("REVERT")
                })
                .child(div().flex_1())
                .child({
                    let sel = sel_file.clone();
                    div()
                        .id("btn-rail-apply")
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0a0a0c))
                        .bg(if is_modified { OK } else { TEXT_DIMMER })
                        .px(px(11.0))
                        .py(px(6.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgb(0x34d399)))
                        .on_click(move |_ev, _window, cx| {
                            let s = sel.clone();
                            app_apply.update(cx, |this, cx| {
                                this.stage_config_version(&s, "Staged via Pending Diff", cx);
                            });
                        })
                        .child("STAGE & APPLY")
                }),
        )
}
