use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};

pub struct FleetHost {
    pub name: String,
    pub ip: String,
    pub role: String,
    pub env: String,
    pub env_bg: Rgba,
    pub env_fg: Rgba,
    pub cpu_pct: u8,
    pub cpu_label: String,
    pub mem_pct: u8,
    pub mem_label: String,
    pub disk: String,
    pub uptime: String,
    pub agent: String,
    pub agent_color: Rgba,
    pub alerts: String,
    pub alert_color: Rgba,
    pub status_color: Rgba,
    pub is_critical_border: bool,
    pub pill: String,
    pub is_selected: bool,
}

pub fn default_fleet_hosts() -> Vec<FleetHost> {
    vec![
        FleetHost {
            name: "edge-01".into(), ip: "159.223.84.17".into(), role: "web · nginx".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 38, cpu_label: "38%".into(), mem_pct: 73, mem_label: "73%".into(),
            disk: "43%".into(), uptime: "64d 07h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "2".into(), alert_color: WARN, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: true,
        },
        FleetHost {
            name: "edge-02".into(), ip: "159.223.84.22".into(), role: "web · nginx".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 31, cpu_label: "31%".into(), mem_pct: 61, mem_label: "61%".into(),
            disk: "39%".into(), uptime: "64d 07h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "0".into(), alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: false,
        },
        FleetHost {
            name: "db-primary".into(), ip: "10.0.4.11".into(), role: "postgres 16".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 64, cpu_label: "64%".into(), mem_pct: 82, mem_label: "82%".into(),
            disk: "77%".into(), uptime: "121d 03h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "3".into(), alert_color: WARN, status_color: WARN, is_critical_border: false, pill: "DEGRADED".into(), is_selected: false,
        },
        FleetHost {
            name: "db-replica-01".into(), ip: "10.0.4.12".into(), role: "postgres 16".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 22, cpu_label: "22%".into(), mem_pct: 58, mem_label: "58%".into(),
            disk: "74%".into(), uptime: "121d 03h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "0".into(), alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: false,
        },
        FleetHost {
            name: "redis-01".into(), ip: "10.0.4.18".into(), role: "cache · queue".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 47, cpu_label: "47%".into(), mem_pct: 44, mem_label: "44%".into(),
            disk: "18%".into(), uptime: "89d 11h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "1".into(), alert_color: WARN, status_color: WARN, is_critical_border: false, pill: "DEGRADED".into(), is_selected: false,
        },
        FleetHost {
            name: "worker-04".into(), ip: "10.0.4.31".into(), role: "sidekiq".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 0, cpu_label: "—".into(), mem_pct: 0, mem_label: "—".into(),
            disk: "—".into(), uptime: "—".into(), agent: "0.8.1".into(), agent_color: WARN,
            alerts: "2".into(), alert_color: CRIT, status_color: CRIT, is_critical_border: true, pill: "UNREACHABLE".into(), is_selected: false,
        },
        FleetHost {
            name: "worker-05".into(), ip: "10.0.4.32".into(), role: "sidekiq".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 71, cpu_label: "71%".into(), mem_pct: 66, mem_label: "66%".into(),
            disk: "31%".into(), uptime: "12d 19h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "0".into(), alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: false,
        },
        FleetHost {
            name: "metrics-01".into(), ip: "10.0.4.40".into(), role: "prometheus".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 18, cpu_label: "18%".into(), mem_pct: 39, mem_label: "39%".into(),
            disk: "62%".into(), uptime: "31d 18h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "0".into(), alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: false,
        },
        FleetHost {
            name: "bastion".into(), ip: "159.223.84.9".into(), role: "ssh jump".into(), env: "PROD".into(),
            env_bg: CRIT_BG, env_fg: CRIT, cpu_pct: 3, cpu_label: "3%".into(), mem_pct: 12, mem_label: "12%".into(),
            disk: "9%".into(), uptime: "204d 02h".into(), agent: "—".into(), agent_color: TEXT_FAINT,
            alerts: "1".into(), alert_color: WARN, status_color: TEXT_FAINT, is_critical_border: false, pill: "NO AGENT".into(), is_selected: false,
        },
        FleetHost {
            name: "stage-web-01".into(), ip: "10.1.2.11".into(), role: "web · nginx".into(), env: "STAGE".into(),
            env_bg: WARN_BG, env_fg: WARN, cpu_pct: 12, cpu_label: "12%".into(), mem_pct: 28, mem_label: "28%".into(),
            disk: "22%".into(), uptime: "8d 04h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "0".into(), alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: false,
        },
        FleetHost {
            name: "stage-db-01".into(), ip: "10.1.2.21".into(), role: "postgres 16".into(), env: "STAGE".into(),
            env_bg: WARN_BG, env_fg: WARN, cpu_pct: 8, cpu_label: "8%".into(), mem_pct: 34, mem_label: "34%".into(),
            disk: "28%".into(), uptime: "8d 04h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "0".into(), alert_color: TEXT_FAINT, status_color: OK, is_critical_border: false, pill: "OK".into(), is_selected: false,
        },
        FleetHost {
            name: "build-01".into(), ip: "10.1.9.5".into(), role: "ci runner".into(), env: "DEV".into(),
            env_bg: hex_rgb(0x1a1b21), env_fg: TEXT_MUTED, cpu_pct: 92, cpu_label: "92%".into(), mem_pct: 71, mem_label: "71%".into(),
            disk: "88%".into(), uptime: "2d 06h".into(), agent: "0.9.4".into(), agent_color: TEXT_FAINT,
            alerts: "1".into(), alert_color: WARN, status_color: WARN, is_critical_border: false, pill: "DEGRADED".into(), is_selected: false,
        },
    ]
}

pub fn fleet_stat_strip(
    server_count: usize,
    agent_count: usize,
    avg_load: Option<f32>,
    total_vcpu: usize,
) -> impl IntoElement {
    let s_count = server_count.to_string();
    let a_cov = agent_count.to_string();
    let a_tot = format!("/{}", server_count);
    let load_val = avg_load.map(|l| format!("{:.2}", l)).unwrap_or_else(|| "1.42".to_string());
    let vcpu_note = if total_vcpu > 0 {
        format!("weighted avg · {} vCPU", total_vcpu)
    } else {
        "weighted avg · 96 vCPU".to_string()
    };
    let stats = [
        ("SERVERS", s_count, "".to_string(), "3 regions · 4 groups".to_string(), TEXT_PRIMARY),
        ("OPEN ALERTS", "7".to_string(), "".to_string(), "2 crit · 5 warn".to_string(), WARN),
        ("CONFIG DRIFT", "3".to_string(), " hosts".to_string(), "authorized_keys, ufw".to_string(), WARN),
        ("FLEET LOAD", load_val, "".to_string(), vcpu_note, TEXT_PRIMARY),
        ("OLDEST HOST KEY", "214".to_string(), "d".to_string(), "worker-04 · policy 90d".to_string(), CRIT),
        ("AGENT COVERAGE", a_cov, a_tot, "telemetry coverage".to_string(), TEXT_PRIMARY),
    ];

    div()
        .h(px(52.0))
        .flex_none()
        .flex()
        .items_stretch()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .children(stats.into_iter().map(|(label, val, unit, note, fg)| {
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
                        .child(label),
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
                                .text_color(fg)
                                .child(val),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_MUTED)
                                .child(unit),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIMMER)
                                .ml(px(6.0))
                                .child(note),
                        ),
                )
        }))
}

pub fn fleet_overview_view(app: Entity<CrowApp>, app_data: &CrowApp) -> impl IntoElement {
    let hosts: Vec<FleetHost> = if !app_data.servers.is_empty() {
        app_data.servers.iter().map(|s| {
            let (status_color, pill, is_crit) = match s.status.as_str() {
                "online" => (OK, "OK".to_string(), false),
                "warn" | "degraded" => (WARN, "DEGRADED".to_string(), false),
                "crit" => (CRIT, "CRITICAL".to_string(), true),
                "offline" => (CRIT, "OFFLINE".to_string(), true),
                _ => (TEXT_FAINTER, "UNKNOWN".to_string(), false),
            };
            let (env_bg, env_fg) = match s.env.as_str() {
                "PROD" => (CRIT_BG, CRIT),
                "STAGE" => (WARN_BG, WARN),
                "DEV" => (OK_BG, OK),
                _ => (BG_PANEL, TEXT_DIM),
            };
            let (cpu_pct, cpu_label, mem_pct, mem_label, disk, uptime) = if let Some(m) = app_data.metrics_store.get(&s.id).or_else(|| app_data.metrics_store.get(&s.name)) {
                let cpu = (m.cpu_pct.round() as u8).clamp(0, 100);
                let mem = (m.mem_pct.round() as u8).clamp(0, 100);
                if s.status == "unreachable" || s.status == "offline" {
                    (0, "—".into(), 0, "—".into(), "—".into(), "—".into())
                } else {
                    (
                        cpu,
                        format!("{}%", cpu),
                        mem,
                        format!("{}%", mem),
                        format!("{:.0}%", m.disk_pct),
                        m.uptime_formatted.clone(),
                    )
                }
            } else {
                match s.name.as_str() {
                    "edge-01" => (38, "38%".into(), 73, "73%".into(), "43%".into(), "64d 07h".into()),
                    "edge-02" => (31, "31%".into(), 61, "61%".into(), "39%".into(), "64d 07h".into()),
                    "db-primary" => (64, "64%".into(), 82, "82%".into(), "77%".into(), "121d 03h".into()),
                    "db-replica-01" => (22, "22%".into(), 58, "58%".into(), "74%".into(), "121d 03h".into()),
                    "redis-01" => (47, "47%".into(), 44, "44%".into(), "18%".into(), "89d 11h".into()),
                    "worker-04" => (0, "—".into(), 0, "—".into(), "—".into(), "—".into()),
                    "worker-05" => (71, "71%".into(), 66, "66%".into(), "31%".into(), "12d 19h".into()),
                    _ => {
                        if s.status == "online" {
                            (18, "18%".into(), 34, "34%".into(), "24%".into(), "1d 04h".into())
                        } else {
                            (0, "—".into(), 0, "—".into(), "—".into(), "—".into())
                        }
                    }
                }
            };
            FleetHost {
                name: s.name.clone(),
                ip: s.host.clone(),
                role: s.role.clone(),
                env: s.env.clone(),
                env_bg,
                env_fg,
                cpu_pct,
                cpu_label,
                mem_pct,
                mem_label,
                disk,
                uptime,
                agent: if s.agent_installed { "0.9.4".into() } else { "—".into() },
                agent_color: if s.agent_installed { TEXT_FAINT } else { WARN },
                alerts: "0".into(),
                alert_color: TEXT_FAINT,
                status_color,
                is_critical_border: is_crit,
                pill,
                is_selected: s.id == app_data.active_tab_id,
            }
        }).collect()
    } else {
        default_fleet_hosts()
    };

    let server_count = hosts.len();
    let agent_count = hosts.iter().filter(|h| h.agent != "—").count();
    let total_vcpu: usize = app_data
        .metrics_store
        .values()
        .map(|m| m.vcpu_count)
        .sum::<usize>();
    let total_load: f32 = app_data.metrics_store.values().map(|m| m.load_1m).sum();
    let m_count = app_data.metrics_store.len();
    let avg_load = if m_count > 0 {
        Some(total_load / m_count as f32)
    } else {
        None
    };

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
        .relative()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Fleet Stat Strip
        .child(fleet_stat_strip(server_count, agent_count, avg_load, total_vcpu))
        // 2. Main content split: host table on left, alerts/activity rail on right
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
                                        .child(format!("ALL ({})", server_count)),
                                )
                                .child(div().text_color(TEXT_MUTED).child("PRODUCTION (8)"))
                                .child(div().text_color(TEXT_MUTED).child("STAGING (2)"))
                                .child(div().text_color(TEXT_MUTED).child("EDGE"))
                                .child(div().text_color(TEXT_MUTED).child("DATA"))
                                .child(div().text_color(TEXT_MUTED).child("BY REGION"))
                                .child({
                                    let app_lab = app.clone();
                                    div()
                                        .id("btn-fleet-local-lab")
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .bg(OK_BG)
                                        .border_1()
                                        .border_color(OK)
                                        .text_color(OK)
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .cursor_pointer()
                                        .font_family(FONT_MONO)
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(10.0))
                                        .on_click(move |_ev, _window, cx| {
                                            app_lab.update(cx, |this, cx| {
                                                this.toggle_local_lab_modal(cx);
                                            });
                                        })
                                        .child("⚡ LOCAL LAB & VMS")
                                })
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIMMER)
                                        .child(format!("{} hosts · sort health ↓ · click to open", server_count)),
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
                                    let host_name = host.name.clone();
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
                                            let hn = host_name.clone();
                                            app_host.update(cx, |this, cx| {
                                                this.switch_tab(&hn, cx);
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
        .children(if app_data.show_local_lab_modal {
            Some(crate::views::fleet::lab_modal::local_lab_modal(
                &app_data.lab_engines,
                &app_data.lab_nodes,
                app.clone(),
                app_data,
            ))
        } else {
            None
        })
}
