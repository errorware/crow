use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::components::icon_button::icon_button;
use crate::components::icons::TablerIcon;
use crate::app::CrowApp;
use crate::journal::retention::{JournalRetentionConfig, JournalStorageMode, JournalTelemetry};
use crate::theme::*;

pub fn retention_boundaries_modal(
    config: &JournalRetentionConfig,
    telemetry: &JournalTelemetry,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_close = app.clone();
    let app_close_scrim = app.clone();
    let app_apply = app.clone();
    let app_crow_config = app.clone();
    let app_mode = app.clone();
    let app_quota = app.clone();
    let app_time = app.clone();
    let app_keep = app.clone();
    let app_fix_persist = app.clone();

    let is_persistent = config.storage == JournalStorageMode::Persistent;
    let is_warning = telemetry.is_volatile_warning || !is_persistent;

    let quota_mb = config.system_max_use_mb;
    let usage_pct = telemetry.usage_percentage(quota_mb).min(100.0);

    div()
        .id("journal-retention-modal-scrim")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x060709, 0.85))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| {
            app_close_scrim.update(cx, |this, cx| {
                this.toggle_journal_retention_modal(cx);
            });
        })
        .child(
            // Modal Card (stop propagation by consuming clicks)
            div()
                .id("journal-retention-modal-card")
                .w(px(640.0))
                .max_h(px(720.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .on_click(|_ev, _window, _cx| {
                    // Prevent modal click from bubbling to scrim
                })
                // Header
                .child(
                    div()
                        .h(px(46.0))
                        .flex_none()
                        .px(px(20.0))
                        .bg(BG_SUBHEAD)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child(t!("logs-retention-modal-systemd-journal-retention-boundaries")),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(9.5))
                                        .text_color(TEXT_TERTIARY)
                                        .child(t!("logs-retention-modal-eliminate-30-minute-silent-log")),
                                ),
                        )
                        .child(
                            icon_button("btn-close-retention-modal", TablerIcon::X, false)
                                .on_click(move |_ev, _window, cx| {
                                    app_close.update(cx, |this, cx| {
                                        this.toggle_journal_retention_modal(cx);
                                    });
                                }),
                        ),
                )
                // Content Body
                .child(
                    div()
                        .id("retention-modal-scroll")
                        .flex_1()
                        .min_h(px(0.0))
                        .overflow_y_scrollbar()
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(18.0))
                        // 1. Storage Health & Crash Protection Banner
                        .child(
                            if is_warning {
                                div()
                                    .p(px(12.0))
                                    .bg(CRIT_BG)
                                    .border_1()
                                    .border_color(CRIT)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(11.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(CRIT)
                                                    .child(t!("logs-retention-modal-critical-storage-is-volatile-auto")),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.0))
                                                    .text_color(TEXT_PRIMARY)
                                                    .child(t!("logs-retention-modal-if-this-host-crashes-or")),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id("btn-fix-persist-journal")
                                            .px(px(10.0))
                                            .py(px(5.0))
                                            .bg(CRIT)
                                            .text_color(rgb(0x0a0a0c))
                                            .font_weight(FontWeight::BOLD)
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(hex_rgb(0xff5555)))
                                            .on_click(move |_ev, _window, cx| {
                                                app_fix_persist.update(cx, |this, cx| {
                                                    this.set_journal_storage_mode(JournalStorageMode::Persistent, cx);
                                                });
                                            })
                                            .child(t!("logs-retention-modal-lock-to-disk")),
                                    )
                            } else {
                                div()
                                    .p(px(12.0))
                                    .bg(OK_BG)
                                    .border_1()
                                    .border_color(OK)
                                    .flex()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(14.0))
                                            .text_color(OK)
                                            .child("✓"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(10.5))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(OK)
                                                    .child(t!("logs-retention-modal-crash-safe-persistent-storage-active")),
                                            )
                                            .child(
                                                div()
                                                    .font_family(FONT_MONO)
                                                    .text_size(px(9.5))
                                                    .text_color(TEXT_SECONDARY)
                                                    .child(t!("logs-retention-modal-active-journals-persist-in-var")),
                                            ),
                                    )
                            }
                        )
                        // 2. Metrics Strip (Usage, Horizon, Daily Burn)
                        .child(
                            div()
                                .grid()
                                .grid_cols(3)
                                .gap(px(10.0))
                                .child(render_stat_card(t!("logs-retention-modal-current-disk-usage"), &telemetry.disk_usage_display(), &t!("logs-retention-modal-active-files", active_files_count = telemetry.active_files_count)))
                                .child(render_stat_card(t!("logs-retention-modal-retention-horizon"), &t!("logs-retention-modal-hours", retention_horizon_hours = format!("{:.0}", telemetry.retention_horizon_hours)), &t!("logs-retention-modal-oldest", oldest_timestamp = telemetry.oldest_timestamp)))
                                .child(render_stat_card(t!("logs-retention-modal-daily-write-rate"), &t!("logs-retention-modal-mb-day", daily_burn_rate_mb = format!("{:.0}", telemetry.daily_burn_rate_mb)), &t!("logs-retention-modal-days-at-current-rate", estimated_retained_days = format!("{:.0}", telemetry.estimated_retained_days))))
                        )
                        // 3. Visual Capacity Gauge Bar
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_SECONDARY)
                                                .child(t!("logs-retention-modal-allocation", disk_usage_display = telemetry.disk_usage_display(), max_use_display = config.max_use_display(), usage_pct = format!("{:.1}", usage_pct))),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_TERTIARY)
                                                .child(t!("logs-retention-modal-systemmaxuse-boundary")),
                                        ),
                                )
                                .child(
                                    div()
                                        .h(px(10.0))
                                        .w_full()
                                        .bg(hex_rgb(0x13141a))
                                        .border_1()
                                        .border_color(BORDER_DEFAULT)
                                        .child(
                                            div()
                                                .h_full()
                                                .w(px((6.0 * usage_pct).max(2.0)))
                                                .bg(if usage_pct > 85.0 { CRIT } else if usage_pct > 60.0 { WARN } else { OK })
                                        ),
                                ),
                        )
                        // 3b. Storage Mode Selector
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_PRIMARY)
                                                .child(t!("logs-retention-modal-storage-persistence-storage")),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_MUTED)
                                                .child(t!("logs-retention-modal-controls-whether-logs-survive-host")),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .children([
                                            (JournalStorageMode::Persistent, t!("logs-retention-modal-persistent-rec"), OK),
                                            (JournalStorageMode::Volatile, t!("logs-retention-modal-volatile-ram-only"), CRIT),
                                            (JournalStorageMode::Auto, t!("logs-retention-modal-auto-fragile"), WARN),
                                        ].into_iter().enumerate().map(|(idx, (mode, label, accent))| {
                                            let is_sel = config.storage == mode;
                                            let app_m = app_mode.clone();
                                            div()
                                                .id(ElementId::NamedInteger("modal-storage-opt".into(), idx as u64))
                                                .flex_1()
                                                .py(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { accent } else { BORDER_DEFAULT })
                                                .text_color(if is_sel { accent } else { TEXT_SECONDARY })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_m.update(cx, |this, cx| {
                                                        this.set_journal_storage_mode(mode, cx);
                                                    });
                                                })
                                                .child(label)
                                        }))
                                )
                        )
                        // 4. Boundary Setting: Max Quota (SystemMaxUse)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_PRIMARY)
                                                .child(t!("logs-retention-modal-max-disk-quota-systemmaxuse")),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_MUTED)
                                                .child(t!("logs-retention-modal-hard-ceiling-before-older-journals")),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .children([512, 2048, 4096, 8192, 16384].into_iter().enumerate().map(|(idx, mb)| {
                                            let is_sel = config.system_max_use_mb == mb;
                                            let label = match mb {
                                                512 => t!("logs-retention-modal-500-mb"),
                                                2048 => t!("logs-retention-modal-2-0-gb"),
                                                4096 => t!("logs-retention-modal-4-0-gb-rec"),
                                                8192 => t!("logs-retention-modal-8-0-gb"),
                                                16384 => t!("logs-retention-modal-16-gb"),
                                                _ => t!("logs-retention-modal-custom"),
                                            };
                                            let app_q = app_quota.clone();
                                            div()
                                                .id(ElementId::NamedInteger("quota-opt".into(), idx as u64))
                                                .flex_1()
                                                .py(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                                .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_q.update(cx, |this, cx| {
                                                        this.set_journal_quota(mb, cx);
                                                    });
                                                })
                                                .child(label)
                                        }))
                                )
                        )
                        // 5. Boundary Setting: Max Retention Horizon (MaxRetentionSec)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_PRIMARY)
                                                .child(t!("logs-retention-modal-time-horizon-maxretentionsec")),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_MUTED)
                                                .child(t!("logs-retention-modal-maximum-age-before-logs-are")),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .children([7, 14, 30, 90, 180, 0].into_iter().enumerate().map(|(idx, days)| {
                                            let is_sel = config.max_retention_days == days;
                                            let label = match days {
                                                7 => t!("logs-retention-modal-7-days"),
                                                14 => t!("logs-retention-modal-14-days"),
                                                30 => t!("logs-retention-modal-30-days-rec"),
                                                90 => t!("logs-retention-modal-90-days"),
                                                180 => t!("logs-retention-modal-180-days"),
                                                0 => t!("logs-retention-modal-unlimited"),
                                                _ => t!("logs-retention-modal-custom"),
                                            };
                                            let app_t = app_time.clone();
                                            div()
                                                .id(ElementId::NamedInteger("time-opt".into(), idx as u64))
                                                .flex_1()
                                                .py(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                                .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_t.update(cx, |this, cx| {
                                                        this.set_journal_retention_days(days, cx);
                                                    });
                                                })
                                                .child(label)
                                        }))
                                )
                        )
                        // 6. Boundary Setting: Root Filesystem Safety Buffer (SystemKeepFree)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.5))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(TEXT_PRIMARY)
                                                .child(t!("logs-retention-modal-safety-disk-reserve-systemkeepfree")),
                                        )
                                        .child(
                                            div()
                                                .font_family(FONT_MONO)
                                                .text_size(px(9.5))
                                                .text_color(TEXT_MUTED)
                                                .child(t!("logs-retention-modal-guarantees-disk-space-stays-free")),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.0))
                                        .children([1024, 2048, 5120].into_iter().enumerate().map(|(idx, mb)| {
                                            let is_sel = config.system_keep_free_mb == mb;
                                            let label = match mb {
                                                1024 => t!("logs-retention-modal-1-0-gb"),
                                                2048 => t!("logs-retention-modal-2-0-gb-rec"),
                                                5120 => t!("logs-retention-modal-5-0-gb"),
                                                _ => t!("logs-retention-modal-custom"),
                                            };
                                            let app_k = app_keep.clone();
                                            div()
                                                .id(ElementId::NamedInteger("keep-opt".into(), idx as u64))
                                                .flex_1()
                                                .py(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .font_family(FONT_MONO)
                                                .text_size(px(10.0))
                                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                                .border_1()
                                                .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                                .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                                .cursor_pointer()
                                                .hover(|s| s.bg(BG_ROW_HOVER))
                                                .on_click(move |_ev, _window, cx| {
                                                    app_k.update(cx, |this, cx| {
                                                        this.set_journal_keep_free(mb, cx);
                                                    });
                                                })
                                                .child(label)
                                        }))
                                )
                        )
                )
                // Footer Actions
                .child(
                    div()
                        .h(px(52.0))
                        .flex_none()
                        .px(px(20.0))
                        .bg(BG_SUBHEAD)
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .justify_between()
                        // Left: Edit in Crow-Config
                        .child(
                            div()
                                .id("btn-jump-to-crow-config")
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_SECONDARY)
                                .hover(|s| s.text_color(TEXT_PRIMARY).bg(BG_ROW_HOVER))
                                .cursor_pointer()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .on_click(move |_ev, _window, cx| {
                                    app_crow_config.update(cx, |this, cx| {
                                        this.toggle_journal_retention_modal(cx);
                                        this.set_active_view("config", cx);
                                        this.select_managed_file("journald.conf", cx);
                                    });
                                })
                                .child(t!("logs-retention-modal-edit-in-crow-config")),
                        )
                        // Right: Save / Close
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .id("btn-apply-journal-boundaries")
                                        .px(px(14.0))
                                        .py(px(6.0))
                                        .bg(TEXT_PRIMARY)
                                        .text_color(rgb(0x0a0a0c))
                                        .font_weight(FontWeight::BOLD)
                                        .font_family(FONT_MONO)
                                        .text_size(px(10.5))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(hex_rgb(0xffffff)))
                                        .on_click(move |_ev, _window, cx| {
                                            app_apply.update(cx, |this, cx| {
                                                this.apply_journal_boundaries(cx);
                                            });
                                        })
                                        .child(t!("logs-retention-modal-apply-to-etc-systemd-journald")),
                                ),
                        ),
                ),
        )
}

fn render_stat_card(label: &str, value: &str, sub: &str) -> impl IntoElement {
    div()
        .p(px(10.0))
        .bg(BG_CONTROL)
        .border_1()
        .border_color(BORDER_DEFAULT)
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_TERTIARY)
                .child(label.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .text_color(TEXT_PRIMARY)
                .child(value.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(9.0))
                .text_color(TEXT_MUTED)
                .child(sub.to_string()),
        )
}
