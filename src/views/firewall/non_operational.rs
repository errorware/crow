use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use super::models::FirewallBackend;

pub fn non_operational_view(
    backend: FirewallBackend,
    reason: &str,
    detected_binaries: &[String],
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let app_enable = app.clone();
    let app_config = app.clone();

    div()
        .id("firewall-non-operational-view")
        .size_full()
        .bg(BG_APP)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .p(px(32.0))
        .child(
            div()
                .w(px(640.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_PANEL)
                .rounded_lg()
                .p(px(24.0))
                .flex()
                .flex_col()
                .gap(px(18.0))
                // Header Icon & Status
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(14.0))
                        .child(
                            div()
                                .size(px(42.0))
                                .rounded_md()
                                .bg(WARN_BG)
                                .border_1()
                                .border_color(WARN)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(tabler_icon(TablerIcon::AlertTriangle).size(px(22.0)).text_color(WARN)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(TEXT_MAX)
                                        .child("FIREWALL ENGINE NOT OPERATIONAL"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_MUTED)
                                        .child(format!("Target system: {}", backend.label())),
                                ),
                        ),
                )
                // Diagnostic Explanation Box
                .child(
                    div()
                        .p(px(14.0))
                        .bg(hex_rgba(0x000000, 0.45))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .child("DIAGNOSTIC STATUS:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_PRIMARY)
                                .child(reason.to_string()),
                        ),
                )
                // Detected Binaries List
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_MUTED)
                                .child("DETECTED NETFILTER TOOLS ON HOST:"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.0))
                                .children(if detected_binaries.is_empty() {
                                    vec![
                                        div()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.5))
                                            .text_color(TEXT_FAINT)
                                            .child("No standard packet filter binaries found (ufw, firewalld, iptables)")
                                            .into_any_element(),
                                    ]
                                } else {
                                    detected_binaries.iter().map(|b| {
                                        div()
                                            .px(px(8.0))
                                            .py(px(3.0))
                                            .bg(BG_CONTROL)
                                            .border_1()
                                            .border_color(BORDER_DEFAULT)
                                            .rounded_sm()
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .text_color(TEXT_SECONDARY)
                                            .child(format!("✓ {}", b))
                                            .into_any_element()
                                    }).collect()
                                }),
                        ),
                )
                // Recommended Terminal Command Card
                .child(
                    div()
                        .p(px(12.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .rounded_sm()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hex_rgb(0x38bdf8))
                                .child("RECOMMENDED INITIALIZATION:"),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_MAX)
                                .child("sudo ufw default deny incoming && sudo ufw default allow outgoing && sudo ufw allow 22/tcp comment 'SSH' && sudo ufw enable"),
                        ),
                )
                // Action Buttons
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pt(px(6.0))
                        .child(
                            div()
                                .id("btn-open-ufw-config")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(BG_CONTROL)
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    app_config.update(cx, |this, cx| {
                                        // Open Config first: entering it resets the selection
                                        // to a listed file, and user.rules belongs to Firewall.
                                        this.set_view("config", cx);
                                        this.select_managed_file("user.rules", cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::FileText).size(px(13.0)).text_color(TEXT_SECONDARY))
                                .child("INSPECT /etc/ufw/user.rules"),
                        )
                        .child(
                            div()
                                .id("btn-quick-enable-firewall")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(14.0))
                                .py(px(6.0))
                                .bg(OK)
                                .rounded_sm()
                                .cursor_pointer()
                                .hover(|s| s.bg(hex_rgb(0x34d399)))
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x0a0a0c))
                                .on_click(move |_ev, _window, cx| {
                                    app_enable.update(cx, |this, cx| {
                                        this.toggle_firewall_active(cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::ShieldCheck).size(px(14.0)).text_color(rgb(0x0a0a0c)))
                                .child("ACTIVATE FIREWALL IN CROW"),
                        ),
                ),
        )
}
