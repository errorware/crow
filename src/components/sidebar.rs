use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

use crate::components::icons::{TablerIcon, tabler_icon};

pub struct NavItemDef {
    pub icon: TablerIcon,
    pub label: &'static str,
    pub badge: &'static str,
    pub view_id: Option<&'static str>,
    pub badge_color: Option<Rgba>,
}

pub fn nav_items() -> &'static [NavItemDef] {
    &[
        NavItemDef { icon: TablerIcon::LayoutDashboard, label: "Overview", badge: "", view_id: Some("overview"), badge_color: None },
        NavItemDef { icon: TablerIcon::Cpu, label: "Processes", badge: "214", view_id: Some("processes"), badge_color: None },
        NavItemDef { icon: TablerIcon::Server, label: "Services", badge: "42", view_id: Some("services"), badge_color: None },
        NavItemDef { icon: TablerIcon::Box, label: "Containers", badge: "11", view_id: Some("containers"), badge_color: None },
        NavItemDef { icon: TablerIcon::AdjustmentsHorizontal, label: "Config", badge: "2", view_id: Some("config"), badge_color: Some(WARN) },
        NavItemDef { icon: TablerIcon::FileText, label: "Logs", badge: "4", view_id: Some("logs"), badge_color: Some(CRIT) },
        NavItemDef { icon: TablerIcon::Folder, label: "Files", badge: "", view_id: Some("files"), badge_color: None },
        NavItemDef { icon: TablerIcon::Clock, label: "Cron", badge: "9", view_id: Some("cron"), badge_color: None },
        NavItemDef { icon: TablerIcon::Users, label: "Users", badge: "6", view_id: Some("users"), badge_color: None },
        NavItemDef { icon: TablerIcon::ShieldCheck, label: "Firewall", badge: "ON", view_id: Some("firewall"), badge_color: Some(OK) },
        NavItemDef { icon: TablerIcon::Terminal2, label: "Terminal", badge: "⌘T", view_id: Some("terminal"), badge_color: None },
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
        // Nav items
        .children(nav_items().iter().enumerate().map(|(idx, item)| {
            let is_active = item.view_id == Some(active_view);
            let view_id_opt = item.view_id;
            let app_clone = app.clone();

            let mut row = div()
                .id(ElementId::NamedInteger("nav-item".into(), idx as u64))
                .relative()
                .flex()
                .items_center()
                .gap(px(10.0))
                .h(px(30.0))
                .px(if collapsed { px(14.0) } else { px(12.0) })
                .bg(if is_active { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                .children(if is_active {
                    Some(left_indicator(TEXT_PRIMARY))
                } else {
                    None
                })
                .cursor_pointer()
                .child(
                    div()
                        .w(px(14.0))
                        .h(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_none()
                        .child(
                            tabler_icon(item.icon)
                                .size(px(14.0))
                                .text_color(if is_active { TEXT_PRIMARY } else { TEXT_DIMMER }),
                        ),
                );

            if !collapsed {
                row = row
                    .child(
                        div()
                            .flex_1()
                            .font_family(FONT_MONO)
                            .text_size(px(11.5))
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
                        .font_family(FONT_MONO)
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
                                    .items_center()
                                    .w_full()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                tabler_icon(TablerIcon::LayoutSidebarLeftCollapse)
                                                    .size(px(13.0))
                                                    .text_color(TEXT_FAINT),
                                            )
                                            .child("Collapse"),
                                    )
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .child("⌘\\"),
                                    ),
                            )
                        } else {
                            Some(
                                div()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .w_full()
                                    .child(
                                        tabler_icon(TablerIcon::LayoutSidebarLeftExpand)
                                            .size(px(14.0))
                                            .text_color(TEXT_FAINT),
                                    ),
                            )
                        })
                }),
        )
}
