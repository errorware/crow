use gpui_kit::*;
use crate::theme::*;
use crate::components::icons::{TablerIcon, tabler_icon};

pub fn pending_diff_rail() -> impl IntoElement {
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
                        .child("PENDING DIFF"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("+2 −2"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child("unified · 1 file"),
                ),
        )
        // Scrollable content body
        .child(
            div()
                .id("pending-diff-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                // Diff Block
                .child(
                    div()
                        .flex_none()
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                .py(px(8.0))
                .font_family("JetBrains Mono")
                .text_size(px(10.5))
                .flex()
                .flex_col()
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .text_color(TEXT_FAINT)
                        .child("--- /etc/postgresql/16/main/pg_hba.conf"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .text_color(TEXT_FAINT)
                        .child("+++ crow.staged (03:41:09Z)"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .bg(DIFF_HUNK_BG)
                        .text_color(TEXT_DIMMER)
                        .child("@@ -9,2 +9,2 @@ IPv4 local connections"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .bg(DIFF_DEL_BG)
                        .text_color(CRIT_INK)
                        .child("- host  all  all  10.0.4.0/24  trust"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .bg(DIFF_ADD_BG)
                        .text_color(OK_INK)
                        .child("+ host  all  all  10.0.4.0/24  scram-sha-256"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .bg(DIFF_HUNK_BG)
                        .text_color(TEXT_DIMMER)
                        .child("@@ -13,1 +13,1 @@ replication"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .bg(DIFF_DEL_BG)
                        .text_color(CRIT_INK)
                        .child("- host  replication  repl  0.0.0.0/0  md5"),
                )
                .child(
                    div()
                        .px(px(12.0))
                        .py(px(1.0))
                        .bg(DIFF_ADD_BG)
                        .text_color(OK_INK)
                        .child("+ host  replication  repl  10.0.4.11/32  scram-sha-256"),
                ),
        )
        // Apply Plan
        .child(
            div()
                .flex_none()
                .border_b_1()
                .border_color(BORDER_PANEL)
                .p(px(10.0))
                .px(px(12.0))
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("APPLY PLAN"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(tabler_icon(TablerIcon::Check).size(px(12.0)).text_color(OK))
                        .child(div().flex_1().min_w(px(0.0)).text_color(OK).child("Back up to /var/backups/crow/pg_hba.2026-09-17T0341Z"))
                        .child(div().flex_none().text_color(TEXT_FAINT).child("12 KB")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(tabler_icon(TablerIcon::Check).size(px(12.0)).text_color(OK))
                        .child(div().flex_1().min_w(px(0.0)).text_color(OK).child("Validate against postgres 16 grammar"))
                        .child(div().flex_none().text_color(TEXT_FAINT).child("no errors")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(tabler_icon(TablerIcon::Check).size(px(12.0)).text_color(OK))
                        .child(div().flex_1().min_w(px(0.0)).text_color(OK).child("Check 6 live sessions against new rules"))
                        .child(div().flex_none().text_color(TEXT_FAINT).child("0 evicted")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().w(px(12.0)).flex_none().text_color(TEXT_DIM).child("○"))
                        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_DIM).child("Write file via atomic rename (0640 postgres:postgres)"))
                        .child(div().flex_none().text_color(TEXT_FAINT).child("staged")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().w(px(12.0)).flex_none().text_color(TEXT_DIM).child("○"))
                        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_DIM).child("SELECT pg_reload_conf() — no restart, no dropped sockets"))
                        .child(div().flex_none().text_color(TEXT_FAINT).child("~200ms")),
                ),
        )
        // Auto-Rollback Armed Notice
        .child(
            div()
                .flex_none()
                .border_b_1()
                .border_color(BORDER_PANEL)
                .p(px(10.0))
                .px(px(12.0))
                .bg(rgb(0x0c0a0a))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(7.0))
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(10.0))
                                .text_color(WARN)
                                .child("⟲"),
                        )
                        .child(
                            div()
                                .font_family("Inter")
                                .text_size(px(10.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(WARN)
                                .child("AUTO-ROLLBACK ARMED"),
                        ),
                )
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .child("If Crow cannot re-authenticate within 60s of reload, the previous file is restored and postgres is reloaded again."),
                ),
        )
        // Blast Radius
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .p(px(10.0))
                .px(px(12.0))
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(TEXT_DIMMER)
                        .child("BLAST RADIUS"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().size(px(6.0)).rounded_full().bg(OK).flex_none())
                        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child("api-01, api-02 (acme_app)"))
                        .child(div().flex_none().text_color(TEXT_DIMMER).child("rule 05/06 unchanged")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().size(px(6.0)).rounded_full().bg(WARN).flex_none())
                        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child("standby db-replica-01"))
                        .child(div().flex_none().text_color(TEXT_DIMMER).child("address narrowed")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().size(px(6.0)).rounded_full().bg(OK).flex_none())
                        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child("analyst pool 10.0.9.0/24"))
                        .child(div().flex_none().text_color(TEXT_DIMMER).child("ldap untouched")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .child(div().size(px(6.0)).rounded_full().bg(WARN).flex_none())
                        .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child("ad-hoc psql from 10.0.4.0/24"))
                        .child(div().flex_none().text_color(TEXT_DIMMER).child("password now required")),
                ),
        ))
        // Action footer
        .child(
            div()
                .h(px(42.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_TERTIARY)
                        .border_1()
                        .border_color(BORDER_KEY)
                        .px(px(9.0))
                        .py(px(5.0))
                        .child("REVERT"),
                )
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .px(px(9.0))
                        .py(px(5.0))
                        .child("DRY RUN ")
                        .child(div().text_color(TEXT_DIMMER).child("⌘⇧⏎")),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family("JetBrains Mono")
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x0a0a0c))
                        .bg(OK)
                        .px(px(11.0))
                        .py(px(6.0))
                        .child("APPLY & RELOAD ⌘⏎"),
                ),
        )
}
