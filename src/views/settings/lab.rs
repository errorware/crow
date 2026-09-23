use gpui_kit::*;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputState, OtpInput, OtpState, Textarea, TextareaState};
use crate::app::CrowApp;
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::components::terminal_text_input;
use crate::theme::*;

/// Render the UI Components Lab sandbox view within Settings.
/// Provides an isolated testing ground for `gpui-component` widgets:
/// - Single-line `Input`
/// - Side-by-side comparison with hand-rolled `terminal_text_input`
/// - Password `Input` with mask toggle
/// - Cleanable `Input` with quick-clear 'X'
/// - Prefixed / Suffixed `Input`
/// - 6-Digit grouped `OtpInput`
/// - Multi-line `Textarea`
/// - Native `Button` styles and variants
/// UI components sandbox (Settings → Components): live gpui-component
/// widgets beside Crow's own text input for side-by-side comparison.
pub struct LabState {
    pub text_input: Entity<InputState>,
    pub cleanable_input: Entity<InputState>,
    pub password_input: Entity<InputState>,
    pub prefix_input: Entity<InputState>,
    pub textarea: Entity<TextareaState>,
    pub otp_input: Entity<OtpState>,
    pub custom_compare_text: String,
    pub custom_compare_cursor: usize,
    pub custom_compare_selection: Option<(usize, usize)>,
    pub custom_compare_drag_anchor: Option<usize>,
}

impl LabState {
    pub fn new(window: &mut Window, cx: &mut Context<CrowApp>) -> Self {
        Self {
        text_input: cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type something here to test gpui-component...")
                .default_value("prod-db-cluster.internal")
        }),
        cleanable_input: cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type to reveal clear button...")
                .default_value("search fleet by tag or region...")
        }),
        password_input: cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter sensitive secret...")
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
                "# Fleet Deployment Manifest\nenv: production\nreplicas: 4\nregion: us-east-1\nauto_drain: true",
            )
        }),
        otp_input: cx.new(|cx| {
            OtpState::new(6, window, cx).default_value("849201")
        }),
        custom_compare_text: "prod-db-cluster.internal".to_string(),
        custom_compare_cursor: 24,
        custom_compare_selection: None,
        custom_compare_drag_anchor: None,
        }
    }
}

pub fn render_components_lab(app: Entity<CrowApp>, app_data: &CrowApp) -> Div {
    let lab = &app_data.lab_state;

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
                                .child("UI COMPONENTS LAB"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_DIM)
                                .child("· gpui-component transition testbed"),
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
                        .child("ISOLATED SANDBOX"),
                ),
        )
        // Scrollable Testbed Content
        .child(
            div()
                .id("lab-scroll-testbed")
                .flex_1()
                .overflow_y_scroll()
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
                                        .child("EVALUATION & TRANSITION SANDBOX"),
                                ),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_DIM)
                                .line_height(relative(1.4))
                                .child(
                                    "This page runs gpui-component widgets directly inside Crow to test platform-native text selection, cursor blinking, clipboard chords, undo/redo, and HarfBuzz text shaping before migrating existing screens.",
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
                                .child("1. DIRECT SIDE-BY-SIDE COMPARISON"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(16.0))
                                // Left: Native gpui-component Input
                                .child(
                                    div()
                                        .flex_1()
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
                                                        .child("NATIVE GPUI-COMPONENT INPUT"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.0))
                                                        .bg(OK_BG)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .text_color(OK_INK)
                                                        .child("Rope Buffer + Shaping"),
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
                                                .child("✓ Native mouse drag selection\n✓ Cmd+A, Cmd+C, Cmd+V, Cmd+Z (undo!)\n✓ Word jumps (Alt+Arrows) & line ends (Cmd+Arrows)"),
                                        ),
                                )
                                // Right: Hand-rolled terminal_text_input
                                .child(
                                    div()
                                        .flex_1()
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
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(TEXT_MUTED)
                                                        .child("CUSTOM HAND-ROLLED DIV INPUT"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.0))
                                                        .bg(hex_rgb(0x1a1a24))
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .text_color(TEXT_FAINT)
                                                        .child("Manual Math + Ticker"),
                                                ),
                                        )
                                        .child({
                                            let app_c = app.clone();
                                            terminal_text_input(
                                                "lab-input-custom",
                                                &lab.custom_compare_text,
                                                "Type in custom input...",
                                                true,
                                                false,
                                                lab.custom_compare_cursor,
                                                lab.custom_compare_selection,
                                                lab.custom_compare_drag_anchor,
                                                app_data.caret.blink,
                                                move |cursor, anchor, selection, _window, cx| {
                                                    app_c.update(cx, |this, cx| {
                                                        this.lab_state.custom_compare_cursor = cursor;
                                                        this.lab_state.custom_compare_drag_anchor = anchor;
                                                        this.lab_state.custom_compare_selection = selection;
                                                        this.caret.blink = true;
                                                        cx.notify();
                                                    });
                                                },
                                            )
                                        })
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_DIMMER)
                                                .line_height(relative(1.4))
                                                .child("· Prepaint canvas origin X translation\n· Fixed 0.6*size monospace character advance\n· Custom 530ms async timer blink loop"),
                                        ),
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
                                .child("2. INPUT VARIANTS & CAPABILITIES"),
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
                                                .child("PASSWORD INPUT (WITH NATIVE MASK TOGGLE & CLEAR BUTTON):"),
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
                                                .child("SEARCH / FILTER INPUT (CLEANABLE):"),
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
                                                .child("PREFIX & SUFFIX ADORNMENTS:"),
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
                                                        .child("PORT 22"),
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
                                .child("3. 6-DIGIT OTP AUTHENTICATOR INPUT"),
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
                                        .child("GROUPED OTP CELLS (SPLIT 3-3 WITH AUTO-ADVANCE & BACKSPACE):"),
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
                                        .child("Direct replacement candidate for Vault Lock and Setup screens."),
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
                                .child("4. MULTI-LINE TEXTAREA"),
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
                                        .child("MULTI-LINE EDITOR WITH WORD-WRAP & MULTI-LINE SELECTION:"),
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
                                .child("5. GPUI-COMPONENT BUTTON MATRIX"),
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
                                .child(Button::new("btn-primary").label("Primary Action").primary())
                                .child(Button::new("btn-secondary").label("Secondary").secondary())
                                .child(Button::new("btn-ghost").label("Ghost").ghost())
                                .child(Button::new("btn-danger").label("Destructive").danger()),
                        ),
                ),
        )
}
