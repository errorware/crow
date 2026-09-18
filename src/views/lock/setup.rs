use gpui_kit::*;
use crate::app::{CrowApp, Screen};
use crate::components::terminal_text_input_styled;
use crate::theme::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStep {
    WarningNotice,
    ConfigureCredentials,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupFieldFocus {
    Password,
    ConfirmPassword,
    TotpConfirm,
}

#[derive(Clone, Debug)]
pub struct SetupState {
    pub step: SetupStep,
    pub password_input: String,
    pub confirm_input: String,
    pub totp_secret: String,
    pub totp_confirm_input: String,
    pub active_focus: SetupFieldFocus,
    pub show_password: bool,
    pub error_message: Option<String>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            step: SetupStep::WarningNotice,
            password_input: String::new(),
            confirm_input: String::new(),
            totp_secret: String::new(),
            totp_confirm_input: String::new(),
            active_focus: SetupFieldFocus::Password,
            show_password: false,
            error_message: None,
        }
    }
}

pub fn vault_setup_view(app: Entity<CrowApp>, app_data: &CrowApp) -> impl IntoElement {
    let state = &app_data.setup_state;
    div()
        .size_full()
        .children(match state.step {
            SetupStep::WarningNotice => Some(render_warning_step(app.clone())),
            SetupStep::ConfigureCredentials => None,
        })
        .children(match state.step {
            SetupStep::ConfigureCredentials => Some(render_credentials_step(app, app_data)),
            SetupStep::WarningNotice => None,
        })
}

fn render_warning_step(app: Entity<CrowApp>) -> impl IntoElement {
    let app_cancel = app.clone();
    let app_proceed = app.clone();

    div()
        .size_full()
        .bg(BG_APP)
        .flex()
        .items_center()
        .justify_center()
        .p(px(20.0))
        .child(
            div()
                .w(px(560.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(CRIT)
                .shadow_lg()
                .flex()
                .flex_col()
                .overflow_hidden()
                // Header
                .child(
                    div()
                        .h(px(52.0))
                        .px(px(20.0))
                        .bg(CRIT_BG)
                        .border_b_1()
                        .border_color(CRIT)
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
                                        .text_color(CRIT)
                                        .child("⚠"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("SECURITY NOTICE · ENABLING PASSWORD LOGON"),
                                ),
                        )
                        .child(
                            div()
                                .px(px(7.0))
                                .py(px(2.0))
                                .bg(BG_KEY)
                                .border_1()
                                .border_color(CRIT)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(CRIT)
                                .child("MANDATORY 2FA"),
                        ),
                )
                // Body
                .child(
                    div()
                        .p(px(22.0))
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .text_color(TEXT_SECONDARY)
                                .line_height(px(18.0))
                                .child("You are about to activate local encrypted vault protection for Crow. Before proceeding, please review these essential security rules:"),
                        )
                        // Warning 1: Password means password AND 2FA
                        .child(
                            div()
                                .p(px(12.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("1. PASSWORD MEANS PASSWORD AND 2FA"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child("There is no password-only logon. Crow requires two-factor authentication (RFC 6238 TOTP) on every unlock. You will pair an authenticator app (1Password, Google Authenticator, YubiKey) in the next step."),
                                ),
                        )
                        // Warning 2: Zero-Knowledge
                        .child(
                            div()
                                .p(px(12.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("2. ZERO-KNOWLEDGE ENCRYPTION AT REST"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child("Your database (~/.config/crow/crow.db) will be encrypted using Argon2id (64MB memory, 3 iterations) and ChaCha20-Poly1305. The master encryption key is never written to disk and is wiped from RAM upon lock."),
                                ),
                        )
                        // Warning 3: No Cloud Recovery
                        .child(
                            div()
                                .p(px(12.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(CRIT_INK_DIM)
                                        .child("3. NO CLOUD BACKUP OR PASSWORD RESET"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child("This is a local-only, single-user system with zero telemetry. If you lose either your master password or your 2FA authenticator, your local credentials and host configs are permanently lost."),
                                ),
                        )
                        // Buttons
                        .child(
                            div()
                                .mt(px(8.0))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .id("btn-cancel-setup-warning")
                                        .h(px(36.0))
                                        .px(px(16.0))
                                        .bg(BG_KEY)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .flex()
                                        .items_center()
                                        .on_click(move |_ev, _window, cx| {
                                            app_cancel.update(cx, |this, cx| {
                                                this.setup_state = SetupState::default();
                                                this.set_screen(Screen::Settings, cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.5))
                                                .text_color(TEXT_SECONDARY)
                                                .child("CANCEL"),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("btn-proceed-to-credentials")
                                        .h(px(36.0))
                                        .px(px(20.0))
                                        .bg(CRIT)
                                        .hover(|s| s.bg(rgb(0xf87171)))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .on_click(move |_ev, _window, cx| {
                                            app_proceed.update(cx, |this, cx| {
                                                this.setup_state.step = SetupStep::ConfigureCredentials;
                                                if this.setup_state.totp_secret.is_empty() {
                                                    this.setup_state.totp_secret = crate::vault::generate_totp_secret();
                                                }
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(12.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x050507))
                                                .child("I UNDERSTAND · SETUP PASSWORD & 2FA →"),
                                        ),
                                ),
                        ),
                ),
        )
}

fn render_credentials_step(app: Entity<CrowApp>, app_data: &CrowApp) -> impl IntoElement {
    let state = &app_data.setup_state;
    let has_error = state.error_message.is_some();
    let error_text = state.error_message.clone().unwrap_or_default();

    let is_pwd_focused = state.active_focus == SetupFieldFocus::Password;
    let is_confirm_focused = state.active_focus == SetupFieldFocus::ConfirmPassword;
    let is_totp_focused = state.active_focus == SetupFieldFocus::TotpConfirm;
    let show_pwd = state.show_password;
    let totp_secret = state.totp_secret.clone();

    let app_pwd_focus = app.clone();
    let app_confirm_focus = app.clone();
    let app_totp_focus = app.clone();
    let app_toggle_show = app.clone();
    let app_back = app.clone();
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
                .w(px(540.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .shadow_lg()
                .flex()
                .flex_col()
                .overflow_hidden()
                // Header
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
                                        .child("CONFIGURE PASSWORD & MANDATORY 2FA"),
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
                                .child("STEP 2 OF 2"),
                        ),
                )
                // Body
                .child(
                    div()
                        .p(px(22.0))
                        .flex()
                        .flex_col()
                        .gap(px(15.0))
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
                                    terminal_text_input_styled(
                                        "setup-input-pwd",
                                        &state.password_input,
                                        "Enter master password (min 8 characters)…",
                                        is_pwd_focused,
                                        !show_pwd,
                                        36.0,
                                        13.0,
                                        if is_pwd_focused { app_data.input_cursor } else { 0 },
                                        if is_pwd_focused { app_data.input_selection } else { None },
                                        if is_pwd_focused { app_data.input_drag_anchor } else { None },
                                        app_data.cursor_blink,
                                        {
                                            let app = app_pwd_focus;
                                            move |cursor, anchor, selection, _window, cx| {
                                                app.update(cx, |this, cx| {
                                                    this.setup_state.active_focus = SetupFieldFocus::Password;
                                                    this.input_cursor = cursor;
                                                    this.input_drag_anchor = anchor;
                                                    this.input_selection = selection;
                                                    this.cursor_blink = true;
                                                    cx.notify();
                                                });
                                            }
                                        },
                                    ),
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
                                    terminal_text_input_styled(
                                        "setup-input-confirm",
                                        &state.confirm_input,
                                        "Re-type password…",
                                        is_confirm_focused,
                                        !show_pwd,
                                        36.0,
                                        13.0,
                                        if is_confirm_focused { app_data.input_cursor } else { 0 },
                                        if is_confirm_focused { app_data.input_selection } else { None },
                                        if is_confirm_focused { app_data.input_drag_anchor } else { None },
                                        app_data.cursor_blink,
                                        {
                                            let app = app_confirm_focus;
                                            move |cursor, anchor, selection, _window, cx| {
                                                app.update(cx, |this, cx| {
                                                    this.setup_state.active_focus = SetupFieldFocus::ConfirmPassword;
                                                    this.input_cursor = cursor;
                                                    this.input_drag_anchor = anchor;
                                                    this.input_selection = selection;
                                                    this.cursor_blink = true;
                                                    cx.notify();
                                                });
                                            }
                                        },
                                    ),
                                ),
                        )
                        // Mandatory 2FA TOTP Card
                        .child(
                            div()
                                .p(px(14.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_STRONG)
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(OK)
                                                        .child("MANDATORY TWO-FACTOR AUTHENTICATION (TOTP)"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .bg(OK_BG)
                                                .border_1()
                                                .border_color(OK)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(OK)
                                                .child("REQUIRED"),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_MUTED)
                                        .child("Add this secret to your Authenticator app (1Password, Google Authenticator, etc.):"),
                                )
                                .child(
                                    div()
                                        .p(px(10.0))
                                        .bg(BG_WINDOW)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(OK)
                                        .text_align(TextAlign::Center)
                                        .child(totp_secret),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_MUTED)
                                        .child("ENTER 6-DIGIT CODE TO CONFIRM PAIRING:"),
                                )
                                .child(
                                    terminal_text_input_styled(
                                        "setup-input-totp",
                                        &state.totp_confirm_input,
                                        "6-digit code (e.g. 123456)",
                                        is_totp_focused,
                                        false,
                                        36.0,
                                        13.0,
                                        if is_totp_focused { app_data.input_cursor } else { 0 },
                                        if is_totp_focused { app_data.input_selection } else { None },
                                        if is_totp_focused { app_data.input_drag_anchor } else { None },
                                        app_data.cursor_blink,
                                        {
                                            let app = app_totp_focus;
                                            move |cursor, anchor, selection, _window, cx| {
                                                app.update(cx, |this, cx| {
                                                    this.setup_state.active_focus = SetupFieldFocus::TotpConfirm;
                                                    this.input_cursor = cursor;
                                                    this.input_drag_anchor = anchor;
                                                    this.input_selection = selection;
                                                    this.cursor_blink = true;
                                                    cx.notify();
                                                });
                                            }
                                        },
                                    ),
                                ),
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
                                        .id("btn-back-to-warning")
                                        .h(px(36.0))
                                        .px(px(14.0))
                                        .bg(BG_KEY)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .flex()
                                        .items_center()
                                        .on_click(move |_ev, _window, cx| {
                                            app_back.update(cx, |this, cx| {
                                                this.setup_state.step = SetupStep::WarningNotice;
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.0))
                                                .text_color(TEXT_SECONDARY)
                                                .child("← BACK"),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("btn-initialize-vault")
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
                                                this.submit_setup(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(12.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x050507))
                                                .child("ACTIVATE PASSWORD & 2FA"),
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
