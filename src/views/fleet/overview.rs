use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};
use crate::components::icons::{TablerIcon, tabler_icon};

pub struct FleetHost {
    pub id: String,
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

pub fn fleet_stat_strip(
    server_count: usize,
    agent_count: usize,
    avg_load: Option<f32>,
    total_vcpu: usize,
) -> impl IntoElement {
    let s_count = server_count.to_string();
    let a_cov = agent_count.to_string();
    let a_tot = format!("/{}", server_count);
    let load_val = if server_count == 0 {
        "—".to_string()
    } else {
        avg_load.map(|l| format!("{:.2}", l)).unwrap_or_else(|| "0.00".to_string())
    };
    let vcpu_note = if server_count == 0 {
        "0 vCPU across fleet".to_string()
    } else if total_vcpu > 0 {
        format!("weighted avg · {} vCPU", total_vcpu)
    } else {
        "weighted avg · 1 vCPU".to_string()
    };
    let alert_count_str = if server_count == 0 { "0".to_string() } else { "0".to_string() };
    let alert_note = if server_count == 0 { "all systems normal".to_string() } else { "0 crit · 0 warn".to_string() };
    let drift_str = "0".to_string();
    let drift_note = if server_count == 0 { "baseline unconfigured".to_string() } else { "in sync with policy".to_string() };
    let host_key_age = if server_count == 0 { "—".to_string() } else { "0".to_string() };
    let host_key_unit = if server_count == 0 { "".to_string() } else { "d".to_string() };
    let host_key_note = if server_count == 0 { "no enrolled keys".to_string() } else { "within 90d policy".to_string() };

    let stats = [
        ("SERVERS", s_count, "".to_string(), if server_count == 0 { "no regions enrolled".to_string() } else { "1 region · 1 group".to_string() }, TEXT_PRIMARY),
        ("OPEN ALERTS", alert_count_str, "".to_string(), alert_note, if server_count == 0 { TEXT_MUTED } else { OK }),
        ("CONFIG DRIFT", drift_str, " hosts".to_string(), drift_note, if server_count == 0 { TEXT_MUTED } else { OK }),
        ("FLEET LOAD", load_val, "".to_string(), vcpu_note, TEXT_PRIMARY),
        ("OLDEST HOST KEY", host_key_age, host_key_unit, host_key_note, if server_count == 0 { TEXT_MUTED } else { OK }),
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
    let hosts: Vec<FleetHost> = if !app_data.fleet.servers.is_empty() {
        app_data.fleet.servers.iter().map(|s| {
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
            let (cpu_pct, cpu_label, mem_pct, mem_label, disk, uptime) = if let Some(m) = app_data.fleet.metrics_store.get(&s.id).or_else(|| app_data.fleet.metrics_store.get(&s.name)) {
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
                if s.status == "online" {
                    (0, "0%".into(), 0, "0%".into(), "—".into(), "—".into())
                } else {
                    (0, "—".into(), 0, "—".into(), "—".into(), "—".into())
                }
            };
            FleetHost {
                id: s.id.clone(),
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
                is_selected: s.id == app_data.fleet.active_tab_id,
            }
        }).collect()
    } else {
        Vec::new()
    };

    let server_count = hosts.len();
    let agent_count = hosts.iter().filter(|h| h.agent != "—").count();
    let total_vcpu: usize = app_data
        .fleet.metrics_store
        .values()
        .map(|m| m.vcpu_count)
        .sum::<usize>();
    let total_load: f32 = app_data.fleet.metrics_store.values().map(|m| m.load_1m).sum();
    let m_count = app_data.fleet.metrics_store.len();
    let avg_load = if m_count > 0 {
        Some(total_load / m_count as f32)
    } else {
        None
    };

    let alerts: Vec<(&'static str, Rgba, Rgba, String, String, String)> = app_data.fleet.servers.iter()
        .filter(|s| s.status == "unreachable" || s.status == "offline" || s.status == "degraded" || s.status == "warn" || s.status == "crit")
        .map(|s| {
            let (lvl, fg, bg) = if s.status == "unreachable" || s.status == "offline" || s.status == "crit" {
                ("CRIT", CRIT, CRIT_BG)
            } else {
                ("WARN", WARN, WARN_BG)
            };
            let msg = if s.status == "unreachable" || s.status == "offline" {
                format!("Host unreachable or offline (port {})", s.port)
            } else {
                "Health degraded or warning status reported".to_string()
            };
            (lvl, fg, bg, s.name.clone(), msg, "now".to_string())
        })
        .collect();

    let activities: Vec<(String, &'static str, Rgba, String, Rgba)> = app_data.fleet.servers.iter().take(10).map(|s| {
        (
            s.last_seen_at.as_deref().and_then(|t| t.split('T').nth(1)).and_then(|t| t.get(0..8)).unwrap_or("00:00:00").to_string(),
            "crow",
            TEXT_FAINT,
            format!("enrolled host {} ({}:{})", s.name, s.host, s.port),
            TEXT_PRIMARY,
        )
    }).collect();

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
                                .child(div().text_color(TEXT_MUTED).child(format!("PRODUCTION ({})", hosts.iter().filter(|h| h.env == "PROD").count())))
                                .child(div().text_color(TEXT_MUTED).child(format!("STAGING ({})", hosts.iter().filter(|h| h.env == "STAGE").count())))
                                .child(div().text_color(TEXT_MUTED).child(format!("DEV / LAB ({})", hosts.iter().filter(|h| h.env == "DEV" || h.env == "LAB").count())))
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
                                .child(div().w(px(22.0)).flex_none().child(""))
                                .child(div().flex_grow(3.0).min_w(px(140.0)).child("HOST"))
                                .child(div().flex_grow(2.0).min_w(px(100.0)).child("ADDRESS"))
                                .child(div().flex_grow(2.0).min_w(px(90.0)).child("ROLE"))
                                .child(div().w(px(60.0)).flex_none().child("ENV"))
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
                                .children(if hosts.is_empty() {
                                    let app_lab = app.clone();
                                    let app_add = app.clone();
                                    Some(
                                        div()
                                            .id("fleet-empty-state")
                                            .size_full()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .justify_center()
                                            .gap(px(16.0))
                                            .p(px(32.0))
                                            // Empty State Icon & Banner
                                            .child(
                                                div()
                                                    .size(px(48.0))
                                                    .border_1()
                                                    .border_color(BORDER_STRONG)
                                                    .bg(BG_PANEL)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        tabler_icon(TablerIcon::Server)
                                                            .size(px(24.0))
                                                            .text_color(TEXT_MUTED),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .items_center()
                                                    .gap(px(4.0))
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_size(px(13.0))
                                                            .text_color(TEXT_MAX)
                                                            .child("NO SERVERS ENROLLED"),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .text_color(TEXT_DIM)
                                                            .child("Connect remote Linux servers via SSH or spin up local isolated lab nodes."),
                                                    ),
                                            )
                                            // CTAs
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(12.0))
                                                    .mt(px(8.0))
                                                    .child(
                                                        div()
                                                            .id("btn-empty-local-lab")
                                                            .px(px(14.0))
                                                            .py(px(7.0))
                                                            .bg(OK_BG)
                                                            .border_1()
                                                            .border_color(OK)
                                                            .text_color(OK)
                                                            .hover(|s| s.bg(BG_ROW_HOVER))
                                                            .cursor_pointer()
                                                            .font_family(FONT_MONO)
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_size(px(11.0))
                                                            .on_click(move |_ev, _window, cx| {
                                                                app_lab.update(cx, |this, cx| {
                                                                    this.toggle_local_lab_modal(cx);
                                                                });
                                                            })
                                                            .child("⚡ ENROLL LOCAL LAB NODE"),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("btn-empty-add-server")
                                                            .px(px(14.0))
                                                            .py(px(7.0))
                                                            .bg(BG_PANEL)
                                                            .border_1()
                                                            .border_color(BORDER_STRONG)
                                                            .text_color(TEXT_PRIMARY)
                                                            .hover(|s| s.bg(BG_ROW_HOVER))
                                                            .cursor_pointer()
                                                            .font_family(FONT_MONO)
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_size(px(11.0))
                                                            .on_click(move |_ev, _window, cx| {
                                                                app_add.update(cx, |this, cx| {
                                                                    this.set_screen(Screen::Onboard, cx);
                                                                });
                                                            })
                                                            .child("+ ADD REMOTE SERVER (SSH)"),
                                                    ),
                                            )
                                    ).into_iter().collect::<Vec<_>>()
                                } else {
                                    hosts.into_iter().enumerate().map(|(idx, host)| {
                                        let app_host = app.clone();
                                        let host_id = host.id.clone();
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
                                                let hid = host_id.clone();
                                                app_host.update(cx, |this, cx| {
                                                    this.switch_tab(&hid, cx);
                                                    this.set_screen(Screen::Server, cx);
                                                });
                                            })
                                            .font_family(FONT_MONO)
                                            .text_size(px(11.5))
                                            // Status dot / glyph
                                            .child(
                                                div()
                                                    .w(px(22.0))
                                                    .flex_none()
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
                                                    .flex_grow(3.0)
                                                    .min_w(px(140.0))
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
                                                    .flex_grow(2.0)
                                                    .min_w(px(100.0))
                                                    .text_color(TEXT_DIM)
                                                    .child(host.ip),
                                            )
                                            // Role
                                            .child(
                                                div()
                                                    .flex_grow(2.0)
                                                    .min_w(px(90.0))
                                                    .text_size(px(11.0))
                                                    .text_color(TEXT_TERTIARY)
                                                    .child(host.role),
                                            )
                                            // Env
                                            .child(
                                                div()
                                                    .w(px(60.0))
                                                    .flex_none()
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
                                    }).collect::<Vec<_>>()
                                }),
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
                                        .child(if alerts.is_empty() { "0 open".to_string() } else { format!("{} open", alerts.len()) }),
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
                                .children(if alerts.is_empty() {
                                    Some(
                                        div()
                                            .px(px(12.0))
                                            .py(px(14.0))
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.5))
                                            .text_color(TEXT_MUTED)
                                            .child(div().size(px(6.0)).rounded_full().bg(OK))
                                            .child("✓ All fleet systems operational"),
                                    ).into_iter().collect::<Vec<_>>()
                                } else {
                                    alerts.iter().map(|(lvl, fg, bg, host, msg, age)| {
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
                                                            .text_size(px(9.5))
                                                            .font_weight(FontWeight::BOLD)
                                                            .px(px(4.0))
                                                            .py(px(1.5))
                                                            .bg(*bg)
                                                            .text_color(*fg)
                                                            .child((*lvl).to_string()),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .text_color(TEXT_SECONDARY)
                                                            .child(host.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(10.0))
                                                            .text_color(TEXT_FAINT)
                                                            .child(age.clone()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.5))
                                                    .text_color(if *lvl == "CRIT" { CRIT_INK } else { WARN_INK })
                                                    .child(msg.clone()),
                                            )
                                    }).collect::<Vec<_>>()
                                }),
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
                                .children(if activities.is_empty() {
                                    Some(
                                        div()
                                            .px(px(12.0))
                                            .py(px(14.0))
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.5))
                                            .text_color(TEXT_DIMMER)
                                            .child("No recent fleet activity recorded"),
                                    ).into_iter().collect::<Vec<_>>()
                                } else {
                                    activities.iter().map(|(ts, actor, actor_color, msg, text_c)| {
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
                                                    .child(ts.clone()),
                                            )
                                            .child(
                                                div()
                                                    .w(px(52.0))
                                                    .text_color(*actor_color)
                                                    .flex_none()
                                                    .child((*actor).to_string()),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .text_color(*text_c)
                                                    .child(msg.clone()),
                                            )
                                    }).collect::<Vec<_>>()
                                }),
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
        .children(if app_data.local_lab.show_modal {
            Some(crate::views::fleet::lab_modal::local_lab_modal(
                &app_data.local_lab.engines,
                &app_data.local_lab.nodes,
                app.clone(),
                app_data,
            ))
        } else {
            None
        })
}
