//! Fleet Setup → ROLLOUTS (ERR-142): a command or a config file across a
//! group, with each server's diff before anything runs.

use gpui_kit::component::input::{Editor, Input};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::patching::Target;
use crate::app::rollouts::{candidates, Current, Kind, RolloutInputs, RolloutState};
use crate::app::CrowApp;
use crate::rollout::{diff_stats, hunks, line_diff, DiffKind};
use crate::theme::*;
use crate::views::fleet::FleetState;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn label(text: &'static str) -> Div {
    mono(9.5, TEXT_DIMMER).font_weight(FontWeight::SEMIBOLD).child(text)
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

pub fn rollouts_page(fleet: &FleetState, st: &RolloutState, inputs: Option<&RolloutInputs>, baselines: &[crate::vault::ConfigBaseline], new_content: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let Some(inputs) = inputs else { return div().into_any_element() };
    let (servers, left_out) = candidates(&fleet.servers, &st.target, |s| fleet.health(s).is_ok());
    let canary = if st.use_canary { st.canary.clone().filter(|c| servers.iter().any(|s| &s.id == c)).or_else(|| servers.first().map(|s| s.id.clone())) } else { None };
    let (app_cmd, app_cfg, app_plan, app_root, app_read) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone());

    let form = div()
        .w(px(520.0))
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(mono(10.0, TEXT_FAINT).line_height(px(14.0)).child("One change across many servers: the canary first (the run pauses so you can check it), then the rest in batches that run side by side. The first failure stops everything, with that server's output."))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label("WHAT"))
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .child(chip("rollout-kind-cmd", "A COMMAND", st.kind == Kind::Command).on_click(move |_ev, _w, cx| app_cmd.update(cx, |this, cx| this.set_rollout_kind(Kind::Command, cx))))
                        .child(chip("rollout-kind-cfg", "A CONFIG FILE", st.kind == Kind::Config).on_click(move |_ev, _w, cx| app_cfg.update(cx, |this, cx| this.set_rollout_kind(Kind::Config, cx)))),
                ),
        )
        .when(st.kind == Kind::Command, |d| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(div().h(px(150.0)).border_1().border_color(BORDER_DEFAULT).bg(BG_APP).child(Editor::new(&inputs.script).h_full().appearance(false).bordered(false).font_family(FONT_MONO).text_size(px(11.0))))
                    .child(
                        div()
                            .id("rollout-as-root")
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .on_click(move |_ev, _w, cx| app_root.update(cx, |this, cx| this.toggle_rollout_root(cx)))
                            .child(div().size(px(12.0)).border_1().border_color(if st.as_root { WARN } else { BORDER_STRONG }).bg(if st.as_root { WARN } else { BG_CONTROL }))
                            .child(mono(10.5, TEXT_SECONDARY).child("as root (through sudo -n)")),
                    ),
            )
        })
        .when(st.kind == Kind::Config, |d| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(label("PATH (ENTER READS EVERY SERVER'S COPY)"))
                    .child(div().flex().gap(px(6.0)).child(div().flex_1().child(Input::new(&inputs.path).font_family(FONT_MONO).text_size(px(11.0)).bg(BG_APP).rounded(px(2.0)))).child(chip("rollout-read", "READ", false).on_click(move |_ev, _w, cx| app_read.update(cx, |this, cx| this.read_rollout_preview(cx)))))
                    .when(!baselines.is_empty(), |d| {
                        d.child(div().flex().flex_wrap().gap(px(4.0)).child(mono(9.5, TEXT_FAINT).mr(px(4.0)).child("from a baseline:")).children(baselines.iter().enumerate().map(|(i, b)| {
                            let (app, path, rev) = (app.clone(), b.path.clone(), b.revision_id.clone());
                            chip(format!("rollout-baseline-{i}"), format!("{} ({})", b.path, b.scope), false).on_click(move |_ev, window, cx| {
                                let (path, rev) = (path.clone(), rev.clone());
                                app.update(cx, |this, cx| this.rollout_from_baseline(&path, &rev, window, cx))
                            })
                        })))
                    })
                    .child(label("NEW CONTENT"))
                    .child(div().h(px(260.0)).border_1().border_color(BORDER_DEFAULT).bg(BG_APP).child(Editor::new(&inputs.content).h_full().appearance(false).bordered(false).font_family(FONT_MONO).text_size(px(11.0))))
                    .child(mono(9.5, TEXT_FAINT).line_height(px(13.0)).child("Each server checks it with the format's own validator first (sshd -t, nginx -t, visudo, fstab…); refused content isn't written. Every write is kept in that server's config history.")),
            )
        })
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label("WHERE"))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .child({
                            let app = app.clone();
                            chip("rollout-target-all", "every server", st.target == Target::All).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_rollout_target(Target::All, cx)))
                        })
                        .children(fleet.groups.iter().map(|g| {
                            let (app, name) = (app.clone(), g.clone());
                            chip(format!("rollout-target-{g}"), format!("group {g}"), st.target == Target::Group(g.clone())).on_click(move |_ev, _w, cx| {
                                let name = name.clone();
                                app.update(cx, |this, cx| this.set_rollout_target(Target::Group(name), cx))
                            })
                        })),
                )
                .child(mono(10.0, TEXT_DIM).child(format!("{} server{}{}", servers.len(), if servers.len() == 1 { "" } else { "s" }, if left_out.is_empty() { String::new() } else { format!(" · {} left out (unreachable, or the machine Crow runs on)", left_out.len()) }))),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label("CANARY · RUNS FIRST, ALONE"))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(6.0))
                        .child({
                            let app = app.clone();
                            chip("rollout-canary-none", "no canary", !st.use_canary).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_rollout_canary(None, cx)))
                        })
                        .children(servers.iter().map(|s| {
                            let (app, id) = (app.clone(), s.id.clone());
                            chip(format!("rollout-canary-{}", s.id), s.name.clone(), canary.as_deref() == Some(s.id.as_str())).on_click(move |_ev, _w, cx| {
                                let id = id.clone();
                                app.update(cx, |this, cx| this.set_rollout_canary(Some(id), cx))
                            })
                        })),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(label("THEN, AT A TIME"))
                .child(div().flex().gap(px(6.0)).children([1usize, 2, 5, 10].into_iter().map(|n| {
                    let app = app.clone();
                    chip(format!("rollout-batch-{n}"), n.to_string(), st.batch == n).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.set_rollout_batch(n, cx)))
                }))),
        )
        .children(st.message.clone().map(|m| mono(10.5, WARN).line_height(px(15.0)).child(m)))
        .child(
            div()
                .id("btn-rollout-plan")
                .flex()
                .items_center()
                .justify_center()
                .h(px(30.0))
                .border_1()
                .border_color(WARN)
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(WARN)
                .cursor_pointer()
                .hover(|s| s.bg(WARN.opacity(0.12)))
                .on_click(move |_ev, window, cx| app_plan.update(cx, |this, cx| this.plan_rollout(window, cx)))
                .child("PREVIEW THE RUN →"),
        );

    // The selected server's diff.
    let preview = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .when(st.kind == Kind::Command, |d| d.child(mono(10.5, TEXT_FAINT).child("A command has no preview: try it on the canary first.")))
        .when(st.kind == Kind::Config, |d| {
            d.child(label("WHAT CHANGES, PER SERVER"))
                .when(st.preview_path.is_empty(), |d| d.child(mono(10.5, TEXT_FAINT).child("Type the path and press Enter to read each server's copy.")))
                .child(div().flex().flex_wrap().gap(px(4.0)).children(servers.iter().map(|s| {
                    let stat = match st.current.get(&s.id) {
                        Some(Current::Read(now)) => {
                            let (a, r) = diff_stats(&line_diff(now, new_content));
                            if a + r == 0 { "same".to_string() } else { format!("+{a} −{r}") }
                        }
                        Some(Current::Missing) => "new file".into(),
                        Some(Current::Failed(_)) => "unreadable".into(),
                        Some(Current::Reading) => "reading…".into(),
                        None => "—".into(),
                    };
                    let (app, id) = (app.clone(), s.id.clone());
                    chip(format!("rollout-preview-{}", s.id), format!("{} · {stat}", s.name), st.selected.as_deref() == Some(s.id.as_str())).on_click(move |_ev, _w, cx| {
                        let id = id.clone();
                        app.update(cx, |this, cx| this.select_rollout_preview(&id, cx))
                    })
                })))
                .children(st.selected.as_ref().and_then(|id| st.current.get(id)).map(|c| match c {
                    Current::Read(now) => {
                        let diff = line_diff(now, new_content);
                        let shown = hunks(&diff, 2);
                        div()
                            .id("rollout-diff")
                            .max_h(px(520.0))
                            .overflow_y_scrollbar()
                            .p(px(8.0))
                            .bg(BG_PANEL)
                            .border_1()
                            .border_color(BORDER_PANEL)
                            .when(diff_stats(&diff) == (0, 0), |d| d.child(mono(10.5, TEXT_DIM).child("Identical: this server is left out.")))
                            .children(shown.into_iter().map(|l| match l {
                                None => mono(10.0, TEXT_GHOST).child("⋯"),
                                Some(l) => {
                                    let (sign, color, bg) = match l.kind {
                                        DiffKind::Added => ("+", OK, OK.opacity(0.08)),
                                        DiffKind::Removed => ("−", CRIT, CRIT.opacity(0.08)),
                                        DiffKind::Same => (" ", TEXT_DIM, hex_rgba(0, 0.0)),
                                    };
                                    mono(10.5, color).bg(bg).whitespace_nowrap().child(format!("{sign} {}", l.text))
                                }
                            }))
                            .into_any_element()
                    }
                    Current::Missing => mono(10.5, TEXT_DIM).child("No such file there yet: it will be created.").into_any_element(),
                    Current::Failed(e) => mono(10.5, CRIT).child(format!("Couldn't read it: {e}")).into_any_element(),
                    Current::Reading => mono(10.5, TEXT_DIM).child("Reading…").into_any_element(),
                }))
                .children(st.selected.as_ref().and_then(|id| match st.current.get(id) {
                    Some(Current::Read(now)) => Some(now.clone()),
                    _ => None,
                }).map(|now| {
                    let app = app.clone();
                    chip("rollout-start-from", "START FROM THIS SERVER'S COPY", false).on_click(move |_ev, window, cx| {
                        let now = now.clone();
                        app.update(cx, |this, cx| this.rollout_content_from(now, window, cx))
                    })
                }))
        });

    div()
        .id("rollouts-page")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .child(div().p(px(18.0)).flex().gap(px(24.0)).child(form).child(preview))
        .into_any_element()
}
