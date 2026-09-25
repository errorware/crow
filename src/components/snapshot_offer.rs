//! Before a change that can lock you out (sshd, firewall rules) on a server
//! whose provider can snapshot it: take a whole-server snapshot first?

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::configs::SnapshotOffer;
use crate::app::CrowApp;
use crate::theme::*;

fn button(id: &'static str, label: String, color: Rgba, enabled: bool, on_click: impl Fn(&mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(if enabled { color.opacity(0.6) } else { BORDER_DEFAULT })
        .text_color(if enabled { color } else { TEXT_FAINTER })
        .font_weight(FontWeight::BOLD)
        .when(enabled, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| on_click(cx)))
        .child(label)
}

pub fn snapshot_offer(offer: &SnapshotOffer, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_snap, a_skip, a_cancel, a_remember) = (app.clone(), app.clone(), app.clone(), app);
    let busy = offer.busy;
    div()
        .id("snapshot-offer-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("snapshot-offer")
                .w(px(560.0))
                .p(px(16.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(WARN.opacity(0.6))
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(format!("Snapshot the whole server at {} first?", offer.provider)))
                .child(div().line_height(px(15.0)).text_color(TEXT_MUTED).child(format!(
                    "You're about to apply {} ({}). A mistake here can lock you out of SSH, and then Crow's file backup can't reach the server to undo it. A {} snapshot restores the whole machine from outside. The change waits until {} accepts the snapshot; if it fails, nothing is applied.",
                    offer.file, offer.description, offer.provider, offer.provider
                )))
                .children(offer.error.clone().map(|e| div().text_color(CRIT).child(e)))
                .child(
                    div()
                        .id("snapshot-offer-remember")
                        .flex()
                        .gap(px(8.0))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| a_remember.update(cx, |this, cx| this.toggle_snapshot_offer_remember(cx)))
                        .child(div().text_color(if offer.remember { TEXT_PRIMARY } else { TEXT_DIMMER }).child(if offer.remember { "[x]" } else { "[ ]" }))
                        .child(div().text_color(TEXT_SECONDARY).child("Remember my choice for these changes")),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .child(button("snapshot-offer-yes", if busy { "TAKING SNAPSHOT…".into() } else { "SNAPSHOT FIRST, THEN APPLY".into() }, OK, !busy, move |cx| a_snap.update(cx, |this, cx| this.answer_snapshot_offer(true, cx))))
                        .child(button("snapshot-offer-no", "APPLY WITHOUT".into(), WARN, !busy, move |cx| a_skip.update(cx, |this, cx| this.answer_snapshot_offer(false, cx))))
                        .child(button("snapshot-offer-cancel", "CANCEL".into(), TEXT_SECONDARY, !busy, move |cx| a_cancel.update(cx, |this, cx| this.cancel_snapshot_offer(cx)))),
                ),
        )
}
