//! Fleet Setup → DR PLANS (ERR-151).

use gpui_kit::component::input::{Editor, Input};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::dr::{DrInputs, DrState};
use crate::app::CrowApp;
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn label(t: &'static str) -> Div {
    mono(9.5, TEXT_DIMMER).font_weight(FontWeight::SEMIBOLD).child(t)
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

fn editor(e: &Entity<gpui_kit::component::input::EditorState>, h: f32) -> impl IntoElement {
    div().h(px(h)).border_1().border_color(BORDER_DEFAULT).bg(BG_APP).child(Editor::new(e).h_full().appearance(false).bordered(false).font_family(FONT_MONO).text_size(px(11.0)))
}

fn input(i: &Entity<gpui_kit::component::input::InputState>) -> impl IntoElement {
    Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))
}

pub fn dr_page(fleet: &FleetState, st: &DrState, inputs: Option<&DrInputs>, app: Entity<CrowApp>) -> impl IntoElement {
    let Some(i) = inputs else { return div().into_any_element() };
    let plan = st.selected.as_ref().and_then(|id| st.plans.iter().find(|p| &p.id == id));
    let (a_new, a_save, a_del, a_check, a_drill, a_finish, a_export) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone(), app.clone(), app.clone());
    let list = div()
        .w(px(240.0))
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(div().flex().items_center().child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(t!("fleet-dr-plans"))).child(div().flex_1()).child(chip("dr-new", t!("fleet-dr-new"), false).on_click(move |_ev, window, cx| a_new.update(cx, |this, cx| this.new_dr_plan(window, cx)))))
        .when(st.plans.is_empty(), |d| d.child(mono(10.0, TEXT_DIM).line_height(px(14.0)).child(t!("fleet-dr-no-plans-yet-a-plan"))))
        .children(st.plans.iter().map(|p| {
            let (app, id) = (app.clone(), p.id.clone());
            let last = p.drills.last();
            chip(format!("dr-plan-{}", p.id), format!("{}{}", p.name, last.map(|d| t!("fleet-dr-drill-min", minutes = d.minutes, d = if d.met_rto { "" } else { " ✗" })).unwrap_or_default()), st.selected.as_deref() == Some(p.id.as_str())).on_click(move |_ev, window, cx| {
                let id = id.clone();
                app.update(cx, |this, cx| this.select_dr_plan(&id, window, cx))
            })
        }));
    let form = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(div().flex().gap(px(8.0)).child(div().flex_1().flex().flex_col().gap(px(4.0)).child(label(t!("fleet-dr-name"))).child(input(&i.name))).child(div().w(px(120.0)).flex().flex_col().gap(px(4.0)).child(label(t!("fleet-dr-rto-min"))).child(input(&i.rto))).child(div().w(px(120.0)).flex().flex_col().gap(px(4.0)).child(label(t!("fleet-dr-rpo-min"))).child(input(&i.rpo))))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(6.0))
                .child(label(t!("fleet-dr-for")))
                .children(fleet.servers.iter().map(|s| {
                    let (app, id) = (app.clone(), s.id.clone());
                    chip(format!("dr-target-{}", s.id), s.name.clone(), st.target == s.id).on_click(move |_ev, _w, cx| {
                        let id = id.clone();
                        app.update(cx, |this, cx| this.set_dr_target(id, cx))
                    })
                }))
                .children(fleet.groups.iter().map(|g| {
                    let (app, t) = (app.clone(), format!("group:{g}"));
                    chip(format!("dr-target-g-{g}"), t!("fleet-dr-group", g = g), st.target == t).on_click(move |_ev, _w, cx| {
                        let t = t.clone();
                        app.update(cx, |this, cx| this.set_dr_target(t, cx))
                    })
                })),
        )
        .child(div().flex().gap(px(8.0)).child(div().flex_1().flex().flex_col().gap(px(4.0)).child(label(t!("fleet-dr-what-matters"))).child(editor(&i.matters, 110.0))).child(div().flex_1().flex().flex_col().gap(px(4.0)).child(label(t!("fleet-dr-backups-path-or-glob-timer"))).child(editor(&i.backups, 110.0))))
        .child(div().flex().flex_col().gap(px(4.0)).child(label(t!("fleet-dr-runbook-markdown-with-commands"))).child(editor(&i.runbook, 200.0)))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(6.0))
                .child(chip("dr-save", t!("fleet-dr-save"), false).on_click(move |_ev, _w, cx| a_save.update(cx, |this, cx| this.save_dr_plan(cx))))
                .child(chip("dr-check", if st.checking > 0 { t!("fleet-dr-checking") } else { t!("fleet-dr-check-against-the-servers") }, false).on_click(move |_ev, _w, cx| a_check.update(cx, |this, cx| this.check_dr_plan(cx))))
                .child(chip("dr-export", t!("fleet-dr-export-md"), false).on_click(move |_ev, _w, cx| a_export.update(cx, |this, cx| this.export_dr_plan(cx))))
                .child(div().flex_1())
                .when(plan.is_some(), |d| d.child(chip("dr-delete", t!("fleet-dr-delete-plan"), false).on_click(move |_ev, _w, cx| a_del.update(cx, |this, cx| this.delete_dr_plan(cx))))),
        )
        .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).line_height(px(15.0)).child(m)))
        .children(st.findings.iter().map(|(server, f)| {
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .p(px(8.0))
                .border_1()
                .border_color(BORDER_PANEL)
                .bg(BG_PANEL)
                .child(mono(10.5, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(server.clone()))
                .children(f.iter().map(|x| mono(10.5, if x.ok { OK } else { CRIT }).child(format!("{} {}", if x.ok { "✓" } else { "✗" }, x.what))))
        }))
        // Drills.
        .when(plan.is_some(), |d| {
            let p = plan.unwrap();
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .p(px(10.0))
                    .border_1()
                    .border_color(BORDER_PANEL)
                    .child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(t!("fleet-dr-drills")))
                    .child(mono(9.5, TEXT_FAINT).line_height(px(13.0)).child(t!("fleet-dr-rebuild-it-for-real-somewhere")))
                    .child(match p.drill_started {
                        Some(start) => div().flex().items_center().gap(px(8.0)).child(mono(10.5, WARN).child(t!("fleet-dr-running-for-min", chrono = (chrono::Utc::now().timestamp() - start) / 60))).child(div().flex_1().child(input(&i.notes))).child(chip("dr-finish", t!("fleet-dr-finish-drill"), false).on_click(move |_ev, _w, cx| a_finish.update(cx, |this, cx| this.finish_drill(cx)))).into_any_element(),
                        None => chip("dr-start", t!("fleet-dr-start-a-drill"), false).on_click(move |_ev, _w, cx| a_drill.update(cx, |this, cx| this.start_drill(cx))).into_any_element(),
                    })
                    .children(p.drills.iter().rev().map(|d| {
                        use chrono::TimeZone;
                        let when = chrono::Local.timestamp_opt(d.started_at, 0).single().map(|t| t.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default();
                        mono(10.5, if d.met_rto { OK } else { CRIT }).child(t!("fleet-dr-min", when = when, minutes = d.minutes, d = if d.met_rto { t!("fleet-dr-met-the-rto") } else { t!("fleet-dr-missed-the-rto") }, d2 = if d.notes.is_empty() { String::new() } else { format!(" · {}", d.notes) }))
                    })),
            )
        });
    div().id("dr-page").flex_1().min_h(px(0.0)).overflow_y_scrollbar().child(div().p(px(18.0)).flex().gap(px(20.0)).child(list).child(form)).into_any_element()
}
