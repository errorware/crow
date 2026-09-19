use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::{RuleAction, RuleDirection, RuleProtocol};

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

pub fn new_rule_modal(
    state: &NewRuleState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let app_backdrop = app.clone();
    let app_submit = app.clone();

    let cmd_preview = state.generate_ufw_command();

    div()
        .id("new-rule-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x000000, 0.65))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_backdrop.update(cx, |this, cx| {
                this.close_new_firewall_rule_modal(cx);
            });
        })
        .child(
            div()
                .id("new-rule-modal-panel")
                .w(px(540.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .rounded_md()
                .flex()
                .flex_col()
                .on_click(|_ev, _window, _cx| {})
                // Modal Header
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(tabler_icon(TablerIcon::ShieldCheck).size(px(16.0)).text_color(OK))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child("ADD FIREWALL RULE"),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-close-new-rule")
                                .p(px(4.0))
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_close.update(cx, |this, cx| {
                                        this.close_new_firewall_rule_modal(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::X).size(px(14.0)).text_color(TEXT_MUTED)),
                        ),
                )
                // Modal Form Body
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        // Action Selection (ALLOW, DENY, LIMIT, REJECT)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("PACKET ACTION"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.0))
                                        .children([
                                            (RuleAction::Allow, "ALLOW", OK, OK_BG),
                                            (RuleAction::Deny, "DENY", CRIT, CRIT_BG),
                                            (RuleAction::Limit, "LIMIT", WARN, WARN_BG),
                                            (RuleAction::Reject, "REJECT", hex_rgb(0xf43f5e), hex_rgba(0xf43f5e, 0.15)),
                                        ].iter().map(|(act, label, col, bg)| {
                                            let is_sel = state.action == *act;
                                            let app_act = app.clone();
                                            let a = *act;
                                            div()
                                                .id(ElementId::NamedInteger(format!("sel-action-{:?}", a).into(), 0))
                                                .flex_1()
                                                .py(px(6.0))
                                                .rounded_sm()
                                                .border_1()
                                                .border_color(if is_sel { *col } else { BORDER_DEFAULT })
                                                .bg(if is_sel { *bg } else { BG_CONTROL })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_sel { *col } else { TEXT_SECONDARY })
                                                .on_click(move |_ev, _window, cx| {
                                                    app_act.update(cx, |this, cx| {
                                                        this.new_firewall_rule_state.action = a;
                                                        cx.notify();
                                                    });
                                                })
                                                .child(*label)
                                        })),
                                ),
                        )
                        // Protocol Selection (TCP, UDP, BOTH, ANY)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("TRANSPORT PROTOCOL"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.0))
                                        .children([
                                            (RuleProtocol::Tcp, "TCP"),
                                            (RuleProtocol::Udp, "UDP"),
                                            (RuleProtocol::Both, "TCP+UDP"),
                                            (RuleProtocol::Any, "ANY PROTOCOL"),
                                        ].iter().map(|(pr, label)| {
                                            let is_sel = state.protocol == *pr;
                                            let app_pr = app.clone();
                                            let p = *pr;
                                            div()
                                                .id(ElementId::NamedInteger(format!("sel-proto-{:?}", p).into(), 0))
                                                .flex_1()
                                                .py(px(5.0))
                                                .rounded_sm()
                                                .border_1()
                                                .border_color(if is_sel { hex_rgb(0x38bdf8) } else { BORDER_DEFAULT })
                                                .bg(if is_sel { hex_rgba(0x38bdf8, 0.15) } else { BG_CONTROL })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_sel { hex_rgb(0x38bdf8) } else { TEXT_SECONDARY })
                                                .on_click(move |_ev, _window, cx| {
                                                    app_pr.update(cx, |this, cx| {
                                                        this.new_firewall_rule_state.protocol = p;
                                                        cx.notify();
                                                    });
                                                })
                                                .child(*label)
                                        })),
                                ),
                        )
                        // Port Field with Presets
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MUTED)
                                                .child("TARGET PORT / SERVICE"),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(TEXT_FAINT)
                                                .child("Quick: 22, 80, 443, 3000, 5432, 6379"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .flex_1()
                                                .px(px(10.0))
                                                .py(px(6.0))
                                                .bg(BG_APP)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .rounded_sm()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.5))
                                                .text_color(TEXT_MAX)
                                                .child(if state.port_input.is_empty() { "e.g. 8080".to_string() } else { state.port_input.clone() }),
                                        )
                                        .children(["22", "80", "443", "5432", "6379"].iter().map(|preset| {
                                            let p_str = preset.to_string();
                                            let app_preset = app.clone();
                                            div()
                                                .id(ElementId::NamedInteger(format!("preset-port-{}", preset).into(), 0))
                                                .px(px(8.0))
                                                .py(px(5.0))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .rounded_sm()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_SECONDARY)
                                                .on_click(move |_ev, _window, cx| {
                                                    let val = p_str.clone();
                                                    app_preset.update(cx, |this, cx| {
                                                        this.new_firewall_rule_state.port_input = val;
                                                        cx.notify();
                                                    });
                                                })
                                                .child(*preset)
                                        })),
                                ),
                        )
                        // Source Address constraint
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("SOURCE ADDRESS / CIDR"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .flex_1()
                                                .px(px(10.0))
                                                .py(px(6.0))
                                                .bg(BG_APP)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .rounded_sm()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.5))
                                                .text_color(TEXT_MAX)
                                                .child(if state.source_input.is_empty() { "Anywhere".to_string() } else { state.source_input.clone() }),
                                        )
                                        .children(["Anywhere", "10.0.0.0/8", "192.168.1.0/24"].iter().map(|preset| {
                                            let s_str = preset.to_string();
                                            let app_src = app.clone();
                                            let is_any = *preset == "Anywhere";
                                            div()
                                                .id(ElementId::NamedInteger(format!("preset-src-{}", preset).into(), 0))
                                                .px(px(8.0))
                                                .py(px(5.0))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .rounded_sm()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_SECONDARY)
                                                .on_click(move |_ev, _window, cx| {
                                                    let val = s_str.clone();
                                                    app_src.update(cx, |this, cx| {
                                                        this.new_firewall_rule_state.source_input = val;
                                                        this.new_firewall_rule_state.is_anywhere = is_any;
                                                        cx.notify();
                                                    });
                                                })
                                                .child(*preset)
                                        })),
                                ),
                        )
                        // Comment Field
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MUTED)
                                        .child("RULE COMMENT / REASON"),
                                )
                                .child(
                                    div()
                                        .px(px(10.0))
                                        .py(px(6.0))
                                        .bg(BG_APP)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .rounded_sm()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .text_color(TEXT_SECONDARY)
                                        .child(if state.comment_input.is_empty() {
                                            "e.g. Microservice API Ingress".to_string()
                                        } else {
                                            state.comment_input.clone()
                                        }),
                                ),
                        )
                        // Live Command Preview Card
                        .child(
                            div()
                                .p(px(10.0))
                                .bg(hex_rgba(0x000000, 0.5))
                                .border_1()
                                .border_color(hex_rgba(0x10b981, 0.3))
                                .rounded_sm()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .child("GENERATED UFW DIRECTIVE:"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_PRIMARY)
                                        .child(cmd_preview),
                                ),
                        ),
                )
                // Modal Footer
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .bg(BG_APP)
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_FAINT)
                                .child("Rule is applied atomically to kernel netfilter tables"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .id("btn-submit-new-rule")
                                        .px(px(14.0))
                                        .py(px(5.5))
                                        .bg(OK)
                                        .rounded_sm()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(hex_rgb(0x34d399)))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x0a0a0c))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.submit_new_firewall_rule(cx);
                                            });
                                        })
                                        .child("COMMIT RULE"),
                                ),
                        ),
                ),
        )
}
