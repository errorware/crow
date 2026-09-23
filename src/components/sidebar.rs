use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

use crate::components::icons::{TablerIcon, tabler_icon};

pub struct NavItemDef {
    pub icon: TablerIcon,
    pub label: &'static str,
    pub view_id: Option<&'static str>,
    /// Fixed text shown instead of a live badge (e.g. a shortcut hint).
    pub hint: &'static str,
}

pub fn nav_items() -> &'static [NavItemDef] {
    &[
        NavItemDef { icon: TablerIcon::LayoutDashboard, label: "Overview", view_id: Some("overview"), hint: "" },
        NavItemDef { icon: TablerIcon::Server, label: "Services", view_id: Some("services"), hint: "" },
        NavItemDef { icon: TablerIcon::Cpu, label: "Processes", view_id: Some("processes"), hint: "" },
        NavItemDef { icon: TablerIcon::Network, label: "Sockets", view_id: Some("sockets"), hint: "" },
        NavItemDef { icon: TablerIcon::Box, label: "Containers", view_id: Some("containers"), hint: "" },
        NavItemDef { icon: TablerIcon::AdjustmentsHorizontal, label: "Config", view_id: Some("config"), hint: "" },
        NavItemDef { icon: TablerIcon::FileText, label: "Logs", view_id: Some("logs"), hint: "" },
        NavItemDef { icon: TablerIcon::Folder, label: "Files", view_id: Some("files"), hint: "" },
        NavItemDef { icon: TablerIcon::Clock, label: "Cron", view_id: Some("cron"), hint: "" },
        NavItemDef { icon: TablerIcon::Users, label: "Users", view_id: Some("users"), hint: "" },
        NavItemDef { icon: TablerIcon::ShieldCheck, label: "Firewall", view_id: Some("firewall"), hint: "" },
        NavItemDef { icon: TablerIcon::Terminal2, label: "Terminal", view_id: Some("terminal"), hint: "⌘T" },
    ]
}

/// Live badge per view id: (text, color). Views without an entry show none.
pub type NavBadges = Vec<(&'static str, String, Rgba)>;

pub fn sidebar(active_view: &str, collapsed: bool, badges: &NavBadges, app: Entity<CrowApp>) -> impl IntoElement {
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
            let (badge_text, badge_color) = badges
                .iter()
                .find(|(view, _, _)| Some(*view) == item.view_id)
                .map(|(_, text, color)| (text.clone(), *color))
                .unwrap_or_else(|| (item.hint.to_string(), TEXT_FAINT));
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
                            .text_color(badge_color)
                            .child(badge_text),
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
                                    .child(concat!("crow v", env!("CARGO_PKG_VERSION"))),
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
                                            .child("agentless · ssh"),
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
