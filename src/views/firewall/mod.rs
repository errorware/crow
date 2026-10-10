pub mod models;
pub mod detector;
pub mod firewalld;
pub mod raw;
pub mod lockout;
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
pub use state::{FirewallState, LockoutConfirm};

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use std::collections::HashMap;
use crate::app::CrowApp;
use crate::config::ConfigFileState;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::components::table_controls::{render_table_controls, Chip};
use gpui_kit::component::input::InputState;

pub fn firewall_view(
    app: Entity<CrowApp>,
    fw: &FirewallState,
    config_file_states: &HashMap<String, ConfigFileState>,
    search: Option<&Entity<InputState>>,
    rule_inputs: Option<&crate::app::firewall::FirewallRuleInputs>,
) -> AnyElement {
    let state = &fw.status;

    // Check operational status
    match state {
        FirewallOperationalState::Active(summary) if summary.is_active => {
            render_active_firewall(summary, app, fw, config_file_states, search, rule_inputs).into_any_element()
        }
        FirewallOperationalState::Inactive { backend, reason, detected_binaries, .. } => {
            non_operational_view(*backend, reason, detected_binaries, app).into_any_element()
        }
        FirewallOperationalState::Unmanaged { detected_binaries, reason } => {
            non_operational_view(FirewallBackend::NoneDetected, reason, detected_binaries, app).into_any_element()
        }
        FirewallOperationalState::Active(summary) => {
            // Active backend struct but is_active == false
            non_operational_view(summary.backend, t!("firewall-firewall-is-currently-disabled-by"), &["ufw".into()], app).into_any_element()
        }
    }
}

/// A quiet header button: text, a hairline border, brighter on hover.
fn tool_button(id: &'static str, label: impl Into<SharedString>, color: Rgba) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(9.0))
        .py(px(3.5))
        .border_1()
        .border_color(BORDER_DEFAULT)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .text_color(color)
        .child(label.into())
}

/// How an action reads in the table: its word, in the color that says it.
fn action_color(action: RuleAction) -> Rgba {
    match action {
        RuleAction::Allow => OK,
        RuleAction::Deny | RuleAction::Reject => CRIT,
        RuleAction::Limit => WARN,
    }
}

fn render_active_firewall(
    summary: &FirewallStatusSummary,
    app: Entity<CrowApp>,
    fw: &FirewallState,
    config_file_states: &HashMap<String, ConfigFileState>,
    search: Option<&Entity<InputState>>,
    rule_inputs: Option<&crate::app::firewall::FirewallRuleInputs>,
) -> impl IntoElement {
    // Crow changes ufw and firewalld (ERR-77); raw nftables/iptables are
    // shown read-only.
    let writable = matches!(summary.backend, FirewallBackend::Ufw | FirewallBackend::Firewalld);
    let is_ufw = summary.backend == FirewallBackend::Ufw;
    let show_raw = fw.show_raw && !writable;

    let file_state = config_file_states.get("user.rules");
    let (is_modified, add_count, del_count) = match file_state {
        Some(st) => {
            let (a, d) = st.diff_stats();
            (st.is_modified(), a, d)
        }
        None => (false, 0, 0),
    };

    let query = fw.search_query.to_lowercase();
    let filtered_rules: Vec<&FirewallRule> = summary
        .rules
        .iter()
        .filter(|r| fw.action_filter.is_none_or(|a| r.action == a))
        .filter(|r| {
            query.is_empty()
                || r.port.to_lowercase().contains(&query)
                || r.source.to_lowercase().contains(&query)
                || r.destination.to_lowercase().contains(&query)
                || r.comment.as_deref().unwrap_or("").to_lowercase().contains(&query)
                || r.action.label().to_lowercase().contains(&query)
        })
        .collect();
    let total_rules = summary.rules.len();

    let chips: Vec<Chip> = [(None, t!("firewall-all")), (Some(RuleAction::Allow), t!("firewall-allow")), (Some(RuleAction::Deny), t!("firewall-deny")), (Some(RuleAction::Limit), t!("firewall-limit"))]
        .into_iter()
        .map(|(act, label)| {
            let app = app.clone();
            Chip {
                id: format!("fw-filter-{label}"),
                label,
                count: summary.rules.iter().filter(|r| act.is_none_or(|a| r.action == a)).count(),
                is_on: fw.action_filter == act,
                alarming: false,
                on_click: Box::new(move |cx| app.update(cx, |this, cx| this.set_firewall_action_filter(act, cx))),
            }
        })
        .collect();

    // Header: title and state, search and filters, then the tools. Only the
    // primary action (+ ADD RULE) and a pending stage carry color.
    let header = {
        let (app_stage, app_audit, app_raw, app_reload, app_cfg, app_toggle, app_new) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone(), app.clone(), app.clone());
        div()
            .h(px(40.0))
            .flex_none()
            .flex()
            .items_center()
            .bg(BG_PANEL)
            .border_b_1()
            .border_color(BORDER_PANEL)
            .child(
                div()
                    .px(px(14.0))
                    .h_full()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .border_r_1()
                    .border_color(BORDER_PANEL)
                    .font_family(FONT_MONO)
                    .child(div().text_size(px(11.0)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_PRIMARY).child(t!("firewall-firewall")))
                    .child(div().flex().items_center().gap(px(6.0)).child(div().size(px(6.0)).rounded_full().bg(OK)).child(
                        div().text_size(px(10.5)).text_color(TEXT_DIM).child(t!("firewall-on", short_name = summary.backend.short_name().to_lowercase(), total_rules = total_rules)),
                    )),
            )
            .child(render_table_controls(search, chips))
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(12.0))
                    .when(is_modified, |d| {
                        d.child(tool_button("btn-stage-firewall-rules", t!("firewall-stage", add_count = add_count, del_count = del_count), WARN).border_color(WARN).on_click(move |_ev, _window, cx| {
                            app_stage.update(cx, |this, cx| this.stage_firewall_rules(t!("firewall-staged-firewall-rule-changes"), cx));
                        }))
                    })
                    .child(
                        tool_button("btn-toggle-firewall-audit-rail", t!("firewall-audit"), if fw.show_audit_rail { TEXT_PRIMARY } else { TEXT_DIM })
                            .when(fw.show_audit_rail, |d| d.bg(BG_CONTROL).border_color(BORDER_STRONG))
                            .on_click(move |_ev, _window, cx| app_audit.update(cx, |this, cx| this.toggle_firewall_audit_rail(cx))),
                    )
                    .when(!writable, |d| {
                        d.child(tool_button("btn-firewall-raw", if show_raw { t!("firewall-rule-table") } else { t!("firewall-raw-ruleset") }, TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                            app_raw.update(cx, |this, cx| this.toggle_firewall_raw(cx));
                        }))
                    })
                    .when(writable, |d| {
                        d.child(tool_button("btn-reload-firewall", t!("firewall-reload"), TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                            app_reload.update(cx, |this, cx| this.reload_firewall(cx));
                        }))
                    })
                    .when(is_ufw, |d| {
                        d.child(tool_button("btn-firewall-open-config", "user.rules", TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                            app_cfg.update(cx, |this, cx| {
                                this.select_managed_file("user.rules", cx);
                                this.set_view("config", cx);
                            });
                        }))
                    })
                    .when(is_ufw, |d| {
                        d.child(
                            tool_button("btn-toggle-firewall-active", t!("firewall-disable"), CRIT_INK_DIM)
                                .hover(|s| s.bg(CRIT_ROW_BG).text_color(CRIT).border_color(BORDER_DANGER_BTN))
                                .on_click(move |_ev, _window, cx| app_toggle.update(cx, |this, cx| this.toggle_firewall_active(cx))),
                        )
                    })
                    .when(writable, |d| {
                        d.child(
                            tool_button("btn-open-new-rule-modal", t!("firewall-add-rule"), OK)
                                .border_color(OK)
                                .font_weight(FontWeight::BOLD)
                                .hover(|s| s.bg(OK_BG).text_color(OK))
                                .on_click(move |_ev, _window, cx| app_new.update(cx, |this, cx| this.open_new_firewall_rule_modal(cx))),
                        )
                    }),
            )
    };

    // Defaults and quick ports on one quiet line. An incoming policy that
    // lets everything in is the one thing here worth a color.
    let policy = |label: &'static str, value: String, warn: bool| {
        div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .child(div().text_color(TEXT_FAINT).child(label))
            .child(div().text_color(if warn { WARN } else { TEXT_SECONDARY }).child(value.to_lowercase()))
    };
    let incoming_open = summary.default_incoming == RuleAction::Allow;
    let defaults_line = div()
        .flex_none()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_x(px(16.0))
        .gap_y(px(6.0))
        .px(px(14.0))
        .py(px(7.0))
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(div().text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(t!("firewall-defaults")))
        .child(policy("incoming", summary.default_incoming.label().to_string(), incoming_open))
        .child(policy("outgoing", summary.default_outgoing.label().to_string(), false))
        .child(policy("routed", summary.default_forward.label().to_string(), false))
        .when(writable, |d| {
            d.child(div().w(px(1.0)).h(px(14.0)).bg(BORDER_DEFAULT)).child(div().text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(t!("firewall-quick-open"))).children(
                common_quick_ports().iter().map(|qp| {
                    let open = summary.rules.iter().any(|r| r.action == RuleAction::Allow && r.port == qp.port.to_string());
                    let (port, proto, name, app) = (qp.port, qp.protocol, qp.name, app.clone());
                    div()
                        .id(ElementId::NamedInteger(format!("qp-toggle-{port}").into(), 0))
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .px(px(6.0))
                        .py(px(1.5))
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.toggle_quick_port(port, proto, name, cx)))
                        .child(div().size(px(6.0)).rounded_full().when(open, |d| d.bg(OK)).when(!open, |d| d.border_1().border_color(TEXT_FAINT)))
                        .child(div().text_color(if open { TEXT_PRIMARY } else { TEXT_DIM }).child(qp.name))
                        .child(div().text_color(TEXT_FAINT).child(format!(":{port}")))
                }),
            )
        });

    let column_header = div()
        .h(px(28.0))
        .flex_none()
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(14.0))
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(TEXT_DIMMER)
        .child(div().w(px(28.0)).flex_none().child("#"))
        .child(div().w(px(64.0)).flex_none().child(t!("firewall-action")))
        .child(div().w(px(44.0)).flex_none().child(t!("firewall-dir")))
        .child(div().w(px(120.0)).flex_none().child(t!("firewall-port")))
        .child(div().flex_grow(1.0).flex_basis(px(0.0)).min_w(px(130.0)).child(t!("firewall-from")))
        .child(div().flex_grow(1.0).flex_basis(px(0.0)).min_w(px(110.0)).child(t!("firewall-to")))
        .child(div().flex_grow(1.5).flex_basis(px(0.0)).min_w(px(120.0)).child(t!("firewall-comment")))
        .child(div().w(px(24.0)).flex_none());

    let body: Vec<AnyElement> = if show_raw {
        vec![div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .children(summary.raw_output.lines().map(|line| div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_SECONDARY).child(if line.is_empty() { " ".to_string() } else { line.to_string() })))
            .into_any_element()]
    } else if filtered_rules.is_empty() {
        let text = if total_rules == 0 { t!("firewall-no-rules-yet-everything-follows") } else { t!("firewall-no-rules-match-try-all") };
        vec![div().px(px(14.0)).py(px(18.0)).font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_FAINT).child(text).into_any_element()]
    } else {
        filtered_rules.iter().map(|rule| render_rule_row(rule, writable, app.clone()).into_any_element()).collect()
    };

    let modal_element = fw.show_new_rule_modal.then(|| new_rule_modal(&fw.new_rule, summary.backend, rule_inputs, app.clone()).into_any_element());
    let toast = fw.pending.as_ref().map(|p| t!("firewall-running", p = p)).or_else(|| fw.toast.clone());

    div()
        .id("firewall-management-view")
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .child(header)
        // Read-only backends, and anything the reader flagged (ERR-76).
        .children((!writable).then(|| notice_strip(match summary.backend {
            FirewallBackend::Nftables | FirewallBackend::Iptables => t!("firewall-read-only-crow-reads-the", label = summary.backend.label()),
            b => t!("firewall-read-only-crow-reads-the-2", label = b.label()),
        }, TEXT_SECONDARY)))
        .children(summary.notice.clone().map(|n| notice_strip(n, WARN)))
        .children((summary.backend == FirewallBackend::Firewalld).then(|| notice_strip(
            t!("firewall-changes-go-to-firewallds-running").to_string(),
            TEXT_DIM,
        )))
        // A change the lock-out guard held back (ERR-77).
        .children(fw.lockout.as_ref().map(|c| lockout_strip(c, app.clone())))
        .child(defaults_line)
        .when(!show_raw, |d| d.child(column_header))
        .child(div().id("firewall-rules-table").flex_1().min_h(px(0.0)).overflow_y_scrollbar().flex().flex_col().children(body))
        .children(toast.map(|msg| {
            div()
                .absolute()
                .bottom(px(16.0))
                .right(px(16.0))
                .px(px(14.0))
                .py(px(8.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .shadow_lg()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(tabler_icon(TablerIcon::Check).size(px(14.0)).text_color(OK))
                .child(div().font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_PRIMARY).child(msg))
        }))
        .children(modal_element)
}

fn render_rule_row(rule: &FirewallRule, writable: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let r_id = rule.id.clone();
    let anywhere = |s: &str| s.contains("Anywhere") || s.is_empty();
    div()
        .id(ElementId::NamedInteger(format!("rule-row-{}", rule.id).into(), rule.number as u64))
        .group("fw-rule")
        .flex()
        .items_center()
        .gap(px(12.0))
        .px(px(14.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .text_size(px(11.0))
        .child(div().w(px(28.0)).flex_none().text_color(TEXT_FAINT).child(rule.number.to_string()))
        .child(div().w(px(64.0)).flex_none().font_weight(FontWeight::SEMIBOLD).text_color(action_color(rule.action)).child(rule.action.label().to_string()))
        .child(div().w(px(44.0)).flex_none().text_color(TEXT_DIM).child(rule.direction.label().to_lowercase()))
        .child(
            div()
                .w(px(120.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().text_color(TEXT_PRIMARY).child(rule.display_port_proto()))
                .when(rule.is_ipv6, |d| d.child(div().text_size(px(9.0)).text_color(TEXT_FAINT).child("v6"))),
        )
        .child(div().flex_grow(1.0).flex_basis(px(0.0)).min_w(px(130.0)).overflow_hidden().text_color(if anywhere(&rule.source) { TEXT_DIM } else { TEXT_PRIMARY }).child(rule.source.clone()))
        .child(div().flex_grow(1.0).flex_basis(px(0.0)).min_w(px(110.0)).overflow_hidden().text_color(if anywhere(&rule.destination) { TEXT_DIM } else { TEXT_PRIMARY }).child(rule.destination.clone()))
        .child(div().flex_grow(1.5).flex_basis(px(0.0)).min_w(px(120.0)).overflow_hidden().text_color(if rule.comment.is_some() { TEXT_SECONDARY } else { TEXT_FAINTER }).child(rule.comment.clone().unwrap_or_else(|| "—".into())))
        .child(div().w(px(24.0)).flex_none().when(writable, |d| {
            // Shown on hover, so a table of rules isn't a column of trash cans.
            d.child(
                crate::components::icon_button::icon_button(ElementId::NamedInteger(format!("del-rule-{}", rule.id).into(), rule.number as u64), TablerIcon::Trash, true)
                    .invisible()
                    .group_hover("fw-rule", |s| s.visible())
                    .on_click(move |_ev, _window, cx| {
                        let target = r_id.clone();
                        app.update(cx, |this, cx| this.delete_firewall_rule(&target, cx));
                    }),
            )
        }))
}

/// One line across the firewall screen.
/// Typed confirmation for a change that would cut off Crow's SSH port.
fn lockout_strip(confirm: &LockoutConfirm, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_ok, app_cancel) = (app.clone(), app);
    let button = |id: &'static str, label: &'static str, color: Rgba| {
        div()
            .id(id)
            .px(px(10.0))
            .py(px(4.0))
            .border_1()
            .border_color(color)
            .cursor_pointer()
            .font_family(FONT_MONO)
            .text_size(px(10.0))
            .font_weight(FontWeight::BOLD)
            .text_color(color)
            .child(label)
    };
    div()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .px(px(16.0))
        .py(px(10.0))
        .bg(CRIT_BG)
        .border_b_1()
        .border_color(CRIT)
        .child(div().font_family(FONT_MONO).text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(CRIT).child(t!("firewall-this-could-lock-you-out")))
        .child(div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_PRIMARY).child(confirm.reason.clone()))
        .children(confirm.commands.iter().map(|c| div().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_SECONDARY).child(format!("  {}", commands::describe(c)))))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_TERTIARY).child(t!("firewall-type-confirm-to-apply-it")))
                .children(confirm.input.as_ref().map(|input| div().w(px(160.0)).child(gpui_kit::component::input::Input::new(input).font_family(FONT_MONO).text_size(px(11.0)))))
                .child(button("btn-fw-lockout-confirm", t!("firewall-apply-anyway"), CRIT).on_click(move |_ev, _window, cx| {
                    app_ok.update(cx, |this, cx| this.confirm_firewall_lockout(cx));
                }))
                .child(button("btn-fw-lockout-cancel", t!("firewall-cancel"), TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    app_cancel.update(cx, |this, cx| this.cancel_firewall_lockout(cx));
                })),
        )
        .children(confirm.error.clone().map(|e| div().font_family(FONT_MONO).text_size(px(10.0)).text_color(CRIT).child(e)))
}

fn notice_strip(text: String, color: Rgba) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(16.0))
        .py(px(6.0))
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .text_color(color)
        .child(text)
}
