use std::collections::BTreeMap;
use chrono::Utc;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};
use crate::views::fleet::FleetState;
use crate::components::icons::{TablerIcon, tabler_icon};

/// Fleet groups, tag taxonomy, and host key lifecycle policies.
pub fn fleet_setup_view(fleet: &FleetState, app: Entity<CrowApp>) -> impl IntoElement {
    let app_close = app.clone();
    let servers = &fleet.servers;
    let total_servers = servers.len();

    // Group servers by group_name
    let mut group_map: BTreeMap<String, Vec<&crate::vault::ServerRecord>> = BTreeMap::new();
    for s in servers {
        let g = if s.group_name.trim().is_empty() {
            "default / unassigned"
        } else {
            s.group_name.trim()
        };
        group_map.entry(g.to_string()).or_default().push(s);
    }

    // Collect tags frequency
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    for s in servers {
        for t in &s.tags {
            if !t.trim().is_empty() {
                *tag_counts.entry(t.trim().to_string()).or_default() += 1;
            }
        }
    }

    // Host key age calculation
    let mut fresh_keys = 0; // < 90d
    let mut mature_keys = 0; // 90-180d
    let mut stale_keys = 0; // 180-365d
    let mut critical_keys = 0; // > 365d
    let mut missing_keys = 0;

    for s in servers {
        if s.host_key_fingerprint.as_ref().map_or(true, |f| f.trim().is_empty()) {
            missing_keys += 1;
            continue;
        }
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&s.created_at) {
            let days = (Utc::now() - created.with_timezone(&Utc)).num_days();
            if days < 90 {
                fresh_keys += 1;
            } else if days < 180 {
                mature_keys += 1;
            } else if days < 365 {
                stale_keys += 1;
            } else {
                critical_keys += 1;
            }
        } else {
            missing_keys += 1;
        }
    }

    let stale_total = stale_keys + critical_keys;

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Top Header
        .child(
            div()
                .h(px(52.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child("FLEET SETUP & POLICIES"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .text_color(TEXT_DIM)
                                .child("groups, tag taxonomy, and host key lifecycle"),
                        ),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-close-fleet-setup")
                        .px(px(10.0))
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_close.update(cx, |this, cx| {
                                this.set_screen(Screen::Fleet, cx);
                            });
                        })
                        .child("CLOSE esc"),
                ),
        )
        // 2. Metrics / KPI Strip
        .child(
            div()
                .h(px(46.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(16.0))
                .bg(hex_rgba(0x000000, 0.2))
                .border_b_1()
                .border_color(BORDER_ROW)
                .gap(px(24.0))
                .font_family(FONT_MONO)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().text_size(px(10.0)).text_color(TEXT_MUTED).child("ENROLLED SERVERS:"))
                        .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(total_servers.to_string())),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().text_size(px(10.0)).text_color(TEXT_MUTED).child("FLEET GROUPS:"))
                        .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(hex_rgb(0x38bdf8)).child(group_map.len().to_string())),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().text_size(px(10.0)).text_color(TEXT_MUTED).child("UNIQUE TAGS:"))
                        .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(hex_rgb(0xa78bfa)).child(tag_counts.len().to_string())),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().text_size(px(10.0)).text_color(TEXT_MUTED).child("HOST KEY HEALTH:"))
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if stale_total > 0 { WARN } else { OK })
                                .child(if stale_total > 0 {
                                    format!("{} STALE (>180d)", stale_total)
                                } else if total_servers == 0 {
                                    "NONE ENROLLED".to_string()
                                } else {
                                    "ALL RECENT (<180d)".to_string()
                                }),
                        ),
                ),
        )
        // 3. Main 2-Column Content Area
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .p(px(16.0))
                .flex()
                .gap(px(16.0))
                // Left Column: Fleet Groups & Member Nodes
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("FLEET TOPOLOGY & GROUP ASSIGNMENTS"),
                        )
                        .children(if group_map.is_empty() {
                            vec![
                                div()
                                    .p(px(24.0))
                                    .bg(BG_PANEL)
                                    .border_1()
                                    .border_color(BORDER_ROW)
                                    .rounded_sm()
                                    .font_family(FONT_MONO)
                                    .text_size(px(11.0))
                                    .text_color(TEXT_FAINT)
                                    .child("No servers currently enrolled in fleet.")
                                    .into_any_element(),
                            ]
                        } else {
                            group_map.into_iter().enumerate().map(|(g_idx, (group_name, members))| {
                                let member_count = members.len();
                                div()
                                    .id(ElementId::NamedInteger("group-card".into(), g_idx as u64))
                                    .bg(BG_PANEL)
                                    .border_1()
                                    .border_color(BORDER_PANEL)
                                    .rounded_md()
                                    .flex()
                                    .flex_col()
                                    // Group Card Header
                                    .child(
                                        div()
                                            .h(px(34.0))
                                            .px(px(12.0))
                                            .bg(BG_SUBHEAD)
                                            .border_b_1()
                                            .border_color(BORDER_ROW)
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .font_family(FONT_MONO)
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(8.0))
                                                    .child(tabler_icon(TablerIcon::Folder).size(px(13.0)).text_color(hex_rgb(0x38bdf8)))
                                                    .child(
                                                        div()
                                                            .text_size(px(11.5))
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(TEXT_MAX)
                                                            .child(group_name),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(1.5))
                                                    .bg(BG_CONTROL)
                                                    .border_1()
                                                    .border_color(BORDER_DEFAULT)
                                                    .rounded_xs()
                                                    .text_size(px(9.5))
                                                    .text_color(TEXT_SECONDARY)
                                                    .child(format!("{} SERVERS", member_count)),
                                            ),
                                    )
                                    // Member Server List
                                    .child(
                                        div()
                                            .p(px(8.0))
                                            .flex()
                                            .flex_col()
                                            .gap(px(4.0))
                                            .children(members.into_iter().enumerate().map(|(m_idx, srv)| {
                                                let app_srv = app.clone();
                                                let srv_id = srv.id.clone();
                                                let is_online = srv.status == "active";

                                                div()
                                                    .id(ElementId::NamedInteger(format!("grp-srv-{g_idx}").into(), m_idx as u64))
                                                    .px(px(10.0))
                                                    .py(px(6.0))
                                                    .bg(BG_APP)
                                                    .border_1()
                                                    .border_color(BORDER_ROW)
                                                    .rounded_sm()
                                                    .cursor_pointer()
                                                    .hover(|s| s.bg(BG_ROW_HOVER).border_color(TEXT_SECONDARY))
                                                    .on_click(move |_ev, _window, cx| {
                                                        let sid = srv_id.clone();
                                                        app_srv.update(cx, |this, cx| {
                                                            this.switch_tab(&sid, cx);
                                                            this.set_screen(Screen::Server, cx);
                                                        });
                                                    })
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .font_family(FONT_MONO)
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(8.0))
                                                            .child(
                                                                div()
                                                                    .size(px(6.0))
                                                                    .rounded_full()
                                                                    .bg(if is_online { OK } else { TEXT_MUTED }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_size(px(11.0))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(TEXT_PRIMARY)
                                                                    .child(srv.name.clone()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_size(px(10.0))
                                                                    .text_color(TEXT_MUTED)
                                                                    .child(format!("{}:{}", srv.host, srv.port)),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(4.0))
                                                            .children(srv.tags.iter().map(|t| {
                                                                div()
                                                                    .px(px(5.0))
                                                                    .py(px(1.0))
                                                                    .bg(hex_rgba(0xa78bfa, 0.12))
                                                                    .border_1()
                                                                    .border_color(hex_rgba(0xa78bfa, 0.3))
                                                                    .text_color(hex_rgb(0xa78bfa))
                                                                    .text_size(px(8.5))
                                                                    .rounded_xs()
                                                                    .child(t.clone())
                                                            })),
                                                    )
                                            })),
                                    )
                                    .into_any_element()
                            }).collect()
                        }),
                )
                // Right Column: Policies & Metadata Taxonomy
                .child(
                    div()
                        .w(px(380.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        // Card 1: Host Key Rotation Policy
                        .child(
                            div()
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .rounded_md()
                                .p(px(14.0))
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .font_family(FONT_MONO)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(tabler_icon(TablerIcon::Key).size(px(14.0)).text_color(hex_rgb(0xfacc15)))
                                        .child(
                                            div()
                                                .text_size(px(11.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MAX)
                                                .child("HOST KEY ROTATION POLICY"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIM)
                                        .child("SSH host keys establish identity and protect against man-in-the-middle impersonation. Crow audits key ages across your fleet to enforce security best practices."),
                                )
                                // Age breakdown rows
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .border_t_1()
                                        .border_color(BORDER_ROW)
                                        .pt(px(8.0))
                                        .child(render_policy_row("Fresh (< 90 days)", fresh_keys, OK))
                                        .child(render_policy_row("Mature (90–180 days)", mature_keys, hex_rgb(0x38bdf8)))
                                        .child(render_policy_row("Stale (> 180 days)", stale_keys, WARN))
                                        .child(render_policy_row("Critical (> 1 year)", critical_keys, CRIT))
                                        .children(if missing_keys > 0 {
                                            Some(render_policy_row("Unfingerprinted", missing_keys, TEXT_MUTED))
                                        } else {
                                            None
                                        }),
                                ),
                        )
                        // Card 2: Tag Taxonomy
                        .child(
                            div()
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .rounded_md()
                                .p(px(14.0))
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .font_family(FONT_MONO)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(tabler_icon(TablerIcon::AdjustmentsHorizontal).size(px(14.0)).text_color(hex_rgb(0xa78bfa)))
                                        .child(
                                            div()
                                                .text_size(px(11.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MAX)
                                                .child("FLEET TAG TAXONOMY"),
                                        ),
                                )
                                .children(if tag_counts.is_empty() {
                                    vec![
                                        div()
                                            .text_size(px(10.5))
                                            .text_color(TEXT_FAINT)
                                            .child("No custom tags defined. Assign tags to nodes during onboarding or in server configuration to categorize your infrastructure.")
                                            .into_any_element(),
                                    ]
                                } else {
                                    vec![
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .gap(px(6.0))
                                            .children(tag_counts.into_iter().map(|(tag, count)| {
                                                div()
                                                    .px(px(8.0))
                                                    .py(px(3.0))
                                                    .bg(BG_CONTROL)
                                                    .border_1()
                                                    .border_color(BORDER_DEFAULT)
                                                    .rounded_sm()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(6.0))
                                                    .child(div().text_size(px(10.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(tag))
                                                    .child(div().text_size(px(9.0)).text_color(TEXT_MUTED).child(count.to_string()))
                                            }))
                                            .into_any_element(),
                                    ]
                                }),
                        )
                        // Card 3: Execution Safety (The Abort Gate)
                        .child(
                            div()
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .rounded_md()
                                .p(px(14.0))
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .font_family(FONT_MONO)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(tabler_icon(TablerIcon::ShieldCheck).size(px(14.0)).text_color(OK))
                                        .child(
                                            div()
                                                .text_size(px(11.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_MAX)
                                                .child("AGENTLESS EXECUTION GUARANTEE"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .child("Crow connects directly via standard SSH. Fleet-wide operations run sequentially and abort at the first failure to prevent blast-radius propagation."),
                                ),
                        ),
                ),
        )
}

fn render_policy_row(label: &'static str, count: usize, color: Rgba) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .py(px(2.0))
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().size(px(6.0)).rounded_full().bg(color))
                .child(div().text_color(TEXT_SECONDARY).child(label)),
        )
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_color(if count > 0 { color } else { TEXT_MUTED })
                .child(format!("{count} servers")),
        )
}
