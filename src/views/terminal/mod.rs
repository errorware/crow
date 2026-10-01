//! The terminal view (ERR-93): Crow's own renderer for an alacritty screen.
//! Each run is shaped with the font's cell width forced, and placed at its
//! column, so the grid stays exact whatever the glyphs.

use std::cell::Cell;
use std::rc::Rc;

use alacritty_terminal::vte::ansi::CursorShape;
use gpui_kit::*;

use crate::app::terminal::TerminalPane;
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
    let focus = pane.focus.clone();

    div()
        .id("terminal-view")
        .size_full()
        .relative()
        .bg(color(palette::BACKGROUND))
        .track_focus(&pane.focus)
        .on_key_down(move |ev: &KeyDownEvent, _window, cx| {
            let handled = app_keys.update(cx, |this, cx| this.terminal_key(ev, cx));
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
                app_wheel.update(cx, |this, cx| this.terminal_scroll(lines, cx));
            }
        })
        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
            window.focus(&focus, cx);
            app_click.update(cx, |_, cx| cx.notify());
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
