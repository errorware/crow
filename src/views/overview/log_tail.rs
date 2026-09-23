use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::journal::JournalEntry;
use crate::components::icons::{TablerIcon, tabler_icon};

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

pub fn socket_log_drawer(
    app_data: &CrowApp,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_expand = app.clone();
    let app_close = app.clone();
    let app_filter = app.clone();

    let entries = &app_data.journal.entries;
    let focused_socket = app_data.sockets.iter().find(|s| s.is_focused).cloned();
    let focused_pid = focused_socket.as_ref().and_then(|s| s.pid);
    let filter_active = app_data.socket_drawer_filter_this_socket && focused_socket.is_some();

    // Filter entries if filter_active is enabled and socket is focused
    let filtered_entries: Vec<&JournalEntry> = if let (true, Some(sock)) = (filter_active, &focused_socket) {
        let p_lower = sock.process.to_lowercase();
        let port_str = sock.local_port.clone();
        let matching: Vec<&JournalEntry> = entries
            .iter()
            .filter(|e| {
                if let (Some(e_pid), Some(s_pid)) = (e.pid, sock.pid) {
                    if e_pid == s_pid {
                        return true;
                    }
                }
                let u_lower = e.unit.to_lowercase();
                let m_lower = e.message.to_lowercase();
                let sys_lower = e.syslog_identifier.to_lowercase();
                (!p_lower.is_empty() && (u_lower.contains(&p_lower) || m_lower.contains(&p_lower) || sys_lower.contains(&p_lower)))
                    || (!port_str.is_empty() && (m_lower.contains(&format!(":{}", port_str)) || m_lower.contains(&format!("port {}", port_str))))
            })
            .collect();
        if matching.is_empty() {
            entries.iter().collect()
        } else {
            matching
        }
    } else {
        entries.iter().collect()
    };

    let err_count = filtered_entries.iter().filter(|e| e.priority.is_error()).count();
    let warn_count = filtered_entries.iter().filter(|e| e.priority.is_warn()).count();

    let hostname = app_data
        .servers
        .iter()
        .find(|s| s.id == app_data.active_tab_id || s.name == app_data.active_tab_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "edge-01".to_string());

    div()
        .id("socket-log-drawer")
        .h(px(250.0))
        .flex_none()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_t_1()
        .border_color(BORDER_PANEL)
        // 1. Drawer Header Bar
        .child(
            div()
                .h(px(32.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .gap(px(10.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            tabler_icon(TablerIcon::Terminal2)
                                .size(px(13.0))
                                .text_color(hex_rgb(0x8ab4ff)),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("SOCKET LOG TAIL"),
                        ),
                )
                // Socket Context Badge (if focused)
                .children(if let Some(sock) = &focused_socket {
                    let proto = sock.protocol.clone();
                    let proto_color = sock.proto_color();
                    let port = sock.local_port.clone();
                    let process = sock.process.clone();
                    let state = sock.state.clone();
                    let state_color = sock.state_color();
                    let pid_str = sock.pid.map(|p| format!("PID {}", p)).unwrap_or_default();

                    Some(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(7.0))
                            .py(px(1.5))
                            .bg(hex_rgba(0x8ab4ff, 0.08))
                            .border_1()
                            .border_color(hex_rgba(0x8ab4ff, 0.25))
                            .rounded_sm()
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(proto_color)
                                    .child(proto),
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(hex_rgb(0x38bdf8))
                                    .child(format!(":{}", port)),
                            )
                            .child(
                                div()
                                    .text_color(TEXT_PRIMARY)
                                    .child(process),
                            )
                            .children(if !pid_str.is_empty() {
                                Some(div().text_color(TEXT_DIMMER).child(pid_str))
                            } else {
                                None
                            })
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(state_color)
                                    .child(state),
                            ),
                    )
                } else {
                    Some(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .text_color(TEXT_FAINT)
                            .child("All host network sockets · click any socket row to trace"),
                    )
                })
                .child(div().flex_1())
                // Filter chip (toggle filter to focused socket)
                .children(if focused_socket.is_some() {
                    let is_filtered = app_data.socket_drawer_filter_this_socket;
                    Some(
                        div()
                            .id("btn-drawer-filter-socket")
                            .px(px(7.0))
                            .py(px(2.0))
                            .bg(if is_filtered { hex_rgba(0x8ab4ff, 0.2) } else { BG_CONTROL })
                            .border_1()
                            .border_color(if is_filtered { hex_rgb(0x8ab4ff) } else { BORDER_DEFAULT })
                            .text_color(if is_filtered { hex_rgb(0x8ab4ff) } else { TEXT_DIMMER })
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                            .font_family(FONT_MONO)
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .on_click(move |_ev, _window, cx| {
                                app_filter.update(cx, |this, cx| {
                                    this.toggle_socket_drawer_filter(cx);
                                });
                            })
                            .child(if is_filtered { "FILTER: SOCKET ✓" } else { "FILTER: ALL" }),
                    )
                } else {
                    None
                })
                // Error count chip
                .child(
                    div()
                        .bg(if err_count > 0 { CRIT } else { BG_CONTROL })
                        .text_color(if err_count > 0 { rgb(0x0a0a0c) } else { TEXT_DIMMER })
                        .font_weight(FontWeight::BOLD)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .px(px(5.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .child(format!("ERROR {}", err_count)),
                )
                // Warn count chip
                .child(
                    div()
                        .bg(if warn_count > 0 { WARN } else { BG_CONTROL })
                        .text_color(if warn_count > 0 { rgb(0x0a0a0c) } else { TEXT_DIMMER })
                        .font_weight(FontWeight::BOLD)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .px(px(5.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .child(format!("WARN {}", warn_count)),
                )
                // Expand to full logs view button
                .child(
                    div()
                        .id("socket-drawer-expand-btn")
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
                        .rounded_sm()
                        .on_click(move |_ev, _window, cx| {
                            let pid = focused_pid;
                            app_expand.update(cx, |this, cx| {
                                if let Some(p) = pid {
                                    this.set_journal_pid_filter(Some(p), cx);
                                }
                                this.set_active_view("logs", cx);
                            });
                        })
                        .child("EXPAND ↗"),
                )
                // Close drawer '✕' button
                .child(
                    div()
                        .id("socket-drawer-close-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(18.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgba(0xffffff, 0.1)).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| {
                            app_close.update(cx, |this, cx| {
                                this.close_socket_drawer(cx);
                            });
                        })
                        .child(
                            tabler_icon(TablerIcon::X)
                                .size(px(12.0))
                                .text_color(TEXT_DIMMER),
                        ),
                ),
        )
        // 2. Full-Width Log Stream Scroll Region
        .child(
            div()
                .id("socket-drawer-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .py(px(4.0))
                .children(filtered_entries.iter().map(|entry| {
                    let is_err = entry.priority.is_error();
                    let is_warn = entry.priority.is_warn();

                    let (lvl_color, msg_color, row_bg) = if is_err {
                        (CRIT, CRIT_INK, CRIT_LOG_BG)
                    } else if is_warn {
                        (WARN, WARN_INK, rgb(0x00000000))
                    } else {
                        (TEXT_FAINT, TEXT_TERTIARY, rgb(0x00000000))
                    };

                    let unit_label = if !entry.unit.is_empty() {
                        entry.unit.clone()
                    } else if !entry.syslog_identifier.is_empty() {
                        entry.syslog_identifier.clone()
                    } else {
                        "kernel".to_string()
                    };

                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(12.0))
                        .py(px(2.0))
                        .bg(row_bg)
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        // Timestamp
                        .child(
                            div()
                                .w(px(85.0))
                                .flex_none()
                                .text_color(TEXT_FAINTER)
                                .child(entry.timestamp_formatted.clone()),
                        )
                        // Priority Badge
                        .child(
                            div()
                                .w(px(42.0))
                                .flex_none()
                                .font_weight(FontWeight::BOLD)
                                .text_color(lvl_color)
                                .child(entry.priority.label()),
                        )
                        // Unit / Process
                        .child(
                            div()
                                .w(px(160.0))
                                .flex_none()
                                .text_color(TEXT_SECONDARY)
                                .child(unit_label),
                        )
                        // PID (if any)
                        .child(
                            div()
                                .w(px(55.0))
                                .flex_none()
                                .text_color(TEXT_DIMMER)
                                .child(entry.pid.map(|p| format!("[{}]", p)).unwrap_or_default()),
                        )
                        // Log Message (Full remaining width!)
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .text_color(msg_color)
                                .child(entry.message.clone()),
                        )
                })),
        )
        // 3. Inline Terminal Prompt Hatch
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child(
                    div()
                        .text_color(OK)
                        .child(format!("root@{}", hostname)),
                )
                .child(
                    div()
                        .text_color(TEXT_FAINTER)
                        .child(":~#"),
                )
                .child(
                    div()
                        .text_color(TEXT_SECONDARY)
                        .child(if let (true, Some(sock)) = (filter_active, &focused_socket) {
                            format!("journalctl -f -u {}.service", sock.process)
                        } else {
                            "journalctl -f".to_string()
                        }),
                )
                .child(
                    div()
                        .w(px(6.0))
                        .h(px(13.0))
                        .bg(TEXT_PRIMARY),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_color(TEXT_FAINTER)
                        .text_size(px(9.5))
                        .child(format!("{} lines · live stream", filtered_entries.len())),
                ),
        )
}
