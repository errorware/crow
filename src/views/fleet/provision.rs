//! Provision Cloud Instance modal (`ERR-224`): launch fresh AWS EC2,
//! Linode, or UpCloud instances pre-seeded with a Crow Vault SSH key.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use crate::components::icon_button::icon_button;
use crate::components::icons::{inherited_icon, TablerIcon};

use crate::app::providers::{ProvisionInputs, ProvisionState};
use crate::app::CrowApp;
use crate::keys::SshKeyRecord;
use crate::theme::*;
use crate::vault::ProviderAccount;

pub fn provision_panel(
    state: &ProvisionState,
    inputs: Option<&ProvisionInputs>,
    accounts: &[ProviderAccount],
    enrolled_keys: &[SshKeyRecord],
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let (a_scrim, a_close, a_cancel, a_submit) = (app.clone(), app.clone(), app.clone(), app.clone());

    div()
        .id("provision-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| a_scrim.update(cx, |this, cx| this.close_provision(cx)))
        .child(
            div()
                .id("provision-panel")
                .w(px(740.0))
                .max_h(px(680.0))
                .flex()
                .flex_col()
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(
                    // Header
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(inherited_icon(TablerIcon::Cloud, px(16.0)))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(t!("fleet-provision-modal-title")),
                                )
                                .child(
                                    crate::components::button::badge(OK)
                                        .text_size(px(9.0))
                                        .child(t!("fleet-provision-ssh-vault-seeded")),
                                ),
                        )
                        .child(
                            icon_button("provision-close", TablerIcon::X, false)
                                .on_click(move |_ev, _window, cx| a_close.update(cx, |this, cx| this.close_provision(cx))),
                        ),
                )
                .child(
                    // Scrollable form body
                    div()
                        .id("provision-panel-scroll")
                        .flex_1()
                        .overflow_y_scrollbar()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(14.0))
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        // Error banner if any
                        .children(state.error.as_ref().map(|err| {
                            div()
                                .p(px(10.0))
                                .bg(CRIT.opacity(0.1))
                                .border_1()
                                .border_color(CRIT.opacity(0.4))
                                .text_color(CRIT)
                                .text_size(px(10.5))
                                .child(format!("⚠ {err}"))
                        }))
                        // 1. Account selector
                        .child(render_account_section(state, accounts, app.clone()))
                        // 2. Server label
                        .child(render_label_section(inputs))
                        // 3. Catalog sections (Region, Plan, Image)
                        .children(if state.loading_catalog {
                            Some(
                                div()
                                    .py(px(20.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(8.0))
                                    .text_color(TEXT_DIMMER)
                                    .child(inherited_icon(TablerIcon::Refresh, px(14.0)))
                                    .child(t!("fleet-provision-loading-catalog")),
                            )
                        } else if let Some(cat) = &state.catalog {
                            Some(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(14.0))
                                    .child(render_options_section(
                                        "provision-opt-region",
                                        t!("fleet-provision-region"),
                                        &cat.regions,
                                        &state.region,
                                        app.clone(),
                                        |app, id, cx| app.update(cx, |this, cx| this.set_provision_region(&id, cx)),
                                    ))
                                    .child(render_options_section(
                                        "provision-opt-plan",
                                        t!("fleet-provision-plan"),
                                        &cat.plans,
                                        &state.plan,
                                        app.clone(),
                                        |app, id, cx| app.update(cx, |this, cx| this.set_provision_plan(&id, cx)),
                                    ))
                                    .child(render_options_section(
                                        "provision-opt-image",
                                        t!("fleet-provision-image"),
                                        &cat.images,
                                        &state.image,
                                        app.clone(),
                                        |app, id, cx| app.update(cx, |this, cx| this.set_provision_image(&id, cx)),
                                    )),
                            )
                        } else {
                            None
                        })
                        // 4. SSH Key selector
                        .child(render_ssh_keys_section(state, enrolled_keys, app.clone()))
                        // 5. Tags input
                        .child(render_tags_section(inputs)),
                )
                .child(
                    // Footer actions
                    div()
                        .h(px(48.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(10.0))
                        .px(px(16.0))
                        .border_t_1()
                        .border_color(BORDER_ROW)
                        .child(
                            action_btn("btn-provision-cancel", t!("common-cancel").into(), TEXT_SECONDARY, true, move |cx| {
                                a_cancel.update(cx, |this, cx| this.close_provision(cx));
                            }),
                        )
                        .child(
                            action_btn(
                                "btn-provision-submit",
                                if state.provisioning {
                                    t!("fleet-provision-launching").into()
                                } else {
                                    t!("fleet-provision-launch-instance").into()
                                },
                                OK,
                                !state.provisioning && !state.loading_catalog,
                                move |cx| {
                                    a_submit.update(cx, |this, cx| this.submit_provision(cx));
                                },
                            ),
                        ),
                ),
        )
}

fn render_account_section(state: &ProvisionState, accounts: &[ProviderAccount], app: Entity<CrowApp>) -> impl IntoElement {
    let prov_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| {
            crate::providers::factory(&a.plugin)
                .is_some_and(|f| (f.manifest)().has_capability(crow_provider_core::capabilities::INSTANCES_CREATE))
        })
        .cloned()
        .collect();

    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_color(TEXT_MUTED)
                .text_size(px(10.0))
                .child(t!("fleet-provision-provider-account")),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(6.0))
                .children(prov_accounts.into_iter().map(|a| {
                    let is_sel = a.id == state.selected_account_id;
                    let display = crate::providers::display_name(&a.plugin);
                    let title = if a.label.is_empty() {
                        display
                    } else {
                        format!("{} ({})", a.label, display)
                    };
                    let (a_sel, acct_id) = (app.clone(), a.id.clone());
                    let chip_id = format!("provision-acct-{}", a.id);
                    chip(chip_id, title, is_sel, move |cx| {
                        let id = acct_id.clone();
                        a_sel.update(cx, |this, cx| this.select_provision_account(&id, cx));
                    })
                })),
        )
}

fn render_label_section(inputs: Option<&ProvisionInputs>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_color(TEXT_MUTED)
                .text_size(px(10.0))
                .child(t!("fleet-provision-instance-label")),
        )
        .children(inputs.map(|i| {
            div()
                .h(px(28.0))
                .px(px(8.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .flex()
                .items_center()
                .child(Input::new(&i.label).text_size(px(11.0)))
        }))
}

fn render_tags_section(inputs: Option<&ProvisionInputs>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_color(TEXT_MUTED)
                .text_size(px(10.0))
                .child(t!("fleet-provision-tags")),
        )
        .children(inputs.map(|i| {
            div()
                .h(px(28.0))
                .px(px(8.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .flex()
                .items_center()
                .child(Input::new(&i.tags).text_size(px(11.0)))
        }))
}

fn render_options_section(
    id_prefix: &'static str,
    title: &str,
    options: &[crow_provider_core::hosts::ProvisionOption],
    selected_id: &str,
    app: Entity<CrowApp>,
    on_select: impl Fn(Entity<CrowApp>, String, &mut App) + 'static + Clone,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_color(TEXT_MUTED)
                .text_size(px(10.0))
                .child(title.to_string()),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(6.0))
                .children(options.iter().map(|opt| {
                    let is_sel = opt.id == selected_id;
                    let opt_id = opt.id.clone();
                    let app = app.clone();
                    let on_sel = on_select.clone();
                    let chip_id = format!("{id_prefix}-{}", opt.id);
                    chip(chip_id, opt.label.clone(), is_sel, move |cx| {
                        on_sel(app.clone(), opt_id.clone(), cx);
                    })
                })),
        )
}

fn render_ssh_keys_section(
    state: &ProvisionState,
    enrolled_keys: &[SshKeyRecord],
    app: Entity<CrowApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_color(TEXT_MUTED)
                .text_size(px(10.0))
                .child(t!("fleet-provision-vault-ssh-key")),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(6.0))
                .children(enrolled_keys.iter().map(|k| {
                    let is_sel = k.id == state.selected_ssh_key_id;
                    let label = format!("{} · {}", k.name, k.algorithm);
                    let (a_key, k_id) = (app.clone(), k.id.clone());
                    let chip_id = format!("provision-key-{}", k.id);
                    chip(chip_id, label, is_sel, move |cx| {
                        let id = k_id.clone();
                        a_key.update(cx, |this, cx| this.set_provision_ssh_key(&id, cx));
                    })
                })),
        )
        .child(
            div()
                .text_color(TEXT_DIMMER)
                .text_size(px(9.5))
                .child(t!("fleet-provision-vault-ssh-key-help")),
        )
}

fn chip(id: String, label: String, selected: bool, on_click: impl Fn(&mut App) + 'static) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .px(px(8.0))
        .py(px(4.0))
        .border_1()
        .border_color(if selected { OK } else { BORDER_DEFAULT })
        .bg(if selected { BG_ROW_HOVER } else { BG_PANEL })
        .text_color(if selected { OK } else { TEXT_SECONDARY })
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .cursor_pointer()
        .hover(|s| s.border_color(BORDER_CONTROL_SEL).text_color(TEXT_PRIMARY))
        .on_click(move |_ev, _window, cx| on_click(cx))
        .child(label)
}

fn action_btn(
    id: &'static str,
    label: String,
    color: Rgba,
    enabled: bool,
    on_click: impl Fn(&mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .px(px(12.0))
        .py(px(5.0))
        .border_1()
        .border_color(if enabled { color.opacity(0.6) } else { BORDER_DEFAULT })
        .bg(if enabled { BG_PANEL } else { BG_APP })
        .text_color(if enabled { color } else { TEXT_FAINTER })
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .when(enabled, |d| {
            d.cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| on_click(cx))
        })
        .child(label)
}
