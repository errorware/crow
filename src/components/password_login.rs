//! The "turn off SSH password login" dialog (ERR-34): what Crow checked,
//! what it will do, who loses password login, and then the step log.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::password_login::{PasswordLoginFlow, PasswordLoginStage};
use crate::app::CrowApp;
use crate::security::sshd_passwords::DROPIN;
use crate::theme::*;

fn button(id: &'static str, label: &'static str, color: Rgba, enabled: bool, on_click: impl Fn(&mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(if enabled { color.opacity(0.6) } else { border_default() })
        .text_color(if enabled { color } else { text_fainter() })
        .font_weight(FontWeight::BOLD)
        .when(enabled, |d| d.cursor_pointer().hover(|s| s.bg(bg_row_hover())).on_click(move |_ev, _window, cx| on_click(cx)))
        .child(label)
}

fn line(text: impl Into<SharedString>, color: Rgba) -> Div {
    div().min_w(px(0.0)).line_height(px(15.0)).text_color(color).child(text.into())
}

pub fn password_login_dialog(flow: &PasswordLoginFlow, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_scrim, a_close, a_run, a_ack) = (app.clone(), app.clone(), app.clone(), app);
    let running = matches!(flow.stage, PasswordLoginStage::Running);
    let body: Div = match &flow.stage {
        PasswordLoginStage::Checking => div().child(line(t!("password-login-reading-sshds-settings"), text_dimmer())),
        PasswordLoginStage::Unavailable(why) => div().child(line(why.clone(), theme().warn)),
        PasswordLoginStage::Running => div().child(line(t!("password-login-working-checking-a-fresh-key"), text_dimmer())),
        PasswordLoginStage::Done(result) => {
            let (log, ok) = match result {
                Ok(l) => (l, true),
                Err(l) => (l, false),
            };
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .children(log.iter().map(|l| line(l.clone(), if l.starts_with('✕') || l.starts_with('⚠') { theme().crit } else if l.starts_with('↺') { theme().warn } else { text_secondary() })))
                .child(line(if ok { t!("password-login-done-keep-a-key-for") } else { t!("password-login-password-login-is-still-on") }, if ok { theme().ok } else { theme().warn }))
        }
        PasswordLoginStage::Ready(pre) => div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(line(t!("password-login-password-login-is-on-anyone"), text_secondary()))
            .child(line(t!("password-login-what-crow-will-do-stopping"), text_muted()))
            .children(crate::components::english_beside::english_beside(t_beside!("password-login-what-crow-will-do-stopping")))
            .children(
                [
                    t!("password-login-1-check-that-a-fresh").to_string(),
                    t!("password-login-2-write-passwordauthentication-no-kbdinteractiveauthentication", dropin = DROPIN),
                    t!("password-login-3-validate-with-sshd-t").to_string(),
                    t!("password-login-4-reload-sshd-open-sessions").to_string(),
                ]
                .into_iter()
                .map(|s| line(s, text_secondary())),
            )
            .children((!pre.password_users.is_empty()).then(|| {
                let ack = flow.ack;
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(line(t!("password-login-these-accounts-have-a-password", pre = pre.password_users.join(", ")), theme().warn))
                    .child(
                        div()
                            .id("pw-login-ack")
                            .flex()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .on_click(move |_ev, _window, cx| a_ack.update(cx, |this, cx| this.toggle_password_login_ack(cx)))
                            .child(div().text_color(if ack { theme().warn } else { text_dimmer() }).child(if ack { "[x]" } else { "[ ]" }))
                            .child(line(t!("password-login-i-understand"), text_secondary())),
                    )
            })),
    };
    let can_run = matches!(&flow.stage, PasswordLoginStage::Ready(p) if p.password_users.is_empty() || flow.ack);
    let show_run = matches!(flow.stage, PasswordLoginStage::Ready(_));

    div()
        .id("pw-login-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| a_scrim.update(cx, |this, cx| this.close_password_login(cx)))
        .child(
            div()
                .id("pw-login")
                .w(px(640.0))
                .p(px(16.0))
                .bg(bg_panel())
                .border_1()
                .border_color(theme().warn.opacity(0.6))
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .font_family(FONT_MONO)
                .text_size(font_px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(div().text_size(font_px(13.0)).font_weight(FontWeight::BOLD).text_color(text_primary()).child(t!("password-login-turn-off-ssh-password-login", server_name = flow.server_name)))
                .child(body)
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .children(show_run.then(|| button("pw-login-run", t!("password-login-turn-off-password-login"), theme().warn, can_run, move |cx| a_run.update(cx, |this, cx| this.run_password_login_off(cx)))))
                        .child(button("pw-login-close", if show_run { t!("password-login-cancel") } else { t!("password-login-close") }, text_secondary(), !running, move |cx| a_close.update(cx, |this, cx| this.close_password_login(cx)))),
                ),
        )
}
