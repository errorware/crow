//! A user plugin's page on a server (ERR-178), drawn only from blocks Crow
//! already has: a status, key and value pairs, a table, buttons, the last
//! run's output, and the run history. Stage 4 runs read actions only.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::input::Input;
use gpui_kit::*;

use crate::app::user_plugins::{RunSlot, UserPluginsState};
use crate::app::CrowApp;
use crate::plugin_host::folder::Installed;
use crate::plugin_host::manifest::{Action, Block, BlockKind, Effect, FieldKind, Page};
use crate::plugin_host::run::{self, State};
use crate::theme::*;
use crate::vault::PluginRun;

pub struct PageData<'a> {
    pub plugin: &'a Installed,
    pub page: &'a Page,
    pub server_id: &'a str,
    pub state: &'a UserPluginsState,
    pub runs: Vec<PluginRun>,
    /// It has a setup that hasn't succeeded with these files.
    pub needs_install: bool,
}

fn title(text: String) -> Div {
    div().text_size(px(10.5)).font_weight(FontWeight::BOLD).text_color(TEXT_SECONDARY).child(text)
}

fn panel() -> Div {
    div().flex().flex_col().gap(px(8.0)).p(px(12.0)).bg(BG_PANEL).border_1().border_color(BORDER_PANEL)
}

fn when_ago(ts: i64) -> String {
    let secs = chrono::Utc::now().timestamp() - ts;
    match secs {
        s if s < 60 => t!("plugin-page-just-now").to_string(),
        s if s < 3600 => t!("plugin-page-minutes-ago", n = s / 60),
        s if s < 86_400 => t!("plugin-page-hours-ago", n = s / 3600),
        _ => chrono::DateTime::from_timestamp(ts, 0).map(|t| t.with_timezone(&chrono::Local).format("%b %d %H:%M").to_string()).unwrap_or_default(),
    }
}

/// What a read block shows while its action hasn't answered, or why it can't.
fn pending_or_error(slot: Option<&RunSlot>) -> Option<Div> {
    match slot {
        None => Some(div().text_size(px(10.5)).text_color(TEXT_DIMMER).child(t!("plugin-page-not-read-yet"))),
        Some(s) if s.running && s.last.is_none() => Some(div().text_size(px(10.5)).text_color(TEXT_DIMMER).child(t!("plugin-page-reading"))),
        Some(RunSlot { last: Some(r), .. }) if !r.ok => {
            let why = r.error.clone().unwrap_or_else(|| r.stderr.trim().lines().last().unwrap_or_default().to_string());
            let code = r.exit_code.map(|c| t!("plugin-page-exit", code = c)).unwrap_or_default();
            Some(div().flex().flex_col().gap(px(2.0)).child(div().text_size(px(10.5)).text_color(CRIT).child(why)).when(!code.is_empty(), |d| d.child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child(code))))
        }
        _ => None,
    }
}

fn block_view(i: usize, b: &Block, d: &PageData, app: &Entity<CrowApp>) -> AnyElement {
    let m = d.plugin.report.manifest.as_ref().expect("an accepted plugin has a manifest");
    let action = b.action.as_deref().and_then(|id| m.actions.iter().find(|a| a.id == id));
    let slot = action.and_then(|a| d.state.slot(&d.plugin.id, &a.id, d.server_id));
    let heading = b.title.clone().or_else(|| action.map(|a| a.label.clone()));
    match b.kind {
        BlockKind::Status => {
            let body = pending_or_error(slot).map(IntoElement::into_any_element).unwrap_or_else(|| {
                let r = slot.and_then(|s| s.last.as_ref()).expect("a read result");
                match run::status(&r.stdout) {
                    Ok(st) => {
                        let color = match st.state {
                            State::Ok => OK,
                            State::Warn => WARN,
                            State::Bad => CRIT,
                            State::Unknown => TEXT_DIM,
                        };
                        div().flex().items_center().gap(px(8.0)).child(div().size(px(9.0)).rounded_full().bg(color)).child(div().text_size(px(12.5)).font_weight(FontWeight::BOLD).text_color(color).child(st.label)).into_any_element()
                    }
                    Err(e) => div().text_size(px(10.5)).text_color(CRIT).child(t!("plugin-page-unreadable", why = e)).into_any_element(),
                }
            });
            panel().children(heading.map(title)).child(body).into_any_element()
        }
        BlockKind::Kv => {
            let body = pending_or_error(slot).map(IntoElement::into_any_element).unwrap_or_else(|| {
                let r = slot.and_then(|s| s.last.as_ref()).expect("a read result");
                match run::kv(&action.expect("checked").output, &r.stdout) {
                    Ok(pairs) => div()
                        .flex()
                        .flex_col()
                        .gap(px(3.0))
                        .children(pairs.into_iter().map(|(k, v)| div().flex().gap(px(12.0)).text_size(px(10.5)).child(div().w(px(180.0)).flex_none().truncate().text_color(TEXT_DIM).child(k)).child(div().min_w(px(0.0)).text_color(TEXT_PRIMARY).child(v))))
                        .into_any_element(),
                    Err(e) => div().text_size(px(10.5)).text_color(CRIT).child(t!("plugin-page-unreadable", why = e)).into_any_element(),
                }
            });
            panel().children(heading.map(title)).child(body).into_any_element()
        }
        BlockKind::Table => {
            let body = pending_or_error(slot).map(IntoElement::into_any_element).unwrap_or_else(|| {
                let r = slot.and_then(|s| s.last.as_ref()).expect("a read result");
                match run::table(&action.expect("checked").output, &r.stdout) {
                    Ok((_, rows)) if rows.is_empty() => div().text_size(px(10.5)).text_color(TEXT_DIM).child(t!("plugin-page-no-rows")).into_any_element(),
                    Ok((cols, rows)) => {
                        let n = cols.len().max(1);
                        let cell = |text: String, head: bool| div().flex_1().min_w(px(0.0)).truncate().px(px(8.0)).py(px(4.0)).text_size(px(10.5)).text_color(if head { TEXT_FAINT } else { TEXT_PRIMARY }).when(head, |d| d.font_weight(FontWeight::BOLD)).child(text);
                        div()
                            .flex()
                            .flex_col()
                            .border_1()
                            .border_color(BORDER_ROW)
                            .child(div().flex().bg(BG_SUBHEAD).children(cols.into_iter().map(|c| cell(c.to_uppercase(), true))))
                            .children(rows.into_iter().take(500).enumerate().map(move |(j, r)| {
                                let mut r = r;
                                r.resize(n, String::new());
                                div().flex().border_t_1().border_color(BORDER_ROW).when(j % 2 == 1, |d| d.bg(BG_ROW_ALT)).children(r.into_iter().map(|c| cell(c, false)))
                            }))
                            .into_any_element()
                    }
                    Err(e) => div().text_size(px(10.5)).text_color(CRIT).child(t!("plugin-page-unreadable", why = e)).into_any_element(),
                }
            });
            panel().children(heading.map(title)).child(body).into_any_element()
        }
        BlockKind::Actions => {
            let buttons = b.actions.iter().filter_map(|id| m.actions.iter().find(|a| &a.id == id)).enumerate().map(|(j, a)| action_button(i * 100 + j, a, d, app.clone()));
            let shown = d.state.shown_output.as_ref().filter(|(p, a, s)| p == &d.plugin.id && s == d.server_id && b.actions.contains(a));
            let pending = d.state.pending.as_ref().filter(|p| p.key.0 == d.plugin.id && p.key.2 == d.server_id && b.actions.contains(&p.key.1));
            panel()
                .child(div().flex().flex_wrap().gap(px(8.0)).children(buttons))
                .children(pending.and_then(|p| m.actions.iter().find(|a| a.id == p.key.1)).map(|a| confirm_panel(a, d, app.clone())))
                .children(shown.filter(|_| pending.is_none()).and_then(|(_, a, _)| d.state.slot(&d.plugin.id, a, d.server_id).map(|s| (a.clone(), s))).map(|(a, s)| console(&d.plugin.id, &a, d.server_id, s, app.clone())))
                .into_any_element()
        }
        BlockKind::Runs => {
            let label = |id: &str| m.actions.iter().find(|a| a.id == id).map_or(id.to_string(), |a| a.label.clone());
            panel()
                .child(title(t!("plugin-page-runs").to_string()))
                .when(d.runs.is_empty(), |p| p.child(div().text_size(px(10.5)).text_color(TEXT_DIM).child(t!("plugin-page-no-runs"))))
                .children(d.runs.iter().take(20).map(|r| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .text_size(px(10.5))
                        .child(div().w(px(110.0)).flex_none().text_color(TEXT_FAINT).child(when_ago(r.started_at)))
                        .child(div().w(px(14.0)).flex_none().text_color(if r.ok { OK } else { CRIT }).child(if r.ok { "✓" } else { "✗" }))
                        .child(div().w(px(160.0)).flex_none().truncate().text_color(TEXT_SECONDARY).child(label(&r.action_id)))
                        .when(r.dry_run, |row| row.child(div().flex_none().px(px(4.0)).bg(BG_CHIP).text_size(px(9.0)).text_color(TEXT_TERTIARY).child(t!("plugin-page-dry-run-chip"))))
                        .when(r.stopped, |row| row.child(div().flex_none().px(px(4.0)).bg(WARN_BG).text_size(px(9.0)).text_color(WARN_INK).child(t!("plugin-page-stopped-chip"))))
                        .child(div().min_w(px(0.0)).truncate().text_color(TEXT_DIM).child(r.summary.clone()))
                }))
                .into_any_element()
        }
    }
}

fn action_button(n: usize, a: &Action, d: &PageData, app: Entity<CrowApp>) -> impl IntoElement {
    let running = d.state.slot(&d.plugin.id, &a.id, d.server_id).is_some_and(|s| s.running);
    let usable = true;
    let (plugin, action) = (d.plugin.id.clone(), a.id.clone());
    let color = match a.effect {
        Effect::Read => TEXT_SECONDARY,
        Effect::Change => WARN,
        Effect::Danger => CRIT,
    };
    div()
        .id(("plugin-action", n))
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(if usable { color } else { BORDER_ROW })
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(if usable { color } else { TEXT_FAINT })
        .when(usable && !running, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.press_plugin_action(&plugin, &action, cx))))
        .child(if running { t!("plugin-page-running", label = a.label) } else { a.label.to_uppercase() })

}

/// The run going (its lines as they arrive, with STOP), or the last one.
pub fn console(plugin: &str, action: &str, server: &str, slot: &RunSlot, app: Entity<CrowApp>) -> impl IntoElement {
    let lines: Vec<(bool, String)> = if slot.running {
        slot.live.lock().map(|l| l.clone()).unwrap_or_default()
    } else {
        match &slot.last {
            Some(r) => r.stdout.lines().map(|l| (false, l.to_string())).chain(r.stderr.lines().map(|l| (true, l.to_string()))).collect(),
            None => Vec::new(),
        }
    };
    let status = match (slot.running, &slot.last) {
        (true, _) => (t!("plugin-page-running-now").to_string(), TEXT_SECONDARY),
        (false, Some(r)) if r.stopped => (r.error.clone().unwrap_or_default(), WARN),
        (false, Some(r)) => match (&r.error, r.exit_code) {
            (Some(e), _) => (e.clone(), CRIT),
            (None, Some(c)) => (t!("plugin-page-exit", code = c), if r.ok { OK } else { CRIT }),
            (None, None) => (String::new(), TEXT_DIM),
        },
        (false, None) => (String::new(), TEXT_DIM),
    };
    let truncated = slot.last.as_ref().is_some_and(|r| r.truncated) && !slot.running;
    let (p, a, s) = (plugin.to_string(), action.to_string(), server.to_string());
    let stopping = slot.stop.load(std::sync::atomic::Ordering::Relaxed);
    let tail = lines.len().saturating_sub(2000);
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(div().text_size(px(9.5)).text_color(status.1).child(status.0))
                .child(div().flex_1())
                .when(slot.running, |d| {
                    d.child(
                        div()
                            .id("btn-plugin-stop")
                            .px(px(8.0))
                            .py(px(3.0))
                            .border_1()
                            .border_color(CRIT)
                            .text_color(CRIT)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .when(!stopping, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.stop_plugin_run(&p, &a, &s, cx))))
                            .child(if stopping { t!("plugin-page-stopping") } else { t!("plugin-page-stop") }),
                    )
                }),
        )
        .when(!lines.is_empty() || slot.running, |d| d.child(
            div()
                .id("plugin-console")
                .max_h(px(320.0))
                .overflow_y_scrollbar()
                .p(px(8.0))
                .bg(BG_WINDOW)
                .border_1()
                .border_color(BORDER_ROW)
                .text_size(px(10.5))
                .children(lines.into_iter().skip(tail).map(|(is_err, l)| div().text_color(if is_err { TEXT_DIM } else { TEXT_PRIMARY }).child(l)).collect::<Vec<_>>()),
        ))
        .when(truncated, |d| d.child(div().text_size(px(9.5)).text_color(WARN).child(t!("plugin-page-truncated"))))
}

/// A change or danger action about to run: what it will run, its inputs,
/// its dry run, then RUN (and the keyword for danger).
fn confirm_panel(a: &Action, d: &PageData, app: Entity<CrowApp>) -> impl IntoElement {
    let pending = d.state.pending.as_ref().expect("a pending run");
    let danger = a.effect == Effect::Danger;
    let color = if danger { CRIT } else { WARN };
    let dry = a.dry_run.as_ref().and_then(|id| d.plugin.report.manifest.as_ref()?.actions.iter().find(|x| &x.id == id));
    let dry_slot = dry.and_then(|x| d.state.slot(&d.plugin.id, &x.id, d.server_id));
    let (app_dry, app_run, app_cancel) = (app.clone(), app.clone(), app.clone());
    let command = crate::plugin_host::check::command_line(a.command.as_deref().unwrap_or_default());
    let runs = match (&a.command, &a.script) {
        (Some(_), _) => command,
        (None, Some(sc)) => format!("{} {sc} {}", a.interpreter.map(|i| i.program()).unwrap_or("sh"), crate::plugin_host::check::command_line(&a.args)).trim_end().to_string(),
        _ => String::new(),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(10.0))
        .border_1()
        .border_color(color)
        .bg(if danger { CRIT_STRIP_BG } else { BG_OVERLAY_PANEL })
        .child(div().text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(color).child(if danger { t!("plugin-page-danger-title", label = a.label) } else { t!("plugin-page-change-title", label = a.label) }))
        .children(a.help.clone().map(|h| div().text_size(px(10.0)).text_color(TEXT_DIM).child(h)))
        .child(div().px(px(8.0)).py(px(5.0)).bg(BG_WINDOW).border_1().border_color(BORDER_ROW).text_size(px(10.5)).text_color(TEXT_PRIMARY).child(format!("{}{runs}", if a.root { "# " } else { "$ " })))
        .children(a.inputs.iter().map(|f| {
            let field: AnyElement = match f.kind {
                FieldKind::Text | FieldKind::Number => match d.state.pending_inputs.iter().find(|(k, _)| k == &f.key) {
                    Some((_, input)) => div().w(px(360.0)).child(Input::new(input).font_family(FONT_MONO).bg(BG_APP)).into_any_element(),
                    None => div().into_any_element(),
                },
                FieldKind::Bool | FieldKind::Choice | FieldKind::Choices => {
                    let options: Vec<String> = if f.kind == FieldKind::Bool { vec!["true".into(), "false".into()] } else { f.choices.clone() };
                    let current = pending.picked.get(&f.key).cloned().unwrap_or_default();
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(options.into_iter().enumerate().map(|(i, o)| {
                            let on = if f.kind == FieldKind::Choices { current.split(',').any(|c| c == o) } else { current == o };
                            let next = if f.kind == FieldKind::Choices {
                                let mut set: Vec<&str> = current.split(',').filter(|c| !c.is_empty() && *c != o).collect();
                                if !on {
                                    set.push(&o);
                                }
                                set.join(",")
                            } else {
                                o.clone()
                            };
                            let (app, key) = (app.clone(), f.key.clone());
                            div()
                                .id(ElementId::Name(format!("plugin-pick-{}-{i}", f.key).into()))
                                .px(px(8.0))
                                .py(px(3.0))
                                .border_1()
                                .border_color(if on { TEXT_PRIMARY } else { BORDER_ROW })
                                .text_size(px(10.5))
                                .text_color(if on { TEXT_PRIMARY } else { TEXT_DIM })
                                .cursor_pointer()
                                .on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.pick_plugin_input(&key, next.clone(), cx)))
                                .child(o)
                        }))
                        .into_any_element()
                }
                FieldKind::Secret => div().into_any_element(),
            };
            div().flex().items_center().gap(px(10.0)).child(div().w(px(180.0)).flex_none().text_size(px(10.5)).text_color(TEXT_SECONDARY).child(f.label.clone())).child(field)
        }))
        .children(dry.map(|x| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .id("btn-plugin-dry-run")
                                .px(px(10.0))
                                .py(px(5.0))
                                .border_1()
                                .border_color(TEXT_SECONDARY)
                                .text_color(TEXT_SECONDARY)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _w, cx| app_dry.update(cx, |this, cx| this.dry_run_pending(cx)))
                                .child(t!("plugin-page-dry-run", label = x.label)),
                        )
                        .child(div().text_size(px(10.0)).text_color(TEXT_DIM).child(t!("plugin-page-dry-run-needed"))),
                )
                .children(dry_slot.map(|s| console(&d.plugin.id, &x.id, d.server_id, s, app.clone())))
        }))
        .when(danger, |p| {
            p.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(div().text_size(px(10.5)).text_color(CRIT_INK).child(t!("plugin-page-type-to-confirm", keyword = a.keyword.clone().unwrap_or_else(|| "CONFIRM".into()))))
                    .children(d.state.keyword_input.as_ref().map(|i| div().w(px(180.0)).child(Input::new(i).font_family(FONT_MONO).bg(BG_APP).border_color(BORDER_DANGER_BTN)))),
            )
            .children(crate::components::english_beside::english_beside(t_beside!("plugin-page-type-to-confirm", keyword = a.keyword.clone().unwrap_or_else(|| "CONFIRM".into()))))
        })
        .children(pending.error.clone().map(|e| div().text_size(px(10.5)).text_color(CRIT).child(e)))
        .child(
            div()
                .flex()
                .gap(px(8.0))
                .child(
                    div()
                        .id("btn-plugin-confirm")
                        .px(px(12.0))
                        .py(px(5.0))
                        .bg(color)
                        .text_color(hex_rgb(0x0a0a0c))
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .on_click(move |_ev, _w, cx| app_run.update(cx, |this, cx| this.confirm_plugin_run(cx)))
                        .child(t!("plugin-page-run", label = a.label.to_uppercase())),
                )
                .child(
                    div()
                        .id("btn-plugin-cancel")
                        .px(px(12.0))
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_SECONDARY)
                        .text_size(px(10.5))
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _w, cx| app_cancel.update(cx, |this, cx| this.cancel_plugin_run(cx)))
                        .child(t!("plugin-page-cancel")),
                ),
        )
}

pub fn plugin_page(d: PageData, app: Entity<CrowApp>) -> impl IntoElement {
    let app_refresh = app.clone();
    let m = d.plugin.report.manifest.as_ref();
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .font_family(FONT_MONO)
        .child(
            div()
                .h(px(40.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(d.page.label.to_uppercase()))
                .child(div().text_size(px(10.0)).text_color(TEXT_FAINT).child(t!("plugin-page-from", name = d.plugin.name(), version = m.map(|m| m.plugin.version.clone()).unwrap_or_default())))
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-plugin-refresh")
                        .px(px(10.0))
                        .py(px(5.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_size(px(10.5))
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _w, cx| app_refresh.update(cx, |this, cx| this.refresh_plugin_page(cx)))
                        .child(t!("plugin-page-refresh")),
                ),
        )
        .children(d.needs_install.then(|| {
            let (app_install, app_console, id) = (app.clone(), app.clone(), d.plugin.id.clone());
            let slot = d.state.slot(&d.plugin.id, crate::app::user_plugins::SETUP, crate::app::user_plugins::HERE);
            let running = slot.is_some_and(|s| s.running);
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .px(px(14.0))
                .py(px(8.0))
                .bg(WARN_BG)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(div().flex_1().text_size(px(10.5)).text_color(WARN_INK).child(t!("plugin-page-needs-install")))
                        .child(
                            div()
                                .id("btn-plugin-install-page")
                                .px(px(10.0))
                                .py(px(4.0))
                                .border_1()
                                .border_color(WARN)
                                .text_color(WARN)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .when(!running, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _w, cx| app_install.update(cx, |this, cx| this.run_plugin_setup(&id, cx))))
                                .child(if running { t!("settings-user-plugins-installing") } else { t!("settings-user-plugins-install") }),
                        ),
                )
                .children(slot.map(|s| console(&d.plugin.id, crate::app::user_plugins::SETUP, crate::app::user_plugins::HERE, s, app_console.clone())))
        }))
        .child(
            div()
                .id("plugin-page-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .children(d.page.blocks.iter().enumerate().map(|(i, b)| block_view(i, b, &d, &app))),
        )
}
