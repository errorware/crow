use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;
use crate::theme::*;
use crate::app::firewall::{FirewallRuleInputs, RuleField};
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::{FirewallBackend, RuleAction, RuleDirection, RuleProtocol};

#[derive(Clone, Debug)]
pub struct NewRuleState {
    pub action: RuleAction,
    pub direction: RuleDirection,
    pub protocol: RuleProtocol,
    pub port_input: String,
    pub source_input: String,
    pub comment_input: String,
    pub is_anywhere: bool,
}

impl Default for NewRuleState {
    fn default() -> Self {
        Self {
            action: RuleAction::Allow,
            direction: RuleDirection::Inbound,
            protocol: RuleProtocol::Tcp,
            port_input: "8080".to_string(),
            source_input: "Anywhere".to_string(),
            comment_input: String::new(),
            is_anywhere: true,
        }
    }
}

impl NewRuleState {
    pub fn generate_ufw_command(&self) -> String {
        let action_str = match self.action {
            RuleAction::Allow => "allow",
            RuleAction::Deny => "deny",
            RuleAction::Reject => "reject",
            RuleAction::Limit => "limit",
        };

        let proto_str = match self.protocol {
            RuleProtocol::Tcp => "proto tcp ",
            RuleProtocol::Udp => "proto udp ",
            _ => "",
        };

        let from_str = if self.is_anywhere || self.source_input.trim().is_empty() {
            "".to_string()
        } else {
            format!("from {} ", self.source_input.trim())
        };

        let port_clean = self.port_input.trim();
        let port_str = if port_clean.is_empty() {
            "any".to_string()
        } else {
            format!("to any port {}", port_clean)
        };

        let comment_str = if self.comment_input.trim().is_empty() {
            "".to_string()
        } else {
            format!(" comment '{}'", self.comment_input.trim())
        };

        format!("ufw {} {}{}{}{}", action_str, proto_str, from_str, port_str, comment_str)
    }
}

/// A choice in one of the dialog's segmented rows: neutral until picked,
/// then outlined; `accent` colors the picked one's text (an action's color).
fn segment(id: String, label: &'static str, selected: bool, accent: Rgba) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .flex_1()
        .py(px(5.0))
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(if selected { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
        .bg(if selected { BG_CONTROL_ALT } else { hex_rgba(0, 0.0) })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if selected { accent } else { TEXT_DIM })
        .child(label)
}

fn field_label(text: &'static str) -> Div {
    div().font_family(FONT_MONO).text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(text)
}

/// A preset that fills a field.
fn preset(id: String, label: &'static str) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .px(px(7.0))
        .py(px(3.0))
        .border_1()
        .border_color(BORDER_DEFAULT)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .text_color(TEXT_DIM)
        .child(label)
}

pub fn new_rule_modal(state: &NewRuleState, backend: FirewallBackend, inputs: Option<&FirewallRuleInputs>, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_close, app_backdrop, app_cancel, app_submit) = (app.clone(), app.clone(), app.clone(), app.clone());

    // What will run: ufw's one directive, or firewall-cmd's pairs (ERR-77).
    let (preview_title, cmd_preview, preview_ok): (&str, Vec<String>, bool) = if backend == FirewallBackend::Firewalld {
        match super::commands::firewalld_new_rule(state) {
            Ok(cmds) => ("WILL RUN (running + saved config)", cmds.iter().map(super::commands::describe).collect(), true),
            Err(e) => ("NOT VALID FOR FIREWALLD", vec![e], false),
        }
    } else {
        ("WILL RUN", vec![state.generate_ufw_command()], true)
    };

    let input = |i: Option<&Entity<InputState>>| div().flex_1().min_w(px(0.0)).children(i.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.5)).bg(BG_APP)));

    div()
        .id("new-rule-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x060709, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| app_backdrop.update(cx, |this, cx| this.close_new_firewall_rule_modal(cx)))
        .child(
            div()
                .id("new-rule-modal-panel")
                .w(px(520.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .flex()
                .flex_col()
                .occlude()
                // Clicks inside stay inside (the backdrop closes it).
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .on_click(|_ev, _window, cx| cx.stop_propagation())
                .child(
                    div()
                        .h(px(40.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(14.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .child(div().text_size(px(11.0)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_PRIMARY).child("ADD RULE"))
                        .child(div().text_size(px(10.5)).text_color(TEXT_DIM).child(backend.short_name().to_lowercase()))
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("btn-close-new-rule")
                                .p(px(3.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| app_close.update(cx, |this, cx| this.close_new_firewall_rule_modal(cx)))
                                .child(tabler_icon(TablerIcon::X).size(px(13.0)).text_color(TEXT_DIM)),
                        ),
                )
                .child(
                    div()
                        .p(px(14.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div().flex().flex_col().gap(px(5.0)).child(field_label("ACTION")).child(div().flex().gap(px(4.0)).children(
                                [(RuleAction::Allow, "ALLOW", OK), (RuleAction::Deny, "DENY", CRIT), (RuleAction::Reject, "REJECT", CRIT), (RuleAction::Limit, "LIMIT", WARN)].into_iter().map(|(a, label, color)| {
                                    let app = app.clone();
                                    segment(format!("sel-action-{label}"), label, state.action == a, color).on_click(move |_ev, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.firewall.new_rule.action = a;
                                            cx.notify();
                                        })
                                    })
                                }),
                            )),
                        )
                        .child(
                            div().flex().flex_col().gap(px(5.0)).child(field_label("PROTOCOL")).child(div().flex().gap(px(4.0)).children(
                                [(RuleProtocol::Tcp, "TCP"), (RuleProtocol::Udp, "UDP"), (RuleProtocol::Both, "TCP+UDP"), (RuleProtocol::Any, "ANY")].into_iter().map(|(p, label)| {
                                    let app = app.clone();
                                    segment(format!("sel-proto-{label}"), label, state.protocol == p, TEXT_MAX).on_click(move |_ev, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.firewall.new_rule.protocol = p;
                                            cx.notify();
                                        })
                                    })
                                }),
                            )),
                        )
                        .child(
                            div().flex().flex_col().gap(px(5.0)).child(field_label("PORT")).child(
                                div().flex().items_center().gap(px(4.0)).child(input(inputs.map(|i| &i.port))).children(["22", "80", "443", "5432", "6379"].into_iter().map(|p| {
                                    let app = app.clone();
                                    preset(format!("preset-port-{p}"), p).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.set_firewall_rule_field(RuleField::Port, p, window, cx)))
                                })),
                            ),
                        )
                        .child(
                            div().flex().flex_col().gap(px(5.0)).child(field_label("FROM")).child(
                                div().flex().items_center().gap(px(4.0)).child(input(inputs.map(|i| &i.source))).children([("anywhere", ""), ("10.0.0.0/8", "10.0.0.0/8"), ("192.168.0.0/16", "192.168.0.0/16")].into_iter().map(|(label, value)| {
                                    let app = app.clone();
                                    preset(format!("preset-src-{label}"), label).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.set_firewall_rule_field(RuleField::Source, value, window, cx)))
                                })),
                            ),
                        )
                        .child(div().flex().flex_col().gap(px(5.0)).child(field_label("COMMENT")).child(input(inputs.map(|i| &i.comment))))
                        .child(
                            div()
                                .p(px(10.0))
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .font_family(FONT_MONO)
                                .child(div().text_size(px(9.0)).font_weight(FontWeight::SEMIBOLD).text_color(if preview_ok { TEXT_DIMMER } else { CRIT }).child(preview_title))
                                .children(cmd_preview.into_iter().map(|line| div().text_size(px(11.0)).text_color(if preview_ok { TEXT_SECONDARY } else { CRIT_INK }).child(line))),
                        ),
                )
                .child(
                    div()
                        .h(px(44.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(14.0))
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .child(div().flex_1().text_size(px(9.5)).text_color(TEXT_FAINT).child("Enter adds it · Esc or a click outside cancels"))
                        .child(
                            div()
                                .id("btn-cancel-new-rule")
                                .px(px(9.0))
                                .py(px(4.0))
                                .cursor_pointer()
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .hover(|s| s.text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _window, cx| app_cancel.update(cx, |this, cx| this.close_new_firewall_rule_modal(cx)))
                                .child("CANCEL"),
                        )
                        .child(
                            div()
                                .id("btn-submit-new-rule")
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(OK)
                                .cursor_pointer()
                                .hover(|s| s.bg(OK_BG))
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK)
                                .on_click(move |_ev, _window, cx| app_submit.update(cx, |this, cx| this.submit_new_firewall_rule(cx)))
                                .child("ADD RULE"),
                        ),
                ),
        )
}
