use gpui_kit::component::scroll::ScrollableElement;
use std::collections::HashSet;
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::journal::{JournalBootScope, JournalEntry, JournalPriority, JournalStorageMode, JournalTimeRange};
use crate::keys::copy_to_clipboard_system;
use gpui_kit::component::input::{Input, InputState};
use crate::components::icons::{tabler_icon, TablerIcon};

pub mod retention_modal;
pub mod state;

pub use state::{AiPanelState, JournalState, JOURNAL_PRESETS};

pub fn logs_explorer_view(
    app: Entity<CrowApp>,
    search: Option<&Entity<InputState>>,
    journal: &JournalState,
    ai_provider: Option<String>,
) -> impl IntoElement {
    let app_clone = app.clone();

    // Compute error and warn counts
    let err_count = journal.entries.iter().filter(|e| e.priority.is_error()).count();
    let warn_count = journal.entries.iter().filter(|e| e.priority.is_warn()).count();
    let total_count = journal.entries.len();

    // Merge in Crow's own action markers (restarts, kills, reloads) so cause and
    // effect show up in the same stream, then filter/sort the combined timeline.
    let mut combined_entries: Vec<&JournalEntry> = journal.entries
        .iter()
        .chain(journal.action_markers.iter())
        .collect();
    combined_entries.sort_by_key(|e| e.timestamp_usec);

    // journalctl already applied every filter to the server's entries (unit,
    // priority as "this level and worse", PID, time, --grep as a regex), so
    // they're shown as returned. Only Crow's own action markers, which never
    // went through journalctl, are filtered here, with the same meanings.
    let marker_visible = |e: &JournalEntry| -> bool {
        if !journal.show_actions || journal.pid_filter.is_some() {
            return false;
        }
        // With a unit selected, keep the actions that touched that unit.
        if let Some(u) = journal.unit_filter.as_deref().filter(|u| *u != "ALL") {
            let base = u.trim_end_matches(".service");
            if !e.message.contains(base) {
                return false;
            }
        }
        if journal.severity_filter.is_some_and(|p| (e.priority as u8) > (p as u8)) {
            return false;
        }
        let q = journal.search.trim().to_lowercase();
        q.is_empty() || e.message.to_lowercase().contains(&q)
    };
    let filtered_entries: Vec<&JournalEntry> = combined_entries
        .into_iter()
        .filter(|e| !e.id.starts_with("crow-action_") || marker_visible(e))
        .collect();

    let filtered_count = filtered_entries.len();

    let export_text = filtered_entries
        .iter()
        .map(|e| {
            format!(
                "{} [{}] {}{}: {}",
                e.timestamp_formatted,
                e.priority.label(),
                clean_unit_display(&e.unit),
                e.pid.map(|p| format!("[{}]", p)).unwrap_or_default(),
                e.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    div()
        .relative()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Top Title & Global Controls Header
        .child(
            div()
                .h(px(38.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                // Left Title cluster
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("SYSTEMD JOURNAL LOGS"),
                        )
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(10.5))
                                .text_color(TEXT_DIMMER)
                                .child("journalctl -o json --output-fields=*"),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(1.5))
                                .bg(BG_CHIP)
                                .text_color(TEXT_SECONDARY)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .child(format!("{}/{} entries", filtered_count, total_count)),
                        ),
                )
                // Right controls: Live tail toggle, Retention boundaries, Clear
                .child({
                    let app_tail = app_clone.clone();
                    let app_clear = app_clone.clone();
                    let app_bound = app_clone.clone();
                    let export_snapshot = export_text.clone();
                    let is_tail = journal.live_tail;
                    let is_warn = journal.telemetry.is_volatile_warning
                        || journal.retention.storage != JournalStorageMode::Persistent;
                    let r = &journal.retention;
                    let bound_text = if is_warn {
                        "⚠ VOLATILE — LOST ON REBOOT".to_string()
                    } else {
                        let days = if r.max_retention_days == 0 { "no age limit".to_string() } else { format!("{}d", r.max_retention_days) };
                        let size = if r.system_max_use_mb == 0 { "no size cap".to_string() } else if r.system_max_use_mb >= 1024 { format!("{:.1}GB", r.system_max_use_mb as f32 / 1024.0) } else { format!("{}MB", r.system_max_use_mb) };
                        format!("{days} · {size} PERSISTENT")
                    };
                    let (bound_color, bound_bg) = if is_warn {
                        (CRIT, CRIT_BG)
                    } else {
                        (OK, OK_BG)
                    };

                    let app_ai = app_clone.clone();
                    let ai_open = journal.ai.open;
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        // AI explanation panel toggle
                        .child(
                            div()
                                .id("journal-ai-btn")
                                .px(px(9.0))
                                .py(px(3.5))
                                .border_1()
                                .border_color(if ai_open { hex_rgb(0xa78bfa) } else { BORDER_DEFAULT })
                                .bg(if ai_open { hex_rgba(0xa78bfa, 0.12) } else { hex_rgba(0, 0.0) })
                                .text_color(if ai_open { hex_rgb(0xa78bfa) } else { TEXT_SECONDARY })
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| app_ai.update(cx, |this, cx| this.toggle_ai_panel(cx)))
                                .child("✦ EXPLAIN"),
                        )
                        // Boundaries & Retention button
                        .child(
                            div()
                                .id("journal-boundaries-btn")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(9.0))
                                .py(px(3.5))
                                .bg(bound_bg)
                                .border_1()
                                .border_color(bound_color)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_bound.update(cx, |this, cx| {
                                        this.toggle_journal_retention_modal(cx);
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(bound_color)
                                        .child(format!("⚙ BOUNDARIES: {}", bound_text)),
                                ),
                        )
                        // Live tail toggle
                        .child(
                            div()
                                .id("journal-live-tail-toggle")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(9.0))
                                .py(px(3.5))
                                .bg(if is_tail { hex_rgba(0x4ade80, 0.12) } else { hex_rgba(0xfbbf24, 0.12) })
                                .border_1()
                                .border_color(if is_tail { OK } else { WARN })
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_tail.update(cx, |this, cx| {
                                        this.toggle_journal_live_tail(cx);
                                    });
                                })
                                .child(
                                    div()
                                        .size(px(6.0))
                                        .rounded_full()
                                        .bg(if is_tail { OK } else { WARN }),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if is_tail { OK } else { WARN })
                                        .child(if is_tail { "LIVE TAIL ON" } else { "PAUSED" }),
                                ),
                        )
                        // Export / copy visible window
                        .child(
                            div()
                                .id("journal-export-btn")
                                .px(px(8.0))
                                .py(px(3.5))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_MUTED)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .on_click(move |_ev, _window, _cx| {
                                    copy_to_clipboard_system(&export_snapshot);
                                })
                                .child("EXPORT ⧉"),
                        )
                        // Clear button
                        .child(
                            div()
                                .id("journal-clear-btn")
                                .px(px(8.0))
                                .py(px(3.5))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_MUTED)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .on_click(move |_ev, _window, cx| {
                                    app_clear.update(cx, |this, cx| {
                                        this.clear_journal(cx);
                                    });
                                })
                                .child("CLEAR"),
                        )
                }),
        )
        // 2. Time & Scope bar — real history navigation, not just "now"
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(16.0))
                .bg(BG_APP)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .child(div().text_color(TEXT_DIMMER).child("RANGE"))
                .children({
                    let ranges = [
                        JournalTimeRange::Live,
                        JournalTimeRange::Last15m,
                        JournalTimeRange::Last1h,
                        JournalTimeRange::Last6h,
                        JournalTimeRange::Last24h,
                        JournalTimeRange::Last7d,
                        JournalTimeRange::AllTime,
                    ];
                    ranges.into_iter().enumerate().map(|(idx, r)| {
                        let app_r = app_clone.clone();
                        let is_sel = journal.time_range == r;
                        div()
                            .id(ElementId::NamedInteger("journal-range-chip".into(), idx as u64))
                            .px(px(7.0))
                            .py(px(2.0))
                            .bg(if is_sel { BG_CHIP } else { hex_rgba(0, 0.0) })
                            .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_DIMMER })
                            .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                            .hover(|s| s.text_color(TEXT_SECONDARY))
                            .cursor_pointer()
                            .on_click(move |_ev, _window, cx| {
                                app_r.update(cx, |this, cx| {
                                    this.set_journal_time_range(r, cx);
                                });
                            })
                            .child(r.label())
                    })
                })
                .child(div().w(px(1.0)).h(px(14.0)).bg(BORDER_PANEL).mx(px(6.0)))
                .child(div().text_color(TEXT_DIMMER).child("BOOT"))
                .children({
                    let boots = [JournalBootScope::Current, JournalBootScope::Previous];
                    boots.into_iter().enumerate().map(|(idx, b)| {
                        let app_b = app_clone.clone();
                        let is_sel = journal.boot == b;
                        div()
                            .id(ElementId::NamedInteger("journal-boot-chip".into(), idx as u64))
                            .px(px(7.0))
                            .py(px(2.0))
                            .bg(if is_sel { BG_CHIP } else { hex_rgba(0, 0.0) })
                            .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_DIMMER })
                            .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                            .hover(|s| s.text_color(TEXT_SECONDARY))
                            .cursor_pointer()
                            .on_click(move |_ev, _window, cx| {
                                app_b.update(cx, |this, cx| {
                                    this.set_journal_boot(b, cx);
                                });
                            })
                            .child(b.label())
                    })
                })
                .child(div().w(px(1.0)).h(px(14.0)).bg(BORDER_PANEL).mx(px(6.0)))
                .child({
                    let app_dedupe = app_clone.clone();
                    let is_on = journal.dedupe;
                    div()
                        .id("journal-dedupe-toggle")
                        .px(px(7.0))
                        .py(px(2.0))
                        .bg(if is_on { BG_CHIP } else { hex_rgba(0, 0.0) })
                        .text_color(if is_on { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .font_weight(if is_on { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .hover(|s| s.text_color(TEXT_SECONDARY))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            app_dedupe.update(cx, |this, cx| {
                                this.toggle_journal_dedupe(cx);
                            });
                        })
                        .child("DEDUPE REPEATS")
                })
                // Quick searches for what people most often look for.
                .child(div().w(px(1.0)).h(px(12.0)).bg(BORDER_PANEL).mx(px(6.0)))
                .child(div().text_color(TEXT_DIMMER).child("QUICK"))
                .children(JOURNAL_PRESETS.iter().enumerate().map(|(i, (label, unit, grep))| {
                    let app_p = app_clone.clone();
                    let on = journal.search == *grep && journal.unit_filter.as_deref() == *unit;
                    div()
                        .id(ElementId::NamedInteger("journal-preset".into(), i as u64))
                        .px(px(7.0))
                        .py(px(2.0))
                        .bg(if on { BG_CHIP } else { hex_rgba(0, 0.0) })
                        .border_1()
                        .border_color(if on { TEXT_SECONDARY } else { BORDER_DEFAULT })
                        .text_color(if on { TEXT_PRIMARY } else { TEXT_DIM })
                        .hover(|s| s.text_color(TEXT_PRIMARY))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| app_p.update(cx, |this, cx| this.apply_journal_preset(i, cx)))
                        .child(*label)
                }))
                .child(div().flex_1())
                .child({
                    let app_more = app_clone.clone();
                    div()
                        .id("journal-load-more-btn")
                        .px(px(7.0))
                        .py(px(2.0))
                        .text_color(TEXT_DIM)
                        .hover(|s| s.text_color(TEXT_PRIMARY))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            app_more.update(cx, |this, cx| {
                                this.load_more_journal(cx);
                            });
                        })
                        .child(format!("↑ load {} older", journal.limit))
                }),
        )
        // 3. Filter Toolbar (Severity chips + Unit filters + Search)
        .child(
            div()
                .h(px(36.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(BG_APP)
                .border_b_1()
                .border_color(BORDER_PANEL)
                // Severity pills
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        // ALL
                        .child({
                            let app_filter = app_clone.clone();
                            let is_sel = journal.severity_filter.is_none();
                            div()
                                .id("journal-severity-all")
                                .px(px(8.0))
                                .py(px(2.5))
                                .bg(if is_sel { TEXT_PRIMARY } else { BG_CONTROL })
                                .text_color(if is_sel { hex_rgb(0x0a0a0c) } else { TEXT_MUTED })
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_filter.update(cx, |this, cx| {
                                        this.set_journal_severity(None, cx);
                                    });
                                })
                                .child("ALL")
                        })
                        // ERR / CRIT
                        .child({
                            let app_filter = app_clone.clone();
                            let is_sel = journal.severity_filter == Some(JournalPriority::Err);
                            div()
                                .id("journal-severity-err")
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .px(px(8.0))
                                .py(px(2.5))
                                .bg(if is_sel { CRIT } else { CRIT_BG })
                                .text_color(if is_sel { hex_rgb(0x0a0a0c) } else { CRIT })
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_filter.update(cx, |this, cx| {
                                        this.set_journal_severity(Some(JournalPriority::Err), cx);
                                    });
                                })
                                .child("ERR+")
                                .children(if err_count > 0 {
                                    Some(
                                        div()
                                            .px(px(4.0))
                                            .py(px(0.5))
                                            .bg(hex_rgba(0, 0.2))
                                            .text_size(px(9.0))
                                            .child(err_count.to_string()),
                                    )
                                } else {
                                    None
                                })
                        })
                        // WARN
                        .child({
                            let app_filter = app_clone.clone();
                            let is_sel = journal.severity_filter == Some(JournalPriority::Warning);
                            div()
                                .id("journal-severity-warn")
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .px(px(8.0))
                                .py(px(2.5))
                                .bg(if is_sel { WARN } else { WARN_BG })
                                .text_color(if is_sel { hex_rgb(0x0a0a0c) } else { WARN })
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_filter.update(cx, |this, cx| {
                                        this.set_journal_severity(Some(JournalPriority::Warning), cx);
                                    });
                                })
                                .child("WARN+")
                                .children(if warn_count > 0 {
                                    Some(
                                        div()
                                            .px(px(4.0))
                                            .py(px(0.5))
                                            .bg(hex_rgba(0, 0.2))
                                            .text_size(px(9.0))
                                            .child(warn_count.to_string()),
                                    )
                                } else {
                                    None
                                })
                        })
                        // INFO
                        .child({
                            let app_filter = app_clone.clone();
                            let is_sel = journal.severity_filter == Some(JournalPriority::Info);
                            div()
                                .id("journal-severity-info")
                                .px(px(8.0))
                                .py(px(2.5))
                                .bg(if is_sel { TEXT_SECONDARY } else { BG_CONTROL })
                                .text_color(if is_sel { hex_rgb(0x0a0a0c) } else { TEXT_MUTED })
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_filter.update(cx, |this, cx| {
                                        this.set_journal_severity(Some(JournalPriority::Info), cx);
                                    });
                                })
                                .child("INFO+")
                        })
                        // Divider
                        .child(div().w(px(1.0)).h(px(14.0)).bg(BORDER_PANEL).mx(px(4.0)))
                        // Unit chips: ALL, KERNEL, then the units logging most on this
                        // server (learned from the last unfiltered read), plus the
                        // selected unit if it isn't among them.
                        .child({
                            let mut units: Vec<String> = vec!["ALL".into(), crate::journal::reader::KERNEL_UNIT.into()];
                            units.extend(journal.known_units.iter().map(|(u, _)| u.clone()));
                            if let Some(sel) = journal.unit_filter.as_ref().filter(|u| !units.contains(u)) {
                                units.push(sel.clone());
                            }
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .children(units.into_iter().enumerate().map(|(idx, u)| {
                                    let app_u = app_clone.clone();
                                    let current_u = journal.unit_filter.as_deref().unwrap_or("ALL");
                                    let is_active = current_u == u.as_str();
                                    div()
                                        .id(ElementId::NamedInteger("journal-unit-pill".into(), idx as u64))
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(if is_active { BG_CHIP } else { hex_rgba(0, 0.0) })
                                        .text_color(if is_active { TEXT_PRIMARY } else { TEXT_DIMMER })
                                        .hover(|s| s.text_color(TEXT_SECONDARY))
                                        .cursor_pointer()
                                        .font_family("JetBrains Mono")
                                        .text_size(px(9.5))
                                        .child(if u == crate::journal::reader::KERNEL_UNIT { "KERNEL".to_string() } else { clean_unit_display(&u) })
                                        .on_click(move |_ev, _window, cx| {
                                            let target = if u == "ALL" { None } else { Some(u.clone()) };
                                            app_u.update(cx, |this, cx| this.set_journal_unit(target, cx));
                                        })
                                }))
                        })
                        // Crow's own actions in the stream (restarts, reloads, ...)
                        .child({
                            let app_act = app_clone.clone();
                            let on = journal.show_actions;
                            div()
                                .id("journal-actions-toggle")
                                .ml(px(4.0))
                                .px(px(6.0))
                                .py(px(2.0))
                                .border_1()
                                .border_color(if on { hex_rgb(0x8ab4ff) } else { BORDER_DEFAULT })
                                .text_color(if on { hex_rgb(0x8ab4ff) } else { TEXT_DIMMER })
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| app_act.update(cx, |this, cx| this.toggle_journal_actions(cx)))
                                .child(if on { "✓ CROW ACTIONS" } else { "CROW ACTIONS" })
                        })
                        // Active PID filter chip + kill action
                        .children(if let Some(fpid) = journal.pid_filter {
                            let app_clear_pid = app_clone.clone();
                            let app_kill_toggle = app_clone.clone();
                            let app_kill_confirm = app_clone.clone();
                            let app_kill_cancel = app_clone.clone();
                            let confirming = journal.pid_kill_confirm;

                            Some(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(div().w(px(1.0)).h(px(14.0)).bg(BORDER_PANEL).mx(px(4.0)))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(5.0))
                                            .px(px(7.0))
                                            .py(px(2.5))
                                            .bg(hex_rgba(0x8ab4ff, 0.14))
                                            .border_1()
                                            .border_color(hex_rgb(0x8ab4ff))
                                            .font_family(FONT_MONO)
                                            .text_size(px(9.5))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(hex_rgb(0x8ab4ff))
                                            .child(format!("PID {}", fpid))
                                            .child(
                                                div()
                                                    .id("journal-pid-filter-clear")
                                                    .cursor_pointer()
                                                    .hover(|s| s.text_color(TEXT_PRIMARY))
                                                    .on_click(move |_ev, _window, cx| {
                                                        app_clear_pid.update(cx, |this, cx| {
                                                            this.set_journal_pid_filter(None, cx);
                                                        });
                                                    })
                                                    .child("✕"),
                                            ),
                                    )
                                    .children(if confirming {
                                        Some(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(5.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(CRIT_INK_DIM)
                                                        .child(format!("terminate PID {}?", fpid)),
                                                )
                                                .child(
                                                    div()
                                                        .id("journal-pid-kill-confirm")
                                                        .px(px(7.0))
                                                        .py(px(2.5))
                                                        .bg(CRIT_BG)
                                                        .border_1()
                                                        .border_color(CRIT)
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(CRIT)
                                                        .on_click(move |_ev, _window, cx| {
                                                            app_kill_confirm.update(cx, |this, cx| {
                                                                this.execute_journal_pid_kill(cx);
                                                            });
                                                        })
                                                        .child("KILL (SIGTERM) ⏎"),
                                                )
                                                .child(
                                                    div()
                                                        .id("journal-pid-kill-cancel")
                                                        .cursor_pointer()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_DIM)
                                                        .hover(|s| s.text_color(TEXT_PRIMARY))
                                                        .on_click(move |_ev, _window, cx| {
                                                            app_kill_cancel.update(cx, |this, cx| {
                                                                this.toggle_journal_pid_kill_confirm(cx);
                                                            });
                                                        })
                                                        .child("cancel"),
                                                )
                                                .into_any_element(),
                                        )
                                    } else {
                                        Some(
                                            div()
                                                .id("journal-pid-kill-btn")
                                                .px(px(7.0))
                                                .py(px(2.5))
                                                .bg(CRIT_BG)
                                                .border_1()
                                                .border_color(BORDER_DANGER_BTN)
                                                .cursor_pointer()
                                                .hover(|s| s.border_color(CRIT))
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(CRIT_INK_DIM)
                                                .on_click(move |_ev, _window, cx| {
                                                    app_kill_toggle.update(cx, |this, cx| {
                                                        this.toggle_journal_pid_kill_confirm(cx);
                                                    });
                                                })
                                                .child("⏻ KILL PID")
                                                .into_any_element(),
                                        )
                                    }),
                            )
                        } else {
                            None
                        }),
                )
                // Search Input — a real query, not a decoration: reaches into full
                // journal history via `--grep` on Enter, not just the loaded window.
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(320.0))
                                .children(search.map(|input| {
                                    Input::new(input)
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .bg(BG_APP)
                                        .rounded(px(2.0))
                                        .prefix(tabler_icon(TablerIcon::Search).size(px(11.0)).text_color(TEXT_DIMMER))
                                })),
                        )
                        .children(if !journal.search.is_empty() {
                            let app_search_clear = app_clone.clone();
                            Some(
                                div()
                                    .id("journal-search-clear-btn")
                                    .font_family(FONT_MONO)
                                    .text_size(px(9.5))
                                    .text_color(TEXT_DIM)
                                    .hover(|s| s.text_color(TEXT_PRIMARY))
                                    .cursor_pointer()
                                    .on_click(move |_ev, _window, cx| {
                                        app_search_clear.update(cx, |this, cx| {
                                            this.journal.search.clear();
                                            this.logs_search = None; // rebuilt empty on the next render
                                            this.run_journal_query(cx);
                                        });
                                    })
                                    .child("✕"),
                            )
                        } else {
                            None
                        }),
                ),
        )
        // 4. Table Column Headers
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_DIMMER)
                .child(div().w(px(105.0)).child("TIMESTAMP"))
                .child(div().w(px(70.0)).child("LEVEL"))
                .child(div().w(px(170.0)).child("UNIT / IDENTIFIER"))
                .child(div().w(px(60.0)).child("PID"))
                .child(div().flex_1().child("MESSAGE")),
        )
        // 5. Log Entries Scrollable Body, with the AI panel beside it
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .child(
            div()
                .id("journal-log-stream")
                .flex_1()
                .min_w(px(0.0))
                .h_full()
                .overflow_y_scrollbar()
                .children(if filtered_entries.is_empty() {
                    vec![
                        div()
                            .h(px(200.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(12.0))
                                    .text_color(TEXT_MUTED)
                                    .child("No journal log entries matched your filter"),
                            )
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.5))
                                    .text_color(TEXT_FAINT)
                                    .child("Try clearing the search query or selecting 'ALL' levels"),
                            )
                            .into_any_element()
                    ]
                } else if journal.dedupe {
                    render_journal_rows_deduped(&filtered_entries, &journal.collapsed_dupe_groups, app_clone.clone(), journal)
                } else {
                    filtered_entries
                        .into_iter()
                        .enumerate()
                        .map(|(idx, entry)| {
                            render_journal_row(entry, idx, app_clone.clone(), journal).into_any_element()
                        })
                        .collect()
                }),
                )
                .children(journal.ai.open.then(|| ai_panel(&journal.ai, ai_provider.clone(), filtered_count, app_clone.clone()))),
        )
        .children(if journal.show_retention_modal {
            Some(retention_modal::retention_boundaries_modal(
                &journal.retention,
                &journal.telemetry,
                app_clone.clone(),
            ))
        } else {
            None
        })
}

/// Collapses consecutive runs of identical (unit, message) entries into one
/// expandable row with an occurrence count — the "same panic x47" case an
/// operator would otherwise scroll past by hand.
fn render_journal_rows_deduped(
    entries: &[&JournalEntry],
    collapsed: &HashSet<String>,
    app: Entity<CrowApp>,
    journal: &JournalState,
) -> Vec<AnyElement> {
    let mut runs: Vec<Vec<&JournalEntry>> = Vec::new();
    for &e in entries {
        if let Some(last_run) = runs.last_mut() {
            let last = last_run.last().unwrap();
            if last.unit == e.unit && last.message == e.message {
                last_run.push(e);
                continue;
            }
        }
        runs.push(vec![e]);
    }

    let mut rows = Vec::new();
    let mut row_idx = 0usize;
    for run in runs {
        if run.len() == 1 {
            rows.push(render_journal_row(run[0], row_idx, app.clone(), journal).into_any_element());
            row_idx += 1;
            continue;
        }

        let first = run[0];
        let last = *run.last().unwrap();
        let key = format!("dupe_{}", first.id);
        let is_collapsed = collapsed.contains(&key);
        let app_toggle = app.clone();
        let toggle_key = key.clone();
        let count = run.len();

        rows.push(
            div()
                .id(ElementId::NamedInteger("journal-dupe-header".into(), row_idx as u64))
                .relative()
                .flex()
                .items_center()
                .gap(px(8.0))
                .min_h(px(26.0))
                .px(px(16.0))
                .py(px(3.0))
                .bg(hex_rgba(0xfacc15, 0.06))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    app_toggle.update(cx, |this, cx| {
                        this.toggle_journal_dupe_group_collapsed(&toggle_key, cx);
                    });
                })
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child(
                    div()
                        .w(px(14.0))
                        .text_size(px(9.0))
                        .text_color(TEXT_DIMMER)
                        .child(if is_collapsed { "▶" } else { "▼" }),
                )
                .child(
                    div()
                        .px(px(5.0))
                        .py(px(1.0))
                        .bg(hex_rgba(0xfacc15, 0.18))
                        .text_color(hex_rgb(0xfacc15))
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(9.5))
                        .child(format!("×{}", count)),
                )
                .child(
                    div()
                        .w(px(150.0))
                        .flex_none()
                        .text_color(hex_rgb(0xa1a1aa))
                        .child(clean_unit_display(&first.unit)),
                )
                .child(
                    div()
                        .flex_1()
                        .text_color(TEXT_SECONDARY)
                        .child(first.message.clone()),
                )
                .child(
                    div()
                        .text_color(TEXT_FAINT)
                        .text_size(px(9.5))
                        .child(format!("{} → {}", first.timestamp_formatted, last.timestamp_formatted)),
                )
                .into_any_element(),
        );
        row_idx += 1;

        if !is_collapsed {
            for e in run {
                rows.push(render_journal_row(e, row_idx, app.clone(), journal).into_any_element());
                row_idx += 1;
            }
        }
    }
    rows
}

/// A synthetic "Crow did X" marker merged into the stream — cause and effect,
/// visually distinct from real journal lines so it never reads as one.
fn render_action_marker_row(entry: &JournalEntry, idx: usize) -> impl IntoElement {
    div()
        .id(ElementId::NamedInteger("journal-action-marker".into(), idx as u64))
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(16.0))
        .py(px(4.0))
        .bg(hex_rgba(0x8ab4ff, 0.05))
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .child(div().flex_1().h(px(1.0)).bg(hex_rgba(0x8ab4ff, 0.25)))
        .child(
            div()
                .text_color(hex_rgb(0x8ab4ff))
                .font_weight(FontWeight::BOLD)
                .child(format!("● {} · {}", entry.timestamp_formatted, entry.message)),
        )
        .child(div().flex_1().h(px(1.0)).bg(hex_rgba(0x8ab4ff, 0.25)))
}

fn render_journal_row(
    entry: &JournalEntry,
    idx: usize,
    app: Entity<CrowApp>,
    journal: &JournalState,
) -> impl IntoElement {
    if entry.unit == "crow-action" {
        return render_action_marker_row(entry, idx).into_any_element();
    }

    let entry_id = entry.id.clone();
    let entry_msg = entry.message.clone();
    let is_expanded = entry.is_expanded;
    let bg_color = if is_expanded {
        hex_rgb(0x13141a)
    } else if idx % 2 == 1 {
        hex_rgba(0x14161c, 0.4)
    } else {
        hex_rgba(0, 0.0)
    };

    let app_click = app.clone();
    let eid = entry_id.clone();

    div()
        .flex()
        .flex_col()
        .border_b_1()
        .border_color(BORDER_PANEL)
        // Primary Row Line
        .child(
            div()
                .id(ElementId::NamedInteger("journal-row".into(), idx as u64))
                .relative()
                .flex()
                .items_baseline()
                .min_h(px(26.0))
                .px(px(16.0))
                .py(px(4.0))
                .bg(bg_color)
                .hover(|s| s.bg(BG_ROW_HOVER))
                .cursor_pointer()
                .on_click(move |_ev, _window, cx| {
                    let id_val = eid.clone();
                    app_click.update(cx, |this, cx| {
                        this.toggle_journal_expanded(&id_val, cx);
                    });
                })
                .children(if entry.priority.is_error() {
                    Some(left_indicator(CRIT))
                } else if entry.priority.is_warn() {
                    Some(left_indicator(WARN))
                } else {
                    None
                })
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                // 1. Timestamp
                .child(
                    div()
                        .w(px(105.0))
                        .flex_none()
                        .text_color(TEXT_FAINT)
                        .child(entry.timestamp_formatted.clone()),
                )
                // 2. Priority Level Pill
                .child(
                    div()
                        .w(px(70.0))
                        .flex_none()
                        .child(
                            div()
                                .w(px(52.0))
                                .px(px(4.0))
                                .py(px(1.0))
                                .bg(entry.priority.badge_bg())
                                .text_color(entry.priority.badge_fg())
                                .text_size(px(8.5))
                                .font_weight(FontWeight::BOLD)
                                .text_align(TextAlign::Center)
                                .child(entry.priority.label()),
                        ),
                )
                // 3. Unit / Comm
                .child(
                    div()
                        .w(px(170.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_color(if entry.priority.is_error() {
                                    CRIT
                                } else {
                                    hex_rgb(0xa1a1aa)
                                })
                                .font_weight(FontWeight::MEDIUM)
                                .child(clean_unit_display(&entry.unit)),
                        ),
                )
                // 4. PID
                .child(
                    div()
                        .w(px(60.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .child(if let Some(pid) = entry.pid {
                            let app_pid = app.clone();
                            let is_filtered = journal.pid_filter == Some(pid);

                            div()
                                .id(ElementId::NamedInteger("journal-pid".into(), idx as u64))
                                .px(px(4.0))
                                .py(px(1.0))
                                .rounded_sm()
                                .bg(if is_filtered { hex_rgba(0x8ab4ff, 0.18) } else { hex_rgba(0, 0.0) })
                                .text_color(if is_filtered { hex_rgb(0x8ab4ff) } else { TEXT_FAINT })
                                .font_weight(if is_filtered { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .hover(|s| s.text_color(hex_rgb(0x8ab4ff)).bg(hex_rgba(0x8ab4ff, 0.12)))
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    cx.stop_propagation();
                                    app_pid.update(cx, |this, cx| {
                                        let next = if is_filtered { None } else { Some(pid) };
                                        this.set_journal_pid_filter(next, cx);
                                    });
                                })
                                .child(pid.to_string())
                                .into_any_element()
                        } else {
                            div().text_color(TEXT_FAINT).child("—").into_any_element()
                        }),
                )
                // 5. Message body
                .child(
                    div()
                        .flex_1()
                        .text_color(if entry.priority.is_error() {
                            TEXT_PRIMARY
                        } else if entry.priority.is_warn() {
                            TEXT_PRIMARY
                        } else {
                            TEXT_SECONDARY
                        })
                        .child(render_highlighted_message(&entry.message, &crate::journal::search_terms(&journal.search))),
                )
                // 6. Expand glyph
                .child(
                    div()
                        .w(px(20.0))
                        .flex_none()
                        .text_align(TextAlign::Right)
                        .text_color(TEXT_DIMMER)
                        .child(if is_expanded { "▲" } else { "▼" }),
                ),
        )
        // Expanded Raw Systemd Fields Inspector Drawer
        .children(if is_expanded {
            Some(render_inspector_drawer(entry, &entry_msg, idx, app.clone()))
        } else {
            None
        })
        .into_any_element()
}

fn render_inspector_drawer(
    entry: &JournalEntry,
    msg: &str,
    idx: usize,
    _app: Entity<CrowApp>,
) -> impl IntoElement {
    let raw_json = serde_json::to_string_pretty(&entry.fields).unwrap_or_default();
    let copy_json_str = raw_json.clone();
    let copy_msg_str = msg.to_string();

    div()
        .px(px(24.0))
        .py(px(10.0))
        .bg(hex_rgb(0x0c0d11))
        .border_b_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .gap(px(8.0))
        // Action header inside inspector
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_TERTIARY)
                        .child("STRUCTURED SYSTEMD JOURNAL FIELDS"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .id(ElementId::NamedInteger("journal-copy-msg".into(), idx as u64))
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_MUTED)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .on_click(move |_ev, _window, _cx| {
                                    copy_to_clipboard_system(&copy_msg_str);
                                })
                                .child("COPY MESSAGE"),
                        )
                        .child(
                            div()
                                .id(ElementId::NamedInteger("journal-copy-json".into(), idx as u64))
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_MUTED)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .on_click(move |_ev, _window, _cx| {
                                    copy_to_clipboard_system(&copy_json_str);
                                })
                                .child("COPY JSON"),
                        ),
                ),
        )
        // Grid of metadata properties
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(8.0))
                .children(entry.fields.iter().map(|(k, v)| {
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(6.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(hex_rgb(0x13141a))
                        .border_1()
                        .border_color(hex_rgb(0x1e2029))
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .child(
                            div()
                                .text_color(TEXT_DIMMER)
                                .font_weight(FontWeight::BOLD)
                                .child(k.clone()),
                        )
                        .child(
                            div()
                                .text_color(TEXT_SECONDARY)
                                .child(v.clone()),
                        )
                })),
        )
}

fn clean_unit_display(unit: &str) -> String {
    if unit.is_empty() {
        return "system".to_string();
    }
    unit.trim_end_matches(".service").to_string()
}

/// The message with the search's literal terms highlighted in place.
fn render_highlighted_message(msg: &str, terms: &[String]) -> impl IntoElement {
    let ranges = crate::journal::match_ranges(msg, terms);
    let style = HighlightStyle { background_color: Some(hex_rgba(0xfacc15, 0.28).into()), color: Some(TEXT_MAX.into()), ..Default::default() };
    div()
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(StyledText::new(msg.to_string()).with_highlights(ranges.into_iter().map(|r| (r, style))))
}

/// Plain-English reading of the lines on screen from the configured AI
/// provider. Nothing is sent until EXPLAIN is clicked.
fn ai_panel(ai: &AiPanelState, provider: Option<String>, lines_on_screen: usize, app: Entity<CrowApp>) -> impl IntoElement {
    let accent = hex_rgb(0xa78bfa);
    let app_go = app.clone();
    let body: AnyElement = match (&ai.answer, ai.loading) {
        (_, true) => div().p(px(14.0)).text_color(TEXT_DIM).child(format!("Asking {} about {} lines…", ai.provider, ai.lines_sent)).into_any_element(),
        (Some(Ok(text)), _) => div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .children(text.lines().map(|l| {
                let heading = l.starts_with('#') || (l.len() > 2 && l.as_bytes()[0].is_ascii_digit() && l.as_bytes()[1] == b'.');
                div().text_color(if heading { TEXT_MAX } else { TEXT_SECONDARY }).font_weight(if heading { FontWeight::BOLD } else { FontWeight::NORMAL }).child(l.trim_start_matches('#').trim_start().to_string())
            }))
            .child(div().pt(px(6.0)).text_size(px(9.0)).text_color(TEXT_FAINTER).child(format!("{} · {} lines sent", ai.provider, ai.lines_sent)))
            .into_any_element(),
        (Some(Err(e)), _) => div().p(px(14.0)).text_color(CRIT).child(e.clone()).into_any_element(),
        (None, false) => div().p(px(14.0)).text_color(TEXT_FAINT).child("Get a plain-English reading of the lines on screen: what's happening, what looks wrong, likely causes and what to check next.").into_any_element(),
    };
    div()
        .id("journal-ai-panel")
        .w(px(400.0))
        .flex_none()
        .h_full()
        .bg(BG_RAIL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().text_color(accent).font_weight(FontWeight::BOLD).child("✦ EXPLAIN LOGS"))
                .child(div().flex_1())
                .child(match &provider {
                    Some(_) if !ai.loading => div()
                        .id("journal-ai-go")
                        .px(px(9.0))
                        .py(px(3.0))
                        .border_1()
                        .border_color(accent)
                        .text_color(accent)
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgba(0xa78bfa, 0.12)))
                        .on_click(move |_ev, _window, cx| app_go.update(cx, |this, cx| this.explain_logs_with_ai(cx)))
                        .child(format!("EXPLAIN {lines_on_screen} LINES"))
                        .into_any_element(),
                    Some(_) => div().text_color(TEXT_FAINT).child("working…").into_any_element(),
                    None => div().text_color(TEXT_FAINT).child("no provider").into_any_element(),
                }),
        )
        .child(div().id("journal-ai-body").flex_1().min_h(px(0.0)).overflow_y_scrollbar().child(body))
        .child(
            div()
                .flex_none()
                .px(px(12.0))
                .py(px(8.0))
                .border_t_1()
                .border_color(BORDER_PANEL)
                .text_size(px(9.0))
                .text_color(TEXT_FAINTER)
                .child(match provider {
                    Some(p) => format!("Sends the log lines on screen (not host names) to {p}. Nothing is sent until you click."),
                    None => "Add an API key for a provider in Settings → Clankers to enable this.".to_string(),
                }),
        )
}
