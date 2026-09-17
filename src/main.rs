use gpui_kit::*;

mod app;
mod components;
mod theme;
mod views;

use app::CrowApp;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        cx.spawn(async move |cx| {
            let mut options = WindowOptions::default();
            options.window_bounds = Some(WindowBounds::Maximized(Bounds {
                origin: Point::default(),
                size: size(px(1600.0), px(1000.0)),
            }));
            options.titlebar = Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: Some(point(px(12.0), px(11.0))),
            });
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
