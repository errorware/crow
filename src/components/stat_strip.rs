use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use crate::theme::*;
use crate::components::sparkline::dynamic_sparkline;
use crate::metrics::{ServerMetrics, SurgeAlert};

pub fn stat_strip(
    metrics: Option<&ServerMetrics>,
    lag_secs: u64,
    surge_alert: Option<&SurgeAlert>,
) -> impl IntoElement {
    let default_m = ServerMetrics::default();
    let m = metrics.unwrap_or(&default_m);

    // CPU formatting
    let cpu_str = format!("{:.1}", m.cpu_pct);
    let cpu_color = if m.cpu_pct > 80.0 {
        CRIT
    } else if m.cpu_pct > 60.0 {
        WARN
    } else {
        OK
    };
    let cpu_sub = format!("{} vCPU · peak {:.1}%", m.vcpu_count, m.cpu_peak.max(m.cpu_pct));

    // Memory formatting
    let (mem_used_str, mem_total_str) = m.mem_formatted();
    let mem_color = if m.mem_pct > 85.0 {
        CRIT
    } else if m.mem_pct > 65.0 {
        WARN
    } else {
        OK
    };
    let mem_ratio = (m.mem_pct / 100.0).clamp(0.01, 1.0);

    // Disk formatting
    let (disk_used_str, disk_total_str) = m.disk_formatted();
    let pct_or_dash = |v: Option<f32>| v.map(|v| format!("{v:.1}%")).unwrap_or_else(|| "—".into());
    let disk_sub = format!("inodes {} · iowait {}", pct_or_dash(m.inodes_pct), pct_or_dash(m.iowait_pct));

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
    let net_sub = format!("↓ {:.1} · ↑ {:.1} Mb/s", rx_mbps, tx_mbps);

    div()
        .flex_none()
        .w_full()
        .flex()
        .flex_col()
        // 1. Foreknowledge and Turbo Buffer Ribbon
        .child(
            div()
                .h(px(24.0))
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .px(px(14.0))
                .bg(if surge_alert.is_some() { hex_rgb(0x1e1215) } else { hex_rgb(0x0e1014) })
                // A warning, not an alarm: hairlines above and below in the Danger
                // Zone buttons' border tone, text in their muted red.
                .when(surge_alert.is_some(), |d| d.border_t_1())
                .border_b_1()
                .border_color(if surge_alert.is_some() { BORDER_DANGER_BTN } else { BORDER_PANEL })
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
                                .bg(if surge_alert.is_some() { CRIT_INK_DIM } else { OK }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if surge_alert.is_some() { CRIT_INK_DIM } else { OK })
                                .child("TURBO BUFFER"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(9.5))
                                .text_color(TEXT_MUTED)
                                .child(format!("· -{}s lag · local memory cache", lag_secs)),
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
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(CRIT_INK_DIM)
                                .child(format!("⚡ FOREKNOWLEDGE ALERT: {} in +{}s", surge.description, surge.lead_seconds)),
                        )
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_FAINT)
                        .child(div().text_color(OK).child("✓"))
                        .child(format!("next +{}s nominal · no upcoming anomalies detected", lag_secs))
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
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
        // 1. CPU LOAD
        .child(
            div()
                .flex_1()
                .pt(px(10.0))
                .pb(px(11.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("CPU LOAD"),
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
                                        .text_size(px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(TEXT_PRIMARY)
                                        .child(cpu_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child("%"),
                                ),
                        )
                        .child(dynamic_sparkline(&m.cpu_history, Some(0.0), Some(100.0), cpu_color)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
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
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("MEMORY"),
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
                                        .text_size(px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(TEXT_PRIMARY)
                                        .child(mem_used_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
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
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child(format!("DISK {} (NVME)", m.disk_mount)),
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
                                        .text_size(px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(TEXT_PRIMARY)
                                        .child(disk_used_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
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
                                .child(div().w(px(5.0)).h(px(15.0)).bg(TEXT_SECONDARY)),
                        ),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
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
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("LOAD AVG"),
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
                                .text_size(px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(TEXT_PRIMARY)
                                .child(load_str),
                        )
                        .child(dynamic_sparkline(&m.load_history, Some(0.0), None, TEXT_DIMMER)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
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
                .border_color(BORDER_PANEL)
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("UPTIME"),
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
                                        .text_size(px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(TEXT_PRIMARY)
                                        .child(uptime_top_val),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child(uptime_top_unit),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(TEXT_PRIMARY)
                                        .ml(px(6.0))
                                        .child(uptime_sub_val),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child(uptime_sub_unit),
                                ),
                        )
                        .child(div().w(px(86.0)).h(px(26.0))),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(6.0))
                        .child(format!("sampled {}", m.last_sample_ts)),
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
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("NETWORK"),
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
                                        .text_size(px(26.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(TEXT_PRIMARY)
                                        .child(net_str),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child("Mb/s"),
                                ),
                        )
                        .child(dynamic_sparkline(&m.net_history, Some(0.0), None, TEXT_SECONDARY)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(6.0))
                        .child(net_sub),
                ),
        )
    )
}
