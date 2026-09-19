use gpui_kit::*;
use gpui_kit::base::InteractiveElementExt;
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

pub fn crow_logo(ink: Rgba) -> impl IntoElement {
    svg()
        .data(include_bytes!("../../assets/logo.svg").as_slice())
        .w(px(16.0))
        .h(px(14.0))
        .text_color(ink)
        .flex_none()
}

#[derive(Clone, Debug)]
pub struct ServerTab {
    pub id: String,
    pub name: String,
    pub status_color: Rgba,
    #[allow(dead_code)]
    pub is_active: bool,
}

pub fn active_tab_gradient_bar() -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            if bounds.size.width <= px(0.0) {
                return;
            }
            let height = bounds.size.height;

            // Continuous slow phase (approx 3.5s per cycle)
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let phase = ((millis % 3500) as f32) / 3500.0;

            let steps = 30;
            let slice_w = bounds.size.width / (steps as f32);

            for i in 0..steps {
                let norm = (i as f32) / (steps as f32);
                // Travelling wave from left to right: (norm - phase) wrapped around [0, 1)
                let mut wave = norm - phase;
                if wave < 0.0 {
                    wave += 1.0;
                }

                // Smooth bell-like shimmer: peak alpha around 0.60, base alpha around 0.12
                let shimmer = ((wave * std::f32::consts::PI * 2.0).sin() + 1.0) * 0.5;
                let alpha = 0.12 + shimmer * 0.48;

                // Subtle emerald green (#3ecf6e)
                let color = Rgba {
                    r: 0.243,
                    g: 0.812,
                    b: 0.431,
                    a: alpha,
                };

                let x0 = bounds.origin.x + slice_w * (i as f32);
                let x1 = x0 + slice_w;

                let mut path = PathBuilder::fill();
                path.move_to(point(x0, bounds.origin.y));
                path.line_to(point(x1, bounds.origin.y));
                path.line_to(point(x1, bounds.origin.y + height));
                path.line_to(point(x0, bounds.origin.y + height));
                path.close();

                if let Ok(built) = path.build() {
                    window.paint_path(built, color);
                }
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .right_0()
    .h(px(2.0))
}

pub fn titlebar(
    tabs: &[ServerTab],
    active_tab_id: &str,
    current_screen: Screen,
    menu_open: bool,
    server_count: usize,
    agent_count: usize,
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
            .px(px(14.0))
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
            .child(crow_logo(menu_ink))
            .child(
                div()
                    .font_family(FONT_MONO)
                    .text_size(px(11.5))
                    .font_weight(FontWeight::BOLD)
                    .text_color(menu_ink)
                    .child("CROW"),
            ),
    );

    // 3. Dynamic Server / Context Tabs Strip
    let is_fleet_active = current_screen == Screen::Fleet;
    let app_fleet = app.clone();
    let app_plus = app.clone();

    bar = bar.child(
        div()
            .flex_1()
            .flex()
            .items_stretch()
            .overflow_hidden()
            // Fleet Root Tab
            .child(
                div()
                    .id("tab-btn-fleet")
                    .relative()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(14.0))
                    .border_r_1()
                    .border_color(BORDER_PANEL)
                    .bg(if is_fleet_active { BG_APP } else { hex_rgba(0, 0.0) })
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _window, cx| {
                        app_fleet.update(cx, |this, cx| {
                            this.set_screen(Screen::Fleet, cx);
                        });
                    })
                    .children(if is_fleet_active {
                        Some(active_tab_gradient_bar())
                    } else {
                        None
                    })
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .font_weight(if is_fleet_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(if is_fleet_active { TEXT_PRIMARY } else { TEXT_MUTED })
                            .child("FLEET"),
                    ),
            )
            // Server Tabs (each with close affordance)
            .children(tabs.iter().enumerate().map(|(idx, tab)| {
                let is_active = current_screen == Screen::Server && tab.id == active_tab_id;
                let tab_id = tab.id.clone();
                let close_tab_id = tab.id.clone();
                let app_tab = app.clone();
                let app_close = app.clone();

                div()
                    .id(ElementId::NamedInteger("server-tab".into(), idx as u64))
                    .relative()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .px(px(14.0))
                    .border_r_1()
                    .border_color(if is_active { BORDER_STRONG } else { BORDER_PANEL })
                    .bg(if is_active { BG_APP } else { hex_rgba(0, 0.0) })
                    .cursor_pointer()
                    .hover(|s| s.bg(if is_active { BG_APP } else { BG_ROW_HOVER }))
                    .on_click(move |_ev, _window, cx| {
                        let tid = tab_id.clone();
                        app_tab.update(cx, |this, cx| {
                            this.switch_tab(&tid, cx);
                            this.set_screen(Screen::Server, cx);
                        });
                    })
                    .children(if is_active {
                        Some(active_tab_gradient_bar())
                    } else {
                        None
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
                            .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(if is_active { TEXT_MAX } else { TEXT_MUTED })
                            .child(tab.name.clone()),
                    )
                    .child(
                        div()
                            .id(ElementId::NamedInteger("close-tab".into(), idx as u64))
                            .cursor_pointer()
                            .hover(|s| s.text_color(TEXT_PRIMARY))
                            .on_click(move |_ev, _window, cx| {
                                let tid = close_tab_id.clone();
                                app_close.update(cx, |this, cx| {
                                    this.close_tab(&tid, cx);
                                });
                            })
                            .child(
                                tabler_icon(TablerIcon::X)
                                    .size(px(11.0))
                                    .text_color(rgb(0x6b7280)),
                            ),
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
                    .id("titlebar-drag-region")
                    .flex_1()
                    .h_full()
                    .on_mouse_down(MouseButton::Left, |_ev, window, _cx| {
                        window.start_window_move();
                    })
                    .on_double_click(|_ev, window, _cx| {
                        window.zoom_window();
                    }),
            ),
    );

    // 4. Session meta (varies by screen)
    let meta_text = if current_screen == Screen::Fleet {
        format!(
            "{} host{} · {} agent{}",
            server_count,
            if server_count == 1 { "" } else { "s" },
            agent_count,
            if agent_count == 1 { "" } else { "s" }
        )
    } else {
        "SSH ED25519 · active".to_string()
    };
    let server_time_text = chrono::Utc::now().format("%H:%M:%S").to_string();
    let local_time_text = chrono::Local::now().format("%H:%M:%S").to_string();

    bar = bar.child(
        div()
            .id("titlebar-meta-drag-region")
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(14.0))
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .text_color(TEXT_FAINT)
            .border_l_1()
            .border_color(BORDER_PANEL)
            .on_mouse_down(MouseButton::Left, |_ev, window, _cx| {
                window.start_window_move();
            })
            .on_double_click(|_ev, window, _cx| {
                window.zoom_window();
            })
            .child(div().child(meta_text))
            .children(if current_screen == Screen::Server {
                Some(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .child(div().text_color(TEXT_DIMMER).child("SRV"))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(TEXT_SECONDARY)
                                .child(server_time_text),
                        )
                        .child(div().text_color(TEXT_DIMMER).child("UTC")),
                )
            } else {
                None
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(div().text_color(TEXT_DIMMER).child("YOU"))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(TEXT_SECONDARY)
                            .child(local_time_text),
                    ),
            ),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    NavigateScreen(Screen),
    NavigateServerView(&'static str),
    ToggleLab,
    TogglePalette,
    About,
    LockVault,
    Quit,
}

struct MenuItem {
    icon: Option<TablerIcon>,
    label: String,
    shortcut: &'static str,
    action: Option<MenuAction>,
    is_danger: bool,
    is_header: bool,
}

pub fn burger_menu_overlay(
    app: Entity<CrowApp>,
    current_screen: Screen,
    active_view: &str,
    active_server_name: Option<String>,
) -> impl IntoElement {
    let app_backdrop = app.clone();

    let server_overview_label = if let Some(ref name) = active_server_name {
        format!("{} · Overview", name)
    } else {
        "Server Overview".to_string()
    };

    let mut items: Vec<MenuItem> = Vec::new();

    // Section 1: FLEET
    items.push(MenuItem {
        icon: None,
        label: "FLEET".to_string(),
        shortcut: "",
        action: None,
        is_danger: false,
        is_header: true,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::LayoutDashboard),
        label: "Fleet Overview".to_string(),
        shortcut: "⌘1",
        action: Some(MenuAction::NavigateScreen(Screen::Fleet)),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Network),
        label: "Fleet Setup — Topology & Policy".to_string(),
        shortcut: "⌘⇧F",
        action: Some(MenuAction::NavigateScreen(Screen::FleetSetup)),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Plus),
        label: "Enroll New Server…".to_string(),
        shortcut: "⌘N",
        action: Some(MenuAction::NavigateScreen(Screen::Onboard)),
        is_danger: false,
        is_header: false,
    });

    // Section 2: THIS SERVER
    items.push(MenuItem {
        icon: None,
        label: "THIS SERVER".to_string(),
        shortcut: "",
        action: None,
        is_danger: false,
        is_header: true,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Server),
        label: server_overview_label,
        shortcut: "⌘2",
        action: Some(MenuAction::NavigateServerView("overview")),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::AdjustmentsHorizontal),
        label: "Managed Configs".to_string(),
        shortcut: "⌘3",
        action: Some(MenuAction::NavigateServerView("config")),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::FileText),
        label: "Systemd Journal Logs".to_string(),
        shortcut: "⌘4",
        action: Some(MenuAction::NavigateServerView("logs")),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Box),
        label: "Local Test Lab & VMs".to_string(),
        shortcut: "",
        action: Some(MenuAction::ToggleLab),
        is_danger: false,
        is_header: false,
    });

    // Section 3: APPLICATION
    items.push(MenuItem {
        icon: None,
        label: "APPLICATION".to_string(),
        shortcut: "",
        action: None,
        is_danger: false,
        is_header: true,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Search),
        label: "Command Palette…".to_string(),
        shortcut: "⌘K",
        action: Some(MenuAction::TogglePalette),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Settings),
        label: "Settings".to_string(),
        shortcut: "⌘,",
        action: Some(MenuAction::NavigateScreen(Screen::Settings)),
        is_danger: false,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::InfoCircle),
        label: "About Crow…".to_string(),
        shortcut: "",
        action: Some(MenuAction::About),
        is_danger: false,
        is_header: false,
    });

    // Section 4: SESSION
    items.push(MenuItem {
        icon: None,
        label: "SESSION".to_string(),
        shortcut: "",
        action: None,
        is_danger: false,
        is_header: true,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Lock),
        label: "Lock Vault & Disconnect".to_string(),
        shortcut: "⇧⌘L",
        action: Some(MenuAction::LockVault),
        is_danger: true,
        is_header: false,
    });
    items.push(MenuItem {
        icon: Some(TablerIcon::Power),
        label: "Quit Crow".to_string(),
        shortcut: "⌘Q",
        action: Some(MenuAction::Quit),
        is_danger: true,
        is_header: false,
    });

    div()
        .id("burger-menu-backdrop")
        .absolute()
        .top(px(36.0))
        .left_0()
        .right_0()
        .bottom_0()
        .occlude()
        .bg(rgba(0x05050759)) // rgba(5,5,7,0.35)
        .on_click(move |_ev, _window, cx| {
            app_backdrop.update(cx, |this, cx| {
                this.close_menu(cx);
            });
        })
        .child(
            div()
                .id("burger-menu-dropdown")
                .occlude()
                .on_click(|_ev, _window, _cx| {}) // capture click so backdrop doesn't close on menu body
                .w(px(296.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .border_t_0()
                .shadow_lg()
                .flex()
                .flex_col()
                .children(items.into_iter().enumerate().map(|(idx, item)| {
                    if item.is_header {
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
                            .child(item.label);
                    }

                    let is_active = match &item.action {
                        Some(MenuAction::NavigateScreen(scr)) => current_screen == *scr,
                        Some(MenuAction::NavigateServerView(view)) => current_screen == Screen::Server && active_view == *view,
                        _ => false,
                    };
                    let app_item = app.clone();
                    let action = item.action.clone();

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
                            match action {
                                Some(MenuAction::Quit) => {
                                    cx.quit();
                                }
                                Some(MenuAction::LockVault) => {
                                    app_item.update(cx, |this, cx| {
                                        this.lock(cx);
                                    });
                                }
                                Some(MenuAction::ToggleLab) => {
                                    app_item.update(cx, |this, cx| {
                                        this.close_menu(cx);
                                        this.toggle_local_lab_modal(cx);
                                    });
                                }
                                Some(MenuAction::TogglePalette) => {
                                    app_item.update(cx, |this, cx| {
                                        this.close_menu(cx);
                                        this.toggle_palette(cx);
                                    });
                                }
                                Some(MenuAction::About) => {
                                    app_item.update(cx, |this, cx| {
                                        this.close_menu(cx);
                                        this.open_about_modal(cx);
                                    });
                                }
                                Some(MenuAction::NavigateScreen(scr)) => {
                                    app_item.update(cx, |this, cx| {
                                        this.close_menu(cx);
                                        this.set_screen(scr, cx);
                                    });
                                }
                                Some(MenuAction::NavigateServerView(view)) => {
                                    app_item.update(cx, |this, cx| {
                                        this.close_menu(cx);
                                        this.set_view(view, cx);
                                        this.set_screen(Screen::Server, cx);
                                    });
                                }
                                None => {}
                            }
                        })
                        .child(
                            div()
                                .w(px(14.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .children(item.icon.map(|icon| {
                                    tabler_icon(icon)
                                        .size(px(13.5))
                                        .text_color(if item.is_danger {
                                            CRIT
                                        } else if is_active {
                                            TEXT_PRIMARY
                                        } else {
                                            TEXT_DIMMER
                                        })
                                })),
                        )
                        .child(
                            div()
                                .flex_1()
                                .font_family(FONT_MONO)
                                .text_size(px(11.5))
                                .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .text_color(if item.is_danger {
                                    CRIT_INK_DIM
                                } else if is_active {
                                    TEXT_MAX
                                } else {
                                    TEXT_PRIMARY
                                })
                                .child(item.label),
                        )
                        .children(if !item.shortcut.is_empty() {
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
                                    .child(item.shortcut),
                            )
                        } else {
                            None
                        })
                })),
        )
}
