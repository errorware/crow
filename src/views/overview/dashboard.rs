//! Overview dashboard: the headline numbers from the Services, Processes and
//! Sockets pages, plus pending updates and the CVEs they fix. Every item
//! links to its row on that page (CVEs open on osv.dev).

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;

use crate::app::CrowApp;
use crate::theme::*;

use super::summary::{summarize_processes, summarize_services, summarize_sockets, Exposure, ProcessSummary, ServiceSummary, SocketSummary};
use super::state::SecurityState;
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
                // Services with Sockets under it, Processes, then Updates & Security.
                .child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap(px(12.0)).child(services_card(&services, app.clone())).child(sockets_card(&sockets, app.clone())))
                .child(div().flex_1().min_w(px(0.0)).flex().flex_col().child(processes_card(&processes, app.clone())))
                .child(div().flex_1().min_w(px(0.0)).flex().flex_col().child(security_card(&overview.security, app))),
        )
}

fn card(title: &'static str, view: &'static str, app: Entity<CrowApp>) -> Div {
    div()
        .w_full()
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

/// Pending package updates (from the server's package manager) and the CVEs
/// they fix (from OSV.dev).
fn security_card(sec: &SecurityState, app: Entity<CrowApp>) -> impl IntoElement {
    let app_refresh = app.clone();
    let header = div()
        .h(px(34.0))
        .flex()
        .items_center()
        .px(px(12.0))
        .border_b_1()
        .border_color(BORDER_PANEL)
        .child(div().font_family(FONT_MONO).text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("UPDATES & SECURITY"))
        .child(div().flex_1())
        .children((!sec.checked_label.is_empty()).then(|| div().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_FAINT).mr(px(10.0)).child(format!("checked {}", sec.checked_label))))
        .child(
            div()
                .id("security-refresh")
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(if sec.loading { TEXT_FAINT } else { hex_rgb(0x8ab4ff) })
                .when(!sec.loading, |d| d.cursor_pointer().hover(|s| s.text_color(TEXT_PRIMARY)).on_click(move |_ev, _window, cx| app_refresh.update(cx, |this, cx| this.refresh_security(true, cx))))
                .child(if sec.loading { "CHECKING…" } else { "REFRESH ↻" }),
        );
    let body = div().flex().flex_col();
    let body = match &sec.updates {
        None if sec.loading => body.child(empty_line("Reading the package manager and looking up CVEs…")),
        None => body.child(empty_line("Not checked yet.")),
        Some(Err(e)) => body.child(div().px(px(12.0)).py(px(8.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(CRIT).child(e.clone())),
        Some(Ok(u)) => {
            let security = u.security_count();
            let fixable = sec.cves.as_ref().and_then(|c| c.as_ref().ok()).map(|c| c.fixable_count());
            let mut b = body.child(stats(vec![
                (u.updates.len().to_string(), "PENDING", if u.updates.is_empty() { OK } else { TEXT_PRIMARY }),
                (security.to_string(), "SECURITY", if security > 0 { CRIT } else { TEXT_MUTED }),
                (fixable.map(|n| n.to_string()).unwrap_or_else(|| "—".into()), "CVES FIXED BY UPDATING", if fixable.unwrap_or(0) > 0 { WARN } else { TEXT_MUTED }),
            ]));

            // Kernel and reboot state.
            b = b.child(section_label("KERNEL"));
            b = b.child(info_line(format!("running {}", if u.kernel.is_empty() { "—" } else { &u.kernel }), TEXT_SECONDARY));
            if let Some((change, is_security)) = u.kernel_update() {
                b = b.child(info_line(format!("{} update pending: {change}", if is_security { "security" } else { "kernel" }), if is_security { CRIT } else { WARN }));
            }
            if u.reboot_required {
                b = b.child(info_line("reboot required to finish installed updates".into(), WARN));
            }
            if let Some(t) = u.cache_time {
                let age_h = (chrono::Utc::now().timestamp() - t).max(0) / 3600;
                let (text, color) = match age_h {
                    0..=23 => (format!("package lists refreshed {}h ago", age_h), TEXT_FAINT),
                    h => (format!("package lists are {} days old — run `apt update` for current results", h / 24), WARN),
                };
                b = b.child(info_line(text, color));
            }

            // CVEs the pending updates fix, most severe first.
            match &sec.cves {
                Some(Ok(c)) => {
                    let mut fixed: Vec<(String, &crate::security::osv::VulnInfo)> = c
                        .fixed_by_upgrade
                        .iter()
                        .flat_map(|(pkg, ids)| ids.iter().filter_map(|id| c.details.get(id).map(|v| (pkg.clone(), v))))
                        .collect();
                    fixed.sort_by_key(|(_, v)| v.rank());
                    b = b.child(section_label("CVES FIXED BY PENDING UPDATES"));
                    if fixed.is_empty() {
                        b = b.child(empty_line("None found for the pending updates."));
                    }
                    for (i, (pkg, v)) in fixed.iter().take(10).enumerate() {
                        let url = format!("https://osv.dev/vulnerability/{}", v.id);
                        let color = match v.rank() { 0 | 1 => CRIT, 2 => WARN, _ => TEXT_SECONDARY };
                        b = b
                            .child(link_row(SharedString::from(format!("dash-cve-{i}")), format!("{}  {}", v.label(), pkg), v.severity.clone().unwrap_or_else(|| "unrated".into()), color, move |_w, cx| cx.open_url(&url)))
                            .children((!v.summary.is_empty()).then(|| {
                                div().px(px(12.0)).pl(px(24.0)).pb(px(4.0)).overflow_hidden().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_FAINT).child(v.summary.clone())
                            }));
                    }
                    if fixed.len() > 10 {
                        b = b.child(empty_line_owned(format!("+{} more", fixed.len() - 10)));
                    }
                    if c.no_fix_yet > 0 {
                        b = b.child(info_line(format!("{} known CVEs in running software have no fix yet", c.no_fix_yet), TEXT_FAINT));
                    }
                }
                Some(Err(e)) => b = b.child(section_label("CVES")).child(info_line(e.clone(), TEXT_FAINT)),
                None => {}
            }

            // Security updates, then the rest.
            let mut pending: Vec<&super::updates::PackageUpdate> = u.updates.iter().collect();
            pending.sort_by_key(|p| (!p.security, p.name.clone()));
            b = b.child(section_label("PENDING UPDATES · SECURITY FIRST"));
            if pending.is_empty() {
                b = b.child(empty_line("Everything is up to date."));
            }
            for p in pending.iter().take(12) {
                let versions = if p.installed.is_empty() { p.candidate.clone() } else { format!("{} → {}", p.installed, p.candidate) };
                b = b.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(12.0))
                        .py(px(3.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .child(div().flex_1().min_w(px(0.0)).overflow_hidden().text_color(if p.security { TEXT_PRIMARY } else { TEXT_SECONDARY }).child(p.name.clone()))
                        .children(p.security.then(|| div().flex_none().text_size(px(8.5)).font_weight(FontWeight::BOLD).text_color(CRIT).child("SECURITY")))
                        .child(div().flex_none().max_w(px(220.0)).overflow_hidden().text_color(TEXT_FAINT).child(versions)),
                );
            }
            if pending.len() > 12 {
                b = b.child(empty_line_owned(format!("+{} more", pending.len() - 12)));
            }
            b.child(div().px(px(12.0)).pt(px(8.0)).font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_FAINTER).child("CVE data: OSV.dev · Crow sends package names and versions only, never host names"))
        }
    };
    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .rounded_md()
        .child(header)
        .child(body)
        .child(div().h(px(8.0)))
}

fn info_line(text: String, color: Rgba) -> Div {
    div().px(px(12.0)).py(px(3.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(color).child(text)
}

fn empty_line_owned(text: String) -> Div {
    div().px(px(12.0)).py(px(4.0)).font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_FAINT).child(text)
}
