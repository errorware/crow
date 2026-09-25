pub mod models;
pub mod detector;
pub mod non_operational;
pub mod new_rule_modal;
pub mod rules_format;
pub mod state;
pub mod commands;

#[allow(unused_imports)]
#[cfg(test)]
pub use models::{default_active_ufw_state, default_ufw_rules};
pub use models::{
    common_quick_ports, correlate_port_firewall, FirewallBackend,
    FirewallOperationalState, FirewallRule, FirewallStatusSummary, PortFirewallMatch,
    RuleAction, RuleDirection, RuleProtocol,
};
pub use detector::detect_firewall_status;
pub use non_operational::non_operational_view;
pub use new_rule_modal::{new_rule_modal, NewRuleState};
#[allow(unused_imports)]
pub use rules_format::{generate_user_rules_content, parse_user_rules_content};
pub use state::FirewallState;

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use std::collections::HashMap;
use crate::app::CrowApp;
use crate::config::ConfigFileState;
use crate::components::icons::{TablerIcon, tabler_icon, inherited_icon};

pub fn firewall_view(
    app: Entity<CrowApp>,
    fw: &FirewallState,
    config_file_states: &HashMap<String, ConfigFileState>,
) -> AnyElement {
    let state = &fw.status;

    // Check operational status
    match state {
        FirewallOperationalState::Active(summary) if summary.is_active => {
            render_active_firewall(summary, app, fw, config_file_states).into_any_element()
        }
        FirewallOperationalState::Inactive { backend, reason, detected_binaries, .. } => {
            non_operational_view(*backend, reason, detected_binaries, app).into_any_element()
        }
        FirewallOperationalState::Unmanaged { detected_binaries, reason } => {
            non_operational_view(FirewallBackend::NoneDetected, reason, detected_binaries, app).into_any_element()
        }
        FirewallOperationalState::Active(summary) => {
            // Active backend struct but is_active == false
            non_operational_view(summary.backend, "Firewall is currently disabled by administrator.", &["ufw".into()], app).into_any_element()
        }
    }
}

fn render_active_firewall(
    summary: &FirewallStatusSummary,
    app: Entity<CrowApp>,
    fw: &FirewallState,
    config_file_states: &HashMap<String, ConfigFileState>,
) -> impl IntoElement {
    let app_new_rule = app.clone();
    let app_toggle_active = app.clone();
    let app_inspect_cfg = app.clone();
    let app_reload = app.clone();
    let app_stage = app.clone();
    let app_toggle_audit = app.clone();

    let file_state = config_file_states.get("user.rules");
    let (is_modified, add_count, del_count, active_rev) = if let Some(st) = file_state {
        let (a, d) = st.diff_stats();
        (st.is_modified(), a, d, st.active_revision)
    } else {
        (false, 0, 0, 1)
    };

    let query = fw.search_query.to_lowercase();
    let action_filter = fw.action_filter;

    let filtered_rules: Vec<&FirewallRule> = summary
        .rules
        .iter()
        .filter(|r| {
            if let Some(target_act) = action_filter {
                r.action == target_act
            } else {
                true
            }
        })
        .filter(|r| {
            if query.is_empty() {
                return true;
            }
            r.port.to_lowercase().contains(&query)
                || r.source.to_lowercase().contains(&query)
                || r.destination.to_lowercase().contains(&query)
                || r.comment.as_deref().unwrap_or("").to_lowercase().contains(&query)
                || r.action.label().to_lowercase().contains(&query)
        })
        .collect();

    let total_rules = summary.rules.len();
    let allow_count = summary.rules.iter().filter(|r| r.action == RuleAction::Allow).count();
    let deny_count = summary.rules.iter().filter(|r| r.action == RuleAction::Deny).count();
    let limit_count = summary.rules.iter().filter(|r| r.action == RuleAction::Limit).count();

    let modal_element = if fw.show_new_rule_modal {
        Some(new_rule_modal(&fw.new_rule, app.clone()).into_any_element())
    } else {
        None
    };

    div()
        .id("firewall-management-view")
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Header Toolbar
        .child(
            div()
                .h(px(44.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(tabler_icon(TablerIcon::ShieldCheck).size(px(18.0)).text_color(OK))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child("FIREWALL & NETWORK SECURITY"),
                        )
                        .child(
                            div()
                                .bg(OK_BG)
                                .border_1()
                                .border_color(OK)
                                .text_color(OK)
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded_sm()
                                .child(format!("{} · ONLINE", summary.backend.short_name())),
                        )
                        .child(
                            div()
                                .bg(hex_rgba(0x8ab4ff, 0.12))
                                .text_color(hex_rgb(0x8ab4ff))
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded_sm()
                                .child(format!("{} ACTIVE RULES", total_rules)),
                        )
                        .child(if is_modified {
                            div()
                                .bg(hex_rgba(0xf59e0b, 0.15))
                                .border_1()
                                .border_color(hex_rgba(0xf59e0b, 0.5))
                                .text_color(WARN)
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded_sm()
                                .child(format!("DIFF: +{} −{}", add_count, del_count))
                        } else {
                            div()
                                .bg(hex_rgba(0x3ecf6e, 0.1))
                                .border_1()
                                .border_color(hex_rgba(0x3ecf6e, 0.3))
                                .text_color(OK)
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded_sm()
                                .child(format!("v{} · AUDITED", active_rev))
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        // STAGE AUDIT COMMIT (if modified)
                        .children(if is_modified {
                            Some(
                                div()
                                    .id("btn-stage-firewall-rules")
                                    .px(px(10.0))
                                    .py(px(4.5))
                                    .bg(OK)
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(hex_rgb(0x34d399)))
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x0a0a0c))
                                    .on_click(move |_ev, _window, cx| {
                                        app_stage.update(cx, |this, cx| {
                                            this.stage_firewall_rules("Staged firewall rule changes", cx);
                                        });
                                    })
                                    .child(format!("STAGE AUDIT (+{} −{})", add_count, del_count)),
                            )
                        } else {
                            None
                        })
                        // AUDIT RAIL TOGGLE
                        .child(
                            div()
                                .id("btn-toggle-firewall-audit-rail")
                                .px(px(10.0))
                                .py(px(4.5))
                                .bg(if fw.show_audit_rail { hex_rgba(0x8ab4ff, 0.15) } else { BG_CONTROL })
                                .border_1()
                                .border_color(if fw.show_audit_rail { hex_rgba(0x8ab4ff, 0.4) } else { BORDER_DEFAULT })
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if fw.show_audit_rail { hex_rgb(0x8ab4ff) } else { TEXT_SECONDARY })
                                .on_click(move |_ev, _window, cx| {
                                    app_toggle_audit.update(cx, |this, cx| {
                                        this.toggle_firewall_audit_rail(cx);
                                    });
                                })
                                .child(if fw.show_audit_rail { "AUDIT RAIL [ON]" } else { "AUDIT RAIL [OFF]" }),
                        )
                        // Reload Firewall Button
                        .child(
                            div()
                                .id("btn-reload-firewall")
                                .px(px(10.0))
                                .py(px(4.5))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    app_reload.update(cx, |this, cx| {
                                        this.reload_firewall(cx);
                                    });
                                })
                                .child("RELOAD ENGINE"),
                        )
                        // View user.rules in Config Editor
                        .child(
                            div()
                                .id("btn-firewall-open-config")
                                .px(px(10.0))
                                .py(px(4.5))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    app_inspect_cfg.update(cx, |this, cx| {
                                        this.select_managed_file("user.rules", cx);
                                        this.set_view("config", cx);
                                    });
                                })
                                .child("INSPECT user.rules"),
                        )
                        // Disable / Enable Toggle
                        .child(
                            div()
                                .id("btn-toggle-firewall-active")
                                .px(px(10.0))
                                .py(px(4.5))
                                .bg(CRIT_BG)
                                .border_1()
                                .border_color(CRIT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgba(0xf87171, 0.25)))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(CRIT)
                                .on_click(move |_ev, _window, cx| {
                                    app_toggle_active.update(cx, |this, cx| {
                                        this.toggle_firewall_active(cx);
                                    });
                                })
                                .child("DISABLE FIREWALL"),
                        )
                        // + ADD RULE Button
                        .child(
                            div()
                                .id("btn-open-new-rule-modal")
                                .px(px(12.0))
                                .py(px(4.5))
                                .bg(OK)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgb(0x34d399)))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0a0a0c))
                                .on_click(move |_ev, _window, cx| {
                                    app_new_rule.update(cx, |this, cx| {
                                        this.open_new_firewall_rule_modal(cx);
                                    });
                                })
                                .child("+ ADD RULE"),
                        ),
                ),
        )
        // 2. Global Policy Strip & Quick Port Toggles
        .child(
            div()
                .p(px(12.0))
                .px(px(16.0))
                .bg(hex_rgba(0x000000, 0.25))
                .border_b_1()
                .border_color(BORDER_ROW)
                .flex()
                .flex_col()
                .gap(px(10.0))
                // Row 1: Global Policies summary & Filter Pills
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            // Global Policies Badges
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("DEFAULT POLICIES:"),
                                )
                                .child(
                                    div()
                                        .px(px(7.0))
                                        .py(px(2.5))
                                        .bg(CRIT_BG)
                                        .border_1()
                                        .border_color(CRIT)
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(CRIT)
                                        .child(format!("INCOMING: {}", summary.default_incoming.label())),
                                )
                                .child(
                                    div()
                                        .px(px(7.0))
                                        .py(px(2.5))
                                        .bg(OK_BG)
                                        .border_1()
                                        .border_color(OK)
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .child(format!("OUTGOING: {}", summary.default_outgoing.label())),
                                )
                                .child(
                                    div()
                                        .px(px(7.0))
                                        .py(px(2.5))
                                        .bg(BG_CONTROL)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_SECONDARY)
                                        .child(format!("ROUTED: {}", summary.default_forward.label())),
                                ),
                        )
                        // Filter Pills
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .children([
                                    (None, "All Rules", total_rules),
                                    (Some(RuleAction::Allow), "Allow", allow_count),
                                    (Some(RuleAction::Deny), "Deny", deny_count),
                                    (Some(RuleAction::Limit), "Limit", limit_count),
                                ].iter().map(|(act, label, count)| {
                                    let is_sel = action_filter == *act;
                                    let a = *act;
                                    let app_flt = app.clone();
                                    div()
                                        .id(ElementId::NamedInteger(format!("flt-act-{:?}", a).into(), 0))
                                        .px(px(8.0))
                                        .py(px(3.0))
                                        .rounded_sm()
                                        .border_1()
                                        .border_color(if is_sel { hex_rgba(0x8ab4ff, 0.4) } else { hex_rgba(0, 0.0) })
                                        .bg(if is_sel { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { TEXT_MAX } else { TEXT_MUTED })
                                        .on_click(move |_ev, _window, cx| {
                                            app_flt.update(cx, |this, cx| {
                                                this.set_firewall_action_filter(a, cx);
                                            });
                                        })
                                        .child(format!("{} ({})", label, count))
                                })),
                        ),
                )
                // Row 2: Service Quick Port Toggles
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MUTED)
                                .child("ONE-CLICK PORT TOGGLES:"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.0))
                                .children(common_quick_ports().iter().map(|qp| {
                                    let is_allowed = summary.rules.iter().any(|r| {
                                        r.action == RuleAction::Allow && r.port == qp.port.to_string()
                                    });
                                    let port_num = qp.port;
                                    let proto = qp.protocol;
                                    let label = qp.name;
                                    let app_qp = app.clone();

                                    div()
                                        .id(ElementId::NamedInteger(format!("qp-toggle-{}", port_num).into(), 0))
                                        .px(px(8.0))
                                        .py(px(3.5))
                                        .bg(if is_allowed { OK_BG } else { BG_CONTROL })
                                        .border_1()
                                        .border_color(if is_allowed { OK } else { BORDER_DEFAULT })
                                        .rounded_sm()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            app_qp.update(cx, |this, cx| {
                                                this.toggle_quick_port(port_num, proto, label, cx);
                                            });
                                        })
                                        .flex()
                                        .items_center()
                                        .gap(px(5.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_allowed { OK } else { TEXT_SECONDARY })
                                                .child(if is_allowed { format!("✓ {}", qp.name) } else { format!("+ {}", qp.name) }),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(if is_allowed { OK } else { TEXT_FAINT })
                                                .child(format!(":{}", qp.port)),
                                        )
                                })),
                        ),
                ),
        )
        // 3. Rules Table Header & Search Filter
        .child(
            div()
                .h(px(36.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_ROW)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MUTED)
                                .child("ACTIVE FILTERING DIRECTIVES"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_FAINT)
                                .child(format!("Showing {} of {} rules", filtered_rules.len(), total_rules)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .child(tabler_icon(TablerIcon::Search).size(px(11.0)).text_color(TEXT_MUTED))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(if fw.search_query.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                .child(if fw.search_query.is_empty() {
                                    "Filter rules by port, IP, comment...".to_string()
                                } else {
                                    fw.search_query.clone()
                                }),
                        )
                        .children(if !fw.search_query.is_empty() {
                            let app_clr = app.clone();
                            Some(
                                div()
                                    .id("btn-clear-fw-search")
                                    .p(px(2.0))
                                    .cursor_pointer()
                                    .on_click(move |_ev, _window, cx| {
                                        app_clr.update(cx, |this, cx| {
                                            this.set_firewall_search("", cx);
                                        });
                                    })
                                    .child(tabler_icon(TablerIcon::X).size(px(10.0)).text_color(TEXT_MUTED))
                            )
                        } else {
                            None
                        }),
                ),
        )
        // 4. Scrollable Rule Matrix
        .child(
            div()
                .id("firewall-rules-table")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(16.0))
                .flex()
                .flex_col()
                .gap(px(6.0))
                // Column Headers
                .child(
                    div()
                        .h(px(26.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .px(px(12.0))
                        .bg(hex_rgba(0x000000, 0.3))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .child(div().w(px(36.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("#"))
                        .child(div().w(px(80.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("ACTION"))
                        .child(div().w(px(60.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("DIR"))
                        .child(div().w(px(110.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("PORT / PROTO"))
                        .child(div().w(px(160.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("SOURCE"))
                        .child(div().w(px(120.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("DESTINATION"))
                        .child(div().flex_1().font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("COMMENT"))
                        .child(div().w(px(60.0)).font_family(FONT_MONO).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MUTED).child("MANAGE")),
                )
                // Rule Rows
                .children(if filtered_rules.is_empty() {
                    vec![
                        div()
                            .p(px(24.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(11.0))
                                    .text_color(TEXT_FAINT)
                                    .child("No firewall rules match your filter"),
                            )
                            .into_any_element(),
                    ]
                } else {
                    filtered_rules.iter().map(|rule| {
                        render_rule_row(rule, app.clone()).into_any_element()
                    }).collect()
                }),
        )
        // Toast Notification Overlay
        .children(if let Some(msg) = fw.pending.as_ref().map(|p| format!("Running: {p} …")).as_ref().or(fw.toast.as_ref()) {
            Some(
                div()
                    .absolute()
                    .bottom(px(16.0))
                    .right(px(16.0))
                    .px(px(14.0))
                    .py(px(8.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_STRONG)
                    .rounded_md()
                    .shadow_lg()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(tabler_icon(TablerIcon::Check).size(px(14.0)).text_color(OK))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(TEXT_PRIMARY)
                            .child(msg.clone()),
                    ),
            )
        } else {
            None
        })
        // New Rule Modal
        .children(modal_element)
}

fn render_rule_row(rule: &FirewallRule, app: Entity<CrowApp>) -> impl IntoElement {
    let r_id = rule.id.clone();
    let app_del = app.clone();

    let (act_col, act_bg) = match rule.action {
        RuleAction::Allow => (OK, OK_BG),
        RuleAction::Deny => (CRIT, CRIT_BG),
        RuleAction::Reject => (hex_rgb(0xf43f5e), hex_rgba(0xf43f5e, 0.15)),
        RuleAction::Limit => (WARN, WARN_BG),
    };

    div()
        .id(ElementId::NamedInteger(format!("rule-row-{}", rule.id).into(), rule.number as u64))
        .h(px(34.0))
        .flex_none()
        .flex()
        .items_center()
        .px(px(12.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_ROW)
        .rounded_sm()
        .hover(|s| s.bg(BG_ROW_HOVER).border_color(BORDER_DEFAULT))
        // 1. Number
        .child(
            div()
                .w(px(36.0))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_FAINT)
                .child(format!("[{:2}]", rule.number)),
        )
        // 2. Action Pill
        .child(
            div()
                .w(px(80.0))
                .child(
                    div()
                        .w(px(60.0))
                        .py(px(1.5))
                        .bg(act_bg)
                        .border_1()
                        .border_color(act_col)
                        .rounded_sm()
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(act_col)
                        .child(rule.action.label()),
                ),
        )
        // 3. Direction
        .child(
            div()
                .w(px(60.0))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_SECONDARY)
                .child(rule.direction.label()),
        )
        // 4. Port / Proto
        .child(
            div()
                .w(px(110.0))
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hex_rgb(0x38bdf8))
                        .child(rule.display_port_proto()),
                )
                .children(if rule.is_ipv6 {
                    Some(
                        div()
                            .px(px(3.0))
                            .py(px(1.0))
                            .bg(BG_CONTROL)
                            .rounded_xs()
                            .font_family(FONT_MONO)
                            .text_size(px(8.0))
                            .text_color(TEXT_FAINT)
                            .child("v6")
                    )
                } else {
                    None
                }),
        )
        // 5. Source
        .child(
            div()
                .w(px(160.0))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(if rule.source.contains("Anywhere") { TEXT_SECONDARY } else { hex_rgb(0xfacc15) })
                .child(rule.source.clone()),
        )
        // 6. Destination
        .child(
            div()
                .w(px(120.0))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_MUTED)
                .child(rule.destination.clone()),
        )
        // 7. Comment
        .child(
            div()
                .flex_1()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(if rule.comment.is_some() { TEXT_PRIMARY } else { TEXT_FAINTER })
                .child(rule.comment.as_deref().unwrap_or("—").to_string()),
        )
        // 8. Actions (Delete rule)
        .child(
            div()
                .w(px(60.0))
                .child(
                    div()
                        .id(ElementId::NamedInteger(format!("del-rule-{}", rule.id).into(), rule.number as u64))
                        .p(px(3.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(CRIT_BG).text_color(CRIT))
                        .text_color(TEXT_MUTED)
                        .on_click(move |_ev, _window, cx| {
                            let target = r_id.clone();
                            app_del.update(cx, |this, cx| {
                                this.delete_firewall_rule(&target, cx);
                            });
                        })
                        .child(inherited_icon(TablerIcon::Trash, px(12.0))),
                ),
        )
}
