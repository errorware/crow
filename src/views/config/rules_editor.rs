use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

pub struct HbaRuleDef {
    pub num: &'static str,
    pub rule_type: &'static str,
    pub database: &'static str,
    pub user: &'static str,
    pub address: &'static str,
    pub addr_note: &'static str,
    pub method: &'static str,
    pub risk: &'static str,
    pub risk_color: Rgba,
    pub is_expanded: bool,
}

pub fn default_hba_rules() -> Vec<HbaRuleDef> {
    vec![
        HbaRuleDef { num: "01", rule_type: "local", database: "all", user: "postgres", address: "—", addr_note: "unix socket", method: "peer", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "02", rule_type: "local", database: "all", user: "all", address: "—", addr_note: "unix socket", method: "peer", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "03", rule_type: "host", database: "all", user: "all", address: "127.0.0.1/32", addr_note: "loopback", method: "scram-sha-256", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "04", rule_type: "host", database: "all", user: "all", address: "::1/128", addr_note: "loopback v6", method: "scram-sha-256", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "05", rule_type: "host", database: "acme_prod", user: "acme_app", address: "10.0.4.19/32", addr_note: "api-01", method: "scram-sha-256", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "06", rule_type: "host", database: "acme_prod", user: "acme_app", address: "10.0.4.22/32", addr_note: "api-02", method: "scram-sha-256", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "07", rule_type: "host", database: "acme_prod", user: "analyst", address: "10.0.9.0/24", addr_note: "vpn pool", method: "ldap", risk: "REVIEW", risk_color: WARN, is_expanded: false },
        HbaRuleDef { num: "08", rule_type: "host", database: "all", user: "all", address: "0.0.0.0/0", addr_note: "↯ internet", method: "md5", risk: "CRITICAL", risk_color: CRIT, is_expanded: false },
        HbaRuleDef { num: "09", rule_type: "host", database: "all", user: "all", address: "10.0.4.0/24", addr_note: "private", method: "scram-sha-256", risk: "EDITED", risk_color: WARN, is_expanded: true },
        HbaRuleDef { num: "10", rule_type: "host", database: "replication", user: "repl", address: "10.0.4.11/32", addr_note: "standby", method: "scram-sha-256", risk: "EDITED", risk_color: WARN, is_expanded: false },
        HbaRuleDef { num: "11", rule_type: "hostssl", database: "metrics", user: "prom", address: "10.0.4.31/32", addr_note: "prometheus", method: "cert", risk: "OK", risk_color: OK, is_expanded: false },
        HbaRuleDef { num: "12", rule_type: "host", database: "template1", user: "all", address: "10.0.4.0/24", addr_note: "private", method: "reject", risk: "OK", risk_color: OK, is_expanded: false },
    ]
}

pub struct MethodOptionDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub tag: &'static str,
    pub color: Rgba,
}

pub fn default_methods() -> &'static [MethodOptionDef] {
    &[
        MethodOptionDef { name: "scram-sha-256", desc: "Salted challenge-response, password never in cleartext", tag: "RECOMMENDED", color: OK },
        MethodOptionDef { name: "cert", desc: "TLS client certificate, no password at all", tag: "STRONGEST", color: OK },
        MethodOptionDef { name: "ldap", desc: "Delegates to directory server — adds a network dependency", tag: "CAUTION", color: WARN },
        MethodOptionDef { name: "md5", desc: "Deprecated hash, replayable on the wire", tag: "WEAK", color: WARN },
        MethodOptionDef { name: "trust", desc: "Any connection accepted with no password whatsoever", tag: "NEVER ON PROD", color: CRIT },
        MethodOptionDef { name: "reject", desc: "Refuse and stop evaluating further rules", tag: "DENY", color: TEXT_MUTED },
    ]
}

pub fn rules_editor(rules: &[HbaRuleDef], app: Entity<CrowApp>) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // File Header
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
                        .font_family("JetBrains Mono")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(TEXT_MAX)
                        .child("pg_hba.conf"),
                )
                .child(
                    div()
                        .bg(OK_BG)
                        .text_color(OK)
                        .font_family("JetBrains Mono")
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child("PARSED 12/12 RULES"),
                )
                .child(
                    div()
                        .bg(WARN_BG)
                        .text_color(WARN)
                        .font_family("JetBrains Mono")
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child("2 UNAPPLIED EDITS"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .px(px(9.0))
                        .py(px(4.0))
                        .child("RAW TEXT ")
                        .child(div().text_color(TEXT_DIMMER).child("⌘/")),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .px(px(9.0))
                        .py(px(4.0))
                        .child("HISTORY · 31"),
                ),
        )
        // Order Warning
        .child(
            div()
                .h(px(28.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_SUBHEAD)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(WARN)
                        .child("▲"),
                )
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .child("First match wins — rule order is evaluated top to bottom. Drag to reorder."),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("⇧↑ / ⇧↓ move rule"),
                ),
        )
        // Column Header
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .bg(BG_SUBHEAD)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family("Inter")
                .text_size(px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(div().w(px(22.0)))
                .child(div().w(px(26.0)).child("#"))
                .child(div().w(px(82.0)).child("TYPE"))
                .child(div().w(px(132.0)).child("DATABASE"))
                .child(div().w(px(128.0)).child("USER"))
                .child(div().flex_1().min_w(px(0.0)).child("ADDRESS"))
                .child(div().w(px(128.0)).child("METHOD"))
                .child(div().w(px(84.0)).text_right().pr(px(12.0)).child("RISK")),
        )
        // Rules rows
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .overflow_hidden()
                .flex()
                .flex_col()
                .children(rules.iter().enumerate().map(|(idx, r)| {
                    let is_open = r.is_expanded;
                    let is_critical = r.risk == "CRITICAL";
                    let rule_num = r.num;
                    let app_expand = app.clone();

                    let left_edge = if is_open {
                        TEXT_PRIMARY
                    } else if is_critical {
                        CRIT
                    } else {
                        hex_rgba(0, 0.0)
                    };

                    let row_bg = if is_open {
                        BG_ROW_SELECTED
                    } else if idx % 2 == 1 {
                        BG_ROW_ALT
                    } else {
                        hex_rgba(0, 0.0)
                    };

                    let (method_bg, method_border, method_fg) = if is_critical {
                        (CRIT_BG, BORDER_DANGER_SEL, CRIT_INK)
                    } else if is_open {
                        (BG_CONTROL_ALT, BORDER_CONTROL_SEL, TEXT_MAX)
                    } else {
                        (BG_CONTROL, BORDER_PANEL, TEXT_SECONDARY)
                    };

                    let risk_bg = match r.risk {
                        "OK" => OK_BG,
                        "REVIEW" | "EDITED" => WARN_BG,
                        _ => CRIT_BG,
                    };

                    div()
                        .id(ElementId::NamedInteger("rule-row".into(), idx as u64))
                        .w_full()
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .child(
                            div()
                                .id(ElementId::NamedInteger("rule-row-inner".into(), idx as u64))
                                .flex()
                                .items_center()
                                .h(px(30.0))
                                .bg(row_bg)
                                .border_l_2()
                                .border_color(left_edge)
                                .font_family("JetBrains Mono")
                                .text_size(px(11.5))
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_expand.update(cx, |this, cx| {
                                        this.toggle_rule_expand(rule_num, cx);
                                    });
                                })
                                // Drag
                                .child(
                                    div()
                                        .w(px(22.0))
                                        .text_center()
                                        .text_size(px(10.0))
                                        .text_color(TEXT_GHOST)
                                        .child("⠿"),
                                )
                                // Line #
                                .child(
                                    div()
                                        .w(px(26.0))
                                        .text_size(px(10.0))
                                        .text_color(TEXT_FAINTER)
                                        .child(r.num),
                                )
                                // Type
                                .child(
                                    div()
                                        .w(px(82.0))
                                        .child(
                                            div()
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(if is_open { BORDER_STRONG } else { BORDER_PANEL })
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .text_color(TEXT_SECONDARY)
                                                .child(r.rule_type),
                                        ),
                                )
                                // Database
                                .child(
                                    div()
                                        .w(px(132.0))
                                        .text_color(if r.database == "all" { TEXT_MUTED } else { TEXT_SECONDARY })
                                        .child(r.database),
                                )
                                // User
                                .child(
                                    div()
                                        .w(px(128.0))
                                        .text_color(if r.user == "all" { TEXT_MUTED } else { TEXT_SECONDARY })
                                        .child(r.user),
                                )
                                // Address + Note
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .text_color(TEXT_SECONDARY)
                                        .child(
                                            div()
                                                .child(r.address),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(9.5))
                                                .text_color(TEXT_FAINT)
                                                .child(r.addr_note),
                                        ),
                                )
                                // Method
                                .child(
                                    div()
                                        .w(px(128.0))
                                        .child(
                                            div()
                                                .bg(method_bg)
                                                .border_1()
                                                .border_color(method_border)
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .text_color(method_fg)
                                                .child(format!("{} ▾", r.method)),
                                        ),
                                )
                                // Risk
                                .child(
                                    div()
                                        .w(px(84.0))
                                        .flex()
                                        .justify_end()
                                        .pr(px(12.0))
                                        .child(
                                            div()
                                                .bg(risk_bg)
                                                .text_color(r.risk_color)
                                                .text_size(px(8.5))
                                                .font_weight(FontWeight::BOLD)
                                                .px(px(5.0))
                                                .py(px(2.0))
                                                .child(r.risk),
                                        ),
                                ),
                        )
                        // Expanded Rule Inspector for line 9
                        .children(if is_open {
                            let current_method = r.method;
                            Some(
                                div()
                                    .bg(hex_rgb(0x0e0f13))
                                    .border_l_2()
                                    .border_color(TEXT_PRIMARY)
                                    .border_t_1()
                                    .border_color(BORDER_PANEL)
                                    .flex()
                                    // Left: Method Picker
                                    .child(
                                        div()
                                            .flex_1()
                                            .p(px(10.0))
                                            .pl(px(22.0))
                                            .pb(px(12.0))
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .font_family("Inter")
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(TEXT_DIMMER)
                                                    .child(format!("AUTH METHOD — LINE {}", r.num)),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .children(default_methods().iter().enumerate().map(|(m_idx, m)| {
                                                        let is_selected = m.name == current_method;
                                                        let m_name = m.name;
                                                        let app_method = app.clone();
                                                        let tag_bg = match m.tag {
                                                            "RECOMMENDED" | "STRONGEST" => OK_BG,
                                                            "CAUTION" | "WEAK" => WARN_BG,
                                                            "NEVER ON PROD" => CRIT_BG,
                                                            _ => BG_CHIP,
                                                        };

                                                        div()
                                                            .id(ElementId::NamedInteger("method-opt".into(), m_idx as u64))
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(10.0))
                                                            .h(px(26.0))
                                                            .px(px(8.0))
                                                            .border_l_2()
                                                            .border_color(if is_selected { OK } else { hex_rgba(0, 0.0) })
                                                            .bg(if is_selected { BG_CONTROL_ALT } else { hex_rgba(0, 0.0) })
                                                            .cursor_pointer()
                                                            .on_click(move |_ev, _window, cx| {
                                                                app_method.update(cx, |this, cx| {
                                                                    this.set_rule_method(rule_num, m_name, cx);
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .w(px(110.0))
                                                                    .flex_none()
                                                                    .font_family("JetBrains Mono")
                                                                    .text_size(px(11.0))
                                                                    .text_color(if is_selected { TEXT_MAX } else { TEXT_SECONDARY })
                                                                    .child(m.name),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex_1()
                                                                    .min_w(px(0.0))
                                                                    .font_family("Inter")
                                                                    .text_size(px(10.5))
                                                                    .text_color(TEXT_DIM)
                                                                    .child(m.desc),
                                                            )
                                                            .child(
                                                                div()
                                                                    .bg(tag_bg)
                                                                    .text_color(m.color)
                                                                    .font_family("JetBrains Mono")
                                                                    .text_size(px(8.5))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .px(px(4.0))
                                                                    .py(px(1.5))
                                                                    .flex_none()
                                                                    .child(m.tag),
                                                            )
                                                    })),
                                            ),
                                    )
                                    // Right: Contextual Documentation
                                    .child(
                                        div()
                                            .w(px(250.0))
                                            .flex_none()
                                            .border_l_1()
                                            .border_color(BORDER_PANEL)
                                            .p(px(10.0))
                                            .px(px(12.0))
                                            .flex()
                                            .flex_col()
                                            .gap(px(7.0))
                                            .child(
                                                div()
                                                    .font_family("Inter")
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(TEXT_DIMMER)
                                                    .child("MAN PAGE · pg_hba"),
                                            )
                                            .child(
                                                div()
                                                    .font_family("Inter")
                                                    .text_size(px(10.5))
                                                    .text_color(TEXT_TERTIARY)
                                                    .child("scram-sha-256 performs SCRAM-SHA-256 authentication to verify the user's password, without transmitting the cleartext password."),
                                            )
                                            .child(
                                                div()
                                                    .font_family("JetBrains Mono")
                                                    .text_size(px(10.0))
                                                    .text_color(TEXT_FAINT)
                                                    .child("requires password_encryption = scram-sha-256"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(6.0))
                                                    .mt(px(2.0))
                                                    .child(
                                                        div()
                                                            .size(px(6.0))
                                                            .rounded_full()
                                                            .bg(OK)
                                                            .flex_none(),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family("JetBrains Mono")
                                                            .text_size(px(10.0))
                                                            .text_color(OK)
                                                            .child("6/6 roles compatible"),
                                                    ),
                                            ),
                                    )
                            )
                        } else {
                            None
                        })
                })),
        )
        // Add rule affordance
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .pl(px(22.0))
                .pr(px(12.0))
                .text_color(TEXT_FAINT)
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .child("+"),
                )
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(11.0))
                        .child("Add rule"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .bg(BG_KEY)
                        .border_1()
                        .border_color(BORDER_KEY)
                        .px(px(5.0))
                        .py(px(1.0))
                        .child("⌘N"),
                )
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(11.0))
                        .text_color(TEXT_FAINTER)
                        .child("·  from template: app user over TLS, replication peer, read-only analyst"),
                ),
        )
}
