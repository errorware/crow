#![recursion_limit = "512"]
use gpui_kit::*;

pub mod ai;
pub mod appearance;
mod app;
mod components;
mod theme;
pub mod palette;
pub mod topology;
pub mod config;
pub mod geoip;
pub mod host;
pub mod journal;
pub mod keys;
pub mod terminal;
pub mod lab;
pub mod metrics;
pub mod os_detect;
pub mod providers;
pub mod region;
pub mod secret_string;
pub mod security;
pub mod update;
pub mod vault;
mod views;

use app::CrowApp;

fn main() {
    // Started by ssh as its askpass helper during a password bootstrap
    // (host::bootstrap): answer the prompt and exit, no window.
    if let Ok(sock) = std::env::var(host::bootstrap::ASKPASS_SOCK_ENV) {
        let prompt = std::env::args().nth(1).unwrap_or_default();
        std::process::exit(host::bootstrap::askpass_main(&sock, &prompt));
    }

    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        crate::theme::init_obsidian_theme(cx);

        // Embed JetBrains Mono so it is always available on all platforms (macOS, Linux, Windows)
        let fonts = vec![
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice()),
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf").as_slice()),
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf").as_slice()),
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf").as_slice()),
        ];
        if let Err(err) = cx.text_system().add_fonts(fonts) {
            eprintln!("Failed to load embedded JetBrains Mono fonts: {err}");
        }

        let (disp_w, disp_h) = cx
            .primary_display()
            .map(|d| {
                let b = d.bounds();
                (b.size.width.as_f32(), b.size.height.as_f32())
            })
            .unwrap_or((1400.0, 900.0));

        let width = (disp_w * 0.92).min(1400.0).max(1100.0);
        let height = (disp_h * 0.88).min(860.0).max(700.0);
        let window_size = size(px(width), px(height));
        let bounds = Bounds::centered(None, window_size, cx);
        let mut options = WindowOptions::default();
        options.window_bounds = Some(WindowBounds::Windowed(bounds));
        options.window_min_size = Some(size(px(1100.0), px(700.0)));
        #[cfg(target_os = "macos")]
        {
            options.titlebar = Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: Some(point(px(12.0), px(11.0))),
            });
        }
        #[cfg(not(target_os = "macos"))]
        {
            options.titlebar = Some(TitlebarOptions {
                title: Some("Crow".into()),
                appears_transparent: true,
                traffic_light_position: None,
            });
            options.window_decorations = Some(WindowDecorations::Client);
            // The resize border around the app (components::window_frame)
            // must show the desktop through it.
            options.window_background = WindowBackgroundAppearance::Transparent;
        }
        options.is_resizable = true;
        // Matches the desktop entry and icon names (packaging/linux), so
        // Wayland and X11 docks show Crow's icon.
        options.app_id = Some("rs.crow.Crow".into());
        options.is_minimizable = true;

        cx.open_window(options, |window, cx| {
            window.activate_window();
            cx.new(|cx| CrowApp::new(window, cx))
        })
        .expect("failed to open Crow window");

        cx.activate(true);
    });
}
