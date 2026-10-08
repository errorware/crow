//! The Email plugin's settings, inside its card (ERR-139): the SMTP server,
//! who gets mail and when, SAVE and SEND TEST.

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::notify::{EmailInputs, EmailState};
use crate::app::CrowApp;
use crate::notify::email::{Delivery, EmailSettings, Security};
use crate::theme::*;

fn label(text: &'static str) -> Div {
    div().font_family(FONT_MONO).text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(text)
}

fn field(text: &'static str, input: &Entity<InputState>) -> Div {
    div().flex_1().min_w(px(0.0)).flex().flex_col().gap(px(4.0)).child(label(text)).child(Input::new(input).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0)))
}

fn segment(id: String, text: &'static str, selected: bool) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .flex_1()
        .py(px(5.0))
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(if selected { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
        .bg(if selected { BG_CONTROL_ALT } else { hex_rgba(0, 0.0) })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if selected { TEXT_PRIMARY } else { TEXT_DIM })
        .child(text)
}

fn button(id: &'static str, text: &'static str, primary: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .h(px(26.0))
        .px(px(12.0))
        .border_1()
        .border_color(if primary { OK } else { BORDER_DEFAULT })
        .bg(if primary { OK_BG } else { hex_rgba(0, 0.0) })
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(if primary { OK } else { TEXT_SECONDARY })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .child(text)
}

fn note(ok: bool, text: String) -> Div {
    div().font_family(FONT_MONO).text_size(px(10.0)).line_height(px(14.0)).text_color(if ok { OK } else { CRIT }).child(text)
}

fn ago(ts: i64) -> String {
    let secs = (chrono::Utc::now().timestamp() - ts).max(0);
    match secs {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86_399 => format!("{} h ago", secs / 3600),
        _ => format!("{} d ago", secs / 86_400),
    }
}

pub fn email_form(app: Entity<CrowApp>, st: &EmailState, inputs: Option<&EmailInputs>) -> impl IntoElement {
    let Some(inputs) = inputs else { return div().into_any_element() };
    let defaults = EmailSettings::default();
    let (crit, warn) = (st.crit.unwrap_or(defaults.crit), st.warn.unwrap_or(defaults.warn));
    let security_row = div().flex().gap(px(4.0)).children(Security::ALL.into_iter().map(|s| {
        let app = app.clone();
        segment(format!("email-sec-{}", s.label()), s.label(), st.security == s).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.set_email_security(s, window, cx)))
    }));
    let delivery_row = |is_crit: bool, current: Delivery| {
        let app = app.clone();
        div().flex().gap(px(4.0)).children(Delivery::ALL.into_iter().map(move |d| {
            let app = app.clone();
            segment(format!("email-{}-{:?}", if is_crit { "crit" } else { "warn" }, d), d.label(), current == d).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_email_delivery(is_crit, d, cx)))
        }))
    };
    let d = &st.dispatch;
    let activity = match (&d.last_error, d.last_sent) {
        (Some(e), _) => Some(note(false, format!("Last send failed: {e}{}", if d.outbox.is_empty() { String::new() } else { format!(" · {} waiting, retried every 5 min", d.outbox.len()) }))),
        (None, Some(t)) => Some(note(true, format!("Last mail sent {}{}", ago(t), if d.queue.is_empty() { String::new() } else { format!(" · {} in the next digest", d.queue.len()) }))),
        (None, None) if !d.queue.is_empty() => Some(note(true, format!("{} in the next digest", d.queue.len()))),
        _ => None,
    };
    let (app_save, app_test) = (app.clone(), app.clone());
    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .pt(px(10.0))
        .border_t_1()
        .border_color(BORDER_PANEL)
        .child(div().flex().gap(px(8.0)).child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap(px(4.0)).child(label("SECURITY")).child(security_row)).child(div().w(px(260.0)).flex().gap(px(8.0)).child(field("SMTP SERVER", &inputs.host)).child(div().w(px(70.0)).child(field("PORT", &inputs.port)))))
        .when(st.security == Security::None, |d| d.child(note(false, "No encryption: the password and mail cross the network in the clear. Only for a relay on this machine or your own network.".into())))
        .child(div().flex().gap(px(8.0)).child(field("USERNAME", &inputs.username)).child(field("PASSWORD", &inputs.password)))
        .child(div().flex().gap(px(8.0)).child(field("FROM", &inputs.from)).child(field("TO (comma separated)", &inputs.to)))
        .child(div().flex().flex_col().gap(px(4.0)).child(label("CRITICAL ALERTS")).child(delivery_row(true, crit)))
        .child(div().flex().flex_col().gap(px(4.0)).child(label("WARNINGS")).child(delivery_row(false, warn)))
        .child(div().flex().gap(px(8.0)).child(field("DIGEST EVERY (MINUTES)", &inputs.digest)).child(field("QUIET HOURS (DIGESTS WAIT)", &inputs.quiet)))
        .child(div().font_family(FONT_MONO).text_size(px(10.0)).line_height(px(14.0)).text_color(TEXT_FAINT).child(
            "Resolved alerts send nothing. Alerts already open when you switch this on aren't mailed. Crow has no agent: mail only goes out while Crow is running.",
        ))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(button("btn-email-save", "SAVE", true).on_click(move |_ev, _window, cx| app_save.update(cx, |this, cx| this.save_email(cx))))
                .child(button("btn-email-test", if st.testing { "SENDING…" } else { "SEND TEST" }, false).on_click(move |_ev, _window, cx| app_test.update(cx, |this, cx| this.send_test_email(cx))))
                .children(st.form_message.clone().map(|(ok, m)| div().flex_1().min_w(px(0.0)).child(note(ok, m)))),
        )
        .children(st.test_result.clone().map(|(ok, m)| note(ok, if ok { m } else { format!("Test failed: {m}") })))
        .children(activity)
        .into_any_element()
}
