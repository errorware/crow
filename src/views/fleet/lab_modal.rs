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
        (true, true) => (t!("fleet-lab-modal-local-vms-containers"), t!("fleet-lab-modal-multipass-vms-and-lab-containers")),
        (true, false) => (t!("fleet-lab-modal-multipass-vms"), t!("fleet-lab-modal-ubuntu-vms-on-this-machine")),
        _ => (t!("fleet-lab-modal-local-lab-containers"), t!("fleet-lab-modal-lab-containers-on-this-machine")),
    };

    div()
        .id("local-lab-modal-scrim")
        .absolute()
        .inset_0()
        // Blocks the page underneath: no hover or clicks through.
        .occlude()
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
                .bg(bg_panel())
                .border_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // keep clicks inside from reaching the backdrop (which closes)
                // 1. Header
                .child(
                    div()
                        .h(px(48.0))
                        .flex_none()
                        .px(px(20.0))
                        .bg(bg_subhead())
                        .border_b_1()
                        .border_color(border_panel())
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
                                        .text_size(font_px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_primary())
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(9.5))
                                        .text_color(text_tertiary())
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
                                        _ if plugins.checking.contains(p.id) => (t!("fleet-lab-modal-checking").to_string(), text_dimmer()),
                                        Some(crate::plugins::PluginStatus::Ready(v)) => (v.clone(), theme().ok),
                                        Some(crate::plugins::PluginStatus::Problem { summary, .. }) => (summary.clone(), theme().warn),
                                        Some(crate::plugins::PluginStatus::Missing { .. }) => (t!("fleet-lab-modal-not-installed").to_string(), text_dimmer()),
                                        _ => ("—".to_string(), text_dimmer()),
                                    };
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(8.0))
                                        .py(px(4.0))
                                        .bg(bg_control())
                                        .border_1()
                                        .border_color(border_panel())
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(10.0))
                                        .child(div().size(px(6.0)).rounded_full().bg(color))
                                        .child(div().font_weight(FontWeight::BOLD).text_color(text_primary()).child(p.name))
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
                                                        .text_size(font_px(10.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(text_primary())
                                                        .child(t!("fleet-lab-modal-discovered-local-test-nodes")),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.5))
                                                        .bg(bg_chip())
                                                        .font_family(FONT_MONO)
                                                        .text_size(font_px(9.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(text_secondary())
                                                        .child(t!("fleet-lab-modal-detected", count = nodes.len())),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-refresh-lab-nodes")
                                                .px(px(8.0))
                                                .py(px(3.0))
                                                .bg(bg_control())
                                                .border_1()
                                                .border_color(border_default())
                                                .text_color(text_muted())
                                                .hover(|s| s.text_color(text_primary()).bg(bg_row_hover()))
                                                .cursor_pointer()
                                                .font_family(FONT_MONO)
                                                .text_size(font_px(9.5))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_refresh.update(cx, |this, cx| {
                                                        this.refresh_lab_nodes(cx);
                                                    });
                                                })
                                                .child(t!("fleet-lab-modal-refresh")),
                                        ),
                                )
                                .child(
                                    div()
                                        .bg(bg_app())
                                        .border_1()
                                        .border_color(border_panel())
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
                                                            .text_size(font_px(11.0))
                                                            .text_color(text_muted())
                                                            .child(t!("fleet-lab-modal-no-local-test-nodes-found")),
                                                    )
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(font_px(9.5))
                                                            .text_color(text_faint())
                                                            .child(t!("fleet-lab-modal-use-the-form-below-to")),
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
                        .bg(bg_subhead())
                        .border_t_1()
                        .border_color(border_panel())
                        .flex()
                        .items_center()
                        .justify_end()
                        .child(
                            div()
                                .id("btn-close-lab-footer")
                                .px(px(14.0))
                                .py(px(5.0))
                                .bg(bg_control())
                                .border_1()
                                .border_color(border_default())
                                .text_color(text_muted())
                                .hover(|s| s.text_color(text_primary()).bg(bg_row_hover()))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .on_click(move |_ev, _window, cx| {
                                    app.update(cx, |this, cx| {
                                        this.toggle_local_lab_modal(cx);
                                    });
                                })
                                .child(t!("fleet-lab-modal-close")),
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
        .border_color(border_row())
        .flex()
        .items_center()
        .justify_between()
        .hover(|s| s.bg(bg_row_hover()))
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
                        .bg(if is_running { theme().ok } else { text_dimmer() }),
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
                                        .text_size(font_px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_primary())
                                        .child(node.name.clone()),
                                )
                                .child(
                                    div()
                                        .px(px(4.5))
                                        .py(px(1.0))
                                        .bg(bg_chip())
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(8.5))
                                        .text_color(text_tertiary())
                                        .child(node.engine.label()),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.0))
                                .text_color(text_muted())
                                .child(t!("fleet-lab-modal-endpoint", distro_display = node.distro_display(), endpoint_display = node.endpoint_display())),
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
                            .bg(bg_control())
                            .border_1()
                            .border_color(border_default())
                            .text_color(text_muted())
                            .hover(|s| s.text_color(theme().crit).bg(bg_row_hover()))
                            .cursor_pointer()
                            .font_family(FONT_MONO)
                            .text_size(font_px(9.5))
                            .on_click(move |_ev, _window, cx| {
                                let id = nid.clone();
                                app_stop.update(cx, |this, cx| {
                                    this.stop_lab_node(&id, cx);
                                });
                            })
                            .child(t!("fleet-lab-modal-stop"))
                    } else {
                        let nid = node_id.clone();
                        div()
                            .id(ElementId::NamedInteger("node-start-btn".into(), idx as u64))
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(bg_control())
                            .border_1()
                            .border_color(border_default())
                            .text_color(theme().ok)
                            .hover(|s| s.bg(bg_row_hover()))
                            .cursor_pointer()
                            .font_family(FONT_MONO)
                            .text_size(font_px(9.5))
                            .on_click(move |_ev, _window, cx| {
                                let id = nid.clone();
                                app_start.update(cx, |this, cx| {
                                    this.start_lab_node(&id, cx);
                                });
                            })
                            .child(t!("fleet-lab-modal-start"))
                    }
                )
                // Enroll button
                .child(
                    if is_enrolled {
                        crate::components::button::badge(theme().ok)
                            .id(ElementId::NamedInteger("node-enrolled-pill".into(), idx as u64))
                            .text_size(font_px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .child(t!("fleet-lab-modal-enrolled"))
                    } else {
                        let n_name = node_name.clone();
                        div()
                            .id(ElementId::NamedInteger("node-enroll-btn".into(), idx as u64))
                            .px(px(8.0))
                            .py(px(3.0))
                            .bg(text_primary())
                            .text_color(rgb(0x0a0a0c))
                            .font_family(FONT_MONO)
                            .font_weight(FontWeight::BOLD)
                            .text_size(font_px(9.5))
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0xffffff)))
                            .on_click(move |_ev, _window, cx| {
                                let name = n_name.clone();
                                app_enroll.update(cx, |this, cx| {
                                    this.enroll_lab_node(&name, cx);
                                });
                            })
                            .child(t!("fleet-lab-modal-enroll-in-crow"))
                    }
                ),
        )
}

// ==========================================
// Multipass VMs (ERR-119)
// ==========================================

fn section_title(text: &'static str) -> Div {
    div().font_family(FONT_MONO).text_size(font_px(10.5)).font_weight(FontWeight::BOLD).text_color(text_primary()).child(text)
}

fn pill(text: String, fg: Rgba) -> Div {
    crate::components::button::badge(fg).text_size(font_px(8.5)).child(text)
}

/// A small text button whose label is built at render time.
fn text_button(id: impl Into<ElementId>, label: String, color: Rgba) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(7.0))
        .py(px(2.0))
        .border_1()
        .border_color(border_default())
        .bg(bg_control())
        .font_family(FONT_MONO)
        .text_size(font_px(9.5))
        .text_color(color)
        .cursor_pointer()
        .hover(|s| s.bg(bg_row_hover()))
        .child(label)
}

/// Status, setup steps, the VMs Multipass has (with import and lifecycle
/// actions) and the launch form.
pub fn multipass_section(lab: &LocalLabState, servers: &[ServerRecord], app: Entity<CrowApp>) -> impl IntoElement {
    let (status_text, status_fg, _, steps): (String, Rgba, Rgba, &[InstallStep]) = match &lab.multipass {
        None => (t!("fleet-lab-modal-checking-2").into(), text_dimmer(), bg_chip(), &[]),
        Some(MultipassStatus::Missing { steps }) => (t!("fleet-lab-modal-not-installed-2").into(), text_dimmer(), bg_chip(), steps),
        Some(MultipassStatus::DaemonDown { steps, .. }) => (t!("fleet-lab-modal-daemon-not-running").into(), theme().warn, hex_rgba(0xf59e0b, 0.12), steps),
        Some(MultipassStatus::Ready { daemon, fixes, .. }) if !fixes.is_empty() => (t!("fleet-lab-modal-needs-a-fix", daemon = daemon), theme().warn, hex_rgba(0xf59e0b, 0.12), fixes),
        Some(MultipassStatus::Ready { daemon, .. }) => (t!("fleet-lab-modal-ready", daemon = daemon), theme().ok, theme().ok_bg, &[]),
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
                .child(div().flex().items_center().gap(px(8.0)).child(section_title(t!("fleet-lab-modal-multipass-vms"))).child(pill(status_text, status_fg)))
                .child(icon_button("btn-refresh-multipass", TablerIcon::Refresh, false).on_click(move |_ev, _window, cx| {
                    app_refresh.update(cx, |this, cx| this.refresh_multipass(cx));
                })),
        )
        .child(div().font_family(FONT_MONO).text_size(font_px(9.5)).text_color(text_tertiary()).child(
            t!("fleet-lab-modal-ubuntu-vms-on-this-machine-2"),
        ))
        .children(detail.map(|d| div().font_family(FONT_MONO).text_size(font_px(9.5)).text_color(theme().warn).child(d)))
        .children(steps.iter().enumerate().map(|(i, s)| setup_step(i, s, app.clone())))
        .children(lab.busy.clone().map(|b| div().font_family(FONT_MONO).text_size(font_px(10.0)).text_color(accent()).child(format!("● {b}"))))
        .children(lab.vm_error.clone().map(|e| div().font_family(FONT_MONO).text_size(font_px(10.0)).text_color(theme().crit).child(e)))
        .children(ready.then(|| {
            div()
                .bg(bg_app())
                .border_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .children(lab.vms.is_empty().then(|| div().p(px(14.0)).font_family(FONT_MONO).text_size(font_px(10.0)).text_color(text_faint()).child(t!("fleet-lab-modal-no-vms-yet-launch-one"))))
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
        .bg(bg_control())
        .border_1()
        .border_color(border_panel())
        .child(div().font_family(FONT_MONO).text_size(font_px(9.5)).text_color(text_secondary()).child(format!("{}. {}", i + 1, s.why)))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_1().min_w(px(0.0)).font_family(FONT_MONO).text_size(font_px(10.0)).text_color(text_primary()).child(s.command.clone()))
                .child(text_button(ElementId::NamedInteger("mp-copy-step".into(), i as u64), t!("fleet-lab-modal-copy").into(), text_secondary()).on_click(move |_ev, _window, cx| {
                    let c = command.clone();
                    app.update(cx, |this, cx| this.copy_text_with_toast(&c, t!("fleet-lab-modal-command-copied"), cx));
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
            actions = actions.child(pill(t!("fleet-lab-modal-in-fleet").into(), theme().ok));
        } else if running {
            let (app_i, name) = (app.clone(), vm.name.clone());
            actions = actions.child(text_button(ElementId::NamedInteger("mp-import".into(), i as u64), t!("fleet-lab-modal-import-to-crow").into(), text_primary()).on_click(move |_ev, _window, cx| {
                let name = name.clone();
                app_i.update(cx, |this, cx| this.import_multipass_vm(&name, cx));
            }));
        }
        actions = if running {
            actions.child(action(1, "STOP", text_secondary(), Lifecycle::Stop)).child(action(2, "SUSPEND", text_secondary(), Lifecycle::Suspend)).child(action(3, "RESTART", text_secondary(), Lifecycle::Restart))
        } else {
            actions.child(action(4, "START", theme().ok, Lifecycle::Start))
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
        .border_color(border_row())
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
                        .child(div().size(px(8.0)).rounded_full().bg(if running { theme().ok } else { text_dimmer() }))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(div().font_family(FONT_MONO).text_size(font_px(11.0)).font_weight(FontWeight::BOLD).text_color(text_primary()).child(vm.name.clone()))
                                .child(div().font_family(FONT_MONO).text_size(font_px(9.0)).text_color(text_muted()).child(format!(
                                    "{} · {} · {}",
                                    vm.state,
                                    vm.address().unwrap_or(t!("fleet-lab-modal-no-address")),
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
                .child(div().flex_1().font_family(FONT_MONO).text_size(font_px(10.0)).text_color(theme().crit).child(t!("fleet-lab-modal-delete-and-its-disk-for", name = vm.name, in_fleet = if in_fleet { t!("fleet-lab-modal-its-fleet-entry-goes-to") } else { "" })))
                .child(text_button(ElementId::NamedInteger("mp-purge-yes".into(), i as u64), t!("fleet-lab-modal-delete-for-good").into(), theme().crit).on_click(move |_ev, _window, cx| {
                    let name = name_yes.clone();
                    app_yes.update(cx, |this, cx| this.run_vm_lifecycle(Lifecycle::Purge, &name, cx));
                }))
                .child(text_button(ElementId::NamedInteger("mp-purge-no".into(), i as u64), t!("fleet-lab-modal-cancel").into(), text_secondary()).on_click(move |_ev, _window, cx| {
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
        .child(div().font_family(FONT_MONO).text_size(font_px(9.0)).text_color(text_tertiary()).child(label))
        .child(icon_button(ElementId::Name(format!("{id}-down").into()), TablerIcon::Minus, false).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| down(this, cx))))
        .child(div().min_w(px(34.0)).flex().justify_center().font_family(FONT_MONO).text_size(font_px(10.5)).text_color(text_primary()).child(value))
        .child(icon_button(ElementId::Name(format!("{id}-up").into()), TablerIcon::Plus, false).on_click(move |_ev, _window, cx| app_up.update(cx, |this, cx| up(this, cx))))
}

fn launch_form(lab: &LocalLabState, app: Entity<CrowApp>) -> impl IntoElement {
    let l = lab.launch.clone();
    let idle = lab.busy.is_none();
    let (app_name, app_go) = (app.clone(), app.clone());
    let (c, m, d) = (l.cpus, l.memory_gb, l.disk_gb);
    div()
        .p(px(12.0))
        .bg(bg_control())
        .border_1()
        .border_color(border_default())
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(section_title(t!("fleet-lab-modal-launch-a-vm")))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(div().font_family(FONT_MONO).text_size(font_px(11.0)).text_color(text_primary()).child(l.name.clone()))
                        .child(icon_button("btn-reroll-vm-name", TablerIcon::Refresh, false).on_click(move |_ev, _window, cx| app_name.update(cx, |this, cx| this.reroll_vm_name(cx)))),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(16.0))
                .child(stepper("vm-cpu", t!("fleet-lab-modal-cpu"), c.to_string(), move |a, cx| a.set_vm_size(c.saturating_sub(1), m, d, cx), move |a, cx| a.set_vm_size(c + 1, m, d, cx), app.clone()))
                .child(stepper("vm-mem", t!("fleet-lab-modal-memory"), t!("fleet-lab-modal-gb", m = m), move |a, cx| a.set_vm_size(c, m.saturating_sub(1), d, cx), move |a, cx| a.set_vm_size(c, m + 1, d, cx), app.clone()))
                .child(stepper("vm-disk", t!("fleet-lab-modal-disk"), t!("fleet-lab-modal-gb-2", d = d), move |a, cx| a.set_vm_size(c, m, d.saturating_sub(5), cx), move |a, cx| a.set_vm_size(c, m, d + 5, cx), app.clone())),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().flex().items_center().gap(px(6.0)).children([("lts", t!("fleet-lab-modal-newest-lts")), ("24.04", "24.04"), ("22.04", "22.04")].into_iter().enumerate().map(|(i, (image, label))| {
                    let selected = l.image == image;
                    let app_img = app.clone();
                    text_button(ElementId::NamedInteger("vm-image".into(), i as u64), label.to_string(), if selected { text_primary() } else { text_tertiary() })
                        .when(selected, |b| b.border_color(text_primary()).bg(bg_nav_active()))
                        .on_click(move |_ev, _window, cx| app_img.update(cx, |this, cx| this.set_vm_image(image, cx)))
                })))
                .child(
                    div()
                        .id("btn-launch-vm")
                        .px(px(14.0))
                        .py(px(6.0))
                        .bg(if idle { text_primary() } else { bg_chip() })
                        .text_color(if idle { rgb(0x0a0a0c) } else { text_dimmer() })
                        .font_weight(FontWeight::BOLD)
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.5))
                        .when(idle, |b| b.cursor_pointer().hover(|s| s.bg(hex_rgb(0xffffff))))
                        .on_click(move |_ev, _window, cx| app_go.update(cx, |this, cx| this.launch_multipass_vm(cx)))
                        .child(t!("fleet-lab-modal-launch-add-to-fleet")),
                ),
        )
}
