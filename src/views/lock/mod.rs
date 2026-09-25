pub mod setup;

use gpui_kit::*;
use crate::app::CrowApp;
use gpui_kit::component::input::Input;
use crate::theme::*;

pub use setup::{vault_setup_view, SetupState, SetupStep};

/// The lock screen's state. The password and code are gpui inputs
/// (`CrowApp::lock_inputs`), dropped once Crow unlocks.
#[derive(Clone, Debug, Default)]
pub struct LockState {
    pub show_password: bool,
    pub error_message: Option<String>,
}

pub fn vault_lock_view(app: Entity<CrowApp>, lock_state: &LockState, inputs: Option<&crate::app::LockInputs>) -> impl IntoElement {
    let state = &lock_state;
    let has_error = state.error_message.is_some();
    let error_text = state.error_message.clone().unwrap_or_default();

    let show_pwd = state.show_password;
    let input = |i: Option<&Entity<gpui_kit::component::input::InputState>>| i.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(13.0)).bg(BG_WINDOW).rounded(px(2.0)));

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
                                                .text_color(TEXT_MUTED)
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
                                                .on_click(move |_ev, window, cx| {
                                                    app_toggle_show.update(cx, |this, cx| this.toggle_lock_show_password(window, cx));
                                                })
                                                .child(if show_pwd { "HIDE" } else { "SHOW" }),
                                        ),
                                )
                                .children(input(inputs.map(|i| &i.password))),
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
                                                .text_color(TEXT_MUTED)
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
                                .children(input(inputs.map(|i| &i.code))),
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
