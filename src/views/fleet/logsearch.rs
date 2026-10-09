//! Fleet Setup → LOG SEARCH (ERR-147).

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::logsearch::LogSearchState;
use crate::app::patching::Target;
use crate::app::CrowApp;
use crate::journal::JournalTimeRange;
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

fn label(t: &'static str) -> Div {
    mono(9.5, TEXT_DIMMER).font_weight(FontWeight::SEMIBOLD).w(px(70.0)).flex_none().child(t)
}

fn input(i: Option<&Entity<gpui_kit::component::input::InputState>>) -> impl IntoElement {
    div().flex_1().children(i.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))
}

pub fn log_search_page(fleet: &FleetState, st: &LogSearchState, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_run, app_save, app_sum) = (app.clone(), app.clone(), app.clone());
    let prios: [(Option<u8>, &str); 5] = [(None, "any"), (Some(3), "err+"), (Some(4), "warning+"), (Some(2), "crit+"), (Some(6), "info+")];
    let ranges = [JournalTimeRange::Last15m, JournalTimeRange::Last1h, JournalTimeRange::Last6h, JournalTimeRange::Last24h, JournalTimeRange::Last7d];
    let terms = crate::journal::search_terms(&st.searched);
    let per_server: Vec<(String, Result<usize, String>)> = st.results.iter().map(|(_, n, r)| (n.clone(), r.as_ref().map(|l| l.len()).map_err(|e| e.clone()))).collect();
    div()
        .id("logsearch-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1200.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(div().flex().items_center().gap(px(8.0)).child(label("SEARCH")).child(input(st.text.as_ref())).child(div().w(px(260.0)).child(input(st.unit.as_ref()))))
                .child(div().flex().items_center().gap(px(6.0)).child(label("PRIORITY")).children(prios.iter().map(|(p, t)| {
                    let (app, p) = (app.clone(), *p);
                    chip(format!("ls-prio-{t}"), *t, st.priority == p).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_log_priority(p, cx)))
                })))
                .child(div().flex().items_center().gap(px(6.0)).child(label("WHEN")).children(ranges.iter().map(|r| {
                    let (app, r) = (app.clone(), *r);
                    chip(format!("ls-range-{}", r.label()), r.label(), st.range == r).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_log_range(r, cx)))
                })).child(div().w(px(16.0))).child(mono(9.5, TEXT_DIMMER).child("AT MOST")).children([50usize, 200, 1000].into_iter().map(|n| {
                    let app = app.clone();
                    chip(format!("ls-limit-{n}"), format!("{n}/server"), st.limit == n).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_log_limit(n, cx)))
                })))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(6.0))
                        .child(label("WHERE"))
                        .child({
                            let app = app.clone();
                            chip("ls-target-all", "every server", st.target == Target::All).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_log_target(Target::All, cx)))
                        })
                        .children(fleet.groups.iter().map(|g| {
                            let (app, name) = (app.clone(), g.clone());
                            chip(format!("ls-target-{g}"), format!("group {g}"), st.target == Target::Group(g.clone())).on_click(move |_ev, _w, cx| {
                                let n = name.clone();
                                app.update(cx, |this, cx| this.set_log_target(Target::Group(n), cx))
                            })
                        }))
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("btn-ls-run")
                                .px(px(14.0))
                                .py(px(5.0))
                                .border_1()
                                .border_color(OK)
                                .bg(OK.opacity(0.1))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK)
                                .cursor_pointer()
                                .hover(|s| s.bg(OK.opacity(0.2)))
                                .on_click(move |_ev, _w, cx| app_run.update(cx, |this, cx| this.run_log_search(cx)))
                                .child(if st.running.is_empty() { "SEARCH ⏎".to_string() } else { format!("SEARCHING {}…", st.running.len()) }),
                        ),
                )
                .child(div().flex().items_center().gap(px(8.0)).child(label("SAVE AS")).child(div().w(px(300.0)).child(input(st.name.as_ref()))).child(chip("btn-ls-save", "SAVE", false).on_click(move |_ev, _w, cx| app_save.update(cx, |this, cx| this.save_log_search(cx)))))
                .children(st.message.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).child(m)))
                // Saved searches.
                .when(!st.saved.is_empty(), |d| {
                    d.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(mono(10.0, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("SAVED"))
                            .children(st.saved.iter().map(|s| {
                                let (a_run, a_alert, a_del, id1, id2, id3) = (app.clone(), app.clone(), app.clone(), s.id.clone(), s.id.clone(), s.id.clone());
                                let alert = if s.alert { format!("ALERT {}", s.level) } else { "ALERT OFF".into() };
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .border_1()
                                    .border_color(BORDER_PANEL)
                                    .bg(BG_PANEL)
                                    .child(mono(10.5, TEXT_PRIMARY).w(px(200.0)).flex_none().child(s.name.clone()))
                                    .child(mono(10.0, TEXT_DIM).flex_1().min_w(px(0.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(format!("\"{}\"{}{} · {}", s.text, if s.unit.is_empty() { String::new() } else { format!(" in {}", s.unit) }, s.priority.map(|p| format!(" · prio ≤{p}")).unwrap_or_default(), if s.group.is_empty() { "every server".to_string() } else { format!("group {}", s.group) })))
                                    .child(chip(format!("ls-saved-run-{}", s.id), "RUN", false).on_click(move |_ev, window, cx| {
                                        let id = id1.clone();
                                        a_run.update(cx, |this, cx| this.run_saved_search(&id, window, cx))
                                    }))
                                    .child(chip(format!("ls-saved-alert-{}", s.id), alert, s.alert).on_click(move |_ev, _w, cx| {
                                        let id = id2.clone();
                                        a_alert.update(cx, |this, cx| this.cycle_saved_alert(&id, cx))
                                    }))
                                    .child(chip(format!("ls-saved-del-{}", s.id), "DELETE", false).on_click(move |_ev, _w, cx| {
                                        let id = id3.clone();
                                        a_del.update(cx, |this, cx| this.delete_saved_search(&id, cx))
                                    }))
                            }))
                            .child(mono(9.5, TEXT_FAINT).child("An alerting search runs every 10 minutes over the last 15, on its servers, while Crow is open; the alert resolves once it stops matching (and is mailed with the Email plugin).")),
                    )
                })
                // Results.
                .when(!st.results.is_empty() || !st.running.is_empty(), |d| {
                    d.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(6.0))
                            .child(mono(10.0, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(format!("{} LINES", st.hits.len())))
                            .children(per_server.iter().map(|(name, r)| match r {
                                Ok(n) => mono(9.5, if *n >= st.limit { WARN } else { TEXT_DIM }).px(px(5.0)).bg(BG_CHIP).child(format!("{name}: {n}{}", if *n >= st.limit { " (limit)" } else { "" })),
                                Err(e) => mono(9.5, CRIT).px(px(5.0)).bg(BG_CHIP).child(format!("{name}: {e}")),
                            }))
                            .child(div().flex_1())
                            .when(!st.hits.is_empty(), |d| d.child(chip("btn-ls-summarize", if st.summarizing { "SUMMARIZING…" } else { "SUMMARIZE (CLANKER)" }, false).on_click(move |_ev, _w, cx| app_sum.update(cx, |this, cx| this.summarize_log_search(cx))))),
                    )
                })
                .children(st.summary.as_ref().map(|s| match s {
                    Ok((text, via)) => div().p(px(10.0)).bg(BG_PANEL).border_1().border_color(BORDER_PANEL).flex().flex_col().gap(px(4.0)).child(mono(10.5, TEXT_PRIMARY).line_height(px(16.0)).child(text.clone())).child(mono(9.0, TEXT_FAINT).child(format!("via {via} · only the matching lines were sent · an AI's reading: check it"))),
                    Err(e) => mono(10.5, CRIT).child(e.clone()),
                }))
                .children(st.hits.iter().take(500).map(|h| {
                    let e = &h.entry;
                    let msg = e.message.clone();
                    let ranges = crate::journal::match_ranges(&msg, &terms);
                    let highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = ranges.into_iter().map(|r| (r, HighlightStyle { background_color: Some(WARN.opacity(0.3).into()), ..Default::default() })).collect();
                    div()
                        .flex()
                        .items_start()
                        .gap(px(8.0))
                        .px(px(6.0))
                        .py(px(2.0))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .child(mono(9.5, TEXT_FAINT).w(px(150.0)).flex_none().child(e.timestamp_formatted.clone()))
                        .child(mono(10.0, TEXT_TERTIARY).w(px(120.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(h.server.clone()))
                        .child(mono(9.5, e.priority.color()).w(px(56.0)).flex_none().child(e.priority.label()))
                        .child(mono(9.5, TEXT_DIM).w(px(150.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(if e.unit.is_empty() { e.syslog_identifier.clone() } else { e.unit.clone() }))
                        .child(div().flex_1().min_w(px(0.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_SECONDARY).child(StyledText::new(msg).with_highlights(highlights)))
                })),
        )
}
