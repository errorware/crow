//! The server's Containers page (ERR-140): its runtimes, compose stacks and
//! containers, and a drawer for the selected one (logs, inspect, actions,
//! the wand).

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::containers::{ContainersState, DrawerTab, Pending};
use crate::app::CrowApp;
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::containers::{Action, Container, RuntimeOutcome, Scan, Stack, StackAction};
use crate::lab::multipass::InstallStep;
use crate::theme::*;
use crate::views::overview::service_inspector::{render_eli5, WAND};

const W_IMAGE: f32 = 220.0;
const W_STATE: f32 = 150.0;
const W_PORTS: f32 = 230.0;
const W_RESTARTS: f32 = 70.0;
const W_RUNTIME: f32 = 110.0;

fn mono(size: f32, color: Rgba) -> Div {
    div().font_family(FONT_MONO).text_size(px(size)).text_color(color)
}

fn button(id: impl Into<SharedString>, label: &'static str, color: Rgba) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .h(px(22.0))
        .px(px(8.0))
        .border_1()
        .border_color(color.opacity(0.5))
        .font_family(FONT_MONO)
        .text_size(px(9.5))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .cursor_pointer()
        .hover(move |s| s.bg(color.opacity(0.12)))
        .child(label)
}

fn state_color(c: &Container) -> Rgba {
    match (c.state.as_str(), c.health.as_deref()) {
        (_, Some("unhealthy")) => CRIT,
        ("running", Some("starting")) | ("restarting", _) | ("paused", _) => WARN,
        ("running", _) => OK,
        ("exited", _) if !c.status.starts_with("Exited (0)") => CRIT,
        _ => TEXT_DIM,
    }
}

pub fn containers_view(st: &ContainersState, server_name: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let scan = st.current();
    let selected = st.selected.as_ref().and_then(|k| scan.and_then(|s| s.find(k)));
    div()
        .size_full()
        .flex()
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .h_full()
                .flex()
                .flex_col()
                .bg(BG_APP)
                .child(header(st, server_name, app.clone()))
                .child(
                    div()
                        .id("containers-scroll")
                        .flex_1()
                        .overflow_y_scrollbar()
                        .p(px(14.0))
                        .flex()
                        .flex_col()
                        .gap(px(14.0))
                        .children(st.busy.clone().map(|b| mono(10.5, WARN).child(format!("Running: {b}…"))))
                        .children(st.result.clone().map(|(ok, m)| mono(10.5, if ok { OK } else { CRIT }).line_height(px(15.0)).child(m)))
                        .child(match &st.scan {
                            None => mono(11.0, TEXT_DIM).child("Reading the server's containers…").into_any_element(),
                            Some(Err(e)) => mono(11.0, CRIT).child(format!("Couldn't read the server: {e}")).into_any_element(),
                            Some(Ok(scan)) => body(scan, st, app.clone()).into_any_element(),
                        }),
                ),
        )
        .children(selected.map(|c| drawer(c, st, app.clone())))
}

fn header(st: &ContainersState, server_name: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let scan = st.current();
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
        .child(mono(12.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(format!("CONTAINERS · {server_name}")))
        .children(scan.map(|s| {
            let running = s.containers.iter().filter(|c| c.is_running()).count();
            div().px(px(6.0)).py(px(2.0)).border_1().border_color(BORDER_DEFAULT).font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_SECONDARY).child(format!("{running}/{} RUNNING", s.containers.len()))
        }))
        .children(scan.into_iter().flat_map(|s| s.usable().map(|r| r.describe(s.login_is_root)).collect::<Vec<_>>()).map(|text| div().px(px(6.0)).py(px(2.0)).bg(BG_CHIP).font_family(FONT_MONO).text_size(px(9.5)).text_color(TEXT_TERTIARY).child(text)))
        .child(div().flex_1())
        .child(
            div()
                .id("btn-containers-refresh")
                .flex()
                .items_center()
                .gap(px(6.0))
                .h(px(26.0))
                .px(px(10.0))
                .border_1()
                .border_color(BORDER_DEFAULT)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(TEXT_SECONDARY)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.refresh_containers(cx)))
                .child(inherited_icon(TablerIcon::Refresh, px(12.0)))
                .child(if st.loading { "READING…" } else { "REFRESH" }),
        )
}

fn body(scan: &Scan, st: &ContainersState, app: Entity<CrowApp>) -> impl IntoElement {
    let problems: Vec<_> = scan
        .runtimes
        .iter()
        .filter_map(|r| match &r.outcome {
            RuntimeOutcome::Problem { summary, fix } => Some((r.runtime.label(), summary.clone(), fix.clone())),
            RuntimeOutcome::Ok { .. } => None,
        })
        .collect();
    let nothing = scan.runtimes.is_empty();
    let stacks = scan.stacks();
    let standalone: Vec<&Container> = scan.containers.iter().filter(|c| c.project().is_none()).collect();
    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .children(problems.into_iter().enumerate().map(|(i, (runtime, summary, fix))| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .p(px(12.0))
                .bg(WARN.opacity(0.06))
                .border_1()
                .border_color(WARN.opacity(0.4))
                .child(mono(11.0, WARN).font_weight(FontWeight::BOLD).child(format!("{runtime}: {summary}")))
                .children(fix.map(|command| crate::views::fleet::lab_modal::setup_step(i, &InstallStep { why: "Fix it on the server".into(), command }, app.clone())))
        }))
        .when(nothing, |d| {
            d.child(mono(11.0, TEXT_DIM).line_height(px(16.0)).child(if scan.missing.len() == 2 {
                "Neither Docker nor Podman is installed on this server.".to_string()
            } else {
                "No container runtime Crow can use here.".to_string()
            }))
        })
        .when(!nothing && scan.containers.is_empty(), |d| d.child(mono(11.0, TEXT_DIM).child("No containers on this server.")))
        .children(stacks.iter().map(|stack| stack_section(stack, scan, st, app.clone())))
        .when(!standalone.is_empty(), |d| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .child(div().flex().items_center().gap(px(8.0)).pb(px(6.0)).child(mono(10.5, TEXT_SECONDARY).font_weight(FontWeight::BOLD).child("STANDALONE")).child(mono(9.5, TEXT_FAINT).child(format!("{} not in a compose stack", standalone.len()))))
                    .child(table_head())
                    .children(standalone.iter().map(|c| row(c, st, app.clone()))),
            )
        })
}

fn stack_section(stack: &Stack, scan: &Scan, st: &ContainersState, app: Entity<CrowApp>) -> impl IntoElement {
    let members: Vec<&Container> = stack.containers.iter().filter_map(|k| scan.find(k)).collect();
    let pending = match &st.pending {
        Some(Pending::Stack { project, action }) if *project == stack.project => Some(*action),
        _ => None,
    };
    let action_button = |action: StackAction, color: Rgba| {
        let (app, project) = (app.clone(), stack.project.clone());
        button(format!("stack-{}-{}", stack.project, action.verb()), match action {
            StackAction::Up => "UP",
            StackAction::Restart => "RESTART",
            StackAction::Down => "DOWN",
        }, color)
        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.request_container_action(Pending::Stack { project: project.clone(), action }, cx)))
    };
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .pb(px(6.0))
                .child(inherited_icon(TablerIcon::Box, px(13.0)).text_color(TEXT_DIM))
                .child(mono(10.5, TEXT_PRIMARY).font_weight(FontWeight::BOLD).child(stack.project.clone()))
                .child(mono(9.5, if stack.running == members.len() { OK } else { WARN }).child(format!("{}/{} running", stack.running, members.len())))
                .children(stack.working_dir.clone().map(|d| mono(9.5, TEXT_FAINT).child(d)))
                .child(mono(9.5, TEXT_FAINT).child(stack.runtime.label()))
                .child(div().flex_1())
                .child(action_button(StackAction::Up, OK))
                .child(action_button(StackAction::Restart, WARN))
                .child(action_button(StackAction::Down, CRIT)),
        )
        .children(pending.map(|action| {
            let what = match action {
                StackAction::Up => "Create and start every service in this stack (it may pull images)?",
                StackAction::Restart => "Restart every container in this stack? Each is down for a moment.",
                StackAction::Down => "Stop and remove this stack's containers and networks? Named volumes are kept.",
            };
            confirm_row(what, crate::containers::stack_argv(stack, action).map(|a| a.join(" ")).unwrap_or_else(|e| e), app.clone())
        }))
        .child(table_head())
        .children(members.into_iter().map(|c| row(c, st, app.clone())))
}

fn confirm_row(question: &'static str, command: String, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_yes, app_no) = (app.clone(), app);
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .mb(px(6.0))
        .p(px(8.0))
        .bg(WARN.opacity(0.06))
        .border_1()
        .border_color(WARN.opacity(0.4))
        .child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap(px(2.0)).child(mono(10.5, TEXT_PRIMARY).child(question)).child(mono(9.5, TEXT_DIM).child(command)))
        .child(button("btn-container-confirm", "CONFIRM", WARN).on_click(move |_ev, _window, cx| app_yes.update(cx, |this, cx| this.confirm_container_pending(cx))))
        .child(button("btn-container-cancel", "CANCEL", TEXT_DIM).on_click(move |_ev, _window, cx| app_no.update(cx, |this, cx| this.cancel_container_pending(cx))))
}

fn table_head() -> impl IntoElement {
    let cell = |w: Option<f32>, text: &'static str| {
        let d = mono(9.0, TEXT_FAINT).font_weight(FontWeight::SEMIBOLD).child(text);
        match w {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.flex_1().min_w(px(0.0)),
        }
    };
    div()
        .flex()
        .items_center()
        .h(px(24.0))
        .px(px(10.0))
        .gap(px(10.0))
        .bg(BG_SUBHEAD)
        .border_1()
        .border_color(BORDER_PANEL)
        .child(cell(None, "NAME"))
        .child(cell(Some(W_IMAGE), "IMAGE"))
        .child(cell(Some(W_STATE), "STATE"))
        .child(cell(Some(W_PORTS), "PUBLISHED PORTS"))
        .child(cell(Some(W_RESTARTS), "RESTARTS"))
        .child(cell(Some(W_RUNTIME), "RUNTIME"))
}

fn ports_cell(c: &Container) -> impl IntoElement {
    let published = c.published();
    div().w(px(W_PORTS)).flex_none().flex().flex_wrap().gap(px(4.0)).when(published.is_empty(), |d| d.child(mono(10.0, TEXT_GHOST).child("—"))).children(published.into_iter().map(|p| {
        let exposed = p.exposed();
        let text = if exposed { format!("{}→{}", p.host_port, p.container) } else { format!("{}:{}→{}", p.host_ip, p.host_port, p.container) };
        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(mono(10.0, if exposed { TEXT_SECONDARY } else { TEXT_DIM }).child(text))
            .when(exposed, |d| d.child(div().px(px(3.0)).border_1().border_color(WARN.opacity(0.45)).font_family(FONT_MONO).text_size(px(8.5)).text_color(WARN.opacity(0.85)).child("ALL NETS")))
    }))
}

fn row(c: &Container, st: &ContainersState, app: Entity<CrowApp>) -> impl IntoElement {
    let key = c.key();
    let is_selected = st.selected.as_deref() == Some(key.as_str());
    let color = state_color(c);
    let state_text = match c.health.as_deref() {
        Some(h) if c.is_running() => format!("{} · {h}", c.state),
        _ => c.state.clone(),
    };
    let restarts = c.restarts.unwrap_or(0);
    div()
        .id(ElementId::Name(format!("container-row-{key}").into()))
        .flex()
        .items_center()
        .min_h(px(30.0))
        .px(px(10.0))
        .gap(px(10.0))
        .border_b_1()
        .border_l_1()
        .border_r_1()
        .border_color(BORDER_PANEL)
        .bg(if is_selected { BG_ROW_SELECTED } else { BG_PANEL })
        .cursor_pointer()
        .hover(|s| s.bg(BG_ROW_HOVER))
        .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.select_container(&key, cx)))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().flex_none().size(px(7.0)).rounded_full().bg(color))
                .child(mono(11.0, TEXT_PRIMARY).whitespace_nowrap().overflow_hidden().text_ellipsis().child(c.name.clone()))
                .children(c.service().filter(|s| !c.name.contains(*s)).map(|s| mono(9.5, TEXT_FAINT).child(s.to_string()))),
        )
        .child(mono(10.5, TEXT_TERTIARY).w(px(W_IMAGE)).flex_none().whitespace_nowrap().overflow_hidden().text_ellipsis().child(c.image.clone()))
        .child(div().w(px(W_STATE)).flex_none().flex().flex_col().child(mono(10.5, color).child(state_text)).child(mono(9.0, TEXT_FAINT).whitespace_nowrap().overflow_hidden().text_ellipsis().child(c.status.clone())))
        .child(ports_cell(c))
        .child(mono(10.5, if restarts > 0 { WARN } else { TEXT_DIM }).w(px(W_RESTARTS)).flex_none().child(c.restarts.map(|r| r.to_string()).unwrap_or_else(|| "—".into())))
        .child(mono(10.0, TEXT_DIM).w(px(W_RUNTIME)).flex_none().child(format!("{}{}", c.runtime.cmd(), if c.scope == crate::containers::Scope::Root { " · root" } else { "" })))
}

fn drawer(c: &Container, st: &ContainersState, app: Entity<CrowApp>) -> impl IntoElement {
    let key = c.key();
    let pending = match &st.pending {
        Some(Pending::Container { key: k, action }) if *k == key => Some(*action),
        _ => None,
    };
    let actions: Vec<(Action, Rgba)> = if c.is_running() { vec![(Action::Restart, WARN), (Action::Stop, CRIT)] } else { vec![(Action::Start, OK)] };
    let eli5_key = format!("{}|{}", c.image, c.service().unwrap_or_default());
    let (app_close, app_wand, app_logs, app_inspect, app_follow) = (app.clone(), app.clone(), app.clone(), app.clone(), app.clone());
    let tab = |id: &'static str, label: &'static str, active: bool| {
        div()
            .id(id)
            .px(px(10.0))
            .py(px(5.0))
            .border_b_2()
            .border_color(if active { TEXT_PRIMARY } else { hex_rgba(0, 0.0) })
            .font_family(FONT_MONO)
            .text_size(px(10.0))
            .font_weight(FontWeight::BOLD)
            .text_color(if active { TEXT_PRIMARY } else { TEXT_DIM })
            .cursor_pointer()
            .hover(|s| s.text_color(TEXT_PRIMARY))
            .child(label)
    };
    let content: AnyElement = match st.tab {
        DrawerTab::Logs => match &st.logs {
            Some((k, Ok(text))) if *k == key => {
                let lines: Vec<&str> = text.lines().collect();
                if lines.is_empty() {
                    mono(10.5, TEXT_DIM).child("No output yet.").into_any_element()
                } else {
                    div()
                        .flex()
                        .flex_col()
                        .children(lines.into_iter().rev().map(|l| {
                            // "2026-10-08T08:11:55.455676000Z message": the time, shortened.
                            let (ts, msg) = match l.split_once(' ') {
                                Some((t, m)) if t.len() >= 19 && t.as_bytes()[4] == b'-' => (t[11..19].to_string(), m.to_string()),
                                _ => (String::new(), l.to_string()),
                            };
                            div().flex().gap(px(8.0)).py(px(1.0)).child(mono(9.5, TEXT_FAINT).flex_none().w(px(54.0)).child(ts)).child(mono(10.0, TEXT_SECONDARY).flex_1().min_w(px(0.0)).child(msg))
                        }))
                        .into_any_element()
                }
            }
            Some((k, Err(e))) if *k == key => mono(10.5, CRIT).child(e.clone()).into_any_element(),
            _ => mono(10.5, TEXT_DIM).child("Reading logs…").into_any_element(),
        },
        DrawerTab::Inspect => match &st.inspect {
            Some((k, Ok(json))) if *k == key => div().flex().flex_col().children(json.lines().map(|l| mono(10.0, TEXT_SECONDARY).whitespace_nowrap().child(l.replace(' ', "\u{a0}")))).into_any_element(),
            Some((k, Err(e))) if *k == key => mono(10.5, CRIT).child(e.clone()).into_any_element(),
            _ => mono(10.5, TEXT_DIM).child("Reading…").into_any_element(),
        },
    };
    div()
        .w(px(520.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(BG_PANEL)
        .border_l_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(14.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().flex_none().size(px(8.0)).rounded_full().bg(state_color(c)))
                        .child(mono(13.0, TEXT_PRIMARY).font_weight(FontWeight::BOLD).flex_1().min_w(px(0.0)).overflow_hidden().text_ellipsis().child(c.name.clone()))
                        .child(
                            div()
                                .id("btn-container-eli5")
                                .size(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(if st.eli5_open { WAND } else { TEXT_DIM })
                                .bg(if st.eli5_open { WAND.opacity(0.14) } else { hex_rgba(0, 0.0) })
                                .cursor_pointer()
                                .hover(|s| s.bg(WAND.opacity(0.14)).text_color(WAND))
                                .on_click(move |_ev, _window, cx| app_wand.update(cx, |this, cx| this.explain_container(cx)))
                                .child(inherited_icon(TablerIcon::Wand, px(14.0))),
                        )
                        .child(
                            div()
                                .id("btn-container-close")
                                .size(px(22.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(TEXT_DIM)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                                .on_click(move |_ev, _window, cx| app_close.update(cx, |this, cx| this.close_container_drawer(cx)))
                                .child(inherited_icon(TablerIcon::X, px(14.0))),
                        ),
                )
                .child(mono(10.5, TEXT_TERTIARY).child(format!("{} · {} · {}", c.image, c.short_id(), c.runtime.label())))
                .child(mono(10.0, TEXT_DIM).child(c.status.clone()))
                .when(st.eli5_open, |d| d.child(render_eli5(st.eli5_loading.contains(&eli5_key), st.eli5.get(&eli5_key))))
                .child(div().flex().gap(px(6.0)).children(actions.into_iter().map(|(action, color)| {
                    let (app, key) = (app.clone(), key.clone());
                    button(format!("container-{}", action.verb()), match action {
                        Action::Start => "START",
                        Action::Stop => "STOP",
                        Action::Restart => "RESTART",
                    }, color)
                    .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.request_container_action(Pending::Container { key: key.clone(), action }, cx)))
                })))
                .children(pending.map(|action| {
                    let what = match action {
                        Action::Start => "Start this container?",
                        Action::Stop => "Stop this container? What it serves goes down until it's started again.",
                        Action::Restart => "Restart this container? It's down for a moment.",
                    };
                    confirm_row(what, crate::containers::action_argv(c, action).join(" "), app.clone())
                })),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .px(px(8.0))
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(tab("container-tab-logs", "LOGS", st.tab == DrawerTab::Logs).on_click(move |_ev, _window, cx| app_logs.update(cx, |this, cx| this.set_container_tab(DrawerTab::Logs, cx))))
                .child(tab("container-tab-inspect", "INSPECT", st.tab == DrawerTab::Inspect).on_click(move |_ev, _window, cx| app_inspect.update(cx, |this, cx| this.set_container_tab(DrawerTab::Inspect, cx))))
                .child(div().flex_1())
                .when(st.tab == DrawerTab::Logs, |d| {
                    d.child(mono(9.0, TEXT_FAINT).mr(px(8.0)).child("newest first")).child(
                        div()
                            .id("btn-container-follow")
                            .flex()
                            .items_center()
                            .gap(px(5.0))
                            .cursor_pointer()
                            .on_click(move |_ev, _window, cx| app_follow.update(cx, |this, cx| this.toggle_container_follow(cx)))
                            .child(div().size(px(6.0)).rounded_full().bg(if st.follow { OK } else { TEXT_GHOST }))
                            .child(mono(9.5, if st.follow { OK } else { TEXT_DIM }).font_weight(FontWeight::BOLD).child(if st.follow { "FOLLOWING" } else { "FOLLOW" })),
                    )
                })
                .when(st.tab == DrawerTab::Inspect, |d| d.child(mono(9.0, TEXT_FAINT).child("environment values hidden"))),
        )
        .child(div().id("container-drawer-scroll").flex_1().min_h(px(0.0)).overflow_y_scrollbar().p(px(12.0)).child(content))
}
