use gpui_kit::*;
use crate::theme::*;
use crate::components::sparkline::sparkline;

const CPU_POINTS: &[(f32, f32)] = &[
    (0.0, 20.0), (8.0, 17.0), (16.0, 21.0), (24.0, 13.0),
    (32.0, 16.0), (40.0, 9.0), (48.0, 14.0), (56.0, 11.0),
    (64.0, 18.0), (72.0, 12.0), (80.0, 8.0), (86.0, 10.0),
];

const MEM_POINTS: &[(f32, f32)] = &[
    (0.0, 18.0), (8.0, 17.0), (16.0, 16.0), (24.0, 16.0),
    (32.0, 14.0), (40.0, 13.0), (48.0, 13.0), (56.0, 11.0),
    (64.0, 10.0), (72.0, 9.0), (80.0, 7.0), (86.0, 6.0),
];

const LOAD_POINTS: &[(f32, f32)] = &[
    (0.0, 14.0), (8.0, 15.0), (16.0, 12.0), (24.0, 17.0),
    (32.0, 15.0), (40.0, 19.0), (48.0, 16.0), (56.0, 20.0),
    (64.0, 17.0), (72.0, 15.0), (80.0, 16.0), (86.0, 13.0),
];

const NET_POINTS: &[(f32, f32)] = &[
    (0.0, 22.0), (8.0, 15.0), (16.0, 18.0), (24.0, 10.0),
    (32.0, 14.0), (40.0, 6.0), (48.0, 12.0), (56.0, 8.0),
    (64.0, 16.0), (72.0, 7.0), (80.0, 11.0), (86.0, 5.0),
];

pub fn stat_strip() -> impl IntoElement {
    div()
        .flex_none()
        .flex()
        .w_full()
        .bg(BG_PANEL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        // 1. CPU LOAD
        .child(
            div()
                .flex_1()
                .p(px(10.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
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
                                .font_family("JetBrains Mono")
                                .text_size(px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(TEXT_PRIMARY)
                                .child("38.4")
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child("%"),
                                ),
                        )
                        .child(sparkline(CPU_POINTS, OK)),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(6.0))
                        .child("8 vCPU · peak 71.2%"),
                ),
        )
        // 2. MEMORY
        .child(
            div()
                .flex_1()
                .p(px(10.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
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
                                .font_family("JetBrains Mono")
                                .text_size(px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(TEXT_PRIMARY)
                                .child("11.7")
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child("/16 GB"),
                                ),
                        )
                        .child(sparkline(MEM_POINTS, WARN)),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(3.0))
                        .bg(rgb(0x1a1b21))
                        .mt(px(8.0))
                        .child(
                            div()
                                .w(px(150.0))
                                .h_full()
                                .bg(WARN),
                        ),
                ),
        )
        // 3. DISK
        .child(
            div()
                .flex_1()
                .p(px(10.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("DISK / (NVME)"),
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
                                .font_family("JetBrains Mono")
                                .text_size(px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(TEXT_PRIMARY)
                                .child("412")
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child("/960 GB"),
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
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(6.0))
                        .child("inodes 6.2% · iowait 0.4%"),
                ),
        )
        // 4. LOAD AVG
        .child(
            div()
                .flex_1()
                .p(px(10.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
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
                                .font_family("JetBrains Mono")
                                .text_size(px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(TEXT_PRIMARY)
                                .child("2.14"),
                        )
                        .child(sparkline(LOAD_POINTS, TEXT_DIMMER)),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(6.0))
                        .child("1m 2.14 · 5m 1.88 · 15m 1.42"),
                ),
        )
        // 5. UPTIME
        .child(
            div()
                .flex_1()
                .p(px(10.0))
                .px(px(14.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("UPTIME"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(26.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(TEXT_PRIMARY)
                        .mt(px(5.0))
                        .child("64")
                        .child(div().text_size(px(13.0)).text_color(TEXT_DIM).child("d "))
                        .child("07")
                        .child(div().text_size(px(13.0)).text_color(TEXT_DIM).child("h")),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(12.0))
                        .child("boot 2026-07-15 02:11Z"),
                ),
        )
        // 6. NETWORK eth0
        .child(
            div()
                .flex_1()
                .p(px(10.0))
                .px(px(14.0))
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("NETWORK eth0"),
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
                                .font_family("JetBrains Mono")
                                .text_size(px(26.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(TEXT_PRIMARY)
                                .child("184")
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(TEXT_DIM)
                                        .child("Mb/s"),
                                ),
                        )
                        .child(sparkline(NET_POINTS, TEXT_SECONDARY)),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_FAINT)
                        .mt(px(6.0))
                        .child("↓ 184 · ↑ 62 · retrans 0.01%"),
                ),
        )
}
