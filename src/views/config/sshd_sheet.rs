//! sshd_config as a settings sheet: the security posture at a glance, then
//! every directive the sshd crow-config plugin knows, grouped by what it
//! does, showing the value sshd actually uses (set in the file, or OpenSSH's
//! default). Choices are clickable options with their risk; unknown
//! directives and Match blocks follow.

use gpui_kit::component::input::Input;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crow_config_core::schema::{FieldType, RiskLevel};

use super::structured_editor::{risk_color, risk_label, small_button, ActiveFieldEdit};
use crate::app::configs::NEW_DIRECTIVE_PREFIX;
use crate::app::CrowApp;
use crate::config::plugins::{SheetRow, SshdSheet};
use crate::theme::*;

const NAME_WIDTH: f32 = 250.0;
const META_WIDTH: f32 = 190.0;

fn risk_of(row: &SheetRow, value: &str) -> Option<RiskLevel> {
    row.def.as_ref()?.options.as_ref()?.iter().find(|o| o.value.eq_ignore_ascii_case(value)).and_then(|o| o.risk.clone())
}

pub fn sshd_sheet_view(file: &str, sheet: &SshdSheet, read_only: bool, active_edit: Option<&ActiveFieldEdit>, app: Entity<CrowApp>) -> impl IntoElement {
    let posture: Vec<&SheetRow> = sheet
        .sections
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter(|r| r.def.as_ref().is_some_and(|d| d.options.is_some() || d.name == "Port"))
        .collect();

    let app_pw = app.clone();
    let include_notes = sheet.include_notes.clone();
    div()
        .flex()
        .flex_col()
        .pb(px(16.0))
        .children((!include_notes.is_empty()).then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .px(px(14.0))
                .py(px(8.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(WARN)
                .child("Some files this config includes couldn't be read, so values below may not be what sshd uses:")
                .children(include_notes.into_iter().map(|n| div().text_color(TEXT_SECONDARY).child(n)))
        }))
        // Password login off, the safe way (ERR-34).
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .py(px(8.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_DIMMER).child("Turning PasswordAuthentication off by hand can lock you out, and a cloud image's drop-in may override it. Crow can do it with checks and automatic undo."))
                .child(
                    div()
                        .id("sshd-password-login-off")
                        .flex_none()
                        .px(px(9.0))
                        .py(px(4.0))
                        .border_1()
                        .border_color(WARN.opacity(0.6))
                        .text_color(WARN)
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| app_pw.update(cx, |this, cx| this.open_password_login(cx)))
                        .child("TURN OFF PASSWORD LOGIN SAFELY…"),
                ),
        )
        // Posture at a glance.
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(8.0))
                .px(px(14.0))
                .py(px(12.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .children(posture.into_iter().map(posture_tile)),
        )
        .children(sheet.sections.iter().map(|(group, rows)| {
            section(group.to_uppercase()).children(rows.iter().enumerate().map(|(i, row)| {
                directive_row(file, row, &format!("{group}-{i}"), read_only, active_edit, app.clone())
            }))
        }))
        .children((!sheet.other.is_empty()).then(|| {
            section("OTHER DIRECTIVES".into()).children(sheet.other.iter().enumerate().map(|(i, row)| directive_row(file, row, &format!("other-{i}"), read_only, active_edit, app.clone())))
        }))
        .children(sheet.scoped.iter().enumerate().map(|(i, (scope, source, rows))| {
            let app = app.clone();
            let title = match source {
                Some(path) => format!("MATCH {scope} · IN {}", path.to_uppercase()),
                None => format!("MATCH {scope} · APPLIES ONLY WHEN THIS MATCHES"),
            };
            section(title).children(rows.iter().enumerate().map(move |(j, row)| directive_row(file, row, &format!("match-{i}-{j}"), read_only, active_edit, app.clone())))
        }))
}

fn section(title: String) -> Div {
    div().flex().flex_col().child(
        div()
            .px(px(14.0))
            .pt(px(16.0))
            .pb(px(6.0))
            .font_family(FONT_MONO)
            .text_size(px(9.5))
            .font_weight(FontWeight::BOLD)
            .text_color(TEXT_FAINT)
            .child(title),
    )
}

fn posture_tile(row: &SheetRow) -> impl IntoElement {
    let value = row.effective().unwrap_or("—").to_string();
    let risk = risk_of(row, &value);
    let color = if row.def.as_ref().is_some_and(|d| d.options.is_some()) { risk_color(risk.as_ref()) } else { TEXT_PRIMARY };
    div()
        .min_w(px(150.0))
        .px(px(10.0))
        .py(px(7.0))
        .bg(BG_APP)
        .border_1()
        .border_color(if matches!(risk, Some(RiskLevel::NeverOnProd | RiskLevel::Deny)) { CRIT } else { BORDER_PANEL })
        .rounded_sm()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .font_family(FONT_MONO)
        .child(div().text_size(px(9.0)).text_color(TEXT_FAINT).child(row.name.clone()))
        .child(
            div()
                .flex()
                .items_baseline()
                .gap(px(6.0))
                .child(div().text_size(px(12.5)).font_weight(FontWeight::BOLD).text_color(color).child(value))
                .children(row.value.is_none().then(|| div().text_size(px(8.5)).text_color(TEXT_FAINTER).child("default"))),
        )
}

fn directive_row(file: &str, row: &SheetRow, key: &str, read_only: bool, active_edit: Option<&ActiveFieldEdit>, app: Entity<CrowApp>) -> impl IntoElement {
    // Rows from an included drop-in are shown where they apply, not edited here.
    let read_only = read_only || row.source.is_some();
    let is_set = row.value.is_some();
    let effective = row.effective().map(str::to_string);
    let help = row.def.as_ref().and_then(|d| d.help.clone());
    let editing = active_edit.filter(|e| match &row.row_id {
        Some(id) => e.row_id == id,
        None => e.row_id.strip_prefix(NEW_DIRECTIVE_PREFIX).is_some_and(|n| n.eq_ignore_ascii_case(&row.name)),
    });

    // Name, with what it does underneath.
    let name_col = div()
        .w(px(NAME_WIDTH))
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .font_family(FONT_MONO)
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(if is_set { FontWeight::BOLD } else { FontWeight::NORMAL })
                .text_color(if row.shadowed { TEXT_FAINT } else if is_set { TEXT_PRIMARY } else { TEXT_DIM })
                .child(row.name.clone()),
        )
        .children(help.map(|h| div().text_size(px(9.5)).text_color(TEXT_FAINT).child(h)));

    // The value: options as clickable choices, anything else as text.
    let options = row.def.as_ref().and_then(|d| d.options.clone());
    let value_col: AnyElement = if let Some(input) = editing.map(|e| e.input) {
        div().w(px(320.0)).child(Input::new(input).font_family(FONT_MONO).text_size(px(11.5)).bg(BG_APP).rounded(px(2.0))).into_any_element()
    } else if let Some(options) = options.filter(|_| !row.shadowed) {
        div()
            .flex()
            .flex_wrap()
            .gap(px(4.0))
            .children(options.into_iter().enumerate().map(|(i, opt)| {
                let selected = effective.as_deref().is_some_and(|v| v.eq_ignore_ascii_case(&opt.value));
                let color = risk_color(opt.risk.as_ref());
                let (app, file, row_id, field, name, value) = (app.clone(), file.to_string(), row.row_id.clone(), row.field_name.clone(), row.name.clone(), opt.value.clone());
                div()
                    .id(SharedString::from(format!("sshd-opt-{key}-{i}")))
                    .px(px(8.0))
                    .py(px(3.0))
                    .rounded_sm()
                    .border_1()
                    .font_family(FONT_MONO)
                    .text_size(px(10.5))
                    .border_color(if selected { color } else { BORDER_DEFAULT })
                    .bg(if selected { color.opacity(0.12) } else { hex_rgba(0, 0.0) })
                    .text_color(if selected { color } else { TEXT_DIMMER })
                    .font_weight(if selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                    .when(!read_only && !selected, |d| {
                        d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_SECONDARY)).on_click(move |_ev, _window, cx| {
                            let (file, value) = (file.clone(), value.clone());
                            app.update(cx, |this, cx| match (&row_id, &field) {
                                (Some(r), Some(f)) => this.set_structured_value(&file, r, f, serde_json::Value::String(value), cx),
                                _ => this.insert_structured_row(&file, Some((name.clone(), value)), cx),
                            });
                        })
                    })
                    .child(opt.value.clone())
            }))
            .into_any_element()
    } else {
        let text = match (&row.value, &effective) {
            (Some(v), _) => v.clone(),
            (None, Some(d)) => d.clone(),
            (None, None) => "not set".into(),
        };
        let is_list = row.def.as_ref().is_some_and(|d| d.field_type == FieldType::StringList);
        let (app, file, row_id, field, name, current) = (app.clone(), file.to_string(), row.row_id.clone(), row.field_name.clone(), row.name.clone(), row.value.clone().unwrap_or_default());
        div()
            .id(SharedString::from(format!("sshd-val-{key}")))
            .max_w(px(420.0))
            .px(px(8.0))
            .py(px(3.0))
            .rounded_sm()
            .border_1()
            .border_color(if is_set { BORDER_DEFAULT } else { hex_rgba(0, 0.0) })
            .font_family(FONT_MONO)
            .text_size(px(11.0))
            .text_color(if is_set { TEXT_PRIMARY } else { TEXT_DIMMER })
            .when(!read_only, |d| {
                d.cursor_pointer().hover(|s| s.bg(BG_CONTROL)).on_click(move |_ev, window, cx| {
                    let file = file.clone();
                    app.update(cx, |this, cx| match (&row_id, &field) {
                        (Some(r), Some(f)) => this.begin_structured_field_edit(&file, r, f, &current, is_list, window, cx),
                        _ => this.begin_new_directive(&file, &name, window, cx),
                    });
                })
            })
            .child(text)
            .into_any_element()
    };

    // Risk of the value in effect, where it came from, and reset.
    let risk = effective.as_deref().and_then(|v| risk_of(row, v));
    let meta = div()
        .w(px(META_WIDTH))
        .flex_none()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(8.0))
        .font_family(FONT_MONO)
        .text_size(px(9.0))
        .children(risk_label(risk.as_ref()).filter(|_| risk != Some(RiskLevel::Recommended)).map(|l| div().font_weight(FontWeight::BOLD).text_color(risk_color(risk.as_ref())).child(l)))
        .child(div().text_color(TEXT_FAINTER).child({
            let at = match (&row.source, row.line) {
                (Some(src), Some(l)) => format!("{}:{l}", src.rsplit('/').next().unwrap_or(src)),
                (None, Some(l)) => format!("L{l}"),
                _ => String::new(),
            };
            match (row.shadowed, &row.shadowed_by, row.line, is_set) {
                (true, Some(by), _, _) => format!("{at} · ignored: {} sets it first", by.rsplit('/').next().unwrap_or(by)),
                (true, None, _, _) => format!("{at} · ignored, set earlier"),
                (_, _, Some(_), _) => at,
                (_, _, None, false) if effective.is_some() => "default".into(),
                _ => String::new(),
            }
        }))
        .children(row.row_id.clone().filter(|_| !read_only).map(|row_id| {
            let (app, file) = (app.clone(), file.to_string());
            let label = if row.def.as_ref().is_some_and(|d| d.default.is_some()) && !row.shadowed { "↺" } else { "✕" };
            small_button(SharedString::from(format!("sshd-reset-{key}")), label, TEXT_DIMMER).on_click(move |_ev, _window, cx| {
                app.update(cx, |this, cx| this.delete_structured_row(&file, &row_id, cx));
            })
        }));

    div()
        .flex()
        .items_center()
        .gap(px(16.0))
        .px(px(14.0))
        .py(px(7.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(name_col)
        .child(div().flex_1().min_w(px(0.0)).child(value_col))
        .child(meta)
}
