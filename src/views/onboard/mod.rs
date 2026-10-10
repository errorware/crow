//! Add Server (ERR-153): three steps. Connect (where it is, how Crow gets
//! there and logs in), Verify (the real host key, then a login that reads
//! the server's facts), Name (how it shows in the fleet). Nothing is shown
//! before it's read.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::onboard::OnboardInputs;
use crate::app::CrowApp;
use crate::os_detect::classify_distro_family;
use crate::theme::*;
use crate::views::fleet::lab_state::LocalLabState;
use crate::views::fleet::FleetState;
use crate::views::settings::keys_state::KeysState;

pub mod probe;
#[allow(unused_imports)]
pub use probe::{gather_facts, log as probe_log, probe_host, trust_host_keys, DetectedFacts, ProbeLog, ProbeResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardStep {
    Connect = 1,
    Verify = 2,
    Name = 3,
}

impl OnboardStep {
    pub const ALL: [OnboardStep; 3] = [OnboardStep::Connect, OnboardStep::Verify, OnboardStep::Name];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Connect => t!("onboard-connect"),
            Self::Verify => t!("onboard-verify"),
            Self::Name => t!("onboard-name"),
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
    // Connect
    pub host: String,
    pub port: String,
    pub user: String,
    pub auth_method: String, // "publickey", "agent", "password"
    pub selected_key_id: Option<String>,
    pub jump_host_id: Option<String>,
    pub password: crate::secret_string::SecretString,
    // Verify
    pub probe_result: Option<ProbeResult>,
    pub probe_logs: Vec<ProbeLog>,
    pub host_key_accepted: bool,
    pub is_probing: bool,
    // Name
    pub label: String,
    pub env: String,
    pub role: String,
    pub group: String,
    pub tags: String,
    /// Other servers can be reached through this one (ERR-152).
    pub make_bastion: bool,
    pub facts: DetectedFacts,
    /// Set when the server was imported from a provider (ERR-46).
    pub provider: Option<ProviderLink>,
    // UI state
    pub focus: OnboardFieldFocus,
    pub error_message: Option<String>,
}

/// Which provider instance a server being added is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderLink {
    pub account: String,
    pub instance: String,
    /// The provider's display name (also what the region tables use).
    pub provider_name: String,
    pub region_code: String,
}

impl OnboardState {
    /// Empty, but for sensible defaults: port 22, root, and key login with
    /// Crow's key (or the first one) when there's a key, else a one-time
    /// password that installs Crow's key.
    pub fn new(enrolled_keys: &[crate::vault::SshKeyRecord]) -> Self {
        let key = enrolled_keys.iter().find(|k| k.name == "crow").or(enrolled_keys.first()).map(|k| k.id.clone());
        Self {
            step: OnboardStep::Connect,
            max_reached_step: OnboardStep::Connect,
            host: String::new(),
            port: "22".into(),
            user: "root".into(),
            auth_method: if key.is_some() { "publickey".into() } else { "password".into() },
            selected_key_id: key,
            jump_host_id: None,
            password: Default::default(),
            probe_result: None,
            probe_logs: Vec::new(),
            host_key_accepted: false,
            is_probing: false,
            label: String::new(),
            env: String::new(),
            role: String::new(),
            group: String::new(),
            tags: String::new(),
            make_bastion: false,
            facts: DetectedFacts::default(),
            provider: None,
            focus: OnboardFieldFocus::Host,
            error_message: None,
        }
    }

    /// The facts were read: a login worked (only a login reads the kernel).
    pub fn logged_in(&self) -> bool {
        self.facts.kernel != "—"
    }

    pub fn endpoint(&self) -> String {
        let port = if self.port.trim().is_empty() { "22" } else { self.port.trim() };
        format!("{}@{}:{port}", self.user.trim(), self.host.trim())
    }
}

const BASTION_BLUE: Rgba = Rgba { r: 0.376, g: 0.647, b: 0.98, a: 1.0 };

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn label(text: &'static str) -> Div {
    mono(10.0, TEXT_DIM).font_weight(FontWeight::BOLD).child(text)
}

fn hint(text: impl Into<SharedString>) -> Div {
    mono(10.0, TEXT_FAINT).line_height(px(14.0)).child(text.into())
}

/// A wizard text field: gpui-component's input, styled like the app's other
/// inputs (sharp corners, mono font).
fn field(inputs: Option<&OnboardInputs>, which: OnboardFieldFocus) -> Div {
    div().w_full().children(inputs.and_then(|i| i.get(which)).map(|state| Input::new(state).font_family(FONT_MONO).text_size(px(12.0)).bg(BG_APP).rounded(px(2.0))))
}

fn labeled(text: &'static str, body: impl IntoElement) -> Div {
    div().flex().flex_col().gap(px(5.0)).child(label(text)).child(body)
}

/// One choice in a row of them: outlined when picked.
fn chip(id: impl Into<SharedString>, text: impl Into<SharedString>, selected: bool, accent: Rgba) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(if selected { accent } else { BORDER_DEFAULT })
        .bg(if selected { BG_CONTROL_ALT } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if selected { TEXT_PRIMARY } else { TEXT_DIM })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .child(text.into())
}

/// A login method: a title and what it means, as a selectable card.
fn method_card(id: &'static str, title: &'static str, about: &'static str, selected: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .p(px(10.0))
        .border_1()
        .border_color(if selected { OK } else { BORDER_DEFAULT })
        .bg(if selected { OK.opacity(0.06) } else { BG_CONTROL })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(div().flex().items_center().gap(px(6.0)).child(div().size(px(8.0)).rounded_full().border_1().border_color(if selected { OK } else { TEXT_FAINT }).when(selected, |d| d.bg(OK))).child(mono(11.0, if selected { TEXT_PRIMARY } else { TEXT_SECONDARY }).font_weight(FontWeight::BOLD).child(title)))
        .child(mono(9.5, TEXT_DIM).line_height(px(13.0)).child(about))
}

fn button(id: &'static str, text: impl Into<SharedString>, primary: bool, enabled: bool) -> Stateful<Div> {
    let color = if !enabled { TEXT_GHOST } else if primary { BG_WINDOW } else { TEXT_SECONDARY };
    div()
        .id(id)
        .flex()
        .items_center()
        .h(px(30.0))
        .px(px(14.0))
        .border_1()
        .border_color(if primary && enabled { OK } else { BORDER_DEFAULT })
        .bg(if primary && enabled { OK } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(11.0))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .when(enabled, |d| d.cursor_pointer().hover(move |s| if primary { s.bg(hex_rgb(0x32b55e)) } else { s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY) }))
        .child(text.into())
}

pub fn onboard_view(app: Entity<CrowApp>, inputs: Option<&OnboardInputs>, fleet: &FleetState, state: &OnboardState, keys: &KeysState, local_lab: &LocalLabState) -> impl IntoElement {
    let (app_cancel, app_back, app_next) = (app.clone(), app.clone(), app.clone());
    let (primary_text, primary_enabled) = match state.step {
        OnboardStep::Connect => (t!("onboard-check-connection").to_string(), true),
        OnboardStep::Verify => (t!("onboard-next-name-it").to_string(), state.host_key_accepted && !state.is_probing),
        OnboardStep::Name => (t!("onboard-add-server").to_string(), true),
    };
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // Title and steps
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(18.0))
                .h(px(52.0))
                .px(px(20.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(mono(15.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(t!("onboard-add-server-2")))
                .child(div().flex().items_center().gap(px(4.0)).children(OnboardStep::ALL.iter().map(|step| {
                    let (active, done) = (state.step == *step, step.num() < state.max_reached_step.num() || (step.num() <= state.max_reached_step.num() && state.step != *step));
                    let reachable = step.num() <= state.max_reached_step.num();
                    let app = app.clone();
                    let step = *step;
                    div()
                        .id(ElementId::NamedInteger("onboard-step".into(), step.num() as u64))
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(10.0))
                        .py(px(4.0))
                        .border_b_2()
                        .border_color(if active { OK } else { hex_rgba(0, 0.0) })
                        .when(reachable && !active, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)))
                        .on_click(move |_ev, _window, cx| {
                            if reachable {
                                app.update(cx, |this, cx| this.onboard_set_step(step, cx));
                            }
                        })
                        .child(mono(10.0, if active { OK } else if done { OK.opacity(0.7) } else { TEXT_FAINT }).child(if done && !active { "✓".to_string() } else { step.num().to_string() }))
                        .child(mono(11.0, if active { TEXT_PRIMARY } else if reachable { TEXT_SECONDARY } else { TEXT_FAINT }).font_weight(if active { FontWeight::BOLD } else { FontWeight::NORMAL }).child(step.label()))
                })))
                .child(div().flex_1())
                .child(button("btn-onboard-cancel", t!("onboard-cancel-esc"), false, true).on_click(move |_ev, _window, cx| app_cancel.update(cx, |this, cx| this.set_screen(crate::app::Screen::Fleet, cx)))),
        )
        // The step
        .child(
            div().id("onboard-scroll").flex_1().min_h(px(0.0)).overflow_y_scrollbar().child(
                div()
                    .max_w(px(820.0))
                    .px(px(28.0))
                    .py(px(22.0))
                    .flex()
                    .flex_col()
                    .gap(px(18.0))
                    .child(match state.step {
                        OnboardStep::Connect => step_connect(app.clone(), inputs, fleet, state, keys, local_lab).into_any_element(),
                        OnboardStep::Verify => step_verify(app.clone(), state, fleet).into_any_element(),
                        OnboardStep::Name => step_name(app.clone(), inputs, fleet, state).into_any_element(),
                    })
                    .children(state.error_message.clone().map(|e| div().p(px(10.0)).bg(CRIT.opacity(0.08)).border_1().border_color(CRIT.opacity(0.5)).child(mono(11.0, CRIT).line_height(px(16.0)).child(e)))),
            ),
        )
        // Back and forward
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .h(px(56.0))
                .px(px(20.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .when(state.step != OnboardStep::Connect, |d| d.child(button("btn-onboard-back", t!("onboard-back"), false, true).on_click(move |_ev, _window, cx| app_back.update(cx, |this, cx| this.onboard_prev_step(cx)))))
                .child(div().flex_1())
                .child(mono(10.0, TEXT_FAINT).child(state.endpoint()).when(state.host.trim().is_empty(), |d| d.invisible()))
                .child(button("btn-onboard-next", primary_text, true, primary_enabled).on_click(move |_ev, _window, cx| {
                    if primary_enabled {
                        app_next.update(cx, |this, cx| this.onboard_next_step(cx));
                    }
                })),
        )
}

// ---------------------------------------------------------------------------
// 1. Connect
// ---------------------------------------------------------------------------

fn step_connect(app: Entity<CrowApp>, inputs: Option<&OnboardInputs>, fleet: &FleetState, state: &OnboardState, keys: &KeysState, local_lab: &LocalLabState) -> impl IntoElement {
    let bastions = crate::app::bastions::choices(None, &fleet.servers);
    let method = state.auth_method.as_str();
    let (app_key, app_agent, app_pw, app_direct) = (app.clone(), app.clone(), app.clone(), app.clone());
    div()
        .flex()
        .flex_col()
        .gap(px(18.0))
        .child(div().flex().flex_col().gap(px(4.0)).child(mono(14.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(t!("onboard-where-is-it-and-how"))).child(hint(t!("onboard-crow-connects-over-ssh-and"))))
        // Lab containers on this machine, one click away.
        .when(!local_lab.nodes.is_empty(), |d| {
            d.child(labeled(
                t!("onboard-or-a-lab-container-on"),
                div().flex().flex_wrap().gap(px(6.0)).children(local_lab.nodes.iter().enumerate().map(|(i, node)| {
                    let (app, name, port, distro) = (app.clone(), node.name.clone(), node.ssh_port.unwrap_or(2222).to_string(), node.distro_display());
                    chip(format!("onboard-lab-{i}"), format!("{} · {}", node.name, distro), false, OK).on_click(move |_ev, _window, cx| {
                        let (name, port, distro) = (name.clone(), port.clone(), distro.clone());
                        app.update(cx, |this, cx| this.onboard_select_local_lab_node(&name, &port, &distro, cx))
                    })
                })),
            ))
        })
        .child(
            div()
                .flex()
                .gap(px(10.0))
                .child(div().flex_1().min_w(px(0.0)).child(labeled(t!("onboard-address"), field(inputs, OnboardFieldFocus::Host))))
                .child(div().w(px(90.0)).flex_none().child(labeled(t!("onboard-port"), field(inputs, OnboardFieldFocus::Port))))
                .child(div().w(px(200.0)).flex_none().child(labeled(t!("onboard-user"), field(inputs, OnboardFieldFocus::User)))),
        )
        // How Crow gets there.
        .child(labeled(
            t!("onboard-reach-it"),
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .child(chip("onboard-route-direct", t!("onboard-directly"), state.jump_host_id.is_none(), OK).on_click(move |_ev, _window, cx| {
                            app_direct.update(cx, |this, cx| {
                                this.onboard_state.jump_host_id = None;
                                cx.notify();
                            })
                        }))
                        .children(bastions.iter().map(|b| {
                            let (app, id) = (app.clone(), b.id.clone());
                            let route = crate::app::bastions::route(&b.id, &fleet.servers);
                            chip(format!("onboard-route-{}", b.id), t!("onboard-via", route = route), state.jump_host_id.as_deref() == Some(b.id.as_str()), BASTION_BLUE).on_click(move |_ev, _window, cx| {
                                let id = id.clone();
                                app.update(cx, |this, cx| {
                                    this.onboard_state.jump_host_id = Some(id);
                                    this.onboard_state.probe_result = None;
                                    cx.notify();
                                })
                            })
                        })),
                )
                .child(hint(if bastions.is_empty() {
                    t!("onboard-only-reachable-through-another-server").to_string()
                } else if state.jump_host_id.is_some() {
                    t!("onboard-every-connection-goes-through-the").to_string()
                } else {
                    t!("onboard-pick-a-bastion-when-this").to_string()
                })),
        ))
        // How Crow logs in.
        .child(labeled(
            t!("onboard-log-in-with"),
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(method_card("onboard-auth-key", t!("onboard-a-key-in-crow"), t!("onboard-a-key-from-settings-keys"), method == "publickey").on_click(move |_ev, _window, cx| app_key.update(cx, |this, cx| this.onboard_set_auth("publickey", cx))))
                        .child(method_card("onboard-auth-password", t!("onboard-a-password-once"), t!("onboard-crow-logs-in-once-installs"), method == "password").on_click(move |_ev, _window, cx| app_pw.update(cx, |this, cx| this.onboard_set_auth("password", cx))))
                        .child(method_card("onboard-auth-agent", t!("onboard-my-ssh-agent"), t!("onboard-whatever-your-agent-and-ssh"), method == "agent").on_click(move |_ev, _window, cx| app_agent.update(cx, |this, cx| this.onboard_set_auth("agent", cx)))),
                )
                .when(method == "publickey", |d| {
                    d.child(if keys.enrolled.is_empty() {
                        hint(t!("onboard-no-keys-in-crow-yet")).into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(6.0))
                            .children(keys.enrolled.iter().map(|k| {
                                let (app, id) = (app.clone(), k.id.clone());
                                chip(format!("onboard-key-{}", k.id), format!("{} · {}", k.name, k.algorithm), state.selected_key_id.as_deref() == Some(k.id.as_str()), OK).on_click(move |_ev, _window, cx| {
                                    let id = id.clone();
                                    app.update(cx, |this, cx| {
                                        this.onboard_state.selected_key_id = Some(id);
                                        cx.notify();
                                    })
                                })
                            }))
                            .into_any_element()
                    })
                })
                .when(method == "password", |d| d.child(div().max_w(px(360.0)).child(field(inputs, OnboardFieldFocus::Password))).child(hint(t!("onboard-used-for-one-login-only"))))
                .when(method == "agent", |d| d.child(hint(t!("onboard-crow-wont-know-which-key")))),
        ))
}

// ---------------------------------------------------------------------------
// 2. Verify
// ---------------------------------------------------------------------------

fn step_verify(app: Entity<CrowApp>, state: &OnboardState, fleet: &FleetState) -> impl IntoElement {
    let via = state.jump_host_id.as_deref().map(|b| crate::app::bastions::route(b, &fleet.servers));
    let result = state.probe_result.as_ref();
    let (app_recheck, app_accept) = (app.clone(), app.clone());
    let headline: (Rgba, String) = match result {
        _ if state.is_probing && result.is_none() => (TEXT_DIM, t!("onboard-checking").into()),
        None => (TEXT_DIM, t!("onboard-not-checked-yet").into()),
        Some(r) if !r.is_reachable => (CRIT, t!("onboard-couldnt-reach-it", r = r.error.clone().unwrap_or_else(|| t!("onboard-no-answer").into()))),
        Some(_) if state.is_probing => (TEXT_DIM, t!("onboard-logging-in-and-reading-the").into()),
        Some(_) if state.logged_in() => (OK, t!("onboard-reached-host-key-trusted-logged").into()),
        Some(_) if state.host_key_accepted => (WARN, t!("onboard-the-host-key-is-trusted").into()),
        Some(r) if r.host_key_mismatch => (CRIT, t!("onboard-the-host-key-changed-since").into()),
        Some(_) => (WARN, t!("onboard-reached-check-the-host-key").into()),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(12.0))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(mono(14.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(state.endpoint()))
                        .children(via.map(|v| mono(10.5, BASTION_BLUE).child(t!("onboard-through", v = v))))
                        .child(mono(11.5, headline.0).line_height(px(16.0)).child(headline.1)),
                )
                .child(button("btn-onboard-recheck", if state.is_probing { t!("onboard-checking-2") } else { t!("onboard-check-again") }, false, !state.is_probing).on_click(move |_ev, _window, cx| app_recheck.update(cx, |this, cx| this.onboard_run_probe(cx)))),
        )
        // The host key: what the server presented, and whether it's trusted.
        .children(result.filter(|r| !r.scanned_keys.is_empty()).map(|r| {
            let (border, title) = if state.host_key_accepted {
                (OK, t!("onboard-host-key-trusted"))
            } else if r.host_key_mismatch {
                (CRIT, t!("onboard-host-key-changed-not-trusted"))
            } else {
                (WARN, t!("onboard-host-key-new-to-this"))
            };
            let (hash, kind) = match r.host_key_fingerprint.rsplit_once(" (") {
                Some((h, k)) => (h.to_string(), k.trim_end_matches(')').to_string()),
                None => (r.host_key_fingerprint.clone(), String::new()),
            };
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(12.0))
                .border_1()
                .border_color(border.opacity(0.6))
                .bg(border.opacity(0.05))
                .child(mono(10.0, border).font_weight(FontWeight::BOLD).child(title))
                .child(mono(12.5, TEXT_PRIMARY).child(hash))
                .child(mono(9.5, TEXT_FAINT).child(kind))
                .when(!state.host_key_accepted && !r.host_key_mismatch, |d| {
                    d.child(hint(t!("onboard-compare-it-with-the-servers")))
                        .child(div().flex().child(button("btn-accept-host-key", t!("onboard-it-matches-trust-it"), true, !state.is_probing).on_click(move |_ev, _window, cx| app_accept.update(cx, |this, cx| this.onboard_accept_host_key(cx)))))
                })
                .when(r.host_key_mismatch, |d| d.child(hint(t!("onboard-possibly-a-reinstalled-server-possibly"))))
        }))
        // What a login read.
        .when(state.logged_in(), |d| d.child(facts_grid(&state.facts)))
        // Everything that happened, in order.
        .when(!state.probe_logs.is_empty(), |d| {
            d.child(labeled(
                t!("onboard-log"),
                div().flex().flex_col().gap(px(3.0)).p(px(10.0)).bg(BG_PANEL).border_1().border_color(BORDER_PANEL).children(state.probe_logs.iter().map(|l| {
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(mono(10.0, TEXT_FAINT).flex_none().child(l.timestamp.clone()))
                        .child(mono(10.0, l.color).flex_none().w(px(12.0)).child(l.glyph.clone()))
                        .child(mono(10.5, TEXT_SECONDARY).flex_1().min_w(px(0.0)).line_height(px(15.0)).child(l.message.clone()))
                        .when(!l.note.is_empty(), |d| d.child(mono(9.5, TEXT_FAINT).flex_none().child(l.note.clone())))
                })),
            ))
        })
}

fn facts_grid(f: &DetectedFacts) -> impl IntoElement {
    let family = classify_distro_family(&f.distro);
    let rows: [(&str, String); 9] = [
        (t!("onboard-distro"), f.distro.clone()),
        (t!("onboard-kernel"), f.kernel.clone()),
        (t!("onboard-arch"), f.arch.clone()),
        (t!("onboard-memory"), f.memory.clone()),
        (t!("onboard-disk"), f.disk.clone()),
        (t!("onboard-init"), f.init.clone()),
        (t!("onboard-firewall"), f.firewall.clone()),
        (t!("onboard-open-ports"), f.open_ports.clone()),
        (t!("onboard-time-sync"), f.time_sync.clone()),
    ];
    labeled(
        t!("onboard-what-crow-read"),
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .bg(BG_PANEL)
            .border_1()
            .border_color(BORDER_PANEL)
            .child(div().flex().flex_wrap().gap_y(px(8.0)).children(rows.into_iter().map(|(k, v)| {
                div().w(px(250.0)).flex().gap(px(8.0)).child(mono(9.5, TEXT_FAINT).w(px(78.0)).flex_none().child(k)).child(mono(10.5, TEXT_SECONDARY).flex_1().min_w(px(0.0)).overflow_hidden().text_ellipsis().whitespace_nowrap().child(v))
            })))
            .when(!f.schema_packs.is_empty(), |d| {
                d.child(div().flex().flex_wrap().gap(px(4.0)).child(mono(9.5, TEXT_FAINT).mr(px(4.0)).child(t!("onboard-config-tools"))).children(f.schema_packs.iter().map(|(name, fg, bg)| div().px(px(5.0)).py(px(1.0)).bg(*bg).font_family(FONT_MONO).text_size(px(9.5)).text_color(*fg).child(name.clone()))))
            })
            .when(!family.is_supported(), |d| d.child(mono(10.0, WARN).line_height(px(14.0)).child(t!("onboard-crow-knows-debian-and-red")))),
    )
}

// ---------------------------------------------------------------------------
// 3. Name
// ---------------------------------------------------------------------------

fn step_name(app: Entity<CrowApp>, inputs: Option<&OnboardInputs>, fleet: &FleetState, state: &OnboardState) -> impl IntoElement {
    let app_bastion = app.clone();
    let envs = ["PROD", "STAGE", "DEV", "LAB"];
    let env_color = |e: &str| match e {
        "PROD" => CRIT,
        "STAGE" => WARN,
        _ => OK,
    };
    let via = state.jump_host_id.as_deref().map(|b| crate::app::bastions::route(b, &fleet.servers));
    let login = match state.auth_method.as_str() {
        "publickey" => "key".to_string(),
        "agent" => t!("onboard-your-ssh-agent").to_string(),
        _ => t!("onboard-password-once").to_string(),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(18.0))
        .child(div().flex().flex_col().gap(px(4.0)).child(mono(14.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(t!("onboard-how-should-it-show-in"))).child(hint(t!("onboard-all-of-this-can-be"))))
        .child(div().max_w(px(420.0)).child(labeled(t!("onboard-name-2"), field(inputs, OnboardFieldFocus::Label))))
        .child(labeled(
            t!("onboard-environment"),
            div().flex().gap(px(6.0)).child({
                let app = app.clone();
                chip("onboard-env-none", "none", state.env.is_empty(), TEXT_DIM).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.onboard_set_env("", cx)))
            })
            .children(envs.iter().map(|e| {
                let app = app.clone();
                let e = *e;
                chip(format!("onboard-env-{e}"), e, state.env == e, env_color(e)).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.onboard_set_env(e, cx)))
            })),
        ))
        .child(labeled(
            t!("onboard-group"),
            div().flex().flex_wrap().gap(px(6.0)).child({
                let app = app.clone();
                chip("onboard-group-none", "none", state.group.is_empty(), TEXT_DIM).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.onboard_set_group("", cx)))
            })
            .children(fleet.groups.iter().map(|g| {
                let (app, name) = (app.clone(), g.clone());
                chip(format!("onboard-group-{g}"), g.clone(), state.group == *g, OK).on_click(move |_ev, _window, cx| {
                    let name = name.clone();
                    app.update(cx, |this, cx| this.onboard_set_group(&name, cx))
                })
            })),
        ))
        .child(div().max_w(px(420.0)).child(labeled(t!("onboard-tags"), field(inputs, OnboardFieldFocus::Tags))))
        // Bastion (ERR-152).
        .child(
            div()
                .id("onboard-bastion-switch")
                .flex()
                .items_center()
                .gap(px(10.0))
                .cursor_pointer()
                .on_click(move |_ev, _window, cx| {
                    app_bastion.update(cx, |this, cx| {
                        this.onboard_state.make_bastion = !this.onboard_state.make_bastion;
                        cx.notify();
                    })
                })
                .child(
                    div()
                        .w(px(30.0))
                        .h(px(16.0))
                        .rounded_full()
                        .p(px(2.0))
                        .flex()
                        .when(state.make_bastion, |d| d.justify_end())
                        .bg(if state.make_bastion { BASTION_BLUE.opacity(0.25) } else { BG_CONTROL })
                        .border_1()
                        .border_color(if state.make_bastion { BASTION_BLUE } else { BORDER_STRONG })
                        .child(div().size(px(10.0)).rounded_full().bg(if state.make_bastion { BASTION_BLUE } else { TEXT_DIM })),
                )
                .child(div().flex().flex_col().gap(px(2.0)).child(mono(11.0, if state.make_bastion { TEXT_PRIMARY } else { TEXT_SECONDARY }).font_weight(FontWeight::BOLD).child(t!("onboard-bastion"))).child(hint(t!("onboard-other-servers-can-be-added")))),
        )
        // What's about to be added.
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .p(px(12.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .child(mono(11.0, TEXT_PRIMARY).child(format!("{} · {}", if state.label.trim().is_empty() { state.host.trim() } else { state.label.trim() }, state.endpoint())))
                .child(mono(10.0, TEXT_DIM).child(t!("onboard-logs-in-with", via = via.map(|v| t!("onboard-through-2", v = v)).unwrap_or_default(), login = login, state = if state.env.is_empty() { String::new() } else { format!(" · {}", state.env) }, state2 = if state.group.is_empty() { String::new() } else { t!("onboard-group-2", group = state.group) })))
                .when(!state.logged_in(), |d| d.child(mono(10.0, WARN).child(t!("onboard-crow-couldnt-log-in-yet")))),
        )
}
