use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};

pub struct TreeNode {
    pub icon: &'static str,
    pub label: &'static str,
    pub note: &'static str,
    pub depth: usize,
    pub is_selected: bool,
    pub color: Option<Rgba>,
}

pub struct MapNode {
    pub name: &'static str,
    pub sub: &'static str,
    pub x: f32,
    pub y: f32,
    pub status_color: Rgba,
    pub is_selected: bool,
}

pub struct MapEdge {
    pub points: &'static [(f32, f32)],
    pub color: Rgba,
    #[allow(dead_code)]
    pub is_dashed: bool,
}

pub fn fleet_setup_view(app: Entity<CrowApp>) -> impl IntoElement {
    let tree_nodes = [
        TreeNode { icon: "⬢", label: "acme-infra", note: "12 hosts", depth: 0, is_selected: true, color: None },
        TreeNode { icon: "⬡", label: "edge", note: "2", depth: 1, is_selected: false, color: None },
        TreeNode { icon: "●", label: "edge-01", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "●", label: "edge-02", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "⬡", label: "data", note: "4", depth: 1, is_selected: false, color: None },
        TreeNode { icon: "●", label: "db-primary", note: "spof?", depth: 2, is_selected: false, color: Some(WARN) },
        TreeNode { icon: "●", label: "db-replica-01", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "●", label: "redis-01", note: "spof", depth: 2, is_selected: false, color: Some(WARN) },
        TreeNode { icon: "●", label: "metrics-01", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "⬡", label: "workers", note: "2", depth: 1, is_selected: false, color: None },
        TreeNode { icon: "■", label: "worker-04", note: "down", depth: 2, is_selected: false, color: Some(CRIT) },
        TreeNode { icon: "●", label: "worker-05", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "⬡", label: "staging", note: "3", depth: 1, is_selected: false, color: None },
        TreeNode { icon: "●", label: "stage-web-01", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "●", label: "stage-db-01", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "●", label: "build-01", note: "", depth: 2, is_selected: false, color: Some(OK) },
        TreeNode { icon: "◇", label: "bastion", note: "ungrouped", depth: 1, is_selected: false, color: Some(TEXT_FAINT) },
    ];

    let nodes = [
        MapNode { name: "bastion", sub: "ssh jump · 204d", x: 30.0, y: 24.0, status_color: TEXT_FAINT, is_selected: false },
        MapNode { name: "lb-01", sub: "haproxy · 2 backends", x: 30.0, y: 178.0, status_color: OK, is_selected: false },
        MapNode { name: "edge-01", sub: "nginx · 412 conn", x: 250.0, y: 86.0, status_color: OK, is_selected: true },
        MapNode { name: "edge-02", sub: "nginx · 388 conn", x: 250.0, y: 246.0, status_color: OK, is_selected: false },
        MapNode { name: "worker-05", sub: "sidekiq · 6 queues", x: 250.0, y: 386.0, status_color: OK, is_selected: false },
        MapNode { name: "redis-01", sub: "queue · no replica", x: 490.0, y: 24.0, status_color: WARN, is_selected: false },
        MapNode { name: "db-primary", sub: "postgres 16 · rw", x: 490.0, y: 164.0, status_color: WARN, is_selected: false },
        MapNode { name: "db-replica-01", sub: "postgres 16 · ro", x: 740.0, y: 164.0, status_color: OK, is_selected: false },
        MapNode { name: "metrics-01", sub: "prometheus · 312 series", x: 740.0, y: 304.0, status_color: OK, is_selected: false },
    ];

    let edges: Vec<MapEdge> = vec![
        MapEdge { points: &[(180.0, 200.0), (215.0, 200.0), (215.0, 108.0), (250.0, 108.0)], color: hex_rgb(0x3a3c46), is_dashed: false },
        MapEdge { points: &[(180.0, 200.0), (215.0, 200.0), (215.0, 268.0), (250.0, 268.0)], color: hex_rgb(0x3a3c46), is_dashed: false },
        MapEdge { points: &[(400.0, 108.0), (445.0, 108.0), (445.0, 46.0), (490.0, 46.0)], color: hex_rgb(0x3a3c46), is_dashed: false },
        MapEdge { points: &[(400.0, 108.0), (445.0, 108.0), (445.0, 186.0), (490.0, 186.0)], color: hex_rgb(0x3a3c46), is_dashed: false },
        MapEdge { points: &[(400.0, 268.0), (452.0, 268.0), (452.0, 186.0), (490.0, 186.0)], color: hex_rgb(0x3a3c46), is_dashed: false },
        MapEdge { points: &[(640.0, 186.0), (740.0, 186.0)], color: OK, is_dashed: false },
        MapEdge { points: &[(400.0, 408.0), (462.0, 408.0), (462.0, 46.0), (490.0, 46.0)], color: hex_rgb(0x3a3c46), is_dashed: false },
        MapEdge { points: &[(180.0, 46.0), (215.0, 46.0), (215.0, 108.0), (250.0, 108.0)], color: hex_rgb(0x5a5d66), is_dashed: true },
        MapEdge { points: &[(740.0, 326.0), (700.0, 326.0), (700.0, 200.0), (640.0, 200.0)], color: hex_rgb(0x5a5d66), is_dashed: true },
    ];

    let policies = [
        ("Host key rotation", "every 90 days", "1 host out of policy (worker-04, 214d)", "ENFORCED", OK, OK_BG),
        ("Authorized keys", "crow-managed", "drift detected on edge-01 /root/.ssh", "DRIFT", CRIT, CRIT_BG),
        ("Baseline config", "hardened-prod v7", "11/12 hosts converged", "ENFORCED", OK, OK_BG),
        ("Access grants", "expire in 30 days", "4 active grants · 1 expiring in 2d", "ENFORCED", OK, OK_BG),
        ("Password auth", "disabled fleet-wide", "sshd_config enforced on connect", "ENFORCED", OK, OK_BG),
        ("Agent version", "pin 0.9.4", "bastion has no agent installed", "PENDING", WARN, WARN_BG),
        ("Fleet action gating", "serial + abort gate", "one host at a time, stop on first failure", "ENFORCED", OK, OK_BG),
    ];

    let rollout_steps = [
        ("✓", OK, "Resolve 12 hosts from 4 groups", "0 skipped"),
        ("✓", OK, "Diff baseline against each host", "1 drift"),
        ("✓", OK, "Verify a rollback path per host", "all ok"),
        ("○", TEXT_FAINT, "Apply serially, newest staging group first", "~6 min"),
        ("○", TEXT_FAINT, "Re-verify SSH reachability after each host", "abort on fail"),
    ];

    let app_close = app.clone();

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
                                .child("FLEET SETUP"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .text_color(TEXT_DIM)
                                .child("groups, dependencies & baseline policy"),
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
        // 2. 3-Column Content: Tree (200px) | Dependency Map (flex-1) | Policy & Rollout Rail (340px)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Left Column: Topology Tree
                .child(
                    div()
                        .w(px(200.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .pt(px(6.0))
                        .child(
                            div()
                                .id("fleet-setup-tree")
                                .flex_1()
                                .overflow_y_scroll()
                                .children(tree_nodes.into_iter().enumerate().map(|(idx, node)| {
                                    div()
                                        .id(ElementId::NamedInteger("tree-node".into(), idx as u64))
                                        .h(px(28.0))
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .pl(px(12.0 + (node.depth as f32 * 14.0)))
                                        .pr(px(12.0))
                                        .bg(if node.is_selected { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                                        .border_l_2()
                                        .border_color(if node.is_selected { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .text_color(node.color.unwrap_or(if node.depth == 2 { OK } else { TEXT_DIMMER }))
                                                .child(node.icon),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .font_family(FONT_MONO)
                                                .text_size(if node.depth == 0 { px(11.5) } else { px(11.0) })
                                                .font_weight(if node.depth < 2 { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                                .text_color(if node.depth == 0 { TEXT_MAX } else if node.depth == 1 { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                                .child(node.label),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(node.color.unwrap_or(TEXT_FAINT))
                                                .child(node.note),
                                        )
                                })),
                        )
                        .child(
                            div()
                                .flex_none()
                                .p(px(12.0))
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_FAINT)
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .child("New group")
                                        .child("⌘G"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .child("Import from Terraform")
                                        .child("⌘I"),
                                ),
                        ),
                )
                // Center Column: Dependency Map Canvas
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .bg(BG_APP)
                        // Canvas Header
                        .child(
                            div()
                                .h(px(30.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("DEPENDENCY MAP"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .child("inferred from open sockets · 9 edges"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .child("drag to link · ⌫ to unlink"),
                                ),
                        )
                        // Legend Bar
                        .child(
                            div()
                                .h(px(26.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_SUBHEAD)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(16.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIMMER)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(div().w(px(16.0)).h(px(1.5)).bg(hex_rgb(0x3a3c46)))
                                        .child("traffic"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(div().w(px(16.0)).h(px(1.5)).bg(OK))
                                        .child("replication"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(div().w(px(16.0)).h(px(1.0)).bg(hex_rgb(0x5a5d66)))
                                        .child("admin / scrape"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .text_color(WARN)
                                        .child("1 single point of failure detected"),
                                ),
                        )
                        // Graph Layout Container with Canvas + Absolute Host Nodes
                        .child(
                            div()
                                .flex_1()
                                .min_h(px(0.0))
                                .relative()
                                .overflow_hidden()
                                // Vector Lines Canvas
                                .child(
                                    canvas(
                                        |_bounds, _window, _cx| (),
                                        move |bounds, (), window, _cx| {
                                            for edge in &edges {
                                                if edge.points.len() < 2 {
                                                    continue;
                                                }
                                                let mut path = PathBuilder::stroke(px(1.0));
                                                let start = edge.points[0];
                                                path.move_to(point(bounds.origin.x + px(start.0), bounds.origin.y + px(start.1)));
                                                for pt in &edge.points[1..] {
                                                    path.line_to(point(bounds.origin.x + px(pt.0), bounds.origin.y + px(pt.1)));
                                                }
                                                if let Ok(built) = path.build() {
                                                    window.paint_path(built, edge.color);
                                                }
                                            }
                                        },
                                    )
                                    .size_full()
                                    .absolute()
                                    .top_0()
                                    .left_0(),
                                )
                                // Nodes
                                .children(nodes.into_iter().enumerate().map(|(idx, node)| {
                                    div()
                                        .id(ElementId::NamedInteger("map-node".into(), idx as u64))
                                        .absolute()
                                        .top(px(node.y))
                                        .left(px(node.x))
                                        .w(px(150.0))
                                        .bg(if node.is_selected { BG_ROW_SELECTED } else { BG_SUBHEAD })
                                        .border_1()
                                        .border_color(if node.status_color == WARN { WARN_BG } else { BORDER_DEFAULT })
                                        .border_l_2()
                                        .border_color(if node.is_selected { TEXT_PRIMARY } else if node.status_color == WARN { WARN } else { hex_rgba(0, 0.0) })
                                        .p(px(8.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(7.0))
                                                .child(
                                                    div()
                                                        .size(px(6.0))
                                                        .rounded_full()
                                                        .bg(node.status_color)
                                                        .flex_none(),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .font_weight(if node.is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                        .text_color(if node.is_selected { TEXT_MAX } else { TEXT_PRIMARY })
                                                        .child(node.name),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_FAINT)
                                                .child(node.sub),
                                        )
                                }))
                                // Floating SPOF Callout
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(26.0))
                                        .left(px(660.0))
                                        .w(px(240.0))
                                        .p(px(10.0))
                                        .bg(hex_rgb(0x100c06))
                                        .border_1()
                                        .border_color(hex_rgb(0x2e2210))
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(7.0))
                                                .child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(WARN).child("▲"))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(WARN)
                                                        .child("SINGLE POINT OF FAILURE"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .line_height(relative(1.45))
                                                .text_color(TEXT_TERTIARY)
                                                .child("redis-01 has no replica and 3 dependents. Losing it stops all queue work."),
                                        ),
                                ),
                        ),
                )
                // Right Column: Fleet Policy & Rollout Plan (340px)
                .child(
                    div()
                        .w(px(340.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
                        // Policy Header
                        .child(
                            div()
                                .h(px(30.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("FLEET POLICY"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIMMER)
                                        .child("applies to 12 hosts"),
                                ),
                        )
                        // Policy List
                        .child(
                            div()
                                .id("fleet-setup-policies")
                                .flex_1()
                                .overflow_y_scroll()
                                .children(policies.iter().map(|(label, val, note, tag, tag_fg, tag_bg)| {
                                    let is_crit = *tag_fg == CRIT;
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .px(px(12.0))
                                        .py(px(9.0))
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .border_l_2()
                                        .border_color(if is_crit { CRIT } else { hex_rgba(0, 0.0) })
                                        .bg(if is_crit { CRIT_ROW_BG } else { hex_rgba(0, 0.0) })
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.5))
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(TEXT_PRIMARY)
                                                        .child(*label),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(4.0))
                                                        .py(px(1.5))
                                                        .bg(*tag_bg)
                                                        .text_color(*tag_fg)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(8.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .child(*tag),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .px(px(7.0))
                                                        .py(px(3.0))
                                                        .bg(BG_OVERLAY_PANEL)
                                                        .border_1()
                                                        .border_color(BORDER_DEFAULT)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.5))
                                                        .text_color(TEXT_SECONDARY)
                                                        .child(*val),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.5))
                                                        .text_color(TEXT_DIMMER)
                                                        .child(*note),
                                                ),
                                        )
                                })),
                        )
                        // Rollout Plan Box
                        .child(
                            div()
                                .flex_none()
                                .p(px(12.0))
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .flex_col()
                                .gap(px(7.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIMMER)
                                        .child("ROLLOUT PLAN"),
                                )
                                .children(rollout_steps.iter().map(|(glyph, fg, step, note)| {
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(9.0))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .child(
                                            div()
                                                .w(px(12.0))
                                                .text_color(*fg)
                                                .child(*glyph),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_color(*fg)
                                                .child(*step),
                                        )
                                        .child(
                                            div()
                                                .text_color(TEXT_FAINT)
                                                .child(*note),
                                        )
                                })),
                        )
                        // Rollout Actions Bar
                        .child(
                            div()
                                .h(px(42.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(12.0))
                                .bg(BG_PANEL)
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(8.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(
                                    div()
                                        .px(px(9.0))
                                        .py(px(5.0))
                                        .border_1()
                                        .border_color(BORDER_KEY)
                                        .text_color(TEXT_TERTIARY)
                                        .child("REVERT"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .px(px(9.0))
                                        .py(px(5.0))
                                        .bg(BG_CONTROL)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .text_color(TEXT_SECONDARY)
                                        .child("DRY RUN"),
                                )
                                .child(
                                    div()
                                        .px(px(11.0))
                                        .py(px(6.0))
                                        .bg(OK)
                                        .text_color(BG_WINDOW)
                                        .font_weight(FontWeight::BOLD)
                                        .child("ROLL OUT ⌘⏎"),
                                ),
                        ),
                ),
        )
}
