//! Fleet Setup → TEAM (ERR-150), and the sign-in screen.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::team::TeamState;
use crate::app::CrowApp;
use crate::components::icons::TablerIcon;
use crate::team::{Permission, Role};
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn chip(id: impl Into<SharedString>, text: impl Into<SharedString>, on: bool) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .px(px(8.0))
        .py(px(3.0))
        .border_1()
        .border_color(if on { TEXT_PRIMARY } else { BORDER_DEFAULT })
        .bg(if on { BG_CONTROL_ALT } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(if on { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if on { TEXT_PRIMARY } else { TEXT_DIM })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(text.into())
}

fn input(i: Option<&Entity<gpui_kit::component::input::InputState>>) -> impl IntoElement {
    div().flex_1().min_w(px(0.0)).children(i.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))
}

pub fn team_page(fleet: &FleetState, st: &TeamState, app: Entity<CrowApp>) -> impl IntoElement {
    let enabled = st.config.enabled();
    let role = st.new_role.clone().unwrap_or(Role::Viewer);
    let envs: Vec<String> = {
        let mut e: Vec<String> = fleet.servers.iter().map(|s| s.env.trim().to_uppercase()).filter(|e| !e.is_empty()).collect();
        e.sort();
        e.dedup();
        e
    };
    let (app_add, app_appr, app_out) = (app.clone(), app.clone(), app.clone());
    div()
        .id("team-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1000.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                .child(div().flex().items_center().gap(px(12.0)).child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("TEAM")).child(mono(10.5, if enabled { OK } else { TEXT_DIM }).child(if enabled { format!("{} member{} · team mode on", st.config.members.len(), if st.config.members.len() == 1 { "" } else { "s" }) } else { "off: Crow is single-user".to_string() })).child(div().flex_1()).when(st.active.is_some(), |d| d.child(chip("team-signout", "SIGN OUT / SWITCH", false).on_click(move |_ev, _w, cx| app_out.update(cx, |this, cx| this.sign_out(cx))))))
                .child(mono(9.5, TEXT_FAINT).line_height(px(14.0)).child("Who uses this Crow, with which role, on which servers. Each member signs in with their own passphrase; their role is enforced on every action that changes a server (service actions, config writes, users and firewall, fleet runs), and every change record carries their name. Members live in this machine's vault: sharing them across machines needs Crow accounts (later)."))
                .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).line_height(px(15.0)).child(m)))
                // Members.
                .children(st.config.members.iter().map(|m| {
                    let (app, id) = (app.clone(), m.id.clone());
                    let me = st.active.as_deref() == Some(m.id.as_str());
                    let perms: Vec<&str> = Permission::ALL.iter().filter(|p| m.role.allows(**p)).map(|p| p.label()).collect();
                    div()
                        .group("team-member")
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(10.0))
                        .py(px(8.0))
                        .border_1()
                        .border_color(if me { OK.opacity(0.5) } else { BORDER_PANEL })
                        .bg(BG_PANEL)
                        .child(mono(11.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).w(px(140.0)).flex_none().child(format!("{}{}", m.name, if me { " (you)" } else { "" })))
                        .child(mono(10.5, if m.role == Role::Admin { WARN } else { TEXT_SECONDARY }).w(px(90.0)).flex_none().child(m.role.name()))
                        .child(mono(10.0, TEXT_DIM).w(px(160.0)).flex_none().child(m.scope.describe()))
                        .child(mono(9.5, TEXT_FAINT).flex_1().min_w(px(0.0)).child(perms.join(", ")))
                        .child(div().w(px(24.0)).flex_none().child(
                            crate::components::icon_button::icon_button(ElementId::Name(format!("team-remove-{}", m.id).into()), TablerIcon::Trash, true)
                                .invisible()
                                .group_hover("team-member", |s| s.visible())
                                .on_click(move |_ev, _w, cx| {
                                    let id = id.clone();
                                    app.update(cx, |this, cx| this.remove_member(&id, cx))
                                }),
                        ))
                }))
                .when(enabled, |d| {
                    d.child(
                        div()
                            .id("team-approval")
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .on_click(move |_ev, _w, cx| app_appr.update(cx, |this, cx| this.toggle_approval(cx)))
                            .child(div().size(px(12.0)).border_1().border_color(if st.config.require_approval { WARN } else { BORDER_STRONG }).bg(if st.config.require_approval { WARN } else { BG_CONTROL }))
                            .child(mono(10.5, TEXT_SECONDARY).child("fleet runs need a second member's approval (their passphrase, typed at the run)")),
                    )
                })
                // Add.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .p(px(12.0))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(mono(10.0, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(if enabled { "ADD A MEMBER" } else { "START A TEAM: THE FIRST MEMBER IS AN ADMIN (YOU)" }))
                        .child(div().flex().gap(px(8.0)).child(div().w(px(200.0)).child(input(st.new_name.as_ref()))).child(input(st.new_pass.as_ref())))
                        .when(enabled, |d| {
                            d.child(div().flex().flex_wrap().items_center().gap(px(6.0)).child(mono(9.5, TEXT_DIMMER).child("ROLE")).children([Role::Viewer, Role::Operator, Role::Admin].into_iter().map(|r| {
                                let (app, on) = (app.clone(), role == r);
                                chip(format!("team-role-{}", r.name()), r.name(), on).on_click(move |_ev, _w, cx| {
                                    let r = r.clone();
                                    app.update(cx, |this, cx| this.set_new_role(r, cx))
                                })
                            })).child({
                                let app = app.clone();
                                let on = matches!(role, Role::Custom { .. });
                                chip("team-role-custom", "CUSTOM", on).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_new_role(Role::Custom { name: "Custom".into(), perms: vec![] }, cx)))
                            }))
                            .when(matches!(role, Role::Custom { .. }), |d| {
                                let perms = match &role {
                                    Role::Custom { perms, .. } => perms.clone(),
                                    _ => vec![],
                                };
                                d.child(div().flex().flex_wrap().gap(px(6.0)).child(mono(9.5, TEXT_DIMMER).child("CAN")).children(Permission::ALL.into_iter().filter(|p| *p != Permission::View).map(|p| {
                                    let (app, on, perms) = (app.clone(), perms.contains(&p), perms.clone());
                                    chip(format!("team-perm-{p:?}"), p.label(), on).on_click(move |_ev, _w, cx| {
                                        let mut next = perms.clone();
                                        if on { next.retain(|x| *x != p) } else { next.push(p) }
                                        app.update(cx, |this, cx| this.set_new_role(Role::Custom { name: "Custom".into(), perms: next }, cx))
                                    })
                                })))
                            })
                            .child(div().flex().flex_wrap().items_center().gap(px(6.0)).child(mono(9.5, TEXT_DIMMER).child("ON")).children(fleet.groups.iter().map(|g| {
                                let (app, name, on) = (app.clone(), g.clone(), st.new_scope_groups.contains(g));
                                chip(format!("team-scope-g-{g}"), format!("group {g}"), on).on_click(move |_ev, _w, cx| {
                                    let n = name.clone();
                                    app.update(cx, |this, cx| this.toggle_new_scope(Some(n), None, cx))
                                })
                            })).children(envs.iter().map(|e| {
                                let (app, name, on) = (app.clone(), e.clone(), st.new_scope_envs.contains(e));
                                chip(format!("team-scope-e-{e}"), format!("env {e}"), on).on_click(move |_ev, _w, cx| {
                                    let n = name.clone();
                                    app.update(cx, |this, cx| this.toggle_new_scope(None, Some(n), cx))
                                })
                            })).child(mono(9.5, TEXT_FAINT).child("(none picked: the whole fleet; envs win over groups)")))
                        })
                        .child(div().flex().child(chip("team-add", if enabled { "ADD MEMBER" } else { "START THE TEAM" }, false).on_click(move |_ev, window, cx| app_add.update(cx, |this, cx| this.add_member(window, cx))))),
                ),
        )
}

/// Over everything while a team has no one signed in.
pub fn sign_in_overlay(st: &TeamState, app: Entity<CrowApp>) -> impl IntoElement {
    let app_go = app.clone();
    div()
        .id("team-signin")
        .occlude()
        .absolute()
        .top(px(36.0))
        .left_0()
        .right_0()
        .bottom_0()
        .bg(BG_WINDOW)
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(420.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .p(px(20.0))
                .border_1()
                .border_color(BORDER_PANEL)
                .bg(BG_PANEL)
                .child(mono(13.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child("Who's at the keyboard?"))
                .child(mono(10.0, TEXT_DIM).line_height(px(14.0)).child("This Crow has a team: sign in so your role applies and your changes carry your name."))
                .child(div().flex().flex_wrap().gap(px(6.0)).children(st.config.members.iter().map(|m| {
                    let (app, id) = (app.clone(), m.id.clone());
                    chip(format!("signin-{}", m.id), format!("{} · {}", m.name, m.role.name()), st.signing_in.as_deref() == Some(m.id.as_str())).on_click(move |_ev, _w, cx| {
                        let id = id.clone();
                        app.update(cx, |this, cx| this.choose_signin(&id, cx))
                    })
                })))
                .when(st.signing_in.is_some(), |d| d.child(input(st.passphrase.as_ref())).child(div().flex().child(chip("signin-go", "SIGN IN", false).on_click(move |_ev, window, cx| app_go.update(cx, |this, cx| this.sign_in(window, cx))))))
                .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { CRIT }).child(m))),
        )
}

/// A refusal or sign-in note, bottom right on every screen.
pub fn team_notice(notice: &(bool, String)) -> impl IntoElement {
    let (ok, text) = notice;
    let color = if *ok { OK } else { CRIT };
    div().absolute().bottom(px(16.0)).right(px(16.0)).max_w(px(520.0)).child(div().p(px(10.0)).border_1().border_color(color).bg(BG_PANEL).child(mono(10.5, color).font_weight(FontWeight::BOLD).line_height(px(15.0)).child(text.clone())))
}
