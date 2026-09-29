//! History view (ERR-86): four single-series charts (CPU, memory, disk,
//! load) on one time axis, from the stored minute samples. Gaps are
//! shaded, never drawn through; alert periods run along the top.

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::*;

use crate::app::history_view::HistoryCache;
use crate::app::CrowApp;
use crate::metrics::chart::{GapKind, Range, Series};
use crate::theme::*;

/// The one series hue: not a status color, so alerts stay distinct.
const SERIES: u32 = 0x38bdf8;

pub fn history_view(cache: &HistoryCache, hover: Option<f32>, app: Entity<CrowApp>) -> impl IntoElement {
    let at = |frac: f32| cache.from + ((cache.to - cache.from) as f32 * frac) as i64;
    let fmt = |t: i64| {
        chrono::DateTime::from_timestamp(t, 0)
            .map(|d| d.with_timezone(&chrono::Local).format(if cache.range == Range::Day { "%H:%M" } else { "%a %H:%M" }).to_string())
            .unwrap_or_default()
    };
    let bucket_frac = cache.range.bucket_secs() as f32 / (cache.to - cache.from) as f32;
    let readout = match hover {
        Some(h) => format!("at {}", fmt(at(h))),
        None => "latest".to_string(),
    };
    let charts: [(&str, &Series, Option<f32>, &str); 4] = [
        ("CPU", &cache.cpu, Some(100.0), "%"),
        ("MEMORY", &cache.mem, Some(100.0), "%"),
        (cache.disk_label.as_str(), &cache.disk, Some(100.0), "%"),
        ("LOAD (1m)", &cache.load, None, ""),
    ];
    let range_button = |r: Range| {
        let app = app.clone();
        let selected = cache.range == r;
        div()
            .id(SharedString::from(format!("hist-range-{}", r.label())))
            .px(px(8.0))
            .py(px(2.0))
            .border_1()
            .border_color(if selected { BORDER_CONTROL_SEL } else { BORDER_DEFAULT })
            .bg(if selected { BG_ROW_SELECTED } else { hex_rgba(0, 0.0) })
            .text_color(if selected { TEXT_MAX } else { TEXT_MUTED })
            .cursor_pointer()
            .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.set_history_range(r, cx)))
            .child(r.label())
    };

    div()
        .id("history-view")
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        .font_family(FONT_MONO)
        .child(
            div()
                .h(px(44.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(16.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(div().text_size(px(12.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child("HISTORY"))
                .child(div().text_size(px(10.5)).text_color(TEXT_DIM).child(format!("one sample a minute while Crow is open · {}", readout)))
                .child(div().flex_1())
                .child(range_button(Range::Day))
                .child(range_button(Range::Week)),
        )
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .p(px(16.0))
                .flex()
                .flex_col()
                .gap(px(14.0))
                .children(charts.into_iter().enumerate().map(|(i, (title, series, fixed_max, unit))| {
                    let value = match hover {
                        Some(h) => series.value_at(h, bucket_frac),
                        None => series.last,
                    };
                    let max = fixed_max.unwrap_or((series.max * 1.2).max(1.0));
                    chart(i, title, series.clone(), max, value.map(|v| format!("{v:.1}{unit}")).unwrap_or_else(|| "—".into()), cache, hover, app.clone())
                }))
                .child(legend(cache)),
        )
}

#[allow(clippy::too_many_arguments)]
fn chart(i: usize, title: &str, series: Series, max: f32, value: String, cache: &HistoryCache, hover: Option<f32>, app: Entity<CrowApp>) -> impl IntoElement {
    let gaps = cache.gaps.clone();
    let alerts = cache.alerts.clone();
    // Where the canvas was painted, for turning the pointer into a time.
    let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
    let painted = bounds.clone();
    div()
        .flex_1()
        .min_h(px(70.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .items_baseline()
                .gap(px(8.0))
                .child(div().text_size(px(10.0)).font_weight(FontWeight::BOLD).text_color(TEXT_DIMMER).child(title.to_string()))
                .child(div().flex_1())
                .child(div().text_size(px(11.0)).text_color(TEXT_PRIMARY).child(value)),
        )
        .child(
            div()
                .id(("history-chart", i))
                .flex_1()
                .relative()
                .border_1()
                .border_color(BORDER_ROW)
                .on_mouse_move(move |ev: &MouseMoveEvent, _window, cx| {
                    // Handled here: the view's own handler clears the crosshair.
                    cx.stop_propagation();
                    let Some(b) = bounds.get() else { return };
                    let x = ((ev.position.x - b.origin.x) / b.size.width).clamp(0.0, 1.0);
                    app.update(cx, |this, cx| this.set_history_hover(Some(x), cx));
                })
                .child(
                    canvas(
                        |_bounds, _window, _cx| (),
                        move |b, (), window, _cx| {
                            painted.set(Some(b));
                            let x = |f: f32| b.origin.x + b.size.width * f;
                            let y = |v: f32| b.origin.y + b.size.height * (1.0 - (v / max).clamp(0.0, 1.0));
                            let rect = |from: f32, to: f32, top: Pixels, h: Pixels, color: Rgba, window: &mut Window| {
                                window.paint_quad(fill(Bounds::new(point(x(from), top), size((x(to) - x(from)).max(px(1.0)), h)), color));
                            };
                            for g in &gaps {
                                let color = match g.kind {
                                    GapKind::CrowClosed => hex_rgba(0xffffff, 0.035),
                                    GapKind::Unreachable => hex_rgba(0xf87171, 0.08),
                                };
                                rect(g.from, g.to, b.origin.y, b.size.height, color, window);
                            }
                            // Hairline grid at half and full scale.
                            for f in [0.5, 1.0] {
                                window.paint_quad(fill(Bounds::new(point(b.origin.x, y(max * f)), size(b.size.width, px(1.0))), hex_rgba(0xffffff, 0.06)));
                            }
                            for (from, to, level, _) in &alerts {
                                rect(*from, *to, b.origin.y, px(3.0), if level == "CRIT" { CRIT } else { WARN }, window);
                            }
                            for seg in &series.segments {
                                if seg.len() == 1 {
                                    let (f, v) = seg[0];
                                    window.paint_quad(fill(Bounds::new(point(x(f) - px(1.5), y(v) - px(1.5)), size(px(3.0), px(3.0))), rgb(SERIES)));
                                    continue;
                                }
                                let mut path = PathBuilder::stroke(px(2.0));
                                path.move_to(point(x(seg[0].0), y(seg[0].1)));
                                for &(f, v) in &seg[1..] {
                                    path.line_to(point(x(f), y(v)));
                                }
                                if let Ok(p) = path.build() {
                                    window.paint_path(p, rgb(SERIES));
                                }
                            }
                            if let Some(h) = hover {
                                window.paint_quad(fill(Bounds::new(point(x(h), b.origin.y), size(px(1.0), b.size.height)), hex_rgba(0xffffff, 0.35)));
                            }
                        },
                    )
                    .size_full(),
                )
                .child(div().absolute().top(px(2.0)).left(px(4.0)).text_size(px(9.0)).text_color(TEXT_FAINT).child(format!("{max:.0}")))
                .child(div().absolute().bottom(px(2.0)).left(px(4.0)).text_size(px(9.0)).text_color(TEXT_FAINT).child("0")),
        )
}

/// What the shading and the top band mean, and the alerts in range.
fn legend(cache: &HistoryCache) -> impl IntoElement {
    let key = |color: Rgba, label: String| div().flex().items_center().gap(px(5.0)).child(div().size(px(9.0)).bg(color)).child(div().text_color(TEXT_DIM).child(label));
    div()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .text_size(px(10.0))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(14.0))
                .child(div().text_color(TEXT_DIM).child(format!(
                    "{} → now",
                    chrono::DateTime::from_timestamp(cache.from, 0).map(|d| d.with_timezone(&chrono::Local).format("%b %d %H:%M").to_string()).unwrap_or_default()
                )))
                .child(key(hex_rgba(0xffffff, 0.12), "Crow wasn't running: nothing watched".into()))
                .child(key(hex_rgba(0xf87171, 0.3), "server unreachable".into()))
                .child(key(WARN, "alert open (warn)".into()))
                .child(key(CRIT, "alert open (crit)".into())),
        )
        .children(cache.alert_labels.iter().map(|(level, text)| {
            div().flex().gap(px(6.0)).child(div().text_color(if level == "CRIT" { CRIT } else { WARN }).font_weight(FontWeight::BOLD).child(level.clone())).child(div().text_color(TEXT_SECONDARY).child(text.clone()))
        }))
}
