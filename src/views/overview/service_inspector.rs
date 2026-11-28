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
        .bg(bg_rail())
        .border_l_1()
        .border_color(border_panel())
        // Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(12.0))
                .bg(bg_panel())
                .border_b_1()
                .border_color(border_panel())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_primary())
                        .child(t!("overview-service-inspector-service-manager")),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .text_color(text_dimmer())
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
                            .text_color(text_fainter()),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(font_px(11.0))
                            .text_color(text_dimmer())
                            .text_align(TextAlign::Center)
                            .child(t!("overview-service-inspector-select-a-service-on-the")),
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
                .text_size(font_px(9.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(text_dimmer())
                .child(label.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(font_px(12.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(value_color)
                .child(value),
        )
}

fn render_focused_panel(svc: &ServiceUnit, vault: &Vault, fleet: &FleetState, overview: &OverviewState, app: Entity<CrowApp>) -> impl IntoElement {
    let is_active = svc.status == "ACTIVE";
    let pill_bg = match svc.status.as_str() {
        "ACTIVE" => theme().ok_bg,
        "DEGRADED" | "PENDING" => theme().warn_bg,
        _ => theme().crit_bg,
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
                                .text_size(font_px(14.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_primary())
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
                                .text_color(if eli5_open { WAND } else { text_dim() })
                                .bg(if eli5_open { WAND.opacity(0.14) } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(WAND.opacity(0.14)).text_color(WAND))
                                .on_click(move |_ev, _window, cx| {
                                    let unit = unit.clone();
                                    app_wand.update(cx, |this, cx| this.toggle_service_eli5(&unit, cx));
                                })
                                .child(crate::components::icons::inherited_icon(crate::components::icons::TablerIcon::Wand, font_px(14.0)))
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
                                .text_size(font_px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(5.0))
                                .py(px(2.0))
                                .child(svc.status.clone()),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.5))
                                .text_color(text_dim())
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
                .child(info_cell(t!("overview-service-inspector-pid"), svc.pid.clone(), text_secondary()))
                .child(info_cell(t!("overview-service-inspector-cpu"), svc.cpu.clone(), text_secondary()))
                .child(info_cell(t!("overview-service-inspector-mem"), svc.mem.clone(), text_secondary()))
                .child(info_cell(t!("overview-service-inspector-rss"), svc.rss.clone(), text_secondary()))
                .child(info_cell(t!("overview-service-inspector-uptime"), svc.uptime.clone(), text_secondary())),
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
                .bg(theme().crit_row_bg)
                .border_1()
                .border_color(theme().crit)
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.5))
                        .text_color(theme().crit_ink)
                        .child(t!("overview-service-inspector-did-not-reach-the-expected", unit = unit)),
                )
                .child(
                    div()
                        .id("btn-svc-retry")
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(theme().crit)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgb(0xef4444)))
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hex_rgb(0x0a0a0c))
                        .on_click(move |_ev, _window, cx| {
                            app_retry.update(cx, |this, cx| {
                                this.run_service_panel_action_now("restart", cx);
                            });
                        })
                        .child(t!("overview-service-inspector-retry")),
                )
        }))
        // Lifecycle actions
        .child(section_divider(t!("overview-service-inspector-lifecycle")))
        .child(render_actions_row(is_active, app.clone()))
        // Confirm bar for disruptive actions
        .children(pending_action.map(|action| render_pending_confirm(&svc.name, &action, overview, app.clone())))
        // Unit file management (View, Edit, Dry-Run & Rollback)
        .child(section_divider(t!("overview-service-inspector-unit-definition")))
        .child({
            let app_unit = app.clone();
            let unit_name = svc.name.clone();
            div()
                .id("btn-open-unit-editor")
                .flex()
                .items_center()
                .justify_between()
                .p(px(10.0))
                .bg(accent_bg())
                .border_1()
                .border_color(accent())
                .cursor_pointer()
                .hover(|s| s.bg(accent_bg()))
                .on_click(move |_ev, _window, cx| {
                    let u = unit_name.clone();
                    app_unit.update(cx, |this, cx| {
                        this.open_unit_editor(&u, cx);
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tabler_icon(TablerIcon::FileText).size(px(15.0)).text_color(accent()))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(accent())
                                        .child(t!("overview-service-inspector-view-edit-unit")),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(9.5))
                                        .text_color(text_dim())
                                        .child(t!("overview-service-inspector-inspect-unit-file-dropins")),
                                ),
                        ),
                )
                .child(
                    tabler_icon(TablerIcon::ChevronRight)
                        .size(px(14.0))
                        .text_color(accent()),
                )
        })
        // Recent Apply Pipeline history for this unit
        .children(if !recent.is_empty() {
            Some(render_recent_actions(&recent, app.clone()))
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
                .bg(bg_control())
                .border_1()
                .border_color(border_default())
                .cursor_pointer()
                .hover(|s| s.bg(bg_row_hover()))
                .on_click(move |_ev, _window, cx| {
                    app_logs.update(cx, |this, cx| {
                        this.jump_to_service_logs(&unit_name, cx);
                    });
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_primary())
                        .child(t!("overview-service-inspector-view-logs-for-this-unit")),
                )
                .child(
                    tabler_icon(TablerIcon::ChevronRight)
                        .size(px(14.0))
                        .text_color(text_dim()),
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
                .child(section_divider(t!("overview-service-inspector-configuration")))
                .child(
                    div()
                        .id("btn-edit-service-config")
                        .flex()
                        .items_center()
                        .justify_between()
                        .p(px(10.0))
                        .bg(hex_rgba(0x4ade80, 0.08))
                        .border_1()
                        .border_color(theme().ok)
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
                                        .text_size(font_px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme().ok)
                                        .child(t!("overview-service-inspector-edit", file = file)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(9.5))
                                        .text_color(text_dim())
                                        .child(t!("overview-service-inspector-open-in-crow-config-structured")),
                                ),
                        )
                        .child(
                            tabler_icon(TablerIcon::ChevronRight)
                                .size(px(14.0))
                                .text_color(theme().ok),
                        ),
                )
        }))
}

/// The durable Apply Pipeline audit trail for one unit — what changed, when,
/// and whether it actually worked, sourced from the `change_records` table.
fn render_recent_actions(records: &[ChangeRecord], app: Entity<CrowApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(section_divider(t!("overview-service-inspector-recent-actions")))
        .child(
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(border_panel())
                .children(records.iter().enumerate().map(|(idx, rec)| {
                    let (dot_color, outcome_label) = match rec.outcome.as_str() {
                        "success" => (theme().ok, t!("overview-service-inspector-ok")),
                        "failed" => (theme().crit, t!("overview-service-inspector-failed")),
                        _ => (theme().warn, t!("overview-service-inspector-pending")),
                    };
                    let when = rec.started_at.split('T').nth(1).and_then(|t| t.get(0..8)).unwrap_or(&rec.started_at).to_string();
                    let is_unit_edit = rec.action_kind == "service_unit_edit";
                    let app_rb = app.clone();
                    let rec_owned = rec.clone();

                    div()
                        .id(ElementId::NamedInteger("svc-recent-action".into(), idx as u64))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(9.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(border_panel())
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .child(div().size(px(6.0)).rounded_full().bg(dot_color).flex_none())
                        .child(
                            div()
                                .w(px(52.0))
                                .flex_none()
                                .text_color(text_faint())
                                .child(when),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(text_secondary())
                                .child(rec.action_kind.replace("service_", "").to_uppercase()),
                        )
                        .children(rec.blast_radius.clone().map(|b| {
                            div()
                                .text_color(text_dimmer())
                                .text_size(font_px(9.0))
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
                        .children(if is_unit_edit {
                            Some(
                                div()
                                    .id(ElementId::NamedInteger("btn-rail-rollback".into(), idx as u64))
                                    .px(px(5.0))
                                    .py(px(1.5))
                                    .bg(bg_control())
                                    .border_1()
                                    .border_color(theme().warn)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme().warn.opacity(0.15)))
                                    .font_family(FONT_MONO)
                                    .text_size(font_px(8.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme().warn)
                                    .on_click(move |_ev, _window, cx| {
                                        let r = rec_owned.clone();
                                        app_rb.update(cx, |this, cx| {
                                            this.rollback_unit_action(&r, cx);
                                        });
                                    })
                                    .child(t!("overview-service-inspector-rollback")),
                            )
                        } else {
                            None
                        })
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
                .text_size(font_px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(text_dimmer())
                .child(label.to_string()),
        )
        .child(div().flex_1().h(px(1.0)).bg(border_panel()))
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
        .hover(|s| s.bg(bg_row_hover()))
        .font_family(FONT_MONO)
        .text_size(font_px(10.5))
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
                t!("overview-service-inspector-start"),
                theme().ok,
                theme().ok_bg,
                theme().ok,
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
                t!("overview-service-inspector-reload"),
                text_secondary(),
                bg_control(),
                border_default(),
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
            t!("overview-service-inspector-restart"),
            theme().warn,
            theme().warn_bg,
            theme().warn,
            move |_ev, _window, cx| {
                app_restart.update(cx, |this, cx| {
                    this.arm_service_panel_action("restart", cx);
                });
            },
        ))
        .children(if is_active {
            Some(action_button(
                "btn-svc-stop",
                t!("overview-service-inspector-stop"),
                theme().crit,
                theme().crit_bg,
                theme().crit,
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
        Some(b) if b.established == 0 && b.listening == 0 => t!("overview-service-inspector-no-active-connections-will-be").to_string(),
        Some(b) => t!("overview-service-inspector-established-and-listening-will-be", established = b.established, listening = b.listening),
        None => t!("overview-service-inspector-checking-active-connections").to_string(),
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
        .bg(theme().crit_row_bg)
        .border_1()
        .border_color(theme().crit)
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(font_px(11.0))
                .text_color(theme().crit_ink)
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
                        .border_color(border_key())
                        .cursor_pointer()
                        .hover(|s| s.bg(bg_row_hover()))
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.5))
                        .text_color(text_tertiary())
                        .on_click(move |_ev, _window, cx| {
                            app_cancel.update(cx, |this, cx| {
                                this.cancel_service_panel_action(cx);
                            });
                        })
                        .child(t!("overview-service-inspector-cancel")),
                )
                .child(
                    div()
                        .id("btn-svc-confirm-execute")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py(px(5.0))
                        .bg(theme().crit)
                        .cursor_pointer()
                        .hover(|s| s.bg(hex_rgb(0xef4444)))
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.5))
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
pub(crate) const WAND: Rgba = Rgba { r: 0.733, g: 0.604, b: 0.969, a: 1.0 };

/// The plain-words explanation under the service's name: how risky
/// stopping or restarting it is, what it is, and what each action does.
pub(crate) fn render_eli5(loading: bool, result: Option<&Result<crate::views::overview::state::ServiceEli5, String>>) -> impl IntoElement {
    use crate::ai::Severity;
    let body = div().flex().flex_col().gap(px(8.0)).p(px(12.0)).bg(WAND.opacity(0.05)).border_1().border_color(WAND.opacity(0.35));
    match (loading, result) {
        (true, _) | (false, None) => body.child(div().font_family(FONT_MONO).text_size(font_px(10.5)).text_color(WAND).child(t!("overview-service-inspector-asking-the-clanker-what-this"))),
        (false, Some(Err(e))) => body.child(div().font_family(FONT_MONO).text_size(font_px(10.5)).line_height(px(15.0)).text_color(theme().crit).child(e.clone())),
        (false, Some(Ok(e))) => body
            .children(e.severity.as_ref().map(|(level, reason)| {
                let (label, color) = match level {
                    Severity::Low => (t!("overview-service-inspector-low-risk"), theme().ok),
                    Severity::Medium => (t!("overview-service-inspector-medium-risk"), theme().warn),
                    Severity::High => (t!("overview-service-inspector-high-risk"), hex_rgb(0xf97316)),
                    Severity::Critical => (t!("overview-service-inspector-critical"), theme().crit),
                };
                div()
                    .flex()
                    .items_start()
                    .gap(px(8.0))
                    .child(crate::components::button::badge(color).flex_none().text_size(font_px(9.0)).font_weight(FontWeight::BOLD).child(label))
                    .child(div().flex_1().min_w(px(0.0)).font_family(FONT_MONO).text_size(font_px(10.5)).line_height(px(15.0)).text_color(text_secondary()).child(t!("overview-service-inspector-to-stop-or-restart", reason = reason)))
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
                    .text_size(font_px(11.0))
                    .line_height(px(16.0))
                    .text_color(text_primary())
                    .children(label.map(|l| div().font_weight(FontWeight::BOLD).text_color(WAND).child(l)))
                    .child(rest)
            }))
            .child(div().font_family(FONT_MONO).text_size(font_px(9.0)).text_color(text_faint()).child(match &e.primary_failed {
                Some(why) => t!("overview-service-inspector-via-the-backup-the-primary", via = e.via, why = why),
                None => t!("overview-service-inspector-via-an-ais-reading-check", via = e.via),
            })),
    }
}
