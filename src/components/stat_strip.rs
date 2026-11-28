use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::scroll::ScrollableElement;
use crate::theme::*;
use crate::components::sparkline::dynamic_sparkline;
use crate::metrics::{ServerMetrics, SurgeAlert};

/// Height of the status strips under the identity bar (buffer / alerts).
pub const STATUS_STRIP_HEIGHT: f32 = 28.0;

pub fn stat_strip(
    metrics: Option<&ServerMetrics>,
    lag_secs: u64,
    surge_alert: Option<&SurgeAlert>,
    disk_drawer_open: bool,
    app_view: Option<Entity<crate::app::CrowApp>>,
) -> impl IntoElement {
    let default_m = ServerMetrics::default();
    let m = metrics.unwrap_or(&default_m);

    // CPU formatting
    let cpu_str = format!("{:.1}", m.cpu_pct);
    let cpu_color = if m.cpu_pct > 80.0 {
        theme().crit
    } else if m.cpu_pct > 60.0 {
        theme().warn
    } else {
        theme().ok
    };
    let cpu_sub = t!("stat-strip-vcpu-peak", vcpu_count = m.vcpu_count, m = format!("{:.1}", m.cpu_peak.max(m.cpu_pct)));

    // Memory formatting
    let (mem_used_str, mem_total_str) = m.mem_formatted();
    let mem_color = if m.mem_pct > 85.0 {
        theme().crit
    } else if m.mem_pct > 65.0 {
        theme().warn
    } else {
        theme().ok
    };
    let mem_ratio = (m.mem_pct / 100.0).clamp(0.01, 1.0);

    // Disk formatting
    let (disk_used_str, disk_total_str) = m.disk_formatted();
    let pct_or_dash = |v: Option<f32>| v.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into());
    let disk_sub = t!("stat-strip-inodes-iowait", pct_or_dash = pct_or_dash(m.inodes_pct), pct_or_dash2 = pct_or_dash(m.iowait_pct));

    // Load formatting
    let load_str = format!("{:.2}", m.load_1m);
    let load_sub = format!("1m {:.2} · 5m {:.2} · 15m {:.2}", m.load_1m, m.load_5m, m.load_15m);

    // Uptime formatting
    let (days, hours, mins) = m.uptime_parts();
    let (uptime_top_val, uptime_top_unit, uptime_sub_val, uptime_sub_unit) = if days > 0 {
        (days.to_string(), "d", format!("{:02}", hours), "h")
    } else {
        (hours.to_string(), "h", format!("{:02}", mins), "m")
    };

    // Network formatting
    let rx_mbps = (m.net_rx_bps as f64 * 8.0) / 1_000_000.0;
    let tx_mbps = (m.net_tx_bps as f64 * 8.0) / 1_000_000.0;
    let net_total_mbps = rx_mbps + tx_mbps;
    let net_str = if net_total_mbps >= 1.0 {
        format!("{:.0}", net_total_mbps)
    } else {
        format!("{:.1}", net_total_mbps)
    };
    let net_sub = t!("stat-strip-mb-s", rx_mbps = format!("{:.1}", rx_mbps), tx_mbps = format!("{:.1}", tx_mbps));

    div()
        .flex_none()
        .w_full()
        .flex()
        .flex_col()
        // 1. Foreknowledge and Turbo Buffer Ribbon
        .child(
            div()
                .h(px(STATUS_STRIP_HEIGHT))
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .px(px(14.0))
                .bg(if surge_alert.is_some() { theme().crit_strip_bg } else { bg_subhead() })
                // A warning, not an alarm: the Danger Zone bar's palette (its
                // background and hairline border) with its buttons' muted red text.
                .when(surge_alert.is_some(), |d| d.border_t_1())
                .border_b_1()
                .border_color(if surge_alert.is_some() { border_danger() } else { border_panel() })
                // Left: Buffer info
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(if surge_alert.is_some() { theme().crit_ink_dim } else { theme().ok }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if surge_alert.is_some() { theme().crit_ink_dim } else { theme().ok })
                                .child(t!("stat-strip-turbo-buffer")),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .text_color(text_muted())
                                .child(t!("stat-strip-s-lag-local-memory-cache", lag_secs = lag_secs)),
                        ),
                )
                // Right: Surge Foreknowledge / Preview
                .child(if let Some(surge) = surge_alert {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme().crit_ink_dim)
                                .child(t!("stat-strip-ahead-in-s", description = surge.description, lead_seconds = surge.lead_seconds)),
                        )
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .text_color(text_faint())
                        .child(div().text_color(theme().ok).child("✓"))
                        .child(t!("stat-strip-next-s-nothing-unusual", lag_secs = lag_secs))
                        .into_any_element()
                }),
        )
        // 2. Main 5-Card Stats Strip
        .child(
            div()
                .flex_none()
                .flex()
                .items_stretch()
                .w_full()
                .bg(bg_panel())
                .border_b_1()
                .border_color(border_panel())
        // 1. CPU LOAD
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_dimmer())
                        .child(t!("stat-strip-cpu-load")),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .gap(px(10.0))
                        .mt(px(5.0))
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary())
                                        .child(cpu_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(13.0))
                                        .text_color(text_dim())
                                        .child("%"),
                                ),
                        )
                        .child(dynamic_sparkline(&m.cpu_history, Some(0.0), Some(100.0), cpu_color)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(text_faint())
                        .mt(px(6.0))
                        .child(cpu_sub),
                ),
        )
        // 2. MEMORY
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_dimmer())
                        .child(t!("stat-strip-memory")),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .gap(px(10.0))
                        .mt(px(5.0))
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary())
                                        .child(mem_used_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(13.0))
                                        .text_color(text_dim())
                                        .child(mem_total_str),
                                ),
                        )
                        .child(dynamic_sparkline(&m.mem_history, Some(0.0), Some(100.0), mem_color)),
                )
                .child(
                    div()
                        .h(px(14.0))
                        .flex()
                        .items_center()
                        .w_full()
                        .mt(px(6.0))
                        .child(
                            div()
                                .w_full()
                                .h(px(3.0))
                                .bg(rgb(0x1a1b21))
                                .child(
                                    div()
                                        .w(relative(mem_ratio))
                                        .h_full()
                                        .bg(mem_color),
                                ),
                        ),
                ),
        )
        // 3. DISK
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_dimmer())
                                .child(t!("stat-strip-disk-nvme", disk_mount = m.disk_mount)),
                        )
                        .children(app_view.clone().map(|app| {
                            div()
                                .id("disk-partition-toggle")
                                .cursor_pointer()
                                .px(px(4.0))
                                .py(px(1.0))
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .text_color(if disk_drawer_open { text_primary() } else { text_dim() })
                                .hover(|s| s.text_color(text_max()).bg(bg_row_hover()))
                                .on_click(move |_ev, _window, cx| {
                                    app.update(cx, |this, cx| {
                                        this.toggle_disk_drawer(cx);
                                    });
                                })
                                .child(if disk_drawer_open { "▴" } else { "▾" })
                        })),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .gap(px(10.0))
                        .mt(px(5.0))
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary())
                                        .child(disk_used_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(13.0))
                                        .text_color(text_dim())
                                        .child(disk_total_str),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_end()
                                .gap(px(2.0))
                                .h(px(26.0))
                                .child(div().w(px(5.0)).h(px(10.0)).bg(rgb(0x2e3038)))
                                .child(div().w(px(5.0)).h(px(12.0)).bg(rgb(0x2e3038)))
                                .child(div().w(px(5.0)).h(px(11.0)).bg(rgb(0x2e3038)))
                                .child(div().w(px(5.0)).h(px(14.0)).bg(rgb(0x2e3038)))
                                .child(div().w(px(5.0)).h(px(13.0)).bg(rgb(0x2e3038)))
                                .child(div().w(px(5.0)).h(px(16.0)).bg(rgb(0x2e3038)))
                                .child(div().w(px(5.0)).h(px(15.0)).bg(text_secondary())),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(text_faint())
                        .mt(px(6.0))
                        .child(disk_sub),
                ),
        )
        // 4. LOAD AVG
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_dimmer())
                        .child(t!("stat-strip-load-avg")),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .gap(px(10.0))
                        .mt(px(5.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary())
                                .child(load_str),
                        )
                        .child(dynamic_sparkline(&m.load_history, Some(0.0), None, text_dimmer())),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(text_faint())
                        .mt(px(6.0))
                        .child(load_sub),
                ),
        )
        // 5. UPTIME
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(border_panel())
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_dimmer())
                        .child(t!("stat-strip-uptime")),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .mt(px(5.0))
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary())
                                        .child(uptime_top_val),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(13.0))
                                        .text_color(text_dim())
                                        .child(uptime_top_unit),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary())
                                        .ml(px(6.0))
                                        .child(uptime_sub_val),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(13.0))
                                        .text_color(text_dim())
                                        .child(uptime_sub_unit),
                                ),
                        )
                        .child(div().w(px(86.0)).h(px(26.0))),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(text_faint())
                        .mt(px(6.0))
                        .child(t!("stat-strip-sampled", last_sample_ts = m.last_sample_ts)),
                ),
        )
        // 6. NETWORK eth0
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_dimmer())
                        .child(t!("stat-strip-network")),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .gap(px(10.0))
                        .mt(px(5.0))
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary())
                                        .child(net_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(13.0))
                                        .text_color(text_dim())
                                        .child(t!("stat-strip-mb-s-2")),
                                ),
                        )
                        .child(dynamic_sparkline(&m.net_history, Some(0.0), None, text_secondary())),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(10.0))
                        .text_color(text_faint())
                        .mt(px(6.0))
                        .child(net_sub),
                ),
        )
    )
}

/// Detailed disk partitions quick view drawer rendered beneath the stat strip.
pub fn disk_partitions_drawer(
    partitions: &[crate::metrics::DiskPartition],
    is_loading: bool,
    app_view: Entity<crate::app::CrowApp>,
) -> impl IntoElement {
    let app_refresh = app_view.clone();
    div()
        .w_full()
        .flex()
        .flex_col()
        .bg(bg_panel())
        .border_b_1()
        .border_color(border_panel())
        // Drawer Header Strip
        .child(
            div()
                .h(px(STATUS_STRIP_HEIGHT))
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .px(px(14.0))
                .bg(bg_subhead())
                .border_b_1()
                .border_color(border_panel())
                // Left: Title + command hint
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .size(px(6.0))
                                .bg(theme().ok),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_primary())
                                .child(t!("stat-strip-disk-partitions")),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .text_color(text_faint())
                                .child("df -h -P | grep -v tmpfs"),
                        ),
                )
                // Right: Status / Refresh button
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .children(if is_loading {
                            Some(
                                div()
                                    .font_family(FONT_MONO)
                                    .text_size(font_px(9.5))
                                    .text_color(theme().warn)
                                    .child(t!("stat-strip-disk-loading")),
                            )
                        } else {
                            None
                        })
                        .child(
                            div()
                                .id("disk-partitions-refresh-btn")
                                .cursor_pointer()
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(bg_control())
                                .border_1()
                                .border_color(border_default())
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .text_color(text_primary())
                                .hover(|s| s.bg(bg_row_hover()).text_color(text_max()))
                                .on_click(move |_ev, _window, cx| {
                                    app_refresh.update(cx, |this, cx| {
                                        this.refresh_disk_partitions(cx);
                                    });
                                })
                                .child(t!("stat-strip-disk-refresh")),
                        ),
                ),
        )
        // Table Header
        .child(
            div()
                .flex()
                .items_center()
                .w_full()
                .px(px(14.0))
                .py(px(6.0))
                .bg(bg_row_alt())
                .border_b_1()
                .border_color(border_panel())
                .font_family(FONT_MONO)
                .text_size(font_px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(text_dimmer())
                .child(div().w(px(160.0)).child(t!("stat-strip-disk-col-mount")))
                .child(div().flex_1().min_w(px(120.0)).child(t!("stat-strip-disk-col-filesystem")))
                .child(div().w(px(70.0)).text_align(TextAlign::Right).child(t!("stat-strip-disk-col-size")))
                .child(div().w(px(70.0)).text_align(TextAlign::Right).child(t!("stat-strip-disk-col-used")))
                .child(div().w(px(80.0)).text_align(TextAlign::Right).child(t!("stat-strip-disk-col-avail")))
                .child(div().w(px(140.0)).pl(px(14.0)).child(t!("stat-strip-disk-col-use"))),
        )
        // Table Rows / Empty state
        .child(
            div()
                .w_full()
                .max_h(px(220.0))
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .children(if partitions.is_empty() {
                    let msg = if is_loading {
                        t!("stat-strip-disk-querying")
                    } else {
                        t!("stat-strip-disk-empty")
                    };
                    vec![
                        div()
                            .w_full()
                            .py(px(16.0))
                            .px(px(14.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(FONT_MONO)
                            .text_size(font_px(11.0))
                            .text_color(text_muted())
                            .child(msg)
                            .into_any_element()
                    ]
                } else {
                    partitions.iter().enumerate().map(|(i, p)| {
                        let bar_color = if p.use_pct >= 85 {
                            theme().crit
                        } else if p.use_pct >= 70 {
                            theme().warn
                        } else {
                            theme().ok
                        };
                        let ratio = (p.use_pct as f32 / 100.0).clamp(0.01, 1.0);
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .px(px(14.0))
                            .py(px(5.0))
                            .bg(if i % 2 == 1 { bg_row_alt() } else { bg_panel() })
                            .hover(|s| s.bg(bg_row_hover()))
                            .border_b_1()
                            .border_color(border_row())
                            .font_family(FONT_MONO)
                            .text_size(font_px(11.0))
                            .child(
                                div()
                                    .w(px(160.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(text_primary())
                                    .child(p.mount.clone()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(120.0))
                                    .text_color(text_dim())
                                    .child(p.filesystem.clone()),
                            )
                            .child(
                                div()
                                    .w(px(70.0))
                                    .text_align(TextAlign::Right)
                                    .text_color(text_secondary())
                                    .child(p.size.clone()),
                            )
                            .child(
                                div()
                                    .w(px(70.0))
                                    .text_align(TextAlign::Right)
                                    .text_color(text_secondary())
                                    .child(p.used.clone()),
                            )
                            .child(
                                div()
                                    .w(px(80.0))
                                    .text_align(TextAlign::Right)
                                    .text_color(text_dim())
                                    .child(p.avail.clone()),
                            )
                            .child(
                                div()
                                    .w(px(140.0))
                                    .pl(px(14.0))
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .h(px(4.0))
                                            .bg(bg_control_alt())
                                            .child(
                                                div()
                                                    .w(relative(ratio))
                                                    .h_full()
                                                    .bg(bar_color),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .w(px(34.0))
                                            .text_align(TextAlign::Right)
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(bar_color)
                                            .child(format!("{}%", p.use_pct)),
                                    ),
                            )
                            .into_any_element()
                    }).collect()
                }),
        )
}
