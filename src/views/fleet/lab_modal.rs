use gpui_kit::*;
use crate::app::CrowApp;
use crate::lab::{EngineStatus, LocalTestNode};
use crate::theme::*;
use crate::views::fleet::lab_state::LocalLabState;

pub fn local_lab_modal(
    engines: &[EngineStatus],
    nodes: &[LocalTestNode],
    app: Entity<CrowApp>,
    local_lab: &LocalLabState,
) -> impl IntoElement {
    let app_close_scrim = app.clone();
    let app_close_btn = app.clone();
    let app_refresh = app.clone();
    let app_create = app.clone();

    div()
        .id("local-lab-modal-scrim")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x060709, 0.85))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_close_scrim.update(cx, |this, cx| {
                this.toggle_local_lab_modal(cx);
            });
        })
        .child(
            // Modal Card (prevent clicks from bubbling to scrim)
            div()
                .id("local-lab-modal-card")
                .w(px(700.0))
                .max_h(px(720.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .on_click(|_ev, _window, _cx| {})
                // 1. Header
                .child(
                    div()
                        .h(px(48.0))
                        .flex_none()
                        .px(px(20.0))
                        .bg(BG_SUBHEAD)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("LOCAL TEST LAB & VMS"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_TERTIARY)
                                        .child("Discover, launch, and enroll Distrobox & Podman environments into Crow"),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-close-lab-modal")
                                .size(px(24.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(TEXT_MUTED)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(14.0))
                                .on_click(move |_ev, _window, cx| {
                                    app_close_btn.update(cx, |this, cx| {
                                        this.toggle_local_lab_modal(cx);
                                    });
                                })
                                .child("✕"),
                        ),
                )
                // 2. Scrollable Body
                .child(
                    div()
                        .id("lab-modal-scroll")
                        .flex_1()
                        .min_h(px(0.0))
                        .overflow_y_scroll()
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(20.0))
                        // A. Engine Status Cards
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_TERTIARY)
                                        .child("LOCAL VIRTUALIZATION / CONTAINER ENGINES"),
                                )
                                .child(
                                    div()
                                        .grid()
                                        .grid_cols(4)
                                        .gap(px(8.0))
                                        .children(engines.iter().map(|eng| {
                                            render_engine_card(eng)
                                        }))
                                )
                        )
                        // B. Discovered Local Nodes Table
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(TEXT_PRIMARY)
                                                        .child("DISCOVERED LOCAL TEST NODES"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.5))
                                                        .bg(BG_CHIP)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(TEXT_SECONDARY)
                                                        .child(format!("{} detected", nodes.len())),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-refresh-lab-nodes")
                                                .px(px(8.0))
                                                .py(px(3.0))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .text_color(TEXT_MUTED)
                                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                                .cursor_pointer()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_refresh.update(cx, |this, cx| {
                                                        this.refresh_lab_nodes(cx);
                                                    });
                                                })
                                                .child("REFRESH 🔄"),
                                        ),
                                )
                                .child(
                                    div()
                                        .bg(BG_APP)
                                        .border_1()
                                        .border_color(BORDER_PANEL)
                                        .flex()
                                        .flex_col()
                                        .children(if nodes.is_empty() {
                                            vec![
                                                div()
                                                    .p(px(24.0))
                                                    .flex()
                                                    .flex_col()
                                                    .items_center()
                                                    .justify_center()
                                                    .gap(px(6.0))
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .text_color(TEXT_MUTED)
                                                            .child("No local test nodes found"),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(9.5))
                                                            .text_color(TEXT_FAINT)
                                                            .child("Use the form below to spin up a fresh Ubuntu or Debian test node"),
                                                    )
                                                    .into_any_element()
                                            ]
                                        } else {
                                            nodes.iter().enumerate().map(|(idx, n)| {
                                                render_node_row(n, idx, app.clone()).into_any_element()
                                            }).collect()
                                        })
                                )
                        )
                        // C. Spin Up New Test Node Form
                        .child(
                            div()
                                .p(px(14.0))
                                .bg(hex_rgb(0x0f1016))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
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
                                                .text_size(px(11.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_PRIMARY)
                                                .child("⚡ SPIN UP A LOCAL TEST NODE"),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_DIMMER)
                                                .child("Instantly creates an isolated Linux environment with SSH"),
                                        ),
                                )
                                // Presets selection
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .children([
                                            ("Ubuntu 24.04", "noble"),
                                            ("Debian 12", "bookworm"),
                                            ("Alpine 3.20", "alpine"),
                                            ("Fedora 40", "fedora"),
                                        ].into_iter().enumerate().map(|(idx, (label, val))| {
                                            let is_sel = local_lab.new_node_distro == val;
                                            let app_d = app.clone();
                                            div()
                                                .id(ElementId::NamedInteger("preset-distro".into(), idx as u64))
                                                .flex_1()
                                                .py(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                                .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_d.update(cx, |this, cx| {
                                                        this.set_new_lab_distro(val, cx);
                                                    });
                                                })
                                                .child(label)
                                        }))
                                )
                                // Port & Engine options
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .text_color(TEXT_MUTED)
                                                        .child("SSH Port: 127.0.0.1:2222 · Engine: Podman/Distrobox"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-spin-up-lab-node")
                                                .px(px(14.0))
                                                .py(px(6.0))
                                                .bg(TEXT_PRIMARY)
                                                .text_color(rgb(0x0a0a0c))
                                                .font_weight(FontWeight::BOLD)
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(hex_rgb(0xffffff)))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_create.update(cx, |this, cx| {
                                                        this.create_lab_node(cx);
                                                    });
                                                })
                                                .child("⚡ LAUNCH & ENROLL INTO CROW"),
                                        ),
                                )
                        )
                )
                // 3. Footer
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .px(px(20.0))
                        .bg(BG_SUBHEAD)
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_end()
                        .child(
                            div()
                                .id("btn-close-lab-footer")
                                .px(px(14.0))
                                .py(px(5.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_MUTED)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .on_click(move |_ev, _window, cx| {
                                    app.update(cx, |this, cx| {
                                        this.toggle_local_lab_modal(cx);
                                    });
                                })
                                .child("CLOSE"),
                        ),
                ),
        )
}

fn render_engine_card(eng: &EngineStatus) -> impl IntoElement {
    let is_avail = eng.is_available;
    let pill_bg = if is_avail { OK_BG } else { BG_CHIP };
    let pill_fg = if is_avail { OK } else { TEXT_DIMMER };
    let border_color = if is_avail { BORDER_DEFAULT } else { BORDER_PANEL };

    div()
        .p(px(8.0))
        .bg(BG_CONTROL)
        .border_1()
        .border_color(border_color)
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(eng.engine.label()),
                )
                .child(
                    div()
                        .px(px(4.0))
                        .py(px(1.0))
                        .bg(pill_bg)
                        .font_family(FONT_MONO)
                        .text_size(px(8.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pill_fg)
                        .child(if is_avail { "OK" } else { "OFF" }),
                ),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(8.5))
                .text_color(if is_avail { TEXT_MUTED } else { TEXT_DIMMER })
                .child(eng.version.clone()),
        )
}

fn render_node_row(node: &LocalTestNode, idx: usize, app: Entity<CrowApp>) -> impl IntoElement {
    let is_running = node.is_running();
    let is_enrolled = node.is_enrolled;
    let node_name = node.name.clone();
    let node_id = node.id.clone();
    let app_start = app.clone();
    let app_stop = app.clone();
    let app_enroll = app.clone();

    div()
        .p(px(10.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .flex()
        .items_center()
        .justify_between()
        .hover(|s| s.bg(BG_ROW_HOVER))
        // Left info
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .size(px(8.0))
                        .rounded_full()
                        .bg(if is_running { OK } else { TEXT_DIMMER }),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(node.name.clone()),
                                )
                                .child(
                                    div()
                                        .px(px(4.5))
                                        .py(px(1.0))
                                        .bg(BG_CHIP)
                                        .font_family(FONT_MONO)
                                        .text_size(px(8.5))
                                        .text_color(TEXT_TERTIARY)
                                        .child(node.engine.label()),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.0))
                                .text_color(TEXT_MUTED)
                                .child(format!("{} · Endpoint: {}", node.distro_display(), node.endpoint_display())),
                        ),
                ),
        )
        // Right Action Buttons
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                // Start / Stop
                .child(
                    if is_running {
                        let nid = node_id.clone();
                        div()
                            .id(ElementId::NamedInteger("node-stop-btn".into(), idx as u64))
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(BG_CONTROL)
                            .border_1()
                            .border_color(BORDER_DEFAULT)
                            .text_color(TEXT_MUTED)
                            .hover(|s| s.text_color(CRIT).bg(BG_ROW_HOVER))
                            .cursor_pointer()
                            .font_family(FONT_MONO)
                            .text_size(px(9.5))
                            .on_click(move |_ev, _window, cx| {
                                let id = nid.clone();
                                app_stop.update(cx, |this, cx| {
                                    this.stop_lab_node(&id, cx);
                                });
                            })
                            .child("■ STOP")
                    } else {
                        let nid = node_id.clone();
                        div()
                            .id(ElementId::NamedInteger("node-start-btn".into(), idx as u64))
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(BG_CONTROL)
                            .border_1()
                            .border_color(BORDER_DEFAULT)
                            .text_color(OK)
                            .hover(|s| s.bg(BG_ROW_HOVER))
                            .cursor_pointer()
                            .font_family(FONT_MONO)
                            .text_size(px(9.5))
                            .on_click(move |_ev, _window, cx| {
                                let id = nid.clone();
                                app_start.update(cx, |this, cx| {
                                    this.start_lab_node(&id, cx);
                                });
                            })
                            .child("▶ START")
                    }
                )
                // Enroll button
                .child(
                    if is_enrolled {
                        div()
                            .id(ElementId::NamedInteger("node-enrolled-pill".into(), idx as u64))
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(OK_BG)
                            .border_1()
                            .border_color(OK)
                            .font_family(FONT_MONO)
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(OK)
                            .child("✓ ENROLLED")
                    } else {
                        let n_name = node_name.clone();
                        div()
                            .id(ElementId::NamedInteger("node-enroll-btn".into(), idx as u64))
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(TEXT_PRIMARY)
                            .text_color(rgb(0x0a0a0c))
                            .font_family(FONT_MONO)
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(9.5))
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0xffffff)))
                            .on_click(move |_ev, _window, cx| {
                                let name = n_name.clone();
                                app_enroll.update(cx, |this, cx| {
                                    this.enroll_lab_node(&name, cx);
                                });
                            })
                            .child("+ ENROLL IN CROW")
                    }
                ),
        )
}
