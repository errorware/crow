//! Overview dashboard: the headline numbers from the Services, Processes and
//! Sockets pages side by side. Every item links to its row on that page.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use crate::app::CrowApp;
use crate::theme::*;

use super::summary::{summarize_processes, summarize_services, summarize_sockets, Exposure, ProcessSummary, ServiceSummary, SocketSummary};
use super::OverviewState;

pub fn overview_dashboard(overview: &OverviewState, app: Entity<CrowApp>) -> impl IntoElement {
    let services = summarize_services(&overview.services);
    let processes = summarize_processes(&overview.processes);
    let sockets = summarize_sockets(&overview.sockets);

    div()
        .id("overview-dashboard")
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .bg(BG_APP)
        .overflow_y_scrollbar()
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(12.0))
                .p(px(14.0))
                .child(services_card(&services, app.clone()))
                .child(processes_card(&processes, app.clone()))
                .child(sockets_card(&sockets, app)),
        )
}

fn card(title: &'static str, view: &'static str, app: Entity<CrowApp>) -> Div {
    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .rounded_md()
        .child(
            div()
                .h(px(34.0))
                .flex()
                .items_center()
                .px(px(12.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().font_family(FONT_MONO).text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(title))
                .child(div().flex_1())
                .child(
                    div()
                        .id(SharedString::from(format!("open-{view}")))
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(hex_rgb(0x8ab4ff))
                        .cursor_pointer()
                        .hover(|s| s.text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| {
                            app.update(cx, |this, cx| this.set_view(view, cx));
                        })
                        .child("OPEN →"),
                ),
        )
}

/// A row of big numbers under a card header.
fn stats(items: Vec<(String, &'static str, Rgba)>) -> Div {
    div().flex().gap(px(18.0)).px(px(12.0)).py(px(12.0)).border_b_1().border_color(BORDER_ROW).children(items.into_iter().map(|(value, label, color)| {
        div()
            .flex()
            .flex_col()
            .child(div().font_family(FONT_MONO).text_size(px(22.0)).font_weight(FontWeight::BOLD).text_color(color).child(value))
            .child(div().font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_FAINT).child(label))
    }))
}

fn section_label(text: &'static str) -> Div {
    div().px(px(12.0)).pt(px(10.0)).pb(px(4.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_FAINTER).child(text)
}

fn empty_line(text: &'static str) -> Div {
    div().px(px(12.0)).py(px(6.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_FAINT).child(text)
}

/// A clickable list line: left text, right text.
fn link_row(id: SharedString, left: String, right: String, right_color: Rgba, on_click: impl Fn(&mut Window, &mut App) + 'static) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(12.0))
        .py(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, window, cx| on_click(window, cx))
        .child(div().flex_1().min_w(px(0.0)).overflow_hidden().font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_SECONDARY).child(left))
        .child(div().flex_none().font_family(FONT_MONO).text_size(px(10.5)).text_color(right_color).child(right))
}

fn services_card(s: &ServiceSummary, app: Entity<CrowApp>) -> impl IntoElement {
    let attention = s.attention.iter().enumerate().map(|(i, (name, status))| {
        let (app, unit) = (app.clone(), name.clone());
        let color = if status == "FAILED" { CRIT } else { WARN };
        link_row(SharedString::from(format!("dash-svc-{i}")), name.clone(), status.clone(), color, move |_w, cx| {
            let unit = unit.clone();
            app.update(cx, |this, cx| this.open_service_from_dashboard(&unit, cx));
        })
    });
    card("SERVICES", "services", app.clone())
        .child(stats(vec![
            (s.active.to_string(), "ACTIVE", OK),
            (s.failed.to_string(), "FAILED", if s.failed > 0 { CRIT } else { TEXT_MUTED }),
            (s.degraded.to_string(), "DEGRADED", if s.degraded > 0 { WARN } else { TEXT_MUTED }),
            (s.inactive.to_string(), "INACTIVE", TEXT_MUTED),
        ]))
        .child(section_label("NEEDS ATTENTION"))
        .children((s.attention.is_empty()).then(|| empty_line("No failed or degraded units.")))
        .children(attention)
        .child(div().h(px(8.0)))
}

fn processes_card(p: &ProcessSummary, app: Entity<CrowApp>) -> impl IntoElement {
    let top_cpu = p.top_cpu.iter().enumerate().map(|(i, (pid, cmd, cpu))| {
        let (app, pid) = (app.clone(), *pid);
        link_row(SharedString::from(format!("dash-cpu-{i}")), format!("{pid:>7}  {cmd}"), format!("{cpu:.1}% CPU"), if *cpu >= 80.0 { CRIT } else if *cpu >= 40.0 { WARN } else { TEXT_SECONDARY }, move |_w, cx| {
            app.update(cx, |this, cx| this.open_process_from_dashboard(pid, cx));
        })
    });
    let top_mem = p.top_mem.iter().enumerate().map(|(i, (pid, cmd, mem, rss))| {
        let (app, pid) = (app.clone(), *pid);
        link_row(SharedString::from(format!("dash-mem-{i}")), format!("{pid:>7}  {cmd}"), format!("{mem:.1}% · {rss}"), if *mem >= 50.0 { WARN } else { TEXT_SECONDARY }, move |_w, cx| {
            app.update(cx, |this, cx| this.open_process_from_dashboard(pid, cx));
        })
    });
    card("PROCESSES", "processes", app.clone())
        .child(stats(vec![
            (p.total.to_string(), "TOTAL", TEXT_PRIMARY),
            (p.zombies.to_string(), "ZOMBIE", if p.zombies > 0 { WARN } else { TEXT_MUTED }),
            (p.blocked.to_string(), "BLOCKED (D)", if p.blocked > 0 { WARN } else { TEXT_MUTED }),
        ]))
        .child(section_label("TOP CPU"))
        .children(p.top_cpu.is_empty().then(|| empty_line("No processes sampled yet.")))
        .children(top_cpu)
        .child(section_label("TOP MEMORY"))
        .children(top_mem)
        .child(div().h(px(8.0)))
}

fn sockets_card(s: &SocketSummary, app: Entity<CrowApp>) -> impl IntoElement {
    let ports = s.ports.iter().enumerate().map(|(i, port)| {
        let (app, id) = (app.clone(), port.socket_id.clone());
        let color = match port.exposure {
            Exposure::AllInterfaces => WARN,
            Exposure::Interface => TEXT_SECONDARY,
            Exposure::Local => TEXT_MUTED,
        };
        link_row(
            SharedString::from(format!("dash-port-{i}")),
            format!("{:<5} {:>5}  {}", port.protocol, port.port, port.process),
            format!("{} · {}", port.exposure.label(), port.address),
            color,
            move |_w, cx| {
                let id = id.clone();
                app.update(cx, |this, cx| this.open_socket_from_dashboard(&id, cx));
            },
        )
    });
    card("SOCKETS", "sockets", app.clone())
        .child(stats(vec![
            (s.listening.to_string(), "LISTENING", TEXT_PRIMARY),
            (s.all_interfaces.to_string(), "ALL INTERFACES", if s.all_interfaces > 0 { WARN } else { TEXT_MUTED }),
            (s.interface.to_string(), "INTERFACE", TEXT_SECONDARY),
            (s.local_only.to_string(), "LOCAL ONLY", OK),
        ]))
        .child(section_label("LISTENING PORTS · WIDEST EXPOSURE FIRST"))
        .children(s.ports.is_empty().then(|| empty_line("No listening sockets sampled yet.")))
        .children(ports)
        .child(div().h(px(8.0)))
}
