use gpui_kit::*;
use crate::app::CrowApp;
use crate::theme::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupFieldFocus {
    Password,
    ConfirmPassword,
    TotpConfirm,
}

#[derive(Clone, Debug)]
pub struct SetupState {
    pub password_input: String,
    pub confirm_input: String,
    pub enable_totp: bool,
    pub totp_secret: String,
    pub totp_confirm_input: String,
    pub active_focus: SetupFieldFocus,
    pub show_password: bool,
    pub error_message: Option<String>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            password_input: String::new(),
            confirm_input: String::new(),
            enable_totp: false,
            totp_secret: String::new(),
            totp_confirm_input: String::new(),
            active_focus: SetupFieldFocus::Password,
            show_password: false,
            error_message: None,
        }
    }
}

pub fn vault_setup_view(app: Entity<CrowApp>, state: &SetupState) -> impl IntoElement {
    let has_error = state.error_message.is_some();
    let error_text = state.error_message.clone().unwrap_or_default();

    let is_pwd_focused = state.active_focus == SetupFieldFocus::Password;
    let is_confirm_focused = state.active_focus == SetupFieldFocus::ConfirmPassword;
    let is_totp_focused = state.active_focus == SetupFieldFocus::TotpConfirm;

    let pwd_display = if state.show_password {
        state.password_input.clone()
    } else {
        "●".repeat(state.password_input.len())
    };

    let confirm_display = if state.show_password {
        state.confirm_input.clone()
    } else {
        "●".repeat(state.confirm_input.len())
    };

    let show_pwd = state.show_password;
    let enable_totp = state.enable_totp;
    let totp_secret = state.totp_secret.clone();
    let totp_display = state.totp_confirm_input.clone();

    let app_pwd_focus = app.clone();
    let app_confirm_focus = app.clone();
    let app_totp_focus = app.clone();
    let app_toggle_show = app.clone();
    let app_toggle_totp = app.clone();
    let app_submit = app.clone();

    div()
        .size_full()
        .bg(BG_APP)
        .flex()
        .items_center()
        .justify_center()
        .p(px(20.0))
        .child(
            div()
                .w(px(520.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .shadow_lg()
                .flex()
                .flex_col()
                .overflow_hidden()
                // Top accent header
                .child(
                    div()
                        .h(px(50.0))
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
                                        .child("INITIALIZE MASTER VAULT"),
                                ),
                        )
                        .child(
                            div()
                                .px(px(7.0))
                                .py(px(2.0))
                                .bg(BG_KEY)
                                .border_1()
                                .border_color(BORDER_KEY)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(OK)
                                .child("FIRST RUN · LOCAL ONLY"),
                        ),
                )
                // Body
                .child(
                    div()
                        .p(px(22.0))
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        // Explanatory note
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_MUTED)
                                .line_height(px(16.0))
                                .child("Crow uses a zero-knowledge local SQLite database (~/.config/crow/crow.db). Your master password derives an encryption key via Argon2id and ChaCha20-Poly1305 to secure all SSH credentials, agent keys, and fleet configs."),
                        )
                        // Error banner if any
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
                        // Field 1: Master Password
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
                                                .id("btn-toggle-setup-show")
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_DIMMER)
                                                .cursor_pointer()
                                                .hover(|s| s.text_color(TEXT_PRIMARY))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_toggle_show.update(cx, |this, cx| {
                                                        this.setup_state.show_password = !this.setup_state.show_password;
                                                        cx.notify();
                                                    });
                                                })
                                                .child(if show_pwd { "HIDE PASSWORD" } else { "SHOW PASSWORD" }),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("setup-input-pwd")
                                        .h(px(36.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(if is_pwd_focused { OK } else { BORDER_DEFAULT })
                                        .px(px(12.0))
                                        .flex()
                                        .items_center()
                                        .cursor_text()
                                        .on_click(move |_ev, _window, cx| {
                                            app_pwd_focus.update(cx, |this, cx| {
                                                this.setup_state.active_focus = SetupFieldFocus::Password;
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(13.0))
                                                .text_color(if pwd_display.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                                .child(if pwd_display.is_empty() { "Enter strong master password…".to_string() } else { pwd_display }),
                                        )
                                        .children(if is_pwd_focused {
                                            Some(div().w(px(2.0)).h(px(16.0)).bg(OK).ml(px(2.0)))
                                        } else {
                                            None
                                        }),
                                ),
                        )
                        // Field 2: Confirm Password
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(if is_confirm_focused { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .child("CONFIRM MASTER PASSWORD"),
                                )
                                .child(
                                    div()
                                        .id("setup-input-confirm")
                                        .h(px(36.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(if is_confirm_focused { OK } else { BORDER_DEFAULT })
                                        .px(px(12.0))
                                        .flex()
                                        .items_center()
                                        .cursor_text()
                                        .on_click(move |_ev, _window, cx| {
                                            app_confirm_focus.update(cx, |this, cx| {
                                                this.setup_state.active_focus = SetupFieldFocus::ConfirmPassword;
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(13.0))
                                                .text_color(if confirm_display.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                                .child(if confirm_display.is_empty() { "Re-type password…".to_string() } else { confirm_display }),
                                        )
                                        .children(if is_confirm_focused {
                                            Some(div().w(px(2.0)).h(px(16.0)).bg(OK).ml(px(2.0)))
                                        } else {
                                            None
                                        }),
                                ),
                        )
                        // 2FA TOTP Toggle Box
                        .child(
                            div()
                                .p(px(12.0))
                                .bg(if enable_totp { BG_PANEL } else { hex_rgba(0, 0.0) })
                                .border_1()
                                .border_color(if enable_totp { BORDER_STRONG } else { BORDER_PANEL })
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .id("btn-toggle-totp-option")
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .cursor_pointer()
                                        .on_click(move |_ev, _window, cx| {
                                            app_toggle_totp.update(cx, |this, cx| {
                                                this.setup_state.enable_totp = !this.setup_state.enable_totp;
                                                if this.setup_state.enable_totp && this.setup_state.totp_secret.is_empty() {
                                                    this.setup_state.totp_secret = crate::vault::generate_totp_secret();
                                                }
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(10.0))
                                                .child(
                                                    div()
                                                        .size(px(16.0))
                                                        .border_1()
                                                        .border_color(if enable_totp { OK } else { BORDER_DEFAULT })
                                                        .bg(if enable_totp { OK_BG } else { BG_PANEL })
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .text_color(OK)
                                                        .child(if enable_totp { "✓" } else { "" }),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .font_family(FONT_MONO)
                                                                .text_size(px(12.0))
                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                .text_color(TEXT_PRIMARY)
                                                                .child("Enable Two-Factor Authentication (RFC 6238 TOTP)"),
                                                        )
                                                        .child(
                                                            div()
                                                                .font_family(FONT_MONO)
                                                                .text_size(px(10.0))
                                                                .text_color(TEXT_DIMMER)
                                                                .child("Requires 6-digit authenticator code (Google Auth, 1Password, Yubikey) on every unlock"),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .bg(BG_KEY)
                                                .border_1()
                                                .border_color(BORDER_KEY)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(if enable_totp { OK } else { TEXT_DIMMER })
                                                .child(if enable_totp { "ENABLED" } else { "OPTIONAL" }),
                                        ),
                                )
                                // If enabled, show secret string and confirmation field
                                .children(if enable_totp {
                                    Some(
                                        div()
                                            .mt(px(4.0))
                                            .pt(px(10.0))
                                            .border_t_1()
                                            .border_color(BORDER_PANEL)
                                            .flex()
                                            .flex_col()
                                            .gap(px(10.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .text_color(TEXT_MUTED)
                                                    .child("SCAN QR OR PASTE SECRET INTO YOUR AUTHENTICATOR:"),
                                            )
                                            .child(
                                                div()
                                                    .p(px(8.0))
                                                    .bg(BG_WINDOW)
                                                    .border_1()
                                                    .border_color(BORDER_DEFAULT)
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(12.5))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(OK)
                                                    .text_align(TextAlign::Center)
                                                    .child(totp_secret),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .text_color(TEXT_MUTED)
                                                    .child("VERIFY 6-DIGIT CODE TO CONFIRM SETUP:"),
                                            )
                                            .child(
                                                div()
                                                    .id("setup-input-totp")
                                                    .h(px(36.0))
                                                    .bg(BG_PANEL)
                                                    .border_1()
                                                    .border_color(if is_totp_focused { OK } else { BORDER_DEFAULT })
                                                    .px(px(12.0))
                                                    .flex()
                                                    .items_center()
                                                    .cursor_text()
                                                    .on_click(move |_ev, _window, cx| {
                                                        app_totp_focus.update(cx, |this, cx| {
                                                            this.setup_state.active_focus = SetupFieldFocus::TotpConfirm;
                                                            cx.notify();
                                                        });
                                                    })
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(13.0))
                                                            .text_color(if totp_display.is_empty() { TEXT_FAINTER } else { TEXT_PRIMARY })
                                                            .child(if totp_display.is_empty() { "6-digit code (e.g. 123456)".to_string() } else { totp_display }),
                                                    )
                                                    .children(if is_totp_focused {
                                                        Some(div().w(px(2.0)).h(px(16.0)).bg(OK).ml(px(2.0)))
                                                    } else {
                                                        None
                                                    }),
                                            ),
                                    )
                                } else {
                                    None
                                }),
                        )
                        // Action buttons
                        .child(
                            div()
                                .mt(px(6.0))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_FAINT)
                                        .child("Tab to cycle fields · ↵ Enter to submit"),
                                )
                                .child(
                                    div()
                                        .id("btn-initialize-vault")
                                        .h(px(36.0))
                                        .px(px(18.0))
                                        .bg(OK)
                                        .hover(|s| s.bg(rgb(0x34d399)))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .on_click(move |_ev, _window, cx| {
                                            app_submit.update(cx, |this, cx| {
                                                this.submit_setup(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(12.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x050507))
                                                .child("CREATE ENCRYPTED VAULT"),
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
