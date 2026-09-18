use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::{ProcessUnit, ServiceUnit, SocketUnit};

pub fn services_table(
    services: &[ServiceUnit],
    processes: &[ProcessUnit],
    sockets: &[SocketUnit],
    active_tab: &str,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Panel Header & Subtabs
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                // Subtab: SERVICES
                .child({
                    let app_clone = app.clone();
                    div()
                        .id("subtab-services")
                        .px(px(14.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .font_weight(if active_tab == "services" { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if active_tab == "services" { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .bg(if active_tab == "services" { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_clone.update(cx, |this, cx| {
                                this.set_services_tab("services", cx);
                            });
                        })
                        .child("SERVICES")
                })
                // Subtab: PROCESSES
                .child({
                    let app_clone = app.clone();
                    div()
                        .id("subtab-processes")
                        .px(px(14.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .font_weight(if active_tab == "processes" { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if active_tab == "processes" { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .bg(if active_tab == "processes" { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_clone.update(cx, |this, cx| {
                                this.set_services_tab("processes", cx);
                            });
                        })
                        .child("PROCESSES")
                })
                // Subtab: SOCKETS
                .child({
                    let app_clone = app.clone();
                    div()
                        .id("subtab-sockets")
                        .px(px(14.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .font_weight(if active_tab == "sockets" { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if active_tab == "sockets" { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .bg(if active_tab == "sockets" { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_clone.update(cx, |this, cx| {
                                this.set_services_tab("sockets", cx);
                            });
                        })
                        .child("SOCKETS")
                })
                .child(div().flex_1())
                // Dynamic counts cluster based on active tab
                .child(render_counts_cluster(active_tab, services, processes, sockets))
                // Sort indicator
                .child(
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
                ),
        )
        // 2. Column Header
        .child(render_column_header(active_tab))
        // 3. Table Body
        .child(
            div()
                .id("overview-subtab-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(match active_tab {
                    "processes" => render_processes_rows(processes, app.clone()),
                    "sockets" => render_sockets_rows(sockets, app.clone()),
                    _ => render_services_rows(services, app.clone()),
                }),
        )
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
            let listen = sockets.iter().filter(|s| s.state == "LISTEN").count();
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
            .child(div().w(px(170.0)).child("LOCAL ENDPOINT"))
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
// Rows: Services
// ---------------------------------------------------------------------------

fn render_services_rows(services: &[ServiceUnit], app: Entity<CrowApp>) -> Vec<AnyElement> {
    if services.is_empty() {
        return vec![
            div()
                .p(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_DIMMER)
                .child("No systemd service units detected on active host")
                .into_any_element(),
        ];
    }

    services.iter().enumerate().map(|(idx, svc)| {
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
                                .child(format!("Restart {}? Any active connections will be dropped.", svc.name)),
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
    }).collect::<Vec<_>>()
}

// ---------------------------------------------------------------------------
// Rows: Processes
// ---------------------------------------------------------------------------

fn render_processes_rows(processes: &[ProcessUnit], app: Entity<CrowApp>) -> Vec<AnyElement> {
    if processes.is_empty() {
        return vec![
            div()
                .p(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_DIMMER)
                .child("No processes detected on active host")
                .into_any_element(),
        ];
    }

    processes.iter().enumerate().map(|(idx, proc_item)| {
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
    }).collect::<Vec<_>>()
}

// ---------------------------------------------------------------------------
// Rows: Sockets
// ---------------------------------------------------------------------------

fn render_sockets_rows(sockets: &[SocketUnit], app: Entity<CrowApp>) -> Vec<AnyElement> {
    if sockets.is_empty() {
        return vec![
            div()
                .p(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_DIMMER)
                .child("No open network sockets detected on active host")
                .into_any_element(),
        ];
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
                            .pl(px(12.0))
                            .child(
                                div()
                                    .bg(hex_rgb(0x14161b))
                                    .text_color(sock.proto_color())
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::BOLD)
                                    .px(px(4.0))
                                    .py(px(1.5))
                                    .child(sock.protocol.clone()),
                            ),
                    )
                    // Col 2: LOCAL ENDPOINT
                    .child(
                        div()
                            .w(px(170.0))
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .child(div().text_color(TEXT_SECONDARY).child(sock.local_addr.clone()))
                            .child(div().text_color(TEXT_DIMMER).child(":"))
                            .child(div().text_color(hex_rgb(0x38bdf8)).font_weight(FontWeight::BOLD).child(sock.local_port.clone())),
                    )
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
