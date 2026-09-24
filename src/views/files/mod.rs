pub mod collector;
pub mod models;
pub mod state;

pub use state::FilesState;

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use gpui_kit::component::input::Input;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use models::FileEntry;

pub fn file_browser_view(app: Entity<CrowApp>, files: &FilesState) -> impl IntoElement {
    let path = files.current_path.clone();
    let entries = &files.entries;

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(BG_APP)
        // 1. Header: title + simulated badge + toolbar
        .child(
            div()
                .h(px(34.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(10.0))
                .px(px(12.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(
                    tabler_icon(TablerIcon::Folder)
                        .size(px(13.0))
                        .text_color(hex_rgb(0x60a5fa)),
                )
                .child(
                    div()
                        .font_family(FONT_MONO)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_PRIMARY)
                        .child("FILES"),
                )
                .child(div().flex_1())
                .child({
                    let app_up = app.clone();
                    let can_go_up = path != "/";
                    div()
                        .id("btn-files-up")
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .px(px(8.0))
                        .py(px(3.5))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(if can_go_up { TEXT_SECONDARY } else { TEXT_FAINTER })
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_up.update(cx, |this, cx| {
                                this.files_go_up(cx);
                            });
                        })
                        .child(tabler_icon(TablerIcon::ChevronRight).size(px(10.0)).text_color(TEXT_FAINT))
                        .child(
                            div()
                                .font_family(FONT_MONO)
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .child("UP"),
                        )
                })
                .child({
                    let app_new = app.clone();
                    div()
                        .id("btn-files-new-folder")
                        .px(px(8.0))
                        .py(px(3.5))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .font_family(FONT_MONO)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .on_click(move |_ev, window, cx| {
                            app_new.update(cx, move |this, cx| {
                                this.toggle_new_folder_prompt(window, cx);
                            });
                        })
                        .child("+ FOLDER")
                })
                .child({
                    let app_refresh = app.clone();
                    div()
                        .id("btn-files-refresh")
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(26.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER))
                        .on_click(move |_ev, _window, cx| {
                            app_refresh.update(cx, |this, cx| {
                                this.load_file_listing(cx);
                            });
                        })
                        .child(tabler_icon(TablerIcon::Refresh).size(px(11.0)).text_color(TEXT_SECONDARY))
                }),
        )
        // 2. Breadcrumb path bar
        .child(render_breadcrumb(&path, app.clone()))
        // 3. Error banner
        .children(files.error.as_ref().map(|msg| {
            let app_dismiss = app.clone();
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .px(px(12.0))
                .py(px(6.0))
                .bg(CRIT_ROW_BG)
                .border_b_1()
                .border_color(CRIT)
                .font_family(FONT_MONO)
                .text_size(px(10.5))
                .text_color(CRIT_INK)
                .child(msg.clone())
                .child(
                    div()
                        .id("btn-files-error-dismiss")
                        .cursor_pointer()
                        .text_color(TEXT_DIMMER)
                        .hover(|s| s.text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| {
                            app_dismiss.update(cx, |this, cx| {
                                this.files.error = None;
                                cx.notify();
                            });
                        })
                        .child("✕"),
                )
        }))
        // 4. New folder inline prompt
        .children(if files.new_folder_open {
            Some(render_new_folder_prompt(files, app.clone()))
        } else {
            None
        })
        // 5. Column header
        .child(
            div()
                .h(px(24.0))
                .flex_none()
                .flex()
                .items_center()
                .px(px(12.0))
                .bg(BG_SUBHEAD)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .font_family(FONT_MONO)
                .text_size(px(9.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(TEXT_DIMMER)
                .child(div().flex_1().min_w(px(0.0)).child("NAME"))
                .child(div().w(px(70.0)).text_align(TextAlign::Right).child("SIZE"))
                .child(div().w(px(90.0)).pl(px(12.0)).child("PERMS"))
                .child(div().w(px(90.0)).child("OWNER"))
                .child(div().w(px(90.0)).child("GROUP"))
                .child(div().w(px(120.0)).child("MODIFIED"))
                .child(div().w(px(40.0))),
        )
        // 6. File list
        .child(
            div()
                .id("files-list-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_y_scrollbar()
                .children(if entries.is_empty() {
                    vec![
                        div()
                            .h(px(160.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(TEXT_DIMMER)
                            .child("Empty directory")
                            .into_any_element(),
                    ]
                } else {
                    entries.iter().enumerate().map(|(idx, e)| render_file_row(e, idx, files, app.clone())).collect()
                }),
        )
}

fn render_breadcrumb(path: &str, app: Entity<CrowApp>) -> impl IntoElement {
    let mut segments: Vec<(String, String)> = vec![("/".to_string(), "/".to_string())];
    let mut acc = String::new();
    for part in path.split('/').filter(|p| !p.is_empty()) {
        acc.push('/');
        acc.push_str(part);
        segments.push((part.to_string(), acc.clone()));
    }

    div()
        .h(px(26.0))
        .flex_none()
        .flex()
        .items_center()
        .gap(px(3.0))
        .px(px(12.0))
        .bg(BG_APP)
        .border_b_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .text_size(px(10.5))
        .children(segments.into_iter().enumerate().map(|(idx, (label, full_path))| {
            let app_click = app.clone();
            let is_last = full_path == path;
            div()
                .flex()
                .items_center()
                .gap(px(3.0))
                .child(
                    div()
                        .id(ElementId::NamedInteger("files-breadcrumb".into(), idx as u64))
                        .cursor_pointer()
                        .text_color(if is_last { TEXT_PRIMARY } else { TEXT_DIM })
                        .font_weight(if is_last { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .hover(|s| s.text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| {
                            let target = full_path.clone();
                            app_click.update(cx, |this, cx| {
                                this.navigate_files_to(&target, cx);
                            });
                        })
                        .child(label),
                )
                .children(if !is_last {
                    Some(div().text_color(TEXT_FAINTER).child("/"))
                } else {
                    None
                })
        }))
}

fn render_new_folder_prompt(files: &FilesState, app: Entity<CrowApp>) -> impl IntoElement {
    let app_confirm = app.clone();
    let app_cancel = app.clone();

    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(12.0))
        .py(px(6.0))
        .bg(BG_OVERLAY_PANEL)
        .border_b_1()
        .border_color(BORDER_STRONG)
        .child(
            tabler_icon(TablerIcon::Folder)
                .size(px(12.0))
                .text_color(hex_rgb(0x60a5fa)),
        )
        .children(files.new_folder_input.as_ref().map(|state| {
            div()
                .w(px(240.0))
                .child(
                    Input::new(state)
                        .id("input-new-folder-name")
                        .font_family(FONT_MONO)
                        .bg(BG_APP)
                        .border_color(BORDER_DEFAULT)
                        .rounded(px(2.0)),
                )
        }))
        .child(
            div()
                .id("btn-new-folder-confirm")
                .px(px(8.0))
                .py(px(3.0))
                .bg(OK_BG)
                .border_1()
                .border_color(OK)
                .text_color(OK)
                .cursor_pointer()
                .hover(|s| s.bg(BG_ROW_HOVER))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .on_click(move |_ev, _window, cx| {
                    app_confirm.update(cx, |this, cx| {
                        this.create_new_folder(cx);
                    });
                })
                .child("CREATE ⏎"),
        )
        .child(
            div()
                .id("btn-new-folder-cancel")
                .px(px(8.0))
                .py(px(3.0))
                .text_color(TEXT_DIM)
                .cursor_pointer()
                .hover(|s| s.text_color(TEXT_PRIMARY))
                .font_family(FONT_MONO)
                .text_size(px(10.0))
                .on_click(move |_ev, window, cx| {
                    app_cancel.update(cx, move |this, cx| {
                        this.toggle_new_folder_prompt(window, cx);
                    });
                })
                .child("cancel"),
        )
}

fn render_file_row(entry: &FileEntry, idx: usize, files: &FilesState, app: Entity<CrowApp>) -> AnyElement {
    let app_nav = app.clone();
    let app_del_toggle = app.clone();
    let name_for_nav = entry.name.clone();
    let name_for_delete = entry.name.clone();
    let is_dir = entry.is_dir;
    let is_pending_delete = files.pending_delete.as_deref() == Some(entry.name.as_str());
    let icon = if entry.is_dir {
        TablerIcon::Folder
    } else if entry.is_symlink {
        TablerIcon::ExternalLink
    } else {
        TablerIcon::FileText
    };

    div()
        .flex()
        .flex_col()
        .border_b_1()
        .border_color(BORDER_PANEL)
        .child(
            div()
                .id(ElementId::NamedInteger("files-row".into(), idx as u64))
                .relative()
                .flex()
                .items_center()
                .h(px(27.0))
                .px(px(12.0))
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .hover(|s| s.bg(BG_ROW_HOVER))
                .cursor_pointer()
                .on_click(move |_ev, _window, cx| {
                    if is_dir {
                        app_nav.update(cx, |this, cx| {
                            this.files_go_into(&name_for_nav, cx);
                        });
                    }
                })
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(tabler_icon(icon).size(px(12.0)).text_color(entry.icon_color()))
                        .child(
                            div()
                                .text_color(if entry.is_dir { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                .child(entry.name.clone()),
                        ),
                )
                .child(div().w(px(70.0)).text_align(TextAlign::Right).text_color(TEXT_DIM).child(entry.display_size()))
                .child(div().w(px(90.0)).pl(px(12.0)).text_color(TEXT_FAINT).child(entry.mode_str.clone()))
                .child(div().w(px(90.0)).text_color(TEXT_DIM).child(entry.owner.clone()))
                .child(div().w(px(90.0)).text_color(TEXT_DIM).child(entry.group.clone()))
                .child(div().w(px(120.0)).text_color(TEXT_FAINT).child(entry.modified.clone()))
                .child(
                    div()
                        .w(px(40.0))
                        .flex()
                        .justify_end()
                        .child(
                            div()
                                .id(ElementId::NamedInteger("files-row-delete".into(), idx as u64))
                                .cursor_pointer()
                                .text_color(TEXT_FAINT)
                                .hover(|s| s.text_color(CRIT))
                                .on_click(move |_ev, _window, cx| {
                                    app_del_toggle.update(cx, |this, cx| {
                                        this.toggle_file_delete_confirm(&name_for_delete, cx);
                                    });
                                })
                                .child(tabler_icon(TablerIcon::Trash).size(px(12.0))),
                        ),
                ),
        )
        .children(if is_pending_delete {
            let app_confirm = app.clone();
            let app_cancel = app.clone();
            let del_name = entry.name.clone();
            let is_dir = entry.is_dir;
            Some(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .h(px(32.0))
                    .pl(px(28.0))
                    .pr(px(12.0))
                    .bg(CRIT_ROW_BG)
                    .child(left_indicator(CRIT))
                    .child(
                        div()
                            .flex_1()
                            .font_family(FONT_MONO)
                            .text_size(px(11.0))
                            .text_color(CRIT_INK)
                            .child(if is_dir {
                                format!("Delete empty directory {}? This cannot be undone.", del_name)
                            } else {
                                format!("Delete {}? This cannot be undone.", del_name)
                            }),
                    )
                    .child(
                        div()
                            .id(ElementId::NamedInteger("files-cancel-delete".into(), idx as u64))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .text_color(TEXT_TERTIARY)
                            .border_1()
                            .border_color(BORDER_KEY)
                            .px(px(8.0))
                            .py(px(3.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(BG_ROW_HOVER))
                            .on_click(move |_ev, _window, cx| {
                                app_cancel.update(cx, |this, cx| {
                                    this.files.pending_delete = None;
                                    cx.notify();
                                });
                            })
                            .child("Cancel"),
                    )
                    .child(
                        div()
                            .id(ElementId::NamedInteger("files-exec-delete".into(), idx as u64))
                            .font_family(FONT_MONO)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(hex_rgb(0x0a0a0c))
                            .bg(CRIT)
                            .px(px(9.0))
                            .py(px(4.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(hex_rgb(0xef4444)))
                            .on_click(move |_ev, _window, cx| {
                                app_confirm.update(cx, |this, cx| {
                                    this.execute_file_delete(cx);
                                });
                            })
                            .child("DELETE ⏎"),
                    ),
            )
        } else {
            None
        })
        .into_any_element()
}
