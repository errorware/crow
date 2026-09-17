use gpui_kit::*;
use crate::theme::*;

pub struct LogLine {
    pub ts: &'static str,
    pub level: &'static str,
    pub msg: &'static str,
}

pub fn sample_logs() -> &'static [LogLine] {
    &[
        LogLine { ts: "03:41:22.481", level: "INFO", msg: "nginx: 159.223.84.17 GET /api/v2/orders 200 14ms" },
        LogLine { ts: "03:41:22.106", level: "INFO", msg: "sidekiq: MailerJob JID-9f21ac done in 212ms" },
        LogLine { ts: "03:41:21.884", level: "WARN", msg: "redis: latency spike 84ms on BLPOP queue:default" },
        LogLine { ts: "03:41:21.552", level: "INFO", msg: "postgres: checkpoint complete, wrote 1428 buffers" },
        LogLine { ts: "03:41:21.310", level: "ERROR", msg: "freshclam: signature mirror timeout after 30s" },
        LogLine { ts: "03:41:20.998", level: "INFO", msg: "docker: container web.2 healthcheck ok" },
        LogLine { ts: "03:41:20.744", level: "INFO", msg: "nginx: 10.0.4.19 POST /webhooks/stripe 204 8ms" },
        LogLine { ts: "03:41:20.401", level: "WARN", msg: "fail2ban: ban 45.134.26.7 (sshd, 6 failures)" },
        LogLine { ts: "03:41:20.118", level: "INFO", msg: "crow-agent: metrics flush 312 series" },
        LogLine { ts: "03:41:19.877", level: "INFO", msg: "prometheus: scrape node_exporter 9100 ok" },
        LogLine { ts: "03:41:19.503", level: "ERROR", msg: "redis: MISCONF unable to persist to disk (errno 28)" },
        LogLine { ts: "03:41:19.244", level: "INFO", msg: "nginx: 159.223.84.17 GET /healthz 200 1ms" },
        LogLine { ts: "03:41:18.912", level: "WARN", msg: "systemd: unattended-upgrades holding 3 packages" },
        LogLine { ts: "03:41:18.660", level: "INFO", msg: "postgres: autovacuum on public.events (21s)" },
        LogLine { ts: "03:41:18.331", level: "INFO", msg: "sidekiq: enqueue ReindexJob queue=low" },
        LogLine { ts: "03:41:18.007", level: "INFO", msg: "docker: pulled ghcr.io/acme/api@sha256:4f1c9b" },
        LogLine { ts: "03:41:17.771", level: "WARN", msg: "kernel: TCP: request_sock_TCP overflow on eth0" },
        LogLine { ts: "03:41:17.442", level: "INFO", msg: "nginx: 10.0.4.22 GET /assets/app.js 304 2ms" },
        LogLine { ts: "03:41:17.119", level: "INFO", msg: "chrony: offset -0.000184s, stratum 2" },
        LogLine { ts: "03:41:16.880", level: "ERROR", msg: "sidekiq: Net::ReadTimeout retry 2/25 JID-c04e18" },
        LogLine { ts: "03:41:16.551", level: "INFO", msg: "crow-agent: ssh keepalive ok (12ms)" },
        LogLine { ts: "03:41:16.203", level: "WARN", msg: "disk: / at 43% growth +2.1GB/24h" },
        LogLine { ts: "03:41:15.964", level: "INFO", msg: "postgres: connection from 10.0.4.19 authenticated" },
        LogLine { ts: "03:41:15.612", level: "INFO", msg: "nginx: 10.0.4.19 GET /api/v2/quotes 200 31ms" },
        LogLine { ts: "03:41:15.288", level: "INFO", msg: "grafana: alert rule 'cpu_high' evaluated ok" },
        LogLine { ts: "03:41:14.955", level: "ERROR", msg: "ufw: BLOCK IN=eth0 SRC=185.220.101.4 DPT=23" },
        LogLine { ts: "03:41:14.702", level: "INFO", msg: "journald: rotated system.journal (128M)" },
        LogLine { ts: "03:41:14.410", level: "INFO", msg: "nginx: 159.223.84.17 GET /api/v2/orders 200 11ms" },
    ]
}

pub fn log_tail() -> impl IntoElement {
    div()
        .w(px(400.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        // Header
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("LOG TAIL"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("journalctl -f -u *"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .size(px(6.0))
                        .rounded_full()
                        .bg(OK)
                        .flex_none(),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child("AUTOSCROLL · pause on hover"),
                ),
        )
        // Filter bar
        .child(
            div()
                .h(px(26.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(12.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family("JetBrains Mono")
                .text_size(px(10.0))
                .child(
                    div()
                        .bg(CRIT)
                        .text_color(rgb(0x0a0a0c))
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child("ERROR 4"),
                )
                .child(
                    div()
                        .bg(WARN)
                        .text_color(rgb(0x0a0a0c))
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child("WARN 17"),
                )
                .child(
                    div()
                        .bg(BG_CHIP)
                        .text_color(TEXT_TERTIARY)
                        .font_weight(FontWeight::BOLD)
                        .px(px(5.0))
                        .py(px(2.0))
                        .child("INFO 1.2k"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_color(TEXT_FAINT)
                        .child("wrap off"),
                ),
        )
        // Log stream
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .overflow_hidden()
                .flex()
                .flex_col()
                .py(px(6.0))
                .children(sample_logs().iter().map(|line| {
                    let is_err = line.level == "ERROR";
                    let is_warn = line.level == "WARN";

                    let (lvl_color, msg_color, row_bg) = if is_err {
                        (CRIT, CRIT_INK, CRIT_LOG_BG)
                    } else if is_warn {
                        (WARN, WARN_INK, rgb(0x00000000))
                    } else {
                        (TEXT_FAINT, TEXT_TERTIARY, rgb(0x00000000))
                    };

                    div()
                        .flex()
                        .gap(px(8.0))
                        .px(px(12.0))
                        .py(px(2.0))
                        .bg(row_bg)
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(
                            div()
                                .text_color(TEXT_FAINTER)
                                .flex_none()
                                .child(line.ts),
                        )
                        .child(
                            div()
                                .w(px(36.0))
                                .flex_none()
                                .font_weight(FontWeight::BOLD)
                                .text_color(lvl_color)
                                .child(line.level),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .text_color(msg_color)
                                .child(line.msg),
                        )
                })),
        )
        // Inline terminal escape hatch
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .font_family("JetBrains Mono")
                .text_size(px(11.0))
                .child(
                    div()
                        .text_color(OK)
                        .child("root@edge-01"),
                )
                .child(
                    div()
                        .text_color(TEXT_FAINTER)
                        .child(":~#"),
                )
                .child(
                    div()
                        .text_color(TEXT_SECONDARY)
                        .child("systemctl status redis"),
                )
                .child(
                    div()
                        .w(px(7.0))
                        .h(px(14.0))
                        .bg(TEXT_PRIMARY),
                ),
        )
}
