//! The one small icon button (ERR-104): a fixed square centering an icon,
//! dim until hovered, with no border. Destructive ones (delete) tint red on
//! hover; the rest (close, clear, dismiss, reset) brighten.
//!
//! Row actions are hidden until the row is hovered: give the row a
//! `.group(name)` and the button `.invisible().group_hover(name, |s| s.visible())`.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::components::icons::{inherited_icon, TablerIcon};
use crate::theme::*;

pub fn icon_button(id: impl Into<ElementId>, icon: TablerIcon, destructive: bool) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(20.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_color(text_dim())
        .map(|d| if destructive { d.hover(|s| s.bg(theme().crit_row_bg).text_color(theme().crit)) } else { d.hover(|s| s.bg(bg_row_hover()).text_color(text_primary())) })
        .child(inherited_icon(icon, font_px(12.0)).size(px(12.0)).justify_center())
}
