use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::views::fleet::state::FleetPage;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::app::region::FleetEnvFilter;
use crate::app::{CrowApp, Screen};
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::metrics::ServerMetrics;
use std::collections::HashMap;
use crate::vault::{ChangeRecord, ServerRecord};
use crate::views::fleet::state::{FleetHealth, FleetState};
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
    drift: Option<&[crate::config::drift::DriftEntry]>,
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

    // From each server's host key file times (ERR-81).
    let ages = crate::views::fleet::state::host_key_ages(servers, chrono::Utc::now().timestamp());
    let (oldest_key_val, oldest_key_unit, oldest_key_note, oldest_key_color) = match &ages.oldest {
        None if server_count == 0 => ("—".to_string(), "".to_string(), "none enrolled".to_string(), TEXT_MUTED),
        None => ("—".to_string(), "".to_string(), "not read yet".to_string(), TEXT_MUTED),
        Some((days, name)) => {
            let (val, unit) = if *days >= 365 { (format!("{:.1}", *days as f64 / 365.25), "y") } else { (days.to_string(), "d") };
            let note = if ages.unknown > 0 { format!("{name} · {} not read yet", ages.unknown) } else { name.clone() };
            (val, unit.to_string(), note, TEXT_PRIMARY)
        }
    };
    // Files that mean something other than their baseline, as of each
    // server's last config read (ERR-74).
    let (drift_val, drift_unit, drift_note, drift_fg) = match drift {
        None => ("—".to_string(), "".to_string(), "not computed yet".to_string(), TEXT_MUTED),
        Some([]) => ("—".to_string(), "".to_string(), "no baselines pinned".to_string(), TEXT_MUTED),
        Some(entries) => {
            let drifted: Vec<_> = entries.iter().filter(|e| e.drift.is_drift()).collect();
            let servers: std::collections::BTreeSet<&str> = drifted.iter().map(|e| e.server_id.as_str()).collect();
            match drifted.len() {
                0 => ("0".to_string(), "".to_string(), format!("{} file{} match baselines · as last read", entries.len(), if entries.len() == 1 { "" } else { "s" }), OK),
                n => (n.to_string(), "".to_string(), format!("on {} server{} · as last read", servers.len(), if servers.len() == 1 { "" } else { "s" }), WARN),
            }
        }
    };

    let stats = [
        ("SERVERS", server_count.to_string(), "".to_string(), group_note, TEXT_PRIMARY),
        ("OPEN ALERTS", (crit + warn).to_string(), "".to_string(), format!("{crit} crit · {warn} warn"), alert_color),
        ("CONFIG DRIFT", drift_val, drift_unit, drift_note, drift_fg),
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
    activity: &[crate::views::audit::model::AuditItem],
    alert_lines: &[crate::views::fleet::alert_lines::AlertLine],
    watch_gap: Option<(i64, i64)>,
) -> impl IntoElement {
    let purge_due = crate::app::archive::purge_due_text(purge_days);
    let health: HashMap<String, FleetHealth> = fleet.servers.iter().map(|s| (s.id.clone(), fleet.health(s))).collect();
    let hosts: Vec<FleetHost> = if !fleet.servers.is_empty() {
        fleet.servers.iter().map(|s| {
            let health = &health[&s.id];
            let status_color = health.color();
            let (pill, is_crit) = match health {
                FleetHealth::Ok => ("OK".to_string(), false),
                FleetHealth::Checking => ("CHECKING".to_string(), false),
                FleetHealth::Down { label, .. } => (label.to_string(), true),
            };
            let (env_bg, env_fg) = match s.env.as_str() {
                "PROD" => (CRIT_BG, CRIT),
                "STAGE" => (WARN_BG, WARN),
                "DEV" => (OK_BG, OK),
                _ => (BG_PANEL, TEXT_DIM),
            };
            // Numbers only from a probe that answered; a failed one keeps
            // stale or default values that aren't readings.
            let reading = fleet.metrics_store.get(&s.id).or_else(|| fleet.metrics_store.get(&s.name)).filter(|m| m.reachable && health.is_ok());
            let (cpu_pct, cpu_label, mem_pct, mem_label, disk, uptime) = match reading {
                Some(m) => {
                    let cpu = (m.cpu_pct.round() as u8).clamp(0, 100);
                    let mem = (m.mem_pct.round() as u8).clamp(0, 100);
                    (cpu, format!("{}%", cpu), mem, format!("{}%", mem), format!("{:.0}%", m.disk_pct.ceil()), m.uptime_formatted.clone())
                }
                None => (0, "—".into(), 0, "—".into(), "—".into(), "—".into()),
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
                alerts: alert_lines.iter().filter(|l| !l.resolved && l.host == s.name).count().to_string(),
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
    let connected_count = health.values().filter(|h| h.is_ok()).count();
    // What an empty alert list means: reachability is all Crow watches yet.
    let checking = health.values().filter(|h| **h == FleetHealth::Checking).count();
    let (quiet_note, quiet_color) = match (fleet.servers.len(), checking) {
        (0, _) => ("No servers enrolled".to_string(), TEXT_MUTED),
        (_, 0) => (format!("All {connected_count} reachable"), OK),
        (_, n) => (format!("Checking {n} server{}", if n == 1 { "" } else { "s" }), TEXT_MUTED),
    };
    // Each reachable server once — metrics_store also keys by name.
    let readings: Vec<&ServerMetrics> = fleet.servers.iter()
        .filter(|s| health[&s.id].is_ok())
        .filter_map(|s| fleet.metrics_store.get(&s.id))
        .filter(|m| m.reachable)
        .collect();
    let total_vcpu: usize = readings.iter().map(|m| m.vcpu_count).sum();
    let avg_load = (!readings.is_empty()).then(|| readings.iter().map(|m| m.load_1m).sum::<f32>() / readings.len() as f32);

    // Stored alerts and servers down right now (ERR-85); resolved and
    // acknowledged ones are dimmed.
    let alerts: Vec<(&'static str, Rgba, Rgba, String, String, String)> = alert_lines.iter().map(|l| {
        let (fg, bg) = match (l.level, l.acknowledged || l.resolved) {
            (_, true) => (TEXT_DIM, BG_CHIP),
            ("CRIT", _) => (CRIT, CRIT_BG),
            _ => (WARN, WARN_BG),
        };
        let msg = if l.acknowledged && !l.resolved { format!("{} · acknowledged", l.message) } else { l.message.clone() };
        (l.level, fg, bg, l.host.clone(), msg, l.age.clone())
    }).collect();
    let open_lines: Vec<_> = alert_lines.iter().filter(|l| !l.resolved).collect();
    let gap_note = watch_gap.map(|(from, to)| {
        let fmt = |t: i64| chrono::DateTime::from_timestamp(t, 0).map(|d| d.with_timezone(&chrono::Local).format("%b %d %H:%M").to_string()).unwrap_or_default();
        format!("Crow wasn't running from {} to {}: nothing was watched then.", fmt(from), fmt(to))
    });

    // The newest of the audit log (ERR-75): what Crow did, where.
    let now = chrono::Local::now();
    let activities: Vec<(String, String, Rgba, String, Rgba)> = activity.iter().take(30).map(|i| {
        let failed = i.outcome == crate::views::audit::model::Outcome::Failed;
        (
            crate::views::audit::model::when_label(i.at, now),
            i.server.clone(),
            TEXT_FAINT,
            i.text.clone(),
            if failed { CRIT } else { TEXT_PRIMARY },
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
            open_lines.iter().filter(|l| l.level == "CRIT").count(),
            open_lines.iter().filter(|l| l.level == "WARN").count(),
            connected_count,
            avg_load,
            total_vcpu,
            fleet.drift.as_deref(),
        ))
        // 2. Active fleet / Archived switch
        .child(fleet_view_tabs(fleet, app.clone()))
        // 3. Server list on top, alerts & activity panel below (drag the
        //    divider). The Archived tab replaces this whole body.
        .children(if fleet.page != FleetPage::Active { None } else { Some(
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
                                        let app_terminal = app.clone();
                                        let terminal_id = host.id.clone();
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
                                                            .bg(if host.is_critical_border { CRIT_BG } else if host.status_color == OK { OK_BG } else { BG_CHIP })
                                                            .text_color(if host.status_color == TEXT_FAINTER { TEXT_DIM } else { host.status_color })
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
                                            // Straight to the server's terminal (ERR-93). Archiving
                                            // stays a deliberate step: the server's Danger Zone, or
                                            // Settings → Servers & Archives.
                                            .child(
                                                div()
                                                    .w(px(78.0))
                                                    .flex_none()
                                                    .flex()
                                                    .justify_end()
                                                    .child(
                                                        div()
                                                            .id(SharedString::from(format!("fleet-terminal-{terminal_id}")))
                                                            .px(px(7.0))
                                                            .py(px(2.0))
                                                            .border_1()
                                                            .border_color(BORDER_DEFAULT)
                                                            .text_color(TEXT_DIM)
                                                            .cursor_pointer()
                                                            .hover(|s| s.bg(BG_CONTROL).text_color(TEXT_PRIMARY).border_color(TEXT_DIM))
                                                            .on_click(move |_ev, _window, cx| {
                                                                let id = terminal_id.clone();
                                                                cx.stop_propagation();
                                                                app_terminal.update(cx, |this, cx| {
                                                                    this.switch_tab(&id, cx);
                                                                    this.set_view("terminal", cx);
                                                                });
                                                            })
                                                            .child(crate::components::icons::inherited_icon(TablerIcon::Terminal2, px(13.0))),
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
                                        .child(format!("{} open", open_lines.len())),
                                )
                                .child(div().flex_1())
                                .children(open_lines.iter().any(|l| l.id.is_some() && !l.acknowledged).then(|| {
                                    let app = app.clone();
                                    div()
                                        .id("btn-ack-all-alerts")
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(TEXT_PRIMARY))
                                        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.acknowledge_alerts(None, cx)))
                                        .child("ACK ALL")
                                })),
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


                                .children(gap_note.map(|n| {
                                    div()
                                        .px(px(12.0))
                                        .py(px(6.0))
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .child(n)
                                }))
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
                                            .child(div().size(px(6.0)).rounded_full().bg(quiet_color))
                                            .child(quiet_note.clone()),
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
                                .child({
                                    let app = app.clone();
                                    div()
                                        .id("btn-open-audit")
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(TEXT_PRIMARY))
                                        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_screen(Screen::Audit, cx)))
                                        .child("audit log · all hosts →")
                                }),
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
                                                    .w(px(84.0))
                                                    .text_color(TEXT_FAINTER)
                                                    .flex_none()
                                                    .child(ts.clone()),
                                            )
                                            .child(
                                                div()
                                                    .w(px(110.0))
                                                    .text_color(*actor_color)
                                                    .flex_none()
                                                    .overflow_hidden()
                                                    .child(actor.clone()),
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
        .children(match fleet.page {
            FleetPage::Archived => Some(archived_panel(fleet, purge_days, purge_audit, app.clone()).into_any_element()),
            FleetPage::Danger => Some(fleet_danger_page(app.clone()).into_any_element()),
            FleetPage::Active => None,
        })
        // 4. Fleet Notice Banner (feedback after fleet action, archive, restore, etc.)
        .children(fleet.notice.as_ref().map(|notice| {
            let app_dismiss = app.clone();
            div()
                .id("fleet-notice-banner")
                .h(px(32.0))
                .flex_none()
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_DEFAULT)
                .flex()
                .items_center()
                .justify_between()
                .gap(px(10.0))
                .font_family(FONT_MONO)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tabler_icon(TablerIcon::InfoCircle).size(px(13.0)).text_color(hex_rgb(0x38bdf8)))
                        .child(div().text_size(px(11.0)).text_color(TEXT_MAX).child(notice.clone())),
                )
                .child(
                    div()
                        .id("btn-dismiss-fleet-notice")
                        .px(px(8.0))
                        .py(px(2.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_xs()
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_MAX))
                        .on_click(move |_ev, _window, cx| {
                            app_dismiss.update(cx, |this, cx| {
                                this.fleet.notice = None;
                                cx.notify();
                            });
                        })
                        .child("DISMISS"),
                )
        }))
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

/// The fleet-wide Danger Zone: actions across every server, run one host
/// at a time on the fleet runner.
fn fleet_danger_page(app: Entity<CrowApp>) -> impl IntoElement {
    use crate::components::danger_zone::{action_row, danger_triangle};
    let reboot = {
        let app = app.clone();
        fleet_action_live("btn-fleet-rolling-reboot", "ROLLING REBOOT".into()).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.plan_rolling_reboot(window, cx)))
    };
    let baseline = {
        let app = app.clone();
        fleet_action_live("btn-fleet-push-baseline", "PUSH BASELINE TO ALL".into()).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.plan_push_baselines(window, cx)))
    };
    let rotate = {
        let app = app.clone();
        fleet_action_live("btn-fleet-rotate-keys", "ROTATE HOST KEYS".into()).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.plan_rotate_host_keys(window, cx)))
    };
    div()
        .id("fleet-danger-page")
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .font_family(FONT_MONO)
        .overflow_y_scrollbar()
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(9.0))
                .px(px(16.0))
                .bg(CRIT_STRIP_BG)
                .border_b_1()
                .border_color(BORDER_DANGER)
                .child(danger_triangle())
                .child(div().text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(CRIT).child("FLEET-WIDE"))
                .child(div().flex_1())
                .child(div().text_size(px(10.0)).text_color(TEXT_DIMMER).child("one host at a time · stops at the first failure")),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .max_w(px(920.0))
                .child(action_row("Rolling reboot", "Reboots every reachable server in turn, each back from a fresh boot before the next.", vec![reboot.into_any_element()]))
                .child(action_row("Push baseline", "Pushes each config baseline to every server whose copy drifted from it.", vec![baseline.into_any_element()]))
                .child(action_row("Rotate all host keys", "New SSH host keys on every server, re-pinned in known_hosts and proven with a fresh login; undone on any failure. Other machines will see a changed key.", vec![rotate.into_any_element()]))
                .child(action_row("Revoke all sessions", "Not built yet.", vec![fleet_action_stub("btn-fleet-revoke-sessions", "NOT YET BUILT".into()).into_any_element()])),
        )
}

/// A fleet-wide action that runs on the fleet runner (ERR-79).
fn fleet_action_live(id: &'static str, label: String) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(4.0))
        .border_1()
        .border_color(BORDER_DANGER_BTN)
        .hover(|s| s.bg(CRIT_BG).text_color(CRIT))
        .cursor_pointer()
        .font_family(FONT_MONO)
        .text_size(px(11.0))
        .text_color(CRIT_INK_DIM)
        .child(label)
}

/// A fleet-wide action that doesn't exist yet: visibly disabled, no handler.
fn fleet_action_stub(id: &'static str, label: String) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(4.0))
        .border_1()
        .border_color(BORDER_DANGER_BTN)
        .opacity(0.45)
        .font_family(FONT_MONO)
        .text_size(px(11.0))
        .text_color(CRIT_INK_DIM)
        .child(label)
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
