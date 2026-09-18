use gpui_kit::*;
use crate::app::CrowApp;
use crate::journal::retention::{JournalRetentionConfig, JournalStorageMode, JournalTelemetry};
use crate::theme::*;

pub fn journald_editor(
    config: &JournalRetentionConfig,
    telemetry: &JournalTelemetry,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_quota = app.clone();
    let app_time = app.clone();
    let app_keep = app.clone();
    let app_persist = app.clone();
    let app_volatile = app.clone();
    let app_auto = app.clone();
    let app_apply = app.clone();

    let is_persistent = config.storage == JournalStorageMode::Persistent;
    let is_volatile = config.storage == JournalStorageMode::Volatile;
    let is_auto = config.storage == JournalStorageMode::Auto;

    div()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // Header Bar
        .child(
            div()
                .h(px(48.0))
                .flex_none()
                .px(px(20.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(13.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("journald.conf"),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(if is_persistent { OK_BG } else { CRIT_BG })
                                .border_1()
                                .border_color(if is_persistent { OK } else { CRIT })
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_persistent { OK } else { CRIT })
                                .child(if is_persistent { "CRASH-SAFE" } else { "VOLATILE RISK" }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_TERTIARY)
                                .child("/etc/systemd/journald.conf"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_MUTED)
                                .child(format!("Usage: {} / {}", telemetry.disk_usage_display(), config.max_use_display())),
                        )
                        .child(
                            div()
                                .id("btn-apply-journald-conf")
                                .px(px(12.0))
                                .py(px(5.0))
                                .bg(TEXT_PRIMARY)
                                .text_color(rgb(0x0a0a0c))
                                .font_weight(FontWeight::BOLD)
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgb(0xffffff)))
                                .on_click(move |_ev, _window, cx| {
                                    app_apply.update(cx, |this, cx| {
                                        this.apply_journal_boundaries(cx);
                                    });
                                })
                                .child("STAGE & APPLY CONFIG"),
                        ),
                ),
        )
        // Main Editor Content
        .child(
            div()
                .id("journald-editor-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .p(px(24.0))
                .flex()
                .flex_col()
                .gap(px(20.0))
                // Context Explanation Banner
                .child(
                    div()
                        .p(px(14.0))
                        .bg(hex_rgb(0x101218))
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_PRIMARY)
                                .child("WHY DEFAULT LINUX JOURNAL CONFIGS FAIL IN PRODUCTION"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .text_color(TEXT_SECONDARY)
                                .child("By default, systemd journal uses silent compile-time limits and Storage=auto. On busy servers or post-incident reboots, logs rotate out in 30 minutes or vanish completely. Crow-Config enforces explicit boundaries so you never lose evidence of a crash."),
                        ),
                )
                // 1. Storage Mode Row
                .child(
                    render_setting_row(
                        "Storage",
                        "Controls where journal files are stored. Volatile keeps logs in RAM (/run), which are permanently wiped on reboot. Persistent guarantees writes to /var/log/journal/.",
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(render_choice_pill(
                                "persistent",
                                "PERSISTENT (Rec)",
                                "Survives reboots",
                                is_persistent,
                                OK,
                                "storage-persistent",
                                {
                                    let app = app_persist.clone();
                                    move |cx| {
                                        app.update(cx, |this, cx| {
                                            this.set_journal_storage_mode(JournalStorageMode::Persistent, cx);
                                        });
                                    }
                                },
                            ))
                            .child(render_choice_pill(
                                "volatile",
                                "VOLATILE",
                                "RAM only · Wiped on reboot",
                                is_volatile,
                                CRIT,
                                "storage-volatile",
                                {
                                    let app = app_volatile.clone();
                                    move |cx| {
                                        app.update(cx, |this, cx| {
                                            this.set_journal_storage_mode(JournalStorageMode::Volatile, cx);
                                        });
                                    }
                                },
                            ))
                            .child(render_choice_pill(
                                "auto",
                                "AUTO",
                                "Default fallback",
                                is_auto,
                                WARN,
                                "storage-auto",
                                {
                                    let app = app_auto.clone();
                                    move |cx| {
                                        app.update(cx, |this, cx| {
                                            this.set_journal_storage_mode(JournalStorageMode::Auto, cx);
                                        });
                                    }
                                },
                            )),
                    )
                )
                // 2. SystemMaxUse (Hard Quota)
                .child(
                    render_setting_row(
                        "SystemMaxUse",
                        "The maximum disk space journals may occupy on persistent storage. When exceeded, systemd removes the oldest archive files.",
                        div()
                            .flex()
                            .gap(px(8.0))
                            .children([512, 2048, 4096, 8192, 16384].into_iter().enumerate().map(|(idx, mb)| {
                                let is_sel = config.system_max_use_mb == mb;
                                let label = match mb {
                                    512 => "500M",
                                    2048 => "2G",
                                    4096 => "4G (Rec)",
                                    8192 => "8G",
                                    16384 => "16G",
                                    _ => "Custom",
                                };
                                let app = app_quota.clone();
                                div()
                                    .id(ElementId::NamedInteger("cfg-quota".into(), idx as u64))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                    .border_1()
                                    .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                    .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.set_journal_quota(mb, cx);
                                        });
                                    })
                                    .child(label)
                            }))
                    )
                )
                // 3. MaxRetentionSec (Time Cutoff)
                .child(
                    render_setting_row(
                        "MaxRetentionSec",
                        "The maximum time to store journal entries. Prevents older log archives from lingering indefinitely once log volume is low.",
                        div()
                            .flex()
                            .gap(px(8.0))
                            .children([7, 14, 30, 90, 180, 0].into_iter().enumerate().map(|(idx, days)| {
                                let is_sel = config.max_retention_days == days;
                                let label = match days {
                                    7 => "7day",
                                    14 => "14day",
                                    30 => "30day (Rec)",
                                    90 => "90day",
                                    180 => "180day",
                                    0 => "0 (Unlimited)",
                                    _ => "Custom",
                                };
                                let app = app_time.clone();
                                div()
                                    .id(ElementId::NamedInteger("cfg-retention".into(), idx as u64))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                    .border_1()
                                    .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                    .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.set_journal_retention_days(days, cx);
                                        });
                                    })
                                    .child(label)
                            }))
                    )
                )
                // 4. SystemKeepFree (Filesystem Protection)
                .child(
                    render_setting_row(
                        "SystemKeepFree",
                        "Guarantees that at least this much disk space is left free on the root filesystem, preventing log-flood out-of-disk crashes.",
                        div()
                            .flex()
                            .gap(px(8.0))
                            .children([1024, 2048, 5120].into_iter().enumerate().map(|(idx, mb)| {
                                let is_sel = config.system_keep_free_mb == mb;
                                let label = match mb {
                                    1024 => "1.0 GB",
                                    2048 => "2.0 GB (Rec)",
                                    5120 => "5.0 GB",
                                    _ => "Custom",
                                };
                                let app = app_keep.clone();
                                div()
                                    .id(ElementId::NamedInteger("cfg-keep".into(), idx as u64))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(if is_sel { BG_NAV_ACTIVE } else { BG_CONTROL })
                                    .border_1()
                                    .border_color(if is_sel { TEXT_PRIMARY } else { BORDER_DEFAULT })
                                    .text_color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                    .cursor_pointer()
                                    .hover(|s| s.bg(BG_ROW_HOVER))
                                    .on_click(move |_ev, _window, cx| {
                                        app.update(cx, |this, cx| {
                                            this.set_journal_keep_free(mb, cx);
                                        });
                                    })
                                    .child(label)
                            }))
                    )
                )
                // 5. Rate Limiting Protection
                .child(
                    render_setting_row(
                        "RateLimitBurst / RateLimitIntervalSec",
                        "Suppresses runaway spam loops from flooding journals and choking the system (10,000 entries per 30 seconds default).",
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .px(px(10.0))
                                    .py(px(4.0))
                                    .bg(BG_CONTROL)
                                    .border_1()
                                    .border_color(BORDER_DEFAULT)
                                    .font_family(FONT_MONO)
                                    .text_size(px(10.0))
                                    .text_color(TEXT_PRIMARY)
                                    .child(format!("{} entries / {}s", config.rate_limit_burst, config.rate_limit_interval_sec)),
                            )
                            .child(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(px(9.5))
                                    .text_color(OK)
                                    .child("✓ Anti-flood enabled"),
                            ),
                    )
                ),
        )
}

fn render_setting_row(
    name: &str,
    description: &str,
    control: impl IntoElement,
) -> impl IntoElement {
    div()
        .p(px(14.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .gap(px(10.0))
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
                        .text_color(TEXT_PRIMARY)
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_MUTED)
                        .child(description.to_string()),
                ),
        )
        .child(control)
}

fn render_choice_pill<F>(
    _value: &str,
    label: &str,
    hint: &str,
    is_selected: bool,
    accent: Rgba,
    id_name: &'static str,
    on_select: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    div()
        .id(id_name)
        .px(px(12.0))
        .py(px(6.0))
        .bg(if is_selected { BG_NAV_ACTIVE } else { BG_CONTROL })
        .border_1()
        .border_color(if is_selected { accent } else { BORDER_DEFAULT })
        .flex()
        .flex_col()
        .gap(px(2.0))
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _window, cx| {
            on_select(cx);
        })
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                .text_color(if is_selected { accent } else { TEXT_PRIMARY })
                .child(label.to_string()),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(8.5))
                .text_color(TEXT_MUTED)
                .child(hint.to_string()),
        )
}
