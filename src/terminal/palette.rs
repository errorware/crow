//! Terminal colors (ERR-93): the ANSI 16 tuned for Crow's dark UI, the
//! xterm 256-color cube and greys, truecolor as given, and any colors the
//! program set with OSC 4/10/11 taking precedence.

use alacritty_terminal::term::color::Colors;
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};

/// Default foreground, background and cursor (0xRRGGBB).
pub const FOREGROUND: u32 = 0xd4d4d8;
pub const BACKGROUND: u32 = 0x0a0a0c;
/// Selected text is washed with this.
pub const SELECTION: u32 = 0x3ecf6e;
pub const CURSOR: u32 = 0xe4e4e7;

/// black, red, green, yellow, blue, magenta, cyan, white, then bright.
const ANSI: [u32; 16] = [
    0x18181b, 0xf87171, 0x4ade80, 0xfbbf24, 0x60a5fa, 0xc084fc, 0x22d3ee, 0xd4d4d8,
    0x52525b, 0xfca5a5, 0x86efac, 0xfde68a, 0x93c5fd, 0xd8b4fe, 0x67e8f9, 0xfafafa,
];

fn rgb(c: u32) -> Rgb {
    Rgb { r: (c >> 16) as u8, g: (c >> 8) as u8, b: c as u8 }
}

pub fn to_u32(c: Rgb) -> u32 {
    (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32
}

/// The 256-color palette entry `i`.
pub fn indexed(i: u8) -> Rgb {
    match i {
        0..=15 => rgb(ANSI[i as usize]),
        16..=231 => {
            let i = i - 16;
            let step = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            Rgb { r: step(i / 36), g: step((i / 6) % 6), b: step(i % 6) }
        }
        232..=255 => {
            let v = 8 + (i - 232) * 10;
            Rgb { r: v, g: v, b: v }
        }
    }
}

/// Resolves a cell color; `bold` brightens the eight base colors, as most
/// terminals do.
pub fn resolve(color: Color, colors: &Colors, bold: bool) -> u32 {
    let named = |n: NamedColor| -> Rgb {
        if let Some(c) = colors[n] {
            return c;
        }
        match n {
            NamedColor::Foreground | NamedColor::BrightForeground => rgb(FOREGROUND),
            NamedColor::Background => rgb(BACKGROUND),
            NamedColor::Cursor => rgb(CURSOR),
            NamedColor::DimForeground => rgb(0x71717a),
            n if (n as usize) < 16 => indexed(n as u8),
            // Dim variants of the eight colors.
            n => {
                let base = indexed((n as usize).saturating_sub(NamedColor::DimBlack as usize) as u8);
                Rgb { r: base.r / 3 * 2, g: base.g / 3 * 2, b: base.b / 3 * 2 }
            }
        }
    };
    to_u32(match color {
        Color::Spec(c) => c,
        Color::Indexed(i) => colors[i as usize].unwrap_or_else(|| indexed(if bold && i < 8 { i + 8 } else { i })),
        Color::Named(n) if bold && (n as usize) < 8 => named(n.to_bright()),
        Color::Named(n) => named(n),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_and_greys_match_xterm() {
        assert_eq!(to_u32(indexed(16)), 0x000000);
        assert_eq!(to_u32(indexed(196)), 0xff0000);
        assert_eq!(to_u32(indexed(231)), 0xffffff);
        assert_eq!(to_u32(indexed(232)), 0x080808);
        assert_eq!(to_u32(indexed(255)), 0xeeeeee);
    }

    #[test]
    fn program_set_colors_win_and_bold_brightens() {
        let mut colors = Colors::default();
        assert_eq!(resolve(Color::Named(NamedColor::Red), &colors, false), 0xf87171);
        assert_eq!(resolve(Color::Named(NamedColor::Red), &colors, true), 0xfca5a5);
        colors[NamedColor::Background] = Some(Rgb { r: 1, g: 2, b: 3 });
        assert_eq!(resolve(Color::Named(NamedColor::Background), &colors, false), 0x010203);
        assert_eq!(resolve(Color::Spec(Rgb { r: 0x12, g: 0x34, b: 0x56 }), &colors, false), 0x123456);
    }
}
