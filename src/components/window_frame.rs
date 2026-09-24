//! Resize edges and the double-click maximize for Crow's client-drawn window
//! (Linux: no system titlebar or frame, so the app provides both).

use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;

/// Width of the invisible grab area around the window.
const RESIZE_BORDER: Pixels = px(6.0);

/// Which edge or corner of a `size` window `pos` is on, if any.
pub fn resize_edge(pos: Point<Pixels>, border: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let corner = border * 3.0;
    let (top, bottom) = (pos.y < border, pos.y > size.height - border);
    let (left, right) = (pos.x < border, pos.x > size.width - border);
    let (near_top, near_bottom) = (pos.y < corner, pos.y > size.height - corner);
    let (near_left, near_right) = (pos.x < corner, pos.x > size.width - corner);
    Some(match () {
        _ if (top && near_left) || (left && near_top) => ResizeEdge::TopLeft,
        _ if (top && near_right) || (right && near_top) => ResizeEdge::TopRight,
        _ if (bottom && near_left) || (left && near_bottom) => ResizeEdge::BottomLeft,
        _ if (bottom && near_right) || (right && near_bottom) => ResizeEdge::BottomRight,
        _ if top => ResizeEdge::Top,
        _ if bottom => ResizeEdge::Bottom,
        _ if left => ResizeEdge::Left,
        _ if right => ResizeEdge::Right,
        _ => return None,
    })
}

fn cursor_for(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

/// Wraps the app in an invisible resize border when the window draws its own
/// decorations and isn't maximized, tiled or fullscreen.
pub fn window_frame(window: &mut Window, content: impl IntoElement) -> impl IntoElement {
    let resizable = match window.window_decorations() {
        Decorations::Client { tiling } => !(window.is_maximized() || window.is_fullscreen() || tiling.top || tiling.bottom || tiling.left || tiling.right),
        Decorations::Server => false,
    };
    window.set_client_inset(if resizable { RESIZE_BORDER } else { px(0.0) });

    div()
        .id("window-frame")
        .size_full()
        .when(resizable, |frame| {
            frame
                .p(RESIZE_BORDER)
                // Resize cursors over the border.
                .child(
                    canvas(
                        |_bounds, window, _cx| {
                            let size = window.window_bounds().get_bounds().size;
                            window.insert_hitbox(Bounds::new(point(px(0.0), px(0.0)), size), HitboxBehavior::Normal)
                        },
                        |_bounds, hitbox, window, _cx| {
                            let size = window.window_bounds().get_bounds().size;
                            if let Some(edge) = resize_edge(window.mouse_position(), RESIZE_BORDER, size) {
                                window.set_cursor_style(cursor_for(edge), &hitbox);
                            }
                        },
                    )
                    .size_full()
                    .absolute(),
                )
                .on_mouse_move(|_ev, window, _cx| window.refresh())
                .on_mouse_down(MouseButton::Left, |ev, window, _cx| {
                    let size = window.window_bounds().get_bounds().size;
                    if let Some(edge) = resize_edge(ev.position, RESIZE_BORDER, size) {
                        window.start_window_resize(edge);
                    }
                })
        })
        .child(content)
}

/// Titlebar mouse-down: a double click toggles maximize, a single press
/// hands the window to the compositor to move. (Once the move starts, the
/// compositor owns the pointer, so a separate double-click handler would
/// never see the second click.)
pub fn titlebar_mouse_down(ev: &MouseDownEvent, window: &mut Window) {
    if ev.click_count >= 2 {
        window.zoom_window();
    } else {
        window.start_window_move();
    }
}

#[cfg(test)]
mod tests {
    use super::resize_edge;
    use gpui_kit::{point, px, size, ResizeEdge};

    #[test]
    fn edges_and_corners_are_detected_inside_the_border() {
        let s = size(px(1000.0), px(800.0));
        let b = px(6.0);
        assert_eq!(resize_edge(point(px(500.0), px(2.0)), b, s), Some(ResizeEdge::Top));
        assert_eq!(resize_edge(point(px(500.0), px(797.0)), b, s), Some(ResizeEdge::Bottom));
        assert_eq!(resize_edge(point(px(3.0), px(400.0)), b, s), Some(ResizeEdge::Left));
        assert_eq!(resize_edge(point(px(997.0), px(400.0)), b, s), Some(ResizeEdge::Right));
        assert_eq!(resize_edge(point(px(2.0), px(10.0)), b, s), Some(ResizeEdge::TopLeft), "corner zone is wider than the border");
        assert_eq!(resize_edge(point(px(995.0), px(798.0)), b, s), Some(ResizeEdge::BottomRight));
        assert_eq!(resize_edge(point(px(500.0), px(400.0)), b, s), None);
    }
}
