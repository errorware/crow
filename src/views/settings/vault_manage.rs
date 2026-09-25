//! Vault & Security while Locked: change the password, or go back to Open.

use gpui_kit::component::input::Input;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::vault_manage::{VaultForm, VaultFormInputs, VaultFormState};
use crate::app::CrowApp;
use crate::security::stance::Stance;
use crate::theme::*;

fn button(id: &'static str, label: &'static str, color: Rgba, enabled: bool, on_click: impl Fn(&mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .px(px(9.0))
        .py(px(4.0))
        .border_1()
        .border_color(if enabled { color.opacity(0.6) } else { BORDER_DEFAULT })
        .text_color(if enabled { color } else { TEXT_FAINTER })
        .font_weight(FontWeight::BOLD)
        .when(enabled, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| on_click(cx)))
        .child(label)
}

pub fn render(state: &VaultFormState, inputs: Option<&VaultFormInputs>, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_change, a_open, a_submit, a_cancel, a_ack) = (app.clone(), app.clone(), app.clone(), app.clone(), app);
    let form = state.open;
    div()
        .mx(px(14.0))
        .my(px(10.0))
        .p(px(12.0))
        .border_1()
        .border_color(BORDER_PANEL)
        .bg(BG_PANEL)
        .flex()
        .flex_col()
        .gap(px(10.0))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_1().font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("PASSWORD"))
                .child(button("vault-change-pw", "CHANGE PASSWORD…", TEXT_SECONDARY, form.is_none(), move |cx| a_change.update(cx, |this, cx| this.open_vault_form(Some(VaultForm::ChangePassword), cx))))
                .child(button("vault-go-open", "GO BACK TO OPEN…", WARN, form.is_none(), move |cx| a_open.update(cx, |this, cx| this.open_vault_form(Some(VaultForm::GoOpen), cx)))),
        )
        .children(state.done.clone().map(|d| div().text_color(OK).child(d)))
        .children(form.map(|form| {
            let inputs = inputs.filter(|i| i.form == form);
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children((form == VaultForm::GoOpen).then(|| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().text_color(WARN).font_weight(FontWeight::BOLD).child("Going back to OPEN means:"))
                        .children(Stance::Open.exposed().iter().map(|e| div().flex().gap(px(6.0)).child(div().text_color(WARN).child("•")).child(div().flex_1().line_height(px(15.0)).text_color(TEXT_SECONDARY).child(*e))))
                }))
                .children(inputs.map(|i| {
                    div().flex().flex_col().gap(px(6.0)).max_w(px(420.0)).children(i.inputs.iter().map(|(_, input)| Input::new(input).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0))))
                }))
                .children((form == VaultForm::GoOpen).then(|| {
                    let ack = state.ack;
                    div()
                        .id("vault-go-open-ack")
                        .flex()
                        .gap(px(8.0))
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            a_ack.update(cx, |this, cx| {
                                this.vault_form.ack = !this.vault_form.ack;
                                cx.notify();
                            })
                        })
                        .child(div().text_color(if ack { WARN } else { TEXT_DIMMER }).child(if ack { "[x]" } else { "[ ]" }))
                        .child(div().text_color(TEXT_SECONDARY).child("I understand: without the password, anything running as me can read Crow's secrets."))
                }))
                .children(state.error.clone().map(|e| div().text_color(CRIT).child(e)))
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .child(button(
                            "vault-form-submit",
                            match form {
                                VaultForm::ChangePassword => "CHANGE PASSWORD",
                                VaultForm::GoOpen => "GO OPEN",
                            },
                            if form == VaultForm::GoOpen { WARN } else { OK },
                            !state.busy && (form != VaultForm::GoOpen || state.ack),
                            move |cx| a_submit.update(cx, |this, cx| this.submit_vault_form(cx)),
                        ))
                        .child(button("vault-form-cancel", "CANCEL", TEXT_SECONDARY, !state.busy, move |cx| a_cancel.update(cx, |this, cx| this.open_vault_form(None, cx)))),
                )
        }))
}
