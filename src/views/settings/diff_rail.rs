//! Settings right rail: pending diff, live sessions warning, and enrolled keychain.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use crate::app::{CrowApp, SettingsSection};
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::config::CrowConfigManager;
use crate::theme::*;
use crate::config::DiffKind;

pub fn render_diff_rail(
    config: &CrowConfigManager,
    _section: SettingsSection,
    keychain: &[(String, String, Rgba)],
    _app: Entity<CrowApp>,
) -> impl IntoElement {
    let total_changed = config.total_changed_count();
    let diff_lines = config.generate_diff();
    let additions = diff_lines.iter().filter(|d| d.kind == DiffKind::Addition).count();
    let deletions = diff_lines.iter().filter(|d| d.kind == DiffKind::Deletion).count();
    let diff_badge = if additions == 0 && deletions == 0 {
        "in sync".to_string()
    } else {
        format!("+{} −{}", additions, deletions)
    };
    let has_conn_changes = config.changed_count_for_section("connection") > 0;

    div()
        .size_full()
        .flex()
        .flex_col()
        // Pending Diff Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .gap(px(8.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("PENDING DIFF"),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(if total_changed > 0 { WARN } else { TEXT_DIMMER })
                        .child(diff_badge),
                ),
        )
        // Diff Snippets
        .child(if diff_lines.is_empty() {
            div()
                .id("pending-diff-empty")
                .flex_none()
                .p(px(14.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .line_height(relative(1.5))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .text_color(OK)
                        .child(inherited_icon(TablerIcon::Check, px(12.0)))
                        .child("Configuration in sync"),
                )
                .child(
                    div()
                        .mt(px(4.0))
                        .text_color(TEXT_FAINTER)
                        .child("Working copy matches ~/.config/crow/config.toml"),
                )
                .into_any_element()
        } else {
            div()
                .id("pending-diff-scroll")
                .flex_none()
                .max_h(px(280.0))
                .overflow_y_scrollbar()
                .py(px(6.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .line_height(relative(1.55))
                .children(diff_lines.into_iter().map(|line| {
                    let (bg_c, text_c) = match line.kind {
                        DiffKind::Hunk => (DIFF_HUNK_BG, TEXT_DIMMER),
                        DiffKind::Addition => (DIFF_ADD_BG, OK_INK),
                        DiffKind::Deletion => (DIFF_DEL_BG, CRIT_INK),
                        DiffKind::Context => (hex_rgba(0, 0.0), TEXT_MUTED),
                    };
                    div()
                        .px(px(12.0))
                        .py(px(1.5))
                        .bg(bg_c)
                        .text_color(text_c)
                        .child(line.text)
                }))
                .into_any_element()
        })
        // Live Sessions Warning (shown if connection changes exist)
        .children(if has_conn_changes {
            Some(
                div()
                    .flex_none()
                    .p(px(12.0))
                    .border_b_1()
                    .border_color(BORDER_PANEL)
                    .bg(hex_rgb(0x0c0a0a))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(WARN).child("▲"))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(WARN)
                                    .child("AFFECTS LIVE SESSIONS"),
                            ),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .line_height(relative(1.5))
                            .text_color(TEXT_TERTIARY)
                            .child("Pending SSH connection changes take effect upon reconnection. Existing active sessions will retain their current parameters."),
                    ),
            )
        } else {
            None
        })
        // Keychain Section
        .child(
            div()
                .flex_1()
                .p(px(12.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_DIMMER)
                        .child("KEYCHAIN"),
                )
                .children(keychain.is_empty().then(|| div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_FAINT).child("No keys enrolled yet.")))
                .children(keychain.iter().map(|(name, note, dot)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .child(
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(*dot)
                                .flex_none(),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(TEXT_SECONDARY)
                                .child(name.clone()),
                        )
                        .child(
                            div()
                                .text_color(TEXT_DIMMER)
                                .child(note.clone()),
                        )
                })),
        )
}
