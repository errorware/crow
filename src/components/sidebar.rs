use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

pub struct NavItemDef {
    pub glyph: &'static str,
    pub label: &'static str,
    pub badge: &'static str,
    pub view_id: Option<&'static str>,
    pub badge_color: Option<Rgba>,
}

pub fn nav_items() -> &'static [NavItemDef] {
    &[
        NavItemDef { glyph: "◈", label: "Overview", badge: "", view_id: Some("overview"), badge_color: None },
        NavItemDef { glyph: "≡", label: "Processes", badge: "214", view_id: None, badge_color: None },
        NavItemDef { glyph: "◉", label: "Services", badge: "42", view_id: None, badge_color: None },
        NavItemDef { glyph: "▣", label: "Containers", badge: "11", view_id: None, badge_color: None },
        NavItemDef { glyph: "◧", label: "Config", badge: "2", view_id: Some("config"), badge_color: Some(WARN) },
        NavItemDef { glyph: "⌗", label: "Logs", badge: "4", view_id: None, badge_color: Some(CRIT) },
        NavItemDef { glyph: "▤", label: "Files", badge: "", view_id: None, badge_color: None },
        NavItemDef { glyph: "◷", label: "Cron", badge: "9", view_id: None, badge_color: None },
        NavItemDef { glyph: "◑", label: "Users", badge: "6", view_id: None, badge_color: None },
        NavItemDef { glyph: "⬡", label: "Firewall", badge: "ON", view_id: None, badge_color: Some(OK) },
        NavItemDef { glyph: "▶", label: "Terminal", badge: "⌘T", view_id: None, badge_color: None },
    ]
}

pub fn sidebar(active_view: &str, collapsed: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let sidebar_w = if collapsed { px(44.0) } else { px(184.0) };

    div()
        .w(sidebar_w)
        .flex_none()
        .h_full()
        .bg(BG_RAIL)
        .border_r_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .pt(px(6.0))
        // Nav items
        .children(nav_items().iter().enumerate().map(|(idx, item)| {
            let is_active = item.view_id == Some(active_view);
            let view_id_opt = item.view_id;
            let app_clone = app.clone();

            let mut row = div()
                .id(ElementId::NamedInteger("nav-item".into(), idx as u64))
                .flex()
                .items_center()
                .gap(px(10.0))
                .h(px(30.0))
                .px(if collapsed { px(14.0) } else { px(12.0) })
                .border_l_2()
                .border_color(if is_active { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
                .bg(if is_active { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                .cursor_pointer()
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(11.0))
                        .w(px(14.0))
                        .text_center()
                        .text_color(if is_active { TEXT_PRIMARY } else { TEXT_DIMMER })
                        .child(item.glyph),
                );

            if !collapsed {
                row = row
                    .child(
                        div()
                            .flex_1()
                            .font_family("Inter")
                            .text_size(px(12.0))
                            .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(if is_active { TEXT_PRIMARY } else { TEXT_TERTIARY })
                            .child(item.label),
                    )
                    .child(
                        div()
                            .font_family("JetBrains Mono")
                            .text_size(px(10.0))
                            .text_color(item.badge_color.unwrap_or(TEXT_FAINT))
                            .child(item.badge),
                    );
            }

            if let Some(target) = view_id_opt {
                row.on_click(move |_ev, _window, cx| {
                    app_clone.update(cx, |this, cx| {
                        this.set_view(target, cx);
                    });
                })
            } else {
                row
            }
        }))
        .child(div().flex_1())
        // Footer
        .child(
            div()
                .border_t_1()
                .border_color(BORDER_PANEL)
                .p(px(9.0))
                .px(if collapsed { px(10.0) } else { px(12.0) })
                .flex()
                .flex_col()
                .gap(px(6.0))
                .children(if !collapsed {
                    Some(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .font_family("JetBrains Mono")
                                    .text_size(px(10.0))
                                    .text_color(TEXT_FAINT)
                                    .child("agent v0.9.4-beta"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(7.0))
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
                                            .text_color(TEXT_DIM)
                                            .child("telemetry 1s"),
                                    ),
                            )
                    )
                } else {
                    None
                })
                .child({
                    let app_collapse = app.clone();
                    div()
                        .id("collapse-toggle-btn")
                        .flex()
                        .justify_between()
                        .items_center()
                        .font_family("Inter")
                        .text_size(px(11.0))
                        .text_color(TEXT_FAINT)
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            app_collapse.update(cx, |this, cx| {
                                this.sidebar_collapsed = !this.sidebar_collapsed;
                                cx.notify();
                            });
                        })
                        .children(if !collapsed {
                            Some(
                                div()
                                    .flex()
                                    .justify_between()
                                    .w_full()
                                    .child("Collapse")
                                    .child(
                                        div()
                                            .font_family("JetBrains Mono")
                                            .child("⌘\\"),
                                    )
                            )
                        } else {
                            Some(
                                div()
                                    .text_center()
                                    .w_full()
                                    .child("⇥")
                            )
                        })
                }),
        )
}
