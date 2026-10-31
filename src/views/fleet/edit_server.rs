//! The Edit server dialog (ERR-166).

use gpui_kit::component::input::Input;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::edit_server::EditServer;
use crate::app::CrowApp;
use crate::components::button::{button, Tone};
use crate::host::{transport_kind, TransportKind};
use crate::theme::*;
use crate::vault::{ServerRecord, SshKeyRecord};

const ENVS: [&str; 4] = ["PROD", "STAGE", "DEV", "LAB"];

fn chip(id: impl Into<SharedString>, text: impl Into<SharedString>, selected: bool) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .px(px(9.0))
        .py(px(4.0))
        .border_1()
        .border_color(if selected { OK } else { BORDER_DEFAULT })
        .bg(if selected { BG_CONTROL_ALT } else { BG_CONTROL })
        .text_size(px(10.5))
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if selected { TEXT_PRIMARY } else { TEXT_DIM })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .child(text.into())
}

fn row(label: impl Into<SharedString>, body: impl IntoElement) -> Div {
    div()
        .flex()
        .items_start()
        .gap(px(12.0))
        .child(div().w(px(96.0)).flex_none().pt(px(5.0)).text_size(px(10.0)).text_color(TEXT_DIMMER).child(label.into()))
        .child(div().flex_1().min_w(px(0.0)).child(body))
}

pub fn edit_server_modal(e: &EditServer, servers: &[ServerRecord], groups: &[String], keys: &[SshKeyRecord], app: Entity<CrowApp>) -> impl IntoElement {
    let server = servers.iter().find(|s| s.id == e.server_id);
    let ssh = server.is_some_and(|s| transport_kind(s) == TransportKind::Ssh);
    let title = server.map(|s| s.name.clone()).unwrap_or_default();
    let address = server.map(|s| format!("{}:{}", s.host, s.port)).unwrap_or_default();
    let (app_scrim, app_cancel, app_save) = (app.clone(), app.clone(), app.clone());

    let mut group_names: Vec<String> = groups.iter().map(|g| g.trim().to_string()).filter(|g| !g.is_empty() && g != "default").collect();
    if !e.group.is_empty() && !group_names.contains(&e.group) {
        group_names.push(e.group.clone());
    }
    let bastions: Vec<(String, String)> = crate::app::bastions::choices(Some(&e.server_id), servers).into_iter().map(|b| (b.id.clone(), crate::app::bastions::route(&b.id, servers))).collect();
    let reach_changes = server.is_some_and(|s| s.key_id != e.key_id || s.jump_host_id != e.jump);

    let pick = |id: String, text: String, selected: bool, f: std::rc::Rc<dyn Fn(&mut EditServer)>| {
        let app = app.clone();
        chip(id, text, selected).on_click(move |_ev, _window, cx| {
            let f = f.clone();
            app.update(cx, |this, cx| this.edit_server_pick(|e| f(e), cx))
        })
    };

    div()
        .id("edit-server-scrim")
        .absolute()
        .inset_0()
        // Blocks the page underneath: no hover or clicks through.
        .occlude()
        .bg(hex_rgba(0x060709, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| app_scrim.update(cx, |this, cx| this.close_edit_server(cx)))
        .child(
            div()
                .id("edit-server-card")
                .occlude()
                .on_click(|_ev, _window, cx| cx.stop_propagation())
                .w(px(560.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .flex()
                .flex_col()
                .font_family(FONT_MONO)
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(12.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .flex_col()
                        .gap(px(3.0))
                        .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child(t!("fleet-edit-server-title", name = title)))
                        .when(ssh, |d| d.child(div().text_size(px(10.0)).text_color(TEXT_DIM).child(t!("fleet-edit-server-address-stays", address = address)))),
                )
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(row(t!("fleet-edit-server-name"), Input::new(&e.name).font_family(FONT_MONO).bg(BG_APP)))
                        .child(row(
                            t!("fleet-edit-server-environment"),
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.0))
                                .child(pick("es-env-none".into(), "none".into(), e.env.is_empty(), std::rc::Rc::new(|e| e.env.clear())))
                                .children(ENVS.iter().map(|v| pick(format!("es-env-{v}"), v.to_string(), e.env == *v, std::rc::Rc::new(move |e| e.env = v.to_string())))),
                        ))
                        .child(row(
                            t!("fleet-edit-server-group"),
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.0))
                                .child(pick("es-group-none".into(), "none".into(), e.group.is_empty() || e.group == "default", std::rc::Rc::new(|e| e.group.clear())))
                                .children(group_names.iter().map(|g| {
                                    let name = g.clone();
                                    pick(format!("es-group-{g}"), g.clone(), &e.group == g, std::rc::Rc::new(move |e| e.group = name.clone()))
                                })),
                        ))
                        .child(row(t!("fleet-edit-server-role"), Input::new(&e.role).font_family(FONT_MONO).bg(BG_APP)))
                        .child(row(t!("fleet-edit-server-tags"), Input::new(&e.tags).font_family(FONT_MONO).bg(BG_APP)))
                        // How Crow reaches it: SSH servers only.
                        .when(ssh, |d| {
                            d.child(div().h(px(1.0)).bg(BORDER_PANEL))
                                .child(div().text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(t!("fleet-edit-server-how-crow-reaches-it")))
                                .child(row(t!("fleet-edit-server-login-user"), Input::new(&e.login).font_family(FONT_MONO).bg(BG_APP)))
                                .child(row(
                                    t!("fleet-edit-server-key"),
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(6.0))
                                        // No key of Crow's: ssh's own default identity.
                                        .child(pick("es-key-default".into(), t!("fleet-edit-server-default-key").to_string(), e.key_id.is_none(), std::rc::Rc::new(|e| e.key_id = None)))
                                        .children(keys.iter().map(|k| {
                                        let id = k.id.clone();
                                        pick(format!("es-key-{}", k.id), k.name.clone(), e.key_id.as_deref() == Some(k.id.as_str()), std::rc::Rc::new(move |e| e.key_id = Some(id.clone())))
                                    })),
                                ))
                                .child(row(
                                    t!("fleet-edit-server-route"),
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(6.0))
                                        .child(pick("es-route-direct".into(), t!("fleet-edit-server-direct").to_string(), e.jump.is_none(), std::rc::Rc::new(|e| e.jump = None)))
                                        .children(bastions.iter().map(|(id, route)| {
                                            let b = id.clone();
                                            pick(format!("es-route-{id}"), t!("fleet-edit-server-via", route = route), e.jump.as_deref() == Some(id.as_str()), std::rc::Rc::new(move |e| e.jump = Some(b.clone())))
                                        })),
                                ))
                        }),
                )
                .children(e.error.clone().map(|err| div().mx(px(16.0)).mb(px(10.0)).p(px(8.0)).bg(CRIT_ROW_BG).text_size(px(10.5)).text_color(CRIT).child(err)))
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(10.0))
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().flex_1().text_size(px(10.0)).text_color(TEXT_DIM).child(if e.testing { t!("fleet-edit-server-trying").to_string() } else if reach_changes { t!("fleet-edit-server-tried-first").to_string() } else { String::new() }))
                        .child(
                            button("btn-edit-server-cancel", Tone::Neutral)
                                .on_click(move |_ev, _window, cx| app_cancel.update(cx, |this, cx| this.close_edit_server(cx)))
                                .child(t!("fleet-edit-server-cancel")),
                        )
                        .child(
                            button("btn-edit-server-save", Tone::Primary)
                                .when(e.testing, |d| d.opacity(0.5))
                                .when(!e.testing, |d| d.on_click(move |_ev, _window, cx| app_save.update(cx, |this, cx| this.save_edit_server(cx))))
                                .child(t!("fleet-edit-server-save")),
                        ),
                ),
        )
}
