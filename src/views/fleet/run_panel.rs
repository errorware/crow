//! The fleet runner's panel (ERR-79): the plan, the keyword, then each
//! host's result as it happens.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use super::run::{RunPhase, StepStatus};
use crate::app::fleet_run::FleetRunner;
use crate::app::CrowApp;
use crate::theme::*;

pub fn fleet_run_panel(runner: &FleetRunner, app: Entity<CrowApp>) -> impl IntoElement {
    let run = &runner.run;
    let button = |id: &'static str, label: String, color: Rgba| {
        div()
            .id(id)
            .px(px(12.0))
            .py(px(5.0))
            .border_1()
            .border_color(color)
            .text_color(color)
            .font_weight(FontWeight::BOLD)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
            .child(label)
    };
    let status_line = match run.phase {
        RunPhase::Confirming if run.steps.is_empty() => "Nothing to run.".to_string(),
        RunPhase::Confirming if !run.stages.is_empty() => {
            let groups = run.stage_groups();
            format!("{} step{} in {} stage{}{}; the first failure stops the run.", run.steps.len(), if run.steps.len() == 1 { "" } else { "s" }, groups.len(), if groups.len() == 1 { "" } else { "s" }, if run.pause_after.is_some() { ", pausing after the canary" } else { "" })
        }
        RunPhase::Confirming => format!("{} step{}, one host at a time; the first failure stops the run.", run.steps.len(), if run.steps.len() == 1 { "" } else { "s" }),
        RunPhase::Paused => "The canary is done. Check it, then CONTINUE with the rest, or STOP here.".to_string(),
        RunPhase::Running if run.stop_requested() => "Stopping after the current host…".to_string(),
        RunPhase::Running => "Running…".to_string(),
        RunPhase::Finished => run.summary(),
    };

    div()
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x000000, 0.55))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("fleet-run-panel")
                .w(px(720.0))
                .flex()
                .flex_col()
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DANGER)
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(12.0))
                        .border_b_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().text_size(px(13.0)).font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child(run.title.clone()))
                        .child(div().text_color(if run.phase == RunPhase::Finished && run.steps.iter().any(|s| matches!(s.status, StepStatus::Failed(_))) { CRIT } else { TEXT_DIM }).child(status_line)),
                )
                .child(
                    div()
                        .id("fleet-run-steps")
                        .max_h(px(440.0))
                        .overflow_y_scrollbar()
                        .flex()
                        .flex_col()
                        .children(run.steps.iter().map(|s| {
                            let (mark, color, detail) = match &s.status {
                                StepStatus::Waiting => ("·", TEXT_FAINT, String::new()),
                                StepStatus::Running => ("▶", WARN, "running".to_string()),
                                StepStatus::Done(m) => ("✓", OK, m.clone()),
                                StepStatus::Skipped(m) => ("=", TEXT_DIM, m.clone()),
                                StepStatus::Failed(e) => ("✗", CRIT, e.clone()),
                                StepStatus::NotRun => ("–", TEXT_FAINTER, "not run".to_string()),
                            };
                            div()
                                .flex()
                                .gap(px(10.0))
                                .px(px(16.0))
                                .py(px(5.0))
                                .border_b_1()
                                .border_color(BORDER_ROW)
                                .child(div().w(px(14.0)).flex_none().text_color(color).child(mark))
                                .child(div().w(px(140.0)).flex_none().font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(s.server.clone()))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .flex()
                                        .flex_col()
                                        .child(div().text_color(TEXT_SECONDARY).child(s.what.clone()))
                                        .children((!detail.is_empty()).then(|| div().text_size(px(10.0)).text_color(color).child(detail))),
                                )
                        }))
                        .children(run.excluded.iter().map(|(server, why)| {
                            div()
                                .flex()
                                .gap(px(10.0))
                                .px(px(16.0))
                                .py(px(5.0))
                                .border_b_1()
                                .border_color(BORDER_ROW)
                                .child(div().w(px(14.0)).flex_none().text_color(WARN).child("!"))
                                .child(div().w(px(140.0)).flex_none().text_color(TEXT_DIM).child(server.clone()))
                                .child(div().flex_1().min_w(px(0.0)).text_color(TEXT_DIM).child(format!("left out · {why}")))
                        })),
                )
                .children(run.error.clone().map(|e| div().px(px(16.0)).py(px(6.0)).text_color(CRIT).child(e)))
                .child(
                    div()
                        .px(px(16.0))
                        .py(px(10.0))
                        .border_t_1()
                        .border_color(BORDER_PANEL)
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .children((run.phase == RunPhase::Confirming && !run.steps.is_empty()).then(|| {
                            div()
                                .w(px(160.0))
                                .child(Input::new(&runner.input))
                        }))
                        .children((run.phase == RunPhase::Confirming && !run.steps.is_empty()).then(|| {
                            let app = app.clone();
                            button("btn-fleet-run-start", format!("TYPE {} TO START", run.keyword), CRIT)
                                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.confirm_fleet_run(cx)))
                        }))
                        .children((run.phase == RunPhase::Paused).then(|| {
                            let app = app.clone();
                            button("btn-fleet-run-continue", "CONTINUE WITH THE REST".into(), OK).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.continue_fleet_run(cx)))
                        }))
                        .children((run.phase == RunPhase::Paused).then(|| {
                            let app = app.clone();
                            button("btn-fleet-run-stop-here", "STOP HERE".into(), WARN).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.stop_fleet_run(cx)))
                        }))
                        .children(runner.after.as_ref().filter(|_| run.phase == RunPhase::Finished).map(|(label, _)| {
                            let app = app.clone();
                            button("btn-fleet-run-after", label.clone(), WARN).on_click(move |_ev, window, cx| app.update(cx, |this, cx| this.run_fleet_after(window, cx)))
                        }))
                        .children((run.phase == RunPhase::Running && !run.stop_requested()).then(|| {
                            let app = app.clone();
                            button("btn-fleet-run-stop", "STOP AFTER THIS HOST".into(), WARN)
                                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.stop_fleet_run(cx)))
                        }))
                        .child(div().flex_1())
                        .children((!matches!(run.phase, RunPhase::Running | RunPhase::Paused)).then(|| {
                            let app = app.clone();
                            button("btn-fleet-run-close", if run.phase == RunPhase::Finished { "CLOSE".into() } else { "CANCEL".into() }, TEXT_SECONDARY)
                                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.close_fleet_run(cx)))
                        })),
                ),
        )
}
