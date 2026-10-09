//! Fleet Setup → PEOPLE (ERR-144): accounts and keys across the fleet,
//! with offboarding and onboarding.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::patching::Target;
use crate::app::people::PeopleState;
use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::people::{accounts, find_keys, keys, Offboard};
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn section(title: &'static str, note: impl Into<SharedString>) -> Div {
    div().flex().items_baseline().gap(px(10.0)).child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(title)).child(mono(9.5, TEXT_FAINT).child(note.into()))
}

fn action(id: impl Into<SharedString>, text: impl Into<SharedString>, color: Rgba) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex_none()
        .px(px(8.0))
        .py(px(3.0))
        .border_1()
        .border_color(color.opacity(0.6))
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .cursor_pointer()
        .hover(move |s| s.bg(color.opacity(0.12)))
        .child(text.into())
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
        .text_color(if on { TEXT_PRIMARY } else { TEXT_DIM })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(text.into())
}

pub fn people_page(fleet: &FleetState, st: &PeopleState, known_keys: &[(String, String)], query: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let scans = st.read();
    let rows = accounts(&scans);
    let key_rows = keys(&scans, known_keys);
    let found = find_keys(&key_rows, query);
    let failed: Vec<String> = st.scans.iter().filter_map(|(id, r)| r.as_ref().err().map(|e| format!("{}: {e}", fleet.servers.iter().find(|s| &s.id == id).map(|s| s.name.as_str()).unwrap_or(id)))).collect();
    let (app_read, app_onboard, app_sudo) = (app.clone(), app.clone(), app.clone());
    div()
        .id("people-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1150.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(20.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("PEOPLE"))
                        .child(mono(10.5, TEXT_DIM).child(format!("{} accounts · {} keys · {} servers read", rows.len(), key_rows.len(), scans.len())))
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("btn-people-read")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .h(px(26.0))
                                .px(px(10.0))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_SECONDARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _w, cx| app_read.update(cx, |this, cx| this.refresh_people(cx)))
                                .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                                .child(if st.reading.is_empty() { "READ AGAIN".to_string() } else { format!("READING {}…", st.reading.len()) }),
                        ),
                )
                .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).child(m)))
                .children((!failed.is_empty()).then(|| mono(10.0, WARN).child(format!("couldn't read: {}", failed.join(" · ")))))
                // Accounts.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(section("WHO CAN LOG IN WHERE", "accounts with a login shell; ★ = sudo, ⊘ = locked, ◆ = Crow logs in as it"))
                        .children(rows.iter().map(|r| {
                            let selected = st.selected.as_deref() == Some(r.username.as_str());
                            let (app_sel, user) = (app.clone(), r.username.clone());
                            let crow_everywhere = r.on.iter().all(|p| p.crow);
                            div()
                                .flex()
                                .flex_col()
                                .border_1()
                                .border_color(if selected { BORDER_STRONG } else { BORDER_PANEL })
                                .child(
                                    div()
                                        .id(ElementId::Name(format!("person-{}", r.username).into()))
                                        .flex()
                                        .items_center()
                                        .gap(px(10.0))
                                        .px(px(10.0))
                                        .min_h(px(30.0))
                                        .bg(if selected { BG_ROW_SELECTED } else { BG_PANEL })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _w, cx| {
                                            let u = user.clone();
                                            app_sel.update(cx, |this, cx| this.select_person(if selected { None } else { Some(u) }, cx))
                                        })
                                        .child(mono(11.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).w(px(140.0)).flex_none().child(r.username.clone()))
                                        .child(div().flex_1().min_w(px(0.0)).flex().flex_wrap().gap(px(4.0)).children(r.on.iter().map(|p| {
                                            let mark = format!("{}{}{}{}", p.server, if p.sudo { " ★" } else { "" }, if p.locked { " ⊘" } else { "" }, if p.crow { " ◆" } else { "" });
                                            div().px(px(5.0)).py(px(1.0)).bg(BG_CHIP).font_family(FONT_MONO).text_size(px(9.5)).text_color(if p.locked { TEXT_FAINT } else if p.sudo { WARN } else { TEXT_TERTIARY }).child(mark)
                                        })))
                                        .child(mono(9.5, TEXT_FAINT).flex_none().child(format!("{} server{}", r.on.len(), if r.on.len() == 1 { "" } else { "s" }))),
                                )
                                .when(selected && !crow_everywhere && r.username != "root", |d| {
                                    let (a1, a2, u1, u2) = (app.clone(), app.clone(), r.username.clone(), r.username.clone());
                                    d.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .px(px(10.0))
                                            .py(px(6.0))
                                            .border_t_1()
                                            .border_color(BORDER_PANEL)
                                            .child(mono(9.5, TEXT_DIM).flex_1().min_w(px(0.0)).line_height(px(13.0)).child("Offboard everywhere: their keys are revoked first, then the account is locked and expired (reversible) or deleted (home kept). Servers where Crow logs in as this account are left out."))
                                            .child(action(format!("offboard-lock-{}", r.username), "LOCK EVERYWHERE", WARN).on_click(move |_ev, window, cx| {
                                                let u = u1.clone();
                                                a1.update(cx, |this, cx| this.plan_offboard(&u, Offboard::Lock, window, cx))
                                            }))
                                            .child(action(format!("offboard-delete-{}", r.username), "DELETE EVERYWHERE", CRIT).on_click(move |_ev, window, cx| {
                                                let u = u2.clone();
                                                a2.update(cx, |this, cx| this.plan_offboard(&u, Offboard::Delete, window, cx))
                                            })),
                                    )
                                })
                        })),
                )
                // Keys.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(section("FIND A KEY EVERYWHERE", "every key in any authorized_keys, named when it's one of Crow's"))
                        .children(st.key_query.as_ref().map(|i| div().max_w(px(560.0)).child(Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0)))))
                        .children(found.into_iter().take(40).map(|k| {
                            let crow_key = k.opens.iter().any(|o| o.3);
                            let (app, blob, label) = (app.clone(), k.blob.clone(), k.label.clone());
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(10.0))
                                .min_h(px(28.0))
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .bg(BG_PANEL)
                                .child(mono(10.5, if k.enrolled { OK } else { TEXT_PRIMARY }).w(px(200.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(format!("{}{}", k.label, if k.enrolled { " (Crow)" } else { "" })))
                                .child(mono(9.5, TEXT_FAINT).w(px(170.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(k.fingerprint.clone()))
                                .child(div().flex_1().min_w(px(0.0)).flex().flex_wrap().gap(px(4.0)).children(k.opens.iter().map(|(_, server, account, crow)| {
                                    div().px(px(5.0)).py(px(1.0)).bg(BG_CHIP).font_family(FONT_MONO).text_size(px(9.5)).text_color(if *crow { OK } else { TEXT_TERTIARY }).child(format!("{account}@{server}{}", if *crow { " ◆" } else { "" }))
                                })))
                                .child(action(format!("revoke-{}", k.fingerprint), "REVOKE EVERYWHERE", CRIT).on_click(move |_ev, window, cx| {
                                    let (blob, label) = (blob.clone(), label.clone());
                                    app.update(cx, |this, cx| this.plan_revoke_key_everywhere(&blob, &label, window, cx))
                                }))
                                .when(crow_key, |d| d.child(mono(9.0, TEXT_FAINT).child("(skips Crow's own logins)")))
                        })),
                )
                // Onboarding.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .p(px(12.0))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .bg(BG_PANEL)
                        .child(section("ONBOARD", "an account with their key on a group's servers (created where missing)"))
                        .child(div().flex().gap(px(8.0)).child(div().w(px(200.0)).children(st.new_user.as_ref().map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))).child(div().flex_1().children(st.new_key.as_ref().map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(6.0))
                                .child({
                                    let app = app.clone();
                                    chip("people-target-all", "every server", st.target == Target::All).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_people_target(Target::All, cx)))
                                })
                                .children(fleet.groups.iter().map(|g| {
                                    let (app, name) = (app.clone(), g.clone());
                                    chip(format!("people-target-{g}"), format!("group {g}"), st.target == Target::Group(g.clone())).on_click(move |_ev, _w, cx| {
                                        let name = name.clone();
                                        app.update(cx, |this, cx| this.set_people_target(Target::Group(name), cx))
                                    })
                                }))
                                .child(div().w(px(12.0)))
                                .child(
                                    div()
                                        .id("people-sudo")
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .cursor_pointer()
                                        .on_click(move |_ev, _w, cx| app_sudo.update(cx, |this, cx| this.toggle_people_sudo(cx)))
                                        .child(div().size(px(12.0)).border_1().border_color(if st.new_sudo { WARN } else { BORDER_STRONG }).bg(if st.new_sudo { WARN } else { BG_CONTROL }))
                                        .child(mono(10.5, TEXT_SECONDARY).child("with sudo")),
                                )
                                .child(div().flex_1())
                                .child(action("btn-onboard-run", "ONBOARD →", OK).on_click(move |_ev, window, cx| app_onboard.update(cx, |this, cx| this.plan_onboard(window, cx)))),
                        ),
                ),
        )
}
