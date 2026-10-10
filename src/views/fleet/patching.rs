//! Fleet Setup → PATCHING (ERR-141): who's behind on updates, each group's
//! maintenance window, and the runs that apply updates and reboot.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::patching::{PatchingState, Target};
use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::patching::Scope;
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn section(title: &'static str, note: impl Into<SharedString>) -> Div {
    div().flex().items_baseline().gap(px(10.0)).child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child(title)).child(mono(9.5, TEXT_FAINT).child(note.into()))
}

fn chip(id: impl Into<SharedString>, text: impl Into<SharedString>, on: bool) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .px(px(9.0))
        .py(px(4.0))
        .border_1()
        .border_color(if on { TEXT_PRIMARY } else { BORDER_DEFAULT })
        .bg(if on { BG_CONTROL_ALT } else { BG_CONTROL })
        .font_family(FONT_MONO)
        .text_size(px(10.0))
        .font_weight(if on { FontWeight::BOLD } else { FontWeight::NORMAL })
        .text_color(if on { TEXT_PRIMARY } else { TEXT_DIM })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
        .child(text.into())
}

fn action(id: &'static str, text: &'static str, color: Rgba) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .h(px(28.0))
        .px(px(12.0))
        .border_1()
        .border_color(color.opacity(0.6))
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .cursor_pointer()
        .hover(move |s| s.bg(color.opacity(0.12)))
        .child(text)
}

fn ago(ts: i64) -> String {
    if ts == 0 {
        return "never".into();
    }
    let s = (chrono::Utc::now().timestamp() - ts).max(0);
    match s {
        0..=59 => t!("fleet-patching-just-now").into(),
        60..=3599 => t!("fleet-patching-min-ago", s = s / 60),
        3600..=86_399 => t!("fleet-patching-h-ago", s = s / 3600),
        _ => t!("fleet-patching-d-ago", s = s / 86_400),
    }
}

const W_GROUP: f32 = 120.0;
const W_MGR: f32 = 60.0;
const W_NUM: f32 = 80.0;
const W_KERNEL: f32 = 200.0;
const W_REBOOT: f32 = 80.0;
const W_WHEN: f32 = 90.0;

pub fn patching_page(fleet: &FleetState, st: &PatchingState, app: Entity<CrowApp>) -> impl IntoElement {
    let scanning = !st.scanning.is_empty();
    let in_target: Vec<_> = fleet.servers.iter().filter(|s| st.target.includes(s)).collect();
    let behind = in_target.iter().filter(|s| st.summaries.get(&s.id).is_some_and(|x| !x.security.is_empty())).count();
    let reboots = in_target.iter().filter(|s| st.summaries.get(&s.id).is_some_and(|x| x.reboot_required)).count();
    let (app_refresh, app_sec, app_all, app_reboot, app_toggle) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone());
    let mut rows: Vec<_> = fleet.servers.iter().collect();
    // The most behind first.
    rows.sort_by_key(|s| {
        let x = st.summaries.get(&s.id);
        (std::cmp::Reverse(x.map_or(0, |x| x.security.len())), std::cmp::Reverse(x.map_or(0, |x| x.total())), s.name.clone())
    });
    div()
        .id("patching-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(
            div()
                .max_w(px(1100.0))
                .p(px(18.0))
                .flex()
                .flex_col()
                .gap(px(22.0))
                // Run: which servers, what, and the guardrails.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .p(px(14.0))
                        .bg(BG_PANEL)
                        .border_1()
                        .border_color(BORDER_PANEL)
                        .child(section("APPLY", t!("fleet-patching-of-behind-on-security-updates", behind = behind, count = in_target.len(), reboots = reboots)))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(6.0))
                                .child({
                                    let app = app.clone();
                                    chip("patch-target-all", t!("fleet-patching-every-server"), st.target == Target::All).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_patch_target(Target::All, cx)))
                                })
                                .children(fleet.groups.iter().map(|g| {
                                    let (app, name) = (app.clone(), g.clone());
                                    chip(format!("patch-target-{g}"), t!("fleet-patching-group", g = g), st.target == Target::Group(g.clone())).on_click(move |_ev, _w, cx| {
                                        let name = name.clone();
                                        app.update(cx, |this, cx| this.set_patch_target(Target::Group(name), cx))
                                    })
                                })),
                        )
                        .child(
                            div()
                                .id("patch-reboot-after")
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .cursor_pointer()
                                .on_click(move |_ev, _w, cx| app_toggle.update(cx, |this, cx| this.toggle_reboot_after(cx)))
                                .child(div().size(px(12.0)).border_1().border_color(if st.reboot_after { OK } else { BORDER_STRONG }).bg(if st.reboot_after { OK } else { BG_CONTROL }))
                                .child(mono(10.5, TEXT_SECONDARY).child(t!("fleet-patching-then-reboot-servers-that-need"))),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(8.0))
                                .child(action("btn-patch-security", "APPLY SECURITY UPDATES", OK).on_click(move |_ev, window, cx| app_sec.update(cx, |this, cx| this.plan_patch(Scope::Security, window, cx))))
                                .child(action("btn-patch-all", "APPLY ALL UPDATES", WARN).on_click(move |_ev, window, cx| app_all.update(cx, |this, cx| this.plan_patch(Scope::All, window, cx))))
                                .child(action("btn-patch-reboot", "REBOOT WHERE NEEDED", CRIT).on_click(move |_ev, window, cx| app_reboot.update(cx, |this, cx| this.plan_reboot_needed(window, cx)))),
                        )
                        .child(mono(9.5, TEXT_FAINT).line_height(px(14.0)).child(
                            t!("fleet-patching-each-run-lists-the-exact"),
                        )),
                )
                // Who's behind.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .child(section(t!("fleet-patching-whos-behind"), t!("fleet-patching-read-every-6-h-while", ago = ago(st.last_scan))))
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .id("btn-patch-refresh")
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
                                        .on_click(move |_ev, _w, cx| app_refresh.update(cx, |this, cx| this.scan_fleet_updates(true, cx)))
                                        .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                                        .child(if scanning { t!("fleet-patching-reading", count = st.scanning.len()) } else { t!("fleet-patching-refresh").to_string() }),
                                ),
                        )
                        .child({
                            let head = |w: Option<f32>, t: &'static str| {
                                let d = mono(9.0, TEXT_FAINT).font_weight(FontWeight::SEMIBOLD).child(t);
                                match w {
                                    Some(w) => d.w(px(w)).flex_none(),
                                    None => d.flex_1().min_w(px(0.0)),
                                }
                            };
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .h(px(24.0))
                                .px(px(10.0))
                                .bg(BG_SUBHEAD)
                                .border_1()
                                .border_color(BORDER_PANEL)
                                .child(head(None, t!("fleet-patching-server")))
                                .child(head(Some(W_GROUP), t!("fleet-patching-group-2")))
                                .child(head(Some(W_MGR), t!("fleet-patching-via")))
                                .child(head(Some(W_NUM), t!("fleet-patching-security")))
                                .child(head(Some(W_NUM), t!("fleet-patching-all")))
                                .child(head(Some(W_KERNEL), t!("fleet-patching-kernel")))
                                .child(head(Some(W_REBOOT), "REBOOT"))
                                .child(head(Some(W_WHEN), t!("fleet-patching-read")))
                        })
                        .children(rows.into_iter().map(|s| {
                            let x = st.summaries.get(&s.id);
                            let reading = st.scanning.contains(&s.id);
                            let dash = || mono(10.5, TEXT_GHOST).child("—");
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .min_h(px(28.0))
                                .px(px(10.0))
                                .bg(BG_PANEL)
                                .border_b_1()
                                .border_l_1()
                                .border_r_1()
                                .border_color(BORDER_PANEL)
                                .child(mono(11.0, TEXT_PRIMARY).flex_1().min_w(px(0.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(s.name.clone()))
                                .child(mono(10.0, TEXT_DIM).w(px(W_GROUP)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(if s.group_name.trim().is_empty() { "—".to_string() } else { s.group_name.trim().to_string() }))
                                .child(match x {
                                    _ if reading => mono(10.0, TEXT_DIM).flex_1().child(t!("fleet-patching-reading-2")).into_any_element(),
                                    None => mono(10.0, TEXT_FAINT).flex_1().child(t!("fleet-patching-not-read-yet")).into_any_element(),
                                    Some(x) if x.error.is_some() => mono(10.0, CRIT).flex_1().min_w(px(0.0)).whitespace_nowrap().overflow_hidden().text_ellipsis().child(t!("fleet-patching-couldnt-read", error = x.error.clone().unwrap_or_default())).into_any_element(),
                                    Some(x) => div()
                                        .flex()
                                        .items_center()
                                        .gap(px(10.0))
                                        .child(mono(10.0, TEXT_DIM).w(px(W_MGR)).flex_none().child(if x.manager.is_empty() { "—".to_string() } else { x.manager.clone() }))
                                        .child(mono(11.0, if x.security.is_empty() { TEXT_DIM } else { CRIT }).font_weight(if x.security.is_empty() { FontWeight::NORMAL } else { FontWeight::BOLD }).w(px(W_NUM)).flex_none().child(x.security.len().to_string()))
                                        .child(mono(11.0, if x.total() == 0 { TEXT_DIM } else { WARN }).w(px(W_NUM)).flex_none().child(x.total().to_string()))
                                        .child(match &x.kernel {
                                            Some(k) => mono(10.0, TEXT_SECONDARY).w(px(W_KERNEL)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(k.clone()).into_any_element(),
                                            None => dash().w(px(W_KERNEL)).flex_none().into_any_element(),
                                        })
                                        .child(if x.reboot_required { mono(10.0, WARN).font_weight(FontWeight::BOLD).w(px(W_REBOOT)).flex_none().child(t!("fleet-patching-needed")).into_any_element() } else { dash().w(px(W_REBOOT)).flex_none().into_any_element() })
                                        .child(mono(9.5, TEXT_FAINT).w(px(W_WHEN)).flex_none().child(ago(x.checked_at)))
                                        .into_any_element(),
                                })
                        })),
                )
                // Windows.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(section(t!("fleet-patching-maintenance-windows"), t!("fleet-patching-per-group-local-time-a")))
                        .when(fleet.groups.is_empty(), |d| d.child(mono(10.5, TEXT_DIM).child(t!("fleet-patching-no-groups-yet-windows-are"))))
                        .children(fleet.groups.iter().map(|g| {
                            let window = st.windows.get(g);
                            let open_now = window.map(|w| w.contains_now());
                            let members = fleet.servers.iter().filter(|s| s.group_name.trim() == g).count();
                            div()
                                .flex()
                                .items_center()
                                .gap(px(12.0))
                                .child(mono(11.0, TEXT_PRIMARY).w(px(160.0)).flex_none().child(format!("{g} ({members})")))
                                .child(div().w(px(240.0)).flex_none().children(st.window_inputs.get(g).map(|i| Input::new(i).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0)))))
                                .child(match (st.window_errors.get(g), open_now) {
                                    (Some(e), _) => mono(10.0, CRIT).child(e.clone()),
                                    (None, Some(true)) => mono(10.0, OK).child(t!("fleet-patching-open-now")),
                                    (None, Some(false)) => mono(10.0, TEXT_DIM).child(t!("fleet-patching-closed-now")),
                                    (None, None) => mono(10.0, TEXT_FAINT).child(t!("fleet-patching-any-time")),
                                })
                        }))
                        .child(mono(9.5, TEXT_FAINT).child(t!("fleet-patching-examples-sun-02-05-mon"))),
                ),
        )
}
