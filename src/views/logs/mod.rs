use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::journal::{JournalEntry, JournalPriority, JournalStorageMode};
use crate::keys::copy_to_clipboard_system;

pub mod retention_modal;

pub fn logs_explorer_view(
    app: Entity<CrowApp>,
    app_data: &CrowApp,
) -> impl IntoElement {
    let app_clone = app.clone();

    // Compute error and warn counts
    let err_count = app_data.journal_entries.iter().filter(|e| e.priority.is_error()).count();
    let warn_count = app_data.journal_entries.iter().filter(|e| e.priority.is_warn()).count();
    let total_count = app_data.journal_entries.len();

    // Filter entries based on level, unit, and search query
    let filtered_entries: Vec<&JournalEntry> = app_data
        .journal_entries
        .iter()
        .filter(|e| {
            // Level filter
            if let Some(prio) = app_data.journal_severity_filter {
                if prio == JournalPriority::Err {
                    if !e.priority.is_error() {
                        return false;
                    }
                } else if e.priority != prio {
                    return false;
                }
            }
            // Unit filter
            if let Some(ref u) = app_data.journal_unit_filter {
                if u != "ALL" && !e.unit.to_lowercase().contains(&u.to_lowercase()) && !e.syslog_identifier.to_lowercase().contains(&u.to_lowercase()) {
                    return false;
                }
            }
            // Search query
            if !app_data.journal_search.trim().is_empty() {
                let q = app_data.journal_search.to_lowercase();
                let matches_msg = e.message.to_lowercase().contains(&q);
                let matches_unit = e.unit.to_lowercase().contains(&q);
                let matches_pid = e.pid.map(|p| p.to_string().contains(&q)).unwrap_or(false);
                if !matches_msg && !matches_unit && !matches_pid {
                    return false;
                }
            }
            true
        })
        .collect();

    let filtered_count = filtered_entries.len();

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
                    let is_tail = app_data.journal_live_tail;
                    let is_warn = app_data.journal_telemetry.is_volatile_warning
                        || app_data.journal_retention.storage != JournalStorageMode::Persistent;
                    let bound_text = if is_warn {
                        "⚠ VOLATILE (30m RISK)"
                    } else {
                        "30d · 4GB PERSISTENT"
                    };
                    let (bound_color, bound_bg) = if is_warn {
                        (CRIT, CRIT_BG)
                    } else {
                        (OK, OK_BG)
                    };

                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
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
        // 2. Filter Toolbar (Severity chips + Unit filters + Search)
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
                            let is_sel = app_data.journal_severity_filter.is_none();
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
                            let is_sel = app_data.journal_severity_filter == Some(JournalPriority::Err);
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
                                .child("ERR")
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
                            let is_sel = app_data.journal_severity_filter == Some(JournalPriority::Warning);
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
                                .child("WARN")
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
                            let is_sel = app_data.journal_severity_filter == Some(JournalPriority::Info);
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
                                .child("INFO")
                        })
                        // Divider
                        .child(div().w(px(1.0)).h(px(14.0)).bg(BORDER_PANEL).mx(px(4.0)))
                        // Unit Pills (Common systemd units)
                        .child({
                            let units = ["ALL", "nginx", "postgres", "redis", "sshd", "kernel", "ufw"];
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .children(units.into_iter().enumerate().map(|(idx, u)| {
                                    let app_u = app_clone.clone();
                                    let current_u = app_data.journal_unit_filter.as_deref().unwrap_or("ALL");
                                    let is_active = current_u == u;
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
                                        .on_click(move |_ev, _window, cx| {
                                            app_u.update(cx, |this, cx| {
                                                let target = if u == "ALL" { None } else { Some(u.to_string()) };
                                                this.set_journal_unit(target, cx);
                                            });
                                        })
                                        .child(u)
                                }))
                        }),
                )
                // Search Input
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .w(px(280.0))
                        .h(px(24.0))
                        .px(px(8.0))
                        .bg(hex_rgb(0x0e0f13))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIMMER)
                                .child("⌕"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(if app_data.journal_search.is_empty() { TEXT_FAINT } else { TEXT_PRIMARY })
                                .child(if app_data.journal_search.is_empty() {
                                    "Filter logs…".to_string()
                                } else {
                                    app_data.journal_search.clone()
                                }),
                        )
                        .children(if !app_data.journal_search.is_empty() {
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
                                            this.journal_search.clear();
                                            cx.notify();
                                        });
                                    })
                                    .child("✕"),
                            )
                        } else {
                            None
                        }),
                ),
        )
        // 3. Table Column Headers
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
        // 4. Log Entries Scrollable Body
        .child(
            div()
                .id("journal-log-stream")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
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
                } else {
                    filtered_entries
                        .into_iter()
                        .enumerate()
                        .map(|(idx, entry)| {
                            render_journal_row(entry, idx, app_clone.clone()).into_any_element()
                        })
                        .collect()
                }),
        )
        .children(if app_data.show_journal_retention_modal {
            Some(retention_modal::retention_boundaries_modal(
                &app_data.journal_retention,
                &app_data.journal_telemetry,
                app_clone.clone(),
            ))
        } else {
            None
        })
}

fn render_journal_row(
    entry: &JournalEntry,
    idx: usize,
    app: Entity<CrowApp>,
) -> impl IntoElement {
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
                        .text_color(TEXT_FAINT)
                        .child(entry.pid.map(|p| p.to_string()).unwrap_or_else(|| "—".to_string())),
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
                        .child(render_highlighted_message(&entry.message)),
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

fn render_highlighted_message(msg: &str) -> impl IntoElement {
    // Keep raw message rendering fast and clean in JetBrains Mono
    div()
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(msg.to_string())
}
