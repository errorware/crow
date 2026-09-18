use gpui_kit::*;

#[allow(dead_code)]
pub fn sparkline(points: &'static [(f32, f32)], stroke_color: Rgba) -> impl IntoElement {
    sparkline_points(points.to_vec(), stroke_color)
}

pub fn sparkline_points(points: Vec<(f32, f32)>, stroke_color: Rgba) -> impl IntoElement {
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

/// Renders a dynamic sparkline from a series of metric values (e.g. CPU %, RAM %, Load).
/// Values are automatically normalized between min_bound and max_bound into the 86x26 canvas.
pub fn dynamic_sparkline(
    values: &[f32],
    min_bound: Option<f32>,
    max_bound: Option<f32>,
    stroke_color: Rgba,
) -> impl IntoElement {
    if values.is_empty() {
        return sparkline_points(vec![(0.0, 24.0), (86.0, 24.0)], stroke_color);
    }

    let min_v = min_bound.unwrap_or_else(|| {
        values.iter().cloned().fold(f32::INFINITY, f32::min)
    });
    let max_v = max_bound.unwrap_or_else(|| {
        values.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    });
    let range = (max_v - min_v).max(0.001);

    let len = values.len();
    let step_x = if len > 1 {
        86.0 / (len - 1) as f32
    } else {
        86.0
    };

    let points: Vec<(f32, f32)> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = i as f32 * step_x;
            let normalized = ((v - min_v) / range).clamp(0.0, 1.0);
            // Canvas Y=0 is top, Y=26 is bottom. Reserve 2px padding top/bottom.
            let y = 24.0 - (normalized * 21.0);
            (x, y)
        })
        .collect();

    sparkline_points(points, stroke_color)
}
