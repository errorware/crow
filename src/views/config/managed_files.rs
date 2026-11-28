use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::app::CrowApp;
use crate::theme::*;
use crate::components::icons::{TablerIcon, tabler_icon};
use gpui_kit::component::input::{Input, InputState};
use crate::views::fleet::FleetState;
use crate::views::config::state::ConfigsState;

pub fn managed_files_rail(selected_file: &str, search_input: Option<&Entity<InputState>>, _fleet: &FleetState, configs: &ConfigsState, app: Entity<CrowApp>) -> impl IntoElement {
    let files = &configs.files;
    let app_clone = app.clone();
    let app_scan = app.clone();
    let sel_file = selected_file.to_string();
    let search_q = configs.search_query.trim().to_lowercase();

    // Filter files if search query is present
    let filtered_files: Vec<_> = files
        .iter()
        .filter(|f| {
            if search_q.is_empty() {
                true
            } else {
                f.name.to_lowercase().contains(&search_q) || f.path_dir.to_lowercase().contains(&search_q)
            }
        })
        .collect();

    let mapped_files: Vec<_> = filtered_files.iter().filter(|f| f.is_schema_mapped).copied().collect();
    let raw_files: Vec<_> = filtered_files.iter().filter(|f| !f.is_schema_mapped).copied().collect();

    div()
        .w(px(240.0))
        .flex_none()
        .h_full()
        .flex()
        .flex_col()
        .bg(bg_rail())
        .border_r_1()
        .border_color(border_panel())
        // 1. Header Toolbar
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .bg(bg_panel())
                .border_b_1()
                .border_color(border_panel())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            tabler_icon(TablerIcon::Settings)
                                .size(px(13.0))
                                .text_color(accent()),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(11.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_primary())
                                .child(t!("config-managed-files-config-files")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(10.0))
                                .text_color(text_faint())
                                .child(format!("{}/{}", filtered_files.len(), files.len())),
                        )
                        .child(
                            div()
                                .id("btn-crawl-refresh")
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(18.0))
                                .bg(bg_control())
                                .border_1()
                                .border_color(border_default())
                                .cursor_pointer()
                                .hover(|s| s.bg(bg_row_hover()).text_color(text_primary()))
                                .on_click(move |_ev, _window, cx| {
                                    app_scan.update(cx, |this, cx| {
                                        this.crawl_system_configs(cx);
                                    });
                                })
                                .child(
                                    tabler_icon(TablerIcon::Refresh)
                                        .size(px(10.5))
                                        .text_color(text_secondary()),
                                ),
                        ),
                ),
        )
        // Unverified distro warning — the crawler only trusted a generic /etc
        // scan for this host; no placeholder configs were fabricated either.
        .children(if !configs.family.is_supported() {
            Some(
                div()
                    .px(px(10.0))
                    .py(px(6.0))
                    .bg(theme().warn_bg)
                    .border_b_1()
                    .border_color(theme().warn)
                    .font_family(FONT_MONO)
                    .text_size(font_px(9.5))
                    .text_color(theme().warn_ink)
                    .child(t!("config-managed-files-unverified-distro-generic-scan-only")),
            )
        } else {
            None
        })
        // 2. Search / Filter Input
        .child(
            div()
                .h(px(30.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(10.0))
                .gap(px(6.0))
                .bg(bg_app())
                .border_b_1()
                .border_color(border_panel())
                .child(
                    tabler_icon(TablerIcon::Search)
                        .size(px(11.0))
                        .text_color(text_faint()),
                )
                .child(
                    div()
                        .flex_1()
                        .child(if let Some(input) = search_input {
                            div().child(Input::new(input).appearance(false).font_family(FONT_MONO).text_size(font_px(11.0)))
                        } else {
                            div()
                        })
                )
                .children(if !configs.search_query.is_empty() {
                    let app_clear = app_clone.clone();
                    Some(
                        div()
                            .id("btn-clear-config-search")
                            .cursor_pointer()
                            .p(px(2.0))
                            .hover(|s| s.bg(bg_row_hover()))
                            .on_mouse_down(MouseButton::Left, move |_ev, _win, cx| {
                                app_clear.update(cx, |this, cx| {
                                    this.configs.search_query.clear();
                                    this.configs.search_focused = false;
                                    cx.notify();
                                });
                            })
                            .child(
                                tabler_icon(TablerIcon::X)
                                    .size(px(10.0))
                                    .text_color(text_muted())
                            )
                            .into_any_element()
                    )
                } else {
                    None
                })
        )
        // 3. Scrollable List of Config Files
        .child(
            div()
                .id("managed-files-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                // --- SECTION 1: CROW UI MAPPED (PINNED TO TOP) ---
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.0))
                        .py(px(4.0))
                        .bg(bg_subhead())
                        .border_b_1()
                        .border_color(border_row())
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(accent())
                                .child(t!("config-managed-files-crow-ui-mapped")),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.0))
                                .text_color(text_faint())
                                .child(mapped_files.len().to_string()),
                        ),
                )
                .children(mapped_files.into_iter().enumerate().map(|(idx, f)| {
                    let is_sel = f.name == sel_file;
                    let is_edited = configs.states.get(&f.name).map(|s| s.is_modified()).unwrap_or(false);
                    let app_click = app_clone.clone();
                    let f_name = f.name.clone();

                    let pill_bg = if is_edited {
                        theme().warn_bg
                    } else {
                        accent_bg()
                    };
                    let pill_fg = if is_edited {
                        theme().warn
                    } else {
                        accent()
                    };
                    let pill_text = if is_edited {
                        t!("config-managed-files-edited")
                    } else {
                        t!("config-managed-files-crow-ui")
                    };

                    div()
                        .id(ElementId::NamedInteger("mapped-file-item".into(), idx as u64))
                        .relative()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .px(px(12.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(border_row())
                        .bg(if is_sel { bg_nav_active() } else { hex_rgba(0, 0.0) })
                        .children(if is_sel {
                            Some(left_indicator(accent()))
                        } else {
                            None
                        })
                        .cursor_pointer()
                        .hover(|s| s.bg(bg_row_hover()))
                        .on_click(move |_ev, _window, cx| {
                            let fn_str = f_name.clone();
                            app_click.update(cx, |this, cx| {
                                this.select_managed_file(&fn_str, cx);
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(11.0))
                                        .font_weight(if is_sel { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { text_max() } else { text_primary() })
                                        .child(f.name.clone()),
                                )
                                .child(
                                    div()
                                        .bg(pill_bg)
                                        .text_color(pill_fg)
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(8.5))
                                        .font_weight(FontWeight::BOLD)
                                        .px(px(4.0))
                                        .py(px(1.5))
                                        .rounded_sm()
                                        .flex_none()
                                        .child(pill_text),
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .text_color(text_fainter())
                                .child(f.path_dir.clone())
                                .children(if let Some(pack) = f.schema_pack {
                                    Some(div().text_color(text_faint()).child(pack))
                                } else {
                                    None
                                }),
                        )
                }))
                // --- SECTION 2: SYSTEM CONFIGS (RAW TRADITIONAL EDITOR) ---
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.0))
                        .py(px(4.0))
                        .mt(px(4.0))
                        .bg(bg_subhead())
                        .border_b_1()
                        .border_color(border_row())
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_muted())
                                .child(t!("config-managed-files-system-configs-raw")),
                        )
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.0))
                                .text_color(text_faint())
                                .child(raw_files.len().to_string()),
                        ),
                )
                .children(raw_files.into_iter().enumerate().map(|(idx, f)| {
                    let is_sel = f.name == sel_file;
                    let is_edited = configs.states.get(&f.name).map(|s| s.is_modified()).unwrap_or(false);
                    let app_click = app_clone.clone();
                    let f_name = f.name.clone();

                    let pill_bg = if is_edited {
                        theme().warn_bg
                    } else if f.is_readonly {
                        bg_chip()
                    } else {
                        bg_chip()
                    };
                    let pill_fg = if is_edited {
                        theme().warn
                    } else if f.is_readonly {
                        text_dimmer()
                    } else {
                        text_muted()
                    };
                    let pill_text = if is_edited {
                        t!("config-managed-files-edited")
                    } else if f.is_readonly {
                        t!("config-managed-files-ro")
                    } else {
                        t!("config-managed-files-raw")
                    };

                    div()
                        .id(ElementId::NamedInteger("raw-file-item".into(), idx as u64))
                        .relative()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .px(px(12.0))
                        .py(px(5.0))
                        .border_b_1()
                        .border_color(border_row())
                        .bg(if is_sel { bg_nav_active() } else { hex_rgba(0, 0.0) })
                        .children(if is_sel {
                            Some(left_indicator(text_primary()))
                        } else {
                            None
                        })
                        .cursor_pointer()
                        .hover(|s| s.bg(bg_row_hover()))
                        .on_click(move |_ev, _window, cx| {
                            let fn_str = f_name.clone();
                            app_click.update(cx, |this, cx| {
                                this.select_managed_file(&fn_str, cx);
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(11.0))
                                        .font_weight(if is_sel { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { text_max() } else { text_secondary() })
                                        .child(f.name.clone()),
                                )
                                .child(
                                    div()
                                        .bg(pill_bg)
                                        .text_color(pill_fg)
                                        .font_family(FONT_MONO)
                                        .text_size(font_px(8.5))
                                        .font_weight(FontWeight::BOLD)
                                        .px(px(4.0))
                                        .py(px(1.0))
                                        .rounded_sm()
                                        .flex_none()
                                        .child(pill_text),
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .font_family(FONT_MONO)
                                .text_size(font_px(9.5))
                                .text_color(text_fainter())
                                .child(f.path_dir.clone())
                                .child(f.display_size()),
                        )
                })),
        )
        // 4. Schema Packs Footer
        .child(
            div()
                .border_t_1()
                .border_color(border_panel())
                .p(px(8.0))
                .px(px(12.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_dimmer())
                        .child(t!("config-managed-files-schema-packs-active")),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(font_px(9.5))
                        .text_color(text_dim())
                        .child(t!("config-managed-files-systemd-postgres-openssh-ufw")),
                ),
        )
}
