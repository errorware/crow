//! Settings → Personalisation → Time zones: the zones the titlebar clock
//! can show your time in.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::clock::{self, ClockState, Zone};
use crate::app::CrowApp;
use crate::components::icon_button::icon_button;
use crate::components::icons::TablerIcon;
use crate::config::CrowConfigManager;
use crate::theme::*;

/// How many search hits are listed.
const HITS: usize = 8;

pub fn time_zones_block(config: &CrowConfigManager, state: &ClockState, app: Entity<CrowApp>) -> impl IntoElement {
    let now = chrono::Utc::now();
    let zones = clock::zones_in(config);
    let selected = clock::selected_in(config);
    let hits = clock::search_zones(&state.query, HITS);
    let local = clock::local_name().unwrap_or_else(|| t!("settings-time-zones-unknown").to_string());

    let header = |label: SharedString| div().pt(px(4.0)).text_size(font_px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(text_dimmer()).child(label);

    div()
        .mx(px(14.0))
        .mb(px(14.0))
        .p(px(14.0))
        .bg(bg_panel())
        .border_1()
        .border_color(border_panel())
        .flex()
        .flex_col()
        .gap(px(8.0))
        .font_family(FONT_MONO)
        .child(div().text_size(font_px(11.5)).font_weight(FontWeight::BOLD).text_color(text_primary()).child(t!("settings-time-zones-title")))
        .child(div().text_size(font_px(10.0)).text_color(text_dim()).child(t!("settings-time-zones-about")))
        .child(
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(border_default())
                .child(zone_row(0, Zone::Local, t!("settings-time-zones-this-computer").to_string(), local, selected == Zone::Local, false, now, app.clone()))
                .children(zones.iter().enumerate().map(|(i, z)| zone_row(i + 1, Zone::Named(*z), clock::city(z.name()), z.name().to_string(), selected == Zone::Named(*z), true, now, app.clone()))),
        )
        .when(zones.is_empty(), |d| d.child(div().text_size(font_px(10.0)).text_color(text_faint()).child(t!("settings-time-zones-none-yet"))))
        .child(header(t!("settings-time-zones-add").into()))
        .children(state.search.as_ref().map(|input| div().w(px(360.0)).child(gpui_kit::component::input::Input::new(input).font_family(FONT_MONO).bg(bg_app()))))
        .when(!hits.is_empty(), |d| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(border_default())
                    .children(hits.into_iter().enumerate().map(|(i, tz)| hit_row(i, tz, zones.contains(&tz), now, app.clone()))),
            )
        })
        .when(hits_empty_for(&state.query), |d| d.child(div().text_size(font_px(10.0)).text_color(text_faint()).child(t!("settings-time-zones-no-match", query = state.query.trim()))))
}

fn hits_empty_for(query: &str) -> bool {
    !query.trim().is_empty() && clock::search_zones(query, 1).is_empty()
}

fn offset_and_time(zone: Zone, now: chrono::DateTime<chrono::Utc>) -> (String, String) {
    let offset = zone.offset_at(now);
    let time = chrono::FixedOffset::east_opt(offset).map(|o| now.with_timezone(&o).format("%H:%M").to_string()).unwrap_or_default();
    (clock::utc_label(offset), time)
}

/// One of the zones you can show your time in. Click to show it; yours
/// can be removed (the trash shows on hover).
#[allow(clippy::too_many_arguments)]
fn zone_row(i: usize, zone: Zone, label: String, name: String, selected: bool, removable: bool, now: chrono::DateTime<chrono::Utc>, app: Entity<CrowApp>) -> impl IntoElement {
    let (offset, time) = offset_and_time(zone, now);
    let app_pick = app.clone();
    div()
        .id(("time-zone-row", i))
        .group("time-zone-row")
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(10.0))
        .py(px(6.0))
        .border_b_1()
        .border_color(border_row())
        .when(selected, |d| d.bg(bg_row_selected()))
        .when(!selected, |d| d.cursor_pointer().hover(|s| s.bg(bg_row_hover())))
        .on_click(move |_ev, _window, cx| {
            app_pick.update(cx, |this, cx| this.select_time_zone(match zone { Zone::Named(tz) => Some(tz), Zone::Local => None }, cx));
        })
        .child(div().size(px(9.0)).flex_none().border_1().border_color(if selected { theme().ok } else { border_strong() }).when(selected, |d| d.bg(theme().ok)))
        .child(div().flex_none().text_size(font_px(11.0)).text_color(text_primary()).child(label))
        .child(div().flex_1().min_w(px(0.0)).truncate().text_size(font_px(9.5)).text_color(text_faint()).child(name))
        .child(div().flex_none().text_size(font_px(10.0)).text_color(text_secondary()).child(time))
        .child(div().w(px(76.0)).flex_none().text_right().text_size(font_px(9.5)).text_color(text_dim()).child(offset))
        .child(div().w(px(20.0)).flex_none().when(removable, |d| {
            let Zone::Named(tz) = zone else { return d };
            d.child(
                icon_button(("del-time-zone", i), TablerIcon::Trash, true)
                    .invisible()
                    .group_hover("time-zone-row", |s| s.visible())
                    .on_mouse_down(MouseButton::Left, |_ev, _window, cx| cx.stop_propagation())
                    .on_click(move |_ev, _window, cx| {
                        cx.stop_propagation();
                        app.update(cx, |this, cx| this.remove_time_zone(tz, cx));
                    }),
            )
        }))
}

/// A search hit, added with its plus.
fn hit_row(i: usize, tz: chrono_tz::Tz, added: bool, now: chrono::DateTime<chrono::Utc>, app: Entity<CrowApp>) -> impl IntoElement {
    let (offset, time) = offset_and_time(Zone::Named(tz), now);
    div()
        .id(("time-zone-hit", i))
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(10.0))
        .py(px(5.0))
        .border_b_1()
        .border_color(border_row())
        .child(div().flex_1().min_w(px(0.0)).truncate().text_size(font_px(10.5)).text_color(if added { text_faint() } else { text_primary() }).child(tz.name()))
        .child(div().flex_none().text_size(font_px(10.0)).text_color(text_secondary()).child(time))
        .child(div().w(px(76.0)).flex_none().text_right().text_size(font_px(9.5)).text_color(text_dim()).child(offset))
        .child(div().w(px(20.0)).flex_none().map(|d| {
            if added {
                d.flex().justify_center().child(crate::components::icons::tabler_icon(TablerIcon::Check).size(px(12.0)).text_color(theme().ok))
            } else {
                d.child(icon_button(("add-time-zone", i), TablerIcon::Plus, false).on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.add_time_zone(tz, cx))))
            }
        }))
}
