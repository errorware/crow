//! Fleet → IMPORT FROM PROVIDERS (ERR-46): every instance the configured
//! provider accounts list, which ones are already in the fleet, and IMPORT
//! for the rest (through Add Server, so host keys and logins are checked).

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use crow_provider_core::hosts::InstanceStatus;

use crate::app::providers::ImportState;
use crate::app::CrowApp;
use crate::theme::*;

fn status_label(s: &InstanceStatus) -> (String, Rgba) {
    match s {
        InstanceStatus::Running => ("running".into(), OK),
        InstanceStatus::Stopped => ("stopped".into(), TEXT_DIMMER),
        InstanceStatus::Other(o) => (o.clone(), WARN),
        other => (format!("{other:?}").to_lowercase(), WARN),
    }
}

pub fn import_panel(state: &ImportState, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_scrim, a_close) = (app.clone(), app.clone());
    let importable = state.rows.iter().filter(|r| r.enrolled_as.is_none()).count();
    div()
        .id("import-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .on_click(move |_ev, _window, cx| a_scrim.update(cx, |this, cx| this.close_import(cx)))
        .child(
            div()
                .id("import-panel")
                .w(px(860.0))
                .max_h(px(620.0))
                .overflow_y_scrollbar()
                .p(px(16.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("IMPORT FROM PROVIDERS"))
                        .child(div().text_color(TEXT_DIMMER).child(if state.loading > 0 {
                            format!("asking {} account(s)…", state.loading)
                        } else {
                            format!("{} instance(s) · {importable} not in the fleet", state.rows.len())
                        }))
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("import-close")
                                .cursor_pointer()
                                .text_color(TEXT_DIMMER)
                                .hover(|s| s.text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _window, cx| a_close.update(cx, |this, cx| this.close_import(cx)))
                                .child("✕"),
                        ),
                )
                .child(div().text_color(TEXT_MUTED).line_height(px(15.0)).child("Instances already in the fleet are matched by IP and linked, so their region comes from the provider. IMPORT opens Add Server filled in: the host key and login are checked as usual."))
                .children((state.linked > 0).then(|| div().text_color(OK).child(format!("Linked {} enrolled server(s) to their instances.", state.linked))))
                .children(state.errors.iter().map(|e| div().text_color(CRIT).child(e.clone())))
                .children(state.missing.iter().map(|m| div().text_color(WARN).child(format!("Linked but missing: {m}"))))
                .children(state.rows.iter().map(|row| {
                    let inst = &row.instance;
                    let country = inst.region.as_deref().and_then(|c| crate::region::locate(&row.provider_name, c)).map(|(cc, _)| cc);
                    let ip = inst.ipv4.first().or(inst.ipv6.first()).map(|ip| ip.to_string()).unwrap_or_else(|| "no address".into());
                    let (status, status_color) = status_label(&inst.status);
                    let (app, account, id) = (app.clone(), row.account.clone(), inst.id.0.clone());
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .px(px(8.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(BORDER_ROW)
                        .child(div().w(px(18.0)).flex_none().children(country.map(|cc| crate::components::flag::flag(cc, 11.0))))
                        .child(div().w(px(200.0)).flex_none().font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(inst.label.clone()))
                        .child(div().w(px(80.0)).flex_none().text_color(TEXT_DIMMER).child(row.provider_name.clone()))
                        .child(div().w(px(130.0)).flex_none().text_color(TEXT_SECONDARY).child(ip))
                        .child(div().w(px(90.0)).flex_none().text_color(TEXT_DIMMER).child(inst.region.clone().unwrap_or_default()))
                        .child(div().w(px(70.0)).flex_none().text_color(status_color).child(status))
                        .child(div().flex_1())
                        .child(match &row.enrolled_as {
                            Some(server) => div().text_color(TEXT_DIMMER).child(format!("in fleet as {server}")).into_any_element(),
                            None => div()
                                .id(SharedString::from(format!("import-{account}-{id}")))
                                .px(px(9.0))
                                .py(px(3.0))
                                .border_1()
                                .border_color(OK.opacity(0.6))
                                .text_color(OK)
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let (account, id) = (account.clone(), id.clone());
                                    app.update(cx, |this, cx| this.import_instance(&account, &id, cx));
                                })
                                .child("IMPORT →")
                                .into_any_element(),
                        })
                })),
        )
}
