//! Settings → Agent (ERR-208): user-level background service (`systemd --user`
//! on Linux, `LaunchAgent` on macOS), bottleneck detector rules, and selective
//! per-server enrollment.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::agent::service::ServiceBackend;
use crate::app::agent::AgentState;
use crate::app::CrowApp;
use crate::components::button::{badge, button, Tone};
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::metrics::bottlenecks::{FLAG_CPU, FLAG_DISK_RATE, FLAG_LOAD, FLAG_MEM};
use crate::theme::*;
use crate::vault::ServerRecord;

const INTERVAL_CHOICES: [(u64, &str); 4] = [
    (30, "30s"),
    (60, "60s"),
    (120, "2m"),
    (300, "5m"),
];

pub fn render_agent_view(
    app: Entity<CrowApp>,
    agent: &AgentState,
    servers: &[ServerRecord],
    vault_password_enabled: bool,
) -> impl IntoElement {
    let enrolled_count = servers.iter().filter(|s| s.agent_installed).count();
    let total_servers = servers.len();
    let (app_sample, app_refresh) = (app.clone(), app.clone());

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_APP)
        .font_family(FONT_MONO)
        // Top Header Bar
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(t!("settings-agent-header-title")),
                )
                .child(
                    badge(if enrolled_count > 0 { OK } else { TEXT_SECONDARY })
                        .text_size(px(9.5))
                        .child(t!(
                            "settings-agent-enrolled-count",
                            enrolled = enrolled_count,
                            total = total_servers
                        )),
                )
                .child(div().flex_1())
                .child(
                    button("btn-agent-sample-now", Tone::Neutral)
                        .when(agent.busy, |d| d.opacity(0.5))
                        .when(!agent.busy, |d| {
                            d.on_click(move |_ev, _window, cx| {
                                app_sample.update(cx, |this, cx| this.run_agent_sample_now(cx))
                            })
                        })
                        .child(t!("settings-agent-sample-now")),
                )
                .child(
                    div()
                        .id("btn-agent-refresh")
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .h(px(26.0))
                        .px(px(10.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| {
                            app_refresh.update(cx, |this, cx| this.refresh_agent_state(cx))
                        })
                        .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                        .child(t!("settings-plugins-re-check")),
                ),
        )
        // Main Scrollable Body
        .child(
            div()
                .id("settings-agent-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                // Intro callout
                .child(
                    div()
                        .p(px(12.0))
                        .bg(BG_OVERLAY_PANEL)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .text_size(px(10.5))
                        .line_height(px(15.0))
                        .text_color(TEXT_MUTED)
                        .child(t!("settings-agent-intro")),
                )
                // Optional notice banner
                .children(agent.notice.as_ref().map(|(ok, msg)| {
                    div()
                        .p(px(10.0))
                        .bg(if *ok { BG_CONTROL_ALT } else { CRIT_ROW_BG })
                        .border_1()
                        .border_color(if *ok { OK } else { CRIT })
                        .text_size(px(10.5))
                        .text_color(if *ok { OK } else { CRIT })
                        .child(msg.clone())
                }))
                // Optional Vault password warning
                .when(vault_password_enabled, |d| {
                    d.child(
                        div()
                            .p(px(10.0))
                            .bg(WARN_BG)
                            .border_1()
                            .border_color(WARN)
                            .text_size(px(10.5))
                            .text_color(WARN)
                            .child(t!("settings-agent-vault-password-warning")),
                    )
                })
                // 1. OS Service Control Card
                .child(service_control_card(app.clone(), agent))
                // 2. Performance Bottleneck Detector Card
                .child(bottleneck_rules_card(app.clone(), agent))
                // 3. Server Enrollment Table
                .child(enrollment_table_card(app, agent, servers)),
        )
}

fn service_control_card(app: Entity<CrowApp>, agent: &AgentState) -> impl IntoElement {
    let backend_label = agent
        .service
        .backend
        .unwrap_or_else(ServiceBackend::current)
        .label();
    let unit_path = agent
        .service
        .unit_path
        .as_ref()
        .map(|p| p.display().to_string())
        .or_else(|| crate::agent::service::current_unit_path().map(|p| p.display().to_string()))
        .unwrap_or_else(|| "—".into());

    let (status_label, status_color) = if agent.service.running {
        (t!("settings-agent-status-active").to_string(), OK)
    } else if agent.service.installed {
        (t!("settings-agent-status-stopped").to_string(), WARN)
    } else {
        (t!("settings-agent-status-not-installed").to_string(), TEXT_MUTED)
    };

    let pid_text = agent
        .service
        .service_pid
        .or(agent.heartbeat.agent_pid)
        .map(|p| format!("PID {p}"))
        .unwrap_or_else(|| "—".into());

    let last_tick_text = agent
        .heartbeat
        .agent_last_tick
        .map(format_ago)
        .unwrap_or_else(|| t!("settings-agent-never").to_string());

    let (app_install, app_uninstall) = (app.clone(), app.clone());

    div()
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .p(px(14.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(t!("settings-agent-service-card-title")),
                )
                .child(badge(status_color).text_size(px(9.5)).child(status_label))
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child(format!("{backend_label} · {pid_text}")),
                )
                .child(div().flex_1())
                .child(
                    button("btn-agent-install", Tone::Primary)
                        .when(agent.busy, |d| d.opacity(0.5))
                        .when(!agent.busy, |d| {
                            d.on_click(move |_ev, _window, cx| {
                                app_install.update(cx, |this, cx| this.install_and_start_agent_service(cx))
                            })
                        })
                        .child(if agent.service.installed {
                            t!("settings-agent-restart-service")
                        } else {
                            t!("settings-agent-enable-and-start")
                        }),
                )
                .when(agent.service.installed, |d| {
                    d.child(
                        button("btn-agent-uninstall", Tone::Danger)
                            .when(agent.busy, |b| b.opacity(0.5))
                            .when(!agent.busy, |b| {
                                b.on_click(move |_ev, _window, cx| {
                                    app_uninstall.update(cx, |this, cx| this.stop_and_uninstall_agent_service(cx))
                                })
                            })
                            .child(t!("settings-agent-stop-and-remove")),
                    )
                }),
        )
        // Metadata grid
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(18.0))
                .pt(px(4.0))
                .border_t_1()
                .border_color(BORDER_ROW)
                .child(meta_kv(t!("settings-agent-unit-file").to_string(), unit_path))
                .child(meta_kv(t!("settings-agent-last-sample").to_string(), last_tick_text))
                .child(meta_kv(
                    t!("settings-agent-coordination").to_string(),
                    t!("settings-agent-yields-when-gui-open").to_string(),
                )),
        )
        // Poll interval picker
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .pt(px(6.0))
                .border_t_1()
                .border_color(BORDER_ROW)
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child(t!("settings-agent-poll-interval")),
                )
                .children(INTERVAL_CHOICES.iter().map(|&(secs, label)| {
                    let selected = agent.interval_secs == secs;
                    let app_int = app.clone();
                    div()
                        .id(SharedString::from(format!("agent-interval-{secs}")))
                        .px(px(9.0))
                        .py(px(3.0))
                        .border_1()
                        .border_color(if selected { OK } else { BORDER_DEFAULT })
                        .bg(if selected { BG_CONTROL_ALT } else { BG_CONTROL })
                        .text_size(px(10.0))
                        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(if selected { TEXT_PRIMARY } else { TEXT_DIM })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| {
                            app_int.update(cx, |this, cx| this.set_agent_interval_secs(secs, cx))
                        })
                        .child(label)
                })),
        )
}

fn meta_kv(label: String, value: String) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(div().text_size(px(9.0)).text_color(TEXT_FAINT).child(label))
        .child(div().text_size(px(10.5)).text_color(TEXT_SECONDARY).child(value))
}

fn bottleneck_rules_card(app: Entity<CrowApp>, agent: &AgentState) -> impl IntoElement {
    let b = &agent.bottlenecks;
    let rules = [
        (
            "rule-cpu",
            FLAG_CPU,
            b.cpu_enabled,
            t!("settings-agent-rule-cpu-title").to_string(),
            t!("settings-agent-rule-cpu-desc").to_string(),
        ),
        (
            "rule-load",
            FLAG_LOAD,
            b.load_enabled,
            t!("settings-agent-rule-load-title").to_string(),
            t!("settings-agent-rule-load-desc").to_string(),
        ),
        (
            "rule-mem",
            FLAG_MEM,
            b.mem_enabled,
            t!("settings-agent-rule-mem-title").to_string(),
            t!("settings-agent-rule-mem-desc").to_string(),
        ),
        (
            "rule-disk",
            FLAG_DISK_RATE,
            b.disk_rate_enabled,
            t!("settings-agent-rule-disk-title").to_string(),
            t!("settings-agent-rule-disk-desc").to_string(),
        ),
    ];

    div()
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .p(px(14.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(t!("settings-agent-bottlenecks-card-title")),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child(t!("settings-agent-bottlenecks-card-subtitle")),
                ),
        )
        .children(rules.into_iter().map(|(id, flag_key, enabled, title, desc)| {
            let app_toggle = app.clone();
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .py(px(6.0))
                .border_t_1()
                .border_color(BORDER_ROW)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(div().text_size(px(10.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_PRIMARY).child(title))
                        .child(div().text_size(px(9.5)).text_color(TEXT_MUTED).child(desc)),
                )
                .child(
                    div()
                        .id(SharedString::from(id))
                        .px(px(10.0))
                        .py(px(4.0))
                        .border_1()
                        .border_color(if enabled { OK } else { BORDER_DEFAULT })
                        .bg(if enabled { BG_CONTROL_ALT } else { BG_CONTROL })
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if enabled { OK } else { TEXT_DIM })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_toggle.update(cx, |this, cx| {
                                this.toggle_agent_bottleneck_rule(flag_key, !enabled, cx)
                            })
                        })
                        .child(if enabled {
                            t!("settings-agent-enabled")
                        } else {
                            t!("settings-agent-disabled")
                        }),
                )
        }))
}

fn enrollment_table_card(
    app: Entity<CrowApp>,
    agent: &AgentState,
    servers: &[ServerRecord],
) -> impl IntoElement {
    let (app_all_on, app_all_off) = (app.clone(), app.clone());

    div()
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .p(px(14.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(t!("settings-agent-enrollment-card-title")),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child(t!("settings-agent-enrollment-card-subtitle")),
                )
                .child(div().flex_1())
                .child(
                    button("btn-agent-enroll-all", Tone::Neutral)
                        .on_click(move |_ev, _window, cx| {
                            app_all_on.update(cx, |this, cx| this.set_all_servers_agent_enrolled(true, cx))
                        })
                        .child(t!("settings-agent-enroll-all")),
                )
                .child(
                    button("btn-agent-unenroll-all", Tone::Neutral)
                        .on_click(move |_ev, _window, cx| {
                            app_all_off.update(cx, |this, cx| this.set_all_servers_agent_enrolled(false, cx))
                        })
                        .child(t!("settings-agent-unenroll-all")),
                ),
        )
        .when(servers.is_empty(), |d| {
            d.child(
                div()
                    .py(px(12.0))
                    .text_size(px(10.5))
                    .text_color(TEXT_FAINT)
                    .child(t!("settings-agent-no-servers-in-fleet")),
            )
        })
        .children(servers.iter().enumerate().map(|(idx, srv)| {
            let enrolled = srv.agent_installed;
            let srv_id = srv.id.clone();
            let app_row = app.clone();
            let last_sample = agent
                .latest_sample_by_server
                .get(&srv.id)
                .copied()
                .map(format_ago)
                .unwrap_or_else(|| t!("settings-agent-never").to_string());
            let bottlenecks = agent.bottlenecks_for_server(&srv.id);

            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .py(px(8.0))
                .border_t_1()
                .border_color(BORDER_ROW)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(3.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(srv.name.clone()),
                                )
                                .when(!srv.env.is_empty(), |d| {
                                    d.child(badge(TEXT_DIM).text_size(px(9.0)).child(srv.env.to_uppercase()))
                                })
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .child(format!("{}@{}:{}", srv.login_user, srv.host, srv.port)),
                                ),
                        )
                        .children(bottlenecks.into_iter().map(|b| {
                            let c = if b.level == "CRIT" { CRIT } else { WARN };
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(badge(c).text_size(px(9.0)).child(b.level.clone()))
                                .child(div().text_size(px(10.0)).text_color(c).child(b.detail.clone()))
                        })),
                )
                .child(
                    div()
                        .text_size(px(9.5))
                        .text_color(TEXT_FAINT)
                        .child(t!("settings-agent-last-sample-row", when = last_sample)),
                )
                .child(
                    div()
                        .id(ElementId::NamedInteger("btn-agent-enroll".into(), idx as u64))
                        .px(px(10.0))
                        .py(px(4.0))
                        .border_1()
                        .border_color(if enrolled { OK } else { BORDER_DEFAULT })
                        .bg(if enrolled { BG_CONTROL_ALT } else { BG_CONTROL })
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if enrolled { OK } else { TEXT_DIM })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            let id = srv_id.clone();
                            app_row.update(cx, |this, cx| this.set_server_agent_enrolled(&id, !enrolled, cx))
                        })
                        .child(if enrolled {
                            t!("settings-agent-enrolled")
                        } else {
                            t!("settings-agent-not-enrolled")
                        }),
                )
        }))
}

fn format_ago(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now.saturating_sub(ts);
    if diff < 10 {
        "just now".into()
    } else if diff < 60 {
        format!("{diff}s ago")
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86_400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86_400)
    }
}
