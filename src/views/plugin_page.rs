//! A user plugin's page on a server (ERR-178), drawn only from blocks Crow
//! already has: a status, key and value pairs, a table, buttons, the last
//! run's output, and the run history. Stage 4 runs read actions only.

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::user_plugins::{RunSlot, UserPluginsState};
use crate::app::CrowApp;
use crate::plugin_host::folder::Installed;
use crate::plugin_host::manifest::{Action, Block, BlockKind, Effect, Page};
use crate::plugin_host::run::{self, State};
use crate::theme::*;
use crate::vault::PluginRun;

pub struct PageData<'a> {
    pub plugin: &'a Installed,
    pub page: &'a Page,
    pub server_id: &'a str,
    pub state: &'a UserPluginsState,
    pub runs: Vec<PluginRun>,
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
            panel()
                .child(div().flex().flex_wrap().gap(px(8.0)).children(buttons))
                .children(shown.and_then(|(_, a, _)| d.state.slot(&d.plugin.id, a, d.server_id)).map(console))
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
                        .child(div().min_w(px(0.0)).truncate().text_color(TEXT_DIM).child(r.summary.clone()))
                }))
                .into_any_element()
        }
    }
}

fn action_button(n: usize, a: &Action, d: &PageData, app: Entity<CrowApp>) -> impl IntoElement {
    let running = d.state.slot(&d.plugin.id, &a.id, d.server_id).is_some_and(|s| s.running);
    let usable = a.effect == Effect::Read;
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
        .when(usable && !running, |d| d.cursor_pointer().hover(|s| s.bg(BG_ROW_HOVER)).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.run_plugin_action(&plugin, &action, false, cx))))
        .child(if running { t!("plugin-page-running", label = a.label) } else { a.label.to_uppercase() })
        .when(!usable, |d| d.tooltip(|window, cx| gpui_kit::component::tooltip::Tooltip::new(t!("app-user-plugins-later-stage")).build(window, cx)))
}

/// The last run's output, as a console.
fn console(slot: &RunSlot) -> impl IntoElement {
    let Some(r) = slot.last.as_ref() else {
        return div().text_size(px(10.5)).text_color(TEXT_DIMMER).child(t!("plugin-page-reading"));
    };
    let mut text = r.stdout.clone();
    if !r.stderr.trim().is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&r.stderr);
    }
    let status = match (&r.error, r.exit_code) {
        (Some(e), _) => e.clone(),
        (None, Some(c)) => t!("plugin-page-exit", code = c),
        (None, None) => String::new(),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(div().text_size(px(9.5)).text_color(if r.ok { OK } else { CRIT }).child(status))
        .child(
            div()
                .id("plugin-console")
                .max_h(px(320.0))
                .overflow_y_scrollbar()
                .p(px(8.0))
                .bg(BG_WINDOW)
                .border_1()
                .border_color(BORDER_ROW)
                .text_size(px(10.5))
                .text_color(TEXT_PRIMARY)
                .children(text.lines().take(2000).map(|l| div().child(l.to_string())).collect::<Vec<_>>()),
        )
        .when(r.truncated, |d| d.child(div().text_size(px(9.5)).text_color(WARN).child(t!("plugin-page-truncated"))))
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
