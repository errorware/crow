use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use crate::app::CrowApp;
use crate::palette::Entry;
use crate::theme::*;

fn category_color(category: &str) -> Rgba {
    match category {
        "SERVER" | "TERMINAL" => hex_rgb(0x8ab4ff),
        "SERVICE" | "PROCESS" => hex_rgb(0xbb9af7),
        "CONFIG" => OK,
        "ACTION" => WARN,
        "KEY" => hex_rgb(0xd6a24a),
        _ => TEXT_DIM,
    }
}

/// The command palette (⌘K, ERR-135): a real search over Crow's servers,
/// pages, config files, actions and keys. ↑/↓ move, Enter runs, Esc closes.
/// `scope`: the server the search is narrowed to (Tab), if any.
pub fn palette_overlay(app: Entity<CrowApp>, scope_label: &str, scope: Option<String>, input: Option<&Entity<InputState>>, results: &[Entry], selected: usize, query: &str) -> impl IntoElement {
    let (app_close1, app_close2, app_keys) = (app.clone(), app.clone(), app.clone());
    let count = if query.trim().is_empty() { format!("{scope_label} · recent first") } else { format!("{scope_label} · {} result{}", results.len(), if results.len() == 1 { "" } else { "s" }) };

    div()
        .id("palette-scrim")
        .absolute()
        .inset_0()
        .occlude()
        .bg(hex_rgba(0x050507, 0.62))
        .flex()
        .items_start()
        .justify_center()
        .pt(px(120.0))
        .on_click(move |_ev, _window, cx| app_close1.update(cx, |this, cx| this.close_palette(cx)))
        .child(
            div()
                .id("palette-panel")
                .occlude()
                .w(px(640.0))
                .max_h(px(520.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                // Before the input sees them: arrows move the selection, Esc closes.
                .capture_key_down(move |ev: &KeyDownEvent, window, cx| {
                    let step = match ev.keystroke.key.as_str() {
                        "up" => -1,
                        "down" => 1,
                        // Into the selected server's own pages, files, services.
                        "tab" => {
                            cx.stop_propagation();
                            app_keys.update(cx, |this, cx| this.palette_tab(window, cx));
                            return;
                        }
                        // On an empty query, back out of the server.
                        "backspace" => {
                            if app_keys.update(cx, |this, cx| this.palette_unscope(cx)) {
                                cx.stop_propagation();
                            }
                            return;
                        }
                        "escape" => {
                            cx.stop_propagation();
                            app_keys.update(cx, |this, cx| this.close_palette(cx));
                            return;
                        }
                        _ => return,
                    };
                    cx.stop_propagation();
                    app_keys.update(cx, |this, cx| this.palette_move(step, cx));
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .h(px(44.0))
                        .px(px(14.0))
                        .border_b_1()
                        .border_color(BORDER_DEFAULT)
                        .child(div().font_family(FONT_MONO).text_size(px(13.0)).text_color(TEXT_FAINTER).child("›"))
                        .children(scope.map(|name| {
                            div().flex_none().px(px(6.0)).py(px(1.0)).bg(hex_rgba(0x8ab4ff, 0.12)).border_1().border_color(hex_rgb(0x8ab4ff)).font_family(FONT_MONO).text_size(px(10.5)).text_color(hex_rgb(0x8ab4ff)).child(format!("{name} ›"))
                        }))
                        .child(div().flex_1().min_w(px(0.0)).children(input.map(|i| Input::new(i).appearance(false).font_family(FONT_MONO).text_size(px(13.0)))))
                        .child(div().flex_none().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_DIMMER).child(count)),
                )
                .child(
                    div()
                        .id("palette-results")
                        .flex()
                        .flex_col()
                        .overflow_y_scrollbar()
                        .children(results.is_empty().then(|| {
                            div().px(px(14.0)).py(px(14.0)).font_family(FONT_MONO).text_size(px(11.0)).text_color(TEXT_FAINT).child("Nothing matches. Try a server name, a page (logs, firewall), a config file, or an action.")
                        }))
                        .children(results.iter().enumerate().map(|(idx, item)| {
                            let is_sel = idx == selected;
                            let (app_run, entry) = (app.clone(), item.clone());
                            div()
                                .id(ElementId::NamedInteger("palette-result".into(), idx as u64))
                                .relative()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .h(px(32.0))
                                .px(px(14.0))
                                .bg(if is_sel { BG_KEY } else { hex_rgba(0, 0.0) })
                                .children(is_sel.then(|| left_indicator(TEXT_PRIMARY)))
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    let e = entry.clone();
                                    app_run.update(cx, |this, cx| this.palette_run(e, cx));
                                })
                                .child(div().w(px(68.0)).flex_none().font_family(FONT_MONO).text_size(px(9.5)).text_color(category_color(item.category)).child(item.category))
                                // The characters the query matched, picked out.
                                .child(div().flex_none().max_w(px(300.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().font_family(FONT_MONO).text_size(px(12.0)).text_color(if is_sel { TEXT_MAX } else { TEXT_SECONDARY }).child(
                                    StyledText::new(item.label.clone()).with_highlights(crate::palette::match_ranges(query, &item.label).into_iter().map(|r| (r, HighlightStyle { color: Some(hex_rgb(0xfbbf24).into()), font_weight: Some(FontWeight::BOLD), ..Default::default() }))),
                                ))
                                .child(div().flex_1().min_w(px(0.0)).overflow_hidden().whitespace_nowrap().text_ellipsis().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_FAINT).child(item.hint.clone()))
                                .children(is_sel.then(|| {
                                    div().flex_none().font_family(FONT_MONO).text_size(px(10.0)).text_color(TEXT_MUTED).bg(BG_KEY).border_1().border_color(BORDER_KEY).px(px(5.0)).py(px(1.0)).child("⏎")
                                }))
                        })),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(16.0))
                        .h(px(30.0))
                        .px(px(14.0))
                        .border_t_1()
                        .border_color(BORDER_DEFAULT)
                        .bg(BG_PANEL)
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("↑↓ navigate")
                        .child("⏎ open")
                        .child("⇥ into a server")
                        .child("⌫ back out")
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("palette-esc-dismiss")
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| app_close2.update(cx, |this, cx| this.close_palette(cx)))
                                .child("esc close"),
                        ),
                ),
        )
}

fn left_indicator(color: Rgba) -> impl IntoElement {
    div().absolute().left_0().top_0().bottom_0().w(px(2.0)).bg(color)
}
