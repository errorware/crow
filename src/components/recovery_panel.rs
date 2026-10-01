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
            .hover(|s| s.bg(BG_ROW_HOVER))
            .child(label)
    };
    let d = panel.diagnosis.as_ref();
    let actions = d.map(|d| d.actions.clone()).unwrap_or_default();
    let idle = panel.busy.is_none();

    div()
        .absolute()
        .inset_0()
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
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DANGER)
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(12.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .gap(px(8.0))
                                .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(CRIT).child(d.map(|d| d.title.clone()).unwrap_or_else(|| "CHECKING".into())))
                                .child(div().text_size(px(13.0)).text_color(TEXT_MAX).child(server_name.to_string())),
                        )
                        .child(div().text_color(TEXT_SECONDARY).child(d.map(|d| d.what.clone()).unwrap_or_default())),
                )
                .children(d.map(|d| {
                    div()
                        .px(px(16.0))
                        .py(px(10.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .children(d.facts.iter().map(|(k, v)| {
                            div()
                                .flex()
                                .gap(px(10.0))
                                .child(div().w(px(110.0)).flex_none().text_color(TEXT_DIM).child(k.clone()))
                                .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_PRIMARY).child(v.clone()))
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
                        .border_color(BORDER_ROW)
                        .child(div().text_size(px(10.0)).font_weight(FontWeight::BOLD).text_color(TEXT_DIMMER).child("WHAT TO DO"))
                        .children(d.steps.iter().enumerate().map(|(i, s)| {
                            div().flex().gap(px(8.0)).child(div().flex_none().text_color(TEXT_DIM).child(format!("{}.", i + 1))).child(div().flex_1().min_w(px(0.0)).text_color(TEXT_SECONDARY).child(s.clone()))
                        }))
                }))
                .children(panel.busy.clone().map(|b| div().px(px(16.0)).py(px(6.0)).text_color(WARN).child(b)))
                .children(panel.error.clone().map(|e| div().px(px(16.0)).py(px(6.0)).text_color(CRIT).child(e)))
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
                            button("btn-recovery-trust", "TRUST NEW KEY", CRIT).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.recovery_trust_new_key(cx)))
                        }))
                        .children((idle && actions.contains(&RecoveryAction::InstallKeyWithPassword)).then(|| {
                            let app = app.clone();
                            button("btn-recovery-install-key", "INSTALL KEY WITH PASSWORD", OK).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.recovery_install_key(cx)))
                        }))
                        .children((idle && actions.contains(&RecoveryAction::Retry)).then(|| {
                            let app = app.clone();
                            button("btn-recovery-retry", "TRY AGAIN", TEXT_SECONDARY).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.recovery_retry(cx)))
                        }))
                        .child(div().flex_1())
                        .child({
                            let app = app.clone();
                            button("btn-recovery-close", "CLOSE", TEXT_DIM).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.close_recovery(cx)))
                        }),
                ),
        )
}
