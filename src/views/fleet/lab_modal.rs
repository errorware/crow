use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::components::icon_button::icon_button;
use crate::components::icons::TablerIcon;
use crate::app::CrowApp;
use crate::lab::multipass::{vm_of, InstallStep, Instance, Lifecycle, MultipassStatus};
use crate::lab::LocalTestNode;
use crate::vault::ServerRecord;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::views::fleet::lab_state::LocalLabState;

/// The lab window: only what's switched on in Settings → Plugins (ERR-138):
/// Multipass VMs, and the containers of the enabled container plugins.
pub fn local_lab_modal(
    nodes: &[LocalTestNode],
    app: Entity<CrowApp>,
    local_lab: &LocalLabState,
    servers: &[ServerRecord],
    plugins: &crate::app::plugins::PluginsState,
) -> impl IntoElement {
    let app_close_scrim = app.clone();
    let app_close_btn = app.clone();
    let app_refresh = app.clone();
    let multipass_on = plugins.enabled.contains("multipass");
    let containers: Vec<&'static crate::plugins::BuiltinPlugin> = crate::plugins::CONTAINER_PLUGINS.iter().filter(|id| plugins.enabled.contains(**id)).filter_map(|id| crate::plugins::get(id)).collect();
    let lab_on = !containers.is_empty();
    let (title, subtitle) = match (multipass_on, lab_on) {
        (true, true) => ("LOCAL VMS & CONTAINERS", "Multipass VMs and lab containers on this machine, enrolled into Crow"),
        (true, false) => ("MULTIPASS VMS", "Ubuntu VMs on this machine: launch, import and power them"),
        _ => ("LOCAL LAB CONTAINERS", "Lab containers on this machine, enrolled into Crow as test nodes"),
    };

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
                .w(px(760.0))
                // A height to fill: with only a max, the scrolling body
                // (flex_1) had nothing to grow into and showed empty.
                .h(px(720.0))
                .max_h(relative(0.9))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // keep clicks inside from reaching the backdrop (which closes)
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
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_TERTIARY)
                                        .child(subtitle),
                                ),
                        )
                        .child(
                            icon_button("btn-close-lab-modal", TablerIcon::X, false)
                                .on_click(move |_ev, _window, cx| {
                                    app_close_btn.update(cx, |this, cx| {
                                        this.toggle_local_lab_modal(cx);
                                    });
                                }),
                        ),
                )
                // 2. Scrollable Body
                .child(
                    div()
                        .id("lab-modal-scroll")
                        .flex_1()
                        .min_h(px(0.0))
                        .overflow_y_scrollbar()
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(20.0))
                        // A. The enabled container plugins and whether they work.
                        .children(lab_on.then(|| {
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(8.0))
                                .children(containers.iter().map(|p| {
                                    let (text, color) = match plugins.status.get(p.id) {
                                        _ if plugins.checking.contains(p.id) => ("checking…".to_string(), TEXT_DIMMER),
                                        Some(crate::plugins::PluginStatus::Ready(v)) => (v.clone(), OK),
                                        Some(crate::plugins::PluginStatus::Problem { summary, .. }) => (summary.clone(), WARN),
                                        Some(crate::plugins::PluginStatus::Missing { .. }) => ("not installed".to_string(), TEXT_DIMMER),
                                        _ => ("—".to_string(), TEXT_DIMMER),
                                    };
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(8.0))
                                        .py(px(4.0))
                                        .bg(BG_CONTROL)
                                        .border_1()
                                        .border_color(BORDER_PANEL)
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .child(div().size(px(6.0)).rounded_full().bg(color))
                                        .child(div().font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(p.name))
                                        .child(div().text_color(color).child(text))
                                }))
                        }))
                        // B. Multipass VMs (ERR-119), when its plugin is on.
                        .children(multipass_on.then(|| multipass_section(local_lab, servers, app.clone())))
                        // C. Discovered lab containers, when a container plugin is on.
                        .children(lab_on.then(||
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
                        ))
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

// ==========================================
// Multipass VMs (ERR-119)
// ==========================================

fn section_title(text: &'static str) -> Div {
    div().font_family(FONT_MONO).text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(text)
}

fn pill(text: String, fg: Rgba, bg: Rgba) -> Div {
    div().px(px(5.0)).py(px(1.0)).bg(bg).font_family(FONT_MONO).text_size(px(8.5)).font_weight(FontWeight::BOLD).text_color(fg).child(text)
}

/// A small text button whose label is built at render time.
fn text_button(id: impl Into<ElementId>, label: String, color: Rgba) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(7.0))
        .py(px(2.0))
        .rounded_sm()
        .border_1()
        .border_color(BORDER_DEFAULT)
        .bg(BG_CONTROL)
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .text_color(color)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(label)
}

/// Status, setup steps, the VMs Multipass has (with import and lifecycle
/// actions) and the launch form.
pub fn multipass_section(lab: &LocalLabState, servers: &[ServerRecord], app: Entity<CrowApp>) -> impl IntoElement {
    let (status_text, status_fg, status_bg, steps): (String, Rgba, Rgba, &[InstallStep]) = match &lab.multipass {
        None => ("CHECKING…".into(), TEXT_DIMMER, BG_CHIP, &[]),
        Some(MultipassStatus::Missing { steps }) => ("NOT INSTALLED".into(), TEXT_DIMMER, BG_CHIP, steps),
        Some(MultipassStatus::DaemonDown { steps, .. }) => ("DAEMON NOT RUNNING".into(), WARN, hex_rgba(0xf59e0b, 0.12), steps),
        Some(MultipassStatus::Ready { daemon, fixes, .. }) if !fixes.is_empty() => (format!("{daemon} · NEEDS A FIX"), WARN, hex_rgba(0xf59e0b, 0.12), fixes),
        Some(MultipassStatus::Ready { daemon, .. }) => (format!("READY · {daemon}"), OK, OK_BG, &[]),
    };
    let ready = lab.multipass.as_ref().is_some_and(|s| s.is_ready());
    let detail = match &lab.multipass {
        Some(MultipassStatus::DaemonDown { detail, .. }) => Some(detail.clone()),
        _ => None,
    };
    let app_refresh = app.clone();

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().flex().items_center().gap(px(8.0)).child(section_title("MULTIPASS VMS")).child(pill(status_text, status_fg, status_bg)))
                .child(icon_button("btn-refresh-multipass", TablerIcon::Refresh, false).on_click(move |_ev, _window, cx| {
                    app_refresh.update(cx, |this, cx| this.refresh_multipass(cx));
                })),
        )
        .child(div().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_TERTIARY).child(
            "Ubuntu VMs on this machine. Crow launches them with its own key, pins their host keys through Multipass, and adds them to the fleet as SSH servers.",
        ))
        .children(detail.map(|d| div().font_family(FONT_MONO).text_size(px(9.5)).text_color(WARN).child(d)))
        .children(steps.iter().enumerate().map(|(i, s)| setup_step(i, s, app.clone())))
        .children(lab.busy.clone().map(|b| div().font_family(FONT_MONO).text_size(px(10.0)).text_color(hex_rgb(0x8ab4ff)).child(format!("● {b}"))))
        .children(lab.vm_error.clone().map(|e| div().font_family(FONT_MONO).text_size(px(10.0)).text_color(CRIT).child(e)))
        .children(ready.then(|| {
            div()
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .children(lab.vms.is_empty().then(|| div().p(px(14.0)).font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_FAINT).child("No VMs yet: launch one below.")))
                .children(lab.vms.iter().enumerate().map(|(i, vm)| vm_row(i, vm, servers, lab, app.clone())))
        }))
        .children(ready.then(|| launch_form(lab, app.clone())))
}

pub(crate) fn setup_step(i: usize, s: &InstallStep, app: Entity<CrowApp>) -> impl IntoElement {
    let command = s.command.clone();
    div()
        .flex()
        .flex_col()
        .gap(px(3.0))
        .p(px(8.0))
        .bg(BG_CONTROL)
        .border_1()
        .border_color(BORDER_PANEL)
        .child(div().font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_SECONDARY).child(format!("{}. {}", i + 1, s.why)))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_1().min_w(px(0.0)).font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_PRIMARY).child(s.command.clone()))
                .child(text_button(ElementId::NamedInteger("mp-copy-step".into(), i as u64), "COPY".into(), TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    let c = command.clone();
                    app.update(cx, |this, cx| this.copy_text_with_toast(&c, "Command copied", cx));
                })),
        )
}

fn vm_row(i: usize, vm: &Instance, servers: &[ServerRecord], lab: &LocalLabState, app: Entity<CrowApp>) -> impl IntoElement {
    let in_fleet = servers.iter().any(|s| vm_of(s) == Some(vm.name.as_str()));
    let running = vm.is_running();
    let idle = lab.busy.is_none();
    let confirming = lab.confirm_purge.as_deref() == Some(vm.name.as_str());
    let action = |n: u64, label: &str, color: Rgba, act: Lifecycle| {
        let (app, name) = (app.clone(), vm.name.clone());
        text_button(ElementId::NamedInteger(format!("mp-{label}").into(), i as u64 * 10 + n), label.to_string(), color).on_click(move |_ev, _window, cx| {
            let name = name.clone();
            app.update(cx, |this, cx| this.run_vm_lifecycle(act, &name, cx));
        })
    };
    let mut actions = div().flex().items_center().gap(px(6.0));
    if idle {
        if in_fleet {
            actions = actions.child(pill("IN FLEET".into(), OK, OK_BG));
        } else if running {
            let (app_i, name) = (app.clone(), vm.name.clone());
            actions = actions.child(text_button(ElementId::NamedInteger("mp-import".into(), i as u64), "IMPORT TO CROW".into(), TEXT_PRIMARY).on_click(move |_ev, _window, cx| {
                let name = name.clone();
                app_i.update(cx, |this, cx| this.import_multipass_vm(&name, cx));
            }));
        }
        actions = if running {
            actions.child(action(1, "STOP", TEXT_SECONDARY, Lifecycle::Stop)).child(action(2, "SUSPEND", TEXT_SECONDARY, Lifecycle::Suspend)).child(action(3, "RESTART", TEXT_SECONDARY, Lifecycle::Restart))
        } else {
            actions.child(action(4, "START", OK, Lifecycle::Start))
        };
        let (app_p, name) = (app.clone(), vm.name.clone());
        actions = actions.child(icon_button(ElementId::NamedInteger("mp-purge".into(), i as u64), TablerIcon::Trash, true).on_click(move |_ev, _window, cx| {
            let name = name.clone();
            app_p.update(cx, |this, cx| this.run_vm_lifecycle(Lifecycle::Purge, &name, cx));
        }));
    }
    let (app_yes, app_no, name_yes) = (app.clone(), app.clone(), vm.name.clone());
    div()
        .flex()
        .flex_col()
        .border_b_1()
        .border_color(BORDER_ROW)
        .child(
            div()
                .p(px(10.0))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(div().size(px(8.0)).rounded_full().bg(if running { OK } else { TEXT_DIMMER }))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(div().font_family(FONT_MONO).text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(vm.name.clone()))
                                .child(div().font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_MUTED).child(format!(
                                    "{} · {} · {}",
                                    vm.state,
                                    vm.address().unwrap_or("no address"),
                                    if vm.release.is_empty() { "—" } else { vm.release.as_str() }
                                ))),
                        ),
                )
                .child(actions),
        )
        .children(confirming.then(|| {
            div()
                .px(px(10.0))
                .pb(px(10.0))
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_1().font_family(FONT_MONO).text_size(px(10.0)).text_color(CRIT).child(format!(
                    "Delete {} and its disk for good?{}",
                    vm.name,
                    if in_fleet { " Its fleet entry goes to the archive." } else { "" }
                )))
                .child(text_button(ElementId::NamedInteger("mp-purge-yes".into(), i as u64), "DELETE FOR GOOD".into(), CRIT).on_click(move |_ev, _window, cx| {
                    let name = name_yes.clone();
                    app_yes.update(cx, |this, cx| this.run_vm_lifecycle(Lifecycle::Purge, &name, cx));
                }))
                .child(text_button(ElementId::NamedInteger("mp-purge-no".into(), i as u64), "CANCEL".into(), TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    app_no.update(cx, |this, cx| this.cancel_vm_purge(cx));
                }))
        }))
}

/// `label  [−] value [+]`.
fn stepper(id: &'static str, label: &'static str, value: String, down: impl Fn(&mut CrowApp, &mut Context<CrowApp>) + 'static, up: impl Fn(&mut CrowApp, &mut Context<CrowApp>) + 'static, app: Entity<CrowApp>) -> impl IntoElement {
    let app_up = app.clone();
    div()
        .flex()
        .items_center()
        .gap(px(4.0))
        .child(div().font_family(FONT_MONO).text_size(px(9.0)).text_color(TEXT_TERTIARY).child(label))
        .child(icon_button(ElementId::Name(format!("{id}-down").into()), TablerIcon::Minus, false).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| down(this, cx))))
        .child(div().min_w(px(34.0)).flex().justify_center().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_PRIMARY).child(value))
        .child(icon_button(ElementId::Name(format!("{id}-up").into()), TablerIcon::Plus, false).on_click(move |_ev, _window, cx| app_up.update(cx, |this, cx| up(this, cx))))
}

fn launch_form(lab: &LocalLabState, app: Entity<CrowApp>) -> impl IntoElement {
    let l = lab.launch.clone();
    let idle = lab.busy.is_none();
    let (app_name, app_go) = (app.clone(), app.clone());
    let (c, m, d) = (l.cpus, l.memory_gb, l.disk_gb);
    div()
        .p(px(12.0))
        .bg(BG_CONTROL)
        .border_1()
        .border_color(BORDER_DEFAULT)
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(section_title("LAUNCH A VM"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_PRIMARY).child(l.name.clone()))
                        .child(icon_button("btn-reroll-vm-name", TablerIcon::Refresh, false).on_click(move |_ev, _window, cx| app_name.update(cx, |this, cx| this.reroll_vm_name(cx)))),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(16.0))
                .child(stepper("vm-cpu", "CPU", c.to_string(), move |a, cx| a.set_vm_size(c.saturating_sub(1), m, d, cx), move |a, cx| a.set_vm_size(c + 1, m, d, cx), app.clone()))
                .child(stepper("vm-mem", "MEMORY", format!("{m} GB"), move |a, cx| a.set_vm_size(c, m.saturating_sub(1), d, cx), move |a, cx| a.set_vm_size(c, m + 1, d, cx), app.clone()))
                .child(stepper("vm-disk", "DISK", format!("{d} GB"), move |a, cx| a.set_vm_size(c, m, d.saturating_sub(5), cx), move |a, cx| a.set_vm_size(c, m, d + 5, cx), app.clone())),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().flex().items_center().gap(px(6.0)).children([("lts", "Newest LTS"), ("24.04", "24.04"), ("22.04", "22.04")].into_iter().enumerate().map(|(i, (image, label))| {
                    let selected = l.image == image;
                    let app_img = app.clone();
                    text_button(ElementId::NamedInteger("vm-image".into(), i as u64), label.to_string(), if selected { TEXT_PRIMARY } else { TEXT_TERTIARY })
                        .when(selected, |b| b.border_color(TEXT_PRIMARY).bg(BG_NAV_ACTIVE))
                        .on_click(move |_ev, _window, cx| app_img.update(cx, |this, cx| this.set_vm_image(image, cx)))
                })))
                .child(
                    div()
                        .id("btn-launch-vm")
                        .px(px(14.0))
                        .py(px(6.0))
                        .bg(if idle { TEXT_PRIMARY } else { BG_CHIP })
                        .text_color(if idle { rgb(0x0a0a0c) } else { TEXT_DIMMER })
                        .font_weight(FontWeight::BOLD)
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .when(idle, |b| b.cursor_pointer().hover(|s| s.bg(hex_rgb(0xffffff))))
                        .on_click(move |_ev, _window, cx| app_go.update(cx, |this, cx| this.launch_multipass_vm(cx)))
                        .child("LAUNCH & ADD TO FLEET"),
                ),
        )
}
