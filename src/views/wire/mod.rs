//! Outbound Wire & Egress Traceability View (`ERR-216`..`ERR-217`).
//!
//! Combines the application-level Outbound Wire Ledger (every SSH command,
//! PTY launch, onboarding probe, Cloud Provider API call, Clanker AI prompt,
//! OSV lookup, External Check, SMTP notification, GeoIP download, GitHub
//! release check, and Plugin execution) with the live Linux `/proc` Kernel
//! Socket Witness.

use std::collections::BTreeMap;

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::Sizable;
use gpui_kit::*;

use crate::app::wire::{WireInputs, WireState, WireViewMode};
use crate::app::CrowApp;
use crate::theme::*;
use crate::wire::proc_net::{AttributedSocket, SocketAttribution};
use crate::wire::{WireCategory, WireEvent, WireStatus};

const BG_ELEVATED: Rgba = BG_CONTROL;
const BG_SURFACE: Rgba = BG_SUBHEAD;
const ACCENT_CYAN: Rgba = hex_rgb(0x38bdf8);
const ACCENT_BLUE: Rgba = hex_rgb(0x3b82f6);
const ACCENT_BLUE_BG: Rgba = hex_rgb(0x132038);
const OK_BORDER: Rgba = OK;
const CRIT_BORDER: Rgba = BORDER_DANGER_SEL;

pub fn wire_view(
    state: &WireState,
    inputs: Option<&WireInputs>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let counters = crate::wire::counters();
    let silenced = crate::wire::is_silenced();
    let all_events = crate::wire::recent_events();
    let unaccounted = state
        .live_sockets
        .iter()
        .filter(|s| s.attribution == SocketAttribution::Unaccounted)
        .count();

    // Collect unique destinations (target_label) with counts for 1-click filtering.
    let mut destinations: BTreeMap<String, usize> = BTreeMap::new();
    for ev in &all_events {
        *destinations.entry(ev.target_label.clone()).or_insert(0) += 1;
    }

    let query_lower = state.query.trim().to_lowercase();
    let filtered: Vec<WireEvent> = all_events
        .iter()
        .filter(|ev| state.category_filter.is_none_or(|c| ev.category == c))
        .filter(|ev| state.status_filter.is_none_or(|s| ev.status == s))
        .filter(|ev| {
            state
                .destination_filter
                .as_ref()
                .is_none_or(|d| &ev.target_label == d || &ev.remote_host == d)
        })
        .filter(|ev| {
            if query_lower.is_empty() {
                return true;
            }
            ev.target_label.to_lowercase().contains(&query_lower)
                || ev.remote_host.to_lowercase().contains(&query_lower)
                || ev.caller.to_lowercase().contains(&query_lower)
                || ev.summary.to_lowercase().contains(&query_lower)
                || ev.protocol.to_lowercase().contains(&query_lower)
                || ev.request_payload.to_lowercase().contains(&query_lower)
                || ev.response_payload.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect();

    let selected_event = state
        .selected_seq
        .and_then(|seq| all_events.into_iter().find(|e| e.seq == seq));

    div()
        .size_full()
        .bg(BG_APP)
        .flex()
        .flex_col()
        .overflow_hidden()
        // 1. Top Identity & Control Header
        .child(render_header(silenced, state, app.clone()))
        // Optional export/status banner
        .children(state.export_message.as_ref().map(|(ok, msg)| {
            div()
                .w_full()
                .px(px(16.0))
                .py(px(6.0))
                .bg(if *ok { OK_BG } else { CRIT_BG })
                .border_b_1()
                .border_color(if *ok { OK_BORDER } else { CRIT_BORDER })
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(if *ok { OK } else { CRIT })
                .child(msg.clone())
        }))
        // 2. 6-Card Telemetry Strip
        .child(render_stat_strip(&counters, state.live_sockets.len(), unaccounted, silenced))
        // 3. Mode & Filter Bar
        .child(render_filter_bar(state, inputs, filtered.len(), &destinations, app.clone()))
        // 4. Main Split Body (Table + Payload Inspector Drawer)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .flex()
                .overflow_hidden()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .h_full()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(match state.mode {
                            WireViewMode::Ledger => {
                                render_ledger_table(&filtered, state.selected_seq, app.clone()).into_any_element()
                            }
                            WireViewMode::KernelSockets => {
                                render_kernel_sockets_table(&state.live_sockets, state.last_socket_scan_ms, app.clone())
                                    .into_any_element()
                            }
                        }),
                )
                .children(
                    selected_event
                        .as_ref()
                        .map(|ev| render_inspector_drawer(ev, app.clone())),
                ),
        )
}

fn render_header(
    silenced: bool,
    _state: &WireState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_silence = app.clone();
    let app_scan = app.clone();
    let app_export = app.clone();
    let app_clear = app.clone();

    div()
        .w_full()
        .px(px(16.0))
        .py(px(10.0))
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .size(px(8.0))
                                .bg(if silenced { CRIT } else { OK }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.5))
                                .text_color(TEXT_MAX)
                                .child(t!("wire-title")),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(1.0))
                                .bg(BG_ELEVATED)
                                .border_1()
                                .border_color(BORDER_STRONG)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(ACCENT_CYAN)
                                .child(t!("wire-badge-dual-witness")),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_SANS)
                        .text_size(px(11.0))
                        .text_color(TEXT_DIM)
                        .child(t!("wire-subtitle")),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                // Outbound Silence Kill-Switch
                .child(
                    div()
                        .id("wire-toggle-silence")
                        .px(px(10.0))
                        .py(px(5.0))
                        .bg(if silenced { CRIT_BG } else { BG_ELEVATED })
                        .border_1()
                        .border_color(if silenced { CRIT } else { BORDER_STRONG })
                        .text_color(if silenced { CRIT } else { TEXT_PRIMARY })
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(10.5))
                        .on_click(move |_ev, _window, cx| {
                            app_silence.update(cx, |this, cx| this.toggle_wire_silence(cx));
                        })
                        .child(if silenced {
                            t!("wire-btn-silence-on")
                        } else {
                            t!("wire-btn-silence-off")
                        }),
                )
                // Scan /proc Now
                .child(
                    div()
                        .id("wire-scan-proc")
                        .px(px(10.0))
                        .py(px(5.0))
                        .bg(BG_ELEVATED)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_PRIMARY)
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .on_click(move |_ev, _window, cx| {
                            app_scan.update(cx, |this, cx| this.refresh_wire_sockets(cx));
                        })
                        .child(t!("wire-btn-scan-proc")),
                )
                // Export JSONL
                .child(
                    div()
                        .id("wire-export-jsonl")
                        .px(px(10.0))
                        .py(px(5.0))
                        .bg(BG_ELEVATED)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_PRIMARY)
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .on_click(move |_ev, _window, cx| {
                            app_export.update(cx, |this, cx| this.export_wire_jsonl(cx));
                        })
                        .child(t!("wire-btn-export-jsonl")),
                )
                // Clear Ledger
                .child(
                    div()
                        .id("wire-clear-ledger")
                        .px(px(10.0))
                        .py(px(5.0))
                        .bg(BG_ELEVATED)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_MUTED)
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .on_click(move |_ev, _window, cx| {
                            app_clear.update(cx, |this, cx| this.clear_wire_history(cx));
                        })
                        .child(t!("wire-btn-clear")),
                ),
        )
}

fn render_stat_strip(
    counters: &crate::wire::WireCounters,
    live_socket_count: usize,
    unaccounted: usize,
    silenced: bool,
) -> impl IntoElement {
    let card = |label: String, value: String, sub: String, color: Rgba| {
        div()
            .flex_1()
            .px(px(12.0))
            .py(px(8.0))
            .bg(BG_PANEL)
            .border_r_1()
            .border_color(BORDER_PANEL)
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(9.5))
                    .text_color(TEXT_DIM)
                    .child(label),
            )
            .child(
                div()
                    .font_family(FONT_MONO)
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(13.0))
                    .text_color(color)
                    .child(value),
            )
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(9.5))
                    .text_color(TEXT_FAINT)
                    .child(sub),
            )
    };

    div()
        .w_full()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .flex()
        .child(card(
            t!("wire-stat-total-calls").into(),
            counters.total_calls.to_string(),
            t!("wire-stat-in-flight", count = counters.in_flight),
            TEXT_MAX,
        ))
        .child(card(
            t!("wire-stat-bytes").into(),
            format!("↑{} · ↓{}", format_bytes(counters.bytes_tx), format_bytes(counters.bytes_rx)),
            t!("wire-stat-redacted-ledger").into(),
            ACCENT_CYAN,
        ))
        .child(card(
            t!("wire-stat-errors-blocked").into(),
            format!("{} / {}", counters.failed_calls, counters.blocked_calls),
            t!("wire-stat-errors-sub").into(),
            if counters.failed_calls > 0 || counters.blocked_calls > 0 {
                WARN
            } else {
                OK
            },
        ))
        .child(card(
            t!("wire-stat-kernel-sockets").into(),
            live_socket_count.to_string(),
            t!("wire-stat-proc-net").into(),
            TEXT_PRIMARY,
        ))
        .child(card(
            t!("wire-stat-kernel-witness").into(),
            if unaccounted == 0 {
                t!("wire-witness-verified").into()
            } else {
                t!("wire-witness-unaccounted", count = unaccounted)
            },
            t!("wire-stat-witness-sub").into(),
            if unaccounted == 0 { OK } else { CRIT },
        ))
        .child(card(
            t!("wire-stat-posture").into(),
            if silenced {
                t!("wire-posture-silenced").into()
            } else {
                t!("wire-posture-tracing").into()
            },
            t!("wire-stat-posture-sub").into(),
            if silenced { CRIT } else { OK },
        ))
}

fn render_filter_bar(
    state: &WireState,
    inputs: Option<&WireInputs>,
    shown_count: usize,
    destinations: &BTreeMap<String, usize>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_ledger = app.clone();
    let app_proc = app.clone();
    let app_all_cat = app.clone();
    let app_all_dest = app.clone();

    div()
        .w_full()
        .bg(BG_SURFACE)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        // Row 1: View Mode + Category Pills + Search
        .child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(6.0))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .flex_wrap()
                        // Mode tabs
                        .child(
                            div()
                                .id("wire-mode-ledger")
                                .px(px(8.0))
                                .py(px(3.0))
                                .bg(if state.mode == WireViewMode::Ledger { ACCENT_BLUE_BG } else { BG_ELEVATED })
                                .border_1()
                                .border_color(if state.mode == WireViewMode::Ledger { ACCENT_BLUE } else { BORDER_DEFAULT })
                                .text_color(if state.mode == WireViewMode::Ledger { TEXT_MAX } else { TEXT_DIM })
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.0))
                                .on_click(move |_ev, _window, cx| {
                                    app_ledger.update(cx, |this, cx| this.set_wire_mode(WireViewMode::Ledger, cx));
                                })
                                .child(t!("wire-tab-ledger", count = shown_count)),
                        )
                        .child(
                            div()
                                .id("wire-mode-proc")
                                .px(px(8.0))
                                .py(px(3.0))
                                .bg(if state.mode == WireViewMode::KernelSockets { ACCENT_BLUE_BG } else { BG_ELEVATED })
                                .border_1()
                                .border_color(if state.mode == WireViewMode::KernelSockets { ACCENT_BLUE } else { BORDER_DEFAULT })
                                .text_color(if state.mode == WireViewMode::KernelSockets { TEXT_MAX } else { TEXT_DIM })
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.0))
                                .on_click(move |_ev, _window, cx| {
                                    app_proc.update(cx, |this, cx| this.set_wire_mode(WireViewMode::KernelSockets, cx));
                                })
                                .child(t!("wire-tab-proc-sockets", count = state.live_sockets.len())),
                        )
                        .child(div().w(px(1.0)).h(px(16.0)).bg(BORDER_DEFAULT).mx(px(4.0)))
                        // ALL category pill
                        .child(
                            div()
                                .id("wire-cat-all")
                                .px(px(7.0))
                                .py(px(2.0))
                                .bg(if state.category_filter.is_none() { BG_ROW_SELECTED } else { BG_ELEVATED })
                                .border_1()
                                .border_color(if state.category_filter.is_none() { BORDER_STRONG } else { BORDER_DEFAULT })
                                .text_color(if state.category_filter.is_none() { TEXT_MAX } else { TEXT_DIM })
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .on_click(move |_ev, _window, cx| {
                                    app_all_cat.update(cx, |this, cx| this.set_wire_category(None, cx));
                                })
                                .child(t!("wire-filter-all")),
                        )
                        .children(WireCategory::ALL.iter().copied().map(|cat| {
                            let active = state.category_filter == Some(cat);
                            let app_c = app.clone();
                            div()
                                .id(SharedString::from(format!("wire-cat-{}", cat.id())))
                                .px(px(7.0))
                                .py(px(2.0))
                                .bg(if active { BG_ROW_SELECTED } else { BG_ELEVATED })
                                .border_1()
                                .border_color(if active { BORDER_STRONG } else { BORDER_DEFAULT })
                                .text_color(if active { TEXT_MAX } else { TEXT_DIM })
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .on_click(move |_ev, _window, cx| {
                                    app_c.update(cx, |this, cx| {
                                        let next = if this.wire.category_filter == Some(cat) { None } else { Some(cat) };
                                        this.set_wire_category(next, cx);
                                    });
                                })
                                .child(cat.badge())
                        })),
                )
                .children(inputs.map(|inp| {
                    div()
                        .w(px(260.0))
                        .child(Input::new(&inp.search).small().cleanable(true))
                })),
        )
        // Row 2: Destination quick-filter pills (if any destinations recorded)
        .children((!destinations.is_empty()).then(|| {
            div()
                .w_full()
                .px(px(12.0))
                .py(px(5.0))
                .border_t_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_center()
                .gap(px(6.0))
                .flex_wrap()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_FAINT)
                        .child(t!("wire-destinations-label")),
                )
                .child(
                    div()
                        .id("wire-dest-all")
                        .px(px(6.0))
                        .py(px(1.5))
                        .bg(if state.destination_filter.is_none() { BG_ROW_SELECTED } else { BG_ELEVATED })
                        .border_1()
                        .border_color(if state.destination_filter.is_none() { BORDER_STRONG } else { BORDER_DEFAULT })
                        .text_color(if state.destination_filter.is_none() { TEXT_MAX } else { TEXT_DIM })
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .on_click(move |_ev, _window, cx| {
                            app_all_dest.update(cx, |this, cx| this.set_wire_destination(None, cx));
                        })
                        .child(t!("wire-filter-all")),
                )
                .children(destinations.iter().take(12).enumerate().map(|(idx, (dest, count))| {
                    let active = state.destination_filter.as_deref() == Some(dest.as_str());
                    let dest_owned = dest.clone();
                    let app_d = app.clone();
                    div()
                        .id(SharedString::from(format!("wire-dest-{idx}")))
                        .px(px(6.0))
                        .py(px(1.5))
                        .bg(if active { ACCENT_BLUE_BG } else { BG_ELEVATED })
                        .border_1()
                        .border_color(if active { ACCENT_BLUE } else { BORDER_DEFAULT })
                        .text_color(if active { TEXT_MAX } else { TEXT_PRIMARY })
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .on_click(move |_ev, _window, cx| {
                            let d = dest_owned.clone();
                            app_d.update(cx, |this, cx| {
                                let next = if this.wire.destination_filter.as_deref() == Some(d.as_str()) {
                                    None
                                } else {
                                    Some(d)
                                };
                                this.set_wire_destination(next, cx);
                            });
                        })
                        .child(format!("{dest} ({count})"))
                }))
        }))
}

fn render_ledger_table(
    events: &[WireEvent],
    selected_seq: Option<u64>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        // Table Header
        .child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(6.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_center()
                .gap(px(8.0))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .text_color(TEXT_DIM)
                .child(div().w(px(54.0)).child(t!("wire-col-seq")))
                .child(div().w(px(86.0)).child(t!("wire-col-time")))
                .child(div().w(px(62.0)).child(t!("wire-col-cat")))
                .child(div().w(px(78.0)).child(t!("wire-col-proto")))
                .child(div().w(px(130.0)).child(t!("wire-col-caller")))
                .child(div().w(px(210.0)).child(t!("wire-col-destination")))
                .child(div().flex_1().child(t!("wire-col-summary")))
                .child(div().w(px(110.0)).child(t!("wire-col-bytes")))
                .child(div().w(px(68.0)).child(t!("wire-col-duration")))
                .child(div().w(px(68.0)).child(t!("wire-col-status"))),
        )
        // Table Rows
        .child(
            div()
                .id("wire-ledger-scroll")
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .children(if events.is_empty() {
                    vec![
                        div()
                            .w_full()
                            .py(px(48.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(TEXT_FAINT)
                            .child(t!("wire-empty-ledger"))
                            .into_any_element(),
                    ]
                } else {
                    events
                        .iter()
                        .take(300)
                        .map(|ev| render_ledger_row(ev, selected_seq == Some(ev.seq), app.clone()).into_any_element())
                        .collect()
                }),
        )
}

fn render_ledger_row(
    ev: &WireEvent,
    selected: bool,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let seq = ev.seq;
    let time_str = format_ts_ms(ev.started_at_ms);
    let dest_str = match &ev.via_hop {
        Some(hop) => format!("{} (via {hop})", ev.target_label),
        None => ev.target_label.clone(),
    };
    let (status_color, status_bg, status_label) = match ev.status {
        WireStatus::Ok => (OK, OK_BG, "OK"),
        WireStatus::Error => (CRIT, CRIT_BG, "ERR"),
        WireStatus::Timeout => (WARN, WARN_BG, "TIMEOUT"),
        WireStatus::Blocked => (CRIT, CRIT_BG, "BLOCKED"),
    };

    div()
        .id(SharedString::from(format!("wire-row-{seq}")))
        .w_full()
        .px(px(12.0))
        .py(px(6.0))
        .bg(if selected { BG_ROW_SELECTED } else { BG_APP })
        .border_b_1()
        .border_color(BORDER_PANEL)
        .hover(|s| s.bg(BG_ROW_HOVER))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(8.0))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .on_click(move |_ev, _window, cx| {
            app.update(cx, |this, cx| {
                let next = if this.wire.selected_seq == Some(seq) { None } else { Some(seq) };
                this.select_wire_event(next, cx);
            });
        })
        .child(div().w(px(54.0)).text_color(TEXT_FAINT).child(format!("#{seq}")))
        .child(div().w(px(86.0)).text_color(TEXT_DIM).child(time_str))
        .child(
            div()
                .w(px(62.0))
                .child(
                    div()
                        .px(px(4.0))
                        .py(px(1.0))
                        .bg(BG_ELEVATED)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_size(px(9.5))
                        .text_color(ACCENT_CYAN)
                        .child(ev.category.badge()),
                ),
        )
        .child(div().w(px(78.0)).text_color(TEXT_DIM).child(ev.protocol.clone()))
        .child(
            div()
                .w(px(130.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_color(TEXT_PRIMARY)
                .child(ev.caller.clone()),
        )
        .child(
            div()
                .w(px(210.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_color(TEXT_MAX)
                .child(dest_str),
        )
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_color(TEXT_PRIMARY)
                .child(ev.summary.clone()),
        )
        .child(
            div()
                .w(px(110.0))
                .text_color(TEXT_DIM)
                .child(format!("↑{} ↓{}", format_bytes(ev.bytes_tx), format_bytes(ev.bytes_rx))),
        )
        .child(
            div()
                .w(px(68.0))
                .text_color(TEXT_DIM)
                .child(format!("{}ms", ev.duration_ms)),
        )
        .child(
            div()
                .w(px(68.0))
                .child(
                    div()
                        .px(px(5.0))
                        .py(px(1.0))
                        .bg(status_bg)
                        .border_1()
                        .border_color(status_color)
                        .text_size(px(9.5))
                        .text_color(status_color)
                        .child(status_label),
                ),
        )
}

fn render_kernel_sockets_table(
    sockets: &[AttributedSocket],
    last_scan_ms: i64,
    _app: Entity<CrowApp>,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(6.0))
                .bg(BG_SURFACE)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_DIM)
                .child(t!(
                    "wire-proc-explainer",
                    time = format_ts_ms(last_scan_ms),
                    pid = std::process::id()
                )),
        )
        .child(
            div()
                .w_full()
                .px(px(12.0))
                .py(px(6.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_center()
                .gap(px(8.0))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .text_color(TEXT_DIM)
                .child(div().w(px(140.0)).child(t!("wire-proc-col-process")))
                .child(div().w(px(68.0)).child(t!("wire-proc-col-proto")))
                .child(div().w(px(190.0)).child(t!("wire-proc-col-local")))
                .child(div().w(px(190.0)).child(t!("wire-proc-col-remote")))
                .child(div().w(px(110.0)).child(t!("wire-proc-col-state")))
                .child(div().w(px(90.0)).child(t!("wire-proc-col-inode")))
                .child(div().flex_1().child(t!("wire-proc-col-attribution"))),
        )
        .child(
            div()
                .id("wire-proc-scroll")
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .children(if sockets.is_empty() {
                    vec![
                        div()
                            .w_full()
                            .py(px(48.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(OK)
                            .child(t!("wire-proc-empty"))
                            .into_any_element(),
                    ]
                } else {
                    sockets
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            let s = &item.socket;
                            let (attr_color, attr_bg, attr_text) = match &item.attribution {
                                SocketAttribution::Accounted(label) => (
                                    OK,
                                    OK_BG,
                                    format!("ACCOUNTED -> {label}"),
                                ),
                                SocketAttribution::ControlMaster(label) => (
                                    ACCENT_CYAN,
                                    BG_ELEVATED,
                                    format!("SSH CONTROLMASTER [mux] -> {label}"),
                                ),
                                SocketAttribution::Loopback => (
                                    TEXT_DIM,
                                    BG_ELEVATED,
                                    "LOOPBACK / LOCAL".into(),
                                ),
                                SocketAttribution::Unaccounted => (
                                    CRIT,
                                    CRIT_BG,
                                    "UNACCOUNTED SOCKET".into(),
                                ),
                            };
                            div()
                                .id(SharedString::from(format!("wire-proc-row-{idx}")))
                                .w_full()
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(BG_APP)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(
                                    div()
                                        .w(px(140.0))
                                        .text_color(TEXT_MAX)
                                        .child(format!("{}:{}", s.comm, s.pid)),
                                )
                                .child(div().w(px(68.0)).text_color(ACCENT_CYAN).child(s.protocol))
                                .child(
                                    div()
                                        .w(px(190.0))
                                        .text_color(TEXT_DIM)
                                        .child(format!("{}:{}", s.local_ip, s.local_port)),
                                )
                                .child(
                                    div()
                                        .w(px(190.0))
                                        .text_color(TEXT_PRIMARY)
                                        .child(format!("{}:{}", s.remote_ip, s.remote_port)),
                                )
                                .child(div().w(px(110.0)).text_color(TEXT_DIM).child(s.state))
                                .child(div().w(px(90.0)).text_color(TEXT_FAINT).child(s.inode.to_string()))
                                .child(
                                    div()
                                        .flex_1()
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(1.5))
                                                .bg(attr_bg)
                                                .border_1()
                                                .border_color(attr_color)
                                                .text_size(px(9.5))
                                                .text_color(attr_color)
                                                .child(attr_text),
                                        ),
                                )
                                .into_any_element()
                        })
                        .collect()
                }),
        )
}

fn render_inspector_drawer(
    ev: &WireEvent,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let kv = |k: String, v: String| {
        div()
            .w_full()
            .flex()
            .items_baseline()
            .justify_between()
            .gap(px(8.0))
            .py(px(3.0))
            .border_b_1()
            .border_color(BORDER_PANEL)
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(10.0))
                    .text_color(TEXT_DIM)
                    .child(k),
            )
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(10.5))
                    .text_color(TEXT_MAX)
                    .child(v),
            )
    };

    div()
        .w(px(460.0))
        .h_full()
        .bg(BG_PANEL)
        .border_l_1()
        .border_color(BORDER_STRONG)
        .flex()
        .flex_col()
        .overflow_hidden()
        // Drawer Header
        .child(
            div()
                .w_full()
                .px(px(14.0))
                .py(px(10.0))
                .bg(BG_ELEVATED)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(TEXT_MAX)
                                .child(format!("{} #{}", t!("wire-drawer-title"), ev.seq)),
                        )
                        .child(
                            div()
                                .px(px(5.0))
                                .py(px(1.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_STRONG)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(ACCENT_CYAN)
                                .child(ev.category.badge()),
                        ),
                )
                .child(
                    div()
                        .id("wire-drawer-close")
                        .px(px(6.0))
                        .py(px(2.0))
                        .cursor_pointer()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_DIM)
                        .hover(|s| s.text_color(TEXT_MAX))
                        .on_click(move |_ev, _window, cx| {
                            app_close.update(cx, |this, cx| this.select_wire_event(None, cx));
                        })
                        .child("✕"),
                ),
        )
        // Drawer Body
        .child(
            div()
                .id("wire-drawer-scroll")
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(kv(t!("wire-meta-timestamp").into(), format_ts_ms(ev.started_at_ms)))
                        .child(kv(t!("wire-meta-caller").into(), ev.caller.clone()))
                        .child(kv(t!("wire-meta-protocol").into(), ev.protocol.clone()))
                        .child(kv(t!("wire-meta-target").into(), ev.target_label.clone()))
                        .child(kv(
                            t!("wire-meta-endpoint").into(),
                            format!("{}:{}", ev.remote_host, ev.remote_port),
                        ))
                        .child(kv(
                            t!("wire-meta-hop").into(),
                            ev.via_hop.clone().unwrap_or_else(|| t!("wire-meta-direct").into()),
                        ))
                        .child(kv(t!("wire-meta-duration").into(), format!("{} ms", ev.duration_ms)))
                        .child(kv(
                            t!("wire-meta-bytes").into(),
                            format!("↑ {} ({} B) · ↓ {} ({} B)", format_bytes(ev.bytes_tx), ev.bytes_tx, format_bytes(ev.bytes_rx), ev.bytes_rx),
                        )),
                )
                .children(ev.error.as_ref().map(|err| {
                    div()
                        .w_full()
                        .p(px(8.0))
                        .bg(CRIT_BG)
                        .border_1()
                        .border_color(CRIT_BORDER)
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(CRIT)
                        .child(err.clone())
                }))
                // Request Payload
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .child(t!("wire-drawer-request")),
                        )
                        .child(
                            div()
                                .w_full()
                                .p(px(10.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_PRIMARY)
                                .child(if ev.request_payload.trim().is_empty() {
                                    t!("wire-drawer-empty").into()
                                } else {
                                    ev.request_payload.clone()
                                }),
                        ),
                )
                // Response Payload
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .child(t!("wire-drawer-response")),
                        )
                        .child(
                            div()
                                .w_full()
                                .p(px(10.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_PRIMARY)
                                .child(if ev.response_payload.trim().is_empty() {
                                    t!("wire-drawer-empty").into()
                                } else {
                                    ev.response_payload.clone()
                                }),
                        ),
                ),
        )
}

fn format_ts_ms(ts_ms: i64) -> String {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_millis_opt(ts_ms)
        .single()
        .map(|dt| dt.format("%H:%M:%S%.3f").to_string())
        .unwrap_or_else(|| ts_ms.to_string())
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
