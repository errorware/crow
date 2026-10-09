//! Fleet Setup → CHECKS (ERR-148).

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::checks::{ChecksState, FormKind};
use crate::app::CrowApp;
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

fn uptime(h: &[(i64, bool, Option<i64>)], secs: i64, now: i64) -> String {
    let r: Vec<_> = h.iter().filter(|(ts, _, _)| now - ts < secs).collect();
    if r.is_empty() {
        return "—".into();
    }
    format!("{:.2}%", 100.0 * r.iter().filter(|x| x.1).count() as f64 / r.len() as f64)
}

pub fn checks_page(fleet: &FleetState, st: &ChecksState, app: Entity<CrowApp>) -> impl IntoElement {
    let now = chrono::Utc::now().timestamp();
    let failing = st.defs.iter().filter(|c| st.last.get(&c.id).is_some_and(|o| !o.ok)).count();
    let app_add = app.clone();
    div()
        .id("checks-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1150.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                .child(div().flex().items_center().gap(px(12.0)).child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("CHECKS FROM OUTSIDE")).child(mono(10.5, if failing > 0 { CRIT } else { OK }).child(format!("{} checks · {failing} failing", st.defs.len()))).child(mono(9.5, TEXT_FAINT).child("run from this machine, only while Crow is open")))
                // Add one.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .p(px(12.0))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .bg(BG_PANEL)
                        .child(div().flex().items_center().gap(px(6.0)).children([(FormKind::Http, "HTTP(S)"), (FormKind::Tcp, "TCP PORT"), (FormKind::Dns, "DNS NAME")].into_iter().map(|(k, t)| {
                            let app = app.clone();
                            chip(format!("check-kind-{t}"), t, st.form == k).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_check_form(k, cx)))
                        })).child(div().w(px(12.0))).child(mono(9.5, TEXT_DIMMER).child("EVERY")).children([(30i64, "30 s"), (60, "1 min"), (300, "5 min")].into_iter().map(|(s, t)| {
                            let app = app.clone();
                            chip(format!("check-every-{s}"), t, st.every == s).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_check_every(s, cx)))
                        })))
                        .child(div().flex().gap(px(8.0)).child(input(st.target.as_ref())).when(st.form != FormKind::Tcp, |d| d.child(input(st.extra.as_ref()))).child(div().w(px(200.0)).child(input(st.name.as_ref()))))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(6.0))
                                .child(mono(9.5, TEXT_DIMMER).child("ATTACH TO"))
                                .child({
                                    let app = app.clone();
                                    chip("check-attach-none", "no server", st.attach.is_none()).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_check_attach(None, cx)))
                                })
                                .children(fleet.servers.iter().map(|s| {
                                    let (app, id) = (app.clone(), s.id.clone());
                                    chip(format!("check-attach-{}", s.id), s.name.clone(), st.attach.as_deref() == Some(s.id.as_str())).on_click(move |_ev, _w, cx| {
                                        let id = id.clone();
                                        app.update(cx, |this, cx| this.set_check_attach(Some(id), cx))
                                    })
                                }))
                                .child(div().flex_1())
                                .child(chip("btn-check-add", "ADD CHECK", false).on_click(move |_ev, _w, cx| app_add.update(cx, |this, cx| this.add_check(cx)))),
                        )
                        .child(mono(9.5, TEXT_FAINT).child("An attached check's alerts land on that server (its page, the map, email). A check alerts after two failures in a row and resolves on the first success.")),
                )
                .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).child(m)))
                // The checks.
                .children(st.defs.iter().map(|c| {
                    let last = st.last.get(&c.id);
                    let h = st.history.get(&c.id).cloned().unwrap_or_default();
                    let color = match last {
                        _ if st.running.contains(&c.id) && last.is_none() => TEXT_DIM,
                        Some(o) if o.ok => OK,
                        Some(_) => CRIT,
                        None => TEXT_FAINT,
                    };
                    let server = c.server_id.as_ref().and_then(|id| fleet.servers.iter().find(|s| &s.id == id)).map(|s| s.name.clone());
                    let (a_run, a_alert, a_del, id1, id2, id3) = (app.clone(), app.clone(), app.clone(), c.id.clone(), c.id.clone(), c.id.clone());
                    let recent: Vec<_> = h.iter().rev().take(60).rev().cloned().collect();
                    let max_lat = recent.iter().filter_map(|x| x.2).max().unwrap_or(1).max(1);
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .px(px(10.0))
                        .py(px(8.0))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .bg(BG_PANEL)
                        .child(div().flex_none().size(px(8.0)).rounded_full().bg(color))
                        .child(
                            div()
                                .w(px(330.0))
                                .flex_none()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(mono(11.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).whitespace_nowrap().overflow_hidden().text_ellipsis().child(c.name.clone()))
                                .child(mono(9.5, TEXT_FAINT).whitespace_nowrap().overflow_hidden().text_ellipsis().child(format!("{} · every {} s{}", c.target(), c.every_secs, server.map(|s| format!(" · on {s}")).unwrap_or_default())))
                                .child(mono(9.5, color).whitespace_nowrap().overflow_hidden().text_ellipsis().child(match last {
                                    Some(o) => format!("{}{}", o.detail, o.cert_days.map(|d| format!(" · cert {d} days")).unwrap_or_default()),
                                    None if st.running.contains(&c.id) => "checking…".into(),
                                    None => "not checked yet".into(),
                                })),
                        )
                        // The last results: a bar each, height = latency, red = failed.
                        .child(div().flex_1().min_w(px(0.0)).h(px(28.0)).flex().items_end().gap(px(1.0)).children(recent.iter().map(|(_, ok, lat)| {
                            let hgt = if *ok { 4.0 + 24.0 * lat.unwrap_or(0) as f32 / max_lat as f32 } else { 28.0 };
                            div().w(px(4.0)).h(px(hgt)).bg(if *ok { OK.opacity(0.7) } else { CRIT })
                        })))
                        .child(div().w(px(150.0)).flex_none().flex().flex_col().child(mono(9.5, TEXT_SECONDARY).child(format!("24 h {}", uptime(&h, 86_400, now)))).child(mono(9.5, TEXT_DIM).child(format!("7 d {}", uptime(&h, 7 * 86_400, now)))))
                        .child(chip(format!("check-run-{}", c.id), if st.running.contains(&c.id) { "…" } else { "RUN" }, false).on_click(move |_ev, _w, cx| {
                            let id = id1.clone();
                            a_run.update(cx, |this, cx| this.run_check(&id, cx))
                        }))
                        .child(chip(format!("check-alert-{}", c.id), if c.alert { "ALERTS ON" } else { "ALERTS OFF" }, c.alert).on_click(move |_ev, _w, cx| {
                            let id = id2.clone();
                            a_alert.update(cx, |this, cx| this.toggle_check_alert(&id, cx))
                        }))
                        .child(chip(format!("check-del-{}", c.id), "DELETE", false).on_click(move |_ev, _w, cx| {
                            let id = id3.clone();
                            a_del.update(cx, |this, cx| this.delete_check(&id, cx))
                        }))
                }))
                .when(st.defs.is_empty(), |d| d.child(mono(10.5, TEXT_DIM).child("No checks yet: add a website, a port or a DNS name above."))),
        )
}
