pub mod setup;

use gpui_kit::*;
use crate::app::CrowApp;
use crate::theme::*;

pub use setup::{vault_setup_view, SetupFieldFocus, SetupState, SetupStep};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockFieldFocus {
    Password,
    Totp,
}

#[derive(Clone, Debug)]
pub struct LockState {
    pub password_input: String,
    pub totp_input: String,
    pub active_focus: LockFieldFocus,
    pub show_password: bool,
    pub error_message: Option<String>,
}

impl Default for LockState {
    fn default() -> Self {
        Self {
            password_input: String::new(),
            totp_input: String::new(),
            active_focus: LockFieldFocus::Password,
            show_password: false,
            error_message: None,
        }
    }
}

pub fn vault_lock_view(app: Entity<CrowApp>, state: &LockState) -> impl IntoElement {
    let has_error = state.error_message.is_some();
    let error_text = state.error_message.clone().unwrap_or_default();

    let is_pwd_focused = state.active_focus == LockFieldFocus::Password;
    let is_totp_focused = state.active_focus == LockFieldFocus::Totp;

    let pwd_display = if state.show_password {
        state.password_input.clone()
    } else {
        "●".repeat(state.password_input.len())
    };

    let show_pwd = state.show_password;
    let totp_display = state.totp_input.clone();

    let app_pwd_focus = app.clone();
    let app_totp_focus = app.clone();
    let app_toggle_show = app.clone();
    let app_submit = app.clone();

    div()
        .size_full()
        .bg(BG_APP)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .p(px(20.0))
        .child(
            div()
                .w(px(460.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .shadow_lg()
                .flex()
                .flex_col()
                .overflow_hidden()
                // Top header
                .child(
                    div()
                        .h(px(52.0))
                        .px(px(20.0))
                        .bg(BG_PANEL)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(18.0))
                                        .text_color(OK)
                                        .child("⬢"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("CROW VAULT LOCKED"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().size(px(6.0)).rounded_full().bg(OK))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_MUTED)
                                        .child("PASSWORD + 2FA"),
                                ),
                        ),
                )
                // Body
                .child(
                    div()
                        .p(px(22.0))
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        // Vault metadata hint
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(10.0))
                                .py(px(6.0))
                                .bg(BG_WINDOW)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIMMER)
                                .child("STORE: ~/.config/crow/crow.db")
                                .child("2FA: MANDATORY"),
                        )
                        // Error message
                        .children(if has_error {
                            Some(
                                div()
                                    .px(px(12.0))
                                    .py(px(8.0))
                                    .bg(CRIT_BG)
                                    .border_1()
                                    .border_color(CRIT)
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .font_family(FONT_MONO)
                                    .text_size(px(11.5))
                                    .text_color(CRIT_INK_DIM)
                                    .child("⚠")
                                    .child(error_text),
                            )
                        } else {
                            None
                        })
                        // Master password input
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if is_pwd_focused { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .child("MASTER PASSWORD"),
                                        )
                                        .child(
                                            div()
                                                .id("btn-toggle-lock-pwd")
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_DIMMER)
                                                .cursor_pointer()
                                                .hover(|s| s.text_color(TEXT_PRIMARY))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_toggle_show.update(cx, |this, cx| {
                                                        this.lock_state.show_password = !this.lock_state.show_password;
                                                        cx.notify();
                                                    });
                                                })
                                                .child(if show_pwd { "HIDE" } else { "SHOW" }),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("lock-input-pwd")
                                        .h(px(38.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(if is_pwd_focused { OK } else { BORDER_DEFAULT })
                                        .px(px(12.0))
                                        .flex()
                                        .items_center()
                                        .cursor_text()
                                        .on_click(move |_ev, _window, cx| {
                                            app_pwd_focus.update(cx, |this, cx| {
                                                this.lock_state.active_focus = LockFieldFocus::Password;
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(13.0))
                                                .text_color(if pwd_display.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                                .child(if pwd_display.is_empty() { "Enter master password…".to_string() } else { pwd_display }),
                                        )
                                        .children(if is_pwd_focused {
                                            Some(div().w(px(2.0)).h(px(16.0)).bg(OK).ml(px(2.0)))
                                        } else {
                                            None
                                        }),
                                ),
                        )
                        // TOTP Code input (always mandatory)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if is_totp_focused { TEXT_PRIMARY } else { TEXT_MUTED })
                                                .child("2FA TOTP CODE"),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_DIMMER)
                                                .child("6 DIGITS"),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("lock-input-totp")
                                        .h(px(38.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(if is_totp_focused { OK } else { BORDER_DEFAULT })
                                        .px(px(12.0))
                                        .flex()
                                        .items_center()
                                        .cursor_text()
                                        .on_click(move |_ev, _window, cx| {
                                            app_totp_focus.update(cx, |this, cx| {
                                                this.lock_state.active_focus = LockFieldFocus::Totp;
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(13.0))
                                                .text_color(if totp_display.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                                .child(if totp_display.is_empty() { "000000".to_string() } else { totp_display }),
                                        )
                                        .children(if is_totp_focused {
                                            Some(div().w(px(2.0)).h(px(16.0)).bg(OK).ml(px(2.0)))
                                        } else {
                                            None
                                        }),
                                ),
                        )
                        // Bottom submit row
                        .child(
                            div()
                                .mt(px(4.0))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_FAINT)
                                        .child("Tab to switch · ↵ to unlock"),
                                )
                                .child(
                                    div()
                                        .id("btn-unlock-vault")
                                        .h(px(36.0))
                                        .px(px(20.0))
                                        .bg(OK)
                                        .hover(|s| s.bg(rgb(0x34d399)))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.submit_unlock(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(12.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x050507))
                                                .child("UNLOCK VAULT"),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.0))
                                                .text_color(rgb(0x050507))
                                                .child("↵"),
                                        ),
                                ),
                        ),
                ),
        )
}
