use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};

pub struct FleetHost {
    pub name: &'static str,
    pub ip: &'static str,
    pub role: &'static str,
    pub env: &'static str,
    pub env_bg: Rgba,
    pub env_fg: Rgba,
    pub cpu_pct: u8,
    pub cpu_label: &'static str,
    pub mem_pct: u8,
    pub mem_label: &'static str,
    pub disk: &'static str,
    pub uptime: &'static str,
    pub agent: &'static str,
    pub agent_color: Rgba,
    pub alerts: &'static str,
    pub alert_color: Rgba,
    pub status_color: Rgba,
    pub is_critical_border: bool,
    pub pill: &'static str,
    pub is_selected: bool,
}

pub fn default_fleet_hosts() -> Vec<FleetHost> {
    vec![
        FleetHost {
            name: "edge-01", ip: "159.223.84.17", role: "web · nginx", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 38, cpu_label: "38%", mem_pct: 73, mem_label: "73%",
            disk: "43%", uptime: "64d 07h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "2", alert_color: WARN, status_color: OK, is_critical_border: false, pill: "OK", is_selected: true,
        },
        FleetHost {
            name: "edge-02", ip: "159.223.84.22", role: "web · nginx", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 31, cpu_label: "31%", mem_pct: 61, mem_label: "61%",
            disk: "39%", uptime: "64d 07h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "0", alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK", is_selected: false,
        },
        FleetHost {
            name: "db-primary", ip: "10.0.4.11", role: "postgres 16", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 64, cpu_label: "64%", mem_pct: 82, mem_label: "82%",
            disk: "77%", uptime: "121d 03h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "3", alert_color: WARN, status_color: WARN, is_critical_border: false, pill: "DEGRADED", is_selected: false,
        },
        FleetHost {
            name: "db-replica-01", ip: "10.0.4.12", role: "postgres 16", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 22, cpu_label: "22%", mem_pct: 58, mem_label: "58%",
            disk: "74%", uptime: "121d 03h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "0", alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK", is_selected: false,
        },
        FleetHost {
            name: "redis-01", ip: "10.0.4.18", role: "cache · queue", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 47, cpu_label: "47%", mem_pct: 44, mem_label: "44%",
            disk: "18%", uptime: "89d 11h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "1", alert_color: WARN, status_color: WARN, is_critical_border: false, pill: "DEGRADED", is_selected: false,
        },
        FleetHost {
            name: "worker-04", ip: "10.0.4.31", role: "sidekiq", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 0, cpu_label: "—", mem_pct: 0, mem_label: "—",
            disk: "—", uptime: "—", agent: "0.8.1", agent_color: WARN,
            alerts: "2", alert_color: CRIT, status_color: CRIT, is_critical_border: true, pill: "UNREACHABLE", is_selected: false,
        },
        FleetHost {
            name: "worker-05", ip: "10.0.4.32", role: "sidekiq", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 71, cpu_label: "71%", mem_pct: 66, mem_label: "66%",
            disk: "31%", uptime: "12d 19h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "0", alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK", is_selected: false,
        },
        FleetHost {
            name: "metrics-01", ip: "10.0.4.40", role: "prometheus", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 18, cpu_label: "18%", mem_pct: 39, mem_label: "39%",
            disk: "62%", uptime: "31d 18h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "0", alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK", is_selected: false,
        },
        FleetHost {
            name: "bastion", ip: "159.223.84.9", role: "ssh jump", env: "PROD",
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 3, cpu_label: "3%", mem_pct: 12, mem_label: "12%",
            disk: "9%", uptime: "204d 02h", agent: "—", agent_color: TEXT_FAINT,
            alerts: "1", alert_color: WARN, status_color: TEXT_FAINT, is_critical_border: false, pill: "NO AGENT", is_selected: false,
        },
        FleetHost {
            name: "stage-web-01", ip: "10.1.2.11", role: "web · nginx", env: "STAGE",
            env_bg: WARN_BG, env_fg: WARN, cpu_pct: 12, cpu_label: "12%", mem_pct: 28, mem_label: "28%",
            disk: "22%", uptime: "8d 04h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "0", alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK", is_selected: false,
        },
        FleetHost {
            name: "stage-db-01", ip: "10.1.2.21", role: "postgres 16", env: "STAGE",
            env_bg: WARN_BG, env_fg: WARN, cpu_pct: 8, cpu_label: "8%", mem_pct: 34, mem_label: "34%",
            disk: "28%", uptime: "8d 04h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "0", alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK", is_selected: false,
        },
        FleetHost {
            name: "build-01", ip: "10.1.9.5", role: "ci runner", env: "DEV",
            env_bg: hex_rgb(0x1a1b21), env_fg: TEXT_MUTED, cpu_pct: 92, cpu_label: "92%", mem_pct: 71, mem_label: "71%",
            disk: "88%", uptime: "2d 06h", agent: "0.9.4", agent_color: TEXT_FAINT,
            alerts: "1", alert_color: WARN, status_color: WARN, is_critical_border: false, pill: "DEGRADED", is_selected: false,
        },
    ]
}

pub fn fleet_stat_strip() -> impl IntoElement {
    let stats = [
        ("SERVERS", "12", "", "3 regions · 4 groups", TEXT_PRIMARY),
        ("OPEN ALERTS", "7", "", "2 crit · 5 warn", WARN),
        ("CONFIG DRIFT", "3", " hosts", "authorized_keys, ufw", WARN),
        ("FLEET LOAD", "1.42", "", "weighted avg · 96 vCPU", TEXT_PRIMARY),
        ("OLDEST HOST KEY", "214", "d", "worker-04 · policy 90d", CRIT),
        ("AGENT COVERAGE", "11", "/12", "bastion pending install", TEXT_PRIMARY),
    ];

    div()
        .h(px(52.0))
        .flex_none()
        .flex()
        .items_stretch()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .children(stats.iter().map(|(label, val, unit, note, fg)| {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_center()
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .gap(px(2.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child(*label),
                )
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(2.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(*fg)
                                .child(*val),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_MUTED)
                                .child(*unit),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIMMER)
                                .ml(px(6.0))
                                .child(*note),
                        ),
                )
        }))
}

pub fn fleet_overview_view(app: Entity<CrowApp>) -> impl IntoElement {
    let hosts = default_fleet_hosts();

    let alerts = [
        ("CRIT", CRIT, CRIT_BG, "worker-04", "ssh handshake timeout — 6 consecutive probes failed", "4m"),
        ("CRIT", CRIT, CRIT_BG, "worker-04", "host key older than policy (214d > 90d)", "4m"),
        ("WARN", WARN, WARN_BG, "db-primary", "pg_hba.conf allows md5 from 0.0.0.0/0", "22m"),
        ("WARN", WARN, WARN_BG, "db-primary", "memory 82% — shared_buffers may be oversized", "1h"),
        ("WARN", WARN, WARN_BG, "build-01", "disk 88% — /var/lib/docker growing 4GB/day", "2h"),
        ("WARN", WARN, WARN_BG, "redis-01", "no persistence to disk (errno 28)", "3h"),
        ("WARN", WARN, WARN_BG, "bastion", "crow-agent not installed — telemetry unavailable", "1d"),
    ];

    let activities = [
        ("03:41:09", "maya", hex_rgb(0x8ab4ff), "staged pg_hba.conf edit on db-primary", TEXT_PRIMARY),
        ("03:38:52", "crow", TEXT_FAINT, "auto-rollback armed for db-primary", WARN),
        ("03:31:20", "maya", hex_rgb(0x8ab4ff), "opened tab edge-01", TEXT_MUTED),
        ("03:22:44", "crow", TEXT_FAINT, "probe failed worker-04 (attempt 6)", CRIT),
        ("03:14:02", "ravi", hex_rgb(0x8ab4ff), "restarted sidekiq@web on worker-05", TEXT_PRIMARY),
        ("03:12:36", "ravi", hex_rgb(0x8ab4ff), "saved fleet settings (3 keys)", TEXT_PRIMARY),
        ("02:58:11", "crow", TEXT_FAINT, "rotated host key on stage-web-01", OK),
        ("02:41:07", "crow", TEXT_FAINT, "snapshot edge-02 complete (1.2 GB)", TEXT_MUTED),
        ("02:30:55", "maya", hex_rgb(0x8ab4ff), "flushed ufw rule 7 on edge-01", WARN),
        ("02:11:19", "crow", TEXT_FAINT, "agent upgraded 0.9.3 → 0.9.4 on 10 hosts", OK),
        ("01:52:40", "ravi", hex_rgb(0x8ab4ff), "granted deploy access to sam@acme.dev (30d)", TEXT_PRIMARY),
        ("01:33:08", "crow", TEXT_FAINT, "baseline drift detected on /root/.ssh", WARN),
        ("01:20:44", "maya", hex_rgb(0x8ab4ff), "closed tab stage-db-01", TEXT_MUTED),
        ("00:58:02", "crow", TEXT_FAINT, "nightly backup verified on 12 hosts", OK),
    ];

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Fleet Stat Strip
        .child(fleet_stat_strip())
        // 2. Main content split: 12-host table on left, alerts/activity rail on right
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Left Table Panel
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        // Sub bar with filters
                        .child(
                            div()
                                .h(px(32.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(14.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(14.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(
                                    div()
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .bg(BG_ROW_SELECTED)
                                        .border_1()
                                        .border_color(BORDER_CONTROL_SEL)
                                        .text_color(TEXT_MAX)
                                        .font_weight(FontWeight::BOLD)
                                        .child("ALL (12)"),
                                )
                                .child(div().text_color(TEXT_MUTED).child("PRODUCTION (8)"))
                                .child(div().text_color(TEXT_MUTED).child("STAGING (2)"))
                                .child(div().text_color(TEXT_MUTED).child("EDGE"))
                                .child(div().text_color(TEXT_MUTED).child("DATA"))
                                .child(div().text_color(TEXT_MUTED).child("BY REGION"))
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIMMER)
                                        .child("12 hosts · sort health ↓ · click to open"),
                                ),
                        )
                        // Table Column Headers
                        .child(
                            div()
                                .h(px(26.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_SUBHEAD)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(TEXT_DIMMER)
                                .child(div().w(px(22.0)).child(""))
                                .child(div().w(px(160.0)).child("HOST"))
                                .child(div().w(px(120.0)).child("ADDRESS"))
                                .child(div().w(px(110.0)).child("ROLE"))
                                .child(div().w(px(60.0)).child("ENV"))
                                .child(div().w(px(95.0)).child("CPU"))
                                .child(div().w(px(95.0)).child("MEM"))
                                .child(div().w(px(55.0)).text_align(TextAlign::Right).child("DISK"))
                                .child(div().w(px(80.0)).text_align(TextAlign::Right).child("UPTIME"))
                                .child(div().w(px(70.0)).text_align(TextAlign::Right).child("AGENT"))
                                .child(div().w(px(65.0)).text_align(TextAlign::Right).child("ALERTS")),
                        )
                        // Table Body
                        .child(
                            div()
                                .id("fleet-host-list")
                                .flex_1()
                                .overflow_y_scroll()
                                .children(hosts.into_iter().enumerate().map(|(idx, host)| {
                                    let app_host = app.clone();
                                    let host_name = host.name;
                                    let is_even = idx % 2 == 0;

                                    div()
                                        .id(ElementId::NamedInteger("fleet-row".into(), idx as u64))
                                        .relative()
                                        .h(px(32.0))
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .px(px(12.0))
                                        .bg(if host.is_selected {
                                            BG_ROW_SELECTED
                                        } else if is_even {
                                            BG_APP
                                        } else {
                                            BG_ROW_ALT
                                        })
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .children(if host.is_selected {
                                            Some(left_indicator(TEXT_PRIMARY))
                                        } else if host.is_critical_border {
                                            Some(left_indicator(CRIT))
                                        } else {
                                            None
                                        })
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .cursor_pointer()
                                        .on_click(move |_ev, _window, cx| {
                                            app_host.update(cx, |this, cx| {
                                                this.switch_tab(host_name, cx);
                                                this.set_screen(Screen::Server, cx);
                                            });
                                        })
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        // Status dot / glyph
                                        .child(
                                            div()
                                                .w(px(22.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    div()
                                                        .size(px(6.0))
                                                        .rounded_full()
                                                        .bg(host.status_color),
                                                ),
                                        )
                                        // Host name + pill
                                        .child(
                                            div()
                                                .w(px(160.0))
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    div()
                                                        .font_weight(if host.is_selected {
                                                            FontWeight::BOLD
                                                        } else {
                                                            FontWeight::NORMAL
                                                        })
                                                        .text_color(if host.is_selected {
                                                            TEXT_MAX
                                                        } else {
                                                            TEXT_PRIMARY
                                                        })
                                                        .child(host.name),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(4.0))
                                                        .py(px(1.5))
                                                        .bg(if host.pill == "UNREACHABLE" {
                                                            CRIT_BG
                                                        } else if host.pill == "DEGRADED" {
                                                            WARN_BG
                                                        } else {
                                                            OK_BG
                                                        })
                                                        .text_color(if host.pill == "UNREACHABLE" {
                                                            CRIT
                                                        } else if host.pill == "DEGRADED" {
                                                            WARN
                                                        } else {
                                                            OK
                                                        })
                                                        .text_size(px(8.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .child(host.pill),
                                                ),
                                        )
                                        // Address
                                        .child(
                                            div()
                                                .w(px(120.0))
                                                .text_color(TEXT_DIM)
                                                .child(host.ip),
                                        )
                                        // Role
                                        .child(
                                            div()
                                                .w(px(110.0))
                                                .text_size(px(11.0))
                                                .text_color(TEXT_TERTIARY)
                                                .child(host.role),
                                        )
                                        // Env
                                        .child(
                                            div()
                                                .w(px(60.0))
                                                .child(
                                                    div()
                                                        .px(px(4.0))
                                                        .py(px(1.5))
                                                        .bg(host.env_bg)
                                                        .text_color(host.env_fg)
                                                        .text_size(px(8.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .child(host.env),
                                                ),
                                        )
                                        // CPU mini meter
                                        .child(
                                            div()
                                                .w(px(95.0))
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    div()
                                                        .w(px(34.0))
                                                        .h(px(3.0))
                                                        .bg(hex_rgb(0x1a1b21))
                                                        .flex_none()
                                                        .child(
                                                            div()
                                                                .w(px(34.0 * (host.cpu_pct as f32 / 100.0)))
                                                                .h(px(3.0))
                                                                .bg(if host.cpu_pct > 85 {
                                                                    CRIT
                                                                } else if host.cpu_pct > 60 {
                                                                    WARN
                                                                } else {
                                                                    OK
                                                                }),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child(host.cpu_label),
                                                ),
                                        )
                                        // MEM mini meter
                                        .child(
                                            div()
                                                .w(px(95.0))
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    div()
                                                        .w(px(34.0))
                                                        .h(px(3.0))
                                                        .bg(hex_rgb(0x1a1b21))
                                                        .flex_none()
                                                        .child(
                                                            div()
                                                                .w(px(34.0 * (host.mem_pct as f32 / 100.0)))
                                                                .h(px(3.0))
                                                                .bg(if host.mem_pct > 85 {
                                                                    CRIT
                                                                } else if host.mem_pct > 60 {
                                                                    WARN
                                                                } else {
                                                                    OK
                                                                }),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child(host.mem_label),
                                                ),
                                        )
                                        // Disk
                                        .child(
                                            div()
                                                .w(px(55.0))
                                                .text_align(TextAlign::Right)
                                                .text_color(if host.disk.starts_with("7") || host.disk.starts_with("8") {
                                                    WARN
                                                } else {
                                                    TEXT_PRIMARY
                                                })
                                                .child(host.disk),
                                        )
                                        // Uptime
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .text_align(TextAlign::Right)
                                                .text_color(TEXT_DIM)
                                                .child(host.uptime),
                                        )
                                        // Agent
                                        .child(
                                            div()
                                                .w(px(70.0))
                                                .text_align(TextAlign::Right)
                                                .text_size(px(10.5))
                                                .text_color(host.agent_color)
                                                .child(host.agent),
                                        )
                                        // Alerts
                                        .child(
                                            div()
                                                .w(px(65.0))
                                                .text_align(TextAlign::Right)
                                                .font_weight(if host.alerts == "0" {
                                                    FontWeight::NORMAL
                                                } else {
                                                    FontWeight::BOLD
                                                })
                                                .text_color(host.alert_color)
                                                .child(host.alerts),
                                        )
                                })),
                        ),
                )
                // Right Rail: Fleet Alerts (top) & Activity Log (bottom)
                .child(
                    div()
                        .w(px(380.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
                        // Alerts Header
                        .child(
                            div()
                                .h(px(34.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("FLEET ALERTS"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .child("7 open"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .child("ack all ⇧A"),
                                ),
                        )
                        // Alerts List
                        .child(
                            div()
                                .flex_none()
                                .flex()
                                .flex_col()
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .children(alerts.iter().map(|(lvl, fg, bg, host, msg, age)| {
                                    div()
                                        .relative()
                                        .flex()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .px(px(12.0))
                                        .py(px(7.0))
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .bg(if *lvl == "CRIT" { CRIT_LOG_BG } else { hex_rgba(0, 0.0) })
                                        .children(if *lvl == "CRIT" {
                                            Some(left_indicator(CRIT))
                                        } else {
                                            None
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(8.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .px(px(4.0))
                                                        .py(px(1.5))
                                                        .bg(*bg)
                                                        .text_color(*fg)
                                                        .child(*lvl),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child(*host),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .text_color(TEXT_FAINT)
                                                        .child(*age),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .text_color(if *lvl == "CRIT" { CRIT_INK } else { WARN_INK })
                                                .child(*msg),
                                        )
                                })),
                        )
                        // Activity Header
                        .child(
                            div()
                                .h(px(30.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("ACTIVITY"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .child("audit log · all hosts"),
                                ),
                        )
                        // Activity Log List
                        .child(
                            div()
                                .id("fleet-activity-log")
                                .flex_1()
                                .overflow_y_scroll()
                                .py(px(5.0))
                                .children(activities.iter().map(|(ts, actor, actor_color, msg, text_c)| {
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .px(px(12.0))
                                        .py(px(2.0))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .line_height(relative(1.45))
                                        .child(
                                            div()
                                                .text_color(TEXT_FAINTER)
                                                .flex_none()
                                                .child(*ts),
                                        )
                                        .child(
                                            div()
                                                .w(px(52.0))
                                                .text_color(*actor_color)
                                                .flex_none()
                                                .child(*actor),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_color(*text_c)
                                                .child(*msg),
                                        )
                                })),
                        ),
                ),
        )
        // 3. Persistent Fleet-Wide Destructive Strip with Abort Gate
        .child(
            div()
                .h(px(46.0))
                .flex_none()
                .flex()
                .items_stretch()
                .bg(CRIT_STRIP_BG)
                .border_t_1()
                .border_color(BORDER_DANGER)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(14.0))
                        .border_r_1()
                        .border_color(BORDER_DANGER)
                        .child(
                            div()
                                .w(px(10.0))
                                .h(px(10.0))
                                .bg(CRIT),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(CRIT)
                                .child("FLEET-WIDE"),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(14.0))
                        .child(
                            div()
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(BORDER_DANGER_BTN)
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(CRIT_INK_DIM)
                                .child("ROLLING REBOOT · 12 HOSTS"),
                        )
                        .child(
                            div()
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(BORDER_DANGER_BTN)
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(CRIT_INK_DIM)
                                .child("ROTATE ALL HOST KEYS"),
                        )
                        .child(
                            div()
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(BORDER_DANGER_BTN)
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(CRIT_INK_DIM)
                                .child("REVOKE ALL SESSIONS"),
                        )
                        .child(
                            div()
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(BORDER_DANGER_BTN)
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(CRIT_INK_DIM)
                                .child("PUSH BASELINE TO ALL"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .px(px(14.0))
                        .border_l_1()
                        .border_color(BORDER_DANGER)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("fleet actions run serially with a per-host abort gate"),
                ),
        )
}
