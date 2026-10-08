//! Settings configuration table: generic config rows, options dropdown,
//! custom value inputs, and section-specific blocks (Personalisation & Security).

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::components::icon_button::icon_button;

use crow_config_core::ir::FieldIr;
use crow_config_core::schema::FieldType;

use crate::app::appearance::FleetBackground;
use crate::app::vault_manage::{VaultFormInputs, VaultFormState};
use crate::app::{CrowApp, Screen, SettingsSection};
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::config::CrowConfigManager;
use crate::theme::*;
use crate::vault::Vault;
use crate::views::settings::state::SettingsState;
use crate::views::settings::vault_manage;

#[derive(Clone, Debug)]
pub struct SettingOption {
    pub value: String,
    pub label: String,
    pub desc: String,
}

pub fn format_field_label(leaf: &str) -> String {
    let mut out = String::new();
    let mut prev_is_sep = true;
    for c in leaf.chars() {
        if c == '_' || c == '.' || c == '-' {
            out.push(' ');
            prev_is_sep = true;
        } else if prev_is_sep {
            out.extend(c.to_uppercase());
            prev_is_sep = false;
        } else {
            out.push(c);
        }
    }
    out
}

pub fn format_display_value(leaf: &str, field: Option<&FieldIr>) -> String {
    if let Some(f) = field {
        match &f.value {
            serde_json::Value::Bool(b) => {
                if *b { "YES".to_string() } else { "NO".to_string() }
            }
            serde_json::Value::Number(n) => {
                if leaf.ends_with("_ms") {
                    format!("{} ms", n)
                } else if leaf.ends_with("_sec") || leaf.ends_with("_secs") || leaf.ends_with("_seconds") {
                    format!("{} s", n)
                } else if leaf.ends_with("_bytes") {
                    format!("{} B", n)
                } else {
                    n.to_string()
                }
            }
            serde_json::Value::String(s) => {
                if s.is_empty() {
                    "(empty)".to_string()
                } else {
                    s.clone()
                }
            }
            serde_json::Value::Array(arr) => {
                let joined = arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                if joined.is_empty() { "(none)".to_string() } else { joined }
            }
            _ => serde_json::to_string(&f.value).unwrap_or_else(|_| "unknown".to_string()),
        }
    } else {
        "unknown".to_string()
    }
}

pub fn get_field_options(
    leaf: &str,
    field: Option<&FieldIr>,
    current_val: &str,
    is_bool: bool,
) -> (Vec<SettingOption>, Option<usize>, bool) {
    if is_bool {
        let is_yes = current_val.eq_ignore_ascii_case("yes") || current_val.eq_ignore_ascii_case("true");
        return (
            vec![
                SettingOption { value: "true".into(), label: "YES".into(), desc: "Enable setting".into() },
                SettingOption { value: "false".into(), label: "NO".into(), desc: "Disable setting".into() },
            ],
            Some(if is_yes { 0 } else { 1 }),
            false,
        );
    }

    match leaf {
        "connect_timeout_sec" => (
            vec![
                SettingOption { value: "5".into(), label: "5s".into(), desc: "Aggressive timeout".into() },
                SettingOption { value: "10".into(), label: "10s".into(), desc: "Standard recommended".into() },
                SettingOption { value: "15".into(), label: "15s".into(), desc: "Tolerant of high-latency links".into() },
                SettingOption { value: "30".into(), label: "30s".into(), desc: "Slow satellite / dialup".into() },
            ],
            None,
            true,
        ),
        "keepalive_interval_sec" => (
            vec![
                SettingOption { value: "15".into(), label: "15s".into(), desc: "Aggressive ping".into() },
                SettingOption { value: "30".into(), label: "30s".into(), desc: "Standard recommended".into() },
                SettingOption { value: "60".into(), label: "60s".into(), desc: "Relaxed heartbeat".into() },
                SettingOption { value: "0".into(), label: "OFF".into(), desc: "Disable keepalive ping".into() },
            ],
            None,
            true,
        ),
        "keepalive_max_count" => (
            vec![
                SettingOption { value: "2".into(), label: "2 attempts".into(), desc: "Fast dead-peer detection".into() },
                SettingOption { value: "3".into(), label: "3 attempts".into(), desc: "Standard recommended".into() },
                SettingOption { value: "5".into(), label: "5 attempts".into(), desc: "Tolerate transient packet drop".into() },
                SettingOption { value: "10".into(), label: "10 attempts".into(), desc: "Very tolerant".into() },
            ],
            None,
            true,
        ),
        "metrics_refresh_sec" => (
            vec![
                SettingOption { value: "1".into(), label: "1s (High resolution)".into(), desc: "Higher local/remote CPU use".into() },
                SettingOption { value: "2".into(), label: "2s (Default)".into(), desc: "Smooth streaming & low load".into() },
                SettingOption { value: "5".into(), label: "5s (Battery/Eco)".into(), desc: "Low background overhead".into() },
                SettingOption { value: "10".into(), label: "10s (Minimal)".into(), desc: "Minimal polling footprint".into() },
            ],
            None,
            true,
        ),
        "cipher_preference" => (
            vec![
                SettingOption { value: "chacha20-poly1305@openssh.com".into(), label: "chacha20-poly1305 (Fast & modern)".into(), desc: "Hardware-agnostic AEAD".into() },
                SettingOption { value: "aes128-gcm@openssh.com".into(), label: "aes128-gcm (AES-NI accelerated)".into(), desc: "Fast on modern Intel/AMD".into() },
                SettingOption { value: "aes256-gcm@openssh.com".into(), label: "aes256-gcm (Maximum depth)".into(), desc: "High security environments".into() },
                SettingOption { value: "aes256-gcm".into(), label: "aes256-gcm".into(), desc: "AES256-GCM only".into() },
            ],
            None,
            true,
        ),
        "default_identity" => (
            vec![
                SettingOption { value: "~/.ssh/id_ed25519".into(), label: "~/.ssh/id_ed25519".into(), desc: "Default Ed25519 key".into() },
                SettingOption { value: "~/.ssh/id_ed25519_bastion".into(), label: "~/.ssh/id_ed25519_bastion".into(), desc: "Dedicated bastion key".into() },
                SettingOption { value: "~/.ssh/id_rsa".into(), label: "~/.ssh/id_rsa".into(), desc: "Legacy RSA identity".into() },
            ],
            None,
            true,
        ),
        _ => {
            if let Some(opts) = field.and_then(|f| f.options.clone()) {
                (
                    opts.into_iter().map(|o| SettingOption {
                        value: o.value.clone(),
                        label: o.label.clone(),
                        desc: format!("Risk: {:?}", o.risk),
                    }).collect(),
                    None,
                    false,
                )
            } else {
                (Vec::new(), None, true)
            }
        }
    }
}

pub fn personalisation_block(bg: &(&FleetBackground, Option<std::path::PathBuf>), app: Entity<CrowApp>) -> impl IntoElement {
    let (state, source) = bg;
    let (app_choose, app_remove) = (app.clone(), app);
    let button = |id: &'static str, color: Rgba| {
        div()
            .id(id)
            .px(px(10.0))
            .py(px(5.0))
            .border_1()
            .border_color(color)
            .text_color(color)
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .font_weight(FontWeight::BOLD)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
    };
    div()
        .m(px(14.0))
        .p(px(14.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .flex()
        .gap(px(16.0))
        .child(
            div()
                .w(px(240.0))
                .h(px(135.0))
                .flex_none()
                .bg(BG_APP)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .child(match (state.rendered.clone(), source) {
                    (Some(p), Some(_)) => img(p).size_full().object_fit(ObjectFit::Cover).into_any_element(),
                    (_, Some(_)) => div().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_FAINT).child("preparing…").into_any_element(),
                    _ => inherited_icon(TablerIcon::Photo, px(28.0)).text_color(TEXT_FAINTER).into_any_element(),
                }),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .font_family(FONT_MONO)
                .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("FLEET PAGE BACKGROUND"))
                .child(div().text_size(px(10.0)).text_color(TEXT_DIM).child("A picture behind the server list, faded and softened so the table stays readable. PNG, JPEG or WebP; Crow keeps its own copy."))
                .children(state.error.clone().map(|e| div().text_size(px(10.0)).text_color(CRIT).child(e)))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .pt(px(6.0))
                        .child(button("btn-choose-background", OK).on_click(move |_ev, _window, cx| app_choose.update(cx, |this, cx| this.choose_fleet_background(cx))).child(if source.is_some() { "CHANGE PICTURE…" } else { "CHOOSE PICTURE…" }))
                        .children(source.is_some().then(|| button("btn-remove-background", TEXT_SECONDARY).on_click(move |_ev, _window, cx| app_remove.update(cx, |this, cx| this.remove_fleet_background(cx))).child("REMOVE"))),
                ),
        )
}

pub fn render_config_section(
    section: SettingsSection,
    app: Entity<CrowApp>,
    vault: &Vault,
    config: &CrowConfigManager,
    settings: &SettingsState,
    fleet_background: (&FleetBackground, Option<std::path::PathBuf>),
    vault_form: (&VaultFormState, Option<&VaultFormInputs>),
    custom_setting_input: Option<&Entity<InputState>>,
) -> impl IntoElement {
    let (title, sub) = match section {
        SettingsSection::General => ("GENERAL", "[general] · application behavior"),
        SettingsSection::Connection => ("CONNECTION & SSH", "[connection] · applies to every host unless overridden"),
        SettingsSection::Keys => ("KEYS & ROTATION", "[keys] · key distribution & policies"),
        SettingsSection::Security => ("VAULT & SECURITY", "[vault] · encrypted secrets & who can open Crow"),
        SettingsSection::Servers => ("SERVERS & ARCHIVES", "[servers] · archiving and how long stored data is kept"),
        SettingsSection::Components => ("UI COMPONENTS LAB", "[lab] · gpui-component testbed & sandbox"),
        SettingsSection::Clankers => ("CLANKERS (AI USABILITY)", "[clankers] · api keys & log eli5 helpers"),
        SettingsSection::Providers => ("PROVIDERS", "[providers] · cloud accounts your servers run on"),
        SettingsSection::Plugins => ("PLUGINS", "[plugins] · integrations, off until you switch them on"),
        SettingsSection::Personalisation => ("PERSONALISATION", "[appearance] · make Crow yours · saved as you change it"),
    };

    let is_auth_enabled = vault.is_password_auth_enabled();
    let open_dropdown = settings.dropdown_open.as_deref();
    let custom_input = &settings.custom_input;

    let sec_prefix = format!("{}.", section.id_prefix());
    let sec_rows: Vec<_> = config
        .ir
        .rows
        .iter()
        .filter(|r| r.row_id.starts_with(&sec_prefix))
        .filter(|r| r.row_id != crate::app::appearance::FLEET_BACKGROUND)
        .filter(|r| crate::config::WIRED_SETTINGS.contains(&r.row_id.as_str()))
        .collect();
    let hidden_settings = config.ir.rows.iter().filter(|r| r.row_id.starts_with(&sec_prefix) && !crate::config::WIRED_SETTINGS.contains(&r.row_id.as_str())).count();

    let has_section_changes = config.changed_count_for_section(section.id_prefix()) > 0;
    let total_changed = config.total_changed_count();
    let sec_prefix_id = section.id_prefix().to_string();

    let app_enable_auth = app.clone();
    let app_lock_now = app.clone();
    let app_reset = app.clone();
    let app_edit = app.clone();
    let app_save = app.clone();

    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .bg(BG_APP)
        // Section Header
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
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(title),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child(sub),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .child("overrides apply per-server in Config view"),
                ),
        )
        // Rows List
        .child(
            div()
                .id("settings-rows-list")
                .flex_1()
                .overflow_y_scrollbar()
                .children((section == SettingsSection::Personalisation).then(|| personalisation_block(&fleet_background, app.clone())))
                .children(if section == SettingsSection::Security {
                    if !is_auth_enabled {
                        Some(
                            div()
                                .m(px(14.0))
                                .p(px(16.0))
                                .bg(BG_OVERLAY_PANEL)
                                .border_1()
                                .border_color(CRIT)
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
                                                .gap(px(8.0))
                                                .child(div().font_family(FONT_MONO).text_size(px(14.0)).text_color(CRIT).child("⚠"))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(12.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(TEXT_PRIMARY)
                                                        .child("PASSWORD LOGON IS NOT CONFIGURED"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .bg(CRIT_BG)
                                                .border_1()
                                                .border_color(CRIT)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(CRIT_INK_DIM)
                                                .child("DISABLED (DEFAULT)"),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child("Without a password, anyone using your computer account can open Crow. Secrets (provider tokens, AI keys) are still encrypted, with the key held by your OS keyring. A master password with two-factor authentication (RFC 6238 TOTP) makes Crow ask before it opens, and locks the secrets' key with that password instead."),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_end()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .id("btn-enable-auth")
                                                .px(px(12.0))
                                                .py(px(6.0))
                                                .bg(OK)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgb(0x4ade80)))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_enable_auth.update(cx, |this, cx| {
                                                        this.setup_state = crate::views::lock::SetupState::default();
                                                        this.set_screen(Screen::VaultSetup, cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(BG_WINDOW)
                                                        .child("SET PASSWORD & 2FA →"),
                                                ),
                                        ),
                                ),
                        )
                    } else {
                        Some(
                            div()
                                .m(px(14.0))
                                .p(px(16.0))
                                .bg(BG_OVERLAY_PANEL)
                                .border_1()
                                .border_color(OK)
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
                                                .gap(px(8.0))
                                                .child(div().font_family(FONT_MONO).text_size(px(14.0)).text_color(OK).child("🛡"))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(12.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(TEXT_PRIMARY)
                                                        .child("PASSWORD + 2FA VAULT CONFIGURED"),
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
                                                .text_color(OK_INK)
                                                .child("LOCKED STANCE"),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .line_height(px(16.0))
                                        .child("Database and keys are secured. Unlocking requires password and 6-digit TOTP code. Auto-lock engages after the configured timeout."),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_end()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .id("btn-lock-vault-now")
                                                .px(px(12.0))
                                                .py(px(6.0))
                                                .bg(hex_rgb(0x1a1a24))
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(hex_rgb(0x222230)))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_lock_now.update(cx, |this, cx| {
                                                        this.lock(cx);
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .text_color(TEXT_PRIMARY)
                                                        .child("LOCK VAULT NOW (⇧⌘L)"),
                                                ),
                                        ),
                                ),
                        )
                    }
                } else {
                    None
                })
                .children((section == SettingsSection::Security && is_auth_enabled).then(|| vault_manage::render(vault_form.0, vault_form.1, app.clone())))
                .children(settings.edit_error.clone().map(|e| div().px(px(14.0)).py(px(8.0)).font_family(FONT_MONO).text_size(px(10.5)).text_color(CRIT).child(e)))
                .children((hidden_settings > 0).then(|| {
                    div()
                        .px(px(14.0))
                        .py(px(8.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .child(format!("{hidden_settings} more setting{} in this section {} not wired to anything yet, so {} hidden rather than pretending to work.", if hidden_settings == 1 { "" } else { "s" }, if hidden_settings == 1 { "is" } else { "are" }, if hidden_settings == 1 { "it's" } else { "they're" }))
                }))
                .children(sec_rows.into_iter().enumerate().map(|(idx, row)| {
                    let is_even = idx % 2 == 0;
                    let field = row.get_field(&row.row_id);
                    let leaf = row.row_id.split_once('.').map(|(_, k)| k).unwrap_or(&row.row_id);
                    let label = format_field_label(leaf);
                    let desc = field.and_then(|f| f.help.clone()).unwrap_or_default();
                    let is_changed = config.is_field_changed(&row.row_id);

                    let is_bool = matches!(field.map(|f| &f.field_type), Some(FieldType::Bool));
                    let is_int = matches!(field.map(|f| &f.field_type), Some(FieldType::Integer));
                    let is_open = open_dropdown == Some(row.row_id.as_str());

                    let display_val = format_display_value(leaf, field);
                    let (options, _selected_idx, allow_custom) = get_field_options(leaf, field, &display_val, is_bool);

                    let row_id_toggle = row.row_id.clone();
                    let row_id_reset = row.row_id.clone();
                    let init_val = display_val.clone();
                    let app_toggle = app.clone();
                    let app_reset = app.clone();

                    div()
                        .p(px(14.0))
                        .bg(if is_even { BG_APP } else { BG_PANEL })
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .items_start()
                                .justify_between()
                                .gap(px(16.0))
                                // Left Column: Title, Description, and Metadata
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(if is_changed { WARN } else { TEXT_PRIMARY })
                                                        .child(label),
                                                )
                                                .children(is_changed.then(|| {
                                                    div()
                                                        .px(px(4.0))
                                                        .py(px(0.5))
                                                        .bg(WARN_BG)
                                                        .border_1()
                                                        .border_color(WARN)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(8.5))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(WARN)
                                                        .child("CHANGED")
                                                })),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(TEXT_MUTED)
                                                .line_height(relative(1.4))
                                                .child(desc),
                                        ),
                                )
                                // Right Column: Interactive Dropdown Button and Reset Control
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .id(ElementId::NamedInteger("btn-setting-dropdown".into(), idx as u64))
                                                .px(px(10.0))
                                                .py(px(5.0))
                                                .bg(if is_open { BG_ROW_SELECTED } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_open { OK } else { BORDER_DEFAULT })
                                                .text_color(if is_open { OK } else { TEXT_PRIMARY })
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::MEDIUM)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    let rid = row_id_toggle.clone();
                                                    let ival = init_val.clone();
                                                    app_toggle.update(cx, |this, cx| {
                                                        this.toggle_settings_dropdown(&rid, &ival, cx);
                                                    });
                                                })
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(display_val.clone())
                                                .child(div().text_size(px(8.0)).text_color(TEXT_FAINTER).child(if is_open { "▲" } else { "▼" })),
                                        )
                                        .children(is_changed.then(|| {
                                            div()
                                                .id(ElementId::NamedInteger("btn-reset-field".into(), idx as u64))
                                                .px(px(6.0))
                                                .py(px(5.0))
                                                .bg(BG_CONTROL)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .text_color(TEXT_FAINT)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                                .on_click(move |_ev, _window, cx| {
                                                    let rid = row_id_reset.clone();
                                                    app_reset.update(cx, |this, cx| {
                                                        this.reset_config_field(&rid, cx);
                                                    });
                                                })
                                                .child("↺")
                                        })),
                                ),
                        )
                        // Expanded Dropdown Overlay Menu
                        .children(is_open.then(|| {
                            let app_custom_apply = app.clone();
                            let app_custom_close = app.clone();
                            let row_id_custom = row.row_id.clone();
                            let row_id_close = row.row_id.clone();

                            div()
                                .id(ElementId::NamedInteger("dropdown-panel".into(), idx as u64))
                                .mt(px(4.0))
                                .p(px(6.0))
                                .bg(BG_PANEL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .children(options.into_iter().enumerate().map(|(opt_idx, opt)| {
                                    let is_active = display_val.eq_ignore_ascii_case(&opt.value)
                                        || (is_bool && display_val.eq_ignore_ascii_case(&opt.label));
                                    let app_opt = app.clone();
                                    let row_id_opt = row.row_id.clone();
                                    let val_opt = opt.value.clone();

                                    div()
                                        .id(ElementId::NamedInteger(format!("opt-{}", row.row_id).into(), opt_idx as u64))
                                        .px(px(8.0))
                                        .py(px(5.0))
                                        .bg(if is_active { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .on_click(move |_ev, _window, cx| {
                                            let rid = row_id_opt.clone();
                                            let val_str = val_opt.clone();
                                            app_opt.update(cx, |this, cx| {
                                                if is_bool {
                                                    let b = val_str == "true";
                                                    this.update_config_field(&rid, serde_json::Value::Bool(b), cx);
                                                } else if is_int {
                                                    if let Ok(n) = val_str.parse::<i64>() {
                                                        this.update_config_field(&rid, serde_json::Value::Number(serde_json::Number::from(n)), cx);
                                                    }
                                                } else {
                                                    this.update_config_field(&rid, serde_json::Value::String(val_str), cx);
                                                }
                                                this.close_settings_dropdown(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.5))
                                                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                        .text_color(if is_active { OK } else { TEXT_PRIMARY })
                                                        .child(opt.label),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_FAINTER)
                                                        .child(format!("· {}", opt.desc)),
                                                ),
                                        )
                                        .children(is_active.then(|| {
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .text_color(OK)
                                                .child("✓")
                                        }))
                                }))
                                .children(allow_custom.then(|| {
                                    div()
                                        .mt(px(4.0))
                                        .pt(px(6.0))
                                        .border_t_1()
                                        .border_color(BORDER_PANEL)
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_DIMMER)
                                                .child("OR ENTER CUSTOM VALUE:"),
                                        )
                                        .child(
                                            if let Some(ref input) = custom_setting_input {
                                                div()
                                                    .flex_1()
                                                    .child(
                                                        Input::new(input)
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .bg(BG_APP)
                                                            .rounded(px(2.0))
                                                    )
                                                    .into_any_element()
                                            } else {
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(11.0))
                                                    .text_color(TEXT_MUTED)
                                                    .child(custom_input.clone())
                                                    .into_any_element()
                                            }
                                        )
                                        .child(
                                            div()
                                                .id("btn-apply-custom-value")
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .bg(OK)
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .text_color(BG_WINDOW)
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::BOLD)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgb(0x4ade80)))
                                                .on_click(move |_ev, _window, cx| {
                                                    let rid = row_id_custom.clone();
                                                    app_custom_apply.update(cx, |this, cx| {
                                                        this.apply_settings_custom_input(&rid, cx);
                                                    });
                                                })
                                                .child("APPLY"),
                                        )
                                        .child(
                                            icon_button("btn-cancel-custom-value", TablerIcon::X, false)
                                                .on_click(move |_ev, _window, cx| {
                                                    let _ = &row_id_close;
                                                    app_custom_close.update(cx, |this, cx| {
                                                        this.close_settings_dropdown(cx);
                                                    });
                                                }),
                                        )
                                }))
                        }))
                }))
        )
        // Bottom Action Bar: Reset Section and Save Buttons
        .child(
            div()
                .h(px(44.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .id("btn-reset-section")
                        .px(px(9.0))
                        .py(px(5.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(if has_section_changes { WARN } else { TEXT_FAINT })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            if has_section_changes {
                                let prefix = sec_prefix_id.clone();
                                app_reset.update(cx, |this, cx| {
                                    this.close_settings_dropdown(cx);
                                    this.reset_config_section(&prefix, cx);
                                });
                            }
                        })
                        .child("RESET SECTION"),
                )
                .child(
                    div()
                        .id("btn-edit-config-toml")
                        .px(px(9.0))
                        .py(px(5.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_edit.read(cx).open_config_file();
                        })
                        .child("EDIT config.toml ⌘/"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-save-settings")
                        .px(px(11.0))
                        .py(px(6.0))
                        .bg(if total_changed > 0 { OK } else { BG_CONTROL })
                        .border_1()
                        .border_color(if total_changed > 0 { OK } else { BORDER_DEFAULT })
                        .text_color(if total_changed > 0 { BG_WINDOW } else { TEXT_FAINT })
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .hover(|s| s.bg(if total_changed > 0 { rgb(0x4ade80) } else { BG_CONTROL }))
                        .on_click(move |_ev, _window, cx| {
                            if total_changed > 0 {
                                app_save.update(cx, |this, cx| {
                                    this.close_settings_dropdown(cx);
                                    this.save_config(cx);
                                });
                            }
                        })
                        .child(if total_changed > 0 {
                            format!("SAVE ⌘S ({} pending)", total_changed)
                        } else {
                            "SAVE ⌘S".to_string()
                        }),
                ),
        )
}
