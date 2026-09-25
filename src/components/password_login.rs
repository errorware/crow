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
        .border_color(if enabled { color.opacity(0.6) } else { BORDER_DEFAULT })
        .text_color(if enabled { color } else { TEXT_FAINTER })
        .font_weight(FontWeight::BOLD)
        .when(enabled, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| on_click(cx)))
        .child(label)
}

fn line(text: impl Into<SharedString>, color: Rgba) -> Div {
    div().min_w(px(0.0)).line_height(px(15.0)).text_color(color).child(text.into())
}

pub fn password_login_dialog(flow: &PasswordLoginFlow, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_scrim, a_close, a_run, a_ack) = (app.clone(), app.clone(), app.clone(), app);
    let running = matches!(flow.stage, PasswordLoginStage::Running);
    let body: Div = match &flow.stage {
        PasswordLoginStage::Checking => div().child(line("Reading sshd's settings…", TEXT_DIMMER)),
        PasswordLoginStage::Unavailable(why) => div().child(line(why.clone(), WARN)),
        PasswordLoginStage::Running => div().child(line("Working: checking a fresh key login, writing the drop-in, validating, reloading, checking again…", TEXT_DIMMER)),
        PasswordLoginStage::Done(result) => {
            let (log, ok) = match result {
                Ok(l) => (l, true),
                Err(l) => (l, false),
            };
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .children(log.iter().map(|l| line(l.clone(), if l.starts_with('✕') || l.starts_with('⚠') { CRIT } else if l.starts_with('↺') { WARN } else { TEXT_SECONDARY })))
                .child(line(if ok { "Done. Keep a key for this server somewhere safe: passwords won't get you in any more." } else { "Password login is still on." }, if ok { OK } else { WARN }))
        }
        PasswordLoginStage::Ready(pre) => div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(line("Password login is ON. Anyone who guesses or steals a password can get in, and bots try constantly. With it off, only SSH keys work.", TEXT_SECONDARY))
            .child(line("What Crow will do, stopping and undoing at the first problem:", TEXT_MUTED))
            .children(
                [
                    "1. Check that a fresh key login works (a new connection, not the one Crow holds).".to_string(),
                    format!("2. Write {DROPIN} (PasswordAuthentication no, KbdInteractiveAuthentication no); it's read before other drop-ins, so a cloud image's own setting can't override it."),
                    "3. Validate with sshd -t, and confirm with sshd -T that passwords are really off.".to_string(),
                    "4. Reload sshd (open sessions stay up), then check a fresh key login again; if that fails, remove the file and reload.".to_string(),
                ]
                .into_iter()
                .map(|s| line(s, TEXT_SECONDARY)),
            )
            .children((!pre.password_users.is_empty()).then(|| {
                let ack = flow.ack;
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(line(format!("These accounts have a password and will lose SSH login with it: {}. Make sure each one has a key first.", pre.password_users.join(", ")), WARN))
                    .child(
                        div()
                            .id("pw-login-ack")
                            .flex()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .on_click(move |_ev, _window, cx| a_ack.update(cx, |this, cx| this.toggle_password_login_ack(cx)))
                            .child(div().text_color(if ack { WARN } else { TEXT_DIMMER }).child(if ack { "[x]" } else { "[ ]" }))
                            .child(line("I understand", TEXT_SECONDARY)),
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
                .bg(BG_PANEL)
                .border_1()
                .border_color(WARN.opacity(0.6))
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(format!("Turn off SSH password login · {}", flow.server_name)))
                .child(body)
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .children(show_run.then(|| button("pw-login-run", "TURN OFF PASSWORD LOGIN", WARN, can_run, move |cx| a_run.update(cx, |this, cx| this.run_password_login_off(cx)))))
                        .child(button("pw-login-close", if show_run { "CANCEL" } else { "CLOSE" }, TEXT_SECONDARY, !running, move |cx| a_close.update(cx, |this, cx| this.close_password_login(cx)))),
                ),
        )
}
