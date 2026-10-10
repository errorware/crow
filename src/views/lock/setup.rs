use gpui_kit::*;
use crate::app::{CrowApp, Screen};
use gpui_kit::component::input::{Input, InputState};
use crate::theme::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStep {
    WarningNotice,
    ConfigureCredentials,
}

/// The setup wizard's state. The password fields are gpui inputs
/// (`CrowApp::setup_inputs`); this holds the TOTP seed: redacted in Debug,
/// wiped on drop.
#[derive(Clone)]
pub struct SetupState {
    pub step: SetupStep,
    pub totp_secret: String,
    /// The otpauth QR code, made once per secret.
    pub totp_qr: Option<std::sync::Arc<gpui_kit::Image>>,
    /// The secret was just copied (the COPY button says so).
    pub secret_copied: bool,
    pub show_password: bool,
    pub error_message: Option<String>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            step: SetupStep::WarningNotice,
            totp_secret: String::new(),
            totp_qr: None,
            secret_copied: false,
            show_password: false,
            error_message: None,
        }
    }
}

impl std::fmt::Debug for SetupState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(t!("lock-setup-setupstate")).field("step", &self.step).field("secrets", &"[secret]").field("error_message", &self.error_message).finish()
    }
}

impl Drop for SetupState {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.totp_secret.zeroize();
    }
}

pub fn vault_setup_view(app: Entity<CrowApp>, setup_state: &SetupState, inputs: Option<&crate::app::SetupInputs>) -> impl IntoElement {
    let state = &setup_state;
    div()
        .size_full()
        .children(match state.step {
            SetupStep::WarningNotice => Some(render_warning_step(app.clone())),
            SetupStep::ConfigureCredentials => None,
        })
        .children(match state.step {
            SetupStep::ConfigureCredentials => Some(render_credentials_step(app, setup_state, inputs)),
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
                                        .child(t!("lock-setup-security-notice-enabling-password-logon")),
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
                                .child(t!("lock-setup-mandatory-2fa")),
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
                                .child(t!("lock-setup-you-are-about-to-activate")),
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
                                        .child(t!("lock-setup-1-password-means-password-and")),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child(t!("lock-setup-there-is-no-password-only")),
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
                                        .child(t!("lock-setup-2-what-the-password-protects")),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child(t!("lock-setup-crow-encrypts-its-secrets-provider")),
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
                                        .child(t!("lock-setup-3-no-cloud-backup-or")),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child(t!("lock-setup-this-is-a-local-only")),
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
                                                .child(t!("lock-setup-cancel")),
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
                                            app_proceed.update(cx, |this, cx| this.enter_setup_credentials(cx));
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(12.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x050507))
                                                .child(t!("lock-setup-i-understand-setup-password-2fa")),
                                        ),
                                ),
                        ),
                ),
        )
}

fn render_credentials_step(app: Entity<CrowApp>, setup_state: &SetupState, inputs: Option<&crate::app::SetupInputs>) -> impl IntoElement {
    let state = &setup_state;
    let has_error = state.error_message.is_some();
    let error_text = state.error_message.clone().unwrap_or_default();

    let show_pwd = state.show_password;
    // Groups of four, easier to read and type: "ABCD EFGH …".
    let totp_secret = state.totp_secret.chars().collect::<Vec<_>>().chunks(4).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join(" ");
    let input = |i: Option<&Entity<InputState>>| i.map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(13.0)).bg(BG_WINDOW).rounded(px(2.0)));

    let app_toggle_show = app.clone();
    let app_copy = app.clone();
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
                                        .child(t!("lock-setup-configure-password-mandatory-2fa")),
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
                                .child(t!("lock-setup-step-2-of-2")),
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
                                                .text_color(TEXT_MUTED)
                                                .child(t!("lock-setup-master-password")),
                                        )
                                        .child(
                                            div()
                                                .id("btn-toggle-setup-show")
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_DIMMER)
                                                .cursor_pointer()
                                                .hover(|s| s.text_color(TEXT_PRIMARY))
                                                .on_click(move |_ev, window, cx| {
                                                    app_toggle_show.update(cx, |this, cx| this.toggle_setup_show_password(window, cx));
                                                })
                                                .child(if show_pwd { t!("lock-setup-hide-password") } else { t!("lock-setup-show-password") }),
                                        ),
                                )
                                .children(input(inputs.map(|i| &i.password))),
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
                                        .text_color(TEXT_MUTED)
                                        .child(t!("lock-setup-confirm-master-password")),
                                )
                                .children(input(inputs.map(|i| &i.confirm))),
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
                                                        .child(t!("lock-setup-mandatory-two-factor-authentication-totp")),
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
                                                .child(t!("lock-setup-required")),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(14.0))
                                        // Scan it...
                                        .children(state.totp_qr.clone().map(|qr| {
                                            div().flex_none().p(px(6.0)).bg(rgb(0xffffff)).child(img(qr).size(px(148.0)))
                                        }))
                                        // ...or type / paste it.
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w(px(0.0))
                                                .flex()
                                                .flex_col()
                                                .gap(px(8.0))
                                                .child(div().font_family(FONT_MONO).text_size(px(10.5)).line_height(px(15.0)).text_color(TEXT_MUTED).child(t!("lock-setup-scan-the-code-with-your")))
                                                .child(
                                                    div()
                                                        .p(px(8.0))
                                                        .bg(BG_WINDOW)
                                                        .border_1()
                                                        .border_color(BORDER_DEFAULT)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(12.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(OK)
                                                        .child(totp_secret),
                                                )
                                                .child(
                                                    div()
                                                        .id("btn-copy-totp-secret")
                                                        .flex_none()
                                                        .px(px(9.0))
                                                        .py(px(4.0))
                                                        .border_1()
                                                        .border_color(if state.secret_copied { OK } else { BORDER_DEFAULT })
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(if state.secret_copied { OK } else { TEXT_SECONDARY })
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                                        .on_click(move |_ev, _window, cx| app_copy.update(cx, |this, cx| this.copy_setup_secret(cx)))
                                                        .child(if state.secret_copied { t!("lock-setup-copied") } else { t!("lock-setup-copy-key") }),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(TEXT_MUTED)
                                        .child(t!("lock-setup-enter-6-digit-code-to")),
                                )
                                .children(input(inputs.map(|i| &i.code))),
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
                                                .child(t!("lock-setup-back")),
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
                                                .child(t!("lock-setup-activate-password-2fa")),
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
