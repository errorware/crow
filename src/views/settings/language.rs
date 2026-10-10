//! Settings → Personalisation → Language (ERR-171).

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::language::{LanguageState, PackEntry};
use crate::app::CrowApp;
use crate::theme::*;

pub fn language_block(state: &LanguageState, current: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let button = |id: &'static str, label: &'static str| {
        div()
            .id(id)
            .px(px(10.0))
            .py(px(5.0))
            .border_1()
            .border_color(TEXT_SECONDARY)
            .text_color(TEXT_SECONDARY)
            .text_size(px(10.5))
            .font_weight(FontWeight::BOLD)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
            .child(label)
    };
    let (app_open, app_export, app_reload) = (app.clone(), app.clone(), app.clone());
    let selected = state.entries.iter().find(|e| e.info.code == current);
    div()
        .mx(px(14.0))
        .mb(px(14.0))
        .p(px(14.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_PANEL)
        .flex()
        .flex_col()
        .gap(px(8.0))
        .font_family(FONT_MONO)
        .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(t!("settings-language-title")))
        .child(div().text_size(px(10.0)).text_color(TEXT_DIM).child(t!("settings-language-about", folder = crate::i18n::languages_dir().display())))
        .child(
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(BORDER_DEFAULT)
                .children(state.entries.iter().enumerate().map(|(i, e)| entry_row(i, e, e.info.code == current, app.clone()))),
        )
        .children(selected.and_then(|e| e.report.as_ref().ok()).filter(|r| !r.refused.is_empty() || r.syntax_errors > 0).map(problems))
        .children(state.message.clone().map(|(ok, m)| div().text_size(px(10.0)).text_color(if ok { OK } else { CRIT }).child(m)))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(8.0))
                .pt(px(4.0))
                .child(button("btn-open-languages", t!("settings-language-open-folder")).on_click(move |_ev, _window, cx| app_open.update(cx, |this, cx| this.open_languages_folder(cx))))
                .child(button("btn-export-language-template", t!("settings-language-export-template")).on_click(move |_ev, _window, cx| app_export.update(cx, |this, cx| this.export_language_template(cx))))
                .child(button("btn-reload-languages", t!("settings-language-reload")).on_click(move |_ev, _window, cx| app_reload.update(cx, |this, cx| this.list_language_packs(cx)))),
        )
}

fn entry_row(i: usize, e: &PackEntry, selected: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let code = e.info.code.clone();
    let usable = e.report.is_ok();
    let detail = match &e.report {
        Ok(r) if r.missing == 0 => t!("settings-language-complete").to_string(),
        Ok(r) => t!("settings-language-coverage", percent = r.percent(), missing = r.missing),
        Err(why) => why.clone(),
    };
    div()
        .id(("language-row", i))
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(10.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .when(selected, |d| d.bg(BG_ROW_SELECTED))
        .when(usable && !selected, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)))
        .when(usable, |d| d.on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.select_language(&code, cx))))
        .child(
            div()
                .size(px(9.0))
                .flex_none()
                .border_1()
                .border_color(if selected { OK } else { BORDER_STRONG })
                .when(selected, |d| d.bg(OK)),
        )
        .child(div().flex_none().text_size(px(11.0)).text_color(if usable { TEXT_PRIMARY } else { TEXT_FAINT }).child(e.info.display_name()))
        .child(div().flex_none().text_size(px(9.5)).text_color(TEXT_FAINT).child(e.info.code.clone()))
        .child(div().flex_1().min_w(px(0.0)).truncate().text_size(px(9.5)).text_color(if usable { TEXT_DIM } else { CRIT }).child(detail))
        .children((!e.info.author.is_empty()).then(|| div().flex_none().max_w(px(180.0)).truncate().text_size(px(9.5)).text_color(TEXT_FAINT).child(t!("settings-language-by", author = e.info.author))))
}

/// What the selected pack got wrong: those messages show in English.
fn problems(r: &crate::i18n::PackReport) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(3.0))
        .p(px(8.0))
        .bg(WARN_BG)
        .text_size(px(10.0))
        .child(div().text_color(WARN_INK).child(t!("settings-language-refused", count = r.refused.len(), errors = r.syntax_errors)))
        .children(r.refused.iter().take(6).map(|(id, why)| div().text_color(TEXT_DIM).child(format!("{id} · {why}"))))
        .children((r.refused.len() > 6).then(|| div().text_color(TEXT_FAINT).child(t!("settings-language-more", count = r.refused.len() - 6))))
}
