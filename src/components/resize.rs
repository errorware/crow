//! A horizontal divider you drag to resize the panel below it (Fleet's
//! alerts/activity, Processes' log). The container that holds both calls
//! `on_drag_move::<BottomPanelResize>` and turns the pointer position into a
//! height with `bottom_panel_height`.

use gpui_kit::*;

use crate::theme::*;

/// Drag payload for a bottom panel's divider.
#[derive(Clone)]
pub struct BottomPanelResize;

/// Drawn under the pointer while dragging a divider: nothing.
pub struct ResizeGhost;

impl Render for ResizeGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The divider itself.
pub fn resize_handle(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(5.0))
        .flex_none()
        .bg(BORDER_PANEL)
        .cursor(CursorStyle::ResizeUpDown)
        .hover(|s| s.bg(BORDER_STRONG))
        .on_drag(BottomPanelResize, |_, _, _, cx| cx.new(|_| ResizeGhost))
}

/// The panel height for a drag event over `bounds` (the container holding
/// the content above and the panel below): the distance from the pointer to
/// the bottom, kept between `min` and leaving `keep_above` for the content.
pub fn bottom_panel_height(ev: &DragMoveEvent<BottomPanelResize>, min: f32, keep_above: f32) -> f32 {
    let height = f32::from(ev.bounds.bottom() - ev.event.position.y);
    let max = f32::from(ev.bounds.size.height) - keep_above;
    height.clamp(min, max.max(min))
}
