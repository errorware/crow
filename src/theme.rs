#![allow(dead_code)]
use std::sync::OnceLock;
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

#[derive(Clone, Debug)]
pub struct ThemePalette {
    pub id: &'static str,
    pub name: &'static str,
    pub is_dark: bool,

    // Surfaces
    pub bg_window: Rgba,
    pub bg_app: Rgba,
    pub bg_chrome: Rgba,
    pub bg_panel: Rgba,
    pub bg_rail: Rgba,
    pub bg_subhead: Rgba,
    pub bg_row_alt: Rgba,
    pub bg_row_hover: Rgba,
    pub bg_row_selected: Rgba,
    pub bg_nav_active: Rgba,
    pub bg_overlay_panel: Rgba,
    pub bg_control: Rgba,
    pub bg_control_alt: Rgba,
    pub bg_chip: Rgba,
    pub bg_key: Rgba,

    // Borders
    pub border_strong: Rgba,
    pub border_default: Rgba,
    pub border_panel: Rgba,
    pub border_row: Rgba,
    pub border_key: Rgba,
    pub border_control_sel: Rgba,
    pub border_danger: Rgba,
    pub border_danger_btn: Rgba,
    pub border_danger_sel: Rgba,

    // Text
    pub text_max: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_tertiary: Rgba,
    pub text_muted: Rgba,
    pub text_dim: Rgba,
    pub text_dimmer: Rgba,
    pub text_faint: Rgba,
    pub text_fainter: Rgba,
    pub text_ghost: Rgba,

    // Chrome Text (Titlebar & Status Bar)
    pub text_chrome: Rgba,
    pub text_chrome_muted: Rgba,
    pub text_chrome_dim: Rgba,

    // Semantic Signals
    pub ok: Rgba,
    pub warn: Rgba,
    pub crit: Rgba,
    pub ok_bg: Rgba,
    pub warn_bg: Rgba,
    pub crit_bg: Rgba,
    pub crit_row_bg: Rgba,
    pub crit_log_bg: Rgba,
    pub crit_strip_bg: Rgba,
    pub crit_ink: Rgba,
    pub crit_ink_dim: Rgba,
    pub warn_ink: Rgba,
    pub ok_ink: Rgba,

    pub diff_del_bg: Rgba,
    pub diff_add_bg: Rgba,
    pub diff_hunk_bg: Rgba,

    // Accent Colors
    pub accent: Rgba,
    pub accent_bg: Rgba,
    pub accent_border: Rgba,
    pub accent_ink: Rgba,

    // Syntax Highlighting Colors
    pub syntax_key: Rgba,
    pub syntax_header: Rgba,
    pub syntax_string: Rgba,
    pub syntax_number: Rgba,
    pub syntax_boolean: Rgba,
    pub syntax_keyword: Rgba,
    pub syntax_comment: Rgba,
    pub syntax_operator: Rgba,
    pub syntax_ip: Rgba,
    pub syntax_special: Rgba,
}

pub fn obsidian_edge_palette() -> ThemePalette {
    ThemePalette {
        id: "obsidian_edge",
        name: "Obsidian Edge",
        is_dark: true,

        bg_window: hex_rgb(0x050507),
        bg_app: hex_rgb(0x0a0a0c),
        bg_chrome: hex_rgb(0x07070a),
        bg_panel: hex_rgb(0x0b0b0e),
        bg_rail: hex_rgb(0x08080b),
        bg_subhead: hex_rgb(0x0c0c10),
        bg_row_alt: hex_rgb(0x0b0b0e),
        bg_row_hover: hex_rgb(0x101116),
        bg_row_selected: hex_rgb(0x15161b),
        bg_nav_active: hex_rgb(0x131419),
        bg_overlay_panel: hex_rgb(0x0d0e12),
        bg_control: hex_rgb(0x15161b),
        bg_control_alt: hex_rgb(0x1a1b21),
        bg_chip: hex_rgb(0x1e1f25),
        bg_key: hex_rgb(0x181920),

        border_strong: hex_rgb(0x2c2d35),
        border_default: hex_rgb(0x23242b),
        border_panel: hex_rgb(0x1c1d22),
        border_row: hex_rgb(0x101116),
        border_key: hex_rgb(0x26272e),
        border_control_sel: hex_rgb(0x3a3c46),
        border_danger: hex_rgb(0x2a1214),
        border_danger_btn: hex_rgb(0x3a1a1d),
        border_danger_sel: hex_rgb(0x5a2024),

        text_max: hex_rgb(0xffffff),
        text_primary: hex_rgb(0xe6e7ea),
        text_secondary: hex_rgb(0xc9cbd2),
        text_tertiary: hex_rgb(0x9a9da6),
        text_muted: hex_rgb(0x8a8d96),
        text_dim: hex_rgb(0x7a7d86),
        text_dimmer: hex_rgb(0x6c6f78),
        text_faint: hex_rgb(0x5a5d66),
        text_fainter: hex_rgb(0x4c4f58),
        text_ghost: hex_rgb(0x3b3c42),

        text_chrome: hex_rgb(0xffffff),
        text_chrome_muted: hex_rgb(0x9ca3af),
        text_chrome_dim: hex_rgb(0x6b7280),

        ok: hex_rgb(0x3ecf6e),
        warn: hex_rgb(0xe0a020),
        crit: hex_rgb(0xe5484d),

        ok_bg: hex_rgb(0x16301f),
        warn_bg: hex_rgb(0x2e2210),
        crit_bg: hex_rgb(0x2e1114),
        crit_row_bg: hex_rgb(0x140d0e),
        crit_log_bg: hex_rgb(0x120c0d),
        crit_strip_bg: hex_rgb(0x0c0809),
        crit_ink: hex_rgb(0xf0b5b7),
        crit_ink_dim: hex_rgb(0xc1898b),
        warn_ink: hex_rgb(0xe6cfa3),
        ok_ink: hex_rgb(0xa8e0bf),

        diff_del_bg: hex_rgb(0x150e0f),
        diff_add_bg: hex_rgb(0x0d1611),
        diff_hunk_bg: hex_rgb(0x101116),

        accent: hex_rgb(0x38bdf8),
        accent_bg: hex_rgba(0x38bdf8, 0.15),
        accent_border: hex_rgba(0x38bdf8, 0.40),
        accent_ink: hex_rgb(0x38bdf8),

        syntax_key: hex_rgb(0x67e8f9),
        syntax_header: hex_rgb(0xa78bfa),
        syntax_string: hex_rgb(0x86efac),
        syntax_number: hex_rgb(0xfde047),
        syntax_boolean: hex_rgb(0xf472b6),
        syntax_keyword: hex_rgb(0xc084fc),
        syntax_comment: hex_rgb(0x6b7280),
        syntax_operator: hex_rgb(0x94a3b8),
        syntax_ip: hex_rgb(0x38bdf8),
        syntax_special: hex_rgb(0xfb923c),
    }
}

pub fn obsidian_light_palette() -> ThemePalette {
    ThemePalette {
        id: "obsidian_light",
        name: "Obsidian Light",
        is_dark: false,

        bg_window: hex_rgb(0xf0f1f5),
        bg_app: hex_rgb(0xf8f9fa),
        bg_chrome: hex_rgb(0xedeff2),
        bg_panel: hex_rgb(0xffffff),
        bg_rail: hex_rgb(0xf1f3f6),
        bg_subhead: hex_rgb(0xedf0f4),
        bg_row_alt: hex_rgb(0xf6f7f9),
        bg_row_hover: hex_rgb(0xebeef3),
        bg_row_selected: hex_rgb(0xe0e4eb),
        bg_nav_active: hex_rgb(0xe5e9f0),
        bg_overlay_panel: hex_rgb(0xffffff),
        bg_control: hex_rgb(0xffffff),
        bg_control_alt: hex_rgb(0xeff1f5),
        bg_chip: hex_rgb(0xe5e7eb),
        bg_key: hex_rgb(0xe2e5ea),

        border_strong: hex_rgb(0xc4c8d1),
        border_default: hex_rgb(0xd8dbe2),
        border_panel: hex_rgb(0xe5e7ec),
        border_row: hex_rgb(0xeff1f4),
        border_key: hex_rgb(0xcfd3dc),
        border_control_sel: hex_rgb(0xa8adb8),
        border_danger: hex_rgb(0xfecdd3),
        border_danger_btn: hex_rgb(0xf87171),
        border_danger_sel: hex_rgb(0xef4444),

        text_max: hex_rgb(0x000000),
        text_primary: hex_rgb(0x0f172a),
        text_secondary: hex_rgb(0x1e293b),
        text_tertiary: hex_rgb(0x334155),
        text_muted: hex_rgb(0x334155),
        text_dim: hex_rgb(0x334155),
        text_dimmer: hex_rgb(0x475569),
        text_faint: hex_rgb(0x475569),
        text_fainter: hex_rgb(0x526075),
        text_ghost: hex_rgb(0x64748b),

        text_chrome: hex_rgb(0x0f172a),
        text_chrome_muted: hex_rgb(0x334155),
        text_chrome_dim: hex_rgb(0x475569),

        ok: hex_rgb(0x166534),
        warn: hex_rgb(0xb45309),
        crit: hex_rgb(0xb91c1c),

        ok_bg: hex_rgb(0xdcfce7),
        warn_bg: hex_rgb(0xfef3c7),
        crit_bg: hex_rgb(0xfee2e2),
        crit_row_bg: hex_rgb(0xfff1f2),
        crit_log_bg: hex_rgb(0xfff5f5),
        crit_strip_bg: hex_rgb(0xfef2f2),
        crit_ink: hex_rgb(0x7f1d1d),
        crit_ink_dim: hex_rgb(0x991b1b),
        warn_ink: hex_rgb(0x78350f),
        ok_ink: hex_rgb(0x14532d),

        diff_del_bg: hex_rgb(0xffe4e6),
        diff_add_bg: hex_rgb(0xdcfce7),
        diff_hunk_bg: hex_rgb(0xf1f5f9),

        accent: hex_rgb(0x1d4ed8),
        accent_bg: hex_rgba(0x1d4ed8, 0.10),
        accent_border: hex_rgba(0x1d4ed8, 0.35),
        accent_ink: hex_rgb(0x1e40af),

        syntax_key: hex_rgb(0x0369a1),
        syntax_header: hex_rgb(0x6d28d9),
        syntax_string: hex_rgb(0x15803d),
        syntax_number: hex_rgb(0xb45309),
        syntax_boolean: hex_rgb(0xbe185d),
        syntax_keyword: hex_rgb(0x7c3aed),
        syntax_comment: hex_rgb(0x64748b),
        syntax_operator: hex_rgb(0x334155),
        syntax_ip: hex_rgb(0x1d4ed8),
        syntax_special: hex_rgb(0xc2410c),
    }
}

pub fn crow_cde_palette() -> ThemePalette {
    ThemePalette {
        id: "crow_cde",
        name: "Crow CDE (DIGITAL UNIX 1996)",
        is_dark: false,

        bg_window: hex_rgb(0x5B6E8C),         // Desktop blue-grey
        bg_app: hex_rgb(0xAEB2C3),            // Base CDE lavender-grey surface
        bg_chrome: hex_rgb(0x75597A),         // Tru64 plum/mauve titlebar
        bg_panel: hex_rgb(0xD9DBE3),          // Inset/sunken wells
        bg_rail: hex_rgb(0xD9DBE3),           // Inset/sunken rail well
        bg_subhead: hex_rgb(0xBFC3D4),
        bg_row_alt: hex_rgb(0xD0D3DE),
        bg_row_hover: hex_rgb(0xC4C7D3),
        bg_row_selected: hex_rgb(0x8E92A3),   // Selected item well
        bg_nav_active: hex_rgb(0x9A9EB0),
        bg_overlay_panel: hex_rgb(0xD9DBE3),
        bg_control: hex_rgb(0xAEB2C3),        // Raised button
        bg_control_alt: hex_rgb(0x9A9EB0),    // Pressed button
        bg_chip: hex_rgb(0x9A9EB0),
        bg_key: hex_rgb(0x9A9EB0),

        border_strong: hex_rgb(0x2A2033),
        border_default: hex_rgb(0x50546A),    // Dark bevel shadow
        border_panel: hex_rgb(0x8E92A3),
        border_row: hex_rgb(0xB9BCC9),
        border_key: hex_rgb(0x50546A),
        border_control_sel: hex_rgb(0x000000),
        border_danger: hex_rgb(0xA3261E),
        border_danger_btn: hex_rgb(0x50546A),
        border_danger_sel: hex_rgb(0xA3261E),

        text_max: hex_rgb(0x000000),
        text_primary: hex_rgb(0x000000),
        text_secondary: hex_rgb(0x1E2130),
        text_tertiary: hex_rgb(0x222636),
        text_muted: hex_rgb(0x282D40),
        text_dim: hex_rgb(0x282D40),
        text_dimmer: hex_rgb(0x2E344A),
        text_faint: hex_rgb(0x2E344A),
        text_fainter: hex_rgb(0x3A415C),
        text_ghost: hex_rgb(0x50546A),

        text_chrome: hex_rgb(0xffffff),
        text_chrome_muted: hex_rgb(0xF0F2F5),
        text_chrome_dim: hex_rgb(0xD9DBE3),

        ok: hex_rgb(0x2F7A3A),
        warn: hex_rgb(0x9A5B00),
        crit: hex_rgb(0xA3261E),

        ok_bg: hex_rgb(0xC4D6C7),
        warn_bg: hex_rgb(0xE2D4BE),
        crit_bg: hex_rgb(0xE0C4C4),
        crit_row_bg: hex_rgb(0xD9C6C6),
        crit_log_bg: hex_rgb(0xD9C6C6),
        crit_strip_bg: hex_rgb(0xD9C6C6),
        crit_ink: hex_rgb(0x7A1510),
        crit_ink_dim: hex_rgb(0x7A1510),
        warn_ink: hex_rgb(0x7A4000),
        ok_ink: hex_rgb(0x144D20),

        diff_del_bg: hex_rgb(0xD0BAC0),
        diff_add_bg: hex_rgb(0xBAD0BF),
        diff_hunk_bg: hex_rgb(0xC4C7D3),

        accent: hex_rgb(0x183B70),
        accent_bg: hex_rgba(0x183B70, 0.15),
        accent_border: hex_rgb(0x4A5C78),
        accent_ink: hex_rgb(0x0E264E),

        syntax_key: hex_rgb(0x0A4E7A),
        syntax_header: hex_rgb(0x522A70),
        syntax_string: hex_rgb(0x1B5E26),
        syntax_number: hex_rgb(0x8A4200),
        syntax_boolean: hex_rgb(0x7A1A40),
        syntax_keyword: hex_rgb(0x183B70),
        syntax_comment: hex_rgb(0x4C5262),
        syntax_operator: hex_rgb(0x1E212B),
        syntax_ip: hex_rgb(0x183B70),
        syntax_special: hex_rgb(0x8F1A14),
    }
}

static ACTIVE_PALETTE: OnceLock<ThemePalette> = OnceLock::new();

pub fn active_palette() -> &'static ThemePalette {
    ACTIVE_PALETTE.get_or_init(obsidian_edge_palette)
}

#[inline(always)]
pub fn theme() -> &'static ThemePalette {
    active_palette()
}

#[inline(always)]
pub fn bg_window() -> Rgba { theme().bg_window }
#[inline(always)]
pub fn bg_app() -> Rgba { theme().bg_app }
#[inline(always)]
pub fn bg_chrome() -> Rgba { theme().bg_chrome }
#[inline(always)]
pub fn bg_panel() -> Rgba { theme().bg_panel }
#[inline(always)]
pub fn bg_rail() -> Rgba { theme().bg_rail }
#[inline(always)]
pub fn bg_subhead() -> Rgba { theme().bg_subhead }
#[inline(always)]
pub fn bg_row_alt() -> Rgba { theme().bg_row_alt }
#[inline(always)]
pub fn bg_row_hover() -> Rgba { theme().bg_row_hover }
#[inline(always)]
pub fn bg_row_selected() -> Rgba { theme().bg_row_selected }
#[inline(always)]
pub fn bg_nav_active() -> Rgba { theme().bg_nav_active }
#[inline(always)]
pub fn bg_overlay_panel() -> Rgba { theme().bg_overlay_panel }
#[inline(always)]
pub fn bg_control() -> Rgba { theme().bg_control }
#[inline(always)]
pub fn bg_control_alt() -> Rgba { theme().bg_control_alt }
#[inline(always)]
pub fn bg_chip() -> Rgba { theme().bg_chip }
#[inline(always)]
pub fn bg_key() -> Rgba { theme().bg_key }

#[inline(always)]
pub fn border_strong() -> Rgba { theme().border_strong }
#[inline(always)]
pub fn border_default() -> Rgba { theme().border_default }
#[inline(always)]
pub fn border_panel() -> Rgba { theme().border_panel }
#[inline(always)]
pub fn border_row() -> Rgba { theme().border_row }
#[inline(always)]
pub fn border_key() -> Rgba { theme().border_key }
#[inline(always)]
pub fn border_control_sel() -> Rgba { theme().border_control_sel }
#[inline(always)]
pub fn border_danger() -> Rgba { theme().border_danger }
#[inline(always)]
pub fn border_danger_btn() -> Rgba { theme().border_danger_btn }
#[inline(always)]
pub fn border_danger_sel() -> Rgba { theme().border_danger_sel }

#[inline(always)]
pub fn text_max() -> Rgba { theme().text_max }
#[inline(always)]
pub fn text_primary() -> Rgba { theme().text_primary }
#[inline(always)]
pub fn text_secondary() -> Rgba { theme().text_secondary }
#[inline(always)]
pub fn text_tertiary() -> Rgba { theme().text_tertiary }
#[inline(always)]
pub fn text_muted() -> Rgba { theme().text_muted }
#[inline(always)]
pub fn text_dim() -> Rgba { theme().text_dim }
#[inline(always)]
pub fn text_dimmer() -> Rgba { theme().text_dimmer }
#[inline(always)]
pub fn text_faint() -> Rgba { theme().text_faint }
#[inline(always)]
pub fn text_fainter() -> Rgba { theme().text_fainter }
#[inline(always)]
pub fn text_ghost() -> Rgba { theme().text_ghost }
#[inline(always)]
pub fn text_chrome() -> Rgba { theme().text_chrome }
#[inline(always)]
pub fn text_chrome_muted() -> Rgba { theme().text_chrome_muted }
#[inline(always)]
pub fn text_chrome_dim() -> Rgba { theme().text_chrome_dim }

#[inline(always)]
pub fn ok() -> Rgba { theme().ok }
#[inline(always)]
pub fn warn() -> Rgba { theme().warn }
#[inline(always)]
pub fn crit() -> Rgba { theme().crit }

#[inline(always)]
pub fn ok_bg() -> Rgba { theme().ok_bg }
#[inline(always)]
pub fn warn_bg() -> Rgba { theme().warn_bg }
#[inline(always)]
pub fn crit_bg() -> Rgba { theme().crit_bg }
#[inline(always)]
pub fn crit_row_bg() -> Rgba { theme().crit_row_bg }
#[inline(always)]
pub fn crit_log_bg() -> Rgba { theme().crit_log_bg }
#[inline(always)]
pub fn crit_strip_bg() -> Rgba { theme().crit_strip_bg }
#[inline(always)]
pub fn crit_ink() -> Rgba { theme().crit_ink }
#[inline(always)]
pub fn crit_ink_dim() -> Rgba { theme().crit_ink_dim }
#[inline(always)]
pub fn warn_ink() -> Rgba { theme().warn_ink }
#[inline(always)]
pub fn ok_ink() -> Rgba { theme().ok_ink }

#[inline(always)]
pub fn diff_del_bg() -> Rgba { theme().diff_del_bg }
#[inline(always)]
pub fn diff_add_bg() -> Rgba { theme().diff_add_bg }
#[inline(always)]
pub fn diff_hunk_bg() -> Rgba { theme().diff_hunk_bg }

#[inline(always)]
pub fn accent() -> Rgba { theme().accent }
#[inline(always)]
pub fn accent_bg() -> Rgba { theme().accent_bg }
#[inline(always)]
pub fn accent_border() -> Rgba { theme().accent_border }
#[inline(always)]
pub fn accent_ink() -> Rgba { theme().accent_ink }

#[inline(always)]
pub fn syntax_key() -> Rgba { theme().syntax_key }
#[inline(always)]
pub fn syntax_header() -> Rgba { theme().syntax_header }
#[inline(always)]
pub fn syntax_string() -> Rgba { theme().syntax_string }
#[inline(always)]
pub fn syntax_number() -> Rgba { theme().syntax_number }
#[inline(always)]
pub fn syntax_boolean() -> Rgba { theme().syntax_boolean }
#[inline(always)]
pub fn syntax_keyword() -> Rgba { theme().syntax_keyword }
#[inline(always)]
pub fn syntax_comment() -> Rgba { theme().syntax_comment }
#[inline(always)]
pub fn syntax_operator() -> Rgba { theme().syntax_operator }
#[inline(always)]
pub fn syntax_ip() -> Rgba { theme().syntax_ip }
#[inline(always)]
pub fn syntax_special() -> Rgba { theme().syntax_special }

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

/// Initialize and apply the selected theme across gpui-component & gpui-kit.
pub fn init_theme(theme_name: &str, cx: &mut App) {
    let palette = match theme_name {
        "obsidian_light" | "light" => obsidian_light_palette(),
        "crow_cde" | "cde" => crow_cde_palette(),
        _ => obsidian_edge_palette(),
    };
    let _ = ACTIVE_PALETTE.set(palette);
    apply_theme_to_gpui(active_palette(), cx);
}

/// Backward-compatible initializer for Obsidian Edge.
pub fn init_obsidian_theme(cx: &mut App) {
    init_theme("obsidian_edge", cx);
}

fn apply_theme_to_gpui(pal: &ThemePalette, cx: &mut App) {
    use std::sync::Arc;
    use gpui_kit::component::theme::{Theme, ThemeMode};
    use gpui_kit::component::highlighter::HighlightTheme;

    let mode = if pal.is_dark { ThemeMode::Dark } else { ThemeMode::Light };
    Theme::change(mode, None, cx);

    let theme = Theme::global_mut(cx);
    theme.radius = px(2.0);
    theme.radius_lg = px(4.0);
    theme.font_family = FONT_MONO.into();
    theme.mono_font_family = FONT_MONO.into();
    theme.scrollbar_mode = gpui_kit::component::scroll::ScrollbarMode::Always;

    theme.background = pal.bg_app.into();
    theme.foreground = pal.text_max.into();
    theme.muted_foreground = pal.text_muted.into();
    theme.input = pal.border_default.into();
    theme.border = pal.border_default.into();
    theme.caret = pal.ok.into();
    theme.ring = pal.ok.into();
    theme.selection = if pal.is_dark {
        hex_rgba(0x38bdf8, 0.40).into()
    } else {
        hex_rgba(0x2563eb, 0.25).into()
    };

    let mut highlight_style = theme.highlight_theme.style.clone();
    highlight_style.editor_background = Some(pal.bg_app.into());
    highlight_style.editor_foreground = Some(pal.text_primary.into());
    theme.highlight_theme = Arc::new(HighlightTheme {
        name: pal.name.to_string(),
        appearance: mode,
        style: highlight_style,
    });

    Theme::sync_base(cx);
}

use std::sync::atomic::{AtomicU32, Ordering};

static FONT_SCALE_BITS: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub fn set_font_scale(scale: f32) {
    FONT_SCALE_BITS.store(scale.to_bits(), Ordering::Relaxed);
}

pub fn font_scale() -> f32 {
    f32::from_bits(FONT_SCALE_BITS.load(Ordering::Relaxed))
}

/// Sizing for fonts and text elements that scales dynamically with GUI zoom / font size.
/// Evaluates to `rems(pixels / 16.0)` so that GPUI's `window.rem_size()` controls the actual pixel size.
#[inline]
pub fn font_px(pixels: f32) -> Rems {
    rems(pixels / 16.0)
}

/// Updates the theme's font sizes and the application font scale based on GUI font size (12 pt base).
pub fn update_theme_font_size(gui_font_size: f32, cx: &mut App) {
    use gpui_kit::component::theme::Theme;
    let scale = (gui_font_size / 12.0).clamp(0.8, 2.0);
    set_font_scale(scale);
    let theme = Theme::global_mut(cx);
    theme.font_size = px(16.0 * scale);
    theme.mono_font_size = px(13.0 * scale);
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

#[cfg(test)]
mod icon_guard {
    /// An svg icon doesn't inherit its parent's text colour: `tabler_icon(..)`
    /// without `.text_color(..)` renders as empty space. Use
    /// `inherited_icon(..)` to follow the surrounding text colour instead.
    #[test]
    fn every_svg_icon_has_a_colour() {
        fn visit(dir: &std::path::Path, hits: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit(&path, hits);
                } else if path.extension().is_some_and(|e| e == "rs") && !path.ends_with("icons.rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    for (i, line) in text.lines().enumerate() {
                        if line.contains(concat!("tabler_icon", "(TablerIcon::")) && !line.contains("text_color") {
                            // The colour may follow on the next lines of the chain.
                            let rest: String = text.lines().skip(i + 1).take(3).collect::<Vec<_>>().join(" ");
                            if !rest.trim_start().starts_with(".text_color") && !rest.contains(".text_color(") {
                                hits.push(format!("{}:{}", path.display(), i + 1));
                            }
                        }
                    }
                }
            }
        }
        let mut hits = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut hits);
        assert!(hits.is_empty(), "icons without a colour (invisible): {hits:?}");
    }
}

#[cfg(test)]
mod backdrop_guard {
    /// A dialog's dark backdrop must `.occlude()`: without it the mouse
    /// still hovers and clicks the page underneath.
    #[test]
    fn every_dialog_backdrop_blocks_the_page() {
        fn visit(dir: &std::path::Path, hits: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit(&path, hits);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    let lines: Vec<&str> = text.lines().collect();
                    for (i, line) in lines.iter().enumerate() {
                        if !line.contains(concat!(".inset", "_0()")) {
                            continue;
                        }
                        let near = lines[i.saturating_sub(4)..(i + 10).min(lines.len())].join(" ");
                        // A backdrop: dark, with a dialog centred on it.
                        let backdrop = near.contains("0x000000") && (near.contains(".justify_center()") || near.contains(".items_center()"));
                        if backdrop && !near.contains(concat!(".occ", "lude()")) {
                            hits.push(format!("{}:{}", path.display(), i + 1));
                        }
                    }
                }
            }
        }
        let mut hits = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut hits);
        assert!(hits.is_empty(), "dialog backdrops that let the mouse through: {hits:?}");
    }
}

#[cfg(test)]
mod shape_guard {
    /// One design language (ERR-197): what you click is square, what you
    /// read is a rounded badge. An element (roughly: from one `div()` to the
    /// next) that takes clicks must not be rounded. A dot or an input on the
    /// same line is not the element itself.
    #[test]
    fn nothing_clickable_is_rounded() {
        fn visit(dir: &std::path::Path, hits: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit(&path, hits);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    let lines: Vec<&str> = text.lines().collect();
                    let mut start = 0;
                    for i in 0..=lines.len() {
                        if i < lines.len() && !(i > start && lines[i].contains(concat!("div", "()"))) {
                            continue;
                        }
                        let seg = &lines[start..i];
                        let clickable = seg.iter().any(|l| l.contains(concat!("cursor_", "pointer")) || l.contains(concat!(".on_", "click(")));
                        let rounded = (0..seg.len()).find(|&j| {
                            let l = seg[j];
                            let input = seg[j.saturating_sub(6)..=j].iter().any(|p| p.contains("Input::new"));
                            l.contains(concat!(".roun", "ded")) && !input && !l.contains(".size(px(")
                        });
                        if let (true, Some(at)) = (clickable, rounded) {
                            hits.push(format!("{}:{}", path.display(), start + at + 1));
                        }
                        start = i;
                    }
                }
            }
        }
        let mut hits = Vec::new();
        visit(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut hits);
        assert!(hits.is_empty(), "rounded things you can click (make them square, or a badge you can't): {hits:?}");
    }
}

#[cfg(test)]
mod palette_tests {
    use super::*;

    #[test]
    fn test_theme_palettes_integrity() {
        let dark = obsidian_edge_palette();
        assert_eq!(dark.id, "obsidian_edge");
        assert!(dark.is_dark);
        assert_eq!(dark.text_max, hex_rgb(0xffffff));

        let light = obsidian_light_palette();
        assert_eq!(light.id, "obsidian_light");
        assert!(!light.is_dark);
        assert_eq!(light.text_max, hex_rgb(0x000000));

        let cde = crow_cde_palette();
        assert_eq!(cde.id, "crow_cde");
        assert!(!cde.is_dark);
        assert_eq!(cde.bg_chrome, hex_rgb(0x75597A));
        assert_eq!(cde.bg_app, hex_rgb(0xAEB2C3));
        assert_eq!(cde.ok, hex_rgb(0x2F7A3A));
        assert_eq!(cde.text_chrome, hex_rgb(0xffffff));
    }
}

/// The language pack's own font (its `### font:` line), tried after Crow's
/// for the characters JetBrains Mono doesn't have: CJK, Arabic, Thai…
/// Without one the system's fallback fonts are used.
pub fn language_font<E: Styled>(mut element: E) -> E {
    if let Some(font) = crate::i18n::current().info.font.clone() {
        element.text_style().font_fallbacks = Some(FontFallbacks::from_fonts(vec![font]));
    }
    element
}
