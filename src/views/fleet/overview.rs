use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::app::region::FleetEnvFilter;
use crate::app::{CrowApp, Screen};
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::host::{connection_state, transport_kind, ConnectionState, TransportKind};
use crate::vault::{ChangeRecord, ServerRecord};
use crate::views::fleet::FleetState;
use crate::components::resize::{bottom_panel_height, resize_handle, BottomPanelResize};
use crate::views::fleet::archived::{archive_confirm_overlay, archived_panel, fleet_view_tabs};
use crate::views::fleet::lab_state::LocalLabState;

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
    pub alerts: String,
    pub alert_color: Rgba,
    pub status_color: Rgba,
    pub is_critical_border: bool,
    pub pill: String,
    pub is_selected: bool,
    /// ISO country code ("" when unknown).
    pub country: String,
}

pub fn fleet_stat_strip(
    servers: &[ServerRecord],
    crit: usize,
    warn: usize,
    connected_count: usize,
    avg_load: Option<f32>,
    total_vcpu: usize,
) -> impl IntoElement {
    let server_count = servers.len();
    let groups: std::collections::BTreeSet<&str> = servers.iter().map(|s| s.group_name.as_str()).filter(|g| !g.is_empty()).collect();
    let regions: std::collections::BTreeSet<&str> = servers.iter().map(|s| s.region_country.as_str()).filter(|c| !c.is_empty()).collect();
    let group_note = match groups.len() {
        0 if server_count == 0 => "none enrolled".to_string(),
        0 => "ungrouped".to_string(),
        1 => "1 group".to_string(),
        n => format!("{n} groups"),
    };
    let group_note = match regions.len() {
        0 => group_note,
        1 => format!("1 region · {group_note}"),
        n => format!("{n} regions · {group_note}"),
    };
    let load_val = avg_load.map(|l| format!("{:.2}", l)).unwrap_or_else(|| "—".to_string());
    let vcpu_note = if total_vcpu > 0 { format!("avg 1m load · {} vCPU", total_vcpu) } else { "no metrics yet".to_string() };
    let alert_color = if crit > 0 { CRIT } else if warn > 0 { WARN } else if server_count == 0 { TEXT_MUTED } else { OK };

    let oldest_key_info = servers
        .iter()
        .filter(|s| s.host_key_fingerprint.as_ref().map_or(false, |f| !f.trim().is_empty()))
        .filter_map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s.created_at)
                .ok()
                .map(|t| (t.with_timezone(&chrono::Utc), s.name.as_str()))
        })
        .min_by_key(|(t, _)| *t);

    let (oldest_key_val, oldest_key_unit, oldest_key_note, oldest_key_color) = match oldest_key_info {
        Some((created, srv_name)) => {
            let days = (chrono::Utc::now() - created).num_days().max(0);
            let count_over_180 = servers
                .iter()
                .filter(|s| s.host_key_fingerprint.as_ref().map_or(false, |f| !f.trim().is_empty()))
                .filter_map(|s| chrono::DateTime::parse_from_rfc3339(&s.created_at).ok())
                .filter(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_days() > 180)
                .count();

            let (val, unit) = if days >= 365 {
                (format!("{:.1}", days as f64 / 365.25), "y".to_string())
            } else {
                (days.to_string(), "d".to_string())
            };

            let note = if count_over_180 > 1 {
                format!("{srv_name} · {count_over_180} > 180d")
            } else {
                srv_name.to_string()
            };

            let color = if days >= 365 {
                WARN
            } else {
                TEXT_PRIMARY
            };

            (val, unit, note, color)
        }
        None => {
            let note = if server_count == 0 {
                "none enrolled".to_string()
            } else {
                "no keys tracked".to_string()
            };
            ("—".to_string(), "".to_string(), note, TEXT_MUTED)
        }
    };

    let stats = [
        ("SERVERS", server_count.to_string(), "".to_string(), group_note, TEXT_PRIMARY),
        ("OPEN ALERTS", (crit + warn).to_string(), "".to_string(), format!("{crit} crit · {warn} warn"), alert_color),
        ("CONFIG DRIFT", "—".to_string(), "".to_string(), "not tracked yet".to_string(), TEXT_MUTED),
        ("FLEET LOAD", load_val, "".to_string(), vcpu_note, TEXT_PRIMARY),
        ("OLDEST HOST KEY", oldest_key_val, oldest_key_unit, oldest_key_note, oldest_key_color),
        ("CONNECTED", connected_count.to_string(), format!("/{}", server_count), "reachable over their transport".to_string(), if connected_count < server_count { WARN } else { TEXT_PRIMARY }),
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

/// `background`: the Personalisation picture (already blurred) and its
/// opacity, drawn behind the server list.
pub fn fleet_overview_view(
    app: Entity<CrowApp>,
    fleet: &FleetState,
    local_lab: &LocalLabState,
    background: Option<(std::path::PathBuf, f32)>,
    purge_days: Option<i64>,
    purge_audit: &[ChangeRecord],
) -> impl IntoElement {
    let purge_due = crate::app::archive::purge_due_text(purge_days);
    let hosts: Vec<FleetHost> = if !fleet.servers.is_empty() {
        fleet.servers.iter().map(|s| {
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
            let (cpu_pct, cpu_label, mem_pct, mem_label, disk, uptime) = if let Some(m) = fleet.metrics_store.get(&s.id).or_else(|| fleet.metrics_store.get(&s.name)) {
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
                alerts: if matches!(s.status.as_str(), "unreachable" | "offline" | "crit" | "degraded" | "warn") { "1".into() } else { "0".into() },
                alert_color: if is_crit { CRIT } else { TEXT_FAINT },
                status_color,
                is_critical_border: is_crit,
                pill,
                is_selected: s.id == fleet.active_tab_id,
                country: s.region_country.clone(),
            }
        }).collect()
    } else {
        Vec::new()
    };

    let server_count = hosts.len();
    // Counts for the tabs and region chips come from the whole fleet; the
    // table shows what the environment tab and region chip select (ERR-36).
    let env_count = |f: FleetEnvFilter| hosts.iter().filter(|h| f.matches(&h.env)).count();
    let env_counts = [FleetEnvFilter::All, FleetEnvFilter::Prod, FleetEnvFilter::Stage, FleetEnvFilter::DevLab].map(env_count);
    let mut region_counts: Vec<(String, usize)> = Vec::new();
    for h in &hosts {
        match region_counts.iter_mut().find(|(c, _)| *c == h.country) {
            Some((_, n)) => *n += 1,
            None => region_counts.push((h.country.clone(), 1)),
        }
    }
    region_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.is_empty().cmp(&b.0.is_empty())).then_with(|| a.0.cmp(&b.0)));
    let hosts: Vec<FleetHost> = hosts
        .into_iter()
        .filter(|h| fleet.env_filter.matches(&h.env))
        .filter(|h| fleet.region_filter.as_ref().is_none_or(|c| *c == h.country))
        .collect();
    let visible_count = hosts.len();
    // Servers Crow can talk to right now: local and lab transports, or SSH
    // whose last command connected.
    let connected_count = fleet.servers.iter().filter(|s| match transport_kind(s) {
        TransportKind::Ssh => connection_state(&s.id) == Some(ConnectionState::Connected),
        _ => true,
    }).count();
    let total_vcpu: usize = fleet.metrics_store
        .values()
        .map(|m| m.vcpu_count)
        .sum::<usize>();
    let total_load: f32 = fleet.metrics_store.values().map(|m| m.load_1m).sum();
    let m_count = fleet.metrics_store.len();
    let avg_load = if m_count > 0 {
        Some(total_load / m_count as f32)
    } else {
        None
    };

    let alerts: Vec<(&'static str, Rgba, Rgba, String, String, String)> = fleet.servers.iter()
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

    let activities: Vec<(String, &'static str, Rgba, String, Rgba)> = fleet.servers.iter().take(10).map(|s| {
        (
            s.created_at.split('T').nth(1).and_then(|t| t.get(0..8)).unwrap_or("—").to_string(),
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
        .child(fleet_stat_strip(
            &fleet.servers,
            alerts.iter().filter(|a| a.0 == "CRIT").count(),
            alerts.iter().filter(|a| a.0 == "WARN").count(),
            connected_count,
            avg_load,
            total_vcpu,
        ))
        // 2. Active fleet / Archived switch
        .child(fleet_view_tabs(fleet, app.clone()))
        // 3. Server list on top, alerts & activity panel below (drag the
        //    divider). The Archived tab replaces this whole body.
        .children(if fleet.show_archived { None } else { Some(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .on_drag_move::<BottomPanelResize>({
                    let app = app.clone();
                    move |ev, _window, cx| {
                        let height = bottom_panel_height(ev, 120.0, 160.0);
                        app.update(cx, |this, cx| {
                            this.fleet.bottom_panel_height = height;
                            cx.notify();
                        });
                    }
                })
                // Left Table Panel
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.0))
                        .relative()
                        .flex()
                        .flex_col()
                        // Personalisation picture, under everything in the list.
                        .children(background.map(|(path, opacity)| {
                            img(path).absolute().inset_0().size_full().object_fit(ObjectFit::Cover).opacity(opacity)
                        }))
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
                                .children([
                                    (FleetEnvFilter::All, "ALL", env_counts[0]),
                                    (FleetEnvFilter::Prod, "PRODUCTION", env_counts[1]),
                                    (FleetEnvFilter::Stage, "STAGING", env_counts[2]),
                                    (FleetEnvFilter::DevLab, "DEV / LAB", env_counts[3]),
                                ].into_iter().map(|(filter, label, count)| {
                                    let app = app.clone();
                                    fleet_tab(SharedString::from(format!("fleet-env-{label}")), format!("{label} ({count})"), fleet.env_filter == filter)
                                        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_fleet_env_filter(filter, cx)))
                                }))
                                .child({
                                    let app = app.clone();
                                    let regions = region_counts.iter().filter(|(c, _)| !c.is_empty()).count();
                                    fleet_tab("fleet-by-region".into(), if regions > 0 { format!("BY REGION ({regions})") } else { "BY REGION".into() }, fleet.region_bar_open)
                                        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.toggle_fleet_region_bar(cx)))
                                })
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
                                .child({
                                    let app_import = app.clone();
                                    div()
                                        .id("btn-fleet-import")
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .text_color(TEXT_SECONDARY)
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .cursor_pointer()
                                        .font_family(FONT_MONO)
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(10.0))
                                        .on_click(move |_ev, _window, cx| app_import.update(cx, |this, cx| this.open_import(cx)))
                                        .child("☁ IMPORT FROM PROVIDERS")
                                })
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIMMER)
                                        .child(if visible_count == server_count { format!("{} hosts · sort health ↓ · click to open", server_count) } else { format!("{visible_count} of {server_count} hosts · click to open") }),
                                ),
                        )
                        // Region chips (BY REGION): one per country, with counts.
                        .children(fleet.region_bar_open.then(|| {
                            let app_detect = app.clone();
                            div()
                                .h(px(32.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(14.0))
                                .bg(BG_APP)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .children(region_counts.iter().map(|(cc, n)| {
                                    let app = app.clone();
                                    let on = fleet.region_filter.as_deref() == Some(cc.as_str());
                                    let target = if on { None } else { Some(cc.clone()) };
                                    let label = if cc.is_empty() { "UNKNOWN".to_string() } else { crate::region::country_name(cc).unwrap_or(cc).to_uppercase() };
                                    div()
                                        .id(SharedString::from(format!("fleet-region-{cc}")))
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(7.0))
                                        .py(px(2.0))
                                        .border_1()
                                        .border_color(if on { TEXT_SECONDARY } else { BORDER_DEFAULT })
                                        .bg(if on { BG_CHIP } else { hex_rgba(0, 0.0) })
                                        .text_color(if on { TEXT_PRIMARY } else { TEXT_DIM })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            let target = target.clone();
                                            app.update(cx, |this, cx| this.set_fleet_region_filter(target, cx));
                                        })
                                        .child(crate::components::flag::flag(cc, 10.0))
                                        .child(format!("{label} {n}"))
                                }))
                                .child(div().flex_1())
                                .children(fleet.region_note.clone().map(|n| div().text_color(TEXT_FAINT).child(n)))
                                .child(
                                    div()
                                        .id("fleet-detect-regions")
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .border_1()
                                        .border_color(if fleet.region_detecting { BORDER_DEFAULT } else { OK })
                                        .text_color(if fleet.region_detecting { TEXT_FAINT } else { OK })
                                        .font_weight(FontWeight::BOLD)
                                        .when(!fleet.region_detecting, |d| {
                                            d.cursor_pointer().hover(|s| s.bg(OK_BG)).on_click(move |_ev, _window, cx| app_detect.update(cx, |this, cx| this.detect_server_regions(None, cx)))
                                        })
                                        .child(if fleet.region_detecting { "DETECTING…" } else { "DETECT REGIONS ↻" }),
                                )
                        }))
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
                                // Same widths as the row cells below; fixed columns are
                                // flex_none so header and rows can't shrink differently.
                                .child(div().flex_grow(3.0).flex_basis(px(0.0)).min_w(px(140.0)).child("HOST"))
                                .child(div().flex_grow(2.0).flex_basis(px(0.0)).min_w(px(110.0)).child("ADDRESS"))
                                .child(div().flex_grow(2.0).flex_basis(px(0.0)).min_w(px(90.0)).pr(px(14.0)).text_align(TextAlign::Right).child("ROLE"))
                                .child(div().w(px(64.0)).flex_none().child("ENV"))
                                .child(div().w(px(95.0)).flex_none().child("CPU"))
                                .child(div().w(px(95.0)).flex_none().child("MEM"))
                                .child(div().w(px(55.0)).flex_none().text_align(TextAlign::Right).child("DISK"))
                                .child(div().w(px(80.0)).flex_none().text_align(TextAlign::Right).child("UPTIME"))
                                .child(div().w(px(65.0)).flex_none().text_align(TextAlign::Right).child("ALERTS"))
                                .child(div().w(px(78.0)).flex_none().text_align(TextAlign::Right).child("")),
                        )
                        // Table Body
                        .child(
                            div()
                                .id("fleet-host-list")
                                .flex_1()
                                .overflow_y_scrollbar()
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
                                        let app_archive = app.clone();
                                        let archive_id = host.id.clone();
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
                                                    .flex_basis(px(0.0))
                                                    .min_w(px(140.0))
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(6.0))
                                                    .child(crate::components::flag::flag(&host.country, 11.0))
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
                                                            .rounded_sm()
                                                            .bg(if matches!(host.pill.as_str(), "UNREACHABLE" | "OFFLINE" | "CRITICAL") {
                                                                CRIT_BG
                                                            } else if host.pill == "UNKNOWN" {
                                                                BG_CHIP
                                                            } else if host.pill == "DEGRADED" {
                                                                WARN_BG
                                                            } else {
                                                                OK_BG
                                                            })
                                                            .text_color(if matches!(host.pill.as_str(), "UNREACHABLE" | "OFFLINE" | "CRITICAL") {
                                                                CRIT
                                                            } else if host.pill == "UNKNOWN" {
                                                                TEXT_DIM
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
                                                    .flex_basis(px(0.0))
                                                    .min_w(px(110.0))
                                                    .overflow_hidden()
                                                    .text_color(TEXT_DIM)
                                                    .child(host.ip),
                                            )
                                            // Role
                                            .child(
                                                div()
                                                    .flex_grow(2.0)
                                                    .flex_basis(px(0.0))
                                                    .min_w(px(90.0))
                                                    .pr(px(14.0))
                                                    .overflow_hidden()
                                                    .text_align(TextAlign::Right)
                                                    .text_size(px(11.0))
                                                    .text_color(TEXT_TERTIARY)
                                                    .child(host.role),
                                            )
                                            // Env
                                            .child(
                                                div()
                                                    .w(px(64.0))
                                                    .flex_none()
                                                    .flex()
                                                    .items_center()
                                                    .child(
                                                        // Hugs its text, like the other badges.
                                                        div()
                                                            .flex_none()
                                                            .rounded_sm()
                                                            .px(px(5.0))
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
                                                    .flex_none()
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
                                                    .flex_none()
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
                                                    .flex_none()
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
                                                    .flex_none()
                                                    .text_align(TextAlign::Right)
                                                    .text_color(TEXT_DIM)
                                                    .child(host.uptime),
                                            )
                                            // Alerts
                                            .child(
                                                div()
                                                    .w(px(65.0))
                                                    .flex_none()
                                                    .text_align(TextAlign::Right)
                                                    .font_weight(if host.alerts == "0" {
                                                        FontWeight::NORMAL
                                                    } else {
                                                        FontWeight::BOLD
                                                    })
                                                    .text_color(host.alert_color)
                                                    .child(host.alerts),
                                            )
                                            // Archive (ERR-32): a server is never deleted outright.
                                            .child(
                                                div()
                                                    .w(px(78.0))
                                                    .flex_none()
                                                    .flex()
                                                    .justify_end()
                                                    .child(
                                                        div()
                                                            .id(SharedString::from(format!("fleet-archive-{archive_id}")))
                                                            .px(px(7.0))
                                                            .py(px(1.5))
                                                            .border_1()
                                                            .border_color(BORDER_DEFAULT)
                                                            .text_color(TEXT_DIMMER)
                                                            .text_size(px(9.0))
                                                            .font_weight(FontWeight::BOLD)
                                                            .cursor_pointer()
                                                            .hover(|s| s.bg(BG_CONTROL).text_color(WARN))
                                                            .on_click(move |_ev, _window, cx| {
                                                                let id = archive_id.clone();
                                                                cx.stop_propagation();
                                                                app_archive.update(cx, |this, cx| {
                                                                    this.fleet.pending_archive = Some(id);
                                                                    cx.notify();
                                                                });
                                                            })
                                                            .child("ARCHIVE"),
                                                    ),
                                            )
                                    }).collect::<Vec<_>>()
                                }),
                        ),
                )
                // Drag handle between the list and the panel
                .child(resize_handle("fleet-panel-resize"))
                // Bottom panel: Fleet Alerts (left) & Activity Log (right)
                .child(
                    div()
                        .h(px(fleet.bottom_panel_height))
                        .flex_none()
                        .flex()
                        .min_h(px(0.0))
                        .bg(BG_RAIL)
                        .overflow_hidden()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .flex()
                                .flex_col()
                                .border_r_1()
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
                                .id("fleet-alerts-list")
                                .flex_1()
                                .min_h(px(0.0))
                                .overflow_y_scrollbar()
                                .flex()
                                .flex_col()


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
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .flex()
                                .flex_col()
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
                                .overflow_y_scrollbar()
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
                        )
                ),
        ) })
        // Archived servers, when that tab is showing.
        .children(if fleet.show_archived {
            Some(archived_panel(fleet, purge_days, purge_audit, app.clone()).into_any_element())
        } else {
            None
        })
        // 4. Persistent Fleet-Wide Destructive Strip with Abort Gate
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
                                .child(format!("ROLLING REBOOT · {} HOSTS", server_count)),
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
        .children(if local_lab.show_modal {
            Some(crate::views::fleet::lab_modal::local_lab_modal(&local_lab.engines, &local_lab.nodes, app.clone(), local_lab))
        } else {
            None
        })
        // Archive confirmation, over everything (ERR-32).
        .children(fleet.pending_archive.as_ref().and_then(|id| fleet.servers.iter().find(|s| &s.id == id)).map(|srv| {
            archive_confirm_overlay(srv, &purge_due, app.clone())
        }))
}

/// One tab in the Fleet list's filter bar.
fn fleet_tab(id: SharedString, label: String, selected: bool) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(8.0))
        .py(px(2.0))
        .border_1()
        .border_color(if selected { BORDER_CONTROL_SEL } else { hex_rgba(0, 0.0) })
        .bg(if selected { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
        .text_color(if selected { TEXT_MAX } else { TEXT_MUTED })
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .cursor_pointer()
        .hover(|s| s.text_color(TEXT_PRIMARY))
        .child(label)
}
