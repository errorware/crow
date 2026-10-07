use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::vault::ChangeRecord;
use super::models::ServiceUnit;
use crate::vault::Vault;
use crate::views::fleet::FleetState;
use crate::views::overview::OverviewState;

/// Maps a systemd unit name to the crow-config managed file that governs it,
/// if one is known. None means there's nothing to hand off to the Config screen yet.
fn config_file_for_service(unit: &str) -> Option<&'static str> {
    let u = unit.to_lowercase();
    if u.contains("postgres") {
        Some("pg_hba.conf")
    } else if u.contains("sshd") || u == "ssh.service" {
        Some("sshd_config")
    } else if u.contains("nginx") {
        Some("nginx.conf")
    } else if u.contains("ufw") {
        Some("ufw/user.rules")
    } else if u.contains("fail2ban") {
        Some("fail2ban/jail.local")
    } else if u.contains("journald") {
        Some("journald.conf")
    } else if u.contains("docker") {
        Some("docker/daemon.json")
    } else if u.contains("cron") {
        Some("crontab")
    } else {
        None
    }
}

pub fn service_inspector_rail(vault: &Vault, fleet: &FleetState, overview: &OverviewState, app: Entity<CrowApp>) -> impl IntoElement {
    let focused = overview.services.iter().find(|s| s.is_focused);

    div()
        .w(px(360.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        // Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("SERVICE MANAGER"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("systemctl"),
                ),
        )
        .children(match focused {
            None => Some(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(6.0))
                    .p(px(24.0))
                    .child(
                        tabler_icon(TablerIcon::Server)
                            .size(px(22.0))
                            .text_color(TEXT_FAINTER),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(TEXT_DIMMER)
                            .text_align(TextAlign::Center)
                            .child("Select a service on the left to manage it"),
                    )
                    .into_any_element(),
            ),
            Some(svc) => Some(
                render_focused_panel(svc, vault, fleet, overview, app)
                    .into_any_element(),
            ),
        })
}

fn info_cell(label: &str, value: String, value_color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(label.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(12.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(value_color)
                .child(value),
        )
}

fn render_focused_panel(svc: &ServiceUnit, vault: &Vault, fleet: &FleetState, overview: &OverviewState, app: Entity<CrowApp>) -> impl IntoElement {
    let is_active = svc.status == "ACTIVE";
    let pill_bg = match svc.status.as_str() {
        "ACTIVE" => OK_BG,
        "DEGRADED" | "PENDING" => WARN_BG,
        _ => CRIT_BG,
    };
    let config_file = config_file_for_service(&svc.name);
    let pending_action = overview.service_panel_pending_action.clone();
    let failed_banner = overview.last_change_outcome.as_ref()
        .filter(|(unit, succeeded)| unit == &svc.name && !succeeded);
    let recent = crate::app::overview::recent_change_records(vault, fleet, &svc.name, 5);
    let eli5_key = format!("{}/{}", fleet.servers.iter().find(|s| s.id == fleet.active_tab_id).map_or("local", |s| s.id.as_str()), svc.name);
    let eli5_open = overview.service_eli5_open.as_deref() == Some(eli5_key.as_str());
    let eli5_panel = eli5_open.then(|| render_eli5(overview.service_eli5_loading.contains(&eli5_key), overview.service_eli5.get(&eli5_key)));

    div()
        .id("service-inspector-scroll")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .p(px(14.0))
        // Identity
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child(svc.name.clone()),
                        )
                        // The wand: what is this, and how risky is touching it?
                        .child({
                            let (app_wand, unit) = (app.clone(), svc.name.clone());
                            div()
                                .id("btn-service-eli5")
                                .size(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(if eli5_open { WAND } else { TEXT_DIM })
                                .bg(if eli5_open { WAND.opacity(0.14) } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(WAND.opacity(0.14)).text_color(WAND))
                                .on_click(move |_ev, _window, cx| {
                                    let unit = unit.clone();
                                    app_wand.update(cx, |this, cx| this.toggle_service_eli5(&unit, cx));
                                })
                                .child(crate::components::icons::inherited_icon(crate::components::icons::TablerIcon::Wand, px(14.0)))
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .bg(pill_bg)
                                .text_color(svc.status_color())
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(5.0))
                                .py(px(2.0))
                                .child(svc.status.clone()),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .child(svc.description.clone()),
                        ),
                ),
        )
        .children(eli5_panel)
        // Info grid
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(16.0))
                .px(px(2.0))
                .child(info_cell("PID", svc.pid.clone(), TEXT_SECONDARY))
                .child(info_cell("CPU", svc.cpu.clone(), TEXT_SECONDARY))
                .child(info_cell("MEM", svc.mem.clone(), TEXT_SECONDARY))
                .child(info_cell("RSS", svc.rss.clone(), TEXT_SECONDARY))
                .child(info_cell("UPTIME", svc.uptime.clone(), TEXT_SECONDARY)),
        )
        // Failed-outcome banner — the "tell the truth if it didn't work" step
        .children(failed_banner.map(|(unit, _)| {
            let app_retry = app.clone();
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .p(px(10.0))
                .bg(CRIT_ROW_BG)
                .border_1()
                .border_color(CRIT)
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(CRIT_INK)
                        .child(format!("{} did not reach the expected state after the last action.", unit)),
                )
                .child(
                    div()
                        .id("btn-svc-retry")
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(CRIT)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgb(0xef4444)))
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hex_rgb(0x0a0a0c))
                        .on_click(move |_ev, _window, cx| {
                            app_retry.update(cx, |this, cx| {
                                this.run_service_panel_action_now("restart", cx);
                            });
                        })
                        .child("RETRY"),
                )
        }))
        // Lifecycle actions
        .child(section_divider("LIFECYCLE"))
        .child(render_actions_row(is_active, app.clone()))
        // Confirm bar for disruptive actions
        .children(pending_action.map(|action| render_pending_confirm(&svc.name, &action, overview, app.clone())))
        // Recent Apply Pipeline history for this unit
        .children(if !recent.is_empty() {
            Some(render_recent_actions(&recent))
        } else {
            None
        })
        // Cross-nav: jump to this unit's log stream
        .child({
            let app_logs = app.clone();
            let unit_name = svc.name.clone();
            div()
                .id("btn-view-service-logs")
                .flex()
                .items_center()
                .justify_between()
                .p(px(10.0))
                .bg(BG_CONTROL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    app_logs.update(cx, |this, cx| {
                        this.jump_to_service_logs(&unit_name, cx);
                    });
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("VIEW LOGS FOR THIS UNIT"),
                )
                .child(
                    tabler_icon(TablerIcon::ChevronRight)
                        .size(px(14.0))
                        .text_color(TEXT_DIM),
                )
        })
        // Config handoff
        .children(config_file.map(|file| {
            let app_cfg = app.clone();
            let file_owned = file.to_string();
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(section_divider("CONFIGURATION"))
                .child(
                    div()
                        .id("btn-edit-service-config")
                        .flex()
                        .items_center()
                        .justify_between()
                        .p(px(10.0))
                        .bg(hex_rgba(0x4ade80, 0.08))
                        .border_1()
                        .border_color(OK)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgba(0x4ade80, 0.16)))
                        .on_click(move |_ev, _window, cx| {
                            app_cfg.update(cx, |this, cx| {
                                this.open_config_for_service(&file_owned, cx);
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .child(format!("EDIT {}", file)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_DIM)
                                        .child("open in crow-config — structured, validated editing"),
                                ),
                        )
                        .child(
                            tabler_icon(TablerIcon::ChevronRight)
                                .size(px(14.0))
                                .text_color(OK),
                        ),
                )
        }))
}

/// The durable Apply Pipeline audit trail for one unit — what changed, when,
/// and whether it actually worked, sourced from the `change_records` table.
fn render_recent_actions(records: &[ChangeRecord]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(section_divider("RECENT ACTIONS"))
        .child(
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(BORDER_PANEL)
                .children(records.iter().enumerate().map(|(idx, rec)| {
                    let (dot_color, outcome_label) = match rec.outcome.as_str() {
                        "success" => (OK, "OK"),
                        "failed" => (CRIT, "FAILED"),
                        _ => (WARN, "PENDING"),
                    };
                    let when = rec.started_at.split('T').nth(1).and_then(|t| t.get(0..8)).unwrap_or(&rec.started_at).to_string();

                    div()
                        .id(ElementId::NamedInteger("svc-recent-action".into(), idx as u64))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(9.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .child(div().size(px(6.0)).rounded_full().bg(dot_color).flex_none())
                        .child(
                            div()
                                .w(px(52.0))
                                .flex_none()
                                .text_color(TEXT_FAINT)
                                .child(when),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(TEXT_SECONDARY)
                                .child(rec.action_kind.replace("service_", "").to_uppercase()),
                        )
                        .children(rec.blast_radius.clone().map(|b| {
                            div()
                                .text_color(TEXT_DIMMER)
                                .text_size(px(9.0))
                                .child(b)
                        }))
                        .child(
                            div()
                                .w(px(52.0))
                                .flex_none()
                                .text_align(TextAlign::Right)
                                .font_weight(FontWeight::BOLD)
                                .text_color(dot_color)
                                .child(outcome_label),
                        )
                })),
        )
}

fn section_divider(label: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(label.to_string()),
        )
        .child(div().flex_1().h(px(1.0)).bg(BORDER_PANEL))
}

fn action_button(
    id: &'static str,
    label: &'static str,
    fg: Rgba,
    bg: Rgba,
    border: Rgba,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .py(px(6.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(fg)
        .on_click(on_click)
        .child(label)
}

fn render_actions_row(is_active: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let app_start = app.clone();
    let app_reload = app.clone();
    let app_restart = app.clone();
    let app_stop = app.clone();

    div()
        .flex()
        .gap(px(6.0))
        .children(if !is_active {
            Some(action_button(
                "btn-svc-start",
                "▶ START",
                OK,
                OK_BG,
                OK,
                move |_ev, _window, cx| {
                    app_start.update(cx, |this, cx| {
                        this.run_service_panel_action_now("start", cx);
                    });
                },
            ))
        } else {
            None
        })
        .children(if is_active {
            Some(action_button(
                "btn-svc-reload",
                "⟳ RELOAD",
                TEXT_SECONDARY,
                BG_CONTROL,
                BORDER_DEFAULT,
                move |_ev, _window, cx| {
                    app_reload.update(cx, |this, cx| {
                        this.run_service_panel_action_now("reload", cx);
                    });
                },
            ))
        } else {
            None
        })
        .child(action_button(
            "btn-svc-restart",
            "↻ RESTART",
            WARN,
            WARN_BG,
            WARN,
            move |_ev, _window, cx| {
                app_restart.update(cx, |this, cx| {
                    this.arm_service_panel_action("restart", cx);
                });
            },
        ))
        .children(if is_active {
            Some(action_button(
                "btn-svc-stop",
                "■ STOP",
                CRIT,
                CRIT_BG,
                CRIT,
                move |_ev, _window, cx| {
                    app_stop.update(cx, |this, cx| {
                        this.arm_service_panel_action("stop", cx);
                    });
                },
            ))
        } else {
            None
        })
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

fn render_pending_confirm(unit_name: &str, action: &str, overview: &OverviewState, app: Entity<CrowApp>) -> impl IntoElement {
    let app_confirm = app.clone();
    let app_cancel = app.clone();
    let verb = action.to_uppercase();

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(10.0))
        .bg(CRIT_ROW_BG)
        .border_1()
        .border_color(CRIT)
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(CRIT_INK)
                .child(format!(
                    "{} {}? {}",
                    verb, unit_name, blast_radius_text(overview, unit_name)
                )),
        )
        .child(
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    div()
                        .id("btn-svc-confirm-cancel")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_KEY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .on_click(move |_ev, _window, cx| {
                            app_cancel.update(cx, |this, cx| {
                                this.cancel_service_panel_action(cx);
                            });
                        })
                        .child("Cancel"),
                )
                .child(
                    div()
                        .id("btn-svc-confirm-execute")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(5.0))
                        .bg(CRIT)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgb(0xef4444)))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hex_rgb(0x0a0a0c))
                        .on_click(move |_ev, _window, cx| {
                            app_confirm.update(cx, |this, cx| {
                                this.execute_service_panel_action(cx);
                            });
                        })
                        .child(format!("{} ⏎", verb)),
                ),
        )
}

/// The wand's colour.
const WAND: Rgba = Rgba { r: 0.733, g: 0.604, b: 0.969, a: 1.0 };

/// The plain-words explanation under the service's name: how risky
/// stopping or restarting it is, what it is, and what each action does.
fn render_eli5(loading: bool, result: Option<&Result<crate::views::overview::state::ServiceEli5, String>>) -> impl IntoElement {
    use crate::ai::Severity;
    let body = div().flex().flex_col().gap(px(8.0)).p(px(12.0)).bg(WAND.opacity(0.05)).border_1().border_color(WAND.opacity(0.35));
    match (loading, result) {
        (true, _) | (false, None) => body.child(div().font_family(FONT_MONO).text_size(px(10.5)).text_color(WAND).child("Asking the clanker what this service is…")),
        (false, Some(Err(e))) => body.child(div().font_family(FONT_MONO).text_size(px(10.5)).line_height(px(15.0)).text_color(CRIT).child(e.clone())),
        (false, Some(Ok(e))) => body
            .children(e.severity.as_ref().map(|(level, reason)| {
                let (label, color) = match level {
                    Severity::Low => ("LOW RISK", OK),
                    Severity::Medium => ("MEDIUM RISK", WARN),
                    Severity::High => ("HIGH RISK", hex_rgb(0xf97316)),
                    Severity::Critical => ("CRITICAL", CRIT),
                };
                div()
                    .flex()
                    .items_start()
                    .gap(px(8.0))
                    .child(div().flex_none().px(px(5.0)).py(px(1.5)).bg(color.opacity(0.15)).border_1().border_color(color).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(color).child(label))
                    .child(div().flex_1().min_w(px(0.0)).font_family(FONT_MONO).text_size(px(10.5)).line_height(px(15.0)).text_color(TEXT_SECONDARY).child(format!("to stop or restart: {reason}")))
            }))
            // One line per point, its label ("If you stop it:") bold.
            .children(e.body.lines().map(str::trim).filter(|l| !l.is_empty()).map(|line| {
                let line = line.trim_start_matches(['-', '*', ' ']).replace("**", "");
                let (label, rest) = match line.split_once(':') {
                    Some((l, r)) if l.len() <= 24 && !l.contains('`') => (Some(format!("{l}:")), r.trim().to_string()),
                    _ => (None, line.clone()),
                };
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(11.0))
                    .line_height(px(16.0))
                    .text_color(TEXT_PRIMARY)
                    .children(label.map(|l| div().font_weight(FontWeight::BOLD).text_color(WAND).child(l)))
                    .child(rest)
            }))
            .child(div().font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_FAINT).child(match &e.primary_failed {
                Some(why) => format!("via the backup, {} · the primary failed: {why}", e.via),
                None => format!("via {} · an AI's reading: check before acting on it", e.via),
            })),
    }
}
