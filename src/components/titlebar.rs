use gpui_kit::*;
use crate::theme::*;
use crate::app::{CrowApp, Screen};
use crate::components::icons::{TablerIcon, tabler_icon};

pub fn hamburger_icon(ink: Rgba) -> impl IntoElement {
    div()
        .w(px(11.0))
        .flex()
        .flex_col()
        .gap(px(2.5))
        .child(div().h(px(1.5)).bg(ink))
        .child(div().h(px(1.5)).bg(ink))
        .child(div().h(px(1.5)).bg(ink))
}

pub fn diamond_mark(ink: Rgba) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let mut path = PathBuilder::fill();
            let mid_x = bounds.origin.x + bounds.size.width / 2.0;
            let mid_y = bounds.origin.y + bounds.size.height / 2.0;
            path.move_to(point(mid_x, bounds.origin.y));
            path.line_to(point(bounds.origin.x + bounds.size.width, mid_y));
            path.line_to(point(mid_x, bounds.origin.y + bounds.size.height));
            path.line_to(point(bounds.origin.x, mid_y));
            path.close();
            if let Ok(built) = path.build() {
                window.paint_path(built, ink);
            }
        },
    )
    .w(px(13.0))
    .h(px(13.0))
    .flex_none()
}

#[derive(Clone, Debug)]
pub struct ServerTab {
    pub id: &'static str,
    pub name: &'static str,
    pub status_color: Rgba,
    #[allow(dead_code)]
    pub is_active: bool,
}

pub fn titlebar(
    tabs: &[ServerTab],
    active_tab_id: &str,
    current_screen: Screen,
    menu_open: bool,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let mut bar = div()
        .h(px(36.0))
        .flex_none()
        .flex()
        .items_stretch()
        .bg(BG_CHROME)
        .border_b_1()
        .border_color(BORDER_PANEL);

    // 1. macOS native traffic lights clearance (only on macOS)
    #[cfg(target_os = "macos")]
    {
        bar = bar.child(
            div()
                .w(px(76.0))
                .flex_none(),
        );
    }

    // 2. Interactive Burger Menu + CROW Mark
    let app_burger = app.clone();
    let menu_ink = if menu_open { TEXT_MAX } else { TEXT_PRIMARY };
    let menu_bg = if menu_open { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) };

    bar = bar.child(
        div()
            .id("burger-menu-btn")
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(11.0))
            .border_r_1()
            .border_color(BORDER_PANEL)
            .bg(menu_bg)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
            .on_click(move |_ev, _window, cx| {
                app_burger.update(cx, |this, cx| {
                    this.toggle_menu(cx);
                });
            })
            .child(hamburger_icon(menu_ink))
            .child(diamond_mark(menu_ink))
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(menu_ink)
                    .child("CROW"),
            ),
    );

    // 3. Tab strip: Pinned Fleet Tab, Server Tabs, and Add Server (+)
    let is_fleet_active = current_screen == Screen::Fleet;
    let app_fleet = app.clone();
    let app_plus = app.clone();

    bar = bar.child(
        div()
            .flex()
            .items_stretch()
            .flex_1()
            .min_w(px(0.0))
            .on_mouse_down(MouseButton::Left, |ev, window, _cx| {
                if ev.click_count == 2 {
                    window.zoom_window();
                } else {
                    window.start_window_move();
                }
            })
            // Pinned FLEET Tab (never closeable, has pin indicator ⌾)
            .child(
                div()
                    .id("tab-pinned-fleet")
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .px(px(13.0))
                    .border_r_1()
                    .border_color(BORDER_PANEL)
                    .bg(if is_fleet_active { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _window, cx| {
                        app_fleet.update(cx, |this, cx| {
                            this.set_screen(Screen::Fleet, cx);
                        });
                    })
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(10.0))
                            .text_color(if is_fleet_active { OK } else { TEXT_DIMMER })
                            .child("⬢"),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .font_weight(if is_fleet_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(if is_fleet_active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .child("FLEET"),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(9.0))
                            .text_color(TEXT_GHOST)
                            .child("⌾"),
                    ),
            )
            // Server Tabs (each with close affordance)
            .children(tabs.iter().enumerate().map(|(idx, tab)| {
                let is_active = current_screen == Screen::Server && tab.id == active_tab_id;
                let tab_id = tab.id;
                let app_tab = app.clone();

                div()
                    .id(ElementId::NamedInteger("server-tab".into(), idx as u64))
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .px(px(14.0))
                    .border_r_1()
                    .border_color(BORDER_PANEL)
                    .bg(if is_active { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _window, cx| {
                        app_tab.update(cx, |this, cx| {
                            this.switch_tab(tab_id, cx);
                            this.set_screen(Screen::Server, cx);
                        });
                    })
                    .child(
                        div()
                            .size(px(6.0))
                            .rounded_full()
                            .bg(tab.status_color)
                            .flex_none(),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(if is_active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .child(tab.name),
                    )
                    .child(
                        tabler_icon(TablerIcon::X)
                            .size(px(11.0))
                            .text_color(rgb(0x6b7280)),
                    )
            }))
            // Add Server Button (+)
            .child(
                div()
                    .id("tab-btn-add-server")
                    .flex()
                    .items_center()
                    .px(px(12.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _window, cx| {
                        app_plus.update(cx, |this, cx| {
                            this.set_screen(Screen::Onboard, cx);
                        });
                    })
                    .child(
                        tabler_icon(TablerIcon::Plus)
                            .size(px(13.0))
                            .text_color(TEXT_FAINT),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .h_full(),
            ),
    );

    // 4. Session meta (varies by screen)
    let meta_text = if current_screen == Screen::Fleet {
        "12 hosts · 11 agents"
    } else {
        "SSH ED25519 · lat 12ms"
    };

    bar = bar.child(
        div()
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(14.0))
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .text_color(TEXT_FAINT)
            .border_l_1()
            .border_color(BORDER_PANEL)
            .child(div().child(meta_text))
            .child(div().child("03:41:22 UTC")),
    );

    // 5. Linux / Windows platform window controls
    #[cfg(not(target_os = "macos"))]
    {
        bar = bar.child(
            div()
                .flex()
                .items_stretch()
                .border_l_1()
                .border_color(BORDER_PANEL)
                // Minimize
                .child(
                    div()
                        .id("win-btn-minimize")
                        .w(px(38.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(|_ev, window, _cx| {
                            window.minimize_window();
                        })
                        .child(
                            tabler_icon(TablerIcon::Minus)
                                .size(px(13.0))
                                .text_color(TEXT_MUTED),
                        ),
                )
                // Maximize / Restore
                .child(
                    div()
                        .id("win-btn-maximize")
                        .w(px(38.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(|_ev, window, _cx| {
                            window.zoom_window();
                        })
                        .child(
                            tabler_icon(TablerIcon::Square)
                                .size(px(12.0))
                                .text_color(TEXT_MUTED),
                        ),
                )
                // Close
                .child(
                    div()
                        .id("win-btn-close")
                        .w(px(38.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(CRIT_BG).text_color(CRIT_INK))
                        .on_click(|_ev, _window, cx| {
                            cx.quit();
                        })
                        .child(
                            tabler_icon(TablerIcon::X)
                                .size(px(13.0))
                                .text_color(TEXT_MUTED),
                        ),
                ),
        );
    }

    bar
}

pub fn burger_menu_overlay(app: Entity<CrowApp>, current_screen: Screen) -> impl IntoElement {
    let app_backdrop = app.clone();

    // Menu entries: (icon, label, key_shortcut, screen_target, view_target, is_danger, is_header)
    let items = [
        // Section 1: FLEET
        ("", "FLEET", "", None, None, false, true),
        ("⬢", "Fleet Overview", "⌘1", Some(Screen::Fleet), None, false, false),
        ("⬡", "Fleet Setup — topology & policy", "⌘⇧F", Some(Screen::FleetSetup), None, false, false),
        ("+", "Add Server…", "⌘N", Some(Screen::Onboard), None, false, false),
        ("⇄", "Import from Terraform / Ansible", "", None, None, false, false),
        // Section 2: THIS SERVER
        ("", "THIS SERVER", "", None, None, false, true),
        ("◈", "edge-01 · Overview", "⌘2", Some(Screen::Server), Some("overview"), false, false),
        ("◧", "Config files", "⌘3", Some(Screen::Server), Some("config"), false, false),
        ("▶", "Open terminal", "⌘T", None, None, false, false),
        ("⇩", "Download diagnostics bundle", "", None, None, false, false),
        // Section 3: APPLICATION
        ("", "APPLICATION", "", None, None, false, true),
        ("⚙", "Settings", "⌘,", Some(Screen::Settings), None, false, false),
        ("⌨", "Keyboard shortcuts", "⌘/", None, None, false, false),
        ("↻", "Check for updates — v1.4.2", "", None, None, false, false),
        // Section 4: SESSION
        ("", "SESSION", "", None, None, false, true),
        ("⏻", "Lock & disconnect all hosts", "⇧⌘L", None, None, true, false),
        ("✕", "Quit Crow", "⌘Q", None, None, true, false),
    ];

    div()
        .id("burger-menu-backdrop")
        .absolute()
        .top(px(36.0))
        .left_0()
        .right_0()
        .bottom_0()
        .bg(rgba(0x05050759)) // rgba(5,5,7,0.35)
        .on_click(move |_ev, _window, cx| {
            app_backdrop.update(cx, |this, cx| {
                this.close_menu(cx);
            });
        })
        .child(
            div()
                .id("burger-menu-dropdown")
                .w(px(288.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .border_t_0()
                .shadow_lg()
                .flex()
                .flex_col()
                .children(items.into_iter().enumerate().map(|(idx, (icon, label, key, target_screen, target_view, is_danger, is_header))| {
                    if is_header {
                        return div()
                            .id(ElementId::NamedInteger("menu-header".into(), idx as u64))
                            .h(px(24.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .px(px(12.0))
                            .bg(BG_PANEL)
                            .border_t_1()
                            .border_color(if idx == 0 { hex_rgba(0, 0.0) } else { BORDER_DEFAULT })
                            .font_family(FONT_MONO)
                            .text_size(px(9.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(TEXT_DIMMER)
                            .child(label);
                    }

                    let is_active = target_screen == Some(current_screen);
                    let app_item = app.clone();

                    div()
                        .id(ElementId::NamedInteger("menu-item".into(), idx as u64))
                        .relative()
                        .h(px(30.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(12.0))
                        .bg(if is_active { BG_KEY } else { hex_rgba(0, 0.0) })
                        .children(if is_active {
                            Some(left_indicator(TEXT_PRIMARY))
                        } else {
                            None
                        })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_KEY))
                        .on_click(move |_ev, _window, cx| {
                            if label == "Quit Crow" {
                                cx.quit();
                                return;
                            }
                            app_item.update(cx, |this, cx| {
                                this.close_menu(cx);
                                if let Some(view) = target_view {
                                    this.set_view(view, cx);
                                }
                                if let Some(scr) = target_screen {
                                    this.set_screen(scr, cx);
                                }
                            });
                        })
                        .child(
                            div()
                                .w(px(14.0))
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_align(TextAlign::Center)
                                .text_color(if is_danger {
                                    CRIT
                                } else if is_active {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_DIMMER
                                })
                                .child(icon),
                        )
                        .child(
                            div()
                                .flex_1()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .text_color(if is_danger {
                                    CRIT_INK_DIM
                                } else if is_active {
                                    TEXT_MAX
                                } else {
                                    TEXT_PRIMARY
                                })
                                .child(label),
                        )
                        .children(if !key.is_empty() {
                            Some(
                                div()
                                    .px(px(5.0))
                                    .py(px(1.0))
                                    .bg(BG_KEY)
                                    .border_1()
                                    .border_color(BORDER_KEY)
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(TEXT_MUTED)
                                    .child(key),
                            )
                        } else {
                            None
                        })
                })),
        )
}
