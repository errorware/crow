//! sshd_config as a settings sheet: the security posture at a glance, then
//! every directive the sshd crow-config plugin knows, grouped by what it
//! does, showing the value sshd actually uses (set in the file, or OpenSSH's
//! default). Choices are clickable options with their risk; unknown
//! directives and Match blocks follow.

use gpui_kit::component::input::Input;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crow_config_core::schema::{FieldType, RiskLevel};

use super::structured_editor::{risk_color, risk_label, ActiveFieldEdit};
use crate::app::configs::NEW_DIRECTIVE_PREFIX;
use crate::app::CrowApp;
use crate::config::plugins::{SheetRow, SshdSheet};
use crate::theme::*;

fn risk_of(row: &SheetRow, value: &str) -> Option<RiskLevel> {
    option_of(row, value).and_then(|o| o.risk.clone())
}

fn option_of<'a>(row: &'a SheetRow, value: &str) -> Option<&'a crow_config_core::schema::EnumOption> {
    row.def.as_ref()?.options.as_ref()?.iter().find(|o| o.value.eq_ignore_ascii_case(value))
}

fn is_risky(risk: Option<&RiskLevel>) -> bool {
    matches!(risk, Some(RiskLevel::Caution | RiskLevel::Weak | RiskLevel::NeverOnProd | RiskLevel::Deny))
}

/// How the posture panel puts a risky directive right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostureFix {
    /// Set it to this value.
    Set(String),
    /// PasswordAuthentication: the guarded flow (ERR-34), never a bare edit.
    PasswordFlow,
}

/// One directive the posture panel speaks for.
pub struct PostureItem<'a> {
    pub row: &'a SheetRow,
    pub value: String,
    pub risk: Option<RiskLevel>,
    /// None when it's fine, or set in a file this editor doesn't write.
    pub fix: Option<PostureFix>,
}

/// The directives with a risk rating, global ones only (Match blocks are
/// listed with their rows), risky first. A fix never locks Crow out: with
/// Crow logging in as root, root login goes to prohibit-password, not no.
pub fn posture<'a>(sheet: &'a SshdSheet, login_user: &str) -> (Vec<PostureItem<'a>>, Vec<PostureItem<'a>>) {
    let (mut risky, mut fine) = (Vec::new(), Vec::new());
    for row in sheet.sections.iter().flat_map(|(_, rows)| rows).filter(|r| !r.shadowed) {
        let Some(options) = row.def.as_ref().and_then(|d| d.options.as_ref()) else { continue };
        let Some(value) = row.effective().map(str::to_string) else { continue };
        let risk = risk_of(row, &value);
        if !is_risky(risk.as_ref()) {
            fine.push(PostureItem { row, value, risk, fix: None });
            continue;
        }
        let fix = if row.source.is_some() {
            None
        } else if row.name.eq_ignore_ascii_case("PasswordAuthentication") {
            Some(PostureFix::PasswordFlow)
        } else if row.name.eq_ignore_ascii_case("PermitRootLogin") {
            Some(PostureFix::Set(if login_user == "root" { "prohibit-password" } else { "no" }.into()))
        } else {
            options.iter().find(|o| o.risk == Some(RiskLevel::Recommended)).map(|o| PostureFix::Set(o.value.clone()))
        };
        risky.push(PostureItem { row, value, risk, fix });
    }
    risky.sort_by_key(|i| !matches!(i.risk, Some(RiskLevel::NeverOnProd | RiskLevel::Deny)));
    (risky, fine)
}

pub fn sshd_sheet_view(file: &str, sheet: &SshdSheet, read_only: bool, login_user: &str, active_edit: Option<&ActiveFieldEdit>, app: Entity<CrowApp>) -> impl IntoElement {
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
        .child(posture_panel(file, sheet, read_only, login_user, app.clone()))
        .children(sheet.sections.iter().map(|(group, rows)| {
            section(group.to_uppercase(), rows.len()).children(rows.iter().enumerate().map(|(i, row)| {
                directive_row(file, row, &format!("{group}-{i}"), read_only, active_edit, app.clone())
            }))
        }))
        .children((!sheet.other.is_empty()).then(|| {
            section("OTHER DIRECTIVES".into(), sheet.other.len()).children(sheet.other.iter().enumerate().map(|(i, row)| directive_row(file, row, &format!("other-{i}"), read_only, active_edit, app.clone())))
        }))
        .children(sheet.scoped.iter().enumerate().map(|(i, (scope, source, rows))| {
            let app = app.clone();
            let title = match source {
                Some(path) => format!("MATCH {scope} · IN {}", path.to_uppercase()),
                None => format!("MATCH {scope} · APPLIES ONLY WHEN THIS MATCHES"),
            };
            section(title, rows.len()).children(rows.iter().enumerate().map(move |(j, row)| directive_row(file, row, &format!("match-{i}-{j}"), read_only, active_edit, app.clone())))
        }))
}

/// What needs attention, each with its fix, and one line for what's fine.
fn posture_panel(file: &str, sheet: &SshdSheet, read_only: bool, login_user: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let (risky, fine) = posture(sheet, login_user);
    let summary = match risky.len() {
        0 => "nothing to fix".to_string(),
        1 => "1 to fix".to_string(),
        n => format!("{n} to fix"),
    };
    div()
        .flex()
        .flex_col()
        .mx(px(14.0))
        .mt(px(12.0))
        .border_1()
        .border_color(if risky.is_empty() { BORDER_PANEL } else { BORDER_DEFAULT })
        .bg(BG_PANEL)
        .font_family(FONT_MONO)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(10.0))
                .py(px(6.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().text_size(px(9.5)).font_weight(FontWeight::BOLD).text_color(TEXT_FAINT).child("SECURITY POSTURE"))
                .child(div().text_size(px(9.5)).text_color(if risky.is_empty() { OK } else { WARN }).child(summary))
                .child(div().flex_1())
                .child(div().text_size(px(9.5)).text_color(TEXT_FAINTER).child(format!("{} fine", fine.len()))),
        )
        .children(risky.into_iter().enumerate().map(|(i, item)| {
            let color = risk_color(item.risk.as_ref());
            let meaning = option_of(item.row, &item.value).map(|o| o.label.clone());
            let where_set = match (&item.row.source, item.row.value.is_some()) {
                (Some(src), _) => Some(format!("set in {}", src.rsplit('/').next().unwrap_or(src))),
                (None, false) => Some("sshd's default".to_string()),
                _ => None,
            };
            div()
                .relative()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_x(px(10.0))
                .gap_y(px(4.0))
                .pl(px(12.0))
                .pr(px(10.0))
                .py(px(7.0))
                .border_b_1()
                .border_color(BORDER_ROW)
                .child(div().absolute().left_0().top_0().bottom_0().w(px(2.0)).bg(color))
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(8.0))
                        .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(item.row.name.clone()))
                        .child(div().text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(color).child(item.value.clone()))
                        .children(risk_label(item.risk.as_ref()).map(|l| div().text_size(px(9.0)).font_weight(FontWeight::BOLD).text_color(color).child(l))),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .text_size(px(10.0))
                        .text_color(TEXT_DIM)
                        .child([meaning, where_set].into_iter().flatten().collect::<Vec<_>>().join(" · ")),
                )
                .children(item.fix.filter(|_| !read_only).map(|fix| {
                    let (app, file, row_id, field, name) = (app.clone(), file.to_string(), item.row.row_id.clone(), item.row.field_name.clone(), item.row.name.clone());
                    let label = match &fix {
                        PostureFix::Set(v) => format!("SET {v}"),
                        PostureFix::PasswordFlow => "TURN OFF SAFELY…".into(),
                    };
                    div()
                        .id(SharedString::from(format!("sshd-posture-fix-{i}")))
                        .flex_none()
                        .px(px(8.0))
                        .py(px(3.0))
                        .border_1()
                        .border_color(OK.opacity(0.6))
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(OK)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app.update(cx, |this, cx| match &fix {
                                PostureFix::PasswordFlow => this.open_password_login(cx),
                                PostureFix::Set(value) => match (&row_id, &field) {
                                    (Some(r), Some(f)) => this.set_structured_value(&file, r, f, serde_json::Value::String(value.clone()), cx),
                                    _ => this.insert_structured_row(&file, Some((name.clone(), value.clone())), cx),
                                },
                            })
                        })
                        .child(label)
                }))
        }))
        .children((!fine.is_empty()).then(|| {
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_x(px(14.0))
                .gap_y(px(4.0))
                .px(px(10.0))
                .py(px(7.0))
                .text_size(px(10.0))
                .child(div().text_color(OK).child("✓"))
                .children(fine.into_iter().map(|item| {
                    div()
                        .flex()
                        .gap(px(5.0))
                        .child(div().text_color(TEXT_DIM).child(item.row.name.clone()))
                        .child(div().text_color(TEXT_SECONDARY).child(item.value))
                }))
        }))
}

fn section(title: String, count: usize) -> Div {
    div().flex().flex_col().child(
        div()
            .flex()
            .items_baseline()
            .gap(px(8.0))
            .mx(px(14.0))
            .pt(px(20.0))
            .pb(px(6.0))
            .border_b_1()
            .border_color(BORDER_PANEL)
            .font_family(FONT_MONO)
            .child(div().text_size(px(9.5)).font_weight(FontWeight::BOLD).text_color(TEXT_TERTIARY).child(title))
            .child(div().text_size(px(9.5)).text_color(TEXT_FAINTER).child(count.to_string())),
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

    // The value: options as clickable choices, anything else as text.
    let options = row.def.as_ref().and_then(|d| d.options.clone());
    let value_col: AnyElement = if let Some(input) = editing.map(|e| e.input) {
        div().w_full().max_w(px(420.0)).child(Input::new(input).font_family(FONT_MONO).text_size(px(11.5)).bg(BG_APP).rounded(px(2.0))).into_any_element()
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
                    .tooltip({
                        let label: SharedString = opt.label.clone().into();
                        move |window, cx| gpui_kit::component::tooltip::Tooltip::new(label.clone()).build(window, cx)
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
            .border_color(if is_set { BORDER_DEFAULT } else { BORDER_PANEL })
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

    // Risk of the value in effect, what it means, where it came from, reset.
    let risk = effective.as_deref().and_then(|v| risk_of(row, v));
    let meaning = effective.as_deref().and_then(|v| option_of(row, v)).map(|o| o.label.clone());
    let at = match (&row.source, row.line) {
        (Some(src), Some(l)) => format!("{}:{l}", src.rsplit('/').next().unwrap_or(src)),
        (None, Some(l)) => format!("line {l}"),
        _ => String::new(),
    };
    let location = match (row.shadowed, &row.shadowed_by) {
        (true, Some(by)) => format!("{at} · ignored: {} sets it first", by.rsplit('/').next().unwrap_or(by)),
        (true, None) => format!("{at} · ignored, set earlier"),
        _ => at,
    };
    let reset = row.row_id.clone().filter(|_| !read_only).map(|row_id| {
        let (app, file) = (app.clone(), file.to_string());
        // Removing a line sshd has a default for resets it; otherwise it deletes.
        let resets = row.def.as_ref().is_some_and(|d| d.default.is_some()) && !row.shadowed;
        let icon = if resets { crate::components::icons::TablerIcon::Refresh } else { crate::components::icons::TablerIcon::Trash };
        crate::components::icon_button::icon_button(SharedString::from(format!("sshd-reset-{key}")), icon, !resets)
            .invisible()
            .group_hover("sshd-row", |s| s.visible())
            .tooltip(move |window, cx| gpui_kit::component::tooltip::Tooltip::new(if resets { "Back to sshd's default (removes the line)" } else { "Remove the line" }).build(window, cx))
            .on_click(move |_ev, _window, cx| {
                app.update(cx, |this, cx| this.delete_structured_row(&file, &row_id, cx));
            })
    });

    // Stacked, like a settings page: name and status, what it does, the
    // control, and what the chosen value means. Fits any pane width.
    div()
        .group("sshd-row")
        .flex()
        .flex_col()
        .gap(px(4.0))
        .px(px(14.0))
        .py(px(9.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .hover(|s| s.bg(BG_ROW_HOVER))
        .font_family(FONT_MONO)
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_x(px(8.0))
                .gap_y(px(2.0))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_set { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(if row.shadowed { TEXT_FAINT } else if is_set { TEXT_PRIMARY } else { TEXT_SECONDARY })
                        .child(row.name.clone()),
                )
                .children(risk_label(risk.as_ref()).filter(|_| is_risky(risk.as_ref()) && !row.shadowed).map(|l| {
                    let c = risk_color(risk.as_ref());
                    div().px(px(4.0)).border_1().border_color(c.opacity(0.5)).text_size(px(8.5)).font_weight(FontWeight::BOLD).text_color(c).child(l)
                }))
                .children((!is_set && effective.is_some()).then(|| div().text_size(px(9.0)).text_color(TEXT_FAINTER).child("default")))
                .child(div().flex_1())
                .children((!location.is_empty()).then(|| div().text_size(px(9.0)).text_color(if row.shadowed { WARN_INK } else { TEXT_FAINTER }).child(location)))
                .children(reset),
        )
        .children(help.map(|h| div().text_size(px(10.0)).line_height(px(14.0)).text_color(TEXT_DIMMER).child(h)))
        .child(div().pt(px(2.0)).child(value_col))
        .children(meaning.filter(|_| !row.shadowed).map(|m| div().text_size(px(9.5)).text_color(TEXT_FAINT).child(format!("→ {m}"))))
}

#[cfg(test)]
mod tests {
    // Not super::*: that brings gpui's #[test] in over the standard one.
    use super::{posture, PostureFix};
    use crate::config::plugins::{sshd_sheet, to_ir, SshdSheet, StructuredFormat};

    fn sheet(text: &str) -> SshdSheet {
        sshd_sheet(&to_ir(StructuredFormat::Sshd, text).unwrap(), &[])
    }

    #[test]
    fn posture_lists_risks_first_with_fixes_that_keep_crow_in() {
        let s = sheet("PermitRootLogin yes\nX11Forwarding yes\nPasswordAuthentication no\n");
        let (risky, fine) = posture(&s, "root");
        let names: Vec<&str> = risky.iter().map(|i| i.row.name.as_str()).collect();
        assert_eq!(names[0], "PermitRootLogin", "never-on-prod first");
        assert!(names.contains(&"X11Forwarding"));
        // KbdInteractiveAuthentication defaults to yes: risky though unset.
        assert!(names.contains(&"KbdInteractiveAuthentication"));
        assert_eq!(risky[0].fix, Some(PostureFix::Set("prohibit-password".into())), "Crow logs in as root: not \"no\"");
        assert!(fine.iter().any(|i| i.row.name == "PasswordAuthentication"));

        let (risky, _) = posture(&s, "deploy");
        assert_eq!(risky[0].fix, Some(PostureFix::Set("no".into())));
    }

    #[test]
    fn password_login_goes_through_the_guarded_flow() {
        let s = sheet("PasswordAuthentication yes\n");
        let (risky, _) = posture(&s, "root");
        let pw = risky.iter().find(|i| i.row.name == "PasswordAuthentication").unwrap();
        assert_eq!(pw.fix, Some(PostureFix::PasswordFlow));
    }
}
