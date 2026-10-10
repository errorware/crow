use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputState, OtpInput, OtpState, Textarea, TextareaState};
use crate::app::CrowApp;
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::theme::*;

/// Render the UI Components Lab sandbox view within Settings.
/// Provides an isolated testing ground for `gpui-component` widgets:
/// - Single-line `Input`
/// - Password `Input` with mask toggle
/// - Cleanable `Input` with quick-clear 'X'
/// - Prefixed / Suffixed `Input`
/// - 6-Digit grouped `OtpInput`
/// - Multi-line `Textarea`
/// - Native `Button` styles and variants
pub struct LabState {
    pub text_input: Entity<InputState>,
    pub cleanable_input: Entity<InputState>,
    pub password_input: Entity<InputState>,
    pub prefix_input: Entity<InputState>,
    pub textarea: Entity<TextareaState>,
    pub otp_input: Entity<OtpState>,
}

impl LabState {
    pub fn new(window: &mut Window, cx: &mut Context<CrowApp>) -> Self {
        Self {
            text_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("settings-lab-type-something-here-to-test"))
                    .default_value("prod-db-cluster.internal")
            }),
            cleanable_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("settings-lab-type-to-reveal-clear-button"))
                    .default_value(t!("settings-lab-search-fleet-by-tag-or"))
            }),
            password_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("settings-lab-enter-sensitive-secret"))
                    .masked(true)
                    .default_value("crow_vault_master_key_9981")
            }),
            prefix_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("10.0.0.1")
                    .default_value("bastion.eu-west-1.aws")
            }),
            textarea: cx.new(|cx| {
                TextareaState::new(window, cx).default_value(
                    t!("settings-lab-fleet-deployment-manifest-env-production"),
                )
            }),
            otp_input: cx.new(|cx| {
                OtpState::new(6, window, cx).default_value("849201")
            }),
        }
    }
}

pub fn render_components_lab(_app: Entity<CrowApp>, lab_state: &LabState) -> Div {
    let lab = &lab_state;

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_APP)
        // Top Header
        .child(
            div()
                .h(px(38.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tabler_icon(TablerIcon::Box).size(px(14.0)).text_color(OK_INK))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("settings-lab-ui-components-lab")),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .child(t!("settings-lab-gpui-component-transition-testbed")),
                        ),
                )
                .child(
                    div()
                        .px(px(7.0))
                        .py(px(2.0))
                        .bg(OK_BG)
                        .border_1()
                        .border_color(OK)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(OK_INK)
                        .child(t!("settings-lab-isolated-sandbox")),
                ),
        )
        // Scrollable Testbed Content
        .child(
            div()
                .id("lab-scroll-testbed")
                .flex_1()
                .overflow_y_scrollbar()
                .p(px(20.0))
                .flex()
                .flex_col()
                .gap(px(24.0))
                // Info Banner
                .child(
                    div()
                        .p(px(14.0))
                        .bg(hex_rgb(0x0c0c10))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded(px(4.0))
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(tabler_icon(TablerIcon::InfoCircle).size(px(13.0)).text_color(OK_INK))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(t!("settings-lab-evaluation-transition-sandbox")),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .line_height(relative(1.4))
                                .child(
                                    t!("settings-lab-this-page-runs-gpui-component"),
                                ),
                        ),
                )
                // Section 1: Direct Side-by-Side Comparison
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("settings-lab-1-primary-native-gpui-input")),
                        )
                        .child(
                            div()
                                .p(px(14.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(OK)
                                .rounded(px(4.0))
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
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(OK_INK)
                                                .child(t!("settings-lab-native-gpui-component-input")),
                                        )
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(1.0))
                                                .bg(OK_BG)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.0))
                                                .text_color(OK_INK)
                                                .child(t!("settings-lab-rope-buffer-shaping")),
                                        ),
                                )
                                .child(
                                    Input::new(&lab.text_input)
                                        .id("lab-input-native")
                                        .cleanable(true)
                                        .w_full()
                                        .font_family(FONT_MONO)
                                        .bg(BG_APP)
                                        .border_color(BORDER_DEFAULT)
                                        .rounded(px(2.0)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_DIMMER)
                                        .line_height(relative(1.4))
                                        .child(t!("settings-lab-native-mouse-drag-selection-cmd")),
                                ),
                        ),
                )
                // Section 2: Input Variations
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("settings-lab-2-input-variants-capabilities")),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(12.0))
                                // Password with Mask Toggle
                                .child(
                                    div()
                                        .p(px(12.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .rounded(px(4.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_DIM)
                                                .child(t!("settings-lab-password-input-with-native-mask")),
                                        )
                                        .child(
                                            Input::new(&lab.password_input)
                                                .id("lab-input-password")
                                                .mask_toggle()
                                                .cleanable(true)
                                                .w_full()
                                                .font_family(FONT_MONO)
                                                .bg(BG_APP)
                                                .border_color(BORDER_DEFAULT)
                                                .rounded(px(2.0)),
                                        ),
                                )
                                // Cleanable Search Input
                                .child(
                                    div()
                                        .p(px(12.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .rounded(px(4.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_DIM)
                                                .child(t!("settings-lab-search-filter-input-cleanable")),
                                        )
                                        .child(
                                            Input::new(&lab.cleanable_input)
                                                .id("lab-input-cleanable")
                                                .cleanable(true)
                                                .prefix(tabler_icon(TablerIcon::Search).size(px(12.0)).text_color(TEXT_FAINT))
                                                .w_full()
                                                .font_family(FONT_MONO)
                                                .bg(BG_APP)
                                                .border_color(BORDER_DEFAULT)
                                                .rounded(px(2.0)),
                                        ),
                                )
                                // Prefixed & Suffixed Address Input
                                .child(
                                    div()
                                        .p(px(12.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .rounded(px(4.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_DIM)
                                                .child(t!("settings-lab-prefix-suffix-adornments")),
                                        )
                                        .child(
                                            Input::new(&lab.prefix_input)
                                                .id("lab-input-prefix")
                                                .prefix(tabler_icon(TablerIcon::Network).size(px(12.0)).text_color(OK_INK))
                                                .suffix(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.0))
                                                        .bg(BG_KEY)
                                                        .border_1()
                                                        .border_color(BORDER_DEFAULT)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .text_color(TEXT_MUTED)
                                                        .child(t!("settings-lab-port-22")),
                                                )
                                                .w_full()
                                                .font_family(FONT_MONO)
                                                .bg(BG_APP)
                                                .border_color(BORDER_DEFAULT)
                                                .rounded(px(2.0)),
                                        ),
                                ),
                        ),
                )
                // Section 3: One-Time Password (OTP) 2FA Input
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("settings-lab-3-6-digit-otp-authenticator")),
                        )
                        .child(
                            div()
                                .p(px(14.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded(px(4.0))
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIM)
                                        .child(t!("settings-lab-grouped-otp-cells-split-3")),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .child(OtpInput::new(&lab.otp_input).groups(2)),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_DIMMER)
                                        .child(t!("settings-lab-direct-replacement-candidate-for-vault")),
                                ),
                        ),
                )
                // Section 4: Multi-line Textarea
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("settings-lab-4-multi-line-textarea")),
                        )
                        .child(
                            div()
                                .p(px(14.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded(px(4.0))
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIM)
                                        .child(t!("settings-lab-multi-line-editor-with-word")),
                                )
                                .child(
                                    Textarea::new(&lab.textarea)
                                        .h(px(90.0))
                                        .w_full()
                                        .font_family(FONT_MONO)
                                        .bg(BG_APP)
                                        .border_color(BORDER_DEFAULT)
                                        .rounded(px(2.0)),
                                ),
                        ),
                )
                // Section 5: Buttons
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("settings-lab-5-gpui-component-button-matrix")),
                        )
                        .child(
                            div()
                                .p(px(14.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded(px(4.0))
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(Button::new("btn-primary").label(t!("settings-lab-primary-action")).primary())
                                .child(Button::new("btn-secondary").label(t!("settings-lab-secondary")).secondary())
                                .child(Button::new("btn-ghost").label(t!("settings-lab-ghost")).ghost())
                                .child(Button::new("btn-danger").label(t!("settings-lab-destructive")).danger()),
                        ),
                ),
        )
}
