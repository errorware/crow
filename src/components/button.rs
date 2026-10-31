//! One design language (ERR-197): anything you click is a flat square with a
//! 1px border; anything you only read is a [`badge`], rounded and filled,
//! with no border. Never the other way round.
//!
//! The accent (`Tone::Primary`) is for the one action a dialog is for; the
//! rest are `Neutral`. `Danger` is for what deletes or reverts.

use gpui_kit::*;

use crate::theme::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Neutral,
    Primary,
    Danger,
}

impl Tone {
    fn ink(self) -> Rgba {
        match self {
            Tone::Neutral => TEXT_SECONDARY,
            Tone::Primary => OK,
            Tone::Danger => CRIT,
        }
    }

    fn edge(self) -> Rgba {
        match self {
            Tone::Neutral => BORDER_STRONG,
            Tone::Primary => OK.opacity(0.6),
            Tone::Danger => CRIT.opacity(0.5),
        }
    }

    fn hover(self) -> Rgba {
        match self {
            Tone::Neutral => BG_ROW_HOVER,
            Tone::Primary => OK_BG,
            Tone::Danger => CRIT_ROW_BG,
        }
    }
}

/// A button: give it `.on_click` and a label or icon child. Padding and text
/// size can be overridden after for small inline ones.
pub fn button(id: impl Into<ElementId>, tone: Tone) -> Stateful<Div> {
    div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(4.0))
        .border_1()
        .border_color(tone.edge())
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(tone.ink())
        .cursor_pointer()
        .hover(move |s| s.bg(tone.hover()).text_color(if tone == Tone::Neutral { TEXT_PRIMARY } else { tone.ink() }))
}

/// Something to read, not press: a rounded pill tinted in `color`, no
/// border. Never give it an `.on_click`.
pub fn badge(color: Rgba) -> Div {
    div()
        .flex_none()
        .flex()
        .items_center()
        .gap(px(4.0))
        .px(px(6.0))
        .py(px(1.0))
        .rounded_full()
        .bg(color.opacity(0.14))
        .font_family(FONT_MONO)
        .text_size(px(9.0))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
}
