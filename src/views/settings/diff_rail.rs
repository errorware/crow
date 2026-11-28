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
        t!("settings-diff-rail-in-sync").to_string()
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
                .bg(bg_panel())
                .border_b_1()
                .border_color(border_panel())
                .gap(px(8.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary())
                        .child(t!("settings-diff-rail-pending-diff")),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(if total_changed > 0 { theme().warn } else { text_dimmer() })
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
                .border_color(border_panel())
                .font_family(FONT_MONO)
                .text_size(font_px(10.5))
                .line_height(relative(1.5))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .text_color(theme().ok)
                        .child(inherited_icon(TablerIcon::Check, font_px(12.0)))
                        .child(t!("settings-diff-rail-configuration-in-sync")),
                )
                .child(
                    div()
                        .mt(px(4.0))
                        .text_color(text_fainter())
                        .child(t!("settings-diff-rail-working-copy-matches-config-crow")),
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
                .border_color(border_panel())
                .font_family(FONT_MONO)
                .text_size(font_px(10.5))
                .line_height(relative(1.55))
                .children(diff_lines.into_iter().map(|line| {
                    let (bg_c, text_c) = match line.kind {
                        DiffKind::Hunk => (theme().diff_hunk_bg, text_dimmer()),
                        DiffKind::Addition => (theme().diff_add_bg, theme().ok_ink),
                        DiffKind::Deletion => (theme().diff_del_bg, theme().crit_ink),
                        DiffKind::Context => (hex_rgba(0, 0.0), text_muted()),
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
                    .border_color(border_panel())
                    .bg(bg_subhead())
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(div().font_family(FONT_MONO).text_size(font_px(10.0)).text_color(theme().warn).child("▲"))
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(font_px(10.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme().warn)
                                    .child(t!("settings-diff-rail-affects-live-sessions")),
                            ),
                    )
                    .child(
                        div()
                            .font_family(FONT_MONO)
                            .text_size(font_px(10.5))
                            .line_height(relative(1.5))
                            .text_color(text_tertiary())
                            .child(t!("settings-diff-rail-pending-ssh-connection-changes-take")),
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
                        .text_size(font_px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_dimmer())
                        .child(t!("settings-diff-rail-keychain")),
                )
                .children(keychain.is_empty().then(|| div().font_family(FONT_MONO).text_size(font_px(10.5)).text_color(text_faint()).child(t!("settings-diff-rail-no-keys-enrolled-yet"))))
                .children(keychain.iter().map(|(name, note, dot)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.5))
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
                                .text_color(text_secondary())
                                .child(name.clone()),
                        )
                        .child(
                            div()
                                .text_color(text_dimmer())
                                .child(note.clone()),
                        )
                })),
        )
}
