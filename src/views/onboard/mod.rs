use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};

pub struct FormField {
    pub label: &'static str,
    #[allow(dead_code)]
    pub key: &'static str,
    pub value: &'static str,
    pub chev: &'static str,
    pub width: f32,
    pub hint: &'static str,
    pub is_changed: bool,
}

pub fn onboard_view(app: Entity<CrowApp>) -> impl IntoElement {
    let steps = [
        ("✓", "Address", true, false),
        ("✓", "Credentials", true, false),
        ("3", "Verify Host", false, true),
        ("4", "Classify", false, false),
        ("5", "Install Agent", false, false),
    ];

    let fields = [
        FormField { label: "Hostname or IP", key: "host", value: "10.0.4.32", chev: "", width: 160.0, hint: "resolves · 1 A record", is_changed: true },
        FormField { label: "Port", key: "port", value: "22", chev: "", width: 70.0, hint: "open · banner SSH-2.0-OpenSSH_9.6p1", is_changed: false },
        FormField { label: "Login user", key: "user", value: "root", chev: "", width: 120.0, hint: "key auth accepted", is_changed: false },
        FormField { label: "Auth method", key: "auth", value: "publickey", chev: "▾", width: 140.0, hint: "password auth refused by host", is_changed: false },
        FormField { label: "Identity", key: "identity_file", value: "id_ed25519_fleet", chev: "▾", width: 190.0, hint: "~/.ssh · SHA256:kQ2…8Lm", is_changed: false },
        FormField { label: "Jump host", key: "proxy_jump", value: "bastion", chev: "▾", width: 130.0, hint: "159.223.84.9 · reachable", is_changed: false },
        FormField { label: "Label", key: "label", value: "worker-05", chev: "", width: 160.0, hint: "must be unique in fleet", is_changed: false },
        FormField { label: "Environment", key: "env", value: "PROD", chev: "▾", width: 110.0, hint: "enables typed confirmations", is_changed: false },
        FormField { label: "Role", key: "role", value: "sidekiq worker", chev: "▾", width: 170.0, hint: "picks the service template", is_changed: false },
        FormField { label: "Group", key: "group", value: "workers", chev: "▾", width: 140.0, hint: "inherits group policy & baseline", is_changed: false },
        FormField { label: "Tags", key: "tags", value: "queue, ruby, eu-west", chev: "", width: 230.0, hint: "free-form · used by fleet filters", is_changed: false },
    ];

    let probe_logs = [
        ("03:41:02", "✓", OK, "tcp connect 10.0.4.32:22", "8ms"),
        ("03:41:02", "✓", OK, "ssh banner SSH-2.0-OpenSSH_9.6p1 Ubuntu-3", ""),
        ("03:41:03", "✓", OK, "kex curve25519-sha256 · cipher chacha20-poly1305", ""),
        ("03:41:03", "✓", OK, "publickey id_ed25519_fleet accepted", ""),
        ("03:41:04", "✓", OK, "sudo -n true — passwordless escalation ok", ""),
        ("03:41:05", "✓", OK, "uname -a · os-release · lscpu · df -h", "read-only"),
        ("03:41:06", "✓", OK, "detected daemons: postgres, redis, docker, ufw", ""),
        ("03:41:07", "▲", WARN, "host key not in known_hosts — waiting on you", ""),
        ("", "○", TEXT_FAINT, "install crow-agent 0.9.4 (deferred to step 5)", "1.8 MB"),
    ];

    let facts = [
        ("DISTRO", "Ubuntu 24.04.1 LTS", TEXT_PRIMARY),
        ("KERNEL", "6.8.0-45-generic", TEXT_SECONDARY),
        ("ARCH", "x86_64 · 4 vCPU", TEXT_SECONDARY),
        ("MEMORY", "8.0 GB", TEXT_SECONDARY),
        ("DISK", "160 GB nvme · 31% used", TEXT_SECONDARY),
        ("INIT", "systemd 255", TEXT_SECONDARY),
        ("OPEN PORTS", "22, 6379, 9100", TEXT_SECONDARY),
        ("FIREWALL", "ufw active · 6 rules", OK),
        ("TIME", "chrony · offset 0.4ms", TEXT_SECONDARY),
        ("EXISTING AGENT", "none found", WARN),
    ];

    let packs = [
        ("openssh 9.6", OK, OK_BG),
        ("postgres 16", OK, OK_BG),
        ("redis 7.2", OK, OK_BG),
        ("ufw 0.36", OK, OK_BG),
        ("systemd 255", OK, OK_BG),
        ("docker 27.1", WARN, WARN_BG),
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
                            app_close.update(cx, |this, cx| {
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
                .children(steps.iter().enumerate().map(|(_idx, (num, label, done, active))| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(16.0))
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .bg(if *active { BG_NAV_ACTIVE } else { hex_rgba(0, 0.0) })
                        .border_b_2()
                        .border_color(if *active { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
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
                                .bg(if *done { OK_BG } else if *active { BG_CONTROL_ALT } else { hex_rgba(0, 0.0) })
                                .text_color(if *done { OK } else if *active { TEXT_MAX } else { TEXT_FAINT })
                                .child(*num),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .font_weight(if *active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .text_color(if *active { TEXT_MAX } else if *done { TEXT_PRIMARY } else { TEXT_DIMMER })
                                .child(*label),
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
                // Left Column: Form
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
                                .children(fields.into_iter().enumerate().map(|(idx, field)| {
                                    let is_even = idx % 2 == 0;

                                    div()
                                        .id(ElementId::NamedInteger("form-field".into(), idx as u64))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(18.0))
                                        .py(px(9.0))
                                        .border_b_1()
                                        .border_color(BORDER_ROW)
                                        .bg(if is_even { BG_APP } else { BG_ROW_ALT })
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .gap(px(2.0))
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(12.0))
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(TEXT_PRIMARY)
                                                        .child(field.label),
                                                )
                                                .child(
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .text_color(TEXT_DIMMER)
                                                        .child(field.hint),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex_none()
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .w(px(field.width))
                                                .px(px(8.0))
                                                .py(px(5.0))
                                                .bg(BG_OVERLAY_PANEL)
                                                .border_1()
                                                .border_color(if field.is_changed { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.0))
                                                .text_color(if field.is_changed { TEXT_MAX } else { TEXT_PRIMARY })
                                                .child(div().flex_1().child(field.value))
                                                .children(if !field.chev.is_empty() {
                                                    Some(div().text_color(TEXT_FAINT).child(field.chev))
                                                } else {
                                                    None
                                                }),
                                        )
                                })),
                        )
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
                                        .px(px(10.0))
                                        .py(px(6.0))
                                        .border_1()
                                        .border_color(BORDER_KEY)
                                        .text_color(TEXT_TERTIARY)
                                        .child("← BACK"),
                                )
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .px(px(14.0))
                                        .py(px(7.0))
                                        .bg(OK)
                                        .text_color(BG_WINDOW)
                                        .font_weight(FontWeight::BOLD)
                                        .child("PROCEED TO CLASSIFY ⌘⏎"),
                                ),
                        ),
                )
                // Right Rail: Fingerprint, Probe Log, Detected Facts, Schema Packs
                .child(
                    div()
                        .w(px(420.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .bg(BG_RAIL)
                        .border_l_1()
                        .border_color(BORDER_PANEL)
                        // Amber Fingerprint Card
                        .child(
                            div()
                                .p(px(14.0))
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .bg(hex_rgb(0x100c06))
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
                                                .child("UNKNOWN HOST KEY FINGERPRINT"),
                                        ),
                                )
                                .child(
                                    div()
                                        .p(px(8.0))
                                        .bg(BG_PANEL)
                                        .border_1()
                                        .border_color(hex_rgb(0x2e2210))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .text_color(TEXT_PRIMARY)
                                        .child("SHA256:4a8b...7f21 (ED25519)"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .pt(px(2.0))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .child(
                                            div()
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .border_1()
                                                .border_color(BORDER_DEFAULT)
                                                .text_color(TEXT_SECONDARY)
                                                .child("COMPARE KNOWN_HOSTS"),
                                        )
                                        .child(
                                            div()
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .border_1()
                                                .border_color(BORDER_DANGER_BTN)
                                                .text_color(CRIT)
                                                .child("REJECT"),
                                        ),
                                ),
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
                                .h(px(140.0))
                                .overflow_y_scroll()
                                .p(px(8.0))
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .children(probe_logs.iter().map(|(ts, glyph, c, msg, note)| {
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(7.0))
                                        .py(px(1.5))
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.0))
                                        .child(div().w(px(10.0)).text_color(*c).child(*glyph))
                                        .child(div().flex_1().text_color(TEXT_MUTED).child(*msg))
                                        .children(if !ts.is_empty() {
                                            Some(div().text_color(TEXT_FAINTER).child(*ts))
                                        } else {
                                            None
                                        })
                                        .children(if !note.is_empty() {
                                            Some(div().text_color(TEXT_FAINT).child(*note))
                                        } else {
                                            None
                                        })
                                })),
                        )
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
                                .children(facts.iter().map(|(k, v, c)| {
                                    div()
                                        .flex()
                                        .justify_between()
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .child(div().text_color(TEXT_DIMMER).child(*k))
                                        .child(div().text_color(*c).font_weight(FontWeight::MEDIUM).child(*v))
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
                                .children(packs.iter().map(|(name, fg, bg)| {
                                    div()
                                        .px(px(6.0))
                                        .py(px(2.5))
                                        .bg(*bg)
                                        .text_color(*fg)
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::BOLD)
                                        .child(*name)
                                })),
                        ),
                ),
        )
}
