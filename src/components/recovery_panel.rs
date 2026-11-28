//! The connection recovery panel (ERR-89).

use gpui_kit::component::input::Input;
use gpui_kit::*;

use crate::app::recovery::RecoveryPanel;
use crate::app::CrowApp;
use crate::host::diagnose::RecoveryAction;
use crate::theme::*;

pub fn recovery_panel(panel: &RecoveryPanel, server_name: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let button = |id: &'static str, label: &'static str, color: Rgba| {
        div()
            .id(id)
            .px(px(12.0))
            .py(px(5.0))
            .border_1()
            .border_color(color)
            .text_color(color)
            .font_weight(FontWeight::BOLD)
            .cursor_pointer()
            .hover(|s| s.bg(bg_row_hover()))
            .child(label)
    };
    let d = panel.diagnosis.as_ref();
    let actions = d.map(|d| d.actions.clone()).unwrap_or_default();
    let idle = panel.busy.is_none();

    div()
        .absolute()
        .inset_0()
        // Blocks the page underneath: no hover or clicks through.
        .occlude()
        .bg(hex_rgba(0x000000, 0.55))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("recovery-panel")
                .w(px(680.0))
                .flex()
                .flex_col()
                .bg(bg_panel())
                .border_1()
                .border_color(border_danger())
                .font_family(FONT_MONO)
                .text_size(font_px(11.0))
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(12.0))
                        .border_b_1()
                        .border_color(border_panel())
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .gap(px(8.0))
                                .child(div().text_size(font_px(13.0)).font_weight(FontWeight::BOLD).text_color(theme().crit).child(d.map(|d| d.title.clone()).unwrap_or_else(|| t!("recovery-panel-checking").into())))
                                .child(div().text_size(font_px(13.0)).text_color(text_max()).child(server_name.to_string())),
                        )
                        .children(d.and_then(|d| crate::components::english_beside::english_beside(crate::i18n::english_of(&d.title, "host-diagnose-"))))
                        .child(div().text_color(text_secondary()).child(d.map(|d| d.what.clone()).unwrap_or_default())),
                )
                .children(d.map(|d| {
                    div()
                        .px(px(16.0))
                        .py(px(10.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .border_b_1()
                        .border_color(border_row())
                        .children(d.facts.iter().map(|(k, v)| {
                            div()
                                .flex()
                                .gap(px(10.0))
                                .child(div().w(px(110.0)).flex_none().text_color(text_dim()).child(k.clone()))
                                .child(div().flex_1().min_w(px(0.0)).text_color(text_primary()).child(v.clone()))
                        }))
                }))
                .children(d.filter(|d| !d.steps.is_empty()).map(|d| {
                    div()
                        .px(px(16.0))
                        .py(px(10.0))
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .border_b_1()
                        .border_color(border_row())
                        .child(div().text_size(font_px(10.0)).font_weight(FontWeight::BOLD).text_color(text_dimmer()).child(t!("recovery-panel-what-to-do")))
                        .children(d.steps.iter().enumerate().map(|(i, s)| {
                            div().flex().gap(px(8.0)).child(div().flex_none().text_color(text_dim()).child(format!("{}.", i + 1))).child(div().flex_1().min_w(px(0.0)).text_color(text_secondary()).child(s.clone()))
                        }))
                }))
                .children(panel.busy.clone().map(|b| div().px(px(16.0)).py(px(6.0)).text_color(theme().warn).child(b)))
                .children(panel.error.clone().map(|e| div().px(px(16.0)).py(px(6.0)).text_color(theme().crit).child(e)))
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(10.0))
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .children((idle && actions.contains(&RecoveryAction::TrustNewHostKey)).then(|| div().w(px(150.0)).child(Input::new(&panel.confirm))))
                        .children((idle && actions.contains(&RecoveryAction::TrustNewHostKey)).then(|| {
                            let app = app.clone();
                            button("btn-recovery-trust", t!("recovery-panel-trust-new-key"), theme().crit).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.recovery_trust_new_key(cx)))
                        }))
                        .children((idle && actions.contains(&RecoveryAction::InstallKeyWithPassword)).then(|| {
                            let app = app.clone();
                            button("btn-recovery-install-key", t!("recovery-panel-install-key-with-password"), theme().ok).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.recovery_install_key(cx)))
                        }))
                        .children((idle && actions.contains(&RecoveryAction::Retry)).then(|| {
                            let app = app.clone();
                            button("btn-recovery-retry", t!("recovery-panel-try-again"), text_secondary()).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.recovery_retry(cx)))
                        }))
                        .child(div().flex_1())
                        .child({
                            let app = app.clone();
                            button("btn-recovery-close", t!("recovery-panel-close"), text_dim()).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.close_recovery(cx)))
                        }),
                ),
        )
}
