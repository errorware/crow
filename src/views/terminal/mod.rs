//! The terminal view (ERR-93): Crow's own renderer for an alacritty screen.
//! Each run is shaped with the font's cell width forced, and placed at its
//! column, so the grid stays exact whatever the glyphs.

use std::cell::Cell;
use std::rc::Rc;

use alacritty_terminal::vte::ansi::CursorShape;
use gpui_kit::*;

use std::collections::HashMap;

use crate::app::terminal::{TabRename, TerminalPane, TerminalWorkspace};
use crate::components::icons::{inherited_icon, TablerIcon};
use crate::terminal::layout::{Axis, Node, PaneId};
use crate::app::CrowApp;
use crate::terminal::palette;
use crate::terminal::session::{Frame, GridSize};
use crate::theme::*;

const FONT_SIZE: f32 = 13.0;
const LINE_HEIGHT: f32 = 1.3;
const PAD: f32 = 10.0;

fn color(c: u32) -> Hsla {
    rgb(c).into()
}

/// The server's terminals: a tab bar, then the active tab's panes.
pub fn workspace_view(ws: &TerminalWorkspace, rename: Option<&TabRename>, frames: &mut HashMap<PaneId, Frame>, focused: Option<PaneId>, app: Entity<CrowApp>) -> impl IntoElement {
    let shortcut = |mac: &'static str, other: &'static str| if cfg!(target_os = "macos") { mac } else { other };
    // Toolbar buttons: icon and its shortcut, quiet until hovered.
    let tool = |id: &'static str, icon: TablerIcon, keys: &'static str| {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(5.0))
            .px(px(8.0))
            .h_full()
            .text_color(TEXT_DIM)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
            .child(inherited_icon(icon, px(12.0)))
            .child(div().text_size(px(9.5)).child(keys))
    };
    let multi = ws.tabs.get(ws.active).is_some_and(|t| t.root.panes().len() > 1);
    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .bg(color(palette::BACKGROUND))
        // Tabs in Crow's own tab style: the active one on the app background
        // with the green band on top (as the server tabs above).
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_stretch()
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .children(ws.tabs.iter().enumerate().map(|(i, _)| {
                    let active = i == ws.active;
                    let rang = !active && ws.tab_rang(i);
                    let (app_sel, app_close) = (app.clone(), app.clone());
                    div()
                        .id(("term-tab", i))
                        .relative()
                        .overflow_hidden()
                        .flex()
                        .items_center()
                        .gap(px(7.0))
                        .pl(px(12.0))
                        .pr(px(6.0))
                        .max_w(px(240.0))
                        .border_r_1()
                        .border_color(if active { BORDER_STRONG } else { BORDER_PANEL })
                        .bg(if active { BG_APP } else { hex_rgba(0, 0.0) })
                        .cursor_pointer()
                        .hover(move |s| s.bg(if active { BG_APP } else { BG_ROW_HOVER }))
                        .on_click(move |_ev, _w, cx| app_sel.update(cx, |this, cx| this.terminal_select_tab(i, cx)))
                        .children(active.then(crate::components::titlebar::active_tab_gradient_bar))
                        .children(rang.then(|| div().size(px(6.0)).rounded_full().bg(WARN).flex_none()))
                        .child(
                            div()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .text_size(px(11.0))
                                .font_weight(if active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .text_color(if active { TEXT_MAX } else { TEXT_MUTED })
                                .child(ws.tab_title(i)),
                        )
                        .child(
                            div()
                                .id(("term-tab-close", i))
                                .flex()
                                .items_center()
                                .justify_center()
                                .flex_none()
                                .size(px(16.0))
                                .rounded_sm()
                                .text_color(TEXT_FAINT)
                                .hover(|s| s.bg(hex_rgba(0xffffff, 0.08)).text_color(TEXT_PRIMARY))
                                .on_mouse_down(MouseButton::Left, |_ev, _w, cx| cx.stop_propagation())
                                .on_click(move |_ev, _w, cx| {
                                    cx.stop_propagation();
                                    app_close.update(cx, |this, cx| this.terminal_close_tab(i, cx));
                                })
                                .child(inherited_icon(TablerIcon::X, px(10.0))),
                        )
                }))
                .child({
                    let app = app.clone();
                    tool("term-new-tab", TablerIcon::Plus, shortcut("⌘T", "Ctrl+Shift+T")).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.terminal_new_tab(cx)))
                })
                .child(div().flex_1())
                .child({
                    let app = app.clone();
                    tool("term-rename-tab", TablerIcon::Pencil, "rename").on_click(move |_ev, window, cx| {
                        app.update(cx, |this, cx| {
                            let active = this.terminal_workspace(cx).map(|ws| ws.active).unwrap_or(0);
                            this.start_tab_rename(active, window, cx);
                        })
                    })
                })
                .child({
                    let app = app.clone();
                    tool("term-split-right", TablerIcon::LayoutColumns, shortcut("⌘D", "Ctrl+Shift+D")).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.terminal_split(Axis::Row, cx)))
                })
                .child({
                    let app = app.clone();
                    tool("term-split-down", TablerIcon::LayoutRows, shortcut("⌘⇧D", "Ctrl+Shift+S")).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.terminal_split(Axis::Column, cx)))
                })
                .child({
                    let app = app.clone();
                    tool("term-close-pane", TablerIcon::X, shortcut("⌘W", "Ctrl+Shift+W")).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.terminal_close_pane(None, cx)))
                }),
        )
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .children(ws.tabs.get(ws.active).map(|tab| node_view(&tab.root, ws, frames, focused, multi, app.clone())))
                .children(ws.tabs.is_empty().then(|| {
                    div()
                        .p(px(16.0))
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_DIM)
                        .child(format!("No terminals open. {} opens one.", shortcut("⌘T", "Ctrl+Shift+T")))
                })),
        )
        .children(rename.map(|r| rename_panel(r, app.clone())))
}

/// The rename panel: a small box dropped from the toolbar's right side, no
/// backdrop. Enter saves, Esc or a click elsewhere cancels.
fn rename_panel(r: &TabRename, app: Entity<CrowApp>) -> impl IntoElement {
    let (app_esc, app_out) = (app.clone(), app);
    div()
        .id("term-rename-panel")
        .absolute()
        .top(px(32.0))
        .right(px(8.0))
        .w(px(280.0))
        .occlude()
        .p(px(10.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .bg(BG_PANEL)
        .border_1()
        .border_color(BORDER_DEFAULT)
        .shadow_lg()
        .font_family(FONT_MONO)
        .on_mouse_down(MouseButton::Left, |_ev, _w, cx| cx.stop_propagation())
        .on_mouse_down_out(move |_ev, _w, cx| app_out.update(cx, |this, cx| this.finish_tab_rename(false, cx)))
        .on_key_down(move |ev: &KeyDownEvent, _w, cx| {
            if ev.keystroke.key == "escape" {
                cx.stop_propagation();
                app_esc.update(cx, |this, cx| this.finish_tab_rename(false, cx));
            }
        })
        .child(gpui_kit::component::input::Input::new(&r.input))
        .child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child("Enter renames · Esc cancels · empty follows the shell"))
}

fn node_view(node: &Node, ws: &TerminalWorkspace, frames: &mut HashMap<PaneId, Frame>, focused: Option<PaneId>, multi: bool, app: Entity<CrowApp>) -> AnyElement {
    match node {
        Node::Pane(id) => match ws.panes.get(id) {
            Some(pane) => div()
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0))
                .border_1()
                .border_color(if multi && focused == Some(*id) { BORDER_CONTROL_SEL } else { hex_rgba(0, 0.0) })
                .child(terminal_view(pane, frames.remove(id), focused == Some(*id), app))
                .into_any_element(),
            None => div().flex_1().into_any_element(),
        },
        Node::Split { axis, first, second } => {
            let divider = match axis {
                Axis::Row => div().w(px(1.0)).h_full().flex_none().bg(BORDER_PANEL),
                Axis::Column => div().h(px(1.0)).w_full().flex_none().bg(BORDER_PANEL),
            };
            let base = div().flex_1().min_w(px(0.0)).min_h(px(0.0)).flex();
            let base = if *axis == Axis::Column { base.flex_col() } else { base };
            base.child(node_view(first, ws, frames, focused, multi, app.clone()))
                .child(divider)
                .child(node_view(second, ws, frames, focused, multi, app))
                .into_any_element()
        }
    }
}

pub fn terminal_view(pane: &TerminalPane, frame: Option<Frame>, focused: bool, app: Entity<CrowApp>) -> impl IntoElement {
    let target = pane.target.clone();
    let status = match (&pane.error, pane.session.as_ref().and_then(|s| s.exited.clone())) {
        (Some(e), _) => Some((format!("{e}"), CRIT)),
        (None, Some(why)) => Some((format!("{why} · press Enter to start a new one"), TEXT_DIM)),
        (None, None) if pane.session.is_none() => Some(("starting…".to_string(), TEXT_DIM)),
        _ => None,
    };
    let scrolled = frame.as_ref().map(|f| f.scrolled).unwrap_or(0);
    let (app_keys, app_wheel, app_click, app_middle, app_drag) = (app.clone(), app.clone(), app.clone(), app.clone(), app);
    let grid = pane.grid.clone();
    let pane_id = pane.id;

    div()
        .id("terminal-view")
        .size_full()
        .relative()
        .bg(color(palette::BACKGROUND))
        .track_focus(&pane.focus)
        .on_key_down(move |ev: &KeyDownEvent, window, cx| {
            let handled = app_keys.update(cx, |this, cx| this.terminal_key(ev, window, cx));
            if handled {
                cx.stop_propagation();
            }
        })
        .on_scroll_wheel(move |ev: &ScrollWheelEvent, _window, cx| {
            let dy = match ev.delta {
                ScrollDelta::Lines(l) => l.y * 3.0,
                ScrollDelta::Pixels(p) => p.y.as_f32() / (FONT_SIZE * LINE_HEIGHT),
            };
            let lines = dy.round() as i32;
            if lines != 0 {
                app_wheel.update(cx, |this, cx| this.terminal_scroll(pane_id, lines, cx));
            }
        })
        .on_mouse_down(MouseButton::Left, move |ev: &MouseDownEvent, _window, cx| {
            app_click.update(cx, |this, cx| this.terminal_select_start(pane_id, ev.position, ev.click_count, ev.modifiers.shift, cx));
        })
        .on_mouse_down(MouseButton::Middle, move |_ev, _window, cx| {
            app_middle.update(cx, |this, cx| this.terminal_paste_primary(pane_id, cx));
        })
        .child(
            canvas(move |_b, _w, _cx| (), move |bounds, (), window, cx| {
                paint_screen(bounds, frame.as_ref(), focused, &target, &grid, window, cx);
                // Drags keep selecting outside the pane, until released anywhere.
                let (moved, released) = (app_drag.clone(), app_drag.clone());
                window.on_mouse_event(move |ev: &MouseMoveEvent, phase, _window, cx| {
                    if phase == DispatchPhase::Bubble && ev.pressed_button == Some(MouseButton::Left) {
                        moved.update(cx, |this, cx| this.terminal_select_drag(pane_id, ev.position, cx));
                    }
                });
                window.on_mouse_event(move |ev: &MouseUpEvent, phase, _window, cx| {
                    if phase == DispatchPhase::Bubble && ev.button == MouseButton::Left {
                        released.update(cx, |this, cx| this.terminal_select_end(pane_id, cx));
                    }
                });
            })
            .size_full(),
        )
        .children((scrolled > 0).then(|| {
            div()
                .absolute()
                .top(px(6.0))
                .right(px(14.0))
                .px(px(6.0))
                .py(px(1.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .text_color(TEXT_DIM)
                .child(format!("↑ {scrolled} lines back · type to return"))
        }))
        .children(status.map(|(text, c)| {
            div()
                .absolute()
                .bottom(px(0.0))
                .left(px(0.0))
                .right(px(0.0))
                .px(px(PAD))
                .py(px(6.0))
                .bg(BG_PANEL)
                .border_t_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .text_color(c)
                .child(text)
        }))
}

fn paint_screen(bounds: Bounds<Pixels>, frame: Option<&Frame>, focused: bool, target: &Rc<Cell<Option<GridSize>>>, grid: &Rc<Cell<Option<(Point<Pixels>, Size<Pixels>)>>>, window: &mut Window, cx: &mut App) {
    let size = px(FONT_SIZE);
    let base = font(FONT_MONO);
    let ts = window.text_system().clone();
    let font_id = ts.resolve_font(&base);
    let cell_w = ts.advance(font_id, size, 'm').map(|s| s.width).unwrap_or(px(FONT_SIZE * 0.6));
    let cell_h = px((FONT_SIZE * LINE_HEIGHT).round());

    // Tell the session how many cells fit (applied by the pump).
    let cols = (((bounds.size.width - px(PAD * 2.0)) / cell_w).floor() as i32).max(2) as u16;
    let rows = (((bounds.size.height - px(PAD * 2.0)) / cell_h).floor() as i32).max(1) as u16;
    target.set(Some(GridSize { cols, rows, cell_w: cell_w.as_f32() as u16, cell_h: cell_h.as_f32() as u16 }));

    let Some(frame) = frame else { return };
    let origin = point(bounds.origin.x + px(PAD), bounds.origin.y + px(PAD));
    grid.set(Some((origin, size_of(cell_w, cell_h))));
    for (row, runs) in frame.rows.iter().enumerate() {
        let y = origin.y + cell_h * row as f32;
        for r in runs {
            let x = origin.x + cell_w * r.col as f32;
            if let Some(bg) = r.bg {
                window.paint_quad(fill(Bounds::new(point(x, y), size_of(cell_w * r.cols as f32, cell_h)), color(bg)));
            }
            if r.text.trim().is_empty() && !r.underline && !r.strike {
                continue;
            }
            let mut f = base.clone();
            if r.bold {
                f.weight = FontWeight::BOLD;
            }
            if r.italic {
                f.style = FontStyle::Italic;
            }
            let run = TextRun {
                len: r.text.len(),
                font: f,
                color: color(r.fg),
                background_color: None,
                underline: r.underline.then(|| UnderlineStyle { thickness: px(1.0), color: Some(color(r.fg)), wavy: false }),
                strikethrough: r.strike.then(|| StrikethroughStyle { thickness: px(1.0), color: Some(color(r.fg)) }),
            };
            let shaped = ts.shape_line(r.text.clone().into(), size, &[run], Some(cell_w));
            let _ = shaped.paint(point(x, y), cell_h, TextAlign::Left, None, window, cx);
        }
    }
    // Selection: a translucent wash over the selected cells.
    for &(row, first, last) in &frame.selection {
        let at = point(origin.x + cell_w * first as f32, origin.y + cell_h * row as f32);
        window.paint_quad(fill(Bounds::new(at, size_of(cell_w * (last - first + 1) as f32, cell_h)), color(palette::SELECTION).opacity(0.3)));
    }
    if let Some((row, col, shape)) = frame.cursor {
        let at = point(origin.x + cell_w * col as f32, origin.y + cell_h * row as f32);
        let c = color(palette::CURSOR);
        let quad = |o: Point<Pixels>, s: Size<Pixels>, w: &mut Window| w.paint_quad(fill(Bounds::new(o, s), c));
        match (shape, focused) {
            (_, false) | (CursorShape::HollowBlock, _) => {
                window.paint_quad(outline(Bounds::new(at, size_of(cell_w, cell_h)), c, BorderStyle::Solid));
            }
            (CursorShape::Beam, true) => quad(at, size_of(px(2.0), cell_h), window),
            (CursorShape::Underline, true) => quad(point(at.x, at.y + cell_h - px(2.0)), size_of(cell_w, px(2.0)), window),
            _ => window.paint_quad(fill(Bounds::new(at, size_of(cell_w, cell_h)), c.opacity(0.55))),
        }
    }
}

fn size_of(w: Pixels, h: Pixels) -> Size<Pixels> {
    gpui_kit::size(w, h)
}
