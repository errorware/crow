use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

pub fn diamond_mark() -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| (),
        move |bounds, (), window, _cx| {
            let mut path = PathBuilder::fill();
            let mid_x = bounds.origin.x + bounds.size.width / 2.0;
            let mid_y = bounds.origin.y + bounds.size.height / 2.0;
            path.move_to(point(mid_x, bounds.origin.y));
            path.line_to(point(bounds.origin.x + bounds.size.width, mid_y));
            path.line_to(point(mid_x, bounds.origin.y + bounds.size.height));
            path.line_to(point(bounds.origin.x, mid_y));
            path.close();
            if let Ok(built) = path.build() {
                window.paint_path(built, TEXT_PRIMARY);
            }
        },
    )
    .w(px(14.0))
    .h(px(14.0))
    .flex_none()
}

#[derive(Clone, Debug)]
pub struct ServerTab {
    pub id: &'static str,
    pub name: &'static str,
    pub status_color: Rgba,
    #[allow(dead_code)]
    pub is_active: bool,
}

pub fn titlebar(tabs: &[ServerTab], active_tab_id: &str, app: Entity<CrowApp>) -> impl IntoElement {
    div()
        .h(px(36.0))
        .flex_none()
        .flex()
        .items_stretch()
        .bg(BG_CHROME)
        .border_b_1()
        .border_color(BORDER_PANEL)
        // 1. macOS native traffic lights clearance
        .child(
            div()
                .w(px(76.0))
                .flex_none(),
        )
        // 2. App mark & name
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(10.0))
                .border_r_1()
                .border_color(BORDER_PANEL)
                .child(diamond_mark())
                .child(
                    div()
                        .font_family("Inter")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("CROW"),
                ),
        )
        // 3. Tab strip
        .child(
            div()
                .flex()
                .items_stretch()
                .flex_1()
                .min_w(px(0.0))
                .children(tabs.iter().enumerate().map(|(idx, tab)| {
                    let is_active = tab.id == active_tab_id;
                    let tab_id = tab.id;
                    let app_tab = app.clone();

                    div()
                        .id(ElementId::NamedInteger("server-tab".into(), idx as u64))
                        .flex()
                        .items_center()
                        .gap(px(7.0))
                        .px(px(14.0))
                        .border_r_1()
                        .border_color(BORDER_PANEL)
                        .bg(if is_active { BG_OVERLAY_PANEL } else { hex_rgba(0, 0.0) })
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            app_tab.update(cx, |this, cx| {
                                this.switch_tab(tab_id, cx);
                            });
                        })
                        .child(
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(tab.status_color)
                                .flex_none(),
                        )
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(11.0))
                                .text_color(if is_active { TEXT_PRIMARY } else { TEXT_MUTED })
                                .child(tab.name),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(0x41434b))
                                .child("×"),
                        )
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .px(px(12.0))
                        .text_size(px(13.0))
                        .text_color(TEXT_FAINT)
                        .child("+"),
                ),
        )
        // 4. Session meta
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(14.0))
                .px(px(14.0))
                .font_family("JetBrains Mono")
                .text_size(px(10.5))
                .text_color(TEXT_FAINT)
                .child(div().child("SSH ED25519"))
                .child(div().child("lat 12ms"))
                .child(div().child("03:41:22 UTC")),
        )
}
