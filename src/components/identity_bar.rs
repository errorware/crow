use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::host::{connection_state, transport_kind, ConnectionState, TransportKind};
use crate::vault::ServerRecord;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::os_detect::{classify_distro_family, DistroFamily};

pub fn identity_bar(server: Option<&ServerRecord>, app: Entity<CrowApp>) -> impl IntoElement {
    let app_clone = app.clone();

    let server_name = server.map(|s| s.name.as_str()).unwrap_or("localhost");
    let endpoint_str = server
        .map(|s| format!("{}@{}:{}", s.login_user, s.host, s.port))
        .unwrap_or_else(|| "operator@127.0.0.1:22".to_string());

    // Live transport state: this machine needs no connection; SSH servers
    // report what their last command saw.
    let (status_color, status_text, status_detail): (Rgba, &str, Option<String>) = match server {
        None => (OK, "LOCAL", None),
        Some(s) => match transport_kind(s) {
            TransportKind::Local => (OK, "LOCAL", None),
            TransportKind::Container => (OK, "LAB CONTAINER", None),
            TransportKind::Ssh => match connection_state(&s.id) {
                None => (TEXT_FAINT, "CONNECTING…", None),
                Some(ConnectionState::Connected) => (OK, "CONNECTED", None),
                Some(state) => (CRIT, state.label(), state.detail().map(str::to_string)),
            },
        },
    };

    let distro_str = server
        .map(|s| {
            if s.os_distro.is_empty() {
                "LINUX".to_string()
            } else {
                s.os_distro.to_uppercase()
            }
        })
        .unwrap_or_else(|| "LINUX".to_string());
    let distro_family = server
        .map(|s| classify_distro_family(&s.os_distro))
        .unwrap_or(DistroFamily::Unknown);

    let env_str = server.map(|s| s.env.as_str()).unwrap_or("LOCAL");
    let (env_bg, env_fg) = match env_str {
        "PROD" => (CRIT, hex_rgb(0x0a0a0c)),
        "STAGE" => (WARN, hex_rgb(0x0a0a0c)),
        "DEV" => (OK, hex_rgb(0x0a0a0c)),
        _ => (BG_CHIP, TEXT_SECONDARY),
    };

    let role_str = server
        .map(|s| s.role.to_uppercase())
        .unwrap_or_else(|| "SYSTEM".to_string());

    let kernel_str = server
        .map(|s| {
            if s.os_kernel.is_empty() {
                String::new()
            } else {
                format!("KERNEL {}", s.os_kernel)
            }
        })
        .unwrap_or_default();

    div()
        .h(px(52.0))
        .flex_none()
        .flex()
        .items_center()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        // 1. Server identity
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .px(px(16.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(15.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child(server_name.to_string()),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(9.5))
                        .text_color(hex_rgb(0x22d3ee))
                        .child(endpoint_str),
                ),
        )
        // 2. Connection state
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .pr(px(16.0))
                .h_full()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .size(px(7.0))
                        .rounded_full()
                        .bg(status_color)
                        .flex_none(),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(status_color)
                        .child(status_text),
                )
                .children(status_detail.map(|d| {
                    div()
                        .max_w(px(320.0))
                        .overflow_hidden()
                        .font_family("JetBrains Mono")
                        .text_size(px(9.5))
                        .text_color(TEXT_FAINT)
                        .child(d)
                })),
        )
        // 3. Badge cluster
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(16.0))
                .h_full()
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .bg(BG_CHIP)
                        .text_color(TEXT_SECONDARY)
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child(distro_str),
                )
                .children(if !distro_family.is_supported() {
                    Some(
                        div()
                            .bg(hex_rgba(0xfbbf24, 0.12))
                            .border_1()
                            .border_color(WARN)
                            .text_color(WARN)
                            .font_family("JetBrains Mono")
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .px(px(6.0))
                            .py(px(2.5))
                            .child("⚠ UNVERIFIED DISTRO"),
                    )
                } else {
                    None
                })
                .child(
                    div()
                        .bg(env_bg)
                        .text_color(env_fg)
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child(env_str.to_string()),
                )
                .child(
                    div()
                        .bg(BG_CHIP)
                        .text_color(TEXT_SECONDARY)
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(3.0))
                        .child(role_str),
                )
                .children(if !kernel_str.is_empty() {
                    Some(
                        div()
                            .bg(BG_CHIP)
                            .text_color(TEXT_TERTIARY)
                            .font_family("JetBrains Mono")
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .px(px(6.0))
                            .py(px(3.0))
                            .child(kernel_str),
                    )
                } else {
                    None
                })
                .child(
                    div()
                        .bg(hex_rgb(0x14161b))
                        .border_1()
                        .border_color(hex_rgb(0x27272a))
                        .text_color(OK)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .px(px(6.0))
                        .py(px(2.5))
                        .child("TURBO (-24s LAG)"),
                ),
        )
        // 4. Spacer
        .child(div().flex_1())
        // 5. Command trigger
        .child(
            div()
                .id("cmd-palette-trigger")
                .flex()
                .items_center()
                .gap(px(7.0))
                .h(px(26.0))
                .px(px(8.0))
                .mr(px(12.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .bg(hex_rgb(0x0e0f13))
                .cursor_pointer()
                .hover(|s| s.border_color(BORDER_STRONG))
                .on_click(move |_ev, _window, cx| {
                    app_clone.update(cx, |this, cx| {
                        this.toggle_palette(cx);
                    });
                })
                .child(
                    tabler_icon(TablerIcon::Search)
                        .size(px(13.0))
                        .text_color(TEXT_FAINT),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_FAINT)
                        .child("Search"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_MUTED)
                        .bg(BG_KEY)
                        .border_1()
                        .border_color(BORDER_KEY)
                        .px(px(5.0))
                        .py(px(1.0))
                        .child("⌘K"),
                ),
        )
        // 6. Action group
        .child(
            div()
                .flex()
                .items_center()
                .mr(px(12.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(5.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .bg(BG_CONTROL)
                        .border_r_1()
                        .border_color(BORDER_DEFAULT)
                        .child("SNAPSHOT"),
                )
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(5.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .child("RECONNECT"),
                ),
        )
}
