//! The Fleet screen's Archived tab (ERR-32): servers that left the fleet and
//! were never deleted, how long each one keeps its data, and Crow's own purge
//! audit trail.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use crate::app::archive::{archived_on_text, purge_status_text};
use crate::app::CrowApp;
use crate::components::icons::{tabler_icon, TablerIcon};
use crate::theme::*;
use crate::vault::{ChangeRecord, ServerRecord};
use crate::views::fleet::FleetState;
use crate::views::fleet::state::FleetPage;

/// ACTIVE FLEET / ARCHIVED / DANGER ZONE switch, plus the last action's result.
pub fn fleet_view_tabs(fleet: &FleetState, app: Entity<CrowApp>) -> impl IntoElement {
    let active_n = fleet.servers.len();
    let archived_n = fleet.archive_count();
    let page = fleet.page;
    let set = |to: FleetPage| {
        let app = app.clone();
        move |cx: &mut App| {
            app.update(cx, |this, cx| {
                this.fleet.page = to;
                cx.notify();
            })
        }
    };

    div()
        .h(px(30.0))
        .flex_none()
        .flex()
        .items_center()
        .px(px(14.0))
        .gap(px(6.0))
        .bg(BG_RAIL)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .child(tab_chip("fleet-tab-active", &format!("ACTIVE FLEET ({active_n})"), page == FleetPage::Active, set(FleetPage::Active)))
        .child(tab_chip("fleet-tab-archived", &format!("ARCHIVED ({archived_n})"), page == FleetPage::Archived, set(FleetPage::Archived)))
        .child(danger_chip(page == FleetPage::Danger, set(FleetPage::Danger)))
        .child(div().flex_1())
        .children(fleet.notice.as_ref().map(|notice| {
            div()
                .text_size(px(10.0))
                .text_color(TEXT_DIMMER)
                .child(notice.clone())
        }))
}

fn tab_chip(id: &'static str, label: &str, is_on: bool, on_click: impl Fn(&mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .px(px(10.0))
        .py(px(3.0))
        .border_1()
        .border_color(if is_on { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
        .bg(if is_on { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
        .text_color(if is_on { TEXT_MAX } else { TEXT_DIMMER })
        .font_weight(if is_on { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_size(px(10.0))
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _window, cx| on_click(cx))
        .child(label.to_string())
}

/// The fleet-wide Danger Zone's tab: red whether on or not.
fn danger_chip(is_on: bool, on_click: impl Fn(&mut App) + 'static) -> impl IntoElement {
    div()
        .id("fleet-tab-danger")
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(3.0))
        .border_1()
        .border_color(if is_on { CRIT } else { BORDER_DANGER_BTN })
        .bg(if is_on { CRIT_BG } else { hex_rgba(0, 0.0) })
        .text_color(if is_on { CRIT } else { CRIT_INK_DIM })
        .font_weight(if is_on { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_size(px(10.0))
        .cursor_pointer()
        .hover(|s| s.bg(CRIT_ROW_BG).text_color(CRIT))
        .on_click(move |_ev, _window, cx| on_click(cx))
        .child(tabler_icon(TablerIcon::AlertTriangle).size(px(11.0)).text_color(if is_on { CRIT } else { CRIT_INK_DIM }))
        .child("DANGER ZONE")
}

/// The archived servers, their purge countdown, and recent purges.
pub fn archived_panel(fleet: &FleetState, purge_days: Option<i64>, audit: &[ChangeRecord], app: Entity<CrowApp>) -> impl IntoElement {
    div()
        .flex_1()
        .min_h(px(0.0))
        .flex()
        .flex_col()
        .child(archived_header())
        .child(
            div()
                .id("fleet-archived-list")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .children(if fleet.archived.is_empty() {
                    vec![empty_archived_state()]
                } else {
                    fleet
                        .archived
                        .iter()
                        .enumerate()
                        .map(|(idx, srv)| archived_row(srv, idx, purge_days, app.clone()).into_any_element())
                        .collect()
                }),
        )
        .child(purge_log(audit))
}

fn archived_header() -> impl IntoElement {
    div()
        .h(px(26.0))
        .flex_none()
        .flex()
        .items_center()
        .px(px(12.0))
        .bg(BG_SUBHEAD)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(TEXT_DIMMER)
        .child(div().w(px(3.0)).flex_none().child(""))
        .child(div().flex_grow(3.0).flex_basis(px(0.0)).min_w(px(140.0)).child("HOST"))
        .child(div().flex_grow(2.0).flex_basis(px(0.0)).min_w(px(110.0)).child("ADDRESS"))
        .child(div().w(px(64.0)).flex_none().child("ENV"))
        .child(div().flex_grow(2.0).flex_basis(px(0.0)).min_w(px(90.0)).child("ROLE"))
        .child(div().w(px(96.0)).flex_none().child("ARCHIVED"))
        .child(div().flex_grow(2.0).flex_basis(px(0.0)).min_w(px(150.0)).child("STORED DATA"))
        .child(div().w(px(90.0)).flex_none().text_align(TextAlign::Right).child(""))
}

fn archived_row(srv: &ServerRecord, idx: usize, purge_days: Option<i64>, app: Entity<CrowApp>) -> impl IntoElement {
    let is_even = idx % 2 == 0;
    let id = srv.id.clone();
    let app_restore = app.clone();
    let purged = srv.purged_at.is_some();
    let (env_bg, env_fg) = match srv.env.as_str() {
        "PROD" => (CRIT_BG, CRIT),
        "STAGE" => (WARN_BG, WARN),
        "DEV" => (OK_BG, OK),
        _ => (BG_PANEL, TEXT_DIM),
    };

    div()
        .id(ElementId::NamedInteger("archived-row".into(), idx as u64))
        .relative()
        .h(px(32.0))
        .flex_none()
        .flex()
        .items_center()
        .px(px(12.0))
        .bg(if is_even { BG_APP } else { BG_ROW_ALT })
        .border_b_1()
        .border_color(BORDER_ROW)
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .text_size(px(11.5))
        .child(left_indicator(if purged { TEXT_FAINTER } else { WARN }))
        .child(
            div()
                .flex_grow(3.0)
                .flex_basis(px(0.0))
                .min_w(px(140.0))
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(srv.name.clone()))
                .child(
                    div()
                        .px(px(4.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .bg(BG_CHIP)
                        .text_color(TEXT_DIM)
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .child("ARCHIVED"),
                ),
        )
        .child(
            div()
                .flex_grow(2.0)
                .flex_basis(px(0.0))
                .min_w(px(110.0))
                .overflow_hidden()
                .text_color(TEXT_DIM)
                .child(format!("{}:{}", srv.host, srv.port)),
        )
        .child(
            div()
                .w(px(64.0))
                .flex_none()
                .child(
                    div()
                        .px(px(4.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .bg(env_bg)
                        .text_color(env_fg)
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .child(srv.env.clone()),
                ),
        )
        .child(
            div()
                .flex_grow(2.0)
                .flex_basis(px(0.0))
                .min_w(px(90.0))
                .overflow_hidden()
                .text_size(px(11.0))
                .text_color(TEXT_TERTIARY)
                .child(srv.role.clone()),
        )
        .child(div().w(px(96.0)).flex_none().text_color(TEXT_DIM).text_size(px(10.5)).child(archived_on_text(srv)))
        .child(
            div()
                .flex_grow(2.0)
                .flex_basis(px(0.0))
                .min_w(px(150.0))
                .overflow_hidden()
                .text_size(px(10.5))
                .text_color(if purged { TEXT_FAINTER } else { WARN })
                .child(purge_status_text(srv, purge_days)),
        )
        .child(
            div()
                .w(px(90.0))
                .flex_none()
                .flex()
                .justify_end()
                .child(
                    div()
                        .id(SharedString::from(format!("archived-restore-{id}")))
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .px(px(8.0))
                        .py(px(2.0))
                        .border_1()
                        .border_color(OK)
                        .bg(OK_BG)
                        .text_color(OK)
                        .font_family(FONT_MONO)
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            let id = id.clone();
                            cx.stop_propagation();
                            app_restore.update(cx, |this, cx| this.restore_archived_server(&id, cx));
                        })
                        .child(tabler_icon(TablerIcon::Refresh).size(px(10.0)).text_color(OK))
                        .child("RESTORE"),
                ),
        )
}

fn empty_archived_state() -> AnyElement {
    div()
        .id("fleet-archived-empty")
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .p(px(32.0))
        .child(
            div()
                .size(px(44.0))
                .border_1()
                .border_color(BORDER_STRONG)
                .bg(BG_PANEL)
                .flex()
                .items_center()
                .justify_center()
                .child(tabler_icon(TablerIcon::Server).size(px(22.0)).text_color(TEXT_MUTED)),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .font_weight(FontWeight::BOLD)
                .text_size(px(13.0))
                .text_color(TEXT_MAX)
                .child("NO ARCHIVED SERVERS"),
        )
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(TEXT_DIM)
                .child("Archiving takes a server out of the fleet without deleting it. Nothing is ever deleted outright."),
        )
        .into_any_element()
}

/// Crow's own purge records — the audit trail the Archived tab exposes.
fn purge_log(audit: &[ChangeRecord]) -> impl IntoElement {
    div()
        .h(px(132.0))
        .flex_none()
        .flex()
        .flex_col()
        .bg(BG_RAIL)
        .border_t_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .h(px(28.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .child(div().font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("PURGE LOG"))
                .child(div().text_color(TEXT_DIMMER).child(format!("{} record(s) · Crow's own audit trail", audit.len()))),
        )
        .child(
            div()
                .id("fleet-purge-log")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .py(px(4.0))
                .children(if audit.is_empty() {
                    vec![div()
                        .px(px(12.0))
                        .py(px(8.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .text_color(TEXT_DIMMER)
                        .child("Nothing has been purged yet.")
                        .into_any_element()]
                } else {
                    audit
                        .iter()
                        .map(|rec| {
                            let when = rec.completed_at.clone().unwrap_or_else(|| rec.started_at.clone());
                            div()
                                .flex()
                                .gap(px(8.0))
                                .px(px(12.0))
                                .py(px(2.0))
                                .font_family(FONT_MONO)
                                .text_size(px(10.5))
                                .child(div().w(px(150.0)).flex_none().text_color(TEXT_FAINTER).child(when))
                                .child(div().w(px(120.0)).flex_none().text_color(TEXT_SECONDARY).child(rec.target.clone()))
                                .child(div().flex_1().text_color(TEXT_DIMMER).child(rec.before_state.clone()))
                                .into_any_element()
                        })
                        .collect()
                }),
        )
}

/// Confirmation before a server leaves the fleet. Says what happens, and when.
pub fn archive_confirm_overlay(srv: &ServerRecord, purge_due: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let app_cancel = app.clone();
    let app_confirm = app.clone();
    let id = srv.id.clone();

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(hex_rgba(0x000000, 0.6))
        // The backdrop swallows clicks so the table underneath stays inert.
        .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
        .child(
            div()
                .w(px(600.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .rounded_sm()
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(34.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(14.0))
                        .bg(BG_PANEL)
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_MAX)
                        .child(tabler_icon(TablerIcon::AlertTriangle).size(px(13.0)).text_color(WARN))
                        .child(format!("ARCHIVE {}", srv.name.to_uppercase())),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .p(px(14.0))
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_SECONDARY)
                        .child(format!("{} leaves the active fleet. Crow stops polling it and closes its SSH connection.", srv.name))
                        .child(
                            div()
                                .text_color(WARN)
                                .child(format!("Its stored data — change history and key attachments — is deleted on {purge_due}.")),
                        )
                        .child(
                            div()
                                .text_color(TEXT_DIM)
                                .child("The server record itself is never deleted: address, port, user, key, jump host, environment, role, group, tags, host-key fingerprint and OS facts all stay. It can be restored from the Archived tab until the purge runs, and after that it returns without its history."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(8.0))
                        .px(px(14.0))
                        .pb(px(14.0))
                        .font_family(FONT_MONO)
                        .text_size(px(10.5))
                        .child(
                            div()
                                .id("archive-cancel")
                                .px(px(12.0))
                                .py(px(6.0))
                                .border_1()
                                .border_color(BORDER_DEFAULT)
                                .text_color(TEXT_SECONDARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    cx.stop_propagation();
                                    app_cancel.update(cx, |this, cx| {
                                        this.fleet.pending_archive = None;
                                        cx.notify();
                                    });
                                })
                                .child("CANCEL"),
                        )
                        .child(
                            div()
                                .id("archive-confirm")
                                .px(px(12.0))
                                .py(px(6.0))
                                .border_1()
                                .border_color(WARN)
                                .bg(WARN_BG)
                                .text_color(WARN)
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let id = id.clone();
                                    cx.stop_propagation();
                                    app_confirm.update(cx, |this, cx| this.archive_server(&id, cx));
                                })
                                .child("ARCHIVE SERVER"),
                        ),
                ),
        )
}
