use gpui_kit::*;
use crate::theme::*;
use crate::app::CrowApp;

pub struct PaletteActionDef {
    pub cat: &'static str,
    pub label: &'static str,
    pub key: &'static str,
    pub is_selected: bool,
}

pub fn palette_actions() -> &'static [PaletteActionDef] {
    &[
        PaletteActionDef { cat: "CONFIG", label: "Edit pg_hba.conf (graphical)", key: "⏎", is_selected: true },
        PaletteActionDef { cat: "CONFIG", label: "Diff pg_hba.conf against fleet baseline", key: "d", is_selected: false },
        PaletteActionDef { cat: "CONFIG", label: "Open postgresql.conf", key: "o", is_selected: false },
        PaletteActionDef { cat: "CONFIG", label: "Harden sshd_config — disable PasswordAuthentication", key: "h", is_selected: false },
        PaletteActionDef { cat: "CONFIG", label: "Reload postgres config (pg_reload_conf)", key: "r", is_selected: false },
        PaletteActionDef { cat: "DANGER", label: "Restore pg_hba.conf from 03:12Z backup", key: "⇧b", is_selected: false },
    ]
}

pub fn palette_overlay(app: Entity<CrowApp>) -> impl IntoElement {
    let app_close1 = app.clone();
    let app_close2 = app.clone();

    div()
        .id("palette-scrim")
        .absolute()
        .inset_0()
        .bg(hex_rgba(0x050507, 0.62))
        .flex()
        .justify_center()
        .pt(px(140.0))
        .on_click(move |_ev, _window, cx| {
            app_close1.update(cx, |this, cx| {
                this.close_palette(cx);
            });
        })
        .child(
            div()
                .id("palette-panel")
                .w(px(620.0))
                .bg(BG_OVERLAY_PANEL)
                .border_1()
                .border_color(BORDER_STRONG)
                .on_click(|_ev, _window, _cx| {}) // capture click
                // Input row
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .h(px(42.0))
                        .px(px(14.0))
                        .border_b_1()
                        .border_color(BORDER_DEFAULT)
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(12.0))
                                .text_color(TEXT_FAINTER)
                                .child("›"),
                        )
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(13.0))
                                .text_color(TEXT_PRIMARY)
                                .child("hba conf"),
                        )
                        .child(
                            div()
                                .w(px(7.0))
                                .h(px(15.0))
                                .bg(TEXT_PRIMARY),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .font_family("JetBrains Mono")
                                .text_size(px(10.0))
                                .text_color(TEXT_DIMMER)
                                .child("edge-01 · 6 results"),
                        ),
                )
                // Result rows
                .children(palette_actions().iter().enumerate().map(|(idx, item)| {
                    let is_sel = item.is_selected;
                    let label = item.label;
                    let cat_color = match item.cat {
                        "DANGER" => CRIT,
                        "FIREWALL" | "KEYS" => WARN,
                        _ => TEXT_DIM,
                    };
                    let app_action = app.clone();

                    div()
                        .id(ElementId::NamedInteger("palette-action".into(), idx as u64))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .h(px(32.0))
                        .px(px(14.0))
                        .bg(if is_sel { BG_KEY } else { hex_rgba(0, 0.0) })
                        .children(if is_sel {
                            Some(left_indicator(TEXT_PRIMARY))
                        } else {
                            None
                        })
                        .cursor_pointer()
                        .on_click(move |_ev, _window, cx| {
                            app_action.update(cx, |this, cx| {
                                if label.contains("pg_hba.conf") {
                                    this.set_view("config", cx);
                                }
                                this.close_palette(cx);
                            });
                        })
                        .child(
                            div()
                                .w(px(60.0))
                                .flex_none()
                                .font_family("JetBrains Mono")
                                .text_size(px(10.0))
                                .text_color(cat_color)
                                .child(item.cat),
                        )
                        .child(
                            div()
                                .flex_1()
                                .font_family(FONT_MONO)
                                .text_size(px(12.0))
                                .text_color(if is_sel { TEXT_MAX } else { TEXT_SECONDARY })
                                .child(item.label),
                        )
                        .child(
                            div()
                                .flex_none()
                                .font_family("JetBrains Mono")
                                .text_size(px(10.0))
                                .text_color(TEXT_MUTED)
                                .bg(BG_KEY)
                                .border_1()
                                .border_color(BORDER_KEY)
                                .px(px(5.0))
                                .py(px(1.0))
                                .child(item.key),
                        )
                }))
                // Footer
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
                        .font_family("JetBrains Mono")
                        .text_size(px(10.0))
                        .text_color(TEXT_DIMMER)
                        .child("↑↓ navigate")
                        .child("⏎ run")
                        .child("⇥ complete")
                        .child("⌘⏎ run on all tabs")
                        .child(div().flex_1())
                        .child(
                            div()
                                .id("palette-esc-dismiss")
                                .cursor_pointer()
                                .on_click(move |_ev, _window, cx| {
                                    app_close2.update(cx, |this, cx| {
                                        this.close_palette(cx);
                                    });
                                })
                                .child("esc dismiss"),
                        ),
                ),
        )
}
