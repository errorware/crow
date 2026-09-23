//! Generic structured editor: renders whatever a crow-config plugin describes
//! (rows of typed fields, enum options with risk ratings, help text, order
//! semantics) with no format-specific UI code. A new crow-config plugin gets
//! an editor here by being registered in `config::plugins`.

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::prelude::FluentBuilder as _;
use crow_config_core::ir::{ConfigDocumentIr, FieldIr, RowIr};
use crow_config_core::schema::{FieldType, RiskLevel, WidgetKind};

use crate::app::configs::{RiskConfirm, RISK_CONFIRM_KEYWORD};
use crate::app::CrowApp;
use crate::config::plugins::{plugin, sshd_match_scopes, DedicatedScreen, StructuredFormat};
use crate::config::ConfigFileState;
use crate::theme::*;

use super::state::ConfigsState;

/// The field being edited inline, if it is in this file.
pub struct ActiveFieldEdit<'a> {
    pub row_id: &'a str,
    pub field: &'a str,
    pub input: &'a Entity<InputState>,
}

pub fn risk_color(risk: Option<&RiskLevel>) -> Rgba {
    match risk {
        Some(RiskLevel::Recommended) => OK,
        Some(RiskLevel::Caution) | Some(RiskLevel::Weak) => WARN,
        Some(RiskLevel::NeverOnProd) | Some(RiskLevel::Deny) => CRIT,
        _ => TEXT_SECONDARY,
    }
}

fn risk_label(risk: Option<&RiskLevel>) -> Option<&'static str> {
    match risk? {
        RiskLevel::Recommended => Some("RECOMMENDED"),
        RiskLevel::Caution => Some("CAUTION"),
        RiskLevel::Weak => Some("WEAK"),
        RiskLevel::NeverOnProd => Some("NEVER ON PROD"),
        RiskLevel::Deny => Some("DENY"),
        RiskLevel::Other(_) => None,
    }
}

/// Display text of a field value (lists joined with spaces).
pub fn value_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(items) => items.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn small_button(id: impl Into<ElementId>, label: &'static str, color: Rgba) -> Stateful<Div> {
    div()
        .id(id)
        .px(px(6.0))
        .py(px(1.5))
        .rounded_sm()
        .border_1()
        .border_color(BORDER_DEFAULT)
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .text_color(color)
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(label)
}

#[allow(clippy::too_many_arguments)]
pub fn structured_editor(
    state: &ConfigFileState,
    format: StructuredFormat,
    ir: &ConfigDocumentIr,
    configs: &ConfigsState,
    active_edit: Option<ActiveFieldEdit>,
    risk_confirm: Option<&RiskConfirm>,
    app: Entity<CrowApp>,
) -> impl IntoElement {
    let file = state.filename.clone();
    let confirm_panel = risk_confirm.map(|c| render_risk_confirm(c, app.clone()));
    let is_modified = state.is_modified();
    let read_only = state.write_blocked.is_some();
    let (add_count, del_count) = state.diff_stats();
    let is_table = ir.shape.kind == WidgetKind::RuleTable;
    let scopes = if format == StructuredFormat::Sshd { sshd_match_scopes(ir) } else { Vec::new() };
    let scope_of = |row_id: &str| scopes.iter().find(|(id, _)| id == row_id).and_then(|(_, s)| s.clone());
    let row_ids: Vec<String> = ir.rows.iter().map(|r| r.row_id.clone()).collect();

    let header = {
        let (app_text, app_hist, app_revert, app_stage) = (app.clone(), app.clone(), app.clone(), app.clone());
        let (f_text, f_revert, f_stage) = (file.clone(), file.clone(), file.clone());
        div()
            .h(px(36.0))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(10.0))
            .px(px(14.0))
            .bg(BG_PANEL)
            .border_b_1()
            .border_color(BORDER_PANEL)
            .child(div().font_family(FONT_MONO).text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(file.clone()))
            .child(
                div()
                    .bg(hex_rgba(0x8ab4ff, 0.12))
                    .text_color(hex_rgb(0x8ab4ff))
                    .font_family(FONT_MONO)
                    .text_size(px(9.0))
                    .font_weight(FontWeight::BOLD)
                    .px(px(5.0))
                    .py(px(1.5))
                    .rounded_sm()
                    .child(format!("CROW UI · {} · {}", ir.plugin_name, ir.shape.kind)),
            )
            .child(
                div()
                    .bg(if is_modified { WARN_BG } else { OK_BG })
                    .text_color(if is_modified { WARN } else { OK })
                    .font_family(FONT_MONO)
                    .text_size(px(9.0))
                    .font_weight(FontWeight::BOLD)
                    .px(px(5.0))
                    .py(px(1.5))
                    .rounded_sm()
                    .child(if is_modified { format!("EDITED (+{} -{})", add_count, del_count) } else { format!("v{} (BASELINE)", state.active_revision) }),
            )
            .child(div().flex_1())
            .child(small_button("btn-structured-text-view", "TEXT VIEW", TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                let f = f_text.clone();
                app_text.update(cx, |this, cx| {
                    this.configs.text_mode.insert(f);
                    cx.notify();
                });
            }))
            .child(small_button("btn-structured-history", "HISTORY", TEXT_TERTIARY).on_click(move |_ev, _window, cx| {
                app_hist.update(cx, |this, cx| this.toggle_config_history(cx));
            }))
            .children(is_modified.then(|| {
                small_button("btn-structured-revert", "REVERT", CRIT).on_click(move |_ev, _window, cx| {
                    let f = f_revert.clone();
                    app_revert.update(cx, |this, cx| this.revert_managed_config(&f, cx));
                })
            }))
            .children(is_modified.then(|| {
                small_button("btn-structured-stage", "STAGE VERSION", OK).on_click(move |_ev, _window, cx| {
                    let f = f_stage.clone();
                    app_stage.update(cx, |this, cx| this.stage_config_version(&f, "Updated via structured editor", cx));
                })
            }))
    };

    let notices = div()
        .flex_none()
        .flex()
        .flex_col()
        .children(ir.shape.order_sensitive.then(|| {
            notice(
                format!("ORDER MATTERS · {}", ir.shape.order_note.clone().unwrap_or_else(|| "rows are evaluated top to bottom".into())),
                TEXT_TERTIARY,
                0x8ab4ff,
            )
        }))
        .children(state.write_blocked.as_ref().map(|r| notice(format!("READ-ONLY · {r}"), WARN, 0xf59e0b)))
        .children(configs.edit_error.as_ref().map(|e| notice(format!("EDIT REJECTED · {e}"), CRIT, 0xef4444)));

    let rows = ir.rows.iter().enumerate().map(|(idx, row)| {
        let scope = scope_of(&row.row_id);
        let is_match_line = row.fields.iter().any(|f| f.name.eq_ignore_ascii_case("match"));
        let row_locked = read_only || scope.is_some() || is_match_line;
        render_row(RowCtx {
            file: &file,
            row,
            index: idx,
            prev_id: idx.checked_sub(1).and_then(|i| row_ids.get(i)).cloned(),
            next_id: row_ids.get(idx + 1).cloned(),
            is_table,
            locked: row_locked,
            scope,
            open_enum: configs.open_enum.as_ref(),
            active_edit: active_edit.as_ref().filter(|e| e.row_id == row.row_id),
            app: app.clone(),
        })
    });

    let add_section = (!read_only).then(|| render_add_section(&file, format, ir, configs.adding_row, app.clone()));

    div()
        .id("structured-config-editor")
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .child(header)
        .child(notices)
        .children(confirm_panel)
        .child(
            div()
                .id("structured-rows")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .py(px(6.0))
                        .children(rows)
                        .children(add_section)
                        .children(ir.rows.is_empty().then(|| {
                            div()
                                .px(px(14.0))
                                .py(px(10.0))
                                .font_family(FONT_MONO)
                                .text_size(px(11.0))
                                .text_color(TEXT_FAINT)
                                .child("No entries — only comments or blank lines.")
                        })),
                ),
        )
}

fn notice(text: String, color: Rgba, tint: u32) -> Div {
    div()
        .px(px(14.0))
        .py(px(6.0))
        .bg(hex_rgba(tint, 0.08))
        .border_b_1()
        .border_color(hex_rgba(tint, 0.25))
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .text_color(color)
        .child(text)
}

struct RowCtx<'a> {
    file: &'a str,
    row: &'a RowIr,
    index: usize,
    prev_id: Option<String>,
    next_id: Option<String>,
    is_table: bool,
    locked: bool,
    scope: Option<String>,
    open_enum: Option<&'a (String, String)>,
    active_edit: Option<&'a ActiveFieldEdit<'a>>,
    app: Entity<CrowApp>,
}

fn render_row(ctx: RowCtx) -> impl IntoElement {
    let RowCtx { file, row, index, prev_id, next_id, is_table, locked, scope, open_enum, active_edit, app } = ctx;
    let row_id = row.row_id.clone();
    // Help text under key/value rows (e.g. what an sshd directive does). Table
    // rows share one schema, so repeating it on every row would only be noise.
    let help = (!is_table).then(|| row.fields.iter().find_map(|f| f.help.clone())).flatten();
    let docs = (!is_table).then(|| row.fields.iter().find_map(|f| f.docs_source.clone())).flatten();
    let opened = open_enum.filter(|(r, _)| *r == row_id).map(|(_, f)| f.clone());

    let fields = row.fields.iter().map(|field| {
        render_field(FieldCtx {
            file,
            row_id: &row_id,
            field,
            locked,
            show_label: is_table || row.fields.len() > 1,
            is_open: opened.as_deref() == Some(field.name.as_str()),
            editing: active_edit.filter(|e| e.field == field.name).map(|e| e.input),
            app: app.clone(),
        })
    });

    let actions = (!locked).then(|| {
        let mut bar = div().flex().items_center().gap(px(4.0)).flex_none();
        if is_table {
            if let Some(prev) = prev_id.clone() {
                let (app_up, f, r) = (app.clone(), file.to_string(), row_id.clone());
                bar = bar.child(small_button(ElementId::NamedInteger("row-up".into(), index as u64), "↑", TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    let (f, r, p) = (f.clone(), r.clone(), prev.clone());
                    app_up.update(cx, |this, cx| this.move_structured_row(&f, &r, None, Some(p), cx));
                }));
            }
            if let Some(next) = next_id.clone() {
                let (app_down, f, r) = (app.clone(), file.to_string(), row_id.clone());
                bar = bar.child(small_button(ElementId::NamedInteger("row-down".into(), index as u64), "↓", TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    let (f, r, n) = (f.clone(), r.clone(), next.clone());
                    app_down.update(cx, |this, cx| this.move_structured_row(&f, &r, Some(n), None, cx));
                }));
            }
        }
        let (app_del, f, r) = (app.clone(), file.to_string(), row_id.clone());
        bar.child(small_button(ElementId::NamedInteger("row-del".into(), index as u64), "✕", CRIT).on_click(move |_ev, _window, cx| {
            let (f, r) = (f.clone(), r.clone());
            app_del.update(cx, |this, cx| this.delete_structured_row(&f, &r, cx));
        }))
    });

    div()
        .id(ElementId::NamedInteger("structured-row".into(), index as u64))
        .flex()
        .flex_col()
        .gap(px(3.0))
        .px(px(14.0))
        .py(px(6.0))
        .border_b_1()
        .border_color(BORDER_ROW)
        .hover(|s| s.bg(BG_ROW_HOVER))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(14.0))
                .child(
                    div()
                        .w(px(34.0))
                        .flex_none()
                        .font_family(FONT_MONO)
                        .text_size(px(9.5))
                        .text_color(TEXT_FAINTER)
                        .child(format!("L{}", row.source_span.start_line)),
                )
                .child(div().flex_1().min_w(px(0.0)).flex().flex_wrap().items_start().gap(px(14.0)).children(fields))
                .children(actions),
        )
        .children(scope.map(|s| {
            div()
                .pl(px(48.0))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .text_color(WARN)
                .child(format!("Scoped to Match {s} — read-only until crow-config models Match blocks (ERR-12)"))
        }))
        .children((help.is_some() || docs.is_some()).then(|| {
            div()
                .pl(px(48.0))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .text_color(TEXT_FAINT)
                .child(match (help, docs) {
                    (Some(h), Some(d)) => format!("{h}  ·  {d}"),
                    (Some(h), None) => h,
                    (None, Some(d)) => d,
                    (None, None) => String::new(),
                })
        }))
}

struct FieldCtx<'a> {
    file: &'a str,
    row_id: &'a str,
    field: &'a FieldIr,
    locked: bool,
    show_label: bool,
    is_open: bool,
    editing: Option<&'a Entity<InputState>>,
    app: Entity<CrowApp>,
}

fn render_field(ctx: FieldCtx) -> impl IntoElement {
    let FieldCtx { file, row_id, field, locked, show_label, is_open, editing, app } = ctx;
    let text = value_text(&field.value);
    let current_risk = field.options.iter().flatten().find(|o| o.value == text).and_then(|o| o.risk.clone());
    let color = if field.valid == Some(false) { CRIT } else { risk_color(current_risk.as_ref()) };
    let id_base = format!("{row_id}:{}", field.name);

    let value_el: AnyElement = if let Some(input) = editing {
        div().min_w(px(180.0)).child(Input::new(input).font_family(FONT_MONO).text_size(px(11.0))).into_any_element()
    } else {
        let (app_click, f, r, name, is_enum) = (app.clone(), file.to_string(), row_id.to_string(), field.name.clone(), field.options.is_some());
        let is_list = field.field_type == FieldType::StringList;
        let current = text.clone();
        div()
            .id(SharedString::from(format!("field-{id_base}")))
            .px(px(6.0))
            .py(px(2.0))
            .rounded_sm()
            .border_1()
            .border_color(if locked { hex_rgba(0, 0.0) } else { BORDER_DEFAULT })
            .font_family(FONT_MONO)
            .text_size(px(11.5))
            .text_color(color)
            .when(!locked, |d| {
                d.cursor_pointer().hover(|s| s.bg(BG_CONTROL)).on_click(move |_ev, window, cx| {
                    let (f, r, n, v) = (f.clone(), r.clone(), name.clone(), current.clone());
                    app_click.update(cx, |this, cx| {
                        if is_enum {
                            this.toggle_structured_enum(&r, &n, cx);
                        } else {
                            this.begin_structured_field_edit(&f, &r, &n, &v, is_list, window, cx);
                        }
                    });
                })
            })
            .child(if text.is_empty() { "—".to_string() } else { text.clone() })
            .into_any_element()
    };

    let options = (is_open && !locked).then(|| {
        let opts = field.options.clone().unwrap_or_default();
        div().flex().flex_col().gap(px(2.0)).pt(px(3.0)).children(opts.into_iter().enumerate().map(|(i, opt)| {
            let (app_pick, f, r, n, v) = (app.clone(), file.to_string(), row_id.to_string(), field.name.clone(), opt.value.clone());
            let selected = opt.value == text;
            div()
                .id(SharedString::from(format!("opt-{id_base}-{i}")))
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(6.0))
                .py(px(2.0))
                .rounded_sm()
                .cursor_pointer()
                .bg(if selected { BG_CONTROL } else { hex_rgba(0, 0.0) })
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    let (f, r, n, v) = (f.clone(), r.clone(), n.clone(), v.clone());
                    app_pick.update(cx, |this, cx| this.set_structured_value(&f, &r, &n, serde_json::Value::String(v), cx));
                })
                .child(div().w(px(150.0)).font_family(FONT_MONO).text_size(px(11.0)).text_color(risk_color(opt.risk.as_ref())).child(opt.value.clone()))
                .children(risk_label(opt.risk.as_ref()).map(|l| {
                    div().w(px(96.0)).font_family(FONT_MONO).text_size(px(8.5)).font_weight(FontWeight::BOLD).text_color(risk_color(opt.risk.as_ref())).child(l)
                }))
                .child(div().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_TERTIARY).child(opt.label.clone()))
        }))
    });

    div()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .children(show_label.then(|| {
            div().font_family(FONT_MONO).text_size(px(8.5)).text_color(TEXT_FAINTER).child(field.name.to_uppercase())
        }))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .children((!show_label).then(|| {
                    div().min_w(px(170.0)).font_family(FONT_MONO).text_size(px(11.5)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(field.name.clone())
                }))
                .child(value_el)
                .children(risk_label(current_risk.as_ref()).filter(|_| current_risk != Some(RiskLevel::Recommended)).map(|l| {
                    div().font_family(FONT_MONO).text_size(px(8.5)).font_weight(FontWeight::BOLD).text_color(color).child(l)
                })),
        )
        .children(field.validation_error.clone().map(|e| div().font_family(FONT_MONO).text_size(px(9.5)).text_color(CRIT).child(e)))
        .children(options)
}

/// "+ ADD" section. Keyed formats (sshd) offer the plugin's known directives
/// that aren't set yet, inserted at their recommended value; list and table
/// formats insert a new row with the plugin's defaults.
fn render_add_section(file: &str, format: StructuredFormat, ir: &ConfigDocumentIr, open: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_toggle, app_add) = (app.clone(), app.clone());
    let f_add = file.to_string();
    let keyed = format == StructuredFormat::Sshd;
    let label = if keyed { "+ ADD DIRECTIVE" } else { "+ ADD ROW" };

    let candidates: Vec<(String, String, Option<RiskLevel>)> = if keyed && open {
        let present: Vec<String> = ir.rows.iter().flat_map(|r| r.fields.iter().map(|f| f.name.to_lowercase())).collect();
        plugin(format)
            .manifest()
            .fields
            .iter()
            .filter(|f| !present.contains(&f.name.to_lowercase()))
            .filter_map(|f| {
                let opts = f.options.as_ref()?;
                let pick = opts.iter().find(|o| o.risk == Some(RiskLevel::Recommended)).or_else(|| opts.first())?;
                Some((f.name.clone(), pick.value.clone(), pick.risk.clone()))
            })
            .collect()
    } else {
        Vec::new()
    };

    div()
        .flex()
        .flex_col()
        .gap(px(3.0))
        .px(px(14.0))
        .py(px(8.0))
        .child(div().flex().child(small_button("btn-structured-add", label, OK).on_click(move |_ev, _window, cx| {
            let f = f_add.clone();
            app_toggle.update(cx, |this, cx| {
                if keyed {
                    this.configs.adding_row = !this.configs.adding_row;
                    cx.notify();
                } else {
                    this.insert_structured_row(&f, None, cx);
                }
            });
        })))
        .children(candidates.into_iter().enumerate().map(|(i, (name, value, risk))| {
            let (app_pick, f) = (app_add.clone(), file.to_string());
            let (n, v) = (name.clone(), value.clone());
            div()
                .id(ElementId::NamedInteger("add-directive".into(), i as u64))
                .flex()
                .gap(px(10.0))
                .px(px(6.0))
                .py(px(2.0))
                .rounded_sm()
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .on_click(move |_ev, _window, cx| {
                    let (f, n, v) = (f.clone(), n.clone(), v.clone());
                    app_pick.update(cx, |this, cx| this.insert_structured_row(&f, Some((n, v)), cx));
                })
                .child(div().w(px(200.0)).font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_PRIMARY).child(name))
                .child(div().font_family(FONT_MONO).text_size(px(11.0)).text_color(risk_color(risk.as_ref())).child(value))
        }))
}

/// Shown for a config file owned by another Crow screen (e.g. crontab → Cron).
pub fn screen_handoff(file: &str, screen: DedicatedScreen, app: Entity<CrowApp>) -> impl IntoElement {
    div()
        .flex_1()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .bg(BG_APP)
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(12.0))
                .text_color(TEXT_SECONDARY)
                .child(format!("{file} is managed on the {} screen.", screen.title())),
        )
        .child(small_button("btn-open-dedicated-screen", "OPEN SCREEN", hex_rgb(0x8ab4ff)).on_click(move |_ev, _window, cx| {
            app.update(cx, |this, cx| this.open_dedicated_screen(screen, cx));
        }))
}

/// Typed confirmation for staging a change that introduces never-on-prod values.
fn render_risk_confirm(confirm: &RiskConfirm, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_ok, app_cancel) = (app.clone(), app);
    div()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .px(px(14.0))
        .py(px(10.0))
        .bg(hex_rgba(0xef4444, 0.08))
        .border_b_1()
        .border_color(hex_rgba(0xef4444, 0.35))
        .child(
            div()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(CRIT)
                .child("This change introduces values rated NEVER ON PROD:"),
        )
        .children(confirm.findings.iter().map(|(field, value, meaning)| {
            div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_SECONDARY).child(format!("  {field} {value} — {meaning}"))
        }))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().font_family(FONT_MONO).text_size(px(10.5)).text_color(TEXT_TERTIARY).child(format!("Type {RISK_CONFIRM_KEYWORD} to stage it:")))
                .children(confirm.input.as_ref().map(|input| div().w(px(160.0)).child(Input::new(input).font_family(FONT_MONO).text_size(px(11.0)))))
                .child(small_button("btn-risk-confirm", "STAGE ANYWAY", CRIT).on_click(move |_ev, _window, cx| {
                    app_ok.update(cx, |this, cx| this.confirm_risky_stage(cx));
                }))
                .child(small_button("btn-risk-cancel", "CANCEL", TEXT_SECONDARY).on_click(move |_ev, _window, cx| {
                    app_cancel.update(cx, |this, cx| this.cancel_risky_stage(cx));
                })),
        )
        .children(confirm.error.clone().map(|e| div().font_family(FONT_MONO).text_size(px(10.0)).text_color(CRIT).child(e)))
}
