//! Add Server (ERR-153): three steps. Connect (where it is, how Crow gets
//! there and logs in), Verify (the real host key, then a login that reads
//! the server's facts), Name (how it shows in the fleet). Nothing is shown
//! before it's read.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::onboard::OnboardInputs;
use crate::app::providers::{ProvisionInputs, ProvisionState};
use crate::app::{CrowApp, Screen, SettingsSection};
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::os_detect::classify_distro_family;
use crate::theme::*;
use crate::vault::ProviderAccount;
use crate::views::fleet::lab_state::LocalLabState;
use crate::views::fleet::FleetState;
use crate::views::settings::keys_state::KeysState;

pub mod probe;
#[allow(unused_imports)]
pub use probe::{gather_facts, log as probe_log, probe_host, trust_host_keys, DetectedFacts, ProbeLog, ProbeResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OnboardMode {
    #[default]
    Ssh,
    CloudProvision,
}

#[derive(Clone, Debug)]
pub struct BootWaitLog {
    pub timestamp: String,
    pub glyph: String,
    pub color: Rgba,
    pub message: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct BootWaitState {
    pub instance_id: String,
    pub label: String,
    pub provider_name: String,
    pub host: String,
    pub port: u16,
    pub selected_key_id: String,
    pub account_id: String,
    pub region_code: String,
    pub tags: String,
    pub logs: Vec<BootWaitLog>,
    pub attempt: usize,
    pub pulse_tick: usize,
    pub done: bool,
}

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

    pub fn icon(&self) -> TablerIcon {
        match self {
            Self::Connect => TablerIcon::Terminal2,
            Self::Verify => TablerIcon::ShieldCheck,
            Self::Name => TablerIcon::Server,
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
    pub mode: OnboardMode,
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
    /// Live cloud VM boot wait and polling state (ERR-223).
    pub boot_wait: Option<BootWaitState>,
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
            mode: OnboardMode::Ssh,
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
            boot_wait: None,
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

/// One choice in a row of them: outlined with Obsidian Edge control selection tokens when picked.
fn chip(id: impl Into<SharedString>, text: impl Into<SharedString>, selected: bool, accent: Rgba) -> Stateful<Div> {
    let sel_border = if accent == OK || accent == TEXT_DIM { BORDER_CONTROL_SEL } else { accent };
    let dot_color = if accent == OK || accent == TEXT_DIM { TEXT_PRIMARY } else { accent };
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(if selected { sel_border } else { BORDER_DEFAULT })
        .bg(if selected { BG_ROW_SELECTED } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if selected { TEXT_MAX } else { TEXT_SECONDARY })
        .cursor_pointer()
        .hover(|s| s.border_color(BORDER_STRONG).bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .when(selected, |d| d.child(div().size(px(5.0)).rounded_full().bg(dot_color)))
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
        .border_color(if selected { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
        .bg(if selected { BG_ROW_SELECTED } else { BG_CONTROL })
        .cursor_pointer()
        .hover(|s| s.border_color(BORDER_STRONG).bg(BG_ROW_HOVER))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    div()
                        .size(px(8.0))
                        .rounded_full()
                        .border_1()
                        .border_color(if selected { TEXT_MAX } else { TEXT_FAINT })
                        .when(selected, |d| d.bg(TEXT_MAX)),
                )
                .child(mono(11.0, if selected { TEXT_MAX } else { TEXT_SECONDARY }).font_weight(FontWeight::BOLD).child(title)),
        )
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

pub fn onboard_view(
    app: Entity<CrowApp>,
    inputs: Option<&OnboardInputs>,
    fleet: &FleetState,
    state: &OnboardState,
    keys: &KeysState,
    local_lab: &LocalLabState,
    provision: &ProvisionState,
    provision_inputs: Option<&ProvisionInputs>,
    provider_accounts: &[ProviderAccount],
) -> impl IntoElement {
    let (app_cancel, app_back, app_next, app_cloud, app_launch) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone());
    let is_cloud = state.mode == OnboardMode::CloudProvision;

    let prov_capable_count = provider_accounts
        .iter()
        .filter(|a| {
            crate::providers::factory(&a.plugin)
                .is_some_and(|f| (f.manifest)().has_capability(crow_provider_core::capabilities::INSTANCES_CREATE))
        })
        .count();

    let is_boot_waiting = state.boot_wait.is_some();
    let (sec_title, sec_subtitle): (String, String) = if is_boot_waiting {
        (
            t!("onboard-boot-title").to_string(),
            t!("onboard-boot-subtitle").to_string(),
        )
    } else if is_cloud {
        (
            t!("fleet-provision-modal-title").to_string(),
            t!("onboard-cloud-subtitle").to_string(),
        )
    } else {
        match state.step {
            OnboardStep::Connect => (
                format!("1. {}", t!("onboard-connect").to_uppercase()),
                t!("onboard-crow-connects-over-ssh-and").to_string(),
            ),
            OnboardStep::Verify => (
                format!("2. {}", t!("onboard-verify").to_uppercase()),
                t!("onboard-compare-it-with-the-servers").to_string(),
            ),
            OnboardStep::Name => (
                format!("3. {}", t!("onboard-name").to_uppercase()),
                t!("onboard-all-of-this-can-be").to_string(),
            ),
        }
    };

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
        // 1. Top Screen Header (matches Settings → Personalisation)
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
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .items_baseline()
                        .gap(px(10.0))
                        .child(
                            div()
                                .flex_none()
                                .font_family(FONT_MONO)
                                .text_size(px(16.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MAX)
                                .child(t!("onboard-add-server-2")),
                        )
                        .child(
                            div()
                                .min_w(px(0.0))
                                .truncate()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .text_color(TEXT_DIM)
                                .child(t!("onboard-header-subtitle")),
                        ),
                )
                .child(
                    button("btn-onboard-cancel", t!("onboard-cancel-esc"), false, true)
                        .on_click(move |_ev, _window, cx| app_cancel.update(cx, |this, cx| this.set_screen(Screen::Fleet, cx))),
                ),
        )
        // 2. 3-Column Workbench: Left Nav Rail (220px) | Center Column (flex-1) | Right Summary Rail (340px)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Left Navigation Rail
                .child(
                    div()
                        .w(px(220.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .pt(px(8.0))
                        // Group 1: Existing Server (SSH)
                        .child(
                            div()
                                .px(px(12.0))
                                .py(px(6.0))
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_FAINT)
                                .child(t!("onboard-nav-existing-ssh")),
                        )
                        .children(OnboardStep::ALL.iter().map(|step| {
                            let active = !is_cloud && state.step == *step;
                            let done = step.num() < state.max_reached_step.num()
                                || (step.num() <= state.max_reached_step.num() && state.step != *step);
                            let reachable = step.num() <= state.max_reached_step.num();
                            let app = app.clone();
                            let step = *step;
                            div()
                                .id(ElementId::NamedInteger("onboard-step".into(), step.num() as u64))
                                .relative()
                                .h(px(34.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(12.0))
                                .bg(if active { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                                .children(active.then(|| left_indicator(TEXT_PRIMARY)))
                                .when(reachable || is_cloud, |d| {
                                    d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER))
                                })
                                .on_click(move |_ev, _window, cx| {
                                    if reachable {
                                        app.update(cx, |this, cx| this.onboard_set_step(step, cx));
                                    } else {
                                        app.update(cx, |this, cx| this.onboard_set_mode(OnboardMode::Ssh, cx));
                                    }
                                })
                                .child(
                                    div()
                                        .w(px(16.0))
                                        .h(px(16.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            tabler_icon(step.icon())
                                                .size(px(14.0))
                                                .text_color(if active {
                                                    TEXT_PRIMARY
                                                } else if done {
                                                    OK
                                                } else {
                                                    TEXT_DIMMER
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .truncate()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(if active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                        .text_color(if active {
                                            TEXT_PRIMARY
                                        } else if reachable {
                                            TEXT_MUTED
                                        } else {
                                            TEXT_FAINT
                                        })
                                        .child(format!("{}. {}", step.num(), step.label())),
                                )
                                .child(
                                    mono(10.0, if done { OK } else { TEXT_FAINT })
                                        .child(if done { "✓".to_string() } else { String::new() }),
                                )
                        }))
                        // Group 2: Cloud Provisioning
                        .child(
                            div()
                                .mt(px(14.0))
                                .px(px(12.0))
                                .py(px(6.0))
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_FAINT)
                                .child(t!("onboard-nav-cloud-provision")),
                        )
                        .child(
                            div()
                                .id("onboard-nav-cloud-vm")
                                .relative()
                                .h(px(34.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(12.0))
                                .bg(if is_cloud { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                                .children(is_cloud.then(|| left_indicator(TEXT_PRIMARY)))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_cloud.update(cx, |this, cx| this.onboard_set_mode(OnboardMode::CloudProvision, cx));
                                })
                                .child(
                                    div()
                                        .w(px(16.0))
                                        .h(px(16.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            tabler_icon(TablerIcon::Cloud)
                                                .size(px(14.0))
                                                .text_color(if is_cloud { TEXT_PRIMARY } else { TEXT_DIMMER }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .truncate()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(if is_cloud { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_cloud { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .child(t!("onboard-nav-deploy-vm")),
                                )
                                .child(
                                    mono(10.0, if prov_capable_count > 0 { OK } else { TEXT_FAINT })
                                        .child(if prov_capable_count > 0 { format!("{prov_capable_count}") } else { String::new() }),
                                ),
                        ),
                )
                // Center Section: Subheader + Scrollable Form + Fixed Bottom Bar
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .bg(BG_APP)
                        // Section Subheader Bar (h=34, BG_PANEL)
                        .child(
                            div()
                                .h(px(34.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(14.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .flex_none()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(sec_title),
                                )
                                .children(is_cloud.then(|| {
                                    crate::components::button::badge(OK)
                                        .text_size(px(9.0))
                                        .child(t!("fleet-provision-ssh-vault-seeded"))
                                }))
                                .child(
                                    div()
                                        .min_w(px(0.0))
                                        .truncate()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_DIM)
                                        .child(sec_subtitle),
                                ),
                        )
                        // Scrollable Form Content
                        .child(
                            div()
                                .id("onboard-scroll")
                                .flex_1()
                                .min_h(px(0.0))
                                .overflow_y_scrollbar()
                                .child(
                                    div()
                                        .max_w(px(760.0))
                                        .p(px(20.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(18.0))
                                        .child(if let Some(ref bw) = state.boot_wait {
                                            render_boot_wait_center(app.clone(), bw).into_any_element()
                                        } else if is_cloud {
                                            render_cloud_provision_center(
                                                app.clone(),
                                                provision,
                                                provision_inputs,
                                                provider_accounts,
                                                keys,
                                                prov_capable_count,
                                            )
                                            .into_any_element()
                                        } else {
                                            match state.step {
                                                OnboardStep::Connect => step_connect(app.clone(), inputs, fleet, state, keys, local_lab).into_any_element(),
                                                OnboardStep::Verify => step_verify(app.clone(), state, fleet).into_any_element(),
                                                OnboardStep::Name => step_name(app.clone(), inputs, fleet, state).into_any_element(),
                                            }
                                        })
                                        .children((!is_cloud && !is_boot_waiting).then(|| state.error_message.clone()).flatten().map(|e| {
                                            div()
                                                .p(px(10.0))
                                                .bg(CRIT.opacity(0.08))
                                                .border_1()
                                                .border_color(CRIT.opacity(0.5))
                                                .child(mono(11.0, CRIT).line_height(px(16.0)).child(e))
                                        })),
                                ),
                        )
                        // Fixed Bottom Action Bar (anchored to Center Column!)
                        .child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .h(px(48.0))
                                .px(px(16.0))
                                .bg(BG_PANEL)
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .when(!is_cloud && !is_boot_waiting && state.step != OnboardStep::Connect, |d| {
                                    d.child(
                                        button("btn-onboard-back", t!("onboard-back"), false, true)
                                            .on_click(move |_ev, _window, cx| app_back.update(cx, |this, cx| this.onboard_prev_step(cx))),
                                    )
                                })
                                .child(div().flex_1())
                                .children(if is_boot_waiting {
                                    let app_skip = app.clone();
                                    vec![
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(mono(10.5, OK).child("●"))
                                            .child(mono(10.0, TEXT_SECONDARY).child(t!("onboard-boot-polling-live")))
                                            .into_any_element(),
                                        button("btn-onboard-boot-skip", t!("onboard-boot-skip-manual"), false, true)
                                            .on_click(move |_ev, _window, cx| app_skip.update(cx, |this, cx| this.skip_provision_boot_wait(cx)))
                                            .into_any_element(),
                                    ]
                                } else if is_cloud {
                                    let can_launch = prov_capable_count > 0 && !provision.provisioning && !provision.loading_catalog;
                                    let launch_label = if provision.provisioning {
                                        t!("fleet-provision-launching").to_string()
                                    } else {
                                        t!("fleet-provision-launch-instance").to_string()
                                    };
                                    vec![
                                        button("btn-onboard-launch-vm", launch_label, true, can_launch)
                                            .on_click(move |_ev, _window, cx| {
                                                if can_launch {
                                                    app_launch.update(cx, |this, cx| this.submit_provision(cx));
                                                }
                                            })
                                            .into_any_element(),
                                    ]
                                } else {
                                    vec![
                                        mono(10.0, TEXT_FAINT)
                                            .child(state.endpoint())
                                            .when(state.host.trim().is_empty(), |d| d.invisible())
                                            .into_any_element(),
                                        button("btn-onboard-next", primary_text, true, primary_enabled)
                                            .on_click(move |_ev, _window, cx| {
                                                if primary_enabled {
                                                    app_next.update(cx, |this, cx| this.onboard_next_step(cx));
                                                }
                                            })
                                            .into_any_element(),
                                    ]
                                }),
                        ),
                )
                // Right Context & Summary Rail (340px)
                .child(
                    div()
                        .w(px(340.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
                        .child(render_onboard_right_rail(
                            state,
                            fleet,
                            keys,
                            provision,
                            provider_accounts,
                            is_cloud,
                        )),
                ),
        )
}

fn render_cloud_provision_center(
    app: Entity<CrowApp>,
    provision: &ProvisionState,
    provision_inputs: Option<&ProvisionInputs>,
    provider_accounts: &[ProviderAccount],
    keys: &KeysState,
    prov_capable_count: usize,
) -> impl IntoElement {
    if prov_capable_count == 0 {
        let (app_prov, app_plug) = (app.clone(), app);
        return div()
            .p(px(16.0))
            .bg(BG_PANEL)
            .border_1()
            .border_color(BORDER_PANEL)
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                mono(12.0, TEXT_PRIMARY)
                    .font_weight(FontWeight::BOLD)
                    .child(t!("fleet-provision-no-providers-configured")),
            )
            .child(
                mono(10.5, TEXT_DIM)
                    .line_height(px(15.0))
                    .child(t!("onboard-cloud-empty-desc")),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(10.0))
                    .pt(px(4.0))
                    .child(
                        button("btn-onboard-goto-providers", t!("onboard-cloud-configure-providers"), true, true)
                            .on_click(move |_ev, _window, cx| {
                                app_prov.update(cx, |this, cx| {
                                    this.set_settings_section(SettingsSection::Providers, cx);
                                    this.set_screen(Screen::Settings, cx);
                                });
                            }),
                    )
                    .child(
                        button("btn-onboard-goto-plugins", t!("onboard-cloud-enable-plugins"), false, true)
                            .on_click(move |_ev, _window, cx| {
                                app_plug.update(cx, |this, cx| {
                                    this.set_settings_section(SettingsSection::Plugins, cx);
                                    this.set_screen(Screen::Settings, cx);
                                });
                            }),
                    ),
            )
            .into_any_element();
    }

    div()
        .p(px(16.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .child(crate::views::fleet::provision::render_provision_form_body(
            provision,
            provision_inputs,
            provider_accounts,
            &keys.enrolled,
            app,
        ))
        .into_any_element()
}

fn render_boot_wait_center(_app: Entity<CrowApp>, bw: &BootWaitState) -> impl IntoElement {
    let num_segments = 36;
    let head = (bw.pulse_tick % num_segments) as i32;

    div()
        .p(px(16.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .gap(px(16.0))
        // Header info banner
        .child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .gap(px(12.0))
                .child(
                    div()
                        .flex()
                        .gap(px(10.0))
                        .child(
                            div()
                                .size(px(32.0))
                                .flex_none()
                                .bg(BG_APP)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(tabler_icon(TablerIcon::Cloud).size(px(18.0)).text_color(OK)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            mono(13.0, TEXT_PRIMARY)
                                                .font_weight(FontWeight::BOLD)
                                                .child(format!("{} · {}", bw.label, bw.provider_name)),
                                        )
                                        .child(
                                            crate::components::button::badge(OK)
                                                .text_size(px(9.0))
                                                .child(bw.region_code.clone()),
                                        ),
                                )
                                .child(mono(10.5, TEXT_DIM).child(t!("onboard-boot-waiting-desc"))),
                        ),
                )
                .child(
                    mono(11.0, TEXT_SECONDARY)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("{}:22", bw.host)),
                ),
        )
        // Indeterminate Wave Progress Bar
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
                            mono(10.0, TEXT_DIM)
                                .font_weight(FontWeight::BOLD)
                                .child(t!("onboard-boot-polling-title", interval = 4)),
                        )
                        .child(
                            mono(9.5, if bw.done { OK } else { TEXT_FAINT })
                                .child(if bw.done {
                                    "SSH Ready".to_string()
                                } else {
                                    format!("Poll #{}", bw.attempt)
                                }),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(8.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .p(px(1.0))
                        .flex()
                        .gap(px(2.0))
                        .children((0..num_segments).map(|i| {
                            let dist = (i as i32 - head).abs().min(num_segments as i32 - (i as i32 - head).abs());
                            let op = match dist {
                                0 => 1.0,
                                1 => 0.70,
                                2 => 0.40,
                                3 => 0.15,
                                _ => 0.03,
                            };
                            div()
                                .flex_1()
                                .h_full()
                                .bg(if bw.done { OK } else { OK.opacity(op) })
                        })),
                ),
        )
        // Live Terminal Log Console
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    mono(10.0, TEXT_DIM)
                        .font_weight(FontWeight::BOLD)
                        .child(t!("onboard-boot-terminal-log")),
                )
                .child(
                    div()
                        .id("onboard-boot-console")
                        .w_full()
                        .h(px(240.0))
                        .overflow_y_scrollbar()
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .p(px(10.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .children(bw.logs.iter().map(|log| {
                            div()
                                .flex()
                                .items_start()
                                .gap(px(8.0))
                                .child(
                                    mono(9.5, TEXT_FAINT)
                                        .flex_none()
                                        .child(format!("[{}]", log.timestamp)),
                                )
                                .child(
                                    mono(10.0, log.color)
                                        .flex_none()
                                        .w(px(16.0))
                                        .child(log.glyph.clone()),
                                )
                                .child(
                                    mono(10.5, log.color)
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .line_height(px(15.0))
                                        .child(log.message.clone()),
                                )
                        })),
                ),
        )
}

fn summary_row(k: impl Into<SharedString>, v: impl Into<SharedString>, v_color: Rgba) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(10.0))
        .py(px(6.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .child(mono(9.5, TEXT_FAINT).font_weight(FontWeight::BOLD).child(k.into()))
        .child(mono(10.5, v_color).truncate().child(v.into()))
}

fn render_onboard_right_rail(
    state: &OnboardState,
    fleet: &FleetState,
    keys: &KeysState,
    provision: &ProvisionState,
    provider_accounts: &[ProviderAccount],
    is_cloud: bool,
) -> impl IntoElement {
    let header_title = if state.boot_wait.is_some() {
        t!("onboard-boot-title")
    } else if is_cloud {
        t!("onboard-instance-spec")
    } else {
        t!("onboard-target-summary")
    };

    let body: AnyElement = if let Some(ref bw) = state.boot_wait {
        let key_name = keys
            .enrolled
            .iter()
            .find(|k| k.id == bw.selected_key_id)
            .map(|k| format!("{} · {}", k.name, k.algorithm))
            .unwrap_or_else(|| "key".to_string());
        div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                div()
                    .p(px(12.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_PANEL)
                    .flex()
                    .flex_col()
                    .child(summary_row(t!("onboard-summary-endpoint"), format!("{}:22", bw.host), TEXT_PRIMARY))
                    .child(summary_row(t!("fleet-provision-provider-account"), bw.provider_name.clone(), TEXT_PRIMARY))
                    .child(summary_row(t!("fleet-provision-instance-label"), bw.label.clone(), TEXT_PRIMARY))
                    .child(summary_row(t!("fleet-provision-region"), bw.region_code.clone(), TEXT_SECONDARY))
                    .child(summary_row(t!("fleet-provision-vault-ssh-key"), key_name, OK))
                    .child(summary_row(
                        "STATUS",
                        if bw.done { "SSH Ready".to_string() } else { format!("Polling #{}", bw.attempt) },
                        if bw.done { OK } else { WARN },
                    )),
            )
            .child(
                div()
                    .p(px(12.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_PANEL)
                    .child(
                        mono(10.0, TEXT_DIM)
                            .line_height(px(15.0))
                            .child(t!("onboard-summary-auto-enroll-note")),
                    ),
            )
            .into_any_element()
    } else if is_cloud {
        let acct_name = provider_accounts
            .iter()
            .find(|a| a.id == provision.selected_account_id)
            .map(|a| {
                let disp = crate::providers::display_name(&a.plugin);
                if a.label.is_empty() || a.label.eq_ignore_ascii_case(&disp) {
                    disp
                } else {
                    format!("{} ({disp})", a.label)
                }
            })
            .unwrap_or_else(|| "—".to_string());
        let key_name = keys
            .enrolled
            .iter()
            .find(|k| k.id == provision.selected_ssh_key_id)
            .map(|k| format!("{} · {}", k.name, k.algorithm))
            .unwrap_or_else(|| "—".to_string());

        div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                div()
                    .p(px(12.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_PANEL)
                    .flex()
                    .flex_col()
                    .child(summary_row(t!("fleet-provision-provider-account"), acct_name, TEXT_PRIMARY))
                    .child(summary_row(
                        t!("fleet-provision-instance-label"),
                        if provision.label.trim().is_empty() { "—".to_string() } else { provision.label.clone() },
                        TEXT_PRIMARY,
                    ))
                    .child(summary_row(
                        t!("fleet-provision-region"),
                        if provision.region.is_empty() { "—".to_string() } else { provision.region.clone() },
                        TEXT_SECONDARY,
                    ))
                    .child(summary_row(
                        t!("fleet-provision-plan"),
                        if provision.plan.is_empty() { "—".to_string() } else { provision.plan.clone() },
                        TEXT_SECONDARY,
                    ))
                    .child(summary_row(
                        t!("fleet-provision-image"),
                        if provision.image.is_empty() { "—".to_string() } else { provision.image.clone() },
                        TEXT_SECONDARY,
                    ))
                    .child(summary_row(t!("fleet-provision-vault-ssh-key"), key_name, OK))
                    .child(summary_row(
                        t!("onboard-tags"),
                        if provision.tags.trim().is_empty() { "—".to_string() } else { provision.tags.clone() },
                        TEXT_DIM,
                    )),
            )
            .child(
                div()
                    .p(px(12.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_PANEL)
                    .child(
                        mono(10.0, TEXT_DIM)
                            .line_height(px(15.0))
                            .child(t!("onboard-summary-auto-enroll-note")),
                    ),
            )
            .into_any_element()
    } else {
        let ep = if state.host.trim().is_empty() {
            "—".to_string()
        } else {
            state.endpoint()
        };
        let route = state
            .jump_host_id
            .as_deref()
            .map(|b| crate::app::bastions::route(b, &fleet.servers))
            .unwrap_or_else(|| t!("onboard-directly").to_string());
        let auth = match state.auth_method.as_str() {
            "publickey" => {
                let kname = state
                    .selected_key_id
                    .as_deref()
                    .and_then(|id| keys.enrolled.iter().find(|k| k.id == id))
                    .map(|k| k.name.as_str())
                    .unwrap_or("key");
                format!("SSH key ({kname})")
            }
            "agent" => t!("onboard-my-ssh-agent").to_string(),
            _ => t!("onboard-a-password-once").to_string(),
        };
        let (hk_text, hk_color) = if state.host_key_accepted {
            (t!("onboard-host-key-trusted").to_string(), OK)
        } else if state.probe_result.as_ref().is_some_and(|r| r.host_key_mismatch) {
            (t!("onboard-host-key-changed-not-trusted").to_string(), CRIT)
        } else if state.probe_result.is_some() {
            (t!("onboard-host-key-new-to-this").to_string(), WARN)
        } else {
            (t!("onboard-not-checked-yet").to_string(), TEXT_DIM)
        };

        div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                div()
                    .p(px(12.0))
                    .bg(BG_PANEL)
                    .border_1()
                    .border_color(BORDER_PANEL)
                    .flex()
                    .flex_col()
                    .child(summary_row(t!("onboard-summary-endpoint"), ep, TEXT_PRIMARY))
                    .child(summary_row(
                        t!("onboard-summary-route"),
                        route,
                        if state.jump_host_id.is_some() { BASTION_BLUE } else { TEXT_SECONDARY },
                    ))
                    .child(summary_row(t!("onboard-summary-auth"), auth, TEXT_SECONDARY))
                    .child(summary_row(t!("onboard-summary-host-key"), hk_text, hk_color))
                    .child(summary_row(
                        t!("onboard-name-2"),
                        if state.label.trim().is_empty() {
                            if state.host.trim().is_empty() { "—".to_string() } else { state.host.trim().to_string() }
                        } else {
                            state.label.trim().to_string()
                        },
                        TEXT_PRIMARY,
                    ))
                    .child(summary_row(
                        t!("onboard-environment"),
                        if state.env.is_empty() { "—".to_string() } else { state.env.clone() },
                        TEXT_SECONDARY,
                    ))
                    .child(summary_row(
                        t!("onboard-group"),
                        if state.group.is_empty() { "—".to_string() } else { state.group.clone() },
                        TEXT_SECONDARY,
                    )),
            )
            .when(!state.probe_logs.is_empty(), |d| {
                d.child(
                    labeled(
                        t!("onboard-log"),
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .p(px(10.0))
                            .bg(BG_PANEL)
                            .border_1()
                            .border_color(BORDER_PANEL)
                            .children(state.probe_logs.iter().map(|l| {
                                div()
                                    .flex()
                                    .gap(px(6.0))
                                    .child(mono(9.5, l.color).flex_none().w(px(10.0)).child(l.glyph.clone()))
                                    .child(mono(10.0, TEXT_SECONDARY).flex_1().min_w(px(0.0)).line_height(px(14.0)).child(l.message.clone()))
                            })),
                    ),
                )
            })
            .into_any_element()
    };

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    mono(10.5, TEXT_PRIMARY)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(header_title),
                ),
        )
        .child(
            div()
                .id("onboard-right-rail-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .child(body),
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
