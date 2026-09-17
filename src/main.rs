use gpui_kit::*;

mod app;
mod components;
mod theme;
pub mod vault;
mod views;

use app::CrowApp;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
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

            cx.open_window(options, |_window, cx| {
                cx.new(|cx| CrowApp::new(cx))
            })
            .expect("failed to open Crow window");
        })
        .detach();
    });
}
