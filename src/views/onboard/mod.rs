use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};
use crate::components::terminal_text_input;
use crate::os_detect::classify_distro_family;

pub mod probe;
#[allow(unused_imports)]
pub use probe::{probe_host, append_to_known_hosts, check_known_hosts, DetectedFacts, ProbeLog, ProbeResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardStep {
    Address = 1,
    Credentials = 2,
    VerifyHost = 3,
    Classify = 4,
    Finish = 5,
}

impl OnboardStep {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Address => "Address",
            Self::Credentials => "Credentials",
            Self::VerifyHost => "Verify Host",
            Self::Classify => "Classify",
            Self::Finish => "Install Agent",
        }
    }

    pub fn num(&self) -> usize {
        *self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardFieldFocus {
    Host,
    Port,
    User,
    Password,
    Label,
    Tags,
    None,
}

#[derive(Clone, Debug)]
pub struct OnboardState {
    pub step: OnboardStep,
    pub max_reached_step: OnboardStep,
    // Step 1: Address
    pub host: String,
    pub port: String,
    // Step 2: Credentials
    pub user: String,
    pub auth_method: String, // "publickey", "agent", "password"
    pub selected_key_id: Option<String>,
    pub jump_host_id: Option<String>,
    pub password: String,
    // Step 3: Probe
    pub probe_result: Option<ProbeResult>,
    pub probe_logs: Vec<ProbeLog>,
    pub host_key_accepted: bool,
    pub is_probing: bool,
    // Step 4: Classify
    pub label: String,
    pub env: String, // "PROD", "STAGE", "DEV", "LAB"
    pub role: String, // "web · nginx", "postgres 16", "cache · queue", "sidekiq", "ssh jump", "custom"
    pub group: String, // "workers", "edge", "data", "staging"
    pub tags: String,
    // Step 5: Install Agent / Review
    pub install_agent: bool,
    pub facts: DetectedFacts,
    // UI state
    pub focus: OnboardFieldFocus,
    pub error_message: Option<String>,
}

impl OnboardState {
    pub fn new(enrolled_keys: &[crate::vault::SshKeyRecord]) -> Self {
        let default_key = enrolled_keys.first().map(|k| k.id.clone());
        Self {
            step: OnboardStep::Address,
            max_reached_step: OnboardStep::Address,
            host: "10.0.4.32".into(),
            port: "22".into(),
            user: "root".into(),
            auth_method: "publickey".into(),
            selected_key_id: default_key,
            jump_host_id: None,
            password: String::new(),
            probe_result: None,
            probe_logs: Vec::new(),
            host_key_accepted: false,
            is_probing: false,
            label: "worker-05".into(),
            env: "PROD".into(),
            role: "sidekiq".into(),
            group: "workers".into(),
            tags: "queue, ruby, eu-west".into(),
            install_agent: false,
            facts: DetectedFacts::default(),
            focus: OnboardFieldFocus::Host,
            error_message: None,
        }
    }
}

pub fn onboard_view(app: Entity<CrowApp>, app_data: &CrowApp) -> impl IntoElement {
    let state = &app_data.onboard_state;

    let steps = [
        OnboardStep::Address,
        OnboardStep::Credentials,
        OnboardStep::VerifyHost,
        OnboardStep::Classify,
        OnboardStep::Finish,
    ];

    let app_cancel = app.clone();
    let app_prev = app.clone();
    let app_next = app.clone();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Top Header
        .child(
            div()
                .h(px(52.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child("ADD SERVER"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .text_color(TEXT_DIM)
                                .child("enroll a new host into fleet"),
                        ),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-cancel-onboard")
                        .px(px(10.0))
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_cancel.update(cx, |this, cx| {
                                this.set_screen(Screen::Fleet, cx);
                            });
                        })
                        .child("CANCEL esc"),
                ),
        )
        // 2. 5-Step Stepper Strip
        .child(
            div()
                .h(px(36.0))
                .flex_none()
                .flex()
                .items_stretch()
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .children(steps.iter().enumerate().map(|(idx, step)| {
                    let is_active = state.step == *step;
                    let is_done = state.step.num() > step.num();
                    let is_clickable = step.num() <= state.max_reached_step.num();

                    let app_step = app.clone();
                    let target_step = *step;

                    let mut tab = div()
                        .id(ElementId::NamedInteger("onboard-step-tab".into(), idx as u64))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(16.0))
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .bg(if is_active { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                        .border_b_2()
                        .border_color(if is_active { TEXT_PRIMARY } else { hex_rgba(0, 0.0) });
                    if is_clickable {
                        tab = tab.cursor_pointer().hover(|h| h.bg(BG_ROW_HOVER));
                    }
                    tab.on_click(move |_ev, _window, cx| {
                        if is_clickable {
                            app_step.update(cx, |this, cx| {
                                this.onboard_set_step(target_step, cx);
                            });
                        }
                    })
                        .child(
                            div()
                                .size(px(18.0))
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .bg(if is_done { OK_BG } else if is_active { BG_CONTROL_ALT } else { hex_rgba(0, 0.0) })
                                .text_color(if is_done { OK } else if is_active { TEXT_MAX } else { TEXT_FAINT })
                                .child(if is_done { "✓".to_string() } else { step.num().to_string() }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .text_color(if is_active { TEXT_MAX } else if is_done { TEXT_PRIMARY } else { TEXT_DIMMER })
                                .child(step.label()),
                        )
                }))
                .child(div().flex_1()),
        )
        // 3. Main Split: Target Form (flex-1) | Facts & Probe Rail (420px)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Left Column: Interactive Step Form
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .bg(BG_APP)
                        .child(
                            div()
                                .id("onboard-form-list")
                                .flex_1()
                                .overflow_y_scroll()
                                .p(px(16.0))
                                .flex()
                                .flex_col()
                                .gap(px(12.0))
                                .child(render_step_content(app.clone(), app_data)),
                        )
                        // Error message if any
                        .children(if let Some(ref err) = state.error_message {
                            Some(
                                div()
                                    .px(px(18.0))
                                    .py(px(6.0))
                                    .bg(hex_rgb(0x2e1114))
                                    .border_t_1()
                                    .border_color(CRIT)
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.5))
                                    .text_color(CRIT)
                                    .child(err.clone()),
                            )
                        } else {
                            None
                        })
                        // Form Bottom Actions Bar
                        .child(
                            div()
                                .h(px(46.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(18.0))
                                .bg(BG_PANEL)
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(8.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(
                                    div()
                                        .id("btn-onboard-back")
                                        .px(px(12.0))
                                        .py(px(6.0))
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .text_color(TEXT_TERTIARY)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            app_prev.update(cx, |this, cx| {
                                                this.onboard_prev_step(cx);
                                            });
                                        })
                                        .child(if state.step == OnboardStep::Address { "✕ CANCEL" } else { "← BACK" }),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .id("btn-onboard-next")
                                        .px(px(16.0))
                                        .py(px(7.0))
                                        .bg(if state.step == OnboardStep::Finish { OK } else { BG_CONTROL })
                                        .border_1()
                                        .border_color(if state.step == OnboardStep::Finish { OK } else { BORDER_DEFAULT })
                                        .text_color(if state.step == OnboardStep::Finish { BG_WINDOW } else { TEXT_MAX })
                                        .font_weight(FontWeight::BOLD)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(if state.step == OnboardStep::Finish { hex_rgb(0x32b55e) } else { BG_ROW_HOVER }))
                                        .on_click(move |_ev, _window, cx| {
                                            app_next.update(cx, |this, cx| {
                                                this.onboard_next_step(cx);
                                            });
                                        })
                                        .child(match state.step {
                                            OnboardStep::Address => "NEXT: CREDENTIALS →",
                                            OnboardStep::Credentials => "NEXT: VERIFY HOST →",
                                            OnboardStep::VerifyHost => "NEXT: CLASSIFY →",
                                            OnboardStep::Classify => "NEXT: REVIEW & FINISH →",
                                            OnboardStep::Finish => "COMPLETE ENROLLMENT ⌘⏎",
                                        }),
                                ),
                        ),
                )
                // Right Rail: Fingerprint, Probe Log, Detected Facts, Schema Packs
                .child(render_right_rail(app.clone(), app_data)),
        )
}

fn render_step_content(app: Entity<CrowApp>, app_data: &CrowApp) -> Div {
    let state = &app_data.onboard_state;

    match state.step {
        OnboardStep::Address => render_step_address(app, app_data),
        OnboardStep::Credentials => render_step_credentials(app, app_data),
        OnboardStep::VerifyHost => render_step_verify(app, state),
        OnboardStep::Classify => render_step_classify(app, app_data),
        OnboardStep::Finish => render_step_finish(app, state, &app_data.keys.enrolled),
    }
}

// -----------------------------------------------------------------------------
// Step 1: Address
// -----------------------------------------------------------------------------
fn render_step_address(app: Entity<CrowApp>, app_data: &CrowApp) -> Div {
    let state = &app_data.onboard_state;
    let app_host = app.clone();
    let app_port = app.clone();
    let is_host_focused = state.focus == OnboardFieldFocus::Host;
    let is_port_focused = state.focus == OnboardFieldFocus::Port;

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(13.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MAX)
                .child("STEP 1: NETWORK ADDRESS & PORT"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_DIM)
                .child("Provide the target server's IPv4, IPv6, or fully qualified domain name (FQDN)."),
        )
        // Local Lab Test Nodes Quick-Pick (if any exist)
        .children(if !app_data.local_lab.nodes.is_empty() {
            let app_pick = app.clone();
            Some(
                div()
                    .p(px(10.0))
                    .bg(hex_rgb(0x0f1016))
                    .border_1()
                    .border_color(BORDER_DEFAULT)
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
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(OK)
                                    .child("⚡ OR PICK A DETECTED LOCAL TEST NODE"),
                            )
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(9.0))
                                    .text_color(TEXT_MUTED)
                                    .child("Distrobox / Podman"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .children(app_data.local_lab.nodes.iter().enumerate().map(|(idx, node)| {
                                let app = app_pick.clone();
                                let node_name = node.name.clone();
                                let port_str = node.ssh_port.unwrap_or(2222).to_string();
                                let distro = node.distro_display();
                                div()
                                    .id(ElementId::NamedInteger("quick-pick-lab-node".into(), idx as u64))
                                    .p(px(6.0))
                                    .bg(BG_CONTROL)
                                    .border_1()
                                    .border_color(BORDER_DEFAULT)
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .on_click(move |_ev, _window, cx| {
                                        let name = node_name.clone();
                                        let p = port_str.clone();
                                        let d = distro.clone();
                                        app.update(cx, |this, cx| {
                                            this.onboard_select_local_lab_node(&name, &p, &d, cx);
                                        });
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .size(px(6.0))
                                                    .rounded_full()
                                                    .bg(if node.is_running() { OK } else { TEXT_DIMMER }),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.5))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(TEXT_PRIMARY)
                                                    .child(node.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.0))
                                                    .text_color(TEXT_TERTIARY)
                                                    .child(format!("({})", node.engine.label())),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(9.5))
                                            .text_color(TEXT_MUTED)
                                            .child(format!("127.0.0.1:{}", node.ssh_port.unwrap_or(2222))),
                                    )
                            }))
                    )
            )
        } else {
            None
        })
        // Host field
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_host_focused { TEXT_PRIMARY } else { TEXT_DIM })
                        .child("HOSTNAME OR IP ADDRESS:"),
                )
                .child(
                    terminal_text_input(
                        "input-onboard-host",
                        &state.host,
                        "e.g. 10.0.4.32 or prod-db.internal",
                        is_host_focused,
                        false,
                        if is_host_focused { app_data.caret.cursor } else { 0 },
                        if is_host_focused { app_data.caret.selection } else { None },
                        if is_host_focused { app_data.caret.drag_anchor } else { None },
                        app_data.caret.blink,
                        {
                            let app = app_host;
                            move |cursor, anchor, selection, _window, cx| {
                                app.update(cx, |this, cx| {
                                    this.onboard_state.focus = OnboardFieldFocus::Host;
                                    this.caret.cursor = cursor;
                                    this.caret.drag_anchor = anchor;
                                    this.caret.selection = selection;
                                    this.caret.blink = true;
                                    cx.notify();
                                });
                            }
                        },
                    ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("resolves · 1 A record or direct routable IP address"),
                ),
        )
        // Port field
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_port_focused { TEXT_PRIMARY } else { TEXT_DIM })
                        .child("SSH PORT:"),
                )
                .child(
                    terminal_text_input(
                        "input-onboard-port",
                        &state.port,
                        "22",
                        is_port_focused,
                        false,
                        if is_port_focused { app_data.caret.cursor } else { 0 },
                        if is_port_focused { app_data.caret.selection } else { None },
                        if is_port_focused { app_data.caret.drag_anchor } else { None },
                        app_data.caret.blink,
                        {
                            let app = app_port;
                            move |cursor, anchor, selection, _window, cx| {
                                app.update(cx, |this, cx| {
                                    this.onboard_state.focus = OnboardFieldFocus::Port;
                                    this.caret.cursor = cursor;
                                    this.caret.drag_anchor = anchor;
                                    this.caret.selection = selection;
                                    this.caret.blink = true;
                                    cx.notify();
                                });
                            }
                        },
                    )
                    .w(px(120.0)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("standard OpenSSH daemon default is 22"),
                ),
        )
}

// -----------------------------------------------------------------------------
// Step 2: Credentials
// -----------------------------------------------------------------------------
fn render_step_credentials(app: Entity<CrowApp>, app_data: &CrowApp) -> Div {
    let state = &app_data.onboard_state;
    let enrolled_keys = &app_data.keys.enrolled;
    let servers = &app_data.fleet.servers;
    let app_user = app.clone();
    let app_auth_pub = app.clone();
    let app_auth_agent = app.clone();
    let app_auth_pass = app.clone();
    let app_pw = app.clone();
    let app_jump_none = app.clone();
    let is_user_focused = state.focus == OnboardFieldFocus::User;
    let is_pw_focused = state.focus == OnboardFieldFocus::Password;

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(13.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MAX)
                .child("STEP 2: LOGIN CREDENTIALS & SSH IDENTITY"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_DIM)
                .child("Configure login user and cryptographic authentication identity."),
        )
        // Login user
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_user_focused { TEXT_PRIMARY } else { TEXT_DIM })
                        .child("LOGIN USER:"),
                )
                .child(
                    terminal_text_input(
                        "input-onboard-user",
                        &state.user,
                        "root (or ubuntu, deploy, admin…)",
                        is_user_focused,
                        false,
                        if is_user_focused { app_data.caret.cursor } else { 0 },
                        if is_user_focused { app_data.caret.selection } else { None },
                        if is_user_focused { app_data.caret.drag_anchor } else { None },
                        app_data.caret.blink,
                        {
                            let app = app_user;
                            move |cursor, anchor, selection, _window, cx| {
                                app.update(cx, |this, cx| {
                                    this.onboard_state.focus = OnboardFieldFocus::User;
                                    this.caret.cursor = cursor;
                                    this.caret.drag_anchor = anchor;
                                    this.caret.selection = selection;
                                    this.caret.blink = true;
                                    cx.notify();
                                });
                            }
                        },
                    ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("administrative remote user (root, ubuntu, deploy, admin)"),
                ),
        )
        // Auth method chips
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIM)
                        .child("AUTHENTICATION METHOD:"),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(
                            div()
                                .id("chip-auth-pubkey")
                                .px(px(10.0))
                                .py(px(5.0))
                                .bg(if state.auth_method == "publickey" { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if state.auth_method == "publickey" { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_auth_pub.update(cx, |this, cx| {
                                        this.onboard_state.auth_method = "publickey".into();
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(if state.auth_method == "publickey" { TEXT_MAX } else { TEXT_DIM })
                                        .child("● Public Key (Vault Key)"),
                                ),
                        )
                        .child(
                            div()
                                .id("chip-auth-agent")
                                .px(px(10.0))
                                .py(px(5.0))
                                .bg(if state.auth_method == "agent" { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if state.auth_method == "agent" { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_auth_agent.update(cx, |this, cx| {
                                        this.onboard_state.auth_method = "agent".into();
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(if state.auth_method == "agent" { TEXT_MAX } else { TEXT_DIM })
                                        .child("○ SSH Agent Forwarding"),
                                ),
                        )
                        .child(
                            div()
                                .id("chip-auth-password")
                                .px(px(10.0))
                                .py(px(5.0))
                                .bg(if state.auth_method == "password" { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if state.auth_method == "password" { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_auth_pass.update(cx, |this, cx| {
                                        this.onboard_state.auth_method = "password".into();
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(if state.auth_method == "password" { TEXT_MAX } else { TEXT_DIM })
                                        .child("○ Password Auth"),
                                ),
                        ),
                ),
        )
        // Public key selector (if publickey)
        .children(if state.auth_method == "publickey" {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(5.0))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(TEXT_DIM)
                            .child("ENROLLED SSH IDENTITY KEY:"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .children(if enrolled_keys.is_empty() {
                                vec![
                                    div()
                                        .p(px(8.0))
                                        .bg(WARN_BG)
                                        .border_1()
                                        .border_color(WARN)
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(WARN_INK)
                                        .child("No SSH keys enrolled in Crow. Go to Settings → Keys & Rotation to import or generate a key.")
                                        .into_any_element(),
                                ]
                            } else {
                                enrolled_keys.iter().enumerate().map(|(idx, k)| {
                                    let is_selected = state.selected_key_id.as_deref() == Some(&k.id);
                                    let app_sel_key = app.clone();
                                    let kid = k.id.clone();

                                    div()
                                        .id(ElementId::NamedInteger("chip-enrolled-key".into(), idx as u64))
                                        .p(px(8.0))
                                        .bg(if is_selected { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                        .border_1()
                                        .border_color(if is_selected { OK } else { BORDER_DEFAULT })
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| {
                                            let kid_c = kid.clone();
                                            app_sel_key.update(cx, |this, cx| {
                                                this.onboard_state.selected_key_id = Some(kid_c);
                                                cx.notify();
                                            });
                                        })
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                        .text_color(if is_selected { OK } else { TEXT_PRIMARY })
                                                        .child(format!("{} {}", if is_selected { "✓" } else { "○" }, k.name)),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(4.0))
                                                        .py(px(1.0))
                                                        .bg(BG_CHIP)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.0))
                                                        .text_color(TEXT_DIM)
                                                        .child(k.algorithm.clone()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_DIMMER)
                                                .child(k.fingerprint.clone()),
                                        )
                                        .into_any_element()
                                }).collect()
                            }),
                    ),
            )
        } else if state.auth_method == "password" {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(if is_pw_focused { TEXT_PRIMARY } else { TEXT_DIM })
                            .child("PASSWORD:"),
                    )
                    .child(
                        terminal_text_input(
                            "input-onboard-pw",
                            &state.password,
                            "Enter remote password…",
                            is_pw_focused,
                            true,
                            if is_pw_focused { app_data.caret.cursor } else { 0 },
                            if is_pw_focused { app_data.caret.selection } else { None },
                            if is_pw_focused { app_data.caret.drag_anchor } else { None },
                            app_data.caret.blink,
                            {
                                let app = app_pw;
                                move |cursor, anchor, selection, _window, cx| {
                                    app.update(cx, |this, cx| {
                                        this.onboard_state.focus = OnboardFieldFocus::Password;
                                        this.caret.cursor = cursor;
                                        this.caret.drag_anchor = anchor;
                                        this.caret.selection = selection;
                                        this.caret.blink = true;
                                        cx.notify();
                                    });
                                }
                            },
                        ),
                    ),
            )
        } else {
            None
        })
        // Jump host / bastion
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIM)
                        .child("JUMP HOST / BASTION:"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .child(
                            div()
                                .id("chip-jump-none")
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(if state.jump_host_id.is_none() { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if state.jump_host_id.is_none() { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_jump_none.update(cx, |this, cx| {
                                        this.onboard_state.jump_host_id = None;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(if state.jump_host_id.is_none() { TEXT_MAX } else { TEXT_DIM })
                                        .child("Direct (none)"),
                                ),
                        )
                        .children(servers.iter().filter(|s| s.role.contains("jump") || s.role.contains("bastion") || s.name == "bastion").enumerate().map(|(idx, s)| {
                            let is_sel = state.jump_host_id.as_deref() == Some(&s.id);
                            let app_jump = app.clone();
                            let sid = s.id.clone();

                            div()
                                .id(ElementId::NamedInteger("chip-jump-server".into(), idx as u64))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(if is_sel { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let sid_c = sid.clone();
                                    app_jump.update(cx, |this, cx| {
                                        this.onboard_state.jump_host_id = Some(sid_c);
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(if is_sel { TEXT_MAX } else { TEXT_DIM })
                                        .child(format!("{} ({} · {})", s.name, s.host, s.role)),
                                )
                        })),
                ),
        )
}

// -----------------------------------------------------------------------------
// Step 3: Verify Host
// -----------------------------------------------------------------------------
fn render_step_verify(app: Entity<CrowApp>, state: &OnboardState) -> Div {
    let app_probe = app.clone();
    let app_accept = app.clone();

    let probe_done = state.probe_result.is_some();
    let is_reachable = state.probe_result.as_ref().map(|p| p.is_reachable).unwrap_or(false);
    let latency = state.probe_result.as_ref().and_then(|p| p.latency_ms);
    let banner = state.probe_result.as_ref().and_then(|p| p.ssh_banner.clone());

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(13.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MAX)
                .child("STEP 3: HOST VERIFICATION & PRE-FLIGHT PROBE"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_DIM)
                .child("Perform an active TCP handshake to measure latency, read the SSH banner, and verify host keys."),
        )
        // Connection test card
        .child(
            div()
                .p(px(12.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_DIM)
                                .child("TARGET CONNECTION DETAILS:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_reachable { OK } else if probe_done { WARN } else { TEXT_DIMMER })
                                .child(if is_reachable {
                                    format!("● ONLINE ({}ms)", latency.unwrap_or(0))
                                } else if probe_done {
                                    "▲ SIMULATED READINESS".to_string()
                                } else {
                                    "○ READY TO PROBE".to_string()
                                }),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(12.0))
                        .text_color(TEXT_MAX)
                        .child(format!("{}@{}:{}", state.user, state.host, state.port)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child(format!("banner: {}", banner.unwrap_or_else(|| "SSH-2.0-OpenSSH_9.6p1".to_string()))),
                )
                .child(
                    div()
                        .id("btn-run-probe")
                        .px(px(12.0))
                        .py(px(6.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_probe.update(cx, |this, cx| {
                                this.onboard_run_probe(cx);
                            });
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK)
                                .child("⚡ RUN CONNECTION PROBE"),
                        ),
                ),
        )
        // Fingerprint verification state
        .child(
            div()
                .p(px(12.0))
                .bg(if state.host_key_accepted { hex_rgb(0x0c1b12) } else { hex_rgb(0x1a1208) })
                .border_1()
                .border_color(if state.host_key_accepted { OK } else { WARN })
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(if state.host_key_accepted { OK } else { WARN })
                                .child(if state.host_key_accepted { "✓" } else { "▲" }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if state.host_key_accepted { OK } else { WARN })
                                .child(if state.host_key_accepted {
                                    "HOST KEY FINGERPRINT VERIFIED & TRUSTED"
                                } else {
                                    "UNKNOWN HOST KEY FINGERPRINT (WAITING ON YOU)"
                                }),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_PRIMARY)
                        .child(state.probe_result.as_ref().map(|p| p.host_key_fingerprint.clone()).unwrap_or_else(|| "SHA256:4a8b812f00... (ED25519)".to_string())),
                )
                .children(if !state.host_key_accepted {
                    Some(
                        div()
                            .id("btn-accept-host-key")
                            .px(px(10.0))
                            .py(px(5.0))
                            .bg(OK)
                            .text_color(BG_WINDOW)
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0x32b55e)))
                            .on_click(move |_ev, _window, cx| {
                                app_accept.update(cx, |this, cx| {
                                    this.onboard_accept_host_key(cx);
                                });
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("✓ ACCEPT & TRUST FINGERPRINT"),
                    )
                } else {
                    None
                }),
        )
}

// -----------------------------------------------------------------------------
// Step 4: Classify
// -----------------------------------------------------------------------------
fn render_step_classify(app: Entity<CrowApp>, app_data: &CrowApp) -> Div {
    let state = &app_data.onboard_state;
    let app_label = app.clone();
    let app_tags = app.clone();
    let is_label_focused = state.focus == OnboardFieldFocus::Label;
    let is_tags_focused = state.focus == OnboardFieldFocus::Tags;

    let envs = ["PROD", "STAGE", "DEV", "LAB"];
    let roles = ["web · nginx", "postgres 16", "cache · queue", "sidekiq", "ssh jump", "prometheus", "custom"];
    let groups = ["workers", "edge", "data", "staging", "fleet", "bastions"];

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(13.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MAX)
                .child("STEP 4: CLASSIFY & METADATA TAXONOMY"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_DIM)
                .child("Assign unique fleet label, environment, role, and policy groups."),
        )
        // Label
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_label_focused { TEXT_PRIMARY } else { TEXT_DIM })
                        .child("SERVER LABEL (UNIQUE IDENTIFIER):"),
                )
                .child(
                    terminal_text_input(
                        "input-onboard-label",
                        &state.label,
                        "e.g. worker-05, edge-eu, db-primary",
                        is_label_focused,
                        false,
                        if is_label_focused { app_data.caret.cursor } else { 0 },
                        if is_label_focused { app_data.caret.selection } else { None },
                        if is_label_focused { app_data.caret.drag_anchor } else { None },
                        app_data.caret.blink,
                        {
                            let app = app_label;
                            move |cursor, anchor, selection, _window, cx| {
                                app.update(cx, |this, cx| {
                                    this.onboard_state.focus = OnboardFieldFocus::Label;
                                    this.caret.cursor = cursor;
                                    this.caret.drag_anchor = anchor;
                                    this.caret.selection = selection;
                                    this.caret.blink = true;
                                    cx.notify();
                                });
                            }
                        },
                    ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_DIMMER)
                        .child("must be unique across all servers in your fleet"),
                ),
        )
        // Environment chips
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIM)
                        .child("ENVIRONMENT:"),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .children(envs.iter().map(|e| {
                            let is_sel = state.env == *e;
                            let app_env = app.clone();
                            let env_val = e.to_string();

                            div()
                                .id(ElementId::Name(format!("chip-env-{}", e).into()))
                                .px(px(12.0))
                                .py(px(4.0))
                                .bg(if is_sel {
                                    match *e {
                                        "PROD" => CRIT_BG,
                                        "STAGE" => WARN_BG,
                                        _ => BG_OVERLAY_PANEL,
                                    }
                                } else {
                                    BG_CONTROL
                                })
                                .border_1()
                                .border_color(if is_sel {
                                    match *e {
                                        "PROD" => CRIT,
                                        "STAGE" => WARN,
                                        _ => TEXT_PRIMARY,
                                    }
                                } else {
                                    BORDER_DEFAULT
                                })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let ev = env_val.clone();
                                    app_env.update(cx, |this, cx| {
                                        this.onboard_state.env = ev;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel {
                                            match *e {
                                                "PROD" => CRIT,
                                                "STAGE" => WARN,
                                                _ => TEXT_MAX,
                                            }
                                        } else {
                                            TEXT_DIM
                                        })
                                        .child(*e),
                                )
                        })),
                ),
        )
        // Role chips
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIM)
                        .child("SERVER ROLE:"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(roles.iter().map(|r| {
                            let is_sel = state.role == *r;
                            let app_r = app.clone();
                            let r_val = r.to_string();

                            div()
                                .id(ElementId::Name(format!("chip-role-{}", r).into()))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(if is_sel { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let rv = r_val.clone();
                                    app_r.update(cx, |this, cx| {
                                        this.onboard_state.role = rv;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(if is_sel { TEXT_MAX } else { TEXT_DIM })
                                        .child(*r),
                                )
                        })),
                ),
        )
        // Group chips
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIM)
                        .child("GROUP ASSIGNMENT:"),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(groups.iter().map(|g| {
                            let is_sel = state.group == *g;
                            let app_g = app.clone();
                            let g_val = g.to_string();

                            div()
                                .id(ElementId::Name(format!("chip-group-{}", g).into()))
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(if is_sel { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let gv = g_val.clone();
                                    app_g.update(cx, |this, cx| {
                                        this.onboard_state.group = gv;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .text_color(if is_sel { TEXT_MAX } else { TEXT_DIM })
                                        .child(*g),
                                )
                        })),
                ),
        )
        // Tags
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_tags_focused { TEXT_PRIMARY } else { TEXT_DIM })
                        .child("TAGS (COMMA SEPARATED):"),
                )
                .child(
                    terminal_text_input(
                        "input-onboard-tags",
                        &state.tags,
                        "e.g. queue, ruby, eu-west, staging",
                        is_tags_focused,
                        false,
                        if is_tags_focused { app_data.caret.cursor } else { 0 },
                        if is_tags_focused { app_data.caret.selection } else { None },
                        if is_tags_focused { app_data.caret.drag_anchor } else { None },
                        app_data.caret.blink,
                        {
                            let app = app_tags;
                            move |cursor, anchor, selection, _window, cx| {
                                app.update(cx, |this, cx| {
                                    this.onboard_state.focus = OnboardFieldFocus::Tags;
                                    this.caret.cursor = cursor;
                                    this.caret.drag_anchor = anchor;
                                    this.caret.selection = selection;
                                    this.caret.blink = true;
                                    cx.notify();
                                });
                            }
                        },
                    ),
                ),
        )
}

// -----------------------------------------------------------------------------
// Step 5: Finish & Install Agent
// -----------------------------------------------------------------------------
fn render_step_finish(
    app: Entity<CrowApp>,
    state: &OnboardState,
    enrolled_keys: &[crate::vault::SshKeyRecord],
) -> Div {
    let app_toggle_agent = app.clone();
    let key_name = state.selected_key_id.as_ref()
        .and_then(|kid| enrolled_keys.iter().find(|k| &k.id == kid).map(|k| k.name.clone()))
        .unwrap_or_else(|| "id_ed25519_fleet".to_string());

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(13.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MAX)
                .child("STEP 5: REVIEW CONFIGURATION & COMMIT ENROLLMENT"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_DIM)
                .child("Review discovered host profile and choose agentless or daemon mode."),
        )
        // Summary Table
        .child(
            div()
                .p(px(12.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIMMER)
                        .child("SERVER CONFIGURATION SUMMARY:"),
                )
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .child(div().text_color(TEXT_DIM).child("Label / Name:"))
                        .child(div().font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child(state.label.clone())),
                )
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .child(div().text_color(TEXT_DIM).child("Endpoint:"))
                        .child(div().text_color(TEXT_PRIMARY).child(format!("{}@{}:{}", state.user, state.host, state.port))),
                )
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .child(div().text_color(TEXT_DIM).child("Authentication:"))
                        .child(div().text_color(OK).child(format!("{} ({})", state.auth_method, key_name))),
                )
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .child(div().text_color(TEXT_DIM).child("Classification:"))
                        .child(div().text_color(TEXT_PRIMARY).child(format!("{} · {} · group: {}", state.env, state.role, state.group))),
                )
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .child(div().text_color(TEXT_DIM).child("Tags:"))
                        .child(div().text_color(TEXT_MUTED).child(state.tags.clone())),
                ),
        )
        // Agent Deployment Choice
        .child(
            div()
                .p(px(12.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIMMER)
                        .child("TELEMETRY MODE:"),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(10.0))
                        .child(
                            div()
                                .id("chip-mode-agentless")
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(if !state.install_agent { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if !state.install_agent { OK } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_toggle_agent.update(cx, |this, cx| {
                                        this.onboard_state.install_agent = false;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(if !state.install_agent { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if !state.install_agent { OK } else { TEXT_DIM })
                                        .child("● Pure Agentless SSH (Zero footprint)"),
                                ),
                        )
                        .child({
                            let app_tog2 = app.clone();
                            div()
                                .id("chip-mode-agent")
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(if state.install_agent { BG_OVERLAY_PANEL } else { BG_CONTROL })
                                .border_1()
                                .border_color(if state.install_agent { OK } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_tog2.update(cx, |this, cx| {
                                        this.onboard_state.install_agent = true;
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(if state.install_agent { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if state.install_agent { OK } else { TEXT_DIM })
                                        .child("○ Install crow-agent 0.9.4 (1.8 MB daemon)"),
                                )
                        }),
                ),
        )
}

// -----------------------------------------------------------------------------
// Right Rail: Fingerprint, Probe Log, Detected Facts, Schema Packs
// -----------------------------------------------------------------------------
fn render_right_rail(app: Entity<CrowApp>, app_data: &CrowApp) -> Div {
    let state = &app_data.onboard_state;
    let app_accept = app.clone();

    let fp = state.probe_result.as_ref()
        .map(|p| p.host_key_fingerprint.clone())
        .unwrap_or_else(|| "SHA256:4a8b812f00... (ED25519)".to_string());

    let facts: [(&'static str, String, Rgba); 10] = [
        ("DISTRO", state.facts.distro.clone(), TEXT_PRIMARY),
        ("KERNEL", state.facts.kernel.clone(), TEXT_SECONDARY),
        ("ARCH", state.facts.arch.clone(), TEXT_SECONDARY),
        ("MEMORY", state.facts.memory.clone(), TEXT_SECONDARY),
        ("DISK", state.facts.disk.clone(), TEXT_SECONDARY),
        ("INIT", state.facts.init.clone(), TEXT_SECONDARY),
        ("OPEN PORTS", state.facts.open_ports.clone(), TEXT_SECONDARY),
        ("FIREWALL", state.facts.firewall.clone(), OK),
        ("TIME", state.facts.time_sync.clone(), TEXT_SECONDARY),
        ("AGENT STATUS", if state.install_agent { "deploy pending".to_string() } else { "agentless SSH".to_string() }, if state.install_agent { WARN } else { OK }),
    ];

    div()
        .w(px(420.0))
        .flex_none()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        // Amber / Green Fingerprint Card
        .child(
            div()
                .p(px(14.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .bg(if state.host_key_accepted { hex_rgb(0x0a140e) } else { hex_rgb(0x100c06) })
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(7.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(if state.host_key_accepted { OK } else { WARN })
                                .child(if state.host_key_accepted { "✓" } else { "▲" }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if state.host_key_accepted { OK } else { WARN })
                                .child(if state.host_key_accepted { "HOST KEY VERIFIED IN KNOWN_HOSTS" } else { "UNKNOWN HOST KEY FINGERPRINT" }),
                        ),
                )
                .child(
                    div()
                        .p(px(8.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(if state.host_key_accepted { hex_rgb(0x1a3322) } else { hex_rgb(0x2e2210) })
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_PRIMARY)
                        .child(fp),
                )
                .children(if !state.host_key_accepted {
                    Some(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .pt(px(2.0))
                            .child(
                                div()
                                    .id("rail-btn-trust-fp")
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .bg(OK)
                                    .text_color(BG_WINDOW)
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(hex_rgb(0x32b55e)))
                                    .on_click(move |_ev, _window, cx| {
                                        app_accept.update(cx, |this, cx| {
                                            this.onboard_accept_host_key(cx);
                                        });
                                    })
                                    .child("TRUST & ADD TO KNOWN_HOSTS"),
                            ),
                    )
                } else {
                    None
                }),
        )
        // Probe Log
        .child(
            div()
                .h(px(28.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_DIMMER)
                .child("CONNECTION PROBE LOG"),
        )
        .child(
            div()
                .id("probe-log-list")
                .h(px(150.0))
                .overflow_y_scroll()
                .p(px(8.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .children(if state.probe_logs.is_empty() {
                    vec![
                        div()
                            .p(px(8.0))
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .text_color(TEXT_FAINT)
                            .child("Probe has not been executed yet. Click 'Run Connection Probe' or advance to step 3 to initiate pre-flight network diagnostics."),
                    ]
                } else {
                    state.probe_logs.iter().map(|log| {
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .py(px(1.5))
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .child(div().w(px(10.0)).text_color(log.color).child(log.glyph.clone()))
                            .child(div().flex_1().text_color(TEXT_MUTED).child(log.message.clone()))
                            .children(if !log.timestamp.is_empty() {
                                Some(div().text_color(TEXT_FAINTER).child(log.timestamp.clone()))
                            } else {
                                None
                            })
                            .children(if !log.note.is_empty() {
                                Some(div().text_color(TEXT_FAINT).child(log.note.clone()))
                            } else {
                                None
                            })
                    }).collect()
                }),
        )
        // Unverified distro warning — Crow only knows Debian- and Red Hat-family
        // conventions; say so plainly instead of quietly guessing wrong paths.
        .children(if !classify_distro_family(&state.facts.distro).is_supported() {
            Some(
                div()
                    .p(px(10.0))
                    .bg(hex_rgba(0xfbbf24, 0.08))
                    .border_b_1()
                    .border_color(WARN)
                    .flex()
                    .items_start()
                    .gap(px(8.0))
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(12.0))
                            .text_color(WARN)
                            .child("⚠"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(WARN)
                                    .child("Unverified distro"),
                            )
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(TEXT_DIM)
                                    .child("Crow only knows Debian- and Red Hat-family config layouts. Config discovery on this host may miss files or point at the wrong paths."),
                            ),
                    ),
            )
        } else {
            None
        })
        // Detected Facts Grid
        .child(
            div()
                .h(px(28.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_DIMMER)
                .child("DETECTED FACTS"),
        )
        .child(
            div()
                .id("facts-grid")
                .flex_1()
                .overflow_y_scroll()
                .p(px(10.0))
                .flex()
                .flex_col()
                .gap(px(5.0))
                .children(facts.into_iter().map(|(k, v, c)| {
                    div()
                        .flex()
                        .justify_between()
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .child(div().text_color(TEXT_DIMMER).child(k))
                        .child(div().text_color(c).font_weight(FontWeight::MEDIUM).child(v))
                })),
        )
        // Detected Schema Packs
        .child(
            div()
                .h(px(28.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_DIMMER)
                .child("DETECTED SCHEMA PACKS"),
        )
        .child(
            div()
                .p(px(10.0))
                .flex()
                .flex_wrap()
                .gap(px(6.0))
                .children(state.facts.schema_packs.iter().map(|(name, fg, bg)| {
                    div()
                        .px(px(6.0))
                        .py(px(2.5))
                        .bg(*bg)
                        .text_color(*fg)
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .child(name.clone())
                })),
        )
}
