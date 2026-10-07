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
use crate::config::plugins::{plugin, sshd_match_scopes, StructuredFormat};
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

pub(crate) fn risk_label(risk: Option<&RiskLevel>) -> Option<&'static str> {
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

pub(crate) fn small_button(id: impl Into<ElementId>, label: &'static str, color: Rgba) -> Stateful<Div> {
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
    let columns = table_columns(ir);
    // sysctl: where the file and the running kernel disagree (ERR-22).
    let sysctl_notes: Vec<Option<(String, bool)>> = if format == StructuredFormat::Sysctl {
        let kv = |r: &crow_config_core::ir::RowIr, n: &str| r.get_field(n).and_then(|f| f.value.as_str()).unwrap_or_default().to_string();
        ir.rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let key = kv(row, "key");
                let later = ir.rows[i + 1..].iter().rev().find(|r| kv(r, "key") == key).map(|r| r.source_span.start_line);
                crate::config::sysctl_live::row_note(&kv(row, "value"), configs.sysctl_live.get(&key), later)
            })
            .collect()
    } else if format == StructuredFormat::Sudoers {
        // What a passwordless rule lets its users do (ERR-22).
        ir.rows
            .iter()
            .map(|row| {
                let field = |n: &str| row.get_field(n).and_then(|f| f.value.as_str()).unwrap_or_default();
                (field("kind") == "rule").then(|| crow_config_schemas::sudoers_rule_risk(field("rule"))).flatten().map(|risk| (format!("lets {} run {risk}", field("who")), true))
            })
            .collect()
    } else if format == StructuredFormat::Fstab {
        // Mounts that can stop a boot (ERR-22).
        let all = crow_config_schemas::fstab_mounts(&state.current_content);
        ir.rows
            .iter()
            .map(|row| {
                let mp = row.get_field("mountpoint").and_then(|f| f.value.as_str()).unwrap_or_default();
                all.iter().find(|m| m.mountpoint == mp).and_then(|m| crow_config_schemas::fstab_boot_risk(m, &all)).map(|r| (r.to_string(), true))
            })
            .collect()
    } else {
        Vec::new()
    };
    let not_running = if format == StructuredFormat::Sysctl { sysctl_notes.iter().flatten().filter(|(_, warn)| *warn).count() } else { 0 };

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
        .children((not_running > 0).then(|| {
            let what = if not_running == 1 { "1 value here isn't".to_string() } else { format!("{not_running} values here aren't") };
            notice(format!("NOT RUNNING YET · {what} what the kernel runs now; they apply at boot or with sysctl --system"), WARN, 0xf59e0b)
        }))
        .children(state.write_blocked.as_ref().map(|r| notice(format!("READ-ONLY · {r}"), WARN, 0xf59e0b)))
        .children(configs.edit_error.as_ref().map(|e| notice(format!("EDIT REJECTED · {e}"), CRIT, 0xef4444)));

    let rows = ir.rows.iter().enumerate().map(|(idx, row)| {
        let scope = scope_of(&row.row_id);
        let is_match_line = row.fields.iter().any(|f| f.name.eq_ignore_ascii_case("match"));
        // logrotate: blocks read as a heading (their log paths) with their
        // directives indented under it; scripts are edited in the text view.
        let block_format = matches!(format, StructuredFormat::Logrotate | StructuredFormat::Systemd | StructuredFormat::Nginx);
        // nginx nests: depth is how many blocks the row is in.
        let depth = if format == StructuredFormat::Nginx { row.scope.as_deref().map_or(0, |s| s.split(crow_config_schemas::nginx::SCOPE_SEP).count()) } else { usize::from(row.scope.is_some()) };
        let nest = match (block_format, row.widget.as_str(), row.scope.is_some()) {
            (true, "scope_row", _) => Nest::Heading(depth.saturating_sub(1)),
            (true, _, true) => Nest::Inside(depth),
            _ => Nest::Flat,
        };
        let row_locked = read_only || scope.is_some() || is_match_line || (block_format && row.widget == "script_row");
        render_row(RowCtx {
            nest,
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
            columns: columns.as_deref(),
            canonical_first: format == StructuredFormat::Hosts,
            note: sysctl_notes.get(idx).cloned().flatten(),
            app: app.clone(),
        })
    });
    let column_header = columns.as_ref().filter(|_| !ir.rows.is_empty()).map(|cols| {
        div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(14.0))
            .py(px(6.0))
            .bg(BG_PANEL)
            .border_b_1()
            .border_color(BORDER_PANEL)
            .font_family(FONT_MONO)
            .text_size(px(9.0))
            .font_weight(FontWeight::BOLD)
            .text_color(TEXT_FAINT)
            .child(div().w(px(34.0)).flex_none().child("LINE"))
            .children(cols.iter().map(|c| column_cell(c.width).child(c.name.to_uppercase())))
    });

    // systemd: a new key belongs in a particular section, so keys are added
    // in the text view for now.
    let add_section = (!read_only && !matches!(format, StructuredFormat::Systemd | StructuredFormat::Nginx)).then(|| render_add_section(&file, format, ir, configs.adding_row, app.clone()));

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
        .children(column_header)
        .child(
            div()
                .id("structured-rows")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                // sshd_config reads as a settings sheet (grouped, defaults shown).
                .children((format == StructuredFormat::Sshd).then(|| {
                    super::sshd_sheet::sshd_sheet_view(&file, &crate::config::plugins::sshd_sheet(ir, &configs.sshd_includes), read_only, active_edit.as_ref(), app.clone())
                }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .py(px(6.0))
                        .when(format == StructuredFormat::Sshd, |d| d.hidden())
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

/// How wide a column is on screen.
#[derive(Clone, Copy, PartialEq)]
enum ColWidth {
    Fixed(f32),
    /// Takes the remaining space (lists, trailing comments).
    Grow,
}

/// One column of a multi-field table (hosts, pg_hba).
pub(crate) struct Column {
    pub name: String,
    width: ColWidth,
}

impl Column {
    /// Its width in pixels, unless it takes the remaining space.
    #[cfg(test)]
    pub(crate) fn fixed_width(&self) -> Option<f32> {
        match self.width {
            ColWidth::Fixed(w) => Some(w),
            ColWidth::Grow => None,
        }
    }
}

/// Width of one character of a field value (JetBrains Mono at 11.5px).
const CHAR_W: f32 = 7.0;

fn width_for(name: &str, field_type: &FieldType) -> ColWidth {
    match field_type {
        _ if name == "comment" => ColWidth::Grow,
        FieldType::IpAddress => ColWidth::Fixed(150.0),
        FieldType::Cidr => ColWidth::Fixed(170.0),
        FieldType::Enum => ColWidth::Fixed(130.0),
        FieldType::Port | FieldType::Bool => ColWidth::Fixed(70.0),
        FieldType::Path => ColWidth::Fixed(220.0),
        FieldType::StringList => ColWidth::Fixed(320.0),
        _ if name == "address" => ColWidth::Fixed(170.0),
        _ => ColWidth::Fixed(130.0),
    }
}

/// Column layout for documents whose rows have several fields; `None` for
/// key/value documents (sshd), which read better as labelled lines. Columns
/// follow the order fields first appear in, so every plugin gets a table
/// without per-plugin layout code.
pub(crate) fn table_columns(ir: &ConfigDocumentIr) -> Option<Vec<Column>> {
    if !ir.rows.iter().any(|r| r.fields.len() > 1) {
        return None;
    }
    let mut cols: Vec<Column> = Vec::new();
    for field in ir.rows.iter().flat_map(|r| &r.fields) {
        if !cols.iter().any(|c| c.name == field.name) {
            cols.push(Column { name: field.name.clone(), width: width_for(&field.name, &field.field_type) });
        }
    }
    // Each column is as wide as its longest value (or its header), so a
    // long key never runs under the next column; very long ones are clipped.
    for col in cols.iter_mut().filter(|c| matches!(c.width, ColWidth::Fixed(_))) {
        // A field with options shows its option's meaning too (a switch
        // and "ignores them"), so those count.
        let longest = ir
            .rows
            .iter()
            .filter_map(|r| r.get_field(&col.name))
            .flat_map(|f| {
                let labels = f.options.iter().flatten().map(|o| o.value.chars().count() + o.label.chars().count() + 6);
                std::iter::once(value_text(&f.value).chars().count()).chain(labels)
            })
            .chain([col.name.len()])
            .max()
            .unwrap_or(0);
        col.width = ColWidth::Fixed((longest as f32 * CHAR_W + 24.0).clamp(56.0, 460.0));
    }
    // The comment always trails.
    if let Some(i) = cols.iter().position(|c| c.name == "comment") {
        let c = cols.remove(i);
        cols.push(c);
    }
    // The last column takes what's left, so nothing in it is ever cut.
    if let Some(last) = cols.last_mut() {
        last.width = ColWidth::Grow;
    }
    Some(cols)
}

fn column_cell(width: ColWidth) -> Div {
    match width {
        ColWidth::Fixed(w) => div().w(px(w)).flex_none().min_w(px(0.0)).overflow_hidden(),
        ColWidth::Grow => div().flex_1().min_w(px(0.0)),
    }
}

/// Where a row sits in a block-structured file.
#[derive(Clone, Copy, PartialEq)]
enum Nest {
    Flat,
    /// A block's opening line, inside this many blocks.
    Heading(usize),
    /// A line inside this many blocks.
    Inside(usize),
}

struct RowCtx<'a> {
    nest: Nest,
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
    columns: Option<&'a [Column]>,
    canonical_first: bool,
    /// A line under the row: (text, whether it's a warning).
    note: Option<(String, bool)>,
    app: Entity<CrowApp>,
}

fn render_row(ctx: RowCtx) -> impl IntoElement {
    let RowCtx { nest, file, row, index, prev_id, next_id, is_table, locked, scope, open_enum, active_edit, columns, canonical_first, note, app } = ctx;
    let row_id = row.row_id.clone();
    // Help text under key/value rows (e.g. what an sshd directive does). Table
    // rows share one schema, so repeating it on every row would only be noise.
    let help = (!is_table).then(|| row.fields.iter().find_map(|f| f.help.clone())).flatten();
    let docs = (!is_table).then(|| row.fields.iter().find_map(|f| f.docs_source.clone())).flatten();
    let opened = open_enum.filter(|(r, _)| *r == row_id).map(|(_, f)| f.clone());

    let field_el = |field: &FieldIr, layout: FieldLayout| {
        render_field(FieldCtx {
            file,
            row_id: &row_id,
            field,
            locked,
            layout,
            role: if field.name == "comment" && columns.is_some() {
                FieldRole::Comment
            } else if canonical_first && field.field_type == FieldType::StringList {
                FieldRole::CanonicalFirst
            } else {
                FieldRole::Plain
            },
            is_open: opened.as_deref() == Some(field.name.as_str()),
            editing: active_edit.filter(|e| e.field == field.name).map(|e| e.input),
            app: app.clone(),
        })
    };
    let fields_el: Div = match columns {
        // One cell per column, in column order; a field the row lacks is an empty cell.
        Some(cols) => div().flex_1().min_w(px(0.0)).flex().items_start().gap(px(14.0)).children(cols.iter().map(|c| {
            column_cell(c.width).children(row.fields.iter().find(|f| f.name == c.name).map(|f| field_el(f, FieldLayout::Column)))
        })),
        None => div().flex_1().min_w(px(0.0)).flex().flex_wrap().items_start().gap(px(14.0)).children(row.fields.iter().map(|f| {
            field_el(f, if is_table || row.fields.len() > 1 { FieldLayout::Labeled } else { FieldLayout::KeyValue })
        })),
    };

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
        .pr(px(14.0))
        .pl(px(14.0 + 20.0 * match nest { Nest::Flat => 0, Nest::Heading(d) | Nest::Inside(d) => d } as f32))
        .py(px(7.0))
        .when(index % 2 == 1 && !matches!(nest, Nest::Heading(_)), |d| d.bg(hex_rgba(0xffffff, 0.018)))
        .when(matches!(nest, Nest::Heading(_)), |d| d.bg(BG_SUBHEAD).mt(px(6.0)).border_t_1().border_color(BORDER_PANEL))
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
                .child(fields_el)
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
        .children(note.map(|(text, warn)| div().pl(px(48.0)).font_family(FONT_MONO).text_size(px(9.5)).text_color(if warn { WARN } else { TEXT_DIM }).child(text)))
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

/// How a field is laid out in its row.
#[derive(Clone, Copy, PartialEq)]
enum FieldLayout {
    /// `Name  value` — single-field key/value rows (sshd).
    KeyValue,
    /// Small label above the value.
    Labeled,
    /// In a table column; the header names it.
    Column,
}

/// How a field's value is shown.
#[derive(Clone, Copy, PartialEq)]
enum FieldRole {
    Plain,
    /// Trailing comment: dimmed, `#`-prefixed.
    Comment,
    /// A list whose first entry is canonical and the rest aliases (hosts).
    CanonicalFirst,
}

struct FieldCtx<'a> {
    file: &'a str,
    row_id: &'a str,
    field: &'a FieldIr,
    locked: bool,
    layout: FieldLayout,
    role: FieldRole,
    is_open: bool,
    editing: Option<&'a Entity<InputState>>,
    app: Entity<CrowApp>,
}

fn render_field(ctx: FieldCtx) -> impl IntoElement {
    let FieldCtx { file, row_id, field, locked, layout, role, is_open, editing, app } = ctx;
    let show_label = layout == FieldLayout::Labeled;
    let text = value_text(&field.value);
    let current_risk = field.options.iter().flatten().find(|o| o.value == text).and_then(|o| o.risk.clone());
    let color = if field.valid == Some(false) { CRIT } else { risk_color(current_risk.as_ref()) };
    let id_base = format!("{row_id}:{}", field.name);

    // Two values, 0 and 1: an on/off switch rather than a picker.
    // Also yes/no and false/true pairs (systemd's hardening keys).
    let pair = |off: &str, on: &str| field.options.as_ref().filter(|o| o.len() == 2 && o.iter().any(|v| v.value == off) && o.iter().any(|v| v.value == on)).map(|o| (o, off.to_string(), on.to_string()));
    let switch = pair("0", "1").or_else(|| pair("no", "yes")).or_else(|| pair("false", "true"));
    let value_el: AnyElement = if let (Some((opts, off_value, on_value)), None) = (switch, editing) {
        let on = text == on_value;
        let label = opts.iter().find(|o| o.value == text).map(|o| o.label.clone()).unwrap_or_else(|| text.clone());
        let (app_flip, f, r, n) = (app.clone(), file.to_string(), row_id.to_string(), field.name.clone());
        div()
            .id(SharedString::from(format!("switch-{id_base}")))
            .flex()
            .items_center()
            .gap(px(8.0))
            .when(!locked, |d| {
                d.cursor_pointer().on_click(move |_ev, _window, cx| {
                    let (f, r, n) = (f.clone(), r.clone(), n.clone());
                    let next = if on { off_value.clone() } else { on_value.clone() };
                    app_flip.update(cx, |this, cx| this.set_structured_value(&f, &r, &n, serde_json::Value::String(next), cx));
                })
            })
            .child(
                div()
                    .w(px(30.0))
                    .h(px(16.0))
                    .flex_none()
                    .rounded_full()
                    .p(px(2.0))
                    .flex()
                    .when(on, |d| d.justify_end())
                    .bg(if on { OK_BG } else { BG_CONTROL })
                    .border_1()
                    .border_color(if on { OK } else { BORDER_STRONG })
                    .child(div().size(px(10.0)).rounded_full().bg(if on { OK } else { TEXT_DIM })),
            )
            .child(div().font_family(FONT_MONO).text_size(px(11.0)).text_color(color).child(label))
            .into_any_element()
    } else if let Some(input) = editing {
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
            .map(|d| match role {
                _ if text.is_empty() => d.child(if role == FieldRole::Comment { String::new() } else { "—".to_string() }),
                FieldRole::Comment => d.text_color(TEXT_FAINT).child(format!("# {text}")),
                FieldRole::CanonicalFirst => {
                    let mut names = text.split_whitespace();
                    let canonical = names.next().unwrap_or_default().to_string();
                    let aliases = names.collect::<Vec<_>>().join(" ");
                    d.flex()
                        .gap(px(8.0))
                        .child(div().font_weight(FontWeight::BOLD).child(canonical))
                        .children((!aliases.is_empty()).then(|| div().text_color(TEXT_TERTIARY).child(aliases)))
                }
                FieldRole::Plain => d.child(text.clone()),
            })
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
                .children((layout == FieldLayout::KeyValue).then(|| {
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
