//! The status bar along the bottom of the Server and Fleet screens: open
//! alerts, Crow's latest notice, and Crow's version.

use gpui_kit::*;

use crate::app::{CrowApp, Screen};
use crate::theme::*;
use crate::views::fleet::alert_lines::AlertLine;

pub fn status_bar(alerts: &[AlertLine], notice: Option<&str>, app: Entity<CrowApp>) -> impl IntoElement {
    let open: Vec<&AlertLine> = alerts.iter().filter(|a| !a.resolved && !a.acknowledged).collect();
    let crit = open.iter().any(|a| a.level == "CRIT");
    let (dot, summary) = match open.len() {
        0 => (OK, "no open alerts".to_string()),
        1 => (if crit { CRIT } else { WARN }, "1 open alert".to_string()),
        n => (if crit { CRIT } else { WARN }, format!("{n} open alerts")),
    };
    // Alerts live on the Fleet screen: clicking takes you there.
    let latest = open.first().map(|a| format!("{} · {} · {}", a.host, a.message, a.age));
    let app_alerts = app.clone();
    div()
        .id("status-bar")
        .h(px(24.0))
        .flex_none()
        .flex()
        .items_center()
        .bg(BG_CHROME)
        .border_t_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .text_color(TEXT_DIM)
        .child(
            div()
                .id("status-alerts")
                .h_full()
                .flex()
                .items_center()
                .gap(px(7.0))
                .px(px(12.0))
                .min_w(px(0.0))
                .flex_shrink(1.0)
                .overflow_hidden()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| app_alerts.update(cx, |this, cx| this.set_screen(Screen::Fleet, cx)))
                .child(div().size(px(6.0)).rounded_full().bg(dot).flex_none())
                .child(div().flex_none().text_color(if open.is_empty() { TEXT_DIM } else { TEXT_SECONDARY }).child(summary))
                .children(latest.map(|l| div().min_w(px(0.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().text_color(TEXT_DIMMER).child(l))),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .children(notice.map(|n| {
                    let app = app.clone();
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .min_w(px(0.0))
                        .child(div().min_w(px(0.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().text_color(TEXT_SECONDARY).child(n.to_string()))
                        .child(
                            div()
                                .id("status-notice-dismiss")
                                .flex_none()
                                .text_color(TEXT_FAINT)
                                .cursor_pointer()
                                .hover(|s| s.text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _window, cx| {
                                    app.update(cx, |this, cx| {
                                        this.fleet.notice = None;
                                        cx.notify();
                                    })
                                })
                                .child("×"),
                        )
                })),
        )
        .child(
            div()
                .h_full()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(12.0))
                .px(px(12.0))
                .border_l_1()
                .border_color(BORDER_PANEL)
                .text_color(TEXT_FAINT)
                .child("agentless · ssh")
                .child(concat!("crow v", env!("CARGO_PKG_VERSION"))),
        )
}
