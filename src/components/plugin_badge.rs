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

/// `size`: the label's text size; the dot follows it. A badge, not a
/// button (ERR-197): rounded and filled, no border.
pub fn badge_chip(b: &BadgeView, size: f32) -> Div {
    let c = color(b.state);
    crate::components::button::badge(c)
        .gap(px(size * 0.45))
        .px(px(size * 0.6))
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .child(div().size(px(size * 0.6)).rounded_full().bg(c))
        .child(div().max_w(px(220.0)).truncate().child(b.text.clone()))
}
