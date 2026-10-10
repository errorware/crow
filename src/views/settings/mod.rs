//! Settings view: split into navigation rail, central content views
//! (routed via exhaustive match), and right rail (diff or keys).

pub mod clankers;
pub mod clankers_state;
pub mod config_table;
pub mod email_form;
pub mod diff_rail;
pub mod keys;
pub mod keys_state;
pub mod lab;
pub mod plugins;
pub mod providers;
pub mod state;
pub mod vault_manage;

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use crate::app::{CrowApp, Screen, SettingsSection};
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::config::CrowConfigManager;
use crate::theme::*;
use crate::vault::Vault;
use crate::views::fleet::FleetState;
use crate::app::keys::KeyModalInputs;
use crate::views::settings::clankers_state::ClankersState;
use crate::views::settings::keys_state::KeysState;
use crate::views::settings::lab::LabState;
use crate::views::settings::state::SettingsState;

pub fn settings_view(
    app: Entity<CrowApp>,
    vault: &Vault,
    config: &CrowConfigManager,
    key_inputs: Option<&KeyModalInputs>,
    fleet: &FleetState,
    keys: &KeysState,
    clankers: &ClankersState,
    settings: &SettingsState,
    lab_state: &LabState,
    section: SettingsSection,
    fleet_background: (&crate::app::appearance::FleetBackground, Option<std::path::PathBuf>),
    providers_state: &crate::app::providers::ProvidersState,
    provider_inputs: Option<&crate::app::providers::ProviderFormInputs>,
    secrets_notice: Option<&str>,
    vault_form: (&crate::app::vault_manage::VaultFormState, Option<&crate::app::vault_manage::VaultFormInputs>),
    clanker_inputs: Option<&crate::app::ClankerInputs>,
    custom_setting_input: Option<&Entity<InputState>>,
    plugins: &crate::app::plugins::PluginsState,
    email: (&crate::app::notify::EmailState, Option<&crate::app::notify::EmailInputs>),
) -> impl IntoElement {
    let is_auth_enabled = vault.is_password_auth_enabled();
    let secrets_blocker = vault.secrets_blocker().filter(|_| !matches!(vault.keyring_state, crate::vault::KeyringState::Loading));

    let nav_items = [
        (TablerIcon::AdjustmentsHorizontal, "General", SettingsSection::General),
        (TablerIcon::Network, "Connection & SSH", SettingsSection::Connection),
        (TablerIcon::Key, "Keys & Rotation", SettingsSection::Keys),
        (TablerIcon::ShieldCheck, "Vault & Security", SettingsSection::Security),
        (TablerIcon::Server, "Servers & Archives", SettingsSection::Servers),
        (TablerIcon::Box, "UI Components Lab", SettingsSection::Components),
        (TablerIcon::Cpu, "Clankers (AI)", SettingsSection::Clankers),
        (TablerIcon::Cloud, "Providers", SettingsSection::Providers),
        (TablerIcon::Settings, "Plugins", SettingsSection::Plugins),
        (TablerIcon::Photo, "Personalisation", SettingsSection::Personalisation),
    ];

    let total_changed = config.total_changed_count();
    let path_str = config.path.display().to_string();
    let status_subtitle = if total_changed == 0 {
        format!("{} · in sync", path_str)
    } else {
        format!(
            "{} · {} pending change{}",
            path_str,
            total_changed,
            if total_changed == 1 { "" } else { "s" }
        )
    };

    let keychain: Vec<(String, String, Rgba)> = keys
        .enrolled
        .iter()
        .take(6)
        .map(|k| {
            let present = k.private_key_path.as_deref().is_some_and(|p| crate::keys::expand_tilde(p).exists());
            let used = match k.attached_servers.len() {
                0 => "no servers".to_string(),
                1 => "1 server".to_string(),
                n => format!("{n} servers"),
            };
            (k.name.clone(), format!("{} · {}", k.algorithm, used), if present { OK } else { TEXT_FAINT })
        })
        .collect();

    let app_close = app.clone();

    // Route center content exhaustively
    let center_content: AnyElement = match section {
        SettingsSection::Keys => keys::render_keys_center_column(app.clone(), keys).into_any_element(),
        SettingsSection::Components => lab::render_components_lab(app.clone(), lab_state).into_any_element(),
        SettingsSection::Clankers => clankers::render_clankers_view(app.clone(), clankers, secrets_blocker.clone(), secrets_notice).into_any_element(),
        SettingsSection::Providers => providers::render_providers_view(app.clone(), providers_state, provider_inputs, secrets_blocker.clone(), secrets_notice, !is_auth_enabled, &plugins.enabled).into_any_element(),
        SettingsSection::Plugins => plugins::render_plugins_view(app.clone(), plugins, &providers_state.accounts, email).into_any_element(),
        SettingsSection::General
        | SettingsSection::Connection
        | SettingsSection::Security
        | SettingsSection::Servers
        | SettingsSection::Personalisation => config_table::render_config_section(
            section,
            app.clone(),
            vault,
            config,
            settings,
            fleet_background,
            vault_form,
            custom_setting_input,
        )
        .into_any_element(),
    };

    // Right rail: keys rail for Keys, diff rail for generic config tables, none for others
    let right_rail: Option<AnyElement> = match section {
        SettingsSection::Keys => Some(
            div()
                .w(px(340.0))
                .flex_none()
                .flex()
                .flex_col()
                .bg(BG_RAIL)
                .border_l_1()
                .border_color(BORDER_PANEL)
                .child(keys::render_keys_right_rail(app.clone(), keys))
                .into_any_element(),
        ),
        SettingsSection::General
        | SettingsSection::Connection
        | SettingsSection::Security
        | SettingsSection::Servers
        | SettingsSection::Personalisation => Some(
            div()
                .w(px(340.0))
                .flex_none()
                .flex()
                .flex_col()
                .bg(BG_RAIL)
                .border_l_1()
                .border_color(BORDER_PANEL)
                .child(diff_rail::render_diff_rail(config, section, &keychain, app.clone()))
                .into_any_element(),
        ),
        SettingsSection::Components
        | SettingsSection::Clankers
        | SettingsSection::Providers
        | SettingsSection::Plugins => None,
    };

    div()
        .relative()
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
                                .child("SETTINGS"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .text_color(TEXT_DIM)
                                .child(status_subtitle),
                        ),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-close-settings")
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
                            app_close.update(cx, |this, cx| {
                                this.close_settings_dropdown(cx);
                                this.set_screen(Screen::Fleet, cx);
                            });
                        })
                        .child("CLOSE esc"),
                ),
        )
        // 2. 3-Column Content: Nav (200px) | Central Section (flex-1) | Right Rail (340px)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Left Navigation Rail
                .child(
                    div()
                        .w(px(200.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .pt(px(6.0))
                        // The components lab is a developer sandbox: debug builds only.
                        .children(nav_items.into_iter().filter(|(_, _, sec)| cfg!(debug_assertions) || *sec != SettingsSection::Components).enumerate().map(|(idx, (icon, label, sec))| {
                            let app_nav = app.clone();
                            let is_active = sec == section;
                            let changed_in_sec = config.changed_count_for_section(sec.id_prefix());
                            let (badge, badge_c) = if sec == SettingsSection::Security {
                                if changed_in_sec > 0 {
                                    (format!("{}", changed_in_sec), Some(WARN))
                                } else if is_auth_enabled {
                                    ("L".to_string(), Some(OK))
                                } else {
                                    ("!".to_string(), Some(WARN))
                                }
                            } else if changed_in_sec > 0 {
                                (format!("{}", changed_in_sec), Some(WARN))
                            } else {
                                ("".to_string(), None)
                            };

                            div()
                                .id(ElementId::NamedInteger("settings-nav".into(), idx as u64))
                                .relative()
                                .h(px(34.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .px(px(12.0))
                                .bg(if is_active { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                                .children(if is_active {
                                    Some(left_indicator(TEXT_PRIMARY))
                                } else {
                                    None
                                })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_nav.update(cx, |this, cx| {
                                        this.close_settings_dropdown(cx);
                                        this.set_settings_section(sec, cx);
                                    });
                                })
                                .child(
                                    div()
                                        .w(px(16.0))
                                        .h(px(16.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            tabler_icon(icon)
                                                .size(px(14.0))
                                                .text_color(if is_active { TEXT_PRIMARY } else { TEXT_DIMMER }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.5))
                                        .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_active { TEXT_PRIMARY } else { TEXT_MUTED })
                                        .child(label),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(badge_c.unwrap_or(TEXT_FAINT))
                                        .child(badge),
                                )
                        })),
                )
                // Center Section
                .child(center_content)
                // Right Rail
                .children(right_rail),
        )
        // Modals for Keys section
        .children(keys::render_key_modals(app.clone(), key_inputs, fleet, keys))
        // Modals for Clankers section
        .children(clankers::render_clanker_modals(app.clone(), clankers, clanker_inputs))
}
