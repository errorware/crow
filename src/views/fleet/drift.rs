//! Fleet Setup → DRIFT (ERR-143): which servers' baselined files drifted,
//! what changed, bring them back or accept them; and a search through
//! every config file Crow has read, on every server.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::drift::DriftPageState;
use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::config::drift::{Drift, DriftEntry};
use crate::rollout::{hunks, line_diff, DiffKind};
use crate::theme::*;
use crate::vault::BASELINE_FLEET;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn section(title: &'static str, note: impl Into<SharedString>) -> Div {
    div().flex().items_baseline().gap(px(10.0)).child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(title)).child(mono(9.5, TEXT_FAINT).child(note.into()))
}

fn action(id: impl Into<SharedString>, text: impl Into<SharedString>, color: Rgba) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .h(px(24.0))
        .px(px(10.0))
        .border_1()
        .border_color(color.opacity(0.6))
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .cursor_pointer()
        .hover(move |s| s.bg(color.opacity(0.12)))
        .child(text.into())
}

fn ago(ts: i64) -> String {
    if ts == 0 {
        return "not yet".into();
    }
    let s = (chrono::Utc::now().timestamp() - ts).max(0);
    match s {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", s / 60),
        _ => format!("{} h ago", s / 3600),
    }
}

/// `selected_texts`: (baseline, server copy) of the selected entry.
pub fn drift_page(fleet: &FleetState, st: &DriftPageState, selected_texts: Option<(String, String)>, app: Entity<CrowApp>) -> impl IntoElement {
    let entries: Vec<&DriftEntry> = fleet.drift.iter().flatten().collect();
    let name = |id: &str| fleet.servers.iter().find(|s| s.id == id).map(|s| s.name.clone()).unwrap_or_else(|| id.to_string());
    let mut paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    paths.sort();
    paths.dedup();
    let drifted = entries.iter().filter(|e| e.drift.is_drift()).count();
    let app_check = app.clone();

    let search = div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(section("SEARCH EVERY SERVER'S CONFIGS", "from what Crow has read; no server is contacted"))
        .children(st.search.as_ref().map(|i| div().max_w(px(640.0)).child(Input::new(i).font_family(FONT_MONO).text_size(px(11.5)).bg(BG_APP).rounded(px(2.0)))))
        .when(st.searched.len() >= 2, |d| {
            d.child(mono(9.5, TEXT_FAINT).child(match st.hits.len() {
                0 => "Nothing found. Crow searches the files it has read: open a server's Config screen to read its files.".to_string(),
                n if n >= crate::app::drift::MAX_HITS => format!("first {n} matches"),
                n => format!("{n} match{}", if n == 1 { "" } else { "es" }),
            }))
            .child(div().flex().flex_col().max_h(px(320.0)).id("drift-search-hits").overflow_y_scrollbar().children(st.hits.iter().enumerate().map(|(i, h)| {
                let (app, hit) = (app.clone(), h.clone());
                div()
                    .id(ElementId::NamedInteger("drift-hit".into(), i as u64))
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .px(px(8.0))
                    .py(px(3.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(BG_ROW_HOVER))
                    .on_click(move |_ev, _w, cx| {
                        let hit = hit.clone();
                        app.update(cx, |this, cx| this.open_search_hit(&hit, cx))
                    })
                    .child(mono(10.5, TEXT_PRIMARY).w(px(140.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(h.server.clone()))
                    .child(mono(10.5, TEXT_TERTIARY).w(px(260.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(h.path.clone()))
                    .child(match &h.line {
                        Some((n, l)) => mono(10.5, TEXT_SECONDARY).flex_1().min_w(px(0.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(format!("{n}: {l}")),
                        None => mono(10.0, TEXT_FAINT).flex_1().child("the file"),
                    })
            })))
        });

    let table = div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .child(section("DRIFT FROM BASELINES", format!("{drifted} drifted · files re-read every 15 min while Crow runs · checked {}", ago(st.last_check))))
                .child(div().flex_1())
                .child(
                    div()
                        .id("btn-drift-check")
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .h(px(24.0))
                        .px(px(10.0))
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _w, cx| app_check.update(cx, |this, cx| this.check_drift(true, cx)))
                        .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                        .child(if st.checking.is_empty() { "CHECK NOW".to_string() } else { format!("READING {}…", st.checking.len()) }),
                ),
        )
        .children(st.message.clone().map(|m| mono(10.5, WARN).line_height(px(15.0)).child(m)))
        .when(paths.is_empty(), |d| d.child(mono(10.5, TEXT_DIM).line_height(px(15.0)).child("No baselines pinned yet. Pin one from a server's Config screen (a file's history → PIN AS BASELINE); every server in that group, or the fleet, is then compared with it.")))
        .children(paths.into_iter().map(|path| {
            let rows: Vec<&&DriftEntry> = entries.iter().filter(|e| e.path == path).collect();
            let n_drift = rows.iter().filter(|e| e.drift.is_drift()).count();
            let app_all = app.clone();
            let p = path.to_string();
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(10.0))
                        .h(px(30.0))
                        .bg(BG_SUBHEAD)
                        .child(mono(11.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(path.to_string()))
                        .child(mono(9.5, if n_drift > 0 { WARN } else { OK }).child(if n_drift > 0 { format!("{n_drift} drifted") } else { "all match".to_string() }))
                        .child(div().flex_1())
                        .when(n_drift > 1, |d| {
                            d.child(action(format!("drift-all-{path}"), format!("BRING ALL {n_drift} BACK"), WARN).on_click(move |_ev, window, cx| {
                                let p = p.clone();
                                app_all.update(cx, |this, cx| this.plan_bring_back(&p, None, window, cx))
                            }))
                        }),
                )
                .children(rows.into_iter().map(|e| {
                    let selected = st.selected.as_ref().is_some_and(|(s, p)| *s == e.server_id && *p == e.path);
                    let (text, color) = match &e.drift {
                        Drift::Identical => ("matches".to_string(), OK),
                        Drift::Cosmetic => ("matches (comments/spacing differ)".to_string(), OK),
                        Drift::Differs(d) => (format!("{} difference{}: {}", d.len(), if d.len() == 1 { "" } else { "s" }, d.first().cloned().unwrap_or_default()), WARN),
                    };
                    let scope = if e.scope == BASELINE_FLEET { "fleet".to_string() } else { e.scope.clone() };
                    let (app, sid, p) = (app.clone(), e.server_id.clone(), e.path.clone());
                    div()
                        .id(ElementId::Name(format!("drift-row-{}-{}", e.server_id, e.path).into()))
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(10.0))
                        .min_h(px(28.0))
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .bg(if selected { BG_ROW_SELECTED } else { BG_PANEL })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _w, cx| {
                            let (sid, p) = (sid.clone(), p.clone());
                            app.update(cx, |this, cx| this.select_drift(&sid, &p, cx))
                        })
                        .child(div().size(px(7.0)).rounded_full().bg(color))
                        .child(mono(11.0, TEXT_PRIMARY).w(px(160.0)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(name(&e.server_id)))
                        .child(mono(10.5, color).flex_1().min_w(px(0.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(text))
                        .child(mono(9.5, TEXT_FAINT).w(px(110.0)).flex_none().child(format!("vs {scope} baseline")))
                }))
        }));

    // The selected entry: its diff and what to do about it.
    let detail = st.selected.as_ref().and_then(|(sid, path)| entries.iter().find(|e| &e.server_id == sid && &e.path == path).map(|e| (*e, sid.clone(), path.clone()))).map(|(e, sid, path)| {
        let (app_back, app_accept) = (app.clone(), app.clone());
        let (sid2, path2) = (sid.clone(), path.clone());
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(section("WHAT CHANGED", format!("{} · {} · baseline (−) vs server (+)", name(&sid), path)))
            .child(match &selected_texts {
                Some((base, now)) => {
                    let diff = line_diff(base, now);
                    div()
                        .id("drift-diff")
                        .max_h(px(380.0))
                        .overflow_y_scrollbar()
                        .p(px(8.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .children(hunks(&diff, 2).into_iter().map(|l| match l {
                            None => mono(10.0, TEXT_GHOST).child("⋯"),
                            Some(l) => {
                                let (sign, color, bg) = match l.kind {
                                    DiffKind::Added => ("+", WARN, WARN.opacity(0.08)),
                                    DiffKind::Removed => ("−", OK, OK.opacity(0.06)),
                                    DiffKind::Same => (" ", TEXT_DIM, hex_rgba(0, 0.0)),
                                };
                                mono(10.5, color).bg(bg).whitespace_nowrap().child(format!("{sign} {}", l.text))
                            }
                        }))
                        .into_any_element()
                }
                None => mono(10.5, TEXT_DIM).line_height(px(15.0)).child("Crow kept only hashes of one of the two: file contents are sealed with the vault's key, and there was none when it was read (or the vault is locked now). It can tell that they differ, not how, and can't push the baseline or pin this copy. Set up the vault (Settings → Vault & Security), then CHECK NOW.").into_any_element(),
            })
            .when(e.drift.is_drift() && selected_texts.is_some(), |d| {
                d.child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(action("drift-bring-back", "BRING BACK TO BASELINE", OK).on_click(move |_ev, window, cx| {
                            let (sid, path) = (sid.clone(), path.clone());
                            app_back.update(cx, |this, cx| this.plan_bring_back(&path, Some(&sid), window, cx))
                        }))
                        .child(action("drift-accept", "ACCEPT THIS VERSION AS THE BASELINE", WARN).on_click(move |_ev, _w, cx| {
                            let (sid, path) = (sid2.clone(), path2.clone());
                            app_accept.update(cx, |this, cx| this.accept_drift(&sid, &path, cx))
                        })),
                )
                .child(mono(9.5, TEXT_FAINT).line_height(px(13.0)).child("Bring back writes the baseline over this server's copy (checked by its validator, kept in history). Accept makes this copy the baseline: every other server in its scope is then compared with it."))
            })
    });

    div()
        .id("drift-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(24.0))
                .child(search)
                .child(div().flex().gap(px(20.0)).child(div().flex_1().min_w(px(0.0)).child(table)).children(detail.map(|d| div().w(px(560.0)).flex_none().child(d)))),
        )
}
