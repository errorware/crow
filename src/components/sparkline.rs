use gpui_kit::*;

pub fn sparkline(points: &'static [(f32, f32)], stroke_color: Rgba) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            if points.is_empty() {
                return;
            }

            let mut path = PathBuilder::stroke(px(1.2));
            let scale_x = bounds.size.width / px(86.0);
            let scale_y = bounds.size.height / px(26.0);

            let first = points[0];
            let p0 = point(
                bounds.origin.x + px(first.0) * scale_x,
                bounds.origin.y + px(first.1) * scale_y,
            );
            path.move_to(p0);

            for pt in &points[1..] {
                let p = point(
                    bounds.origin.x + px(pt.0) * scale_x,
                    bounds.origin.y + px(pt.1) * scale_y,
                );
                path.line_to(p);
            }

            if let Ok(built_path) = path.build() {
                window.paint_path(built_path, stroke_color);
            }
        },
    )
    .w(px(86.0))
    .h(px(26.0))
    .flex_none()
}
