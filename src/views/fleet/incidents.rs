//! Fleet Setup → INCIDENTS (ERR-149).

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::incidents::IncidentsState;
use crate::app::CrowApp;
use crate::incident::EventKind;
use crate::metrics::alerts::Alert;
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn chip(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .px(px(8.0))
        .py(px(3.0))
        .border_1()
        .border_color(BORDER_DEFAULT)
        .bg(BG_CONTROL)
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .text_color(TEXT_SECONDARY)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .child(text.into())
}

fn when(ts: i64) -> String {
    use chrono::TimeZone;
    chrono::Local.timestamp_opt(ts, 0).single().map(|t| t.format("%m-%d %H:%M:%S").to_string()).unwrap_or_default()
}

pub fn incidents_page(fleet: &FleetState, st: &IncidentsState, recent: &[Alert], app: Entity<CrowApp>) -> impl IntoElement {
    let name = |id: &str| fleet.servers.iter().find(|s| s.id == id).map(|s| s.name.clone()).unwrap_or_else(|| id.to_string());
    let (app_sum, app_exp) = (app.clone(), app.clone());
    // Pick an incident: a recent alert, or a window on a server.
    let picker = div()
        .w(px(360.0))
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(mono(10.0, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("FROM AN ALERT (LAST 7 DAYS)"))
        .when(recent.is_empty(), |d| d.child(mono(10.0, TEXT_DIM).child("No alerts in the last 7 days.")))
        .children(recent.iter().take(40).map(|a| {
            let (app, id) = (app.clone(), a.id.clone());
            let color = if a.resolved_at.is_some() { TEXT_DIM } else if a.level == "CRIT" { CRIT } else { WARN };
            div()
                .id(ElementId::Name(format!("incident-alert-{}", a.id).into()))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .px(px(8.0))
                .py(px(5.0))
                .border_1()
                .border_color(BORDER_PANEL)
                .bg(BG_PANEL)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _w, cx| {
                    let id = id.clone();
                    app.update(cx, |this, cx| this.open_incident_for_alert(&id, cx))
                })
                .child(div().flex().gap(px(6.0)).child(mono(9.5, color).font_weight(FontWeight::BOLD).child(a.level.clone())).child(mono(9.5, TEXT_TERTIARY).child(name(&a.server_id))).child(div().flex_1()).child(mono(9.0, TEXT_FAINT).child(format!("{}{}", when(a.opened_at), if a.resolved_at.is_some() { " · resolved" } else { "" }))))
                .child(mono(10.0, TEXT_SECONDARY).whitespace_nowrap().overflow_hidden().text_ellipsis().child(a.detail.clone()))
        }))
        .child(mono(10.0, TEXT_SECONDARY).font_weight(FontWeight::BOLD).mt(px(8.0)).child("OR A WINDOW ON A SERVER"))
        .children(fleet.servers.iter().map(|s| {
            div().flex().items_center().gap(px(4.0)).child(mono(10.0, TEXT_PRIMARY).w(px(140.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(s.name.clone())).children([(3600i64, "1H"), (6 * 3600, "6H"), (86_400, "24H")].into_iter().map(|(secs, t)| {
                let (app, id) = (app.clone(), s.id.clone());
                chip(format!("incident-win-{}-{secs}", s.id), t).on_click(move |_ev, _w, cx| {
                    let id = id.clone();
                    app.update(cx, |this, cx| this.open_incident_window(&id, secs, cx))
                })
            }))
        }));

    let timeline = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(match &st.scope {
            None => mono(10.5, TEXT_DIM).child("Pick an alert or a window: the timeline merges the alerts, Crow's own changes, failed checks and error logs (err and worse) around it.").into_any_element(),
            Some(s) => div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(div().flex_1().min_w(px(0.0)).flex().flex_col().child(mono(11.5, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(s.title.clone())).child(mono(9.5, TEXT_FAINT).child(format!("{} → {} · {} events{}", when(s.from), when(s.to), st.events.len(), if st.loading > 0 { " · reading logs…" } else { "" }))))
                .child(chip("btn-incident-summary", if st.summarizing { "SUMMARIZING…" } else { "SUMMARIZE (CLANKER)" }).on_click(move |_ev, _w, cx| app_sum.update(cx, |this, cx| this.summarize_incident(cx))))
                .child(chip("btn-incident-export", "EXPORT (.md)").on_click(move |_ev, _w, cx| app_exp.update(cx, |this, cx| this.export_incident(cx))))
                .into_any_element(),
        })
        .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).child(m)))
        .children(st.summary.as_ref().map(|s| match s {
            Ok((text, via)) => div().p(px(10.0)).bg(BG_PANEL).border_1().border_color(BORDER_PANEL).flex().flex_col().gap(px(4.0)).child(mono(10.5, TEXT_PRIMARY).line_height(px(16.0)).child(text.clone())).child(mono(9.0, TEXT_FAINT).child(format!("via {via} · a likely sequence, an AI's reading: check it against the timeline"))),
            Err(e) => mono(10.5, CRIT).child(e.clone()),
        }))
        .when(st.scope.is_some() && st.events.is_empty() && st.loading == 0, |d| d.child(mono(10.5, TEXT_DIM).child("Nothing in this window.")))
        .children(st.events.iter().map(|e| {
            let color = match e.kind {
                EventKind::AlertOpened if e.level == "CRIT" => CRIT,
                EventKind::AlertOpened => WARN,
                EventKind::AlertResolved => OK,
                EventKind::Change => hex_rgb(0x8ab4ff),
                EventKind::CheckFailed => CRIT,
                EventKind::Log => TEXT_DIM,
            };
            div()
                .flex()
                .items_start()
                .gap(px(8.0))
                .px(px(6.0))
                .py(px(3.0))
                .border_b_1()
                .border_color(BORDER_ROW)
                .child(mono(9.5, TEXT_FAINT).w(px(100.0)).flex_none().child(when(e.ts)))
                .child(mono(9.5, color).font_weight(FontWeight::BOLD).w(px(70.0)).flex_none().child(e.kind.label()))
                .child(mono(9.5, TEXT_TERTIARY).w(px(110.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(e.server.clone()))
                .child(mono(10.5, if e.kind == EventKind::Log { TEXT_SECONDARY } else { TEXT_PRIMARY }).flex_1().min_w(px(0.0)).child(e.text.clone()))
        }));

    div().id("incidents-page").flex_1().min_h(px(0.0)).overflow_y_scrollbar().child(div().p(px(18.0)).flex().gap(px(20.0)).child(picker).child(timeline))
}
