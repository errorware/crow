use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::config::DiffKind;
use crate::components::icons::{TablerIcon, tabler_icon};

pub fn pending_diff_rail(app_data: &CrowApp, app: Entity<CrowApp>) -> impl IntoElement {
    let sel_file = &app_data.selected_managed_file;
    let file_state = app_data.config_file_states.get(sel_file);
    let app_revert = app.clone();
    let app_apply = app.clone();

    let (add_count, del_count, diff_lines, is_modified) = if let Some(st) = file_state {
        let (a, d) = st.diff_stats();
        let diff = st.diff();
        let modified = st.is_modified();
        (a, d, diff, modified)
    } else {
        (0, 0, Vec::new(), false)
    };

    let header_stats = if is_modified {
        format!("+{} −{}", add_count, del_count)
    } else if sel_file == "pg_hba.conf" {
        "+2 −2".to_string() // Demo diff for pg_hba
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
                        .text_color(if is_modified || sel_file == "pg_hba.conf" { WARN } else { OK })
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
                    } else if sel_file == "pg_hba.conf" {
                        // Static sample diff for pg_hba when clean
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
                                    .child("--- /etc/postgresql/16/main/pg_hba.conf"),
                            )
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(1.0))
                                    .text_color(TEXT_FAINT)
                                    .child("+++ crow.staged (atomic)"),
                            )
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(1.0))
                                    .bg(DIFF_HUNK_BG)
                                    .text_color(TEXT_DIMMER)
                                    .child("@@ -9,2 +9,2 @@ IPv4 local connections"),
                            )
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(1.0))
                                    .bg(DIFF_DEL_BG)
                                    .text_color(CRIT_INK)
                                    .child("- host  all  all  10.0.4.0/24  trust"),
                            )
                            .child(
                                div()
                                    .px(px(12.0))
                                    .py(px(1.0))
                                    .bg(DIFF_ADD_BG)
                                    .text_color(OK_INK)
                                    .child("+ host  all  all  10.0.4.0/24  scram-sha-256"),
                            )
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
                            .child(div().child(format!("Working copy is in sync with baseline"))).into_any_element()
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
                                .child(div().flex_1().min_w(px(0.0)).text_color(OK).child(format!("Atomic rewrite with revision backup")))
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
                // Auto-Rollback Guard
                .child(
                    div()
                        .flex_none()
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .p(px(10.0))
                        .px(px(12.0))
                        .bg(rgb(0x0c0a0a))
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(7.0))
                                .child(
                                    tabler_icon(TablerIcon::ShieldCheck)
                                        .size(px(11.0))
                                        .text_color(OK),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .child("REVERSION GUARD ARMED"),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_TERTIARY)
                                .child("Full snapshots are created for every change. Instant single-click revert is always available."),
                        ),
                ),
        )
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
                        .bg(if is_modified || sel_file == "pg_hba.conf" { OK } else { TEXT_DIMMER })
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
