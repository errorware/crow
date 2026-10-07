use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::config::{compute_unified_diff, DiffKind};
use gpui_kit::prelude::FluentBuilder as _;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::views::config::state::ConfigsState;

/// The pending diff, history and apply plan for `file` (the Config page's
/// selected file, or `crontab` on the Cron page).
pub fn pending_diff_rail(configs: &ConfigsState, file: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let sel_file = &file.to_string();
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
    let notice = match (&configs.save_error, file_state.and_then(|st| st.write_blocked.as_ref()), &configs.history_error) {
        (Some(err), _, _) => Some((format!("SAVE FAILED · {err}"), CRIT, 0xef4444)),
        (None, Some(reason), _) => Some((format!("READ-ONLY · {reason}"), WARN, 0xf59e0b)),
        (None, None, Some(err)) => Some((format!("HISTORY · {err}"), WARN, 0xf59e0b)),
        (None, None, None) => None,
    };

    // A previewed revision (ERR-73) takes the diff area: what restoring it
    // would change on the host.
    let preview = configs.previewed_revision().zip(file_state).map(|(rev, st)| {
        let lines = compute_unified_diff(&st.baseline_content, rev.content.as_deref().unwrap_or_default());
        (rev.version, lines)
    });
    let (title, header_stats, header_color) = match &preview {
        Some((v, lines)) => {
            let add = lines.iter().filter(|l| l.kind == DiffKind::Addition).count();
            let del = lines.iter().filter(|l| l.kind == DiffKind::Deletion).count();
            (format!("RESTORE v{v}?"), if add + del == 0 { "same as host".to_string() } else { format!("+{add} −{del}") }, TEXT_SECONDARY)
        }
        None if is_modified => ("PENDING DIFF".to_string(), format!("+{} −{}", add_count, del_count), WARN),
        None => ("PENDING DIFF".to_string(), "clean".to_string(), OK),
    };
    let plan = apply_plan(configs, sel_file, file_state);
    let baseline_rev_id = file_state.and_then(|st| configs.baselines.get(st.path.to_string_lossy().as_ref())).map(|b| b.baseline.revision_id.clone());

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
                        .child(title),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(header_color)
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
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                // Dynamic Diff Block if file is modified
                .child(
                    if let Some((v, lines)) = preview {
                        let sel = sel_file.clone();
                        diff_block(format!("--- on host/{}", sel_file), format!("+++ v{v}/{}", sel_file), lines)
                            .child(
                                div()
                                    .id("btn-close-preview")
                                    .mx(px(12.0))
                                    .mt(px(8.0))
                                    .font_family(FONT_MONO)
                                    .text_size(px(9.5))
                                    .text_color(TEXT_DIM)
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(TEXT_PRIMARY))
                                    .on_click({
                                        let app = app.clone();
                                        move |_ev, _window, cx| {
                                            let f = sel.clone();
                                            app.update(cx, |this, cx| {
                                                this.configs.toggle_preview(&f, v);
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child("← back to pending diff"),
                            )
                            .into_any_element()
                    } else if is_modified && !diff_lines.is_empty() {
                        diff_block(format!("--- baseline/{}", sel_file), format!("+++ staged/{} (unsaved)", sel_file), diff_lines).into_any_element()
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
                            .child(div().child("No unsaved edits"))
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
                                .child("ON SAVE"),
                        )
                        .children(plan.into_iter().map(|(ok, text)| {
                            let color = if ok { OK } else { WARN };
                            div()
                                .flex()
                                .items_start()
                                .gap(px(9.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(div().w(px(12.0)).flex_none().text_color(color).child(if ok { "✓" } else { "!" }))
                                .child(div().flex_1().min_w(px(0.0)).text_color(if ok { TEXT_SECONDARY } else { WARN }).child(text))
                        })),
                )
                .children(file_state.filter(|st| st.read_from_host).map(|st| baseline_section(configs, sel_file, st, app.clone())))
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
                                        .child(if active_rev == 0 { "on host: not recorded".to_string() } else { format!("On host: v{}", active_rev) }),
                                ),
                        )
                        .children(revisions.into_iter().rev().map(|rev| {
                            let is_current = rev.version == active_rev;
                            let restorable = rev.content.is_some();
                            let outside = rev.source == crate::config::history::SOURCE_OBSERVED && rev.message == "Changed outside Crow";
                            let previewing = configs.preview_revision.as_ref() == Some(&(sel_file.clone(), rev.version));
                            let app_preview = app.clone();
                            let file_preview = sel_file.clone();
                            let is_baseline = baseline_rev_id.as_deref() == Some(rev.id.as_str()) && !rev.id.is_empty();
                            let app_rollback = app.clone();
                            let fn_str = sel_file.clone();
                            let v = rev.version;

                            div()
                                .id(ElementId::NamedInteger("rev-card".into(), v as u64))
                                .p(px(8.0))
                                .bg(if previewing { BG_ROW_SELECTED } else if is_current { hex_rgba(0x3ecf6e, 0.05) } else { hex_rgba(0xffffff, 0.02) })
                                .border_1()
                                .border_color(if previewing { BORDER_CONTROL_SEL } else if is_current { hex_rgba(0x3ecf6e, 0.3) } else { BORDER_DEFAULT })
                                .when(restorable && !is_current, |d| {
                                    d.cursor_pointer().hover(|s| s.border_color(TEXT_DIM)).on_click(move |_ev, _window, cx| {
                                        let f = file_preview.clone();
                                        app_preview.update(cx, |this, cx| {
                                            this.configs.toggle_preview(&f, v);
                                            cx.notify();
                                        });
                                    })
                                })
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
                                        .children(if !is_current && !restorable {
                                            Some(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.0))
                                                    .text_color(TEXT_FAINT)
                                                    .child("HASH ONLY")
                                                    .into_any_element()
                                            )
                                        } else if !is_current {
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
                                                        cx.stop_propagation();
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
                                        )
                                        .child(div().flex_1())
                                        .children(if is_baseline {
                                            Some(div().text_color(OK).font_weight(FontWeight::BOLD).child("BASELINE").into_any_element())
                                        } else if restorable && !rev.id.is_empty() {
                                            let app_pin = app.clone();
                                            let f = sel_file.clone();
                                            Some(
                                                div()
                                                    .id(ElementId::NamedInteger("btn-pin-baseline".into(), v as u64))
                                                    .px(px(6.0))
                                                    .py(px(1.0))
                                                    .text_color(TEXT_DIM)
                                                    .cursor_pointer()
                                                    .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                                    .on_click(move |_ev, _window, cx| {
                                                        cx.stop_propagation();
                                                        let f = f.clone();
                                                        app_pin.update(cx, |this, cx| this.pin_config_baseline(&f, v, cx));
                                                    })
                                                    .child("PIN AS BASELINE")
                                                    .into_any_element(),
                                            )
                                        } else {
                                            None
                                        }),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(if outside { WARN } else { TEXT_TERTIARY })
                                        .child(if restorable { rev.message } else { format!("{} · {}", rev.message, crate::config::history::MISSING_CONTENT) }),
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

/// A unified diff with its two header lines.
fn diff_block(old: String, new: String, lines: Vec<crate::config::ConfigDiffLine>) -> Div {
    div()
        .flex_none()
        .border_b_1()
        .border_color(BORDER_PANEL)
        .py(px(8.0))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .flex()
        .flex_col()
        .child(div().px(px(12.0)).py(px(1.0)).text_color(TEXT_FAINT).child(old))
        .child(div().px(px(12.0)).py(px(1.0)).text_color(TEXT_FAINT).child(new))
        .children(lines.into_iter().map(|line| {
            let (bg, fg) = match line.kind {
                DiffKind::Hunk => (DIFF_HUNK_BG, TEXT_DIMMER),
                DiffKind::Deletion => (DIFF_DEL_BG, CRIT_INK),
                DiffKind::Addition => (DIFF_ADD_BG, OK_INK),
                DiffKind::Context => (rgb(0x00000000), TEXT_SECONDARY),
            };
            div().px(px(12.0)).py(px(1.5)).bg(bg).text_color(fg).child(line.text)
        }))
}

/// What saving the selected file actually does, as (reassuring, text) lines:
/// how it's written, what checks it, and what history keeps.
fn apply_plan(configs: &ConfigsState, file: &str, st: Option<&crate::config::ConfigFileState>) -> Vec<(bool, String)> {
    let Some(st) = st else { return Vec::new() };
    let mut plan = vec![(true, "Written atomically: temp file, then rename".to_string())];
    match configs.structured_format(file) {
        Some(format) => {
            plan.push((true, "Structured edits keep comments and layout".to_string()));
            let checks = crate::config::plugins::file_validators(format);
            if checks.is_empty() {
                plan.push((false, "No host validator for this format".to_string()));
            } else {
                let cmds = checks.iter().map(|c| format!("`{}`", c.replace("{file}", "<file>"))).collect::<Vec<_>>().join(", ");
                plan.push((true, format!("Checked on the host first: {cmds}")));
            }
        }
        None => plan.push((false, "Plain text: nothing checks it before it's written".to_string())),
    }
    plan.push(match (st.read_from_host, configs.history_sealed) {
        (false, _) => (false, "Not kept in history: Crow didn't read this file from the host".to_string()),
        (true, Some(true)) => (true, "Every version kept in history, encrypted".to_string()),
        (true, Some(false)) => (false, "History keeps hashes only: no encryption key available".to_string()),
        (true, None) => (false, "History not loaded yet".to_string()),
    });
    plan
}

/// This server's copy against the baseline in force for it, if any (ERR-74).
fn baseline_section(configs: &ConfigsState, file: &str, st: &crate::config::ConfigFileState, app: Entity<CrowApp>) -> impl IntoElement {
    use crate::app::configs::scope_label;
    use crate::config::drift::Drift;
    let path = st.path.to_string_lossy().into_owned();
    let applied = configs.baselines.get(&path);
    let button = |id: &'static str, label: &'static str| {
        div()
            .id(id)
            .px(px(8.0))
            .py(px(3.0))
            .border_1()
            .border_color(BORDER_DEFAULT)
            .text_size(px(9.5))
            .text_color(TEXT_SECONDARY)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
            .child(label)
    };
    let mut body = div().flex().flex_col().gap(px(5.0)).text_size(px(10.5));
    match applied {
        None => {
            body = body.child(div().text_color(TEXT_DIM).child(format!(
                "None pinned. PIN AS BASELINE on a revision makes it the known-good {} for {}; servers that differ show as drift.",
                st.filename,
                scope_label(&configs.baseline_scope)
            )));
        }
        Some(b) => {
            let drift = b.drift(&path, &st.baseline_content);
            let (ok, headline) = match &drift {
                Drift::Identical => (true, "Matches the baseline".to_string()),
                Drift::Cosmetic => (true, "Same settings as the baseline; only comments or layout differ".to_string()),
                Drift::Differs(_) => (false, "Drifted from the baseline".to_string()),
            };
            body = body
                .child(div().text_color(if ok { OK } else { WARN }).child(format!("{} {headline}", if ok { "✓" } else { "!" })))
                .children(match drift {
                    Drift::Differs(lines) => {
                        let more = lines.len().saturating_sub(8);
                        let mut rows: Vec<AnyElement> = lines.into_iter().take(8).map(|l| div().pl(px(14.0)).text_color(TEXT_SECONDARY).child(l).into_any_element()).collect();
                        if more > 0 {
                            rows.push(div().pl(px(14.0)).text_color(TEXT_FAINT).child(format!("… {more} more")).into_any_element());
                        }
                        rows
                    }
                    _ => Vec::new(),
                })
                .child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child(format!(
                    "pinned from {} for {} by {}",
                    b.from_server,
                    scope_label(&b.baseline.scope),
                    b.baseline.set_by
                )))
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .children((!ok && b.content.is_some()).then(|| {
                            let app = app.clone();
                            let file = file.to_string();
                            button("btn-load-baseline", "LOAD BASELINE INTO EDITOR")
                                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.load_baseline_into_editor(&file, cx)))
                        }))
                        .child({
                            let app = app.clone();
                            let (path, scope) = (path.clone(), b.baseline.scope.clone());
                            button("btn-unpin-baseline", "UNPIN")
                                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.unpin_config_baseline(&path, &scope, cx)))
                        }),
                );
        }
    }
    div()
        .flex_none()
        .border_b_1()
        .border_color(BORDER_PANEL)
        .p(px(10.0))
        .px(px(12.0))
        .flex()
        .flex_col()
        .gap(px(7.0))
        .font_family(FONT_MONO)
        .child(div().text_size(px(10.0)).font_weight(FontWeight::BOLD).text_color(TEXT_DIMMER).child("BASELINE"))
        .child(body)
}
