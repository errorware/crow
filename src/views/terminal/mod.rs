//! The terminal view (ERR-93): Crow's own renderer for an alacritty screen.
//! Each run is shaped with the font's cell width forced, and placed at its
//! column, so the grid stays exact whatever the glyphs.

use std::cell::Cell;
use std::rc::Rc;

use alacritty_terminal::vte::ansi::CursorShape;
use gpui_kit::*;

use std::collections::HashMap;

use crate::app::terminal::{TabMenu, TabRename, TerminalPane, TerminalWorkspace};
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
pub fn workspace_view(ws: &TerminalWorkspace, rename: Option<&TabRename>, menu: Option<&TabMenu>, frames: &mut HashMap<PaneId, Frame>, focused: Option<PaneId>, app: Entity<CrowApp>) -> impl IntoElement {
    let tool = |id: &'static str, icon: TablerIcon, tip: &'static str| {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(5.0))
            .px(px(7.0))
            .py(px(3.0))
            .text_color(TEXT_DIM)
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
            .child(inherited_icon(icon, px(12.0)))
            .child(div().text_size(px(9.5)).child(tip))
    };
    let shortcut = |mac: &'static str, other: &'static str| if cfg!(target_os = "macos") { mac } else { other };
    let multi = ws.tabs.get(ws.active).is_some_and(|t| t.root.panes().len() > 1);
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(color(palette::BACKGROUND))
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(2.0))
                .px(px(6.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .children(ws.tabs.iter().enumerate().map(|(i, _)| {
                    let selected = i == ws.active;
                    let rang = !selected && ws.tab_rang(i);
                    let (app_sel, app_close) = (app.clone(), app.clone());
                    div()
                        .id(("term-tab", i))
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .pl(px(10.0))
                        .pr(px(4.0))
                        .py(px(3.0))
                        .max_w(px(220.0))
                        .border_1()
                        .border_color(if selected { BORDER_CONTROL_SEL } else { hex_rgba(0, 0.0) })
                        .bg(if selected { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
                        .text_color(if rang { WARN } else if selected { TEXT_MAX } else { TEXT_MUTED })
                        .cursor_pointer()
                        .hover(|s| s.text_color(TEXT_PRIMARY))
                        // Double-click to rename; right-click for the tab menu.
                        .on_click(move |ev, window, cx| {
                            let double = ev.click_count() >= 2;
                            app_sel.update(cx, |this, cx| {
                                this.terminal_select_tab(i, cx);
                                if double {
                                    this.start_tab_rename(i, window, cx);
                                }
                            })
                        })
                        .on_mouse_down(MouseButton::Right, {
                            let app = app.clone();
                            move |ev: &MouseDownEvent, _w, cx| {
                                cx.stop_propagation();
                                let at = ev.position;
                                app.update(cx, |this, cx| this.open_tab_menu(i, at, cx));
                            }
                        })
                        .child(div().overflow_hidden().whitespace_nowrap().text_ellipsis().child(format!("{}{}", if rang { "● " } else { "" }, ws.tab_title(i))))
                        .child(
                            div()
                                .id(("term-tab-close", i))
                                .px(px(4.0))
                                .text_color(TEXT_FAINT)
                                .hover(|s| s.text_color(CRIT))
                                .on_click(move |_ev, _w, cx| {
                                    cx.stop_propagation();
                                    app_close.update(cx, |this, cx| this.terminal_close_tab(i, cx));
                                })
                                .child("×"),
                        )
                }))
                .child({
                    let app = app.clone();
                    tool("term-new-tab", TablerIcon::Plus, shortcut("⌘T", "Ctrl+Shift+T")).on_click(move |_ev, _w, cx| app.update(cx, |this, cx| this.terminal_new_tab(cx)))
                })
                .child(div().flex_1())
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
                .relative()
                .flex()
                .children(match ws.tabs.get(ws.active) {
                    Some(tab) => Some(node_view(&tab.root, ws, frames, focused, multi, app.clone())),
                    None => None,
                })
                .children(menu.map(|m| tab_menu(m, ws, app.clone())))
                .children(rename.map(|r| rename_popup(r, app.clone())))
                .children(ws.tabs.is_empty().then(|| {
                    div()
                        .p(px(16.0))
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .text_color(TEXT_DIM)
                        .child(format!("No terminals open. {} opens one.", shortcut("⌘T", "Ctrl+Shift+T")))
                })),
        )
}

/// The rename popup: a small dialog over the terminals.
fn rename_popup(r: &TabRename, app: Entity<CrowApp>) -> impl IntoElement {
    let button = |id: &'static str, label: &'static str, primary: bool| {
        div()
            .id(id)
            .px(px(12.0))
            .py(px(4.0))
            .border_1()
            .border_color(if primary { hex_rgb(0x8ab4ff) } else { BORDER_DEFAULT })
            .text_color(if primary { TEXT_MAX } else { TEXT_SECONDARY })
            .cursor_pointer()
            .hover(|s| s.bg(BG_ROW_HOVER))
            .child(label)
    };
    let (app_esc, app_save, app_cancel) = (app.clone(), app.clone(), app);
    div()
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x000000, 0.45))
        .flex()
        .items_start()
        .justify_center()
        .pt(px(60.0))
        .child(
            div()
                .id("term-rename-popup")
                .occlude()
                .w(px(340.0))
                .p(px(14.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .on_key_down(move |ev: &KeyDownEvent, _w, cx| {
                    if ev.keystroke.key == "escape" {
                        cx.stop_propagation();
                        app_esc.update(cx, |this, cx| this.finish_tab_rename(false, cx));
                    }
                })
                .child(div().font_weight(FontWeight::BOLD).text_color(TEXT_MAX).child("RENAME TAB"))
                .child(gpui_kit::component::input::Input::new(&r.input))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(div().flex_1().text_size(px(10.0)).text_color(TEXT_FAINT).child("Enter saves · Esc cancels"))
                        .child(button("term-rename-cancel", "CANCEL", false).on_click(move |_ev, _w, cx| app_cancel.update(cx, |this, cx| this.finish_tab_rename(false, cx))))
                        .child(button("term-rename-save", "SAVE", true).on_click(move |_ev, _w, cx| app_save.update(cx, |this, cx| this.finish_tab_rename(true, cx)))),
                ),
        )
}

/// The right-click menu on a tab.
fn tab_menu(m: &TabMenu, ws: &TerminalWorkspace, app: Entity<CrowApp>) -> impl IntoElement {
    let i = m.tab;
    let shortcut = |mac: &'static str, other: &'static str| if cfg!(target_os = "macos") { mac } else { other };
    let named = ws.tabs.get(i).is_some_and(|t| t.name.is_some());
    let many = ws.tabs.len() > 1;
    let item = |id: &'static str, label: &'static str, keys: &'static str, danger: bool, action: Box<dyn Fn(&mut CrowApp, &mut Window, &mut Context<CrowApp>)>| {
        let app = app.clone();
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(16.0))
            .px(px(10.0))
            .py(px(5.0))
            .text_color(if danger { CRIT_INK_DIM } else { TEXT_SECONDARY })
            .cursor_pointer()
            .hover(move |s| s.bg(BG_ROW_HOVER).text_color(if danger { CRIT } else { TEXT_MAX }))
            .on_click(move |_ev, window, cx| app.update(cx, |this, cx| action(this, window, cx)))
            .child(div().flex_1().child(label))
            .child(div().text_color(TEXT_FAINT).child(keys))
    };
    let rule = || div().h(px(1.0)).my(px(3.0)).bg(BORDER_PANEL);
    let close_app = app.clone();
    deferred(
        anchored().position(m.at).snap_to_window_with_margin(px(8.0)).child(
            div()
                .id("term-tab-menu")
                .w(px(240.0))
                .py(px(4.0))
                .bg(BG_PANEL)
                .border_1()
                .border_color(BORDER_DEFAULT)
                .shadow_lg()
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                // Clicks on the menu stay on the menu: the pane below would
                // otherwise take the mouse-down and close it before the click.
                .occlude()
                .on_mouse_down(MouseButton::Left, |_ev, _w, cx| cx.stop_propagation())
                .on_mouse_down_out(move |_ev, _w, cx| close_app.update(cx, |this, cx| this.close_tab_menu(cx)))
                .child(item("tm-rename", "Rename…", shortcut("⌘R", "Ctrl+Shift+R"), false, Box::new(move |this, window, cx| this.start_tab_rename(i, window, cx))))
                .children(named.then(|| item("tm-reset", "Use the shell's title", "", false, Box::new(move |this, _w, cx| this.reset_tab_name(i, cx)))))
                .child(rule())
                .child(item("tm-split-right", "Split right", shortcut("⌘D", "Ctrl+Shift+D"), false, Box::new(|this, _w, cx| {
                    this.close_tab_menu(cx);
                    this.terminal_split(Axis::Row, cx)
                })))
                .child(item("tm-split-down", "Split down", shortcut("⌘⇧D", "Ctrl+Shift+S"), false, Box::new(|this, _w, cx| {
                    this.close_tab_menu(cx);
                    this.terminal_split(Axis::Column, cx)
                })))
                .child(rule())
                .children(many.then(|| item("tm-close-others", "Close other tabs", "", true, Box::new(move |this, _w, cx| this.terminal_close_other_tabs(i, cx)))))
                .child(item("tm-close", "Close tab", "", true, Box::new(move |this, _w, cx| this.terminal_close_tab(i, cx)))),
        ),
    )
    .with_priority(1)
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
    let (app_keys, app_wheel, app_click) = (app.clone(), app.clone(), app);
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
        .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
            app_click.update(cx, |this, cx| this.terminal_focus_pane(pane_id, cx));
        })
        .child(canvas(move |_b, _w, _cx| (), move |bounds, (), window, cx| paint_screen(bounds, frame.as_ref(), focused, &target, window, cx)).size_full())
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

fn paint_screen(bounds: Bounds<Pixels>, frame: Option<&Frame>, focused: bool, target: &Rc<Cell<Option<GridSize>>>, window: &mut Window, cx: &mut App) {
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
