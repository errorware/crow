//! A user plugin's badge (ERR-182): a dot in its state's colour and its
//! short label, as on the identity bar, the fleet table and the map.

use gpui_kit::*;

use crate::plugin_host::badges::BadgeView;
use crate::plugin_host::run::State;
use crate::theme::*;

pub fn color(state: State) -> Rgba {
    match state {
        State::Ok => OK,
        State::Warn => WARN,
        State::Bad => CRIT,
        State::Unknown => TEXT_DIM,
    }
}

/// `size`: the label's text size; the dot follows it.
pub fn badge_chip(b: &BadgeView, size: f32) -> Div {
    let c = color(b.state);
    div()
        .flex_none()
        .flex()
        .items_center()
        .gap(px(size * 0.45))
        .px(px(size * 0.5))
        .py(px(1.0))
        .border_1()
        .border_color(c.opacity(0.5))
        .font_family(FONT_MONO)
        .text_size(px(size))
        .text_color(c)
        .child(div().size(px(size * 0.6)).rounded_full().bg(c))
        .child(div().max_w(px(220.0)).truncate().child(b.text.clone()))
}
