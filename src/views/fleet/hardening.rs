//! Fleet Setup → HARDENING (ERR-145): every server's findings, a guarded
//! fix for each, accepted risks, and the report.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::security::hardening::{Accepted, Finding};
use crate::theme::*;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn action(id: impl Into<SharedString>, text: impl Into<SharedString>, color: Rgba) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .flex_none()
        .items_center()
        .h(px(22.0))
        .px(px(8.0))
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

fn header_button(id: &'static str, icon: Option<TablerIcon>, text: String) -> Stateful<Div> {
    div()
        .id(id)
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
        .children(icon.map(|i| inherited_icon(i, px(12.0))))
        .child(text)
}

/// One server's row: (id, name, login user, findings — `None` when its
/// posture wasn't read).
pub type ServerFindings = (String, String, String, Option<Vec<(Finding, Option<Accepted>)>>);

pub fn hardening_page(rows: &[ServerFindings], reason: Option<&Entity<InputState2>>, message: Option<(bool, String)>, scanning: bool, checked_at: i64, app: Entity<CrowApp>) -> impl IntoElement {
    let open: usize = rows.iter().filter_map(|r| r.3.as_ref()).map(|f| f.iter().filter(|(_, a)| a.is_none()).count()).sum();
    let crit: usize = rows.iter().filter_map(|r| r.3.as_ref()).map(|f| f.iter().filter(|(x, a)| a.is_none() && x.critical()).count()).sum();
    let accepted: usize = rows.iter().filter_map(|r| r.3.as_ref()).map(|f| f.iter().filter(|(_, a)| a.is_some()).count()).sum();
    // Fix a kind everywhere: codes with more than one open finding.
    let mut codes: Vec<(&'static str, usize)> = Vec::new();
    for f in rows.iter().filter_map(|r| r.3.as_ref()).flatten().filter(|(_, a)| a.is_none()) {
        match codes.iter_mut().find(|(c, _)| *c == f.0.code()) {
            Some(e) => e.1 += 1,
            None => codes.push((f.0.code(), 1)),
        }
    }
    let (app_check, app_report) = (app.clone(), app.clone());
    let checked = if checked_at == 0 { "not yet".to_string() } else { format!("{} min ago", ((chrono::Utc::now().timestamp() - checked_at).max(0)) / 60) };
    div()
        .id("hardening-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1100.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("POSTURE"))
                        .child(mono(10.5, if crit > 0 { CRIT } else if open > 0 { WARN } else { OK }).child(format!("{open} open ({crit} critical) · {accepted} accepted")))
                        .child(mono(9.5, TEXT_FAINT).child(format!("read {checked}, as root where Crow can (sshd -T, the firewall's status, ss)")))
                        .child(div().flex_1())
                        .child(header_button("btn-hardening-check", Some(TablerIcon::Refresh), if scanning { "READING…".into() } else { "CHECK NOW".into() }).on_click(move |_ev, _w, cx| app_check.update(cx, |this, cx| this.check_posture(cx))))
                        .child(header_button("btn-hardening-report", None, "EXPORT REPORT (.md)".into()).on_click(move |_ev, _w, cx| app_report.update(cx, |this, cx| this.export_posture_report(cx)))),
                )
                .children(message.map(|(ok, m)| mono(10.5, if ok { OK } else { WARN }).line_height(px(15.0)).child(m)))
                .child(div().flex().items_center().gap(px(10.0)).child(mono(9.5, TEXT_DIMMER).font_weight(FontWeight::SEMIBOLD).child("REASON TO ACCEPT")).child(div().flex_1().max_w(px(560.0)).children(reason.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))))
                .when(codes.iter().any(|(_, n)| *n > 1), |d| {
                    d.child(div().flex().flex_wrap().gap(px(6.0)).child(mono(9.5, TEXT_FAINT).mr(px(4.0)).child("fix everywhere:")).children(codes.into_iter().filter(|(_, n)| *n > 1).map(|(code, n)| {
                        let app = app.clone();
                        action(format!("fix-all-{code}"), format!("{code} ({n})"), OK).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.plan_hardening_fix(code, None, window, cx)))
                    })))
                })
                .child(mono(9.5, TEXT_FAINT).line_height(px(13.0)).child("Every fix first proves a fresh key login works, then changes one thing, checks it took (sshd -t, sshd -T, the firewall's status), and proves a fresh login still works. Anything that fails is undone. Crow never sets PermitRootLogin no where it logs in as root, and a firewall always allows the SSH port."))
                .children(rows.iter().map(|(id, name, user, findings)| {
                    div()
                        .flex()
                        .flex_col()
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(10.0))
                                .h(px(28.0))
                                .bg(BG_SUBHEAD)
                                .child(mono(11.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(name.clone()))
                                .child(mono(9.5, TEXT_FAINT).child(format!("Crow logs in as {user}")))
                                .child(div().flex_1())
                                .child(match findings {
                                    None => mono(9.5, TEXT_DIM).child("not read (unreachable, or not checked yet)"),
                                    Some(f) if f.is_empty() => mono(9.5, OK).child("nothing found"),
                                    Some(f) => mono(9.5, TEXT_DIM).child(format!("{} finding{}", f.len(), if f.len() == 1 { "" } else { "s" })),
                                }),
                        )
                        .children(findings.iter().flatten().map(|(f, accepted)| {
                            let code = f.code();
                            let color = match (accepted, f.critical()) {
                                (Some(_), _) => TEXT_DIM,
                                (None, true) => CRIT,
                                (None, false) => WARN,
                            };
                            let (app_fix, app_acc, app_un, sid, sid2, sid3) = (app.clone(), app.clone(), app.clone(), id.clone(), id.clone(), id.clone());
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(10.0))
                                .min_h(px(32.0))
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .bg(BG_PANEL)
                                .child(div().flex_none().size(px(7.0)).rounded_full().bg(color))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .flex()
                                        .flex_col()
                                        .child(mono(10.5, if accepted.is_some() { TEXT_DIM } else { TEXT_PRIMARY }).child(f.describe()))
                                        .child(match accepted {
                                            Some(a) => mono(9.5, TEXT_FAINT).child(format!("accepted by {}: {}", a.by, a.reason)),
                                            None => mono(9.5, TEXT_FAINT).child(format!("fix: {}", f.fix(user))),
                                        }),
                                )
                                .when(accepted.is_none() && f.fixable(user), |d| {
                                    d.child(action(format!("fix-{id}-{code}"), "FIX", OK).on_click(move |_ev, window, cx| {
                                        let sid = sid.clone();
                                        app_fix.update(cx, |this, cx| this.plan_hardening_fix(code, Some(&sid), window, cx))
                                    }))
                                })
                                .when(accepted.is_none(), |d| {
                                    d.child(action(format!("accept-{id}-{code}"), "ACCEPT", WARN).on_click(move |_ev, _w, cx| {
                                        let sid = sid2.clone();
                                        app_acc.update(cx, |this, cx| this.accept_risk(&sid, code, cx))
                                    }))
                                })
                                .when(accepted.is_some(), |d| {
                                    d.child(action(format!("unaccept-{id}-{code}"), "FLAG AGAIN", TEXT_DIM).on_click(move |_ev, _w, cx| {
                                        let sid = sid3.clone();
                                        app_un.update(cx, |this, cx| this.unaccept_risk(&sid, code, cx))
                                    }))
                                })
                        }))
                })),
        )
}

type InputState2 = gpui_kit::component::input::InputState;
