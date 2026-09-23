#![recursion_limit = "256"]
use gpui_kit::*;

mod app;
mod components;
mod theme;
pub mod config;
pub mod host;
pub mod journal;
pub mod keys;
pub mod lab;
pub mod metrics;
pub mod os_detect;
pub mod vault;
mod views;

use app::CrowApp;

fn main() {
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

        cx.spawn(async move |cx| {
            let mut options = WindowOptions::default();
            options.window_bounds = Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1600.0), px(1000.0)),
            }));
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
            }
            options.is_resizable = true;
            options.is_minimizable = true;

            cx.open_window(options, |window, cx| {
                cx.new(|cx| CrowApp::new(window, cx))
            })
            .expect("failed to open Crow window");
        })
        .detach();
    });
}
