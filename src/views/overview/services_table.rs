use gpui_kit::component::scroll::ScrollableElement;
use std::collections::{HashMap, HashSet};
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::{ProcessUnit, ServiceUnit, SocketUnit};
use crate::views::firewall::{correlate_port_firewall, FirewallOperationalState, PortFirewallMatch};
use crate::views::overview::{OverviewState, SocketsViewMode};
use crate::app::overview::TablePage;
use crate::components::table_controls::{render_table_controls, Chip};
use crate::views::overview::state::{filter_processes, filter_services, page_of, ProcessFilter, ServiceFilter, TABLE_PAGE_SIZE};
use gpui_kit::component::input::InputState;
use gpui_kit::prelude::FluentBuilder as _;

pub fn services_table(
    overview: &OverviewState,
    search: Option<&Entity<InputState>>,
    firewall: Option<&FirewallOperationalState>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let services = &overview.services;
    let processes = &overview.processes;
    let sockets = &overview.sockets;
    let active_tab = overview.active_tab.as_str();

    div()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Panel Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                // Page title (each table is its own page in the sidebar)
                .child(
                    div()
                        .px(px(14.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(active_tab.to_uppercase()),
                )
                // Services and Processes: live search and filter chips.
                .children(match active_tab {
                    "services" => Some(render_table_controls(search, service_chips(overview, app.clone()))),
                    "processes" => Some(render_table_controls(search, process_chips(overview, app.clone()))),
                    _ => None,
                })
                .child(div().flex_1())
                // Dynamic counts cluster based on active tab
                .child(render_counts_cluster(active_tab, services, processes, sockets))
                // Group-by toggle (services / processes only — sockets has no natural grouping key)
                .children(if active_tab == "services" || active_tab == "processes" {
                    Some(render_group_toggle(active_tab, overview, app.clone()))
                } else {
                    None
                })
                // Sockets view mode toggle: TABLE | MAP
                .children(if active_tab == "sockets" {
                    let app_tbl = app.clone();
                    let app_map = app.clone();
                    let is_map = overview.sockets_subview == SocketsViewMode::Map;
                    Some(
                        div()
                            .h_full()
                            .flex()
                            .items_center()
                            .border_l_1()
                            .border_color(BORDER_PANEL)
                            .px(px(6.0))
                            .gap(px(3.0))
                            .child(
                                div()
                                    .id("btn-sockets-subview-table")
                                    .h(px(22.0))
                                    .flex()
                                    .items_center()
                                    .px(px(8.0))
                                    .rounded(px(3.0))
                                    .cursor_pointer()
                                    .bg(if !is_map { hex_rgba(0xffffff, 0.08) } else { hex_rgba(0, 0.0) })
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(if !is_map { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                    .text_color(if !is_map { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                    .on_click(move |_ev, _window, cx| {
                                        app_tbl.update(cx, |this, cx| {
                                            this.set_sockets_subview(SocketsViewMode::Table, cx);
                                        });
                                    })
                                    .child("TABLE"),
                            )
                            .child(
                                div()
                                    .id("btn-sockets-subview-map")
                                    .h(px(22.0))
                                    .flex()
                                    .items_center()
                                    .px(px(8.0))
                                    .rounded(px(3.0))
                                    .cursor_pointer()
                                    .bg(if is_map { hex_rgba(0x8ab4ff, 0.16) } else { hex_rgba(0, 0.0) })
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(if is_map { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                    .text_color(if is_map { hex_rgb(0x8ab4ff) } else { TEXT_SECONDARY })
                                    .on_click(move |_ev, _window, cx| {
                                        app_map.update(cx, |this, cx| {
                                            this.set_sockets_subview(SocketsViewMode::Map, cx);
                                        });
                                    })
                                    .child("MAP"),
                            ),
                    )
                } else {
                    None
                })
                // Socket Log Drawer Toggle (sockets subtab only)
                .children(if active_tab == "sockets" {
                    let app_drawer = app.clone();
                    let is_open = overview.socket_drawer_open;
                    Some(
                        div()
                            .id("btn-toggle-socket-drawer")
                            .h_full()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(12.0))
                            .border_l_1()
                            .border_color(BORDER_PANEL)
                            .cursor_pointer()
                            .bg(if is_open { hex_rgba(0x8ab4ff, 0.12) } else { hex_rgba(0, 0.0) })
                            .hover(|s| s.bg(BG_ROW_HOVER))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .font_weight(if is_open { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(if is_open { hex_rgb(0x8ab4ff) } else { TEXT_SECONDARY })
                            .on_click(move |_ev, _window, cx| {
                                app_drawer.update(cx, |this, cx| {
                                    this.toggle_socket_drawer(cx);
                                });
                            })
                            .child(
                                tabler_icon(if is_open { TablerIcon::ChevronDown } else { TablerIcon::Terminal2 })
                                    .size(px(12.0))
                                    .text_color(if is_open { hex_rgb(0x8ab4ff) } else { TEXT_DIMMER }),
                            )
                            .child(if is_open { "LOG DRAWER (ACTIVE)" } else { "LOG DRAWER" }),
                    )
                } else {
                    None
                })
                // Sort indicator
                .children(if active_tab == "sockets" && overview.sockets_subview == SocketsViewMode::Map {
                    None
                } else {
                    Some(
                        div()
                            .h_full()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .px(px(12.0))
                            .border_l_1()
                            .border_color(BORDER_PANEL)
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.5))
                                    .text_color(TEXT_TERTIARY)
                                    .child(match active_tab {
                                        "sockets" => "sort port ↑",
                                        "services" => "sort status · name",
                                        _ => "sort cpu ↓",
                                    }),
                            )
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(TEXT_MUTED)
                                    .bg(BG_KEY)
                                    .border_1()
                                    .border_color(BORDER_KEY)
                                    .px(px(5.0))
                                    .py(px(1.0))
                                    .child("s"),
                            ),
                    )
                }),
        )
        // Main view content: If Sockets Map subview is selected, render sockets map view. Otherwise, table view.
        .children(if active_tab == "sockets" && overview.sockets_subview == SocketsViewMode::Map {
            Some(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .child(super::sockets_map::render_sockets_map(overview, firewall, app.clone())),
            )
        } else {
            None
        })
        // 2. Column Header (tables only)
        .children(if active_tab == "sockets" && overview.sockets_subview == SocketsViewMode::Map {
            None
        } else {
            Some(render_column_header(active_tab))
        })
        // 3. Table Body (tables only)
        .children(if active_tab == "sockets" && overview.sockets_subview == SocketsViewMode::Map {
            None
        } else {
            Some(
                div()
                    .id("overview-subtab-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scrollbar()
                    .flex()
                    .flex_col()
                    .children(match active_tab {
                        "processes" => {
                            let filtered = filter_processes(processes, overview.process_filter, &overview.process_query);
                            let shown: Vec<ProcessUnit> = page_of(&filtered, overview.process_page).0.iter().map(|p| (*p).clone()).collect();
                            if shown.is_empty() && !processes.is_empty() {
                                vec![empty_state("No processes match — try ALL, or clear the search")]
                            } else if overview.group_processes {
                                render_processes_rows_grouped(&shown, &overview.collapsed_process_groups, app.clone())
                            } else {
                                render_processes_rows(&shown, app.clone())
                            }
                        }
                        "sockets" => render_sockets_rows(sockets, firewall, app.clone()),
                        _ => {
                            let shown = page_services(overview);
                            if shown.is_empty() && !services.is_empty() {
                                vec![empty_state("No services match — try ALL, or clear the search")]
                            } else if overview.group_services {
                                render_services_rows_grouped(&shown, &overview.collapsed_service_groups, overview, app.clone())
                            } else {
                                render_services_rows(&shown, overview, app.clone())
                            }
                        }
                    }),
            )
        })
        .children(match active_tab {
            "services" => render_pager(TablePage::Services, filter_services(services, overview.service_filter, &overview.service_query).len(), overview.service_page, app.clone()),
            "processes" => render_pager(TablePage::Processes, filter_processes(processes, overview.process_filter, &overview.process_query).len(), overview.process_page, app.clone()),
            _ => None,
        })
}

/// The services on the current page, after the filter and search.
fn page_services(overview: &OverviewState) -> Vec<ServiceUnit> {
    let filtered = filter_services(&overview.services, overview.service_filter, &overview.service_query);
    page_of(&filtered, overview.service_page).0.iter().map(|s| (*s).clone()).collect()
}

fn service_chips(overview: &OverviewState, app: Entity<CrowApp>) -> Vec<Chip> {
    ServiceFilter::ALL
        .into_iter()
        .map(|filter| {
            let count = overview.services.iter().filter(|s| filter.matches(&s.status)).count();
            let app = app.clone();
            Chip {
                id: format!("svc-filter-{}", filter.label()),
                label: filter.label(),
                count,
                is_on: overview.service_filter == filter,
                alarming: filter == ServiceFilter::Failed && count > 0,
                on_click: Box::new(move |cx| app.update(cx, |this, cx| this.set_service_filter(filter, cx))),
            }
        })
        .collect()
}

fn process_chips(overview: &OverviewState, app: Entity<CrowApp>) -> Vec<Chip> {
    ProcessFilter::ALL
        .into_iter()
        .map(|filter| {
            let count = overview.processes.iter().filter(|p| filter.matches(p)).count();
            let app = app.clone();
            Chip {
                id: format!("proc-filter-{}", filter.label()),
                label: filter.label(),
                count,
                is_on: overview.process_filter == filter,
                alarming: false,
                on_click: Box::new(move |cx| app.update(cx, |this, cx| this.set_process_filter(filter, cx))),
            }
        })
        .collect()
}

/// "1–50 of 87 · ‹ PREV · 1 / 2 · NEXT ›" under a table, when there's more
/// than one page.
fn render_pager(table: TablePage, total: usize, page: usize, app: Entity<CrowApp>) -> Option<Div> {
    let pages = total.div_ceil(TABLE_PAGE_SIZE).max(1);
    if pages <= 1 {
        return None;
    }
    let page = page.min(pages - 1);
    let first = page * TABLE_PAGE_SIZE + 1;
    let last = (first + TABLE_PAGE_SIZE - 1).min(total);
    let button = |id: &'static str, label: &'static str, target: Option<usize>| {
        let app = app.clone();
        div()
            .id(id)
            .px(px(8.0))
            .py(px(2.0))
            .border_1()
            .border_color(BORDER_DEFAULT)
            .text_color(if target.is_some() { TEXT_SECONDARY } else { TEXT_FAINTER })
            .when_some(target, |d, p| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_table_page(table, p, cx))))
            .child(label)
    };
    Some(
        div()
            .h(px(30.0))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(14.0))
            .bg(BG_PANEL)
            .border_t_1()
            .border_color(BORDER_PANEL)
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .child(div().text_color(TEXT_DIMMER).child(format!("{first}–{last} of {total}")))
            .child(div().flex_1())
            .child(button("table-page-prev", "‹ PREV", page.checked_sub(1)))
            .child(div().text_color(TEXT_FAINT).child(format!("{} / {}", page + 1, pages)))
            .child(button("table-page-next", "NEXT ›", (page + 1 < pages).then_some(page + 1))),
    )
}

// ---------------------------------------------------------------------------
// Group-by toggle
// ---------------------------------------------------------------------------

fn render_group_toggle(active_tab: &str, overview: &OverviewState, app: Entity<CrowApp>) -> impl IntoElement {
    let is_on = if active_tab == "services" { overview.group_services } else { overview.group_processes };
    let tab_owned = active_tab.to_string();
    let label = if active_tab == "services" { "GROUP BY STATUS" } else { "GROUP BY NAME" };

    div()
        .id("btn-group-toggle")
        .h_full()
        .flex()
        .items_center()
        .gap(px(5.0))
        .px(px(10.0))
        .border_l_1()
        .border_color(BORDER_PANEL)
        .bg(if is_on { BG_CHIP } else { hex_rgba(0, 0.0) })
        .text_color(if is_on { TEXT_PRIMARY } else { TEXT_DIMMER })
        .hover(|s| s.text_color(TEXT_SECONDARY))
        .cursor_pointer()
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::BOLD)
        .on_click(move |_ev, _window, cx| {
            app.update(cx, |this, cx| {
                if tab_owned == "services" {
                    this.toggle_group_services(cx);
                } else {
                    this.toggle_group_processes(cx);
                }
            });
        })
        .child(
            tabler_icon(TablerIcon::Filter)
                .size(px(10.0))
                .text_color(if is_on { TEXT_PRIMARY } else { TEXT_DIMMER }),
        )
        .child(label)
}

// ---------------------------------------------------------------------------
// Dynamic Counts Clusters
// ---------------------------------------------------------------------------

fn render_counts_cluster(
    active_tab: &str,
    services: &[ServiceUnit],
    processes: &[ProcessUnit],
    sockets: &[SocketUnit],
) -> impl IntoElement {
    match active_tab {
        "processes" => {
            let total = processes.len();
            let running = processes.iter().filter(|p| p.stat.starts_with('R')).count();
            let high_cpu = processes.iter().filter(|p| p.cpu > 10.0).count();

            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .px(px(12.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child(div().text_color(TEXT_DIMMER).child(format!("{} processes", total)))
                .child(div().text_color(OK).child(format!("{} running", running)))
                .children(if high_cpu > 0 {
                    Some(div().text_color(WARN).child(format!("{} high cpu", high_cpu)))
                } else {
                    None
                })
        }
        "sockets" => {
            let total = sockets.len();
            let listen = sockets.iter().filter(|s| super::summary::is_listening(s)).count();
            let estab = sockets.iter().filter(|s| s.state.contains("ESTAB")).count();

            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .px(px(12.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child(div().text_color(TEXT_DIMMER).child(format!("{} sockets", total)))
                .child(div().text_color(OK).child(format!("{} listening", listen)))
                .children(if estab > 0 {
                    Some(div().text_color(hex_rgb(0x38bdf8)).child(format!("{} established", estab)))
                } else {
                    None
                })
        }
        _ => {
            let total = services.len();
            let active = services.iter().filter(|s| s.status == "ACTIVE").count();
            let degraded = services.iter().filter(|s| s.status == "DEGRADED" || s.status == "PENDING").count();
            let failed = services.iter().filter(|s| s.status == "FAILED").count();

            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .px(px(12.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child(div().text_color(TEXT_DIMMER).child(format!("{} units", total)))
                .child(div().text_color(OK).child(format!("{} active", active)))
                .children(if degraded > 0 {
                    Some(div().text_color(WARN).child(format!("{} degraded", degraded)))
                } else {
                    None
                })
                .children(if failed > 0 {
                    Some(div().text_color(CRIT).child(format!("{} failed", failed)))
                } else {
                    None
                })
        }
    }
}

// ---------------------------------------------------------------------------
// Column Headers
// ---------------------------------------------------------------------------

fn render_column_header(active_tab: &str) -> impl IntoElement {
    let header_box = div()
        .h(px(26.0))
        .flex_none()
        .flex()
        .items_center()
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(TEXT_DIMMER);

    match active_tab {
        "processes" => header_box
            .child(div().w(px(74.0)).pl(px(12.0)).child("PID"))
            .child(div().w(px(96.0)).child("USER"))
            .child(div().w(px(70.0)).text_right().child("CPU %"))
            .child(div().w(px(70.0)).text_right().child("MEM %"))
            .child(div().w(px(74.0)).text_right().child("RSS"))
            .child(div().w(px(64.0)).text_center().child("STAT"))
            .child(div().w(px(80.0)).text_right().child("TIME"))
            .child(div().flex_1().min_w(px(0.0)).pl(px(12.0)).child("COMMAND"))
            .child(div().w(px(78.0)).text_right().pr(px(12.0)).child("ACTIONS")),
        "sockets" => header_box
            .child(div().w(px(64.0)).pl(px(12.0)).child("PROTO"))
            .child(div().w(px(230.0)).child("LOCAL ENDPOINT / FIREWALL"))
            .child(div().w(px(150.0)).child("PEER ENDPOINT"))
            .child(div().w(px(90.0)).child("STATE"))
            .child(div().flex_1().min_w(px(0.0)).child("PROCESS / SERVICE"))
            .child(div().w(px(74.0)).text_right().pr(px(12.0)).child("PID")),
        _ => header_box
            .child(div().w(px(26.0)))
            .child(div().flex_1().min_w(px(0.0)).child("UNIT"))
            .child(div().w(px(84.0)).child("PID"))
            .child(div().w(px(74.0)).text_right().child("CPU %"))
            .child(div().w(px(72.0)).text_right().child("MEM %"))
            .child(div().w(px(72.0)).text_right().child("RSS"))
            .child(div().w(px(84.0)).text_right().child("UPTIME"))
            .child(div().w(px(108.0)).text_right().pr(px(12.0)).child("ACTIONS")),
    }
}

// ---------------------------------------------------------------------------
// Group header row (shared look for both services-by-status and processes-by-name)
// ---------------------------------------------------------------------------

fn render_group_header(
    id_name: &'static str,
    row_idx: usize,
    dot_color: Option<Rgba>,
    label: String,
    count: usize,
    noun: &str,
    trailing: Option<(String, String)>,
    is_collapsed: bool,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(ElementId::NamedInteger(id_name.into(), row_idx as u64))
        .flex()
        .items_center()
        .h(px(26.0))
        .px(px(12.0))
        .gap(px(8.0))
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(on_toggle)
        .child(
            div()
                .w(px(12.0))
                .text_size(px(9.0))
                .text_color(TEXT_DIMMER)
                .child(if is_collapsed { "▶" } else { "▼" }),
        )
        .children(dot_color.map(|c| div().size(px(6.0)).rounded_full().bg(c).flex_none()))
        .child(
            div()
                .flex_1()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_PRIMARY)
                .child(label),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_DIMMER)
                .child(format!("{} {}{}", count, noun, if count == 1 { "" } else { "s" })),
        )
        .children(trailing.map(|(cpu, mem)| {
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .w(px(70.0))
                        .text_right()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .child(cpu),
                )
                .child(
                    div()
                        .w(px(70.0))
                        .text_right()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .child(mem),
                )
        }))
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Rows: Services
// ---------------------------------------------------------------------------

fn status_rank(status: &str) -> u8 {
    match status {
        "FAILED" => 0,
        "DEGRADED" | "PENDING" => 1,
        "ACTIVE" => 2,
        _ => 3,
    }
}

fn status_group_color(status: &str) -> Rgba {
    match status {
        "FAILED" => CRIT,
        "DEGRADED" | "PENDING" => WARN,
        "ACTIVE" => OK,
        _ => TEXT_MUTED,
    }
}

fn empty_state(message: &'static str) -> AnyElement {
    div()
        .p(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_MONO)
        .text_size(px(11.0))
        .text_color(TEXT_DIMMER)
        .child(message)
        .into_any_element()
}

fn render_services_rows(services: &[ServiceUnit], overview: &OverviewState, app: Entity<CrowApp>) -> Vec<AnyElement> {
    if services.is_empty() {
        return vec![empty_state("No systemd service units detected on active host")];
    }

    services.iter().enumerate().map(|(idx, svc)| render_service_row(svc, idx, overview, app.clone())).collect::<Vec<_>>()
}

fn render_services_rows_grouped(services: &[ServiceUnit], collapsed: &HashSet<String>, overview: &OverviewState, app: Entity<CrowApp>) -> Vec<AnyElement> {
    if services.is_empty() {
        return vec![empty_state("No systemd service units detected on active host")];
    }

    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<&ServiceUnit>> = HashMap::new();
    for svc in services {
        if !order.contains(&svc.status) {
            order.push(svc.status.clone());
        }
        groups.entry(svc.status.clone()).or_default().push(svc);
    }
    order.sort_by_key(|s| status_rank(s));

    let mut rows = Vec::new();
    let mut row_idx = 0usize;
    for status in order {
        let members = groups.get(&status).cloned().unwrap_or_default();
        let count = members.len();
        let key = status.clone();
        let is_collapsed = collapsed.contains(&key);
        let app_toggle = app.clone();

        rows.push(render_group_header(
            "svc-group-header",
            row_idx,
            Some(status_group_color(&status)),
            status.clone(),
            count,
            "unit",
            None,
            is_collapsed,
            move |_ev, _window, cx| {
                app_toggle.update(cx, |this, cx| {
                    this.toggle_service_group_collapsed(&key, cx);
                });
            },
        ));
        row_idx += 1;

        if !is_collapsed {
            for svc in members {
                rows.push(render_service_row(svc, row_idx, overview, app.clone()));
                row_idx += 1;
            }
        }
    }
    rows
}

/// Real blast-radius phrasing for a restart/stop confirm — falls back to a
/// "checking…" state while the async socket lookup is still in flight.
fn blast_radius_text(overview: &OverviewState, unit: &str) -> String {
    match overview.blast_radius.as_ref().filter(|b| b.for_unit == unit) {
        Some(b) if b.established == 0 && b.listening == 0 => "No active connections will be dropped.".to_string(),
        Some(b) => format!(
            "{} established connection{} and {} listening socket{} will be dropped.",
            b.established, if b.established == 1 { "" } else { "s" },
            b.listening, if b.listening == 1 { "" } else { "s" },
        ),
        None => "Checking active connections…".to_string(),
    }
}

fn render_service_row(svc: &ServiceUnit, idx: usize, overview: &OverviewState, app: Entity<CrowApp>) -> AnyElement {
    let is_focus = svc.is_focused;
    let is_failed = svc.status == "FAILED";
    let cpu_num: f32 = svc.cpu.parse().unwrap_or(0.0);
    let pill_bg = match svc.status.as_str() {
        "ACTIVE" => OK_BG,
        "DEGRADED" | "PENDING" => WARN_BG,
        _ => CRIT_BG,
    };

    let row_bg = if is_focus {
        BG_ROW_SELECTED
    } else if idx % 2 == 1 {
        BG_ROW_ALT
    } else {
        hex_rgba(0, 0.0)
    };

    let svc_name = svc.name.clone();
    let app_row = app.clone();
    let app_restart = app.clone();
    let app_cancel = app.clone();
    let app_confirm = app.clone();

    div()
        .id(ElementId::NamedInteger("service-row".into(), idx as u64))
        .w_full()
        .child(
            div()
                .id(ElementId::NamedInteger("service-row-inner".into(), idx as u64))
                .relative()
                .flex()
                .items_center()
                .h(px(29.0))
                .bg(row_bg)
                .font_family(FONT_MONO)
                .text_size(px(11.5))
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click({
                    let name = svc_name.clone();
                    move |_ev, _window, cx| {
                        app_row.update(cx, |this, cx| {
                            this.focus_service(&name, cx);
                        });
                    }
                })
                .children(if is_focus {
                    Some(left_indicator(TEXT_PRIMARY))
                } else {
                    None
                })
                // Col 1: Glyph
                .child(
                    div()
                        .w(px(26.0))
                        .text_center()
                        .text_size(px(9.0))
                        .text_color(svc.status_color())
                        .child(if is_failed { "■" } else { "●" }),
                )
                // Col 2: Name + Pill
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_color(if is_focus { TEXT_MAX } else { TEXT_SECONDARY })
                                .font_weight(if is_focus { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .child(svc.name.clone()),
                        )
                        .child(
                            div()
                                .bg(pill_bg)
                                .text_color(svc.status_color())
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(5.0))
                                .py(px(2.0))
                                .flex_none()
                                .child(svc.status.clone()),
                        ),
                )
                // Col 3: PID
                .child(div().w(px(84.0)).text_color(TEXT_DIM).child(svc.pid.clone()))
                // Col 4: CPU %
                .child(
                    div()
                        .w(px(74.0))
                        .text_right()
                        .text_color(if cpu_num > 15.0 { TEXT_MAX } else { TEXT_SECONDARY })
                        .child(svc.cpu.clone()),
                )
                // Col 5: MEM %
                .child(div().w(px(72.0)).text_right().text_color(TEXT_SECONDARY).child(svc.mem.clone()))
                // Col 6: RSS
                .child(div().w(px(72.0)).text_right().text_color(TEXT_DIM).child(svc.rss.clone()))
                // Col 7: UPTIME
                .child(div().w(px(84.0)).text_right().text_color(TEXT_DIM).child(svc.uptime.clone()))
                // Col 8: ACTIONS
                .child(
                    div()
                        .w(px(108.0))
                        .flex()
                        .justify_end()
                        .gap(px(3.0))
                        .pr(px(10.0))
                        .text_size(px(10.5))
                        .child(
                            div()
                                .id(ElementId::NamedInteger("btn-restart".into(), idx as u64))
                                .px(px(6.0))
                                .py(px(2.0))
                                .border_1()
                                .border_color(if is_focus { BORDER_STRONG } else { BORDER_ROW })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click({
                                    let name = svc_name.clone();
                                    move |_ev, _window, cx| {
                                        app_restart.update(cx, |this, cx| {
                                            this.toggle_service_confirm(&name, cx);
                                        });
                                    }
                                })
                                .child(
                                    tabler_icon(TablerIcon::Refresh)
                                        .size(px(10.0))
                                        .text_color(if is_focus { TEXT_SECONDARY } else { TEXT_FAINT }),
                                ),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .border_1()
                                .border_color(if is_focus { BORDER_STRONG } else { BORDER_ROW })
                                .child(
                                    tabler_icon(TablerIcon::PlayerStop)
                                        .size(px(10.0))
                                        .text_color(if is_focus { TEXT_SECONDARY } else { TEXT_FAINT }),
                                ),
                        ),
                ),
        )
        // Inline Destructive Confirmation
        .children(if svc.show_confirm {
            Some(
                div()
                    .id(ElementId::NamedInteger("confirm-svc-row".into(), idx as u64))
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .h(px(34.0))
                    .pl(px(28.0))
                    .pr(px(12.0))
                    .bg(CRIT_ROW_BG)
                    .child(left_indicator(CRIT))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.5))
                            .text_color(CRIT_INK)
                            .child(format!("Restart {}? {}", svc.name, blast_radius_text(overview, &svc.name))),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .id(ElementId::NamedInteger("btn-cancel-svc".into(), idx as u64))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .text_color(TEXT_TERTIARY)
                            .border_1()
                            .border_color(BORDER_KEY)
                            .px(px(8.0))
                            .py(px(3.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(BG_ROW_HOVER))
                            .on_click({
                                let name = svc_name.clone();
                                move |_ev, _window, cx| {
                                    app_cancel.update(cx, |this, cx| {
                                        this.toggle_service_confirm(&name, cx);
                                    });
                                }
                            })
                            .child("Cancel ")
                            .child(div().text_color(TEXT_DIMMER).child("esc")),
                    )
                    .child(
                        div()
                            .id(ElementId::NamedInteger("btn-exec-restart".into(), idx as u64))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(hex_rgb(0x0a0a0c))
                            .bg(CRIT)
                            .px(px(9.0))
                            .py(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0xef4444)))
                            .on_click({
                                let name = svc_name.clone();
                                move |_ev, _window, cx| {
                                    app_confirm.update(cx, |this, cx| {
                                        this.execute_service_restart(&name, cx);
                                    });
                                }
                            })
                            .child("RESTART ⏎"),
                    ),
            )
        } else {
            None
        })
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Rows: Processes
// ---------------------------------------------------------------------------

fn render_processes_rows(processes: &[ProcessUnit], app: Entity<CrowApp>) -> Vec<AnyElement> {
    if processes.is_empty() {
        return vec![empty_state("No processes detected on active host")];
    }

    processes.iter().enumerate().map(|(idx, proc_item)| render_process_row(proc_item, idx, app.clone())).collect::<Vec<_>>()
}

fn render_processes_rows_grouped(processes: &[ProcessUnit], collapsed: &HashSet<String>, app: Entity<CrowApp>) -> Vec<AnyElement> {
    if processes.is_empty() {
        return vec![empty_state("No processes detected on active host")];
    }

    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<&ProcessUnit>> = HashMap::new();
    for p in processes {
        if !order.contains(&p.command) {
            order.push(p.command.clone());
        }
        groups.entry(p.command.clone()).or_default().push(p);
    }
    order.sort_by(|a, b| {
        let cpu_a: f32 = groups.get(a).map(|m| m.iter().map(|p| p.cpu).sum()).unwrap_or(0.0);
        let cpu_b: f32 = groups.get(b).map(|m| m.iter().map(|p| p.cpu).sum()).unwrap_or(0.0);
        cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut rows = Vec::new();
    let mut row_idx = 0usize;
    for cmd in order {
        let members = groups.get(&cmd).cloned().unwrap_or_default();
        let count = members.len();
        let total_cpu: f32 = members.iter().map(|p| p.cpu).sum();
        let total_mem: f32 = members.iter().map(|p| p.mem).sum();
        let key = cmd.clone();
        let is_collapsed = collapsed.contains(&key);
        let app_toggle = app.clone();

        rows.push(render_group_header(
            "proc-group-header",
            row_idx,
            None,
            cmd.clone(),
            count,
            "proc",
            Some((format!("{:.1}", total_cpu), format!("{:.1}", total_mem))),
            is_collapsed,
            move |_ev, _window, cx| {
                app_toggle.update(cx, |this, cx| {
                    this.toggle_process_group_collapsed(&key, cx);
                });
            },
        ));
        row_idx += 1;

        if !is_collapsed {
            for p in members {
                rows.push(render_process_row(p, row_idx, app.clone()));
                row_idx += 1;
            }
        }
    }
    rows
}

fn render_process_row(proc_item: &ProcessUnit, idx: usize, app: Entity<CrowApp>) -> AnyElement {
    let is_focus = proc_item.is_focused;
    let pid = proc_item.pid;
    let comm = proc_item.command.clone();

    let row_bg = if is_focus {
        BG_ROW_SELECTED
    } else if idx % 2 == 1 {
        BG_ROW_ALT
    } else {
        hex_rgba(0, 0.0)
    };

    let app_row = app.clone();
    let app_term = app.clone();
    let app_cancel = app.clone();
    let app_kill = app.clone();

    div()
        .id(ElementId::NamedInteger("process-row".into(), idx as u64))
        .w_full()
        .child(
            div()
                .id(ElementId::NamedInteger("process-row-inner".into(), idx as u64))
                .relative()
                .flex()
                .items_center()
                .h(px(28.0))
                .bg(row_bg)
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    app_row.update(cx, |this, cx| {
                        this.focus_process(pid, cx);
                    });
                })
                .children(if is_focus {
                    Some(left_indicator(TEXT_PRIMARY))
                } else {
                    None
                })
                // Col 1: PID
                .child(div().w(px(74.0)).pl(px(12.0)).text_color(hex_rgb(0x38bdf8)).child(proc_item.pid.to_string()))
                // Col 2: USER
                .child(div().w(px(96.0)).text_color(TEXT_SECONDARY).child(proc_item.user.clone()))
                // Col 3: CPU %
                .child(
                    div()
                        .w(px(70.0))
                        .text_right()
                        .text_color(if proc_item.cpu > 15.0 { TEXT_MAX } else { TEXT_SECONDARY })
                        .child(format!("{:.1}", proc_item.cpu)),
                )
                // Col 4: MEM %
                .child(div().w(px(70.0)).text_right().text_color(TEXT_SECONDARY).child(format!("{:.1}", proc_item.mem)))
                // Col 5: RSS
                .child(div().w(px(74.0)).text_right().text_color(TEXT_DIM).child(proc_item.rss.clone()))
                // Col 6: STAT
                .child(
                    div()
                        .w(px(64.0))
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .bg(hex_rgb(0x14161b))
                                .text_color(proc_item.stat_color())
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(4.0))
                                .py(px(1.5))
                                .child(proc_item.stat_label()),
                        ),
                )
                // Col 7: TIME
                .child(div().w(px(80.0)).text_right().text_color(TEXT_FAINT).child(proc_item.time.clone()))
                // Col 8: COMMAND
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .pl(px(12.0))
                        .text_color(if is_focus { TEXT_MAX } else { TEXT_PRIMARY })
                        .font_weight(if is_focus { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .child(proc_item.command.clone()),
                )
                // Col 9: ACTIONS
                .child(
                    div()
                        .w(px(78.0))
                        .flex()
                        .justify_end()
                        .pr(px(10.0))
                        .child(
                            div()
                                .id(ElementId::NamedInteger("btn-term-proc".into(), idx as u64))
                                .px(px(6.0))
                                .py(px(2.0))
                                .border_1()
                                .border_color(if is_focus { BORDER_STRONG } else { BORDER_ROW })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_term.update(cx, |this, cx| {
                                        this.toggle_process_confirm(pid, cx);
                                    });
                                })
                                .child(
                                    tabler_icon(TablerIcon::PlayerStop)
                                        .size(px(10.0))
                                        .text_color(if is_focus { CRIT } else { TEXT_FAINT }),
                                ),
                        ),
                ),
        )
        // Inline Kill Confirmation
        .children(if proc_item.show_confirm {
            Some(
                div()
                    .id(ElementId::NamedInteger("confirm-proc-row".into(), idx as u64))
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .h(px(34.0))
                    .pl(px(28.0))
                    .pr(px(12.0))
                    .bg(CRIT_ROW_BG)
                    .child(left_indicator(CRIT))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.5))
                            .text_color(CRIT_INK)
                            .child(format!("Terminate {} (PID {})? Send SIGTERM.", comm, pid)),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .id(ElementId::NamedInteger("btn-cancel-proc".into(), idx as u64))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .text_color(TEXT_TERTIARY)
                            .border_1()
                            .border_color(BORDER_KEY)
                            .px(px(8.0))
                            .py(px(3.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(BG_ROW_HOVER))
                            .on_click(move |_ev, _window, cx| {
                                app_cancel.update(cx, |this, cx| {
                                    this.toggle_process_confirm(pid, cx);
                                });
                            })
                            .child("Cancel ")
                            .child(div().text_color(TEXT_DIMMER).child("esc")),
                    )
                    .child(
                        div()
                            .id(ElementId::NamedInteger("btn-exec-kill".into(), idx as u64))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(hex_rgb(0x0a0a0c))
                            .bg(CRIT)
                            .px(px(9.0))
                            .py(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0xef4444)))
                            .on_click(move |_ev, _window, cx| {
                                app_kill.update(cx, |this, cx| {
                                    this.execute_process_kill(pid, cx);
                                });
                            })
                            .child("KILL (SIGTERM) ⏎"),
                    ),
            )
        } else {
            None
        })
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Rows: Sockets
// ---------------------------------------------------------------------------

fn render_sockets_rows(
    sockets: &[SocketUnit],
    firewall: Option<&FirewallOperationalState>,
    app: Entity<CrowApp>,
) -> Vec<AnyElement> {
    if sockets.is_empty() {
        return vec![empty_state("No open network sockets detected on active host")];
    }

    sockets.iter().enumerate().map(|(idx, sock)| {
        let is_focus = sock.is_focused;
        let sock_id = format!("{}:{}:{}", sock.protocol, sock.local_port, idx);

        let row_bg = if is_focus {
            BG_ROW_SELECTED
        } else if idx % 2 == 1 {
            BG_ROW_ALT
        } else {
            hex_rgba(0, 0.0)
        };

        let app_row = app.clone();

        div()
            .id(ElementId::NamedInteger("socket-row".into(), idx as u64))
            .w_full()
            .child(
                div()
                    .id(ElementId::NamedInteger("socket-row-inner".into(), idx as u64))
                    .relative()
                    .flex()
                    .items_center()
                    .h(px(28.0))
                    .bg(row_bg)
                    .font_family(FONT_MONO)
                    .text_size(px(11.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _window, cx| {
                        let sid = sock_id.clone();
                        app_row.update(cx, |this, cx| {
                            this.focus_socket(&sid, cx);
                        });
                    })
                    .children(if is_focus {
                        Some(left_indicator(TEXT_PRIMARY))
                    } else {
                        None
                    })
                    // Col 1: PROTO
                    .child(
                        div()
                            .w(px(64.0))
                            .flex_none()
                            .pl(px(12.0))
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .flex_none()
                                    .rounded_sm()
                                    .bg(hex_rgb(0x14161b))
                                    .text_color(sock.proto_color())
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::BOLD)
                                    .px(px(4.0))
                                    .py(px(1.5))
                                    .child(sock.protocol.clone()),
                            ),
                    )
                    // Col 2: LOCAL ENDPOINT / FIREWALL
                    .child({
                        let is_listen = super::summary::is_listening(sock);
                        let fw_pill = if is_listen {
                            let fw_match = correlate_port_firewall(firewall, &sock.local_port, &sock.protocol);
                            let app_fw = app.clone();
                            let port_val = sock.local_port.clone();
                            let label = match fw_match {
                                PortFirewallMatch::Allowed { rule_number, .. } => format!("FW ALLOW #{}", rule_number),
                                PortFirewallMatch::Denied { rule_number, .. } => format!("FW DENY #{}", rule_number),
                                PortFirewallMatch::AllowedDefault => "FW ALLOW (DEFAULT)".to_string(),
                                PortFirewallMatch::NoRule => "NO FW RULE".to_string(),
                                PortFirewallMatch::Inactive => "FW OFF".to_string(),
                            };
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("btn-fw-pill".into(), idx as u64))
                                    .ml(px(6.0))
                                    .px(px(4.0))
                                    .py(px(1.0))
                                    .bg(fw_match.color().opacity(0.12))
                                    .border_1()
                                    .border_color(fw_match.color())
                                    .text_color(fw_match.color())
                                    .text_size(px(8.5))
                                    .font_weight(FontWeight::BOLD)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        let p = port_val.clone();
                                        app_fw.update(cx, |this, cx| {
                                            this.set_view("firewall", cx);
                                            this.firewall.search_query = p;
                                        });
                                    })
                                    .child(label),
                            )
                        } else {
                            None
                        };

                        div()
                            .w(px(230.0))
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(div().text_color(TEXT_SECONDARY).child(sock.local_addr.clone()))
                            .child(div().text_color(TEXT_DIMMER).child(":"))
                            .child(div().text_color(hex_rgb(0x38bdf8)).font_weight(FontWeight::BOLD).child(sock.local_port.clone()))
                            .children(fw_pill)
                    })
                    // Col 3: PEER ENDPOINT
                    .child(
                        div()
                            .w(px(150.0))
                            .text_color(TEXT_MUTED)
                            .child(format!("{}:{}", sock.peer_addr, sock.peer_port)),
                    )
                    // Col 4: STATE
                    .child(
                        div()
                            .w(px(90.0))
                            .child(
                                div()
                                    .text_color(sock.state_color())
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(10.0))
                                    .child(sock.state.clone()),
                            ),
                    )
                    // Col 5: PROCESS / SERVICE
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_color(if is_focus { TEXT_MAX } else { TEXT_PRIMARY })
                            .font_weight(if is_focus { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .child(sock.process.clone()),
                    )
                    // Col 6: PID
                    .child(
                        div()
                            .w(px(74.0))
                            .text_right()
                            .pr(px(12.0))
                            .text_color(TEXT_DIM)
                            .child(sock.pid.map(|p| p.to_string()).unwrap_or_else(|| "—".to_string())),
                    ),
            )
            .into_any_element()
    }).collect::<Vec<_>>()
}
