//! The About box (ERR-161): old-school and wide. The crow down the left,
//! the app's name and build in a header, where to find the project, and
//! the details in a dark scroll box.

use std::sync::{Arc, OnceLock};

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::app::{CrowApp, UpdateState};
use crate::components::icons::{TablerIcon, tabler_icon};

const REPO: &str = "https://github.com/errorware/crow";
const WEBSITE: &str = "https://crow.rs";
const FORUM: &str = "https://forum.errorware.net";

fn png(cell: &'static OnceLock<Arc<Image>>, bytes: &'static [u8]) -> Arc<Image> {
    cell.get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Png, bytes.to_vec()))).clone()
}

fn side_picture() -> Arc<Image> {
    static CELL: OnceLock<Arc<Image>> = OnceLock::new();
    png(&CELL, include_bytes!("../../assets/about/crow-about-sidepic.png"))
}

fn app_icon() -> Arc<Image> {
    static CELL: OnceLock<Arc<Image>> = OnceLock::new();
    png(&CELL, include_bytes!("../../assets/about/crow-app-icon.png"))
}

/// What COPY SYSTEM INFO puts on the clipboard, for bug reports.
fn system_report() -> String {
    format!(
        "Crow v{}\nPlatform: {} ({})\nProfile: {}\nRepository: {REPO}\nWebsite: {WEBSITE}\nCommunity: {FORUM}\nEngine: GPUI Kit (WGPU) + crow-config-core\nVault: Argon2id + ChaCha20-Poly1305 + TOTP\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        if cfg!(debug_assertions) { "debug" } else { "release" },
    )
}

pub fn about_modal(app: Entity<CrowApp>, copied_toast: bool, update: &UpdateState, auto_install: bool) -> impl IntoElement {
    let app_scrim = app.clone();
    let app_x = app.clone();
    let app_close = app.clone();
    let app_copy = app.clone();
    let version = env!("CARGO_PKG_VERSION");
    let profile = if cfg!(debug_assertions) { "debug build" } else { "release build" };

    div()
        .id("about-modal-scrim")
        .absolute()
        .inset_0()
        .occlude()
        .bg(hex_rgba(0x050507, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| app_scrim.update(cx, |this, cx| this.close_about_modal(cx)))
        .child(
            div()
                .id("about-modal-panel")
                .occlude()
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // keep clicks inside from reaching the backdrop (which closes)
                .w(px(780.0))
                .h(px(440.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .shadow_lg()
                .flex()
                .font_family(FONT_MONO)
                // The crow, down the whole left side.
                .child(
                    div()
                        .w(px(232.0))
                        .h_full()
                        .flex_none()
                        .overflow_hidden()
                        .border_r_1()
                        .border_color(BORDER_STRONG)
                        .bg(hex_rgb(0x000000))
                        .child(img(side_picture()).size_full().object_fit(ObjectFit::Cover)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .h_full()
                        .flex()
                        .flex_col()
                        // Title bar
                        .child(
                            div()
                                .h(px(30.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pl(px(14.0))
                                .pr(px(6.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_color(BORDER_PANEL)
                                .child(div().text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(TEXT_SECONDARY).child("About Crow"))
                                .child(crate::components::icon_button::icon_button("about-close-btn", TablerIcon::X, false).on_click(move |_ev, _window, cx| app_x.update(cx, |this, cx| this.close_about_modal(cx)))),
                        )
                        // Header: icon, name, version, what it is
                        .child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(16.0))
                                .px(px(20.0))
                                .pt(px(18.0))
                                .pb(px(14.0))
                                .child(img(app_icon()).size(px(64.0)).flex_none())
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(3.0))
                                        .child(
                                            div()
                                                .flex()
                                                .items_baseline()
                                                .gap(px(10.0))
                                                .child(div().text_size(px(22.0)).font_weight(FontWeight::EXTRA_BOLD).text_color(TEXT_MAX).child("Crow"))
                                                .child(div().text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(OK).child(format!("v{version}"))),
                                        )
                                        .child(div().text_size(px(11.0)).text_color(TEXT_SECONDARY).child("Native Linux server manager over SSH"))
                                        .child(div().text_size(px(10.0)).text_color(TEXT_FAINT).child(format!("{} · {} · {profile}", std::env::consts::OS, std::env::consts::ARCH))),
                                ),
                        )
                        // Where to find the project
                        .child(
                            div()
                                .flex_none()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .px(px(20.0))
                                .pb(px(12.0))
                                .child(link_row("about-link-website", "Website", WEBSITE))
                                .child(link_row("about-link-repo", "Source", REPO))
                                .child(link_row("about-link-forum", "Community", FORUM)),
                        )
                        // The details, in a dark scroll box
                        .child(
                            div()
                                .flex_1()
                                .min_h(px(0.0))
                                .mx(px(20.0))
                                .bg(hex_rgb(0x08080a))
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .child(
                                    div()
                                        .id("about-details-scroll")
                                        .size_full()
                                        .overflow_y_scrollbar()
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .p(px(12.0))
                                        // A newer version (ERR-88).
                                        .children(update_panel(update, auto_install, app.clone()))
                                        .child(heading("BUILD"))
                                        .child(info_row("Platform", &format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH)))
                                        .child(info_row("Profile", profile))
                                        .child(info_row("UI toolkit", "GPUI Kit 0.6.1 · WGPU"))
                                        .child(info_row("Config engine", "crow-config-core · schema IR"))
                                        .child(info_row("Vault", "Argon2id · ChaCha20-Poly1305 · TOTP"))
                                        .child(heading("CREDITS"))
                                        .child(info_row("Icons", "Tabler Icons · MIT"))
                                        .child(info_row("Flags", "flag-icons · MIT"))
                                        .child(info_row("Distro logos", "Dashboard Icons by homarr-labs · Apache-2.0"))
                                        .child(info_row("GeoIP data", crate::geoip::ATTRIBUTION))
                                        .child(info_row("Font", "JetBrains Mono · OFL-1.1"))
                                        .child(heading("LICENSE"))
                                        .child(div().text_size(px(10.0)).line_height(px(15.0)).text_color(TEXT_DIM).child("AEUPL-1.2. Free for any non-commercial use. See LICENSE in the repository.")),
                                ),
                        )
                        // Footer
                        .child(
                            div()
                                .h(px(48.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(20.0))
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
                                            cx.write_to_clipboard(ClipboardItem::new_string(system_report()));
                                            app_copy.update(cx, |this, cx| {
                                                this.about_copied_toast = true;
                                                cx.notify();
                                            });
                                        })
                                        .child(tabler_icon(TablerIcon::Copy).size(px(12.0)).text_color(if copied_toast { OK } else { TEXT_SECONDARY }))
                                        .child(div().font_weight(FontWeight::BOLD).text_size(px(10.5)).text_color(if copied_toast { OK } else { TEXT_PRIMARY }).child(if copied_toast { "✓ COPIED SYSTEM INFO" } else { "COPY SYSTEM INFO" })),
                                )
                                .child(
                                    div()
                                        .id("about-dismiss-btn")
                                        .px(px(20.0))
                                        .py(px(5.0))
                                        .bg(BG_CONTROL)
                                        .border_1()
                                        .border_color(BORDER_STRONG)
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MAX)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| app_close.update(cx, |this, cx| this.close_about_modal(cx)))
                                        .child("OK"),
                                ),
                        ),
                ),
        )
}

/// A labelled link that opens in the browser.
fn link_row(id: &'static str, label: &'static str, url: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .text_size(px(11.0))
        .child(div().w(px(80.0)).flex_none().text_color(TEXT_MUTED).child(label))
        .child(
            div()
                .id(id)
                .text_color(hex_rgb(0x8ab4ff))
                .cursor_pointer()
                .hover(|s| s.underline().text_color(hex_rgb(0xb4ceff)))
                .on_click(move |_ev, _window, cx| cx.open_url(url))
                .child(url.trim_start_matches("https://")),
        )
}

fn heading(title: &'static str) -> impl IntoElement {
    div().pt(px(4.0)).text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(TEXT_FAINT).child(title)
}

/// The offered release: notes, skip, and install (when installing is on)
/// or restart (once installed).
fn update_panel(update: &UpdateState, auto_install: bool, app: Entity<CrowApp>) -> Option<impl IntoElement> {
    if update.available.is_none() && update.note.is_none() {
        return None;
    }
    let button = |id: &'static str, label: &'static str, color: Rgba| {
        div()
            .id(id)
            .px(px(10.0))
            .py(px(4.0))
            .border_1()
            .border_color(color)
            .cursor_pointer()
            .font_family(FONT_MONO)
            .text_size(px(10.0))
            .font_weight(FontWeight::BOLD)
            .text_color(color)
            .hover(|s| s.bg(BG_ROW_HOVER))
            .child(label)
    };
    let (a_notes, a_skip, a_install, a_restart) = (app.clone(), app.clone(), app.clone(), app);
    Some(
        div()
            .bg(OK_BG)
            .border_1()
            .border_color(OK)
            .p(px(12.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .children(update.available.as_ref().map(|r| {
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(OK)
                    .child(format!("CROW {}{} IS AVAILABLE", r.version, if r.prerelease { " (PRE-RELEASE)" } else { "" }))
            }))
            .children(update.note.clone().map(|n| div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_PRIMARY).child(n)))
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .when(update.available.is_some(), |d| {
                        d.child(button("btn-update-notes", "RELEASE NOTES", TEXT_SECONDARY).on_click(move |_e, _w, cx| a_notes.update(cx, |this, cx| this.open_release_notes(cx))))
                    })
                    .when(update.available.is_some() && !update.installed, |d| {
                        d.child(button("btn-update-skip", "SKIP THIS VERSION", TEXT_SECONDARY).on_click(move |_e, _w, cx| a_skip.update(cx, |this, cx| this.skip_update(cx))))
                    })
                    .when(auto_install && update.available.is_some() && !update.installed && !update.installing, |d| {
                        d.child(button("btn-update-install", "VERIFY & INSTALL", OK).on_click(move |_e, _w, cx| a_install.update(cx, |this, cx| this.install_update(cx))))
                    })
                    .when(update.installed, |d| {
                        d.child(button("btn-update-restart", "RESTART NOW", OK).on_click(move |_e, _w, cx| a_restart.update(cx, |this, cx| this.restart_after_update(cx))))
                    }),
            )
            .when(!auto_install && update.available.is_some(), |d| {
                d.child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_MUTED).child("Crow doesn't download anything unless you turn on installing in Settings → General."))
            }),
    )
}

fn info_row(label: &'static str, value: &str) -> impl IntoElement {
    div()
        .flex()
        .gap(px(12.0))
        .text_size(px(10.5))
        .child(div().w(px(110.0)).flex_none().text_color(TEXT_MUTED).child(label))
        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_PRIMARY).child(value.to_string()))
}
