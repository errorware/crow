#![allow(dead_code)]
use gpui_kit::*;

pub const fn hex_rgb(hex: u32) -> Rgba {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    Rgba { r, g, b, a: 1.0 }
}

pub const fn hex_rgba(hex: u32, a: f32) -> Rgba {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    Rgba { r, g, b, a }
}

pub fn left_indicator(color: Rgba) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .w(px(3.0))
        .bg(color)
}

// Surfaces
pub const BG_WINDOW: Rgba = hex_rgb(0x050507);
pub const BG_APP: Rgba = hex_rgb(0x0a0a0c);
pub const BG_CHROME: Rgba = hex_rgb(0x07070a);
pub const BG_PANEL: Rgba = hex_rgb(0x0b0b0e);
pub const BG_RAIL: Rgba = hex_rgb(0x08080b);
pub const BG_SUBHEAD: Rgba = hex_rgb(0x0c0c10);
pub const BG_ROW_ALT: Rgba = hex_rgb(0x0b0b0e);
pub const BG_ROW_HOVER: Rgba = hex_rgb(0x101116);
pub const BG_ROW_SELECTED: Rgba = hex_rgb(0x15161b);
pub const BG_NAV_ACTIVE: Rgba = hex_rgb(0x131419);
pub const BG_OVERLAY_PANEL: Rgba = hex_rgb(0x0d0e12);
pub const BG_CONTROL: Rgba = hex_rgb(0x15161b);
pub const BG_CONTROL_ALT: Rgba = hex_rgb(0x1a1b21);
pub const BG_CHIP: Rgba = hex_rgb(0x1e1f25);
pub const BG_KEY: Rgba = hex_rgb(0x181920);

// Borders
pub const BORDER_STRONG: Rgba = hex_rgb(0x2c2d35);
pub const BORDER_DEFAULT: Rgba = hex_rgb(0x23242b);
pub const BORDER_PANEL: Rgba = hex_rgb(0x1c1d22);
pub const BORDER_ROW: Rgba = hex_rgb(0x101116);
pub const BORDER_KEY: Rgba = hex_rgb(0x26272e);
pub const BORDER_CONTROL_SEL: Rgba = hex_rgb(0x3a3c46);
pub const BORDER_DANGER: Rgba = hex_rgb(0x2a1214);
pub const BORDER_DANGER_BTN: Rgba = hex_rgb(0x3a1a1d);
pub const BORDER_DANGER_SEL: Rgba = hex_rgb(0x5a2024);

// Text
pub const TEXT_MAX: Rgba = hex_rgb(0xffffff);
pub const TEXT_PRIMARY: Rgba = hex_rgb(0xe6e7ea);
pub const TEXT_SECONDARY: Rgba = hex_rgb(0xc9cbd2);
pub const TEXT_TERTIARY: Rgba = hex_rgb(0x9a9da6);
pub const TEXT_MUTED: Rgba = hex_rgb(0x8a8d96);
pub const TEXT_DIM: Rgba = hex_rgb(0x7a7d86);
pub const TEXT_DIMMER: Rgba = hex_rgb(0x6c6f78);
pub const TEXT_FAINT: Rgba = hex_rgb(0x5a5d66);
pub const TEXT_FAINTER: Rgba = hex_rgb(0x4c4f58);
pub const TEXT_GHOST: Rgba = hex_rgb(0x3b3c42);

// Semantic Signals
pub const OK: Rgba = hex_rgb(0x3ecf6e);
pub const WARN: Rgba = hex_rgb(0xe0a020);
pub const CRIT: Rgba = hex_rgb(0xe5484d);

pub const OK_BG: Rgba = hex_rgb(0x16301f);
pub const WARN_BG: Rgba = hex_rgb(0x2e2210);
pub const CRIT_BG: Rgba = hex_rgb(0x2e1114);
pub const CRIT_ROW_BG: Rgba = hex_rgb(0x140d0e);
pub const CRIT_LOG_BG: Rgba = hex_rgb(0x120c0d);
pub const CRIT_STRIP_BG: Rgba = hex_rgb(0x0c0809);
pub const CRIT_INK: Rgba = hex_rgb(0xf0b5b7);
pub const CRIT_INK_DIM: Rgba = hex_rgb(0xc1898b);
pub const WARN_INK: Rgba = hex_rgb(0xe6cfa3);
pub const OK_INK: Rgba = hex_rgb(0xa8e0bf);

pub const DIFF_DEL_BG: Rgba = hex_rgb(0x150e0f);
pub const DIFF_ADD_BG: Rgba = hex_rgb(0x0d1611);
pub const DIFF_HUNK_BG: Rgba = hex_rgb(0x101116);

// Fonts — Unified Monospace Default across the entire application
pub const FONT_MONO: &str = "JetBrains Mono";
pub const FONT_SANS: &str = "JetBrains Mono";

/// Initialize and enforce the Obsidian Edge dark theme across gpui-component & gpui-kit.
/// Replaces the default light theme with dark backgrounds, legible high-contrast text,
/// emerald caret, and sharp terminal corners (2px radius).
pub fn init_obsidian_theme(cx: &mut App) {
    use std::sync::Arc;
    use gpui_kit::component::theme::{Theme, ThemeMode};
    use gpui_kit::component::highlighter::HighlightTheme;

    // Switch gpui-component from default light mode to dark mode
    Theme::change(ThemeMode::Dark, None, cx);

    let theme = Theme::global_mut(cx);
    // Terminal aesthetic: sharp 2px corners instead of bubbly rounded pills
    theme.radius = px(2.0);
    theme.radius_lg = px(4.0);
    theme.font_family = FONT_MONO.into();
    theme.mono_font_family = FONT_MONO.into();
    // Long lists show their scrollbar at all times, not only while scrolling.
    theme.scrollbar_mode = gpui_kit::component::scroll::ScrollbarMode::Always;

    // Obsidian Edge dark palette
    theme.background = BG_APP.into();
    theme.foreground = TEXT_MAX.into();
    theme.muted_foreground = TEXT_MUTED.into();
    theme.input = BORDER_DEFAULT.into();
    theme.border = BORDER_DEFAULT.into();
    theme.caret = OK.into();
    theme.ring = OK.into();
    theme.selection = hex_rgba(0x38bdf8, 0.40).into();

    // Configure the editor highlight theme for solid dark background and crisp white foreground
    let mut highlight_style = theme.highlight_theme.style.clone();
    highlight_style.editor_background = Some(BG_APP.into());
    highlight_style.editor_foreground = Some(TEXT_MAX.into());
    theme.highlight_theme = Arc::new(HighlightTheme {
        name: "Obsidian Edge".to_string(),
        appearance: ThemeMode::Dark,
        style: highlight_style,
    });

    // Re-project semantic tokens onto the base layer
    Theme::sync_base(cx);
}

#[cfg(test)]
mod scrollbar_guard {
    /// GPUI's `overflow_y_scroll` scrolls without drawing a scrollbar (ERR-17);
    /// every scrolling list uses `overflow_y_scrollbar` instead.
    #[test]
    fn no_list_scrolls_without_a_scrollbar() {
        fn visit(dir: &std::path::Path, hits: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit(&path, hits);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    for (i, line) in text.lines().enumerate() {
                        if line.contains(concat!(".overflow_y_", "scroll()")) {
                            hits.push(format!("{}:{}", path.display(), i + 1));
                        }
                    }
                }
            }
        }
        let mut hits = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut hits);
        assert!(hits.is_empty(), "use .overflow_y_scrollbar() instead: {hits:?}");
    }
}
