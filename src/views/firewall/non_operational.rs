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
    // Crow's enable/inspect actions are ufw's; for other backends it only
    // says what to run.
    let ufw = backend == FirewallBackend::Ufw;
    let recommended = match backend {
        FirewallBackend::Firewalld => "sudo systemctl enable --now firewalld",
        _ => "sudo ufw default deny incoming && sudo ufw default allow outgoing && sudo ufw allow 22/tcp comment 'SSH' && sudo ufw enable",
    };

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
                .p(px(22.0))
                .flex()
                .flex_col()
                .gap(px(18.0))
                // Header Icon & Status
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(14.0))
                        .child(div().pt(px(2.0)).child(tabler_icon(TablerIcon::AlertTriangle).size(px(16.0)).text_color(WARN)))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(TEXT_PRIMARY)
                                        .child("NO FIREWALL RUNNING"),
                                )
                                .child(
                                    div()
                                        .font_family(FONT_MONO)
                                        .text_size(px(11.0))
                                        .text_color(TEXT_DIM)
                                        .child(backend.label().to_string()),
                                ),
                        ),
                )
                // Diagnostic Explanation Box
                .child(
                    div()
                        .p(px(14.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(TEXT_DIMMER)
                                .child("WHY"),
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
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(TEXT_DIMMER)
                                .child("FIREWALL TOOLS ON THE HOST"),
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
                                            .border_1()
                                            .border_color(BORDER_DEFAULT)
                                            .font_family(FONT_MONO)
                                            .text_size(px(10.0))
                                            .text_color(TEXT_SECONDARY)
                                            .child(b.clone())
                                            .into_any_element()
                                    }).collect()
                                }),
                        ),
                )
                // Recommended Terminal Command Card
                .child(
                    div()
                        .p(px(12.0))
                        .bg(BG_APP)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(TEXT_DIMMER)
                                .child(if ufw || backend == FirewallBackend::Firewalld { "TO TURN IT ON:" } else { "TO SET ONE UP (ufw):" }),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_SECONDARY)
                                .child(recommended),
                        ),
                )
                // Action Buttons
                .children(ufw.then(|| 
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
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .text_color(TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| {
                                    app_config.update(cx, |this, cx| {
                                        // Open Config first: entering it resets the selection
                                        // to a listed file, and user.rules belongs to Firewall.
                                        this.set_view("config", cx);
                                        this.select_managed_file("user.rules", cx);
                                    });
                                })
                                .child("user.rules"),
                        )
                        .child(
                            div()
                                .id("btn-quick-enable-firewall")
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(OK)
                                .cursor_pointer()
                                .hover(|s| s.bg(OK_BG))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(OK)
                                .on_click(move |_ev, _window, cx| {
                                    app_enable.update(cx, |this, cx| {
                                        this.toggle_firewall_active(cx);
                                    });
                                })
                                .child("TURN THE FIREWALL ON"),
                        ),
                )),
        )
}
