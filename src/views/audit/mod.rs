//! Audit log page on Fleet Setup (ERR-75, ERR-156): every change Crow made, on every server.

pub mod model;

use chrono::Local;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use crate::app::CrowApp;
use crate::theme::*;
use model::{AuditFilter, AuditItem, KindGroup, Outcome};

pub fn audit_view(items: &[AuditItem], filter: &AuditFilter, app: Entity<CrowApp>) -> impl IntoElement {
    let now = Local::now();
    // Filter chips list what's in the log, not every possible value.
    let mut servers: Vec<(String, String)> = Vec::new();
    for i in items {
        if !servers.iter().any(|(id, _)| *id == i.server_id) {
            servers.push((i.server_id.clone(), i.server.clone()));
        }
    }
    servers.sort_by(|a, b| a.1.cmp(&b.1));
    let groups: Vec<KindGroup> = KindGroup::ALL.into_iter().filter(|g| items.iter().any(|i| i.group == *g)).collect();
    let shown: Vec<&AuditItem> = items.iter().filter(|i| filter.matches(i)).collect();
    let failed = shown.iter().filter(|i| i.outcome == Outcome::Failed).count();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .font_family(FONT_MONO)
        .child(
            div()
                .h(px(52.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(16.0))
                .gap(px(10.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().text_size(px(16.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child("AUDIT LOG"))
                .child(div().text_size(px(11.5)).text_color(TEXT_DIM).child(format!(
                    "{} change{} · {failed} failed · every server",
                    shown.len(),
                    if shown.len() == 1 { "" } else { "s" }
                )))
        )
        .child(filter_row("SERVER", filter.server_id.is_none(), servers.into_iter().map(|(id, name)| {
            let selected = filter.server_id.as_deref() == Some(id.as_str());
            (name, selected, AuditFilter { server_id: (!selected).then_some(id), group: filter.group })
        }).collect(), AuditFilter { server_id: None, group: filter.group }, app.clone()))
        .child(filter_row("KIND", filter.group.is_none(), groups.into_iter().map(|g| {
            let selected = filter.group == Some(g);
            (g.label().to_string(), selected, AuditFilter { server_id: filter.server_id.clone(), group: (!selected).then_some(g) })
        }).collect(), AuditFilter { server_id: filter.server_id.clone(), group: None }, app.clone()))
        .child(
            div()
                .id("audit-list")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .children(if shown.is_empty() {
                    vec![div().p(px(16.0)).text_size(px(11.0)).text_color(TEXT_MUTED).child("Nothing recorded yet.").into_any_element()]
                } else {
                    shown.into_iter().map(|i| audit_row(i, now)).collect()
                }),
        )
}

fn filter_row(label: &'static str, all_selected: bool, chips: Vec<(String, bool, AuditFilter)>, all: AuditFilter, app: Entity<CrowApp>) -> impl IntoElement {
    let chip = |id: SharedString, text: String, selected: bool, next: AuditFilter, app: Entity<CrowApp>| {
        div()
            .id(id)
            .px(px(8.0))
            .py(px(2.0))
            .border_1()
            .border_color(if selected { BORDER_CONTROL_SEL } else { hex_rgba(0, 0.0) })
            .bg(if selected { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
            .text_color(if selected { TEXT_MAX } else { TEXT_MUTED })
            .cursor_pointer()
            .hover(|s| s.text_color(TEXT_PRIMARY))
            .on_click(move |_ev, _window, cx| {
                let next = next.clone();
                app.update(cx, |this, cx| {
                    this.audit_filter = next;
                    cx.notify();
                })
            })
            .child(text)
    };
    div()
        .flex_none()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(6.0))
        .px(px(16.0))
        .py(px(6.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .text_size(px(10.5))
        .child(div().w(px(56.0)).text_color(TEXT_DIMMER).child(label))
        .child(chip(format!("audit-{label}-all").into(), "all".into(), all_selected, all, app.clone()))
        .children(chips.into_iter().enumerate().map(|(n, (text, selected, next))| chip(format!("audit-{label}-{n}").into(), text, selected, next, app.clone())))
}

fn audit_row(i: &AuditItem, now: chrono::DateTime<Local>) -> AnyElement {
    let (mark, color) = match i.outcome {
        Outcome::Done => ("✓", OK),
        Outcome::Failed => ("✗", CRIT),
        Outcome::Pending => ("…", TEXT_DIM),
    };
    div()
        .flex()
        .items_start()
        .gap(px(12.0))
        .px(px(16.0))
        .py(px(6.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .text_size(px(11.0))
        .child(div().w(px(96.0)).flex_none().text_color(TEXT_FAINT).child(model::when_label(i.at, now)))
        .child(div().w(px(14.0)).flex_none().text_color(color).child(mark))
        .child(div().w(px(150.0)).flex_none().text_color(TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(i.server.clone()))
        .child(div().w(px(64.0)).flex_none().text_color(TEXT_DIM).child(i.group.label()))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(div().text_color(if i.outcome == Outcome::Failed { CRIT } else { TEXT_SECONDARY }).child(i.text.clone()))
                .children(i.detail.clone().map(|d| div().text_size(px(10.0)).text_color(TEXT_FAINT).child(d))),
        )
        .into_any_element()
}
