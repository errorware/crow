use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use crate::keys::copy_to_clipboard_system;

pub fn about_modal(app: Entity<CrowApp>, copied_toast: bool) -> impl IntoElement {
    let app_close1 = app.clone();
    let app_close2 = app.clone();
    let app_copy = app.clone();

    let os_str = std::env::consts::OS;
    let arch_str = std::env::consts::ARCH;
    let profile_str = if cfg!(debug_assertions) { "Debug (Dev)" } else { "Release" };
    let version_str = env!("CARGO_PKG_VERSION");
    let authors_str = "Nelson <nelson@errorware.net>";
    let website_str = "https://errorware.net";
    let repo_str = "https://github.com/errorware/crow";

    let system_report = format!(
        "Crow Server Manager\nVersion: v{}\nPlatform: {} ({})\nProfile: {}\nAuthor: {}\nWebsite: {}\nRepository: {}\nEngine: GPUI Kit (WGPU) + crow-config-core\nVault: Argon2id + ChaCha20-Poly1305 + TOTP\n",
        version_str, os_str, arch_str, profile_str, authors_str, website_str, repo_str
    );

    div()
        .id("about-modal-scrim")
        .absolute()
        .inset_0()
        .occlude()
        .bg(hex_rgba(0x050507, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_close1.update(cx, |this, cx| {
                this.close_about_modal(cx);
            });
        })
        .child(
            div()
                .id("about-modal-panel")
                .occlude()
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // keep clicks inside from reaching the backdrop (which closes)
                .w(px(520.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .shadow_lg()
                .flex()
                .flex_col()
                // 1. Header Bar
                .child(
                    div()
                        .h(px(38.0))
                        .bg(BG_PANEL)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(14.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(tabler_icon(TablerIcon::InfoCircle).size(px(14.0)).text_color(OK))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(11.5))
                                        .text_color(TEXT_MAX)
                                        .child("ABOUT CROW"),
                                ),
                        )
                        .child(
                            div()
                                .id("about-close-btn")
                                .size(px(24.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_close2.update(cx, |this, cx| {
                                        this.close_about_modal(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::X).size(px(13.0)).text_color(TEXT_MUTED)),
                        ),
                )
                // 2. Banner & Brand Identity
                .child(
                    div()
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(18.0))
                        // Title row
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(14.0))
                                .child(
                                    div()
                                        .size(px(46.0))
                                        .bg(hex_rgb(0x10131a))
                                        .border_1()
                                        .border_color(BORDER_STRONG)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(tabler_icon(TablerIcon::Server).size(px(24.0)).text_color(OK)),
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
                                                    div()
                                                        .font_family(FONT_MONO)
                                                        .font_weight(FontWeight::EXTRA_BOLD)
                                                        .text_size(px(18.0))
                                                        .text_color(TEXT_MAX)
                                                        .child("CROW"),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.5))
                                                        .bg(hex_rgb(0x131d16))
                                                        .border_1()
                                                        .border_color(OK)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(OK)
                                                        .child(format!("v{}", version_str)),
                                                )
                                                .child(
                                                    div()
                                                        .px(px(6.0))
                                                        .py(px(1.5))
                                                        .bg(BG_CHIP)
                                                        .font_family(FONT_MONO)
                                                        .text_size(px(10.0))
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(TEXT_DIMMER)
                                                        .child("OBSIDIAN EDGE"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(11.0))
                                                .text_color(TEXT_SECONDARY)
                                                .child("Modern Linux Server Management & Telemetry Hub"),
                                        ),
                                ),
                        )
                        // Specs Grid
                        .child(
                            div()
                                .bg(hex_rgb(0x0a0a0d))
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .p(px(12.0))
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(info_row("PLATFORM", &format!("{} ({})", os_str, arch_str)))
                                .child(info_row("BUILD TARGET", "x86_64-unknown-linux-gnu"))
                                .child(info_row("BUILD PROFILE", profile_str))
                                .child(info_row("UI TOOLKIT", "GPUI Kit v0.6.1 · WGPU Native"))
                                .child(info_row("SECURITY VAULT", "Argon2id · ChaCha20-Poly1305 · TOTP"))
                                .child(info_row("CONFIG CORE", "crow-config-core · Schema IR")),
                        )
                        // Author & Website metadata
                        .child(
                            div()
                                .bg(hex_rgb(0x0a0a0d))
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .p(px(12.0))
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(info_row("AUTHOR", authors_str))
                                .child(info_row("ORGANIZATION", "Errorware"))
                                .child(info_row("DOCS WEBSITE", website_str))
                                .child(info_row("SOURCE CODE", repo_str)),
                        ),
                )
                // 3. Footer Actions
                .child(
                    div()
                        .h(px(46.0))
                        .bg(BG_PANEL)
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .px(px(16.0))
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .id("copy-sysinfo-btn")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(10.0))
                                .py(px(5.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(if copied_toast { OK } else { BORDER_DEFAULT })
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_CONTROL_ALT))
                                .on_click(move |_ev, _window, cx| {
                                    copy_to_clipboard_system(&system_report);
                                    app_copy.update(cx, |this, cx| {
                                        this.about_copied_toast = true;
                                        cx.notify();
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Copy).size(px(12.0)).text_color(if copied_toast { OK } else { TEXT_SECONDARY }))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(10.5))
                                        .text_color(if copied_toast { OK } else { TEXT_PRIMARY })
                                        .child(if copied_toast { "✓ COPIED SYSTEM INFO" } else { "COPY SYSTEM INFO" }),
                                ),
                        )
                        .child(
                            div()
                                .id("about-dismiss-btn")
                                .px(px(16.0))
                                .py(px(5.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_STRONG)
                                .font_family(FONT_MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .text_color(TEXT_MAX)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click({
                                    let app_close = app.clone();
                                    move |_ev, _window, cx| {
                                        app_close.update(cx, |this, cx| {
                                            this.close_about_modal(cx);
                                        });
                                    }
                                })
                                .child("CLOSE"),
                        ),
                ),
        )
}

fn info_row(label: &'static str, value: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_MUTED)
                .child(label),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_PRIMARY)
                .child(value.to_string()),
        )
}
