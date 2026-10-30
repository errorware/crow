//! How a fleet run that went on in the background ended: a toast at the
//! bottom right. A good result goes by itself; a failure stays until read.

use gpui_kit::*;

use crate::app::fleet_run::RunToast;
use crate::app::CrowApp;
use crate::components::icon_button::icon_button;
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::theme::*;

pub fn run_toast(toast: &RunToast, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_view, app_close) = (app.clone(), app);
    let (ink, icon) = if toast.ok { (OK, TablerIcon::Check) } else { (CRIT, TablerIcon::X) };
    div()
        .absolute()
        .right(px(16.0))
        .bottom(px(36.0))
        .child(
            div()
                .id("run-toast")
                .occlude()
                .w(px(380.0))
                .flex()
                .items_start()
                .gap(px(10.0))
                .p(px(12.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(ink)
                .shadow_lg()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .child(div().pt(px(1.0)).child(tabler_icon(icon).size(px(14.0)).text_color(ink)))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child(toast.title.clone()))
                        .child(div().text_color(if toast.ok { TEXT_DIM } else { CRIT }).child(toast.summary.clone()))
                        .child(
                            div()
                                .id("run-toast-view")
                                .pt(px(2.0))
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(TEXT_SECONDARY)
                                .cursor_pointer()
                                .hover(|s| s.text_color(TEXT_MAX))
                                .on_click(move |_ev, _window, cx| app_view.update(cx, |this, cx| this.show_fleet_run(cx)))
                                .child(t!("run-toast-view-details")),
                        ),
                )
                .child(icon_button("run-toast-dismiss", TablerIcon::X, false).on_click(move |_ev, _window, cx| app_close.update(cx, |this, cx| this.dismiss_run_toast(cx)))),
        )
}
