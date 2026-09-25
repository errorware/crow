//! The security stance badge (titlebar) and its panel (ERR-60): which
//! stance Crow is in, what it protects, what it exposes, what it costs, and
//! what weakens it right now.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::{CrowApp, Screen};
use crate::security::stance::{Stance, StanceReport};
use crate::theme::*;

fn stance_color(stance: Stance) -> Rgba {
    match stance {
        Stance::Open => WARN,
        Stance::Locked => OK,
    }
}

/// "OPEN" / "LOCKED", with how many things weaken it.
pub fn stance_badge(report: &StanceReport, app: Entity<CrowApp>) -> impl IntoElement {
    let color = stance_color(report.stance);
    let weak = report.findings.len();
    div()
        .id("stance-badge")
        .flex()
        .items_center()
        .gap(px(5.0))
        .px(px(7.0))
        .py(px(2.0))
        .border_1()
        .border_color(color.opacity(0.6))
        .bg(color.opacity(0.08))
        .cursor_pointer()
        .hover(|s| s.bg(color.opacity(0.16)))
        .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation()) // not a window drag
        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.toggle_stance_panel(cx)))
        .child(div().font_weight(FontWeight::BOLD).text_color(color).child(report.stance.label()))
        .children((weak > 0).then(|| div().font_weight(FontWeight::BOLD).text_color(CRIT).child(format!("· {weak} weak"))))
}

fn list(title: &'static str, color: Rgba, items: impl IntoIterator<Item = String>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(div().text_size(px(9.5)).font_weight(FontWeight::BOLD).text_color(TEXT_FAINT).child(title))
        .children(items.into_iter().map(move |t| {
            div()
                .flex()
                .gap(px(6.0))
                .child(div().flex_none().text_color(color).child("•"))
                .child(div().flex_1().line_height(px(15.0)).text_color(TEXT_SECONDARY).child(t))
        }))
}

fn button(id: &'static str, label: &'static str, color: Rgba, on_click: impl Fn(&mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(color.opacity(0.6))
        .text_color(color)
        .font_weight(FontWeight::BOLD)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _window, cx| on_click(cx))
        .child(label)
}

pub fn stance_panel(report: &StanceReport, app: Entity<CrowApp>) -> impl IntoElement {
    let stance = report.stance;
    let color = stance_color(stance);
    let other = match stance {
        Stance::Open => Stance::Locked,
        Stance::Locked => Stance::Open,
    };
    let (a_close, a_switch, a_settings) = (app.clone(), app.clone(), app.clone());

    div()
        .id("stance-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .on_click(move |_ev, _window, cx| a_close.update(cx, |this, cx| this.toggle_stance_panel(cx)))
        .child(
            div()
                .id("stance-panel")
                .absolute()
                .top(px(40.0))
                .right(px(10.0))
                .w(px(540.0))
                .max_h(px(640.0))
                .overflow_y_scrollbar()
                .p(px(16.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(color.opacity(0.7))
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(10.0))
                        .child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child("SECURITY STANCE"))
                        .child(div().text_size(px(15.0)).font_weight(FontWeight::BOLD).text_color(color).child(stance.label()))
                        .child(div().text_color(TEXT_DIMMER).child(match stance {
                            Stance::Open => "no vault password · key held by your OS keyring",
                            Stance::Locked => "vault password + 2FA · key locked by your password",
                        })),
                )
                .children((!report.findings.is_empty()).then(|| list("WEAKENING IT RIGHT NOW", CRIT, report.findings.clone())))
                .child(list("WHAT IT PROTECTS", OK, stance.protects().iter().map(|s| s.to_string())))
                .child(list("WHAT YOU'RE EXPOSED TO", WARN, stance.exposed().iter().map(|s| s.to_string())))
                .child(list("WHAT IT COSTS", TEXT_DIMMER, stance.costs().iter().map(|s| s.to_string())))
                .child(
                    div()
                        .pt(px(10.0))
                        .border_t_1()
                        .border_color(BORDER_ROW)
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(div().text_color(TEXT_MUTED).line_height(px(15.0)).child(format!(
                            "The other stance, {}: {}",
                            other.label(),
                            match other {
                                Stance::Locked => "nothing is usable without your password and a 2FA code, at the price of unlocking every time and losing the secrets if you lose either.",
                                Stance::Open => "no unlocking, at the price of anything running as you being able to read every secret.",
                            }
                        )))
                        .child(
                            div()
                                .flex()
                                .gap(px(6.0))
                                .child(match stance {
                                    Stance::Open => button("stance-switch", "SWITCH TO LOCKED →", OK, move |cx| {
                                        a_switch.update(cx, |this, cx| {
                                            this.stance_panel_open = false;
                                            this.set_screen(Screen::VaultSetup, cx);
                                        })
                                    })
                                    .into_any_element(),
                                    Stance::Locked => button("stance-lock", "LOCK NOW", OK, move |cx| {
                                        a_switch.update(cx, |this, cx| {
                                            this.stance_panel_open = false;
                                            this.lock(cx);
                                        })
                                    })
                                    .into_any_element(),
                                })
                                .child(button("stance-settings", "VAULT & SECURITY SETTINGS", TEXT_SECONDARY, move |cx| {
                                    a_settings.update(cx, |this, cx| {
                                        this.stance_panel_open = false;
                                        this.screen = Screen::Settings;
                                        this.set_settings_section(crate::app::SettingsSection::Security, cx);
                                    })
                                })),
                        ),
                ),
        )
}

fn stance_card(stance: Stance, action: AnyElement, extra: Option<AnyElement>) -> impl IntoElement {
    let color = stance_color(stance);
    div()
        .flex_1()
        .min_w(px(0.0))
        .p(px(12.0))
        .border_1()
        .border_color(color.opacity(0.6))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_baseline()
                .gap(px(8.0))
                .child(div().text_size(px(14.0)).font_weight(FontWeight::BOLD).text_color(color).child(stance.label()))
                .child(div().text_color(TEXT_DIMMER).child(match stance {
                    Stance::Open => "no password",
                    Stance::Locked => "password + 2FA",
                })),
        )
        .child(list("PROTECTS", OK, stance.protects().iter().map(|s| s.to_string())))
        .child(list("EXPOSED TO", WARN, stance.exposed().iter().map(|s| s.to_string())))
        .child(list("COSTS", TEXT_DIMMER, stance.costs().iter().map(|s| s.to_string())))
        .child(div().flex_1())
        .children(extra)
        .child(action)
}

/// Shown the first time a secret is saved while Open: choose a stance on
/// purpose. Staying Open needs an explicit "I understand".
pub fn stance_choice_modal(acknowledged: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let (a_cancel, a_tick, a_open, a_locked) = (app.clone(), app.clone(), app.clone(), app);
    let open_action = div()
        .id("stance-choose-open")
        .px(px(10.0))
        .py(px(6.0))
        .border_1()
        .border_color(if acknowledged { WARN.opacity(0.7) } else { BORDER_DEFAULT })
        .text_color(if acknowledged { WARN } else { TEXT_FAINTER })
        .font_weight(FontWeight::BOLD)
        .when(acknowledged, |d| {
            d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _window, cx| a_open.update(cx, |this, cx| this.choose_open(cx)))
        })
        .child("STAY OPEN AND SAVE")
        .into_any_element();
    let tick = div()
        .id("stance-ack")
        .flex()
        .gap(px(8.0))
        .cursor_pointer()
        .on_click(move |_ev, _window, cx| {
            a_tick.update(cx, |this, cx| {
                this.stance_choice_ack = !this.stance_choice_ack;
                cx.notify();
            })
        })
        .child(div().flex_none().text_color(if acknowledged { WARN } else { TEXT_DIMMER }).child(if acknowledged { "[x]" } else { "[ ]" }))
        .child(div().flex_1().line_height(px(15.0)).text_color(TEXT_SECONDARY).child("I understand: while I'm logged in, anything running as me can read these secrets."))
        .into_any_element();
    let locked_action = button("stance-choose-locked", "GO LOCKED: SET A PASSWORD →", OK, move |cx| a_locked.update(cx, |this, cx| this.choose_locked(cx))).into_any_element();

    div()
        .id("stance-choice-scrim")
        .occlude()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000b0))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("stance-choice")
                .w(px(900.0))
                .p(px(18.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("Where should this secret stand?"))
                .child(div().line_height(px(15.0)).text_color(TEXT_MUTED).child("Crow is about to store its first secret. Both stances encrypt it; they differ in who can get at it and what you give up. Pick one on purpose. You can switch to Locked later from the stance badge."))
                .child(div().flex().gap(px(12.0)).child(stance_card(Stance::Open, open_action, Some(tick))).child(stance_card(Stance::Locked, locked_action, None)))
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .child(button("stance-choice-cancel", "CANCEL (DON'T SAVE)", TEXT_SECONDARY, move |cx| a_cancel.update(cx, |this, cx| this.cancel_stance_choice(cx)))),
                ),
        )
}
