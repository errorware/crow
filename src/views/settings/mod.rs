use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen, SettingsSection};

pub struct SettingRow {
    pub label: &'static str,
    pub desc: &'static str,
    pub key: &'static str,
    pub is_toggle: bool,
    pub toggle_val: bool,
    pub val_str: &'static str,
    pub has_chevron: bool,
    pub is_changed: bool,
}

pub fn settings_view(app: Entity<CrowApp>, section: SettingsSection) -> impl IntoElement {
    let nav_items = [
        ("◈", "General", "", SettingsSection::General, None),
        ("⇄", "Connection & SSH", "3", SettingsSection::Connection, Some(WARN)),
        ("⚿", "Keys & Rotation", "", SettingsSection::Keys, None),
        ("◷", "Telemetry", "", SettingsSection::General, None),
        ("⌗", "Logs & Retention", "", SettingsSection::General, None),
        ("◉", "Notifications", "", SettingsSection::General, None),
        ("◧", "Schema Packs", "4", SettingsSection::General, Some(OK)),
        ("▲", "Danger Defaults", "", SettingsSection::General, None),
        ("⌨", "Keymap", "", SettingsSection::General, None),
        ("🛡", "Vault & Security", "L", SettingsSection::Security, Some(OK)),
        ("⬡", "About", "", SettingsSection::General, None),
    ];

    let connection_rows = [
        SettingRow {
            label: "Strict host key checking",
            desc: "Refuse to connect when a host key changes. Unknown keys must be verified manually on first contact.",
            key: "strict_host_key_checking = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: true,
        },
        SettingRow {
            label: "SSH agent forwarding",
            desc: "Forward your local agent into sessions. Convenient, and a lateral-movement risk on shared hosts.",
            key: "agent_forwarding = false",
            is_toggle: true,
            toggle_val: false,
            val_str: "",
            has_chevron: false,
            is_changed: true,
        },
        SettingRow {
            label: "Connection multiplexing",
            desc: "Reuse one TCP connection per host for all panels. Reduces handshakes from ~14 to 1.",
            key: "control_master = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Keepalive interval",
            desc: "Seconds between keepalive probes. Lower detects dead hosts faster, costs more chatter.",
            key: "keepalive_interval = 15",
            is_toggle: false,
            toggle_val: false,
            val_str: "15 s",
            has_chevron: false,
            is_changed: true,
        },
        SettingRow {
            label: "Connect timeout",
            desc: "Give up on a TCP handshake after this long.",
            key: "connect_timeout = 10",
            is_toggle: false,
            toggle_val: false,
            val_str: "10 s",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Reconnect backoff",
            desc: "Delay curve between reconnect attempts after a drop.",
            key: "reconnect_backoff = exponential",
            is_toggle: false,
            toggle_val: false,
            val_str: "exponential",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Preferred ciphers",
            desc: "Restrict negotiation to modern AEAD ciphers only.",
            key: "ciphers = chacha20, aes256-gcm",
            is_toggle: false,
            toggle_val: false,
            val_str: "chacha20, aes256-gcm",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Jump host",
            desc: "Route all connections through a bastion by default.",
            key: "proxy_jump = bastion",
            is_toggle: false,
            toggle_val: false,
            val_str: "bastion",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Per-host SSH config",
            desc: "Read ~/.ssh/config and honour Host blocks, Match rules and IdentityFile.",
            key: "read_ssh_config = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Sudo escalation",
            desc: "How Crow elevates when an action needs root.",
            key: "become_method = sudo -n",
            is_toggle: false,
            toggle_val: false,
            val_str: "sudo -n",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Command echo in logs",
            desc: "Record every command Crow runs in the per-host audit log.",
            key: "echo_commands = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
    ];

    let general_rows = [
        SettingRow {
            label: "Open fleet overview on launch",
            desc: "Start on the pinned Fleet tab instead of the last server you had open.",
            key: "open_fleet_on_launch = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Restore tabs from last session",
            desc: "Reopen the server tabs you had, and reconnect them.",
            key: "restore_tabs = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Confirm before closing a tab with staged edits",
            desc: "Prevents losing unapplied config changes.",
            key: "confirm_close_dirty = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Telemetry refresh",
            desc: "How often stat strips and tables repoll.",
            key: "refresh_interval = 1 s",
            is_toggle: false,
            toggle_val: false,
            val_str: "1 s",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Numeric font",
            desc: "Font used for all monospace data.",
            key: "mono_font = JetBrains Mono",
            is_toggle: false,
            toggle_val: false,
            val_str: "JetBrains Mono",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Interface density",
            desc: "Row heights across every table.",
            key: "density = compact",
            is_toggle: false,
            toggle_val: false,
            val_str: "compact",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Menu bar icon",
            desc: "Keep a fleet health indicator in the system menu bar.",
            key: "menu_bar_icon = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Send anonymous crash reports",
            desc: "Stack traces only. Never host names, addresses, or config contents.",
            key: "crash_reports = false",
            is_toggle: true,
            toggle_val: false,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
    ];

    let security_rows = [
        SettingRow {
            label: "Local SQLite Vault",
            desc: "Zero-knowledge authenticated encrypted database at ~/.config/crow/crow.db.",
            key: "vault_storage = sqlite_chacha20poly1305",
            is_toggle: false,
            toggle_val: false,
            val_str: "active · encrypted",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Key Derivation Function",
            desc: "Argon2id (64 MB memory cost, 3 iterations, 4 lanes).",
            key: "kdf = argon2id",
            is_toggle: false,
            toggle_val: false,
            val_str: "argon2id (64MB)",
            has_chevron: false,
            is_changed: false,
        },
        SettingRow {
            label: "Two-factor authentication (2FA)",
            desc: "Require RFC 6238 TOTP 6-digit verification code on vault unlock.",
            key: "totp_required = dynamic",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: true,
        },
        SettingRow {
            label: "Auto-lock timeout",
            desc: "Lock vault and wipe decrypted keys from RAM after inactivity.",
            key: "auto_lock_minutes = 15",
            is_toggle: false,
            toggle_val: false,
            val_str: "15 min",
            has_chevron: true,
            is_changed: false,
        },
        SettingRow {
            label: "Wipe memory on lock",
            desc: "Zeroize MasterKey from memory immediately on lock or exit.",
            key: "zeroize_on_drop = true",
            is_toggle: true,
            toggle_val: true,
            val_str: "",
            has_chevron: false,
            is_changed: false,
        },
    ];

    let rows: &[SettingRow] = match section {
        SettingsSection::Connection => &connection_rows,
        SettingsSection::General => &general_rows,
        SettingsSection::Keys => &connection_rows[..3],
        SettingsSection::Security => &security_rows,
    };

    let title = match section {
        SettingsSection::Connection => "CONNECTION & SSH",
        SettingsSection::General => "GENERAL",
        SettingsSection::Keys => "KEYS & ROTATION",
        SettingsSection::Security => "VAULT & SECURITY",
    };

    let sub = match section {
        SettingsSection::Connection => "[connection] · applies to every host unless overridden",
        SettingsSection::General => "[general] · application behavior",
        SettingsSection::Keys => "[keys] · key distribution & policies",
        SettingsSection::Security => "[vault] · local encrypted sqlite & master key",
    };

    let keychain = [
        ("id_ed25519_fleet (default)", "enrolled 11 hosts", OK),
        ("id_ed25519_bastion", "jump host only", OK),
        ("id_rsa_legacy", "4096-bit · retire", WARN),
        ("yubikey-5c (sk-ed25519)", "not present", TEXT_FAINT),
    ];

    let app_close = app.clone();

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
                                .child("~/.config/crow/config.toml · 3 pending changes"),
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
                        .children(nav_items.into_iter().enumerate().map(|(idx, (icon, label, badge, sec, badge_c))| {
                            let app_nav = app.clone();
                            let is_active = sec == section && (
                                (sec == SettingsSection::Connection && label == "Connection & SSH")
                                || (sec == SettingsSection::General && label == "General")
                                || (sec == SettingsSection::Keys && label == "Keys & Rotation")
                                || (sec == SettingsSection::Security && label == "Vault & Security")
                            );

                            div()
                                .id(ElementId::NamedInteger("settings-nav".into(), idx as u64))
                                .relative()
                                .h(px(30.0))
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
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .w(px(14.0))
                                        .text_align(TextAlign::Center)
                                        .text_color(if is_active { TEXT_PRIMARY } else { TEXT_DIMMER })
                                        .child(icon),
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
                                        .child("overrides apply per-server in the server's Config view"),
                                ),
                        )
                        // Rows List
                        .child(
                            div()
                                .id("settings-rows-list")
                                .flex_1()
                                .overflow_y_scroll()
                                .children(rows.iter().enumerate().map(|(idx, row)| {
                                    let is_even = idx % 2 == 0;

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
                                        .bg(if row.is_changed {
                                            BG_OVERLAY_PANEL
                                        } else if is_even {
                                            BG_APP
                                        } else {
                                            BG_ROW_ALT
                                        })
                                        .children(if row.is_changed {
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
                                                                .text_color(if row.is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                                                                .child(row.label),
                                                        )
                                                        .child(
                                                            div()
                                                                .children(if row.is_changed {
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
                                                        .child(row.desc),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(9.5))
                                                        .text_color(TEXT_FAINTER)
                                                        .child(row.key),
                                                ),
                                        )
                                        // Right control
                                        .child(
                                            div()
                                                .flex_none()
                                                .flex()
                                                .items_center()
                                                .justify_end()
                                                .children(if row.is_toggle {
                                                    Some(
                                                        div()
                                                            .flex()
                                                            .border_1()
                                                            .border_color(BORDER_DEFAULT)
                                                            .child(
                                                                div()
                                                                    .px(px(9.0))
                                                                    .py(px(3.0))
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.0))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .border_r_1()
                                                                    .border_color(BORDER_DEFAULT)
                                                                    .bg(if row.toggle_val { OK_BG } else { hex_rgba(0, 0.0) })
                                                                    .text_color(if row.toggle_val { OK } else { TEXT_FAINT })
                                                                    .child("ON"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .px(px(9.0))
                                                                    .py(px(3.0))
                                                                    .font_family(FONT_MONO)
                                                                    .text_size(px(10.0))
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .bg(if !row.toggle_val { BG_CHIP } else { hex_rgba(0, 0.0) })
                                                                    .text_color(if !row.toggle_val { TEXT_DIM } else { TEXT_FAINT })
                                                                    .child("OFF"),
                                                            ),
                                                    )
                                                } else {
                                                    Some(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(6.0))
                                                            .px(px(9.0))
                                                            .py(px(4.0))
                                                            .bg(BG_OVERLAY_PANEL)
                                                            .border_1()
                                                            .border_color(if row.is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                                                            .font_family(FONT_MONO)
                                                            .text_size(px(11.0))
                                                            .text_color(if row.is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                                                            .child(row.val_str)
                                                            .children(if row.has_chevron {
                                                                Some(div().text_color(TEXT_FAINT).child("▾"))
                                                            } else {
                                                                None
                                                            }),
                                                    )
                                                }),
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
                                        .px(px(9.0))
                                        .py(px(5.0))
                                        .border_1()
                                        .border_color(BORDER_KEY)
                                        .text_color(TEXT_TERTIARY)
                                        .child("RESET SECTION"),
                                )
                                .child(
                                    div()
                                        .px(px(9.0))
                                        .py(px(5.0))
                                        .bg(BG_CONTROL)
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .text_color(TEXT_SECONDARY)
                                        .child("EDIT config.toml ⌘/"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .px(px(11.0))
                                        .py(px(6.0))
                                        .bg(OK)
                                        .text_color(BG_WINDOW)
                                        .font_weight(FontWeight::BOLD)
                                        .child("SAVE ⌘S"),
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
                                        .text_color(TEXT_DIMMER)
                                        .child("+3 −3"),
                                ),
                        )
                        // Diff Snippets
                        .child(
                            div()
                                .flex_none()
                                .py(px(8.0))
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .line_height(relative(1.55))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_HUNK_BG).text_color(TEXT_DIMMER).child("@@ [connection] @@"))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_DEL_BG).text_color(CRIT_INK).child("- keepalive_interval = 30"))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_ADD_BG).text_color(OK_INK).child("+ keepalive_interval = 15"))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_DEL_BG).text_color(CRIT_INK).child("- strict_host_key_checking = \"ask\""))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_ADD_BG).text_color(OK_INK).child("+ strict_host_key_checking = \"yes\""))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_DEL_BG).text_color(CRIT_INK).child("- agent_forwarding = true"))
                                .child(div().px(px(12.0)).py(px(1.0)).bg(DIFF_ADD_BG).text_color(OK_INK).child("+ agent_forwarding = false")),
                        )
                        // Live Sessions Warning
                        .child(
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
                                        .child("Disabling agent forwarding drops 2 open forwards on bastion. Existing SSH sessions are not renegotiated until reconnect."),
                                ),
                        )
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
