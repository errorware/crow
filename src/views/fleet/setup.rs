use std::collections::BTreeMap;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::app::{CrowApp, Screen};
use crate::views::fleet::FleetState;
use crate::components::icons::{TablerIcon, tabler_icon};

/// Fleet groups, tag taxonomy, and host key lifecycle policies.
/// `map`: the FLEET MAP tab's content, built only while that tab shows.
pub fn fleet_setup_view(fleet: &FleetState, map: Option<AnyElement>, app: Entity<CrowApp>) -> impl IntoElement {
    let app_close = app.clone();
    let on_map = map.is_some();
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

    // What Crow knows about host keys today: whether a fingerprint was
    // pinned at enrollment. Key age needs the key's own timestamp (ERR-20).
    let pinned_keys = servers.iter().filter(|s| s.host_key_fingerprint.as_ref().is_some_and(|f| !f.trim().is_empty())).count();
    let missing_keys = total_servers - pinned_keys;
    let ages = crate::views::fleet::state::host_key_ages(servers, chrono::Utc::now().timestamp());

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
                                .child("groups, tag taxonomy, host key lifecycle, and the fleet map"),
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
        .child(setup_tabs(fleet.setup_page, app.clone()))
        .children(map)
        // 2. Metrics / KPI Strip
        .when(!on_map, |d| d.child(
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
                        .child(div().text_size(px(10.0)).text_color(TEXT_MUTED).child("HOST KEYS PINNED:"))
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if total_servers == 0 { TEXT_MUTED } else if missing_keys > 0 { WARN } else { OK })
                                .child(if total_servers == 0 {
                                    "NONE ENROLLED".to_string()
                                } else {
                                    format!("{pinned_keys}/{total_servers}")
                                }),
                        ),
                ),
        ))
        // 3. Main 2-Column Content Area
        .when(!on_map, |d| d.child(
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
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("FLEET TOPOLOGY & GROUP ASSIGNMENTS"),
                                )
                                .child({
                                    let app_btn = app.clone();
                                    div()
                                        .id("btn-setup-manage-groups")
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .border_1()
                                        .border_color(hex_rgb(0x38bdf8))
                                        .text_color(hex_rgb(0x38bdf8))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            app_btn.update(cx, |this, cx| {
                                                this.fleet.group_bar_open = true;
                                                this.reload_server_groups();
                                                this.set_screen(Screen::Fleet, cx);
                                            });
                                        })
                                        .child("+ MANAGE / NEW GROUP")
                                }),
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
                                                    .child(format!("{member_count} SERVER{}", if member_count == 1 { "" } else { "S" })),
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
                                                let is_online = srv.status == "online";

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
                                                            .gap(px(6.0))
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
                                                            }))
                                                            .child({
                                                                let app_grp = app.clone();
                                                                let sid = srv.id.clone();
                                                                div()
                                                                    .id(SharedString::from(format!("btn-change-group-{}", sid)))
                                                                    .px(px(6.0))
                                                                    .py(px(1.5))
                                                                    .bg(BG_CONTROL)
                                                                    .border_1()
                                                                    .border_color(BORDER_DEFAULT)
                                                                    .rounded_xs()
                                                                    .text_size(px(9.0))
                                                                    .text_color(TEXT_SECONDARY)
                                                                    .cursor_pointer()
                                                                    .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_MAX).border_color(BORDER_STRONG))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        cx.stop_propagation();
                                                                        let sid = sid.clone();
                                                                        app_grp.update(cx, |this, cx| {
                                                                            this.set_group_assign_target(Some(sid), cx);
                                                                        });
                                                                    })
                                                                    .child("CHANGE GROUP ▾")
                                                            }),
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
                                                .child("HOST KEYS"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .text_color(TEXT_DIM)
                                        .child("Crow pins each server's host key fingerprint at enrollment and refuses to connect if it changes. Ages come from the key files' dates on each server; there's no rotation policy yet."),
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
                                        .child(render_policy_row("Fingerprint pinned", pinned_keys, OK))
                                        .child(render_policy_row("Not pinned", missing_keys, if missing_keys > 0 { WARN } else { TEXT_MUTED }))
                                        .child(render_policy_row("Keys under a year old", ages.buckets[0], OK))
                                        .child(render_policy_row("Keys 1–2 years old", ages.buckets[1], TEXT_SECONDARY))
                                        .child(render_policy_row("Keys over 2 years old", ages.buckets[2], WARN))
                                        .children((ages.unknown > 0).then(|| render_policy_row("Key age not read yet", ages.unknown, TEXT_MUTED))),
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
                                        .child("Crow talks plain SSH and installs nothing on your servers. Fleet-wide actions aren't built yet; when they are, they'll run one host at a time and stop at the first failure."),
                                ),
                        ),
                ),
        ))
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
                .child(format!("{count} server{}", if count == 1 { "" } else { "s" })),
        )
}

/// GROUPS & POLICIES / FLEET MAP.
fn setup_tabs(page: crate::views::fleet::state::SetupPage, app: Entity<CrowApp>) -> impl IntoElement {
    use crate::views::fleet::state::SetupPage;
    let tab = |id: &'static str, label: &'static str, to: SetupPage| {
        let (app, on) = (app.clone(), page == to);
        div()
            .id(id)
            .h_full()
            .flex()
            .items_center()
            .px(px(12.0))
            .border_b_2()
            .border_color(if on { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .font_weight(if on { FontWeight::BOLD } else { FontWeight::NORMAL })
            .text_color(if on { TEXT_PRIMARY } else { TEXT_TERTIARY })
            .cursor_pointer()
            .hover(|s| s.text_color(TEXT_PRIMARY))
            .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_setup_page(to, cx)))
            .child(label)
    };
    div()
        .h(px(34.0))
        .flex_none()
        .flex()
        .items_end()
        .px(px(8.0))
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .child(tab("setup-tab-policies", "GROUPS & POLICIES", SetupPage::Policies))
        .child(tab("setup-tab-patching", "PATCHING", SetupPage::Patching))
        .child(tab("setup-tab-rollouts", "ROLLOUTS", SetupPage::Rollouts))
        .child(tab("setup-tab-drift", "DRIFT & SEARCH", SetupPage::Drift))
        .child(tab("setup-tab-hardening", "HARDENING", SetupPage::Hardening))
        .child(tab("setup-tab-certificates", "CERTIFICATES", SetupPage::Certificates))
        .child(tab("setup-tab-people", "PEOPLE", SetupPage::People))
        .child(tab("setup-tab-logsearch", "LOG SEARCH", SetupPage::LogSearch))
        .child(tab("setup-tab-checks", "CHECKS", SetupPage::Checks))
        .child(tab("setup-tab-map", "FLEET MAP", SetupPage::Map))
}
