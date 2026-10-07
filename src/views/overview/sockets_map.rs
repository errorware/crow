//! Live Sockets Connection Map ("Little Snitch" view of the server)
//! Visualizes remote peers (incoming & outgoing) communicating with
//! local host processes and listening ports.

use std::collections::{HashMap, HashSet};

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::components::icon_button::icon_button;

use crate::app::CrowApp;
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::theme::*;
use crate::views::firewall::{correlate_port_firewall, FirewallOperationalState, PortFirewallMatch};
use crate::views::overview::collector::categorize_peer;
use crate::views::overview::models::{ConnectionDirection, ConnectionMapItem, PeerCategory, SocketUnit};
use crate::views::overview::state::{MapFilter, OverviewState};
use crate::views::overview::summary::is_listening;

/// Derives resolved connection items from listening and established sockets
pub fn resolve_connection_map_items(sockets: &[SocketUnit]) -> Vec<ConnectionMapItem> {
    // 1. Collect all local listening ports to distinguish incoming vs outgoing
    let listening_ports: HashSet<String> = sockets
        .iter()
        .filter(|s| is_listening(s))
        .map(|s| s.local_port.clone())
        .collect();

    let mut items = Vec::new();

    for sock in sockets {
        let is_estab = sock.state.contains("ESTAB") || sock.state.contains("ESTABLISHED");
        if !is_estab {
            continue;
        }

        // An established connection is incoming if its local port is one of our listening ports
        let direction = if listening_ports.contains(&sock.local_port) {
            ConnectionDirection::Incoming
        } else {
            ConnectionDirection::Outgoing
        };

        let peer_category = categorize_peer(&sock.peer_addr);

        items.push(ConnectionMapItem {
            socket: sock.clone(),
            direction,
            peer_category,
            remote_host: None,
        });
    }

    items
}

/// Formats byte counts nicely (e.g. 1.2 MB, 450 KB, 80 B)
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn render_sockets_map(
    overview: &OverviewState,
    firewall: Option<&FirewallOperationalState>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let all_items = resolve_connection_map_items(&overview.sockets);

    // Apply filtering: direction, loopback, and process focus
    let filtered_items: Vec<&ConnectionMapItem> = all_items
        .iter()
        .filter(|item| match overview.map_filter {
            MapFilter::Both => true,
            MapFilter::Incoming => item.direction == ConnectionDirection::Incoming,
            MapFilter::Outgoing => item.direction == ConnectionDirection::Outgoing,
        })
        .filter(|item| {
            if overview.map_hide_loopback {
                item.peer_category != PeerCategory::Loopback
            } else {
                true
            }
        })
        .filter(|item| {
            if let Some(ref proc_focus) = overview.map_process_focus {
                item.socket.process.eq_ignore_ascii_case(proc_focus)
            } else {
                true
            }
        })
        .collect();

    // Group incoming and outgoing connections by remote peer address
    let mut incoming_peers: HashMap<String, Vec<&ConnectionMapItem>> = HashMap::new();
    let mut outgoing_peers: HashMap<String, Vec<&ConnectionMapItem>> = HashMap::new();
    let mut local_processes: HashMap<String, Vec<&ConnectionMapItem>> = HashMap::new();

    for item in &filtered_items {
        let proc_key = if item.socket.process.is_empty() || item.socket.process == "—" {
            "unassigned".to_string()
        } else {
            item.socket.process.clone()
        };
        local_processes.entry(proc_key).or_default().push(item);

        match item.direction {
            ConnectionDirection::Incoming => {
                incoming_peers.entry(item.socket.peer_addr.clone()).or_default().push(item);
            }
            ConnectionDirection::Outgoing => {
                outgoing_peers.entry(item.socket.peer_addr.clone()).or_default().push(item);
            }
        }
    }

    // Sort peers: highest connection count first
    let mut sorted_incoming: Vec<(String, Vec<&ConnectionMapItem>)> = incoming_peers.into_iter().collect();
    sorted_incoming.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

    let mut sorted_outgoing: Vec<(String, Vec<&ConnectionMapItem>)> = outgoing_peers.into_iter().collect();
    sorted_outgoing.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

    let mut sorted_procs: Vec<(String, Vec<&ConnectionMapItem>)> = local_processes.into_iter().collect();
    sorted_procs.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

    // Active selected item details
    let selected_conn = overview
        .selected_connection_id
        .as_ref()
        .and_then(|id| all_items.iter().find(|i| {
            let item_id = format!("{}:{}:{}", i.socket.local_port, i.socket.peer_addr, i.socket.peer_port);
            &item_id == id
        }));

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Connection Map Sub-toolbar & Filter Strip
        .child(render_map_filter_toolbar(overview, all_items.len(), filtered_items.len(), app.clone()))
        // 2. Main 3-Column Diagram Canvas
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .bg(BG_PANEL)
                // Left Column: Remote Peers (Incoming)
                .child(render_peers_column(
                    "INCOMING PEERS",
                    "Remote clients initiating traffic into listening services",
                    &sorted_incoming,
                    ConnectionDirection::Incoming,
                    overview,
                    app.clone(),
                ))
                // Middle Column: Local Host Processes & Services
                .child(render_local_column(
                    "LOCAL PROCESSES & DAEMONS",
                    "Host processes maintaining open sockets",
                    &sorted_procs,
                    overview,
                    app.clone(),
                ))
                // Right Column: Remote Peers (Outgoing)
                .child(render_peers_column(
                    "OUTGOING PEERS",
                    "External endpoints reached by host processes",
                    &sorted_outgoing,
                    ConnectionDirection::Outgoing,
                    overview,
                    app.clone(),
                )),
        )
        // 3. Bottom Inspector Drawer for Selected Connection
        .children(selected_conn.map(|c| render_connection_inspector(c, firewall, app.clone())))
}

fn render_map_filter_toolbar(
    overview: &OverviewState,
    total_conns: usize,
    shown_conns: usize,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_filter = app.clone();
    let app_loopback = app.clone();
    let app_clear_proc = app.clone();

    div()
        .h(px(36.0))
        .flex_none()
        .flex()
        .items_center()
        .px(px(14.0))
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .gap(px(12.0))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        // Traffic direction filter segmented buttons
        .child(
            div()
                .flex()
                .items_center()
                .border_1()
                .border_color(BORDER_DEFAULT)
                .children(MapFilter::ALL.into_iter().enumerate().map(|(idx, f)| {
                    let is_active = overview.map_filter == f;
                    let a = app_filter.clone();
                    div()
                        .id(ElementId::NamedInteger("btn-map-filter".into(), idx as u64))
                        .px(px(9.0))
                        .py(px(3.5))
                        .bg(if is_active { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .text_color(if is_active { OK } else { TEXT_FAINT })
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            a.update(cx, |this, cx| this.set_map_filter(f, cx));
                        })
                        .child(f.label())
                })),
        )
        // Hide Loopback Toggle
        .child(
            div()
                .id("btn-toggle-loopback")
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(9.0))
                .py(px(3.5))
                .border_1()
                .border_color(if overview.map_hide_loopback { OK } else { BORDER_DEFAULT })
                .bg(if overview.map_hide_loopback { hex_rgba(0x4ade80, 0.12) } else { hex_rgba(0, 0.0) })
                .text_color(if overview.map_hide_loopback { OK } else { TEXT_DIMMER })
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    app_loopback.update(cx, |this, cx| this.toggle_map_hide_loopback(cx));
                })
                .child("HIDE 127.0.0.1"),
        )
        // Process Focus Chip (if any)
        .children(overview.map_process_focus.as_ref().map(|proc_name| {
            let p_name = proc_name.clone();
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .py(px(3.0))
                .bg(hex_rgba(0x38bdf8, 0.15))
                .border_1()
                .border_color(hex_rgb(0x38bdf8))
                .text_color(hex_rgb(0x38bdf8))
                .child(format!("FOCUS: {}", p_name))
                .child(
                    icon_button("btn-clear-proc-focus", TablerIcon::X, false)
                        .on_click(move |_ev, _window, cx| {
                            app_clear_proc.update(cx, |this, cx| this.set_map_process_focus(None, cx));
                        }),
                )
        }))
        .child(div().flex_1())
        // Telemetry count badge
        .child(
            div()
                .text_color(TEXT_DIM)
                .child(format!("SHOWING {} OF {} ESTABLISHED CONNECTIONS", shown_conns, total_conns)),
        )
}

fn render_peers_column(
    header_title: &'static str,
    header_subtitle: &'static str,
    peers: &[(String, Vec<&ConnectionMapItem>)],
    _direction: ConnectionDirection,
    overview: &OverviewState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .border_r_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        // Column Header
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .p(px(8.0))
                .px(px(14.0))
                .bg(BG_APP)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_center()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(header_title),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_FAINTER)
                        .child(header_subtitle),
                ),
        )
        // Peers List
        .child(
            div()
                .id(format!("peers-list-{}", header_title.replace(' ', "-")))
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .p(px(10.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(if peers.is_empty() {
                    vec![div()
                        .p(px(16.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_FAINT)
                        .child("No connections in this category.")
                        .into_any_element()]
                } else {
                    peers
                        .iter()
                        .map(|(addr, items)| render_peer_card(addr, items, overview, app.clone()).into_any_element())
                        .collect()
                }),
        )
}

fn render_peer_card(
    addr: &str,
    items: &[&ConnectionMapItem],
    overview: &OverviewState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let first = items[0];
    let cat = first.peer_category;
    let ports_summary = items
        .iter()
        .map(|i| i.socket.peer_port.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let total_sent: u64 = items.iter().filter_map(|i| i.socket.bytes_sent).sum();
    let total_recv: u64 = items.iter().filter_map(|i| i.socket.bytes_recv).sum();

    let is_focused = items.iter().any(|i| {
        let item_id = format!("{}:{}:{}", i.socket.local_port, i.socket.peer_addr, i.socket.peer_port);
        overview.selected_connection_id.as_deref() == Some(&item_id)
    });

    let first_id = format!("{}:{}:{}", first.socket.local_port, first.socket.peer_addr, first.socket.peer_port);
    let app_card = app.clone();

    let card_id = format!("peer-card-{}", addr);
    div()
        .id(ElementId::Name(card_id.into()))
        .p(px(10.0))
        .bg(if is_focused { BG_ROW_SELECTED } else { BG_APP })
        .border_1()
        .border_color(if is_focused { OK } else { BORDER_DEFAULT })
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
        .on_click(move |_ev, _window, cx| {
            let id = first_id.clone();
            app_card.update(cx, |this, cx| {
                if this.overview.selected_connection_id.as_deref() == Some(&id) {
                    this.select_connection_item(None, cx);
                } else {
                    this.select_connection_item(Some(id), cx);
                }
            });
        })
        .flex()
        .flex_col()
        .gap(px(4.0))
        // Top line: Remote IP & category badge
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .child(div().font_family(FONT_MONO).text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(addr.to_string()))
                        // Its reverse DNS name, when it has one (ERR-100).
                        .children(overview.peer_names.get(crate::views::overview::collector::bare_peer_addr(addr)).cloned().flatten().map(|name| {
                            div().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_DIM).overflow_hidden().whitespace_nowrap().text_ellipsis().child(name)
                        })),
                )
                .child(
                    div()
                        .px(px(5.0))
                        .py(px(1.0))
                        .bg(cat.color().opacity(0.12))
                        .border_1()
                        .border_color(cat.color().opacity(0.4))
                        .font_family(FONT_MONO)
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(cat.color())
                        .child(cat.label()),
                ),
        )
        // Middle line: Ports and connection count
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .text_color(TEXT_DIM)
                .child(format!("{} connection{} · ports: {}", items.len(), if items.len() == 1 { "" } else { "s" }, ports_summary)),
        )
        // Bottom line: Traffic statistics if available
        .children((total_sent > 0 || total_recv > 0).then(|| {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .font_family(FONT_MONO)
                .text_size(px(9.0))
                .text_color(TEXT_FAINTER)
                .child(format!("↑ {}", format_bytes(total_sent)))
                .child(format!("↓ {}", format_bytes(total_recv)))
        }))
}

fn render_local_column(
    header_title: &'static str,
    header_subtitle: &'static str,
    procs: &[(String, Vec<&ConnectionMapItem>)],
    overview: &OverviewState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .border_r_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        // Column Header
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .p(px(8.0))
                .px(px(14.0))
                .bg(BG_APP)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_center()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(header_title),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_FAINTER)
                        .child(header_subtitle),
                ),
        )
        // Processes List
        .child(
            div()
                .id("local-procs-list")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .p(px(10.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(if procs.is_empty() {
                    vec![div()
                        .p(px(16.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_FAINT)
                        .child("No active process connections.")
                        .into_any_element()]
                } else {
                    procs
                        .iter()
                        .map(|(proc_name, items)| render_process_card(proc_name, items, overview, app.clone()).into_any_element())
                        .collect()
                }),
        )
}

fn render_process_card(
    proc_name: &str,
    items: &[&ConnectionMapItem],
    overview: &OverviewState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let pid_str = items
        .first()
        .and_then(|i| i.socket.pid)
        .map(|p| format!("PID {}", p))
        .unwrap_or_else(|| "proc unprivileged".to_string());

    let in_count = items.iter().filter(|i| i.direction == ConnectionDirection::Incoming).count();
    let out_count = items.iter().filter(|i| i.direction == ConnectionDirection::Outgoing).count();

    let is_focused = overview.map_process_focus.as_deref() == Some(proc_name);
    let app_focus = app.clone();
    let p_name = proc_name.to_string();

    let card_id = format!("proc-card-{}", proc_name);
    div()
        .id(ElementId::Name(card_id.into()))
        .p(px(10.0))
        .bg(if is_focused { hex_rgba(0x38bdf8, 0.12) } else { BG_APP })
        .border_1()
        .border_color(if is_focused { hex_rgb(0x38bdf8) } else { BORDER_DEFAULT })
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
        .on_click(move |_ev, _window, cx| {
            let target = p_name.clone();
            app_focus.update(cx, |this, cx| {
                if this.overview.map_process_focus.as_deref() == Some(&target) {
                    this.set_map_process_focus(None, cx);
                } else {
                    this.set_map_process_focus(Some(target), cx);
                }
            });
        })
        .flex()
        .flex_col()
        .gap(px(4.0))
        // Process name & PID badge
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_MAX)
                        .child(proc_name.to_string()),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_FAINT)
                        .child(pid_str),
                ),
        )
        // In/Out Flow Counts
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .child(
                    div()
                        .text_color(if in_count > 0 { OK } else { TEXT_FAINTER })
                        .child(format!("← {} in", in_count)),
                )
                .child(
                    div()
                        .text_color(if out_count > 0 { hex_rgb(0x60a5fa) } else { TEXT_FAINTER })
                        .child(format!("→ {} out", out_count)),
                ),
        )
}

fn render_connection_inspector(
    item: &ConnectionMapItem,
    firewall: Option<&FirewallOperationalState>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let app_proc = app.clone();
    let sock = &item.socket;
    let proc_target = sock.process.clone();
    let fw_match = correlate_port_firewall(firewall, &sock.local_port, &sock.protocol);

    div()
        .h(px(64.0))
        .flex_none()
        .bg(BG_APP)
        .border_t_1()
        .border_color(OK)
        .p(px(10.0))
        .px(px(16.0))
        .flex()
        .items_center()
        .justify_between()
        .font_family(FONT_MONO)
        // Left info: Local endpoint ↔ Remote endpoint
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
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(TEXT_PRIMARY)
                                .child(format!("{}:{} ↔ {}:{}", sock.local_addr, sock.local_port, sock.peer_addr, sock.peer_port)),
                        )
                        .child(
                            div()
                                .px(px(5.0))
                                .py(px(0.5))
                                .bg(item.peer_category.color().opacity(0.12))
                                .border_1()
                                .border_color(item.peer_category.color())
                                .text_color(item.peer_category.color())
                                .text_size(px(8.5))
                                .font_weight(FontWeight::BOLD)
                                .child(item.peer_category.label()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(9.5))
                        .text_color(TEXT_DIM)
                        .child(format!("Protocol: {} · State: {} · Direction: {:?}", sock.protocol, sock.state, item.direction)),
                ),
        )
        // Center: Process & PID jump button
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(TEXT_DIM)
                        .child(format!("Process: {} (PID {:?})", sock.process, sock.pid)),
                )
                .child(
                    div()
                        .id("btn-inspector-jump-proc")
                        .px(px(8.0))
                        .py(px(4.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_size(px(9.5))
                        .text_color(TEXT_PRIMARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            let p = proc_target.clone();
                            app_proc.update(cx, |this, cx| {
                                this.set_view("processes", cx);
                                this.overview.process_query = p;
                            });
                        })
                        .child("VIEW IN PROCESSES →"),
                ),
        )
        // Center-Right: Firewall Rule Status & Jump
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .bg(fw_match.color().opacity(0.12))
                        .border_1()
                        .border_color(fw_match.color())
                        .text_color(fw_match.color())
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .child(fw_match.label()),
                )
                .children(match fw_match {
                    PortFirewallMatch::Allowed { .. } | PortFirewallMatch::Denied { .. } => {
                        let app_fw = app.clone();
                        let target_port = sock.local_port.clone();
                        Some(
                            div()
                                .id("btn-inspector-jump-fw")
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_size(px(9.5))
                                .text_color(TEXT_PRIMARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let p = target_port.clone();
                                    app_fw.update(cx, |this, cx| {
                                        this.set_view("firewall", cx);
                                        this.firewall.search_query = p;
                                    });
                                })
                                .child("VIEW IN FIREWALL →"),
                        )
                    }
                    _ => None,
                }),
        )
        // Right: Close button
        .child(
            div()
                .id("btn-inspector-close")
                .cursor_pointer()
                .p(px(4.0))
                .text_color(TEXT_DIM)
                .hover(|s| s.text_color(TEXT_PRIMARY))
                .on_click(move |_ev, _window, cx| {
                    app_close.update(cx, |this, cx| this.select_connection_item(None, cx));
                })
                .child(tabler_icon(TablerIcon::X).size(px(14.0)).text_color(TEXT_SECONDARY)),
        )
}

