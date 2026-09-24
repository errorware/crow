//! Search box and filter chips for table pages (Services, Processes, Users).

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use crate::components::icons::{tabler_icon, TablerIcon};
use crate::theme::*;

/// One filter chip: label, count, selected, count shown in red, and what a
/// click does.
pub struct Chip {
    pub id: String,
    pub label: &'static str,
    pub count: usize,
    pub is_on: bool,
    pub alarming: bool,
    pub on_click: Box<dyn Fn(&mut App)>,
}

/// Search box and filter chips (with counts) in a table page's header.
pub fn render_table_controls(search: Option<&Entity<InputState>>, chips: Vec<Chip>) -> impl IntoElement {
    div()
        .h_full()
        .flex()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .border_r_1()
        .border_color(BORDER_PANEL)
        .children(search.map(|input| {
            div().w(px(240.0)).child(
                Input::new(input)
                    .font_family(FONT_MONO)
                    .text_size(px(11.0))
                    .bg(BG_APP)
                    .rounded(px(2.0))
                    .prefix(tabler_icon(TablerIcon::Search).size(px(11.0)).text_color(TEXT_DIMMER)),
            )
        }))
        .children(chips.into_iter().map(|chip| {
            let on_click = chip.on_click;
            div()
                .id(SharedString::from(chip.id))
                .flex()
                .items_center()
                .gap(px(5.0))
                .px(px(8.0))
                .py(px(3.0))
                .border_1()
                .border_color(if chip.is_on { TEXT_SECONDARY } else { BORDER_DEFAULT })
                .bg(if chip.is_on { BG_CHIP } else { hex_rgba(0, 0.0) })
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(if chip.is_on { TEXT_PRIMARY } else { TEXT_DIMMER })
                .on_click(move |_ev, _window, cx| on_click(cx))
                .child(chip.label)
                .child(div().font_weight(FontWeight::NORMAL).text_color(if chip.alarming { CRIT } else { TEXT_FAINT }).child(chip.count.to_string()))
        }))
}

