use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen, SettingsSection};
use crate::config::{CrowConfigManager, DiffKind};
use crate::components::icons::{TablerIcon, tabler_icon};
use crow_config_core::schema::FieldType;

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

fn format_int_val(leaf: &str, n: i64) -> String {
    match leaf {
        "keepalive_interval" | "connect_timeout" | "refresh_interval" => format!("{} s", n),
        "auto_lock_minutes" => format!("{} min", n),
        "auto_rotate_days" => format!("{} days", n),
        "font_size" => format!("{} px", n),
        "log_buffer_lines" => format!("{} lines", n),
        _ => format!("{}", n),
    }
}

fn next_int_preset(leaf: &str, curr: i64) -> i64 {
    let presets: &[i64] = match leaf {
        "connect_timeout" => &[5, 10, 15, 30, 60],
        "keepalive_interval" => &[10, 15, 30, 60],
        "auto_lock_minutes" => &[5, 15, 30, 60],
        "refresh_interval" => &[1, 2, 5, 10],
        "font_size" => &[11, 12, 13, 14, 15],
        "log_buffer_lines" => &[500, 1000, 2000, 5000],
        "auto_rotate_days" => &[30, 60, 90, 180, 365],
        _ => &[1, 5, 10, 20],
    };
    let curr_idx = presets.iter().position(|&p| p == curr).unwrap_or(0);
    presets[(curr_idx + 1) % presets.len()]
}

fn next_str_preset(leaf: &str, curr: &str) -> Option<String> {
    let presets: &[&str] = match leaf {
        "font_family" => &["JetBrains Mono, monospace", "Fira Code, monospace", "SF Mono, monospace"],
        "ciphers" => &[
            "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com",
            "aes256-gcm@openssh.com",
            "chacha20-poly1305@openssh.com",
        ],
        "default_identity" => &["~/.ssh/id_ed25519", "~/.ssh/id_ed25519_bastion", "~/.ssh/id_rsa"],
        _ => return None,
    };
    let curr_idx = presets.iter().position(|&p| p == curr).unwrap_or(0);
    Some(presets[(curr_idx + 1) % presets.len()].to_string())
}

fn render_setting_control(
    app: Entity<CrowApp>,
    idx: usize,
    row_id: &str,
    leaf: &str,
    field: Option<&crow_config_core::ir::FieldIr>,
    is_changed: bool,
) -> impl IntoElement {
    let target_row_id = row_id.to_string();
    let leaf_str = leaf.to_string();

    match field.map(|f| &f.field_type) {
        Some(FieldType::Bool) => {
            let curr_val = field.and_then(|f| f.value.as_bool()).unwrap_or(false);
            let next_val = !curr_val;
            let app_toggle = app.clone();
            let row_id_clone = target_row_id.clone();

            div()
                .id(ElementId::NamedInteger("ctl-bool".into(), idx as u64))
                .flex()
                .border_1()
                .border_color(if is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                .cursor_pointer()
                .hover(|s| s.border_color(TEXT_SECONDARY))
                .on_click(move |_ev, _window, cx| {
                    let r_id = row_id_clone.clone();
                    app_toggle.update(cx, |this, cx| {
                        this.update_config_field(&r_id, serde_json::Value::Bool(next_val), cx);
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
                )
        }
        Some(FieldType::Enum) => {
            let curr_str = field.and_then(|f| f.value.as_str()).unwrap_or("");
            let options = field.and_then(|f| f.options.clone()).unwrap_or_default();
            let app_enum = app.clone();
            let row_id_clone = target_row_id.clone();

            let next_val = if !options.is_empty() {
                let curr_idx = options.iter().position(|o| o.value == curr_str).unwrap_or(0);
                let next_idx = (curr_idx + 1) % options.len();
                options[next_idx].value.clone()
            } else {
                curr_str.to_string()
            };

            div()
                .id(ElementId::NamedInteger("ctl-enum".into(), idx as u64))
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(9.0))
                .py(px(4.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(if is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    let r_id = row_id_clone.clone();
                    let n_val = next_val.clone();
                    app_enum.update(cx, |this, cx| {
                        this.update_config_field(&r_id, serde_json::Value::String(n_val), cx);
                    });
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(if is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                        .child(curr_str.to_string()),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_FAINT)
                        .child("▾"),
                )
        }
        Some(FieldType::Other(cow)) if cow == "integer" => {
            let curr_num = field.and_then(|f| f.value.as_i64()).unwrap_or(0);
            let display_str = format_int_val(&leaf_str, curr_num);
            let next_val = next_int_preset(&leaf_str, curr_num);
            let app_num = app.clone();
            let row_id_clone = target_row_id.clone();

            div()
                .id(ElementId::NamedInteger("ctl-int".into(), idx as u64))
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(9.0))
                .py(px(4.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(if is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    let r_id = row_id_clone.clone();
                    app_num.update(cx, |this, cx| {
                        this.update_config_field(&r_id, serde_json::Value::Number(serde_json::Number::from(next_val)), cx);
                    });
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(if is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                        .child(display_str),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .text_color(TEXT_FAINT)
                        .child("▾"),
                )
        }
        _ => {
            let curr_str = field.and_then(|f| f.value.as_str()).unwrap_or("");
            let maybe_next = next_str_preset(&leaf_str, curr_str);
            let app_str = app.clone();
            let row_id_clone = target_row_id.clone();
            let has_preset = maybe_next.is_some();

            div()
                .id(ElementId::NamedInteger("ctl-str".into(), idx as u64))
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(9.0))
                .py(px(4.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(if is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                .children(if has_preset {
                    Some(div().cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)))
                } else {
                    None
                })
                .on_click(move |_ev, _window, cx| {
                    if let Some(next_str) = &maybe_next {
                        let r_id = row_id_clone.clone();
                        let n_val = next_str.clone();
                        app_str.update(cx, |this, cx| {
                            this.update_config_field(&r_id, serde_json::Value::String(n_val), cx);
                        });
                    }
                })
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(if is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                        .child(curr_str.to_string()),
                )
                .children(if has_preset {
                    Some(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(9.0))
                            .text_color(TEXT_FAINT)
                            .child("▾")
                    )
                } else {
                    None
                })
        }
    }
}

pub fn settings_view(
    app: Entity<CrowApp>,
    config: &CrowConfigManager,
    section: SettingsSection,
    is_auth_enabled: bool,
) -> impl IntoElement {
    let nav_items = [
        (TablerIcon::AdjustmentsHorizontal, "General", SettingsSection::General),
        (TablerIcon::Network, "Connection & SSH", SettingsSection::Connection),
        (TablerIcon::Key, "Keys & Rotation", SettingsSection::Keys),
        (TablerIcon::ShieldCheck, "Vault & Security", SettingsSection::Security),
    ];

    let (title, sub) = match section {
        SettingsSection::General => ("GENERAL", "[general] · application behavior"),
        SettingsSection::Connection => ("CONNECTION & SSH", "[connection] · applies to every host unless overridden"),
        SettingsSection::Keys => ("KEYS & ROTATION", "[keys] · key distribution & policies"),
        SettingsSection::Security => ("VAULT & SECURITY", "[vault] · local encrypted sqlite & master key"),
    };

    let keychain = [
        ("id_ed25519_fleet (default)", "enrolled 11 hosts", OK),
        ("id_ed25519_bastion", "jump host only", OK),
        ("id_rsa_legacy", "4096-bit · retire", WARN),
        ("yubikey-5c (sk-ed25519)", "not present", TEXT_FAINT),
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

    let sec_prefix = format!("{}.", section.id_prefix());
    let sec_rows: Vec<_> = config
        .ir
        .rows
        .iter()
        .filter(|r| r.row_id.starts_with(&sec_prefix))
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
                .child(
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
                                .overflow_y_scroll()
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

                                    let raw_val_str = match field.map(|f| &f.value) {
                                        Some(serde_json::Value::Bool(b)) => format!("{}", b),
                                        Some(serde_json::Value::Number(n)) => format!("{}", n),
                                        Some(serde_json::Value::String(s)) => format!("\"{}\"", s),
                                        _ => String::new(),
                                    };
                                    let key_repr = format!("{} = {}", leaf, raw_val_str);

                                    div()
                                        .id(ElementId::NamedInteger("setting-row".into(), idx as u64))
                                        .relative()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(16.0))
                                        .px(px(14.0))
                                        .py(px(9.0))
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .bg(if is_changed {
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
                                                .child(render_setting_control(app.clone(), idx, row.row_id.as_str(), leaf, field, is_changed)),
                                        )
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
                // Right Rail: Pending Diff, Session Warning, Keychain (340px)
                .child(
                    div()
                        .w(px(340.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
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
                                        .child(tabler_icon(TablerIcon::Check).size(px(12.0)))
                                        .child("Configuration in sync"),
                                )
                                .child(
                                    div()
                                        .mt(px(4.0))
                                        .text_color(TEXT_FAINTER)
                                        .child("Working copy matches ~/.config/crow/config.toml"),
                                )
                        } else {
                            div()
                                .id("pending-diff-scroll")
                                .flex_none()
                                .max_h(px(280.0))
                                .overflow_y_scroll()
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
                                                .child(*name),
                                        )
                                        .child(
                                            div()
                                                .text_color(TEXT_DIMMER)
                                                .child(*note),
                                        )
                                })),
                        ),
                ),
        )
}
