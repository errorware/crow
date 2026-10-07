//! The move-to-group dialog (ERR-97).

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::views::fleet::FleetState;
use crate::components::icons::{TablerIcon, tabler_icon};

pub fn group_assign_modal(
    server_id: &str,
    fleet: &FleetState,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let app_close_scrim = app.clone();
    let server = fleet.servers.iter().find(|s| s.id == server_id);
    let server_name = server.map(|s| s.name.as_str()).unwrap_or("Server");
    let current_group = server.map(|s| s.group_name.as_str()).unwrap_or("default");

    // Collect all unique groups, ensuring default/unassigned is at the top
    let mut all_groups = vec!["default".to_string()];
    for g in &fleet.groups {
        let trimmed = g.trim();
        if !trimmed.is_empty() && trimmed != "default" && !all_groups.contains(&trimmed.to_string()) {
            all_groups.push(trimmed.to_string());
        }
    }
    for s in &fleet.servers {
        let trimmed = s.group_name.trim();
        if !trimmed.is_empty() && trimmed != "default" && !all_groups.contains(&trimmed.to_string()) {
            all_groups.push(trimmed.to_string());
        }
    }

    div()
        .id("group-assign-modal-scrim")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x060709, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_close_scrim.update(cx, |this, cx| {
                this.set_group_assign_target(None, cx);
            });
        })
        .child(
            div()
                .id("group-assign-modal-card")
                .w(px(380.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .rounded_md()
                .shadow_lg()
                .flex()
                .flex_col()
                .font_family(FONT_MONO)
                // Clicks inside stay inside (the backdrop closes it).
                .occlude()
                .on_click(|_ev, _window, cx| cx.stop_propagation())
                // Header
                .child(
                    div()
                        .h(px(40.0))
                        .px(px(14.0))
                        .bg(BG_SUBHEAD)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(tabler_icon(TablerIcon::Folder).size(px(14.0)).text_color(hex_rgb(0x38bdf8)))
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child(format!("GROUP · {server_name}")),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-group-assign-close")
                                .cursor_pointer()
                                .p(px(4.0))
                                .text_color(TEXT_DIM)
                                .hover(|s| s.text_color(TEXT_MAX))
                                .on_click(move |_ev, _window, cx| {
                                    app_close.update(cx, |this, cx| {
                                        this.set_group_assign_target(None, cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::X).size(px(14.0)).text_color(TEXT_DIM)),
                        ),
                )
                // Subhead
                .child(
                    div()
                        .px(px(14.0))
                        .py(px(8.0))
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .child("Move this server to a group:"),
                )
                // Group List
                .child(
                    div()
                        .max_h(px(240.0))
                        .overflow_y_scrollbar()
                        .py(px(4.0))
                        .children(all_groups.into_iter().map(|grp| {
                            let app_sel = app.clone();
                            let s_id = server_id.to_string();
                            let g_val = grp.clone();
                            let is_assigned = (grp == "default" && (current_group.is_empty() || current_group == "default"))
                                || grp == current_group;
                            let display_label = if grp == "default" {
                                "no group".to_string()
                            } else {
                                grp.clone()
                            };

                            div()
                                .id(SharedString::from(format!("group-assign-pick-{grp}")))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(14.0))
                                .py(px(6.0))
                                .cursor_pointer()
                                .bg(if is_assigned { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let sid = s_id.clone();
                                    let g = g_val.clone();
                                    app_sel.update(cx, |this, cx| {
                                        this.assign_server_group(&sid, &g, cx);
                                    });
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(tabler_icon(TablerIcon::Folder).size(px(12.0)).text_color(if is_assigned { hex_rgb(0x38bdf8) } else { TEXT_DIM }))
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(if is_assigned { TEXT_MAX } else { TEXT_PRIMARY })
                                                .font_weight(if is_assigned { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .child(display_label),
                                        ),
                                )
                                .children(is_assigned.then(|| {
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(OK)
                                        .font_weight(FontWeight::BOLD)
                                        .child("✓")
                                }))
                        })),
                ),
        )
}
