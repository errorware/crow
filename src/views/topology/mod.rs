//! The fleet map (ERR-120): Crow, its servers, jump hosts, local VMs and
//! lab containers, and the keys that unlock them, with the risks Crow has
//! read on each. Drag to pan, scroll to zoom, click a node to inspect it.

pub mod state;

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::CrowApp;
use crate::components::icon_button::icon_button;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::theme::*;
use crate::topology::layout::{layout, Layout, NODE_H, NODE_W};
use crate::topology::{EdgeKind, Graph, Health, Level, Node, NodeKind};

const JUMP_BLUE: u32 = 0x8ab4ff;
const KEY_AMBER: u32 = 0xd6a24a;

fn level_color(level: Option<Level>) -> Rgba {
    match level {
        Some(Level::Crit) => CRIT,
        Some(Level::Warn) => WARN,
        None => hex_rgba(0xffffff, 0.10),
    }
}

fn kind_icon(kind: NodeKind) -> TablerIcon {
    match kind {
        NodeKind::Crow => TablerIcon::Network,
        NodeKind::Server => TablerIcon::Server,
        NodeKind::Vm => TablerIcon::Cpu,
        NodeKind::Container | NodeKind::Workload => TablerIcon::Box,
        NodeKind::Key => TablerIcon::Key,
    }
}

fn kind_label(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Crow => "CROW",
        NodeKind::Server => "SERVER",
        NodeKind::Vm => "MULTIPASS VM",
        NodeKind::Container => "LAB CONTAINER",
        NodeKind::Workload => "CONTAINER",
        NodeKind::Key => "SSH KEY",
    }
}

/// Whether the filters leave a node at full strength.
fn shown(n: &Node, lane: Option<&str>, risky_only: bool) -> bool {
    let lane_ok = match (lane, n.kind) {
        (None, _) | (_, NodeKind::Crow | NodeKind::Key) => true,
        (Some(l), _) => n.lane == l,
    };
    lane_ok && (!risky_only || !n.risks.is_empty() || n.kind == NodeKind::Crow)
}

pub fn topology_page(app_state: &CrowApp, app: Entity<CrowApp>) -> impl IntoElement {
    let graph = app_state.topology_graph();
    let placed = layout(&graph);
    let st = &app_state.topology;
    let selected = st.selected.as_ref().and_then(|id| graph.node(id)).cloned();
    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .child(toolbar(app_state, &graph, app.clone()))
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .child(map_area(app_state, &graph, &placed, app.clone()))
                .children(selected.map(|n| inspector(app_state, &graph, &n, app.clone()))),
        )
}

fn chip(id: impl Into<ElementId>, label: String, on: bool) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(8.0))
        .py(px(2.0))
        .border_1()
        .border_color(if on { TEXT_PRIMARY } else { BORDER_DEFAULT })
        .bg(if on { BG_NAV_ACTIVE } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .text_color(if on { TEXT_PRIMARY } else { TEXT_TERTIARY })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(label)
}

fn toolbar(app_state: &CrowApp, graph: &Graph, app: Entity<CrowApp>) -> impl IntoElement {
    let st = &app_state.topology;
    let mut lanes: Vec<String> = graph.nodes.iter().filter(|n| !matches!(n.kind, NodeKind::Crow | NodeKind::Key)).map(|n| n.lane.clone()).collect();
    lanes.sort();
    lanes.dedup();
    let (crit, warn) = graph.nodes.iter().fold((0, 0), |(c, w), n| match n.worst() {
        Some(Level::Crit) => (c + 1, w),
        Some(Level::Warn) => (c, w + 1),
        None => (c, w),
    });
    let checked = if st.scanning {
        "checking posture…".to_string()
    } else if st.posture_checked_at == 0 {
        "posture not checked yet".to_string()
    } else {
        let mins = (chrono::Utc::now().timestamp() - st.posture_checked_at) / 60;
        format!("posture checked {} ago · {} servers", if mins < 1 { "just now".to_string() } else if mins < 60 { format!("{mins} min") } else { format!("{} h", mins / 60) }, st.posture.len())
    };
    let (app_all, app_risky, app_scan, app_out, app_in, app_fit) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone(), app.clone());
    div()
        .h(px(36.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .px(px(14.0))
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(chip("topo-lane-all", "ALL".into(), st.lane.is_none()).on_click(move |_ev, _w, cx| app_all.update(cx, |t, cx| t.topology_set_lane(None, cx))))
                .children(lanes.into_iter().enumerate().map(|(i, l)| {
                    let (app, on, lane) = (app.clone(), st.lane.as_deref() == Some(l.as_str()), l.clone());
                    chip(ElementId::NamedInteger("topo-lane".into(), i as u64), l, on).on_click(move |_ev, _w, cx| {
                        let lane = lane.clone();
                        app.update(cx, |t, cx| t.topology_set_lane(if on { None } else { Some(lane) }, cx))
                    })
                }))
                .child(div().w(px(10.0)))
                .child(chip("topo-risky", format!("AT RISK ONLY · {crit} crit · {warn} warn"), st.risky_only).on_click(move |_ev, _w, cx| app_risky.update(cx, |t, cx| t.topology_toggle_risky(cx)))),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().font_family(FONT_MONO).text_size(px(9.5)).text_color(if st.scanning { hex_rgb(JUMP_BLUE) } else { TEXT_FAINT }).child(checked))
                .child(chip("topo-scan", "CHECK POSTURE".into(), false).on_click(move |_ev, _w, cx| app_scan.update(cx, |t, cx| t.check_posture(cx))))
                .child(icon_button("topo-zoom-out", TablerIcon::Minus, false).on_click(move |_ev, _w, cx| app_out.update(cx, |t, cx| t.topology_zoom(1.0 / 1.2, (300.0, 200.0), cx))))
                .child(div().min_w(px(36.0)).flex().justify_center().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_SECONDARY).child(format!("{:.0}%", st.zoom * 100.0)))
                .child(icon_button("topo-zoom-in", TablerIcon::Plus, false).on_click(move |_ev, _w, cx| app_in.update(cx, |t, cx| t.topology_zoom(1.2, (300.0, 200.0), cx))))
                .child(chip("topo-fit", "RESET VIEW".into(), false).on_click(move |_ev, _w, cx| app_fit.update(cx, |t, cx| t.topology_reset_view(cx)))),
        )
}

fn map_area(app_state: &CrowApp, graph: &Graph, placed: &Layout, app: Entity<CrowApp>) -> impl IntoElement {
    let st = &app_state.topology;
    let (pan, zoom) = (st.pan, st.zoom);
    let lane = st.lane.clone();
    let risky_only = st.risky_only;
    let selected = st.selected.clone();
    let screen = move |(x, y): (f32, f32)| (x * zoom + pan.0, y * zoom + pan.1);
    // The map's bounds, captured at paint, so wheel zoom can center on the cursor.
    let origin: Rc<Cell<(f32, f32)>> = Rc::new(Cell::new((0.0, 0.0)));

    // Lines: (from, to, kind, highlighted, dimmed).
    let lines: Vec<((f32, f32), (f32, f32), EdgeKind, bool, bool)> = graph
        .edges
        .iter()
        .filter_map(|e| {
            let (a, b) = (placed.at.get(&e.from)?, placed.at.get(&e.to)?);
            let (na, nb) = (graph.node(&e.from)?, graph.node(&e.to)?);
            let hi = selected.as_deref().is_some_and(|s| s == e.from || s == e.to);
            let dim = !(shown(na, lane.as_deref(), risky_only) && shown(nb, lane.as_deref(), risky_only));
            Some((screen((a.0 + NODE_W, a.1 + NODE_H / 2.0)), screen((b.0, b.1 + NODE_H / 2.0)), e.kind, hi, dim))
        })
        .collect();
    let paint_origin = origin.clone();
    let edges = canvas(
        move |bounds, _window, _cx| {
            paint_origin.set((bounds.origin.x.as_f32(), bounds.origin.y.as_f32()));
        },
        move |bounds, (), window, _cx| {
            let o = bounds.origin;
            for (from, to, kind, hi, dim) in &lines {
                let width = if *hi { 2.2 } else { 1.3 };
                let mut path = match kind {
                    EdgeKind::Unlocks | EdgeKind::Runs => PathBuilder::stroke(px(width)).dash_array(&[px(4.0), px(4.0)]),
                    _ => PathBuilder::stroke(px(width)),
                };
                let p0 = point(o.x + px(from.0), o.y + px(from.1));
                let p1 = point(o.x + px(to.0), o.y + px(to.1));
                let bend = ((to.0 - from.0).abs() * 0.45).max(30.0);
                path.move_to(p0);
                path.cubic_bezier_to(p1, point(p0.x + px(bend), p0.y), point(p1.x - px(bend), p1.y));
                // Key lines stay faint until their key or server is selected.
                let alpha = if *dim { 0.10 } else if *hi { 0.95 } else if *kind == EdgeKind::Unlocks { 0.22 } else { 0.45 };
                let color = match kind {
                    EdgeKind::Ssh => hex_rgba(0x9aa4b2, alpha),
                    EdgeKind::ViaJump => hex_rgba(JUMP_BLUE, alpha),
                    EdgeKind::Unlocks => hex_rgba(KEY_AMBER, alpha),
                    EdgeKind::Runs => hex_rgba(0x6c6f78, alpha),
                };
                if let Ok(p) = path.build() {
                    window.paint_path(p, color);
                }
            }
        },
    )
    .absolute()
    .inset_0();

    let (app_down, app_move, app_up, app_wheel) = (app.clone(), app.clone(), app.clone(), app.clone());
    let wheel_origin = origin.clone();
    div()
        .id("topology-map")
        .relative()
        .flex_1()
        .h_full()
        .overflow_hidden()
        .bg(BG_APP)
        .when(st.drag.is_some(), |d| d.cursor_grabbing())
        .on_mouse_down(MouseButton::Left, move |ev, _w, cx| {
            let at = (ev.position.x.as_f32(), ev.position.y.as_f32());
            app_down.update(cx, |t, cx| t.topology_drag_start(at, cx));
        })
        .on_mouse_move(move |ev, _w, cx| {
            if ev.pressed_button == Some(MouseButton::Left) {
                let at = (ev.position.x.as_f32(), ev.position.y.as_f32());
                app_move.update(cx, |t, cx| t.topology_drag_to(at, cx));
            }
        })
        .on_mouse_up(MouseButton::Left, move |ev, _w, cx| {
            let at = (ev.position.x.as_f32(), ev.position.y.as_f32());
            app_up.update(cx, |t, cx| t.topology_drag_end(Some(at), cx));
        })
        .on_scroll_wheel(move |ev: &ScrollWheelEvent, _w, cx| {
            let dy = match ev.delta {
                ScrollDelta::Lines(l) => l.y * 40.0,
                ScrollDelta::Pixels(p) => p.y.as_f32(),
            };
            if dy == 0.0 {
                return;
            }
            let (ox, oy) = wheel_origin.get();
            let at = (ev.position.x.as_f32() - ox, ev.position.y.as_f32() - oy);
            let factor = (dy / 400.0).exp();
            app_wheel.update(cx, |t, cx| t.topology_zoom(factor, at, cx));
        })
        .child(edges)
        .children(placed.lanes.iter().map(|(text, x, y)| {
            let (sx, sy) = screen((*x, *y));
            div().absolute().left(px(sx)).top(px(sy)).font_family(FONT_MONO).text_size(px(9.0 * zoom.max(0.8))).font_weight(FontWeight::BOLD).text_color(TEXT_FAINTER).child(text.clone())
        }))
        .children(graph.nodes.iter().enumerate().filter_map(|(i, n)| {
            let at = placed.at.get(&n.id)?;
            let (sx, sy) = screen(*at);
            Some(node_card(i, n, (sx, sy), zoom, selected.as_deref() == Some(n.id.as_str()), shown(n, lane.as_deref(), risky_only), app.clone()))
        }))
        .children(graph.nodes.len().le(&1).then(|| {
            div()
                .absolute()
                .left(px(pan.0 + 260.0))
                .top(px(pan.1 + 8.0))
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_FAINT)
                .child("No servers yet: enroll one, or launch a Multipass VM from LOCAL LAB & VMS.")
        }))
        .child(legend())
}

fn legend() -> impl IntoElement {
    let item = |color: Rgba, label: &'static str| div().flex().items_center().gap(px(5.0)).child(div().w(px(16.0)).h(px(2.0)).bg(color)).child(label);
    div()
        .absolute()
        .left(px(12.0))
        .bottom(px(10.0))
        .flex()
        .gap(px(14.0))
        .px(px(10.0))
        .py(px(5.0))
        .bg(hex_rgba(0x0a0b0e, 0.85))
        .border_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(9.0))
        .text_color(TEXT_TERTIARY)
        .child(item(hex_rgb(0x9aa4b2), "SSH"))
        .child(item(hex_rgb(JUMP_BLUE), "via jump host"))
        .child(item(hex_rgb(KEY_AMBER), "key unlocks"))
        .child(item(CRIT, "critical risk"))
        .child(item(WARN, "warning"))
}

fn node_card(i: usize, n: &Node, (x, y): (f32, f32), zoom: f32, selected: bool, shown: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let z = |v: f32| px(v * zoom);
    let worst = n.worst();
    let id = n.id.clone();
    let health = match n.health {
        Health::Ok => OK,
        Health::Down => CRIT,
        Health::Unknown => TEXT_DIMMER,
    };
    div()
        .id(ElementId::NamedInteger("topo-node".into(), i as u64))
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(z(NODE_W))
        .h(z(NODE_H))
        .flex()
        .bg(if selected { BG_NAV_ACTIVE } else { BG_PANEL })
        .border_1()
        .border_color(if selected { TEXT_PRIMARY } else { BORDER_DEFAULT })
        .when(!shown, |d| d.opacity(0.25))
        .cursor_pointer()
        .hover(|s| s.border_color(BORDER_STRONG))
        // A node takes its own clicks; the map behind would start a drag.
        .on_mouse_down(MouseButton::Left, |_ev, _w, cx| cx.stop_propagation())
        .on_click(move |_ev, _w, cx| {
            let id = id.clone();
            app.update(cx, |t, cx| t.topology_select(Some(id), cx))
        })
        .child(div().w(z(3.0)).h_full().flex_none().bg(level_color(worst)))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .justify_center()
                .gap(z(3.0))
                .px(z(9.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(z(6.0))
                        .text_color(if n.kind == NodeKind::Key { hex_rgb(KEY_AMBER) } else { TEXT_SECONDARY })
                        .child(inherited_icon(kind_icon(n.kind), z(13.0)))
                        .child(div().flex_1().min_w(px(0.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().font_family(FONT_MONO).text_size(z(11.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(n.label.clone()))
                        .children((n.kind != NodeKind::Key).then(|| div().size(z(7.0)).flex_none().rounded_full().bg(health))),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(z(6.0))
                        .child(div().flex_1().min_w(px(0.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().font_family(FONT_MONO).text_size(z(9.5)).text_color(TEXT_DIM).child(n.detail.clone()))
                        .children((!n.risks.is_empty()).then(|| {
                            div().flex_none().px(z(4.0)).bg(hex_rgba(if worst == Some(Level::Crit) { 0xef4444 } else { 0xf59e0b }, 0.15)).font_family(FONT_MONO).text_size(z(9.0)).font_weight(FontWeight::BOLD).text_color(level_color(worst)).child(format!("{} RISK{}", n.risks.len(), if n.risks.len() == 1 { "" } else { "S" }))
                        })),
                ),
        )
}

fn row(label: &'static str, value: String) -> impl IntoElement {
    div().flex().gap(px(8.0)).font_family(FONT_MONO).text_size(px(10.0)).child(div().w(px(96.0)).flex_none().text_color(TEXT_FAINT).child(label)).child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child(value))
}

fn heading(text: &'static str) -> impl IntoElement {
    div().pt(px(8.0)).font_family(FONT_MONO).text_size(px(9.5)).font_weight(FontWeight::BOLD).text_color(TEXT_TERTIARY).child(text)
}

/// The drawer for the selected node: what it is, its risks and where they
/// come from, how it connects, and the way into it.
fn inspector(app_state: &CrowApp, graph: &Graph, n: &Node, app: Entity<CrowApp>) -> impl IntoElement {
    let server = n.server_id.as_ref().and_then(|id| app_state.fleet.servers.iter().find(|s| &s.id == id));
    let metrics = n.server_id.as_ref().and_then(|id| app_state.fleet.metrics_store.get(id));
    let posture = n.server_id.as_ref().and_then(|id| app_state.topology.posture.get(id));
    let key = n.id.strip_prefix("key:").and_then(|id| app_state.keys.enrolled.iter().find(|k| k.id == id));
    let links: Vec<(String, String, EdgeKind)> = graph
        .edges_of(&n.id)
        .filter_map(|e| {
            let other = if e.from == n.id { &e.to } else { &e.from };
            graph.node(other).map(|o| (o.id.clone(), o.label.clone(), e.kind))
        })
        .collect();
    let app_close = app.clone();
    let mut body = div().flex().flex_col().gap(px(6.0)).p(px(14.0));

    // Risks first: they're why you'd click.
    body = body.child(heading(if n.risks.is_empty() { "NO RISKS FOUND" } else { "RISKS" }));
    for r in &n.risks {
        body = body.child(
            div()
                .flex()
                .gap(px(8.0))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .child(div().w(px(36.0)).flex_none().font_weight(FontWeight::BOLD).text_color(level_color(Some(r.level))).child(if r.level == Level::Crit { "CRIT" } else { "WARN" }))
                .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child(r.text.clone()))
                .child(div().flex_none().text_color(TEXT_FAINTER).child(r.source)),
        );
    }
    if let Some(s) = server {
        body = body.child(heading("SERVER"));
        body = body.child(row("address", n.detail.clone()));
        if !s.os_distro.is_empty() {
            body = body.child(row("os", s.os_distro.clone()));
        }
        if !s.os_kernel.is_empty() {
            body = body.child(row("kernel", s.os_kernel.clone()));
        }
        body = body.child(row("env · group", format!("{} · {}", if s.env.is_empty() { "—" } else { &s.env }, if s.group_name.is_empty() { "—" } else { &s.group_name })));
        body = body.child(row("host key", s.host_key_fingerprint.clone().unwrap_or_else(|| "not pinned".into())));
        if let Some(m) = metrics.filter(|m| m.reachable) {
            body = body.child(row("load", format!("cpu {:.0}% · mem {:.0}% · disk {:.0}%", m.cpu_pct, m.mem_pct, m.disk_pct)));
            if !m.uptime_formatted.is_empty() {
                body = body.child(row("uptime", m.uptime_formatted.clone()));
            }
        }
        body = body.child(heading("POSTURE"));
        match posture {
            None => body = body.child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_FAINT).child("Not checked yet (CHECK POSTURE reads it on reachable servers).")),
            Some(p) => {
                let or_unknown = |v: Option<String>| v.unwrap_or_else(|| "couldn't read (needs root)".into());
                body = body
                    .child(row("root login", or_unknown(p.root_login.clone())))
                    .child(row("passwords", or_unknown(p.password_auth.map(|b| if b { "accepted".into() } else { "refused".into() }))))
                    .child(row("firewall", or_unknown(p.firewall.clone().map(|f| if f.is_empty() { "none".into() } else { f }))))
                    .child(row("open to all", if p.exposed_ports.is_empty() { "nothing".into() } else { p.exposed_ports.iter().map(u16::to_string).collect::<Vec<_>>().join(", ") }));
            }
        }
    }
    if let Some(k) = key {
        body = body.child(heading("KEY")).child(row("algorithm", k.algorithm.clone())).child(row("fingerprint", k.fingerprint.clone())).child(row("created", k.created_at.chars().take(10).collect()));
        body = body.child(row("private key", k.private_key_path.clone().unwrap_or_else(|| "not on this machine".into())));
    }
    if !links.is_empty() {
        body = body.child(heading("CONNECTED TO"));
        for (i, (id, label, kind)) in links.into_iter().enumerate() {
            let app = app.clone();
            let how = match kind {
                EdgeKind::Ssh => "ssh",
                EdgeKind::ViaJump => "via jump",
                EdgeKind::Unlocks => "key",
                EdgeKind::Runs => "runs",
            };
            body = body.child(
                div()
                    .id(ElementId::NamedInteger("topo-link".into(), i as u64))
                    .flex()
                    .gap(px(8.0))
                    .px(px(4.0))
                    .py(px(1.0))
                    .font_family(FONT_MONO)
                    .text_size(px(10.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _w, cx| {
                        let id = id.clone();
                        app.update(cx, |t, cx| t.topology_select(Some(id), cx))
                    })
                    .child(div().w(px(60.0)).flex_none().text_color(TEXT_FAINT).child(how))
                    .child(div().text_color(TEXT_PRIMARY).child(label)),
            );
        }
    }

    let actions = server.map(|s| {
        let (app_open, app_term, id_open, id_term) = (app.clone(), app.clone(), s.id.clone(), s.id.clone());
        div()
            .flex()
            .gap(px(8.0))
            .p(px(14.0))
            .border_t_1()
            .border_color(BORDER_PANEL)
            .child(chip("topo-open-server", "OPEN SERVER".into(), true).on_click(move |_ev, _w, cx| {
                let id = id_open.clone();
                app_open.update(cx, |t, cx| t.topology_open_server(&id, "overview", cx))
            }))
            .child(chip("topo-open-terminal", "TERMINAL".into(), false).on_click(move |_ev, _w, cx| {
                let id = id_term.clone();
                app_term.update(cx, |t, cx| t.topology_open_server(&id, "terminal", cx))
            }))
    });

    div()
        .w(px(360.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_PANEL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px(px(14.0))
                .py(px(10.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(div().font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_FAINT).child(kind_label(n.kind)))
                        .child(div().font_family(FONT_MONO).text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(n.label.clone())),
                )
                .child(icon_button("topo-inspector-close", TablerIcon::X, false).on_click(move |_ev, _w, cx| app_close.update(cx, |t, cx| t.topology_select(None, cx)))),
        )
        .child(div().id("topo-inspector-body").flex_1().min_h(px(0.0)).overflow_y_scrollbar().child(body))
        .children(actions)
}
