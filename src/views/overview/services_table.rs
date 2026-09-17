use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};

#[derive(Clone, Debug)]
pub struct ServiceUnit {
    pub name: &'static str,
    pub status: &'static str,
    pub status_color: Rgba,
    pub pid: &'static str,
    pub cpu: &'static str,
    pub mem: &'static str,
    pub rss: &'static str,
    pub uptime: &'static str,
    pub is_focused: bool,
    pub show_confirm: bool,
}

pub fn default_services() -> Vec<ServiceUnit> {
    vec![
        ServiceUnit { name: "nginx.service", status: "ACTIVE", status_color: OK, pid: "1412", cpu: "24.8", mem: "6.2", rss: "1.0G", uptime: "18d 04h", is_focused: true, show_confirm: true },
        ServiceUnit { name: "postgresql@16-main", status: "ACTIVE", status_color: OK, pid: "1189", cpu: "18.1", mem: "31.4", rss: "5.0G", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "redis-server.service", status: "DEGRADED", status_color: WARN, pid: "1902", cpu: "9.4", mem: "4.1", rss: "672M", uptime: "2h 14m", is_focused: false, show_confirm: false },
        ServiceUnit { name: "docker.service", status: "ACTIVE", status_color: OK, pid: "993", cpu: "7.2", mem: "8.8", rss: "1.4G", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "crow-agent.service", status: "ACTIVE", status_color: OK, pid: "20441", cpu: "5.6", mem: "1.2", rss: "196M", uptime: "3d 11h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "sidekiq@web.service", status: "ACTIVE", status_color: OK, pid: "8812", cpu: "4.9", mem: "6.7", rss: "1.1G", uptime: "7d 02h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "clamav-freshclam", status: "FAILED", status_color: CRIT, pid: "—", cpu: "0.0", mem: "0.0", rss: "—", uptime: "—", is_focused: false, show_confirm: false },
        ServiceUnit { name: "prometheus.service", status: "ACTIVE", status_color: OK, pid: "2210", cpu: "3.8", mem: "5.4", rss: "864M", uptime: "31d 18h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "grafana-server", status: "ACTIVE", status_color: OK, pid: "2291", cpu: "2.4", mem: "3.9", rss: "620M", uptime: "31d 18h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "containerd.service", status: "ACTIVE", status_color: OK, pid: "981", cpu: "2.1", mem: "2.2", rss: "352M", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "ssh.service", status: "ACTIVE", status_color: OK, pid: "764", cpu: "1.4", mem: "0.4", rss: "64M", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "fail2ban.service", status: "DEGRADED", status_color: WARN, pid: "1077", cpu: "1.1", mem: "0.9", rss: "148M", uptime: "9h 41m", is_focused: false, show_confirm: false },
        ServiceUnit { name: "cron.service", status: "ACTIVE", status_color: OK, pid: "812", cpu: "0.8", mem: "0.2", rss: "32M", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "systemd-journald", status: "ACTIVE", status_color: OK, pid: "411", cpu: "0.7", mem: "1.1", rss: "176M", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "unattended-upgrades", status: "PENDING", status_color: WARN, pid: "3110", cpu: "0.4", mem: "0.6", rss: "96M", uptime: "4h 02m", is_focused: false, show_confirm: false },
        ServiceUnit { name: "chrony.service", status: "ACTIVE", status_color: OK, pid: "722", cpu: "0.2", mem: "0.1", rss: "18M", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "ufw.service", status: "ACTIVE", status_color: OK, pid: "688", cpu: "0.0", mem: "0.1", rss: "12M", uptime: "64d 07h", is_focused: false, show_confirm: false },
        ServiceUnit { name: "logrotate.timer", status: "ACTIVE", status_color: OK, pid: "—", cpu: "0.0", mem: "0.0", rss: "—", uptime: "64d 07h", is_focused: false, show_confirm: false },
    ]
}

pub fn services_table(
    services: &[ServiceUnit],
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
        // Panel Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child({
                    let app_clone = app.clone();
                    div()
                        .id("subtab-services")
                        .px(px(12.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_SANS)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .font_weight(if active_tab == "services" { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if active_tab == "services" { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .bg(if active_tab == "services" { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .on_click(move |_ev, _window, cx| {
                            app_clone.update(cx, |this, cx| {
                                this.set_services_tab("services", cx);
                            });
                        })
                        .child("SERVICES")
                })
                .child({
                    let app_clone = app.clone();
                    div()
                        .id("subtab-processes")
                        .px(px(12.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_SANS)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .font_weight(if active_tab == "processes" { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if active_tab == "processes" { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .bg(if active_tab == "processes" { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .on_click(move |_ev, _window, cx| {
                            app_clone.update(cx, |this, cx| {
                                this.set_services_tab("processes", cx);
                            });
                        })
                        .child("PROCESSES")
                })
                .child({
                    let app_clone = app.clone();
                    div()
                        .id("subtab-sockets")
                        .px(px(12.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_SANS)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .font_weight(if active_tab == "sockets" { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(if active_tab == "sockets" { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .bg(if active_tab == "sockets" { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .on_click(move |_ev, _window, cx| {
                            app_clone.update(cx, |this, cx| {
                                this.set_services_tab("sockets", cx);
                            });
                        })
                        .child("SOCKETS")
                })
                .child(div().flex_1())
                // Counts cluster
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .px(px(12.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().text_color(TEXT_DIMMER).child("42 units"))
                        .child(div().text_color(OK).child("38 active"))
                        .child(div().text_color(WARN).child("3 degraded"))
                        .child(div().text_color(CRIT).child("1 failed")),
                )
                // Sort section
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
                                .font_family("Inter")
                                .text_size(px(11.0))
                                .text_color(TEXT_TERTIARY)
                                .child("sort cpu ↓"),
                        )
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(10.0))
                                .text_color(TEXT_MUTED)
                                .bg(BG_KEY)
                                .border_1()
                                .border_color(BORDER_KEY)
                                .px(px(5.0))
                                .py(px(1.0))
                                .child("f"),
                        ),
                ),
        )
        // Column Header
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .bg(BG_SUBHEAD)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family("Inter")
                .text_size(px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(div().w(px(26.0)))
                .child(div().flex_1().min_w(px(0.0)).child("UNIT"))
                .child(div().w(px(96.0)).child("PID"))
                .child(div().w(px(78.0)).text_right().child("CPU %"))
                .child(div().w(px(72.0)).text_right().child("MEM %"))
                .child(div().w(px(72.0)).text_right().child("RSS"))
                .child(div().w(px(92.0)).text_right().child("UPTIME"))
                .child(div().w(px(118.0)).text_right().pr(px(12.0)).child("ACTIONS")),
        )
        // Table body
        .child(
            div()
                .id("services-table-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(services.iter().enumerate().map(|(idx, svc)| {
                    let is_focus = svc.is_focused;
                    let is_failed = svc.status == "FAILED";
                    let cpu_num: f32 = svc.cpu.parse().unwrap_or(0.0);
                    let pill_bg = match svc.status {
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

                    let svc_name = svc.name;
                    let app_row = app.clone();
                    let app_restart = app.clone();
                    let app_cancel = app.clone();
                    let app_confirm = app.clone();

                    div()
                        .id(ElementId::NamedInteger("service-row".into(), idx as u64))
                        .w_full()
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .child(
                            div()
                                .id(ElementId::NamedInteger("service-row-inner".into(), idx as u64))
                                .flex()
                                .items_center()
                                .h(px(29.0))
                                .bg(row_bg)
                                .border_l_2()
                                .border_color(if is_focus { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
                                .font_family("JetBrains Mono")
                                .text_size(px(11.5))
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_row.update(cx, |this, cx| {
                                        this.focus_service(svc_name, cx);
                                    });
                                })
                                // Col 1: Glyph
                                .child(
                                    div()
                                        .w(px(26.0))
                                        .text_center()
                                        .text_size(px(9.0))
                                        .text_color(svc.status_color)
                                        .child(if is_failed { "■" } else { "●" }),
                                )
                                // Col 2: Name + State Pill
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
                                                .child(svc.name),
                                        )
                                        .child(
                                            div()
                                                .bg(pill_bg)
                                                .text_color(svc.status_color)
                                                .text_size(px(9.0))
                                                .font_weight(FontWeight::BOLD)
                                                .px(px(5.0))
                                                .py(px(2.0))
                                                .flex_none()
                                                .child(svc.status),
                                        ),
                                )
                                // Col 3: PID
                                .child(
                                    div()
                                        .w(px(96.0))
                                        .text_color(TEXT_DIM)
                                        .child(svc.pid),
                                )
                                // Col 4: CPU %
                                .child(
                                    div()
                                        .w(px(78.0))
                                        .text_right()
                                        .text_color(if cpu_num > 15.0 { TEXT_MAX } else { TEXT_SECONDARY })
                                        .child(svc.cpu),
                                )
                                // Col 5: MEM %
                                .child(
                                    div()
                                        .w(px(72.0))
                                        .text_right()
                                        .text_color(TEXT_SECONDARY)
                                        .child(svc.mem),
                                )
                                // Col 6: RSS
                                .child(
                                    div()
                                        .w(px(72.0))
                                        .text_right()
                                        .text_color(TEXT_DIM)
                                        .child(svc.rss),
                                )
                                // Col 7: UPTIME
                                .child(
                                    div()
                                        .w(px(92.0))
                                        .text_right()
                                        .text_color(TEXT_DIM)
                                        .child(svc.uptime),
                                )
                                // Col 8: ACTIONS
                                .child(
                                    div()
                                        .w(px(118.0))
                                        .flex()
                                        .justify_end()
                                        .gap(px(2.0))
                                        .pr(px(10.0))
                                        .text_size(px(10.5))
                                        .text_color(if is_focus { TEXT_SECONDARY } else { TEXT_FAINT })
                                        .child(
                                            div()
                                                .id(ElementId::NamedInteger("btn-restart".into(), idx as u64))
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .border_1()
                                                .border_color(if is_focus { BORDER_STRONG } else { BORDER_ROW })
                                                .cursor_pointer()
                                                .on_click(move |_ev, _window, cx| {
                                                    app_restart.update(cx, |this, cx| {
                                                        this.toggle_service_confirm(svc_name, cx);
                                                    });
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
                                        )
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .border_1()
                                                .border_color(if is_focus { BORDER_STRONG } else { BORDER_ROW })
                                                .child(
                                                    tabler_icon(TablerIcon::Dots)
                                                        .size(px(10.0))
                                                        .text_color(if is_focus { TEXT_SECONDARY } else { TEXT_FAINT }),
                                                ),
                                        ),
                                ),
                        )
                        // Inline Destructive Confirmation row if show_confirm is active
                        .children(if svc.show_confirm {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("confirm-row".into(), idx as u64))
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .h(px(34.0))
                                    .pl(px(28.0))
                                    .pr(px(12.0))
                                    .bg(CRIT_ROW_BG)
                                    .border_l_2()
                                    .border_color(CRIT)
                                    .child(
                                        div()
                                            .font_family("JetBrains Mono")
                                            .text_size(px(11.5))
                                            .text_color(CRIT_INK)
                                            .child(format!("Restart {}? 412 active connections will be dropped.", svc.name)),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        div()
                                            .id(ElementId::NamedInteger("btn-cancel-confirm".into(), idx as u64))
                                            .font_family("JetBrains Mono")
                                            .text_size(px(10.5))
                                            .text_color(TEXT_TERTIARY)
                                            .border_1()
                                            .border_color(BORDER_KEY)
                                            .px(px(8.0))
                                            .py(px(3.0))
                                            .cursor_pointer()
                                            .on_click(move |_ev, _window, cx| {
                                                app_cancel.update(cx, |this, cx| {
                                                    this.toggle_service_confirm(svc_name, cx);
                                                });
                                            })
                                            .child("Cancel ")
                                            .child(div().text_color(TEXT_DIMMER).child("esc")),
                                    )
                                    .child(
                                        div()
                                            .id(ElementId::NamedInteger("btn-do-restart".into(), idx as u64))
                                            .font_family("JetBrains Mono")
                                            .text_size(px(10.5))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(hex_rgb(0x0a0a0c))
                                            .bg(CRIT)
                                            .px(px(9.0))
                                            .py(px(4.0))
                                            .cursor_pointer()
                                            .on_click(move |_ev, _window, cx| {
                                                app_confirm.update(cx, |this, cx| {
                                                    this.toggle_service_confirm(svc_name, cx);
                                                });
                                            })
                                            .child("RESTART ⏎"),
                                    )
                            )
                        } else {
                            None
                        })
                })),
        )
}
