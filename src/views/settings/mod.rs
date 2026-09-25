pub mod clankers_state;
pub mod keys_state;
pub mod state;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen, SettingsSection};
use crate::config::DiffKind;
use crate::components::icons::{TablerIcon, tabler_icon, inherited_icon};
use crate::components::terminal_text_input_styled;
use crow_config_core::schema::FieldType;

pub mod keys;
use self::keys::{render_key_modals, render_keys_center_column, render_keys_right_rail};
use crate::vault::Vault;
use crate::config::CrowConfigManager;
use crate::components::text_caret::TextCaret;
use crate::views::fleet::FleetState;
use crate::views::settings::keys_state::KeysState;
use crate::views::settings::clankers_state::ClankersState;
use crate::views::settings::state::SettingsState;
use crate::views::settings::lab::LabState;
pub mod lab;
pub mod clankers;

fn format_field_label(leaf: &str) -> String {
    match leaf {
        "strict_host_key_checking" => "Strict host key checking".into(),
        "agent_forwarding" => "SSH agent forwarding".into(),
        "control_master" => "Connection multiplexing".into(),
        "keepalive_interval" => "Keepalive interval".into(),
        "connect_timeout" => "Connect timeout".into(),
        "reconnect_backoff" => "Reconnect backoff".into(),
        "ciphers" => "Preferred ciphers".into(),
        "compression" => "Compression".into(),
        "theme" => "Window theme".into(),
        "font_family" => "Font family".into(),
        "font_size" => "Base font size".into(),
        "refresh_interval" => "Refresh interval".into(),
        "titlebar_latency" => "Titlebar latency probe".into(),
        "confirm_destructive" => "Confirm destructive actions".into(),
        "log_buffer_lines" => "Log buffer ceiling".into(),
        "notify_failures" => "Desktop failure notifications".into(),
        "auto_update_check" => "Auto update check".into(),
        "default_identity" => "Default identity".into(),
        "auto_rotate_days" => "Key auto-rotate threshold".into(),
        "enforce_ed25519_only" => "Enforce Ed25519 keys only".into(),
        "agent_integration" => "Local SSH agent integration".into(),
        "auto_lock_minutes" => "Auto-lock inactive vault".into(),
        "zeroize_on_drop" => "Wipe memory on lock".into(),
        _ => {
            let mut chars = leaf.replace('_', " ").chars().collect::<Vec<_>>();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect()
        }
    }
}

fn format_display_value(leaf: &str, field: Option<&crow_config_core::ir::FieldIr>) -> String {
    match field.map(|f| &f.value) {
        Some(serde_json::Value::Number(n)) => {
            let num = n.as_i64().unwrap_or(0);
            match leaf {
                "refresh_interval" | "connect_timeout" | "keepalive_interval" => format!("{} s", num),
                "auto_lock_minutes" => format!("{} min", num),
                "auto_rotate_days" => format!("{} days", num),
                "font_size" => format!("{} px", num),
                "log_buffer_lines" => format!("{} lines", num),
                _ => format!("{}", num),
            }
        }
        Some(serde_json::Value::String(s)) => match leaf {
            "theme" => match s.as_str() {
                "obsidian_edge" => "Obsidian Edge".into(),
                "slate_dark" => "Slate Dark".into(),
                other => other.to_string(),
            },
            _ => s.clone(),
        },
        Some(serde_json::Value::Bool(b)) => format!("{}", b),
        _ => String::new(),
    }
}

struct SettingOption {
    value: String,
    label: String,
    desc: String,
}

fn get_field_options(
    leaf: &str,
    field: Option<&crow_config_core::ir::FieldIr>,
) -> (Vec<SettingOption>, Option<&'static str>, bool) {
    match leaf {
        "refresh_interval" => (
            vec![
                SettingOption { value: "1".into(), label: "1 s".into(), desc: "Aggressive (1 sec — high frequency polling)".into() },
                SettingOption { value: "2".into(), label: "2 s".into(), desc: "Recommended (2 sec — balanced telemetry & CPU)".into() },
                SettingOption { value: "5".into(), label: "5 s".into(), desc: "Standard (5 sec — moderate background polling)".into() },
                SettingOption { value: "10".into(), label: "10 s".into(), desc: "Conservative (10 sec — low network overhead)".into() },
            ],
            Some("seconds"),
            true,
        ),
        "connect_timeout" => (
            vec![
                SettingOption { value: "5".into(), label: "5 s".into(), desc: "Fast fail (5 sec — responsive drop detection)".into() },
                SettingOption { value: "10".into(), label: "10 s".into(), desc: "Standard (10 sec — default TCP handshake)".into() },
                SettingOption { value: "15".into(), label: "15 s".into(), desc: "Relaxed (15 sec — high-latency WAN)".into() },
                SettingOption { value: "30".into(), label: "30 s".into(), desc: "Satellite / congested links (30 sec)".into() },
            ],
            Some("seconds"),
            true,
        ),
        "keepalive_interval" => (
            vec![
                SettingOption { value: "10".into(), label: "10 s".into(), desc: "Active probe (10 sec — fast drop detection)".into() },
                SettingOption { value: "15".into(), label: "15 s".into(), desc: "Standard (15 sec — default heartbeat)".into() },
                SettingOption { value: "30".into(), label: "30 s".into(), desc: "Relaxed (30 sec — low traffic chatter)".into() },
                SettingOption { value: "60".into(), label: "60 s".into(), desc: "Passive (60 sec — minimal heartbeat packets)".into() },
            ],
            Some("seconds"),
            true,
        ),
        "font_size" => (
            vec![
                SettingOption { value: "11".into(), label: "11 px".into(), desc: "Compact (maximum density)".into() },
                SettingOption { value: "12".into(), label: "12 px".into(), desc: "Standard (default readability)".into() },
                SettingOption { value: "13".into(), label: "13 px".into(), desc: "Medium (enhanced clarity)".into() },
                SettingOption { value: "14".into(), label: "14 px".into(), desc: "Large (comfortable view)".into() },
            ],
            Some("px"),
            true,
        ),
        "log_buffer_lines" => (
            vec![
                SettingOption { value: "1000".into(), label: "1,000".into(), desc: "Minimal memory footprint".into() },
                SettingOption { value: "5000".into(), label: "5,000".into(), desc: "Balanced log tail".into() },
                SettingOption { value: "10000".into(), label: "10,000".into(), desc: "Standard audit depth".into() },
                SettingOption { value: "25000".into(), label: "25,000".into(), desc: "Deep diagnostic history".into() },
            ],
            Some("lines"),
            true,
        ),
        "auto_lock_minutes" => (
            vec![
                SettingOption { value: "5".into(), label: "5 min".into(), desc: "High security (quick auto-lock)".into() },
                SettingOption { value: "15".into(), label: "15 min".into(), desc: "Standard timeout (recommended)".into() },
                SettingOption { value: "30".into(), label: "30 min".into(), desc: "Relaxed workspace".into() },
                SettingOption { value: "60".into(), label: "60 min".into(), desc: "Extended session".into() },
            ],
            Some("minutes"),
            true,
        ),
        "auto_rotate_days" => (
            vec![
                SettingOption { value: "30".into(), label: "30 days".into(), desc: "Monthly rotation".into() },
                SettingOption { value: "60".into(), label: "60 days".into(), desc: "Bi-monthly".into() },
                SettingOption { value: "90".into(), label: "90 days".into(), desc: "Quarterly (industry standard)".into() },
                SettingOption { value: "180".into(), label: "180 days".into(), desc: "Semi-annual rotation".into() },
            ],
            Some("days"),
            true,
        ),
        "theme" => (
            vec![
                SettingOption { value: "obsidian_edge".into(), label: "Obsidian Edge".into(), desc: "High-contrast dark terminal aesthetic".into() },
                SettingOption { value: "slate_dark".into(), label: "Slate Dark".into(), desc: "Subtle charcoal slate palette".into() },
            ],
            None,
            false,
        ),
        "reconnect_backoff" => (
            vec![
                SettingOption { value: "exponential".into(), label: "exponential".into(), desc: "Exponential delay (1s, 2s, 4s… capped at 30s)".into() },
                SettingOption { value: "linear".into(), label: "linear".into(), desc: "Linear incremental delay".into() },
                SettingOption { value: "fixed".into(), label: "fixed".into(), desc: "Fixed constant retry delay".into() },
            ],
            None,
            false,
        ),
        "font_family" => (
            vec![
                SettingOption { value: "JetBrains Mono".into(), label: "JetBrains Mono".into(), desc: "Default monospace (recommended)".into() },
                SettingOption { value: "Fira Code".into(), label: "Fira Code".into(), desc: "Programming ligatures support".into() },
                SettingOption { value: "SF Mono".into(), label: "SF Mono".into(), desc: "Apple system monospace".into() },
            ],
            None,
            true,
        ),
        "ciphers" => (
            vec![
                SettingOption { value: "chacha20-poly1305,aes256-gcm".into(), label: "chacha20, aes256-gcm".into(), desc: "Modern AEAD only (recommended)".into() },
                SettingOption { value: "chacha20-poly1305".into(), label: "chacha20-poly1305".into(), desc: "ChaCha20-Poly1305 only".into() },
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

pub fn settings_view(
    app: Entity<CrowApp>,
    vault: &Vault, config: &CrowConfigManager, caret: &TextCaret, fleet: &FleetState, keys: &KeysState, clankers: &ClankersState, settings: &SettingsState, lab_state: &LabState,
    section: SettingsSection,
    fleet_background: (&crate::app::appearance::FleetBackground, Option<std::path::PathBuf>),
) -> impl IntoElement {
    let is_auth_enabled = vault.is_password_auth_enabled();
    let open_dropdown = settings.dropdown_open.as_deref();
    let custom_input = &settings.custom_input;
    let nav_items = [
        (TablerIcon::AdjustmentsHorizontal, "General", SettingsSection::General),
        (TablerIcon::Network, "Connection & SSH", SettingsSection::Connection),
        (TablerIcon::Key, "Keys & Rotation", SettingsSection::Keys),
        (TablerIcon::ShieldCheck, "Vault & Security", SettingsSection::Security),
        (TablerIcon::Server, "Servers & Archives", SettingsSection::Servers),
        (TablerIcon::Box, "UI Components Lab", SettingsSection::Components),
        (TablerIcon::Cpu, "Clankers (AI)", SettingsSection::Clankers),
        (TablerIcon::Photo, "Personalisation", SettingsSection::Personalisation),
    ];

    let (title, sub) = match section {
        SettingsSection::General => ("GENERAL", "[general] · application behavior"),
        SettingsSection::Connection => ("CONNECTION & SSH", "[connection] · applies to every host unless overridden"),
        SettingsSection::Keys => ("KEYS & ROTATION", "[keys] · key distribution & policies"),
        SettingsSection::Security => ("VAULT & SECURITY", "[vault] · local encrypted sqlite & master key"),
        SettingsSection::Servers => ("SERVERS & ARCHIVES", "[servers] · archiving and how long stored data is kept"),
        SettingsSection::Components => ("UI COMPONENTS LAB", "[lab] · gpui-component testbed & sandbox"),
        SettingsSection::Clankers => ("CLANKERS (AI USABILITY)", "[clankers] · api keys & log eli5 helpers"),
        SettingsSection::Personalisation => ("PERSONALISATION", "[appearance] · make Crow yours · saved as you change it"),
    };

    // The vault's enrolled keys (a private key file missing on disk is dimmed).
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

    let sec_prefix = format!("{}.", section.id_prefix());
    let sec_rows: Vec<_> = config
        .ir
        .rows
        .iter()
        .filter(|r| r.row_id.starts_with(&sec_prefix))
        // The picture is chosen with the picker below, not typed as a path.
        .filter(|r| r.row_id != crate::app::appearance::FLEET_BACKGROUND)
        .collect();

    let has_section_changes = config.changed_count_for_section(section.id_prefix()) > 0;
    let sec_prefix_id = section.id_prefix().to_string();

    let diff_lines = config.generate_diff();
    let additions = diff_lines.iter().filter(|d| d.kind == DiffKind::Addition).count();
    let deletions = diff_lines.iter().filter(|d| d.kind == DiffKind::Deletion).count();
    let diff_badge = if additions == 0 && deletions == 0 {
        "in sync".to_string()
    } else {
        format!("+{} −{}", additions, deletions)
    };
    let has_conn_changes = config.changed_count_for_section("connection") > 0;

    let app_close = app.clone();
    let app_enable_auth = app.clone();
    let app_lock_now = app.clone();
    let app_reset = app.clone();
    let app_edit = app.clone();
    let app_save = app.clone();

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
        // 2. 3-Column Content: Nav (200px) | Rows (flex-1) | Pending Diff Rail (340px)
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                // Left Nav Column
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
                        .children(nav_items.into_iter().enumerate().map(|(idx, (icon, label, sec))| {
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
                // Center Settings Rows Column
                .children(if section == SettingsSection::Keys {
                    Some(render_keys_center_column(app.clone(), keys).into_any_element())
                } else if section == SettingsSection::Components {
                    Some(lab::render_components_lab(app.clone(), caret, lab_state).into_any_element())
                } else if section == SettingsSection::Clankers {
                    Some(clankers::render_clankers_view(app.clone(), clankers).into_any_element())
                } else {
                    None
                })
                .children(if section != SettingsSection::Keys && section != SettingsSection::Components && section != SettingsSection::Clankers {
                    Some(
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
                                                        .child("By default, Crow operates with direct unauthenticated local access. You can protect your local keys, sessions, and configuration by enabling master password logon and mandatory two-factor authentication (RFC 6238 TOTP)."),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .justify_end()
                                                        .child(
                                                            div()
                                                                .id("btn-enable-password-logon")
                                                                .h(px(32.0))
                                                                .px(px(16.0))
                                                                .bg(CRIT)
                                                                .hover(|s| s.bg(rgb(0xf87171)))
                                                                .cursor_pointer()
                                                                .flex()
                                                                .items_center()
                                                                .gap(px(6.0))
                                                                .on_click(move |_ev, _window, cx| {
                                                                    app_enable_auth.update(cx, |this, cx| {
                                                                        this.setup_state = crate::views::lock::SetupState::default();
                                                                        this.set_screen(Screen::VaultSetup, cx);
                                                                    });
                                                                })
                                                                .child(
                                                                    div()
                                                                        .font_family(FONT_MONO)
                                                                        .text_size(px(11.5))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(rgb(0x050507))
                                                                        .child("ENABLE PASSWORD & MANDATORY 2FA…"),
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
                                                                .child(div().font_family(FONT_MONO).text_size(px(14.0)).text_color(OK).child("✓"))
                                                                .child(
                                                                    div()
                                                                        .font_family(FONT_MONO)
                                                                        .text_size(px(12.5))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(TEXT_PRIMARY)
                                                                        .child("LOCAL ENCRYPTED VAULT ACTIVE"),
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
                                                                .text_color(OK)
                                                                .child("PASSWORD + 2FA ENFORCED"),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(11.0))
                                                        .text_color(TEXT_MUTED)
                                                        .line_height(px(16.0))
                                                        .child("Database is encrypted at ~/.config/crow/crow.db via Argon2id + ChaCha20-Poly1305. MasterKey is zeroized on lock."),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .justify_end()
                                                        .child(
                                                            div()
                                                                .id("btn-lock-vault-now")
                                                                .h(px(30.0))
                                                                .px(px(14.0))
                                                                .bg(BG_KEY)
                                                                .border_1()
                                                                .border_color(BORDER_DEFAULT)
                                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                                .cursor_pointer()
                                                                .flex()
                                                                .items_center()
                                                                .gap(px(6.0))
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
                                .children(sec_rows.into_iter().enumerate().map(|(idx, row)| {
                                    let is_even = idx % 2 == 0;
                                    let field = row.get_field(&row.row_id);
                                    let leaf = row.row_id.split_once('.').map(|(_, k)| k).unwrap_or(&row.row_id);
                                    let label = format_field_label(leaf);
                                    let desc = field.and_then(|f| f.help.clone()).unwrap_or_default();
                                    let is_changed = config.is_field_changed(&row.row_id);

                                    let is_bool = matches!(field.map(|f| &f.field_type), Some(FieldType::Bool));
                                    let is_int = matches!(field.map(|f| &f.field_type), Some(FieldType::Other(cow)) if cow == "integer");
                                    let is_open = open_dropdown == Some(row.row_id.as_str());

                                    let display_val = format_display_value(leaf, field);
                                    let raw_val_str = match field.map(|f| &f.value) {
                                        Some(serde_json::Value::Bool(b)) => format!("{}", b),
                                        Some(serde_json::Value::Number(n)) => format!("{}", n),
                                        Some(serde_json::Value::String(s)) => s.clone(),
                                        _ => String::new(),
                                    };

                                    let key_repr = if is_bool || is_int {
                                        format!("{} = {}", leaf, raw_val_str)
                                    } else {
                                        format!("{} = \"{}\"", leaf, raw_val_str)
                                    };

                                    let (presets, unit_suffix, allow_custom) = get_field_options(leaf, field);

                                    let app_toggle = app.clone();
                                    let app_btn = app.clone();
                                    let app_apply = app.clone();
                                    let app_cancel = app.clone();
                                    let app_custom = app.clone();
                                    let row_id_str = row.row_id.clone();
                                    let row_id_for_apply = row.row_id.clone();
                                    let initial_val_for_open = raw_val_str.clone();

                                    div()
                                        .id(ElementId::NamedInteger("setting-row".into(), idx as u64))
                                        .relative()
                                        .flex()
                                        .flex_col()
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .bg(if is_open {
                                            hex_rgb(0x0e0e12)
                                        } else if is_changed {
                                            BG_OVERLAY_PANEL
                                        } else if is_even {
                                            BG_APP
                                        } else {
                                            BG_ROW_ALT
                                        })
                                        .children(if is_changed {
                                            Some(left_indicator(WARN))
                                        } else {
                                            None
                                        })
                                        // 1. Primary Row Line
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .gap(px(16.0))
                                                .px(px(14.0))
                                                .py(px(9.0))
                                                // Left info
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w(px(0.0))
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
                                                                        .text_size(px(12.0))
                                                                        .font_weight(FontWeight::MEDIUM)
                                                                        .text_color(if is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                                                                        .child(label),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .children(if is_changed {
                                                                            Some(
                                                                                div()
                                                                                    .px(px(4.0))
                                                                                    .py(px(1.5))
                                                                                    .bg(WARN_BG)
                                                                                    .text_color(WARN)
                                                                                    .font_family(FONT_MONO)
                                                                                    .text_size(px(8.5))
                                                                                    .font_weight(FontWeight::BOLD)
                                                                                    .child("CHANGED"),
                                                                            )
                                                                        } else {
                                                                            None
                                                                        }),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .font_family(FONT_MONO)
                                                                .text_size(px(10.5))
                                                                .text_color(TEXT_DIM)
                                                                .line_height(relative(1.45))
                                                                .child(desc),
                                                        )
                                                        .child(
                                                            div()
                                                                .font_family(FONT_MONO)
                                                                .text_size(px(9.5))
                                                                .text_color(TEXT_FAINTER)
                                                                .child(key_repr),
                                                        ),
                                                )
                                                // Right control
                                                .child(
                                                    div()
                                                        .flex_none()
                                                        .flex()
                                                        .items_center()
                                                        .justify_end()
                                                        .children(if is_bool {
                                                            let curr_val = field.and_then(|f| f.value.as_bool()).unwrap_or(false);
                                                            let next_val = !curr_val;
                                                            let r_id = row_id_str.clone();

                                                            Some(
                                                                div()
                                                                    .id(ElementId::NamedInteger("ctl-bool".into(), idx as u64))
                                                                    .flex()
                                                                    .border_1()
                                                                    .border_color(if is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                                                                    .cursor_pointer()
                                                                    .hover(|s| s.border_color(TEXT_SECONDARY))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        let target = r_id.clone();
                                                                        app_toggle.update(cx, |this, cx| {
                                                                            this.update_config_field(&target, serde_json::Value::Bool(next_val), cx);
                                                                        });
                                                                    })
                                                                    .child(
                                                                        div()
                                                                            .px(px(9.0))
                                                                            .py(px(3.0))
                                                                            .font_family(FONT_MONO)
                                                                            .text_size(px(10.0))
                                                                            .font_weight(FontWeight::BOLD)
                                                                            .border_r_1()
                                                                            .border_color(BORDER_DEFAULT)
                                                                            .bg(if curr_val { OK_BG } else { hex_rgba(0, 0.0) })
                                                                            .text_color(if curr_val { OK } else { TEXT_FAINT })
                                                                            .child("ON"),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .px(px(9.0))
                                                                            .py(px(3.0))
                                                                            .font_family(FONT_MONO)
                                                                            .text_size(px(10.0))
                                                                            .font_weight(FontWeight::BOLD)
                                                                            .bg(if !curr_val { BG_CHIP } else { hex_rgba(0, 0.0) })
                                                                            .text_color(if !curr_val { TEXT_DIM } else { TEXT_FAINT })
                                                                            .child("OFF"),
                                                                    ),
                                                            )
                                                        } else {
                                                            let r_id = row_id_str.clone();
                                                            let init_val = initial_val_for_open.clone();

                                                            Some(
                                                                div()
                                                                    .id(ElementId::NamedInteger("ctl-btn".into(), idx as u64))
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap(px(6.0))
                                                                    .px(px(9.0))
                                                                    .py(px(4.0))
                                                                    .bg(if is_open { hex_rgb(0x1a1a24) } else { BG_OVERLAY_PANEL })
                                                                    .border_1()
                                                                    .border_color(if is_open { TEXT_PRIMARY } else if is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                                                                    .cursor_pointer()
                                                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                                                    .on_click(move |_ev, _window, cx| {
                                                                        let target = r_id.clone();
                                                                        let val_snap = init_val.clone();
                                                                        app_btn.update(cx, |this, cx| {
                                                                            this.toggle_settings_dropdown(&target, &val_snap, cx);
                                                                        });
                                                                    })
                                                                    .child(
                                                                        div()
                                                                            .font_family(FONT_MONO)
                                                                            .text_size(px(11.0))
                                                                            .text_color(if is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                                                                            .child(display_val),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .font_family(FONT_MONO)
                                                                            .text_size(px(9.0))
                                                                            .text_color(if is_open { TEXT_PRIMARY } else { TEXT_FAINT })
                                                                            .child(if is_open { "▴" } else { "▾" }),
                                                                    ),
                                                            )
                                                        }),
                                                ),
                                        )
                                        // 2. Options Dropdown Tray (shown when is_open == true)
                                        .children(if is_open && !is_bool {
                                            Some(
                                                div()
                                                    .id(ElementId::NamedInteger("dropdown-tray".into(), idx as u64))
                                                    .mx(px(14.0))
                                                    .mb(px(10.0))
                                                    .p(px(10.0))
                                                    .bg(hex_rgb(0x070709))
                                                    .border_1()
                                                    .border_color(hex_rgb(0x272730))
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(6.0))
                                                    // Header
                                                    .child(
                                                        div()
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(9.5))
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(TEXT_DIMMER)
                                                            .child("SELECT PRESET OPTION:"),
                                                    )
                                                    // Presets list
                                                    .children(presets.into_iter().enumerate().map(|(opt_idx, opt)| {
                                                        let app_opt = app.clone();
                                                        let r_id = row_id_str.clone();
                                                        let opt_val_str = opt.value.clone();
                                                        let is_selected = opt_val_str == raw_val_str;

                                                        div()
                                                            .id(ElementId::NamedInteger("opt-item".into(), (idx * 100 + opt_idx) as u64))
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(8.0))
                                                            .px(px(8.0))
                                                            .py(px(4.0))
                                                            .bg(if is_selected { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                                                            .border_1()
                                                            .border_color(if is_selected { BORDER_DEFAULT } else { hex_rgba(0, 0.0) })
                                                            .cursor_pointer()
                                                            .hover(|s| s.bg(BG_ROW_HOVER))
                                                            .on_click(move |_ev, _window, cx| {
                                                                let target = r_id.clone();
                                                                let new_json_val = if is_int {
                                                                    serde_json::Value::Number(serde_json::Number::from(opt_val_str.parse::<i64>().unwrap_or(0)))
                                                                } else {
                                                                    serde_json::Value::String(opt_val_str.clone())
                                                                };
                                                                app_opt.update(cx, |this, cx| {
                                                                    this.update_config_field(&target, new_json_val, cx);
                                                                    this.close_settings_dropdown(cx);
                                                                });
                                                            })
                                                            // Radio indicator
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.0))
                                                                    .text_color(if is_selected { OK } else { TEXT_FAINT })
                                                                    .child(if is_selected { "●" } else { "○" }),
                                                            )
                                                            // Option label
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(11.0))
                                                                    .font_weight(if is_selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                                                    .text_color(if is_selected { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                                                    .child(opt.label),
                                                            )
                                                            // Option description
                                                            .child(
                                                                div()
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.0))
                                                                    .text_color(TEXT_DIMMER)
                                                                    .child(opt.desc),
                                                            )
                                                    }))
                                                    // Optional Custom Value Textbox
                                                    .children(if allow_custom {
                                                        let r_id = row_id_for_apply.clone();
                                                        let unit_str = unit_suffix.unwrap_or("").to_string();

                                                        Some(
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
                                                                 // Textbox display with active indicator
                                                                 .child(
                                                                     terminal_text_input_styled(
                                                                         ElementId::NamedInteger("custom-input-box".into(), idx as u64),
                                                                         custom_input,
                                                                         "value…",
                                                                         true,
                                                                         false,
                                                                         26.0,
                                                                         11.0,
                                                                         caret.cursor,
                                                                         caret.selection,
                                                                         caret.drag_anchor,
                                                                         caret.blink,
                                                                         {
                                                                             let app = app_custom;
                                                                             move |cursor, anchor, selection, _window, cx| {
                                                                                 app.update(cx, |this, cx| {
                                                                                     this.caret.cursor = cursor;
                                                                                     this.caret.drag_anchor = anchor;
                                                                                     this.caret.selection = selection;
                                                                                     this.caret.blink = true;
                                                                                     cx.notify();
                                                                                 });
                                                                             }
                                                                         },
                                                                     )
                                                                     .children(if !unit_str.is_empty() {
                                                                         Some(
                                                                             div()
                                                                                 .font_family(FONT_MONO)
                                                                                 .text_size(px(10.0))
                                                                                 .text_color(TEXT_FAINT)
                                                                                 .child(unit_str),
                                                                         )
                                                                     } else {
                                                                         None
                                                                     }),
                                                                 )
                                                                // Apply Button
                                                                .child(
                                                                    div()
                                                                        .id(ElementId::NamedInteger("btn-apply-custom".into(), idx as u64))
                                                                        .h(px(26.0))
                                                                        .px(px(10.0))
                                                                        .bg(BG_KEY)
                                                                        .border_1()
                                                                        .border_color(BORDER_DEFAULT)
                                                                        .flex()
                                                                        .items_center()
                                                                        .cursor_pointer()
                                                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                                                        .on_click(move |_ev, _window, cx| {
                                                                            let target = r_id.clone();
                                                                            app_apply.update(cx, |this, cx| {
                                                                                this.apply_settings_custom_input(&target, cx);
                                                                            });
                                                                        })
                                                                        .child(
                                                                            div()
                                                                                .font_family(FONT_MONO)
                                                                                .text_size(px(10.0))
                                                                                .font_weight(FontWeight::BOLD)
                                                                                .text_color(OK)
                                                                                .child("APPLY ↵"),
                                                                        ),
                                                                )
                                                                // Cancel Button
                                                                .child(
                                                                    div()
                                                                        .id(ElementId::NamedInteger("btn-cancel-custom".into(), idx as u64))
                                                                        .h(px(26.0))
                                                                        .px(px(8.0))
                                                                        .border_1()
                                                                        .border_color(BORDER_PANEL)
                                                                        .flex()
                                                                        .items_center()
                                                                        .cursor_pointer()
                                                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                                                        .on_click(move |_ev, _window, cx| {
                                                                            app_cancel.update(cx, |this, cx| {
                                                                                this.close_settings_dropdown(cx);
                                                                            });
                                                                        })
                                                                        .child(
                                                                            div()
                                                                                .font_family(FONT_MONO)
                                                                                .text_size(px(10.0))
                                                                                .text_color(TEXT_TERTIARY)
                                                                                .child("CANCEL esc"),
                                                                        ),
                                                                ),
                                                        )
                                                    } else {
                                                        None
                                                    }),
                                            )
                                        } else {
                                            None
                                        })
                                })),
                        )
                        // Section Bottom Actions Bar
                        .child(
                            div()
                                .h(px(42.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .px(px(14.0))
                                .bg(BG_PANEL)
                                .border_t_1()
                                .border_color(BORDER_PANEL)
                                .gap(px(8.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(
                                    div()
                                        .id("btn-reset-section")
                                        .px(px(9.0))
                                        .py(px(5.0))
                                        .border_1()
                                        .border_color(if has_section_changes { BORDER_KEY } else { BORDER_PANEL })
                                        .text_color(if has_section_changes { TEXT_PRIMARY } else { TEXT_FAINTER })
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
                        ),
                    )
                } else {
                    None
                })
                // Right Rail: Pending Diff, Session Warning, Keychain (340px)
                .children(if section != SettingsSection::Components && section != SettingsSection::Clankers {
                    Some(
                        div()
                        .w(px(340.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
                        .children(if section == SettingsSection::Keys {
                            Some(render_keys_right_rail(app.clone(), keys))
                        } else {
                            None
                        })
                        .children(if section != SettingsSection::Keys {
                            Some(
                                div()
                                    .size_full()
                                    .flex()
                                    .flex_col()
                                    // Pending Diff Header
                                    .child(
                                        div()
                                            .h(px(34.0))
                                            .flex_none()
                                            .flex()
                                            .items_center()
                                            .px(px(12.0))
                                            .bg(BG_PANEL)
                                            .border_b_1()
                                            .border_color(BORDER_PANEL)
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(11.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(TEXT_PRIMARY)
                                                    .child("PENDING DIFF"),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .text_color(if total_changed > 0 { WARN } else { TEXT_DIMMER })
                                                    .child(diff_badge),
                                            ),
                                    )
                        // Diff Snippets
                        .child(if diff_lines.is_empty() {
                            div()
                                .id("pending-diff-empty")
                                .flex_none()
                                .p(px(14.0))
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .line_height(relative(1.5))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .text_color(OK)
                                        .child(inherited_icon(TablerIcon::Check, px(12.0)))
                                        .child("Configuration in sync"),
                                )
                                .child(
                                    div()
                                        .mt(px(4.0))
                                        .text_color(TEXT_FAINTER)
                                        .child("Working copy matches ~/.config/crow/config.toml"),
                                )
                                .into_any_element()
                        } else {
                            div()
                                .id("pending-diff-scroll")
                                .flex_none()
                                .max_h(px(280.0))
                                .overflow_y_scrollbar()
                                .py(px(6.0))
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .line_height(relative(1.55))
                                .children(diff_lines.into_iter().map(|line| {
                                    let (bg_c, text_c) = match line.kind {
                                        DiffKind::Hunk => (DIFF_HUNK_BG, TEXT_DIMMER),
                                        DiffKind::Addition => (DIFF_ADD_BG, OK_INK),
                                        DiffKind::Deletion => (DIFF_DEL_BG, CRIT_INK),
                                        DiffKind::Context => (hex_rgba(0, 0.0), TEXT_MUTED),
                                    };
                                    div()
                                        .px(px(12.0))
                                        .py(px(1.5))
                                        .bg(bg_c)
                                        .text_color(text_c)
                                        .child(line.text)
                                }))
                                .into_any_element()
                        })
                        // Live Sessions Warning (shown if connection changes exist)
                        .children(if has_conn_changes {
                            Some(
                                div()
                                    .flex_none()
                                    .p(px(12.0))
                                    .border_b_1()
                                    .border_color(BORDER_PANEL)
                                    .bg(hex_rgb(0x0c0a0a))
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(7.0))
                                            .child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(WARN).child("▲"))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(WARN)
                                                    .child("AFFECTS LIVE SESSIONS"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.5))
                                            .line_height(relative(1.5))
                                            .text_color(TEXT_TERTIARY)
                                            .child("Pending SSH connection changes take effect upon reconnection. Existing active sessions will retain their current parameters."),
                                    ),
                            )
                        } else {
                            None
                        })
                        // Keychain Section
                        .child(
                            div()
                                .flex_1()
                                .p(px(12.0))
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_DIMMER)
                                        .child("KEYCHAIN"),
                                )
                                .children(keychain.is_empty().then(|| div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_FAINT).child("No keys enrolled yet.")))
                                .children(keychain.iter().map(|(name, note, dot)| {
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(9.0))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .child(
                                            div()
                                                .size(px(6.0))
                                                .rounded_full()
                                                .bg(*dot)
                                                .flex_none(),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_color(TEXT_SECONDARY)
                                                .child(name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_color(TEXT_DIMMER)
                                                .child(note.clone()),
                                        )
                                })),
                        ),
                    )
                } else {
                    None
                }),
                        )
                } else {
                    None
                }),
        )
        .children(render_key_modals(app.clone(), caret, fleet, keys))
        .children(clankers::render_clanker_modals(app.clone(), caret, clankers))
}

/// Fleet page background: preview, choose, remove. Opacity and blur are the
/// ordinary setting rows below it.
fn personalisation_block(bg: &(&crate::app::appearance::FleetBackground, Option<std::path::PathBuf>), app: Entity<CrowApp>) -> impl IntoElement {
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
            // Preview, as drawn (blurred) when ready.
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
