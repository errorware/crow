pub mod collector;
pub mod models;
pub mod state;

pub use state::FilesState;

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;
use crate::components::icon_button::icon_button;
use gpui_kit::component::input::Input;
use crate::theme::*;
use crate::app::CrowApp;
use crate::components::icons::{TablerIcon, tabler_icon};
use gpui_kit::prelude::FluentBuilder as _;
use state::{FilePreview, PermsEdit};
use models::FileEntry;

pub fn file_browser_view(app: Entity<CrowApp>, files: &FilesState) -> impl IntoElement {
    let path = files.current_path.clone();

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
                    icon_button("btn-files-error-dismiss", TablerIcon::X, false)
                        .on_click(move |_ev, _window, cx| {
                            app_dismiss.update(cx, |this, cx| {
                                this.files.error = None;
                                cx.notify();
                            });
                        }),
                )
        }))
        // 4. New folder inline prompt
        .children(if files.new_folder_open {
            Some(render_new_folder_prompt(files, app.clone()))
        } else {
            None
        })
        // 5. The listing, with the viewer beside it when a file is open.
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .flex()
                .child(render_listing(files, app.clone()))
                .children(files.preview.as_ref().map(|pv| render_preview(pv, app.clone()))),
        )
}

fn render_listing(files: &FilesState, app: Entity<CrowApp>) -> impl IntoElement {
    let entries = &files.entries;
    // With the viewer open, owner/group/modified make way for it.
    let compact = files.preview.is_some();
    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
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
                .child(div().w(px(70.0)).flex_none().text_align(TextAlign::Right).child("SIZE"))
                .child(div().w(px(100.0)).flex_none().pl(px(12.0)).child("PERMS"))
                .child(div().w(px(90.0)).flex_none().child("OWNER"))
                .when(!compact, |d| d.child(div().w(px(90.0)).flex_none().child("GROUP")).child(div().w(px(120.0)).flex_none().child("MODIFIED")))
                .child(div().w(px(48.0)).flex_none()),
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
                    entries.iter().enumerate().map(|(idx, e)| render_file_row(e, idx, files, compact, app.clone())).collect()
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
                // The root segment is already a slash: no separator after it.
                .children(if !is_last && idx > 0 {
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

fn render_file_row(entry: &FileEntry, idx: usize, files: &FilesState, compact: bool, app: Entity<CrowApp>) -> AnyElement {
    let app_nav = app.clone();
    let app_del_toggle = app.clone();
    let app_perms = app.clone();
    let app_edit = app.clone();
    let name_for_nav = entry.name.clone();
    let name_for_delete = entry.name.clone();
    let name_for_perms = entry.name.clone();
    let path_for_edit = files.child_path(&entry.name);
    let is_dir = entry.is_dir;
    let is_pending_delete = files.pending_delete.as_deref() == Some(entry.name.as_str());
    let perms_open = files.perms.as_ref().filter(|p| p.name == entry.name);
    let is_previewed = !is_dir && files.preview.as_ref().is_some_and(|pv| pv.path == path_for_edit);
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
                .group("files-row")
                .h(px(27.0))
                .px(px(12.0))
                .font_family(FONT_MONO)
                .text_size(px(11.0))
                .when(is_previewed, |d| d.bg(BG_ROW_SELECTED))
                .hover(|s| s.bg(BG_ROW_HOVER))
                .cursor_pointer()
                // Folders open; files open in the viewer.
                .on_click(move |_ev, _window, cx| {
                    app_nav.update(cx, |this, cx| {
                        if is_dir {
                            this.files_go_into(&name_for_nav, cx);
                        } else {
                            this.files_open_preview(&name_for_nav, cx);
                        }
                    });
                })
                .children(is_previewed.then(|| left_indicator(TEXT_PRIMARY)))
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
                                .min_w(px(0.0))
                                .truncate()
                                .text_color(if entry.is_dir { TEXT_PRIMARY } else { TEXT_SECONDARY })
                                .child(entry.name.clone()),
                        ),
                )
                .child(div().w(px(70.0)).flex_none().text_align(TextAlign::Right).text_color(TEXT_DIM).child(entry.display_size()))
                // The permissions, coloured; click to change them.
                .child(
                    div()
                        .w(px(100.0))
                        .flex_none()
                        .pl(px(12.0))
                        .child(
                            div()
                                .id(ElementId::NamedInteger("files-row-perms".into(), idx as u64))
                                .px(px(3.0))
                                .mx(px(-3.0))
                                .border_1()
                                .border_color(if perms_open.is_some() { BORDER_STRONG } else { hex_rgba(0, 0.0) })
                                .hover(|s| s.bg(BG_CONTROL).border_color(BORDER_DEFAULT))
                                .on_click(move |_ev, _window, cx| {
                                    cx.stop_propagation();
                                    app_perms.update(cx, |this, cx| this.files_toggle_perms(&name_for_perms, cx));
                                })
                                .child(perms_text(&entry.mode_str)),
                        ),
                )
                .child(div().w(px(90.0)).flex_none().min_w(px(0.0)).truncate().text_color(TEXT_DIM).child(entry.owner.clone()))
                .when(!compact, |d| {
                    d.child(div().w(px(90.0)).flex_none().truncate().text_color(TEXT_DIM).child(entry.group.clone()))
                        .child(div().w(px(120.0)).flex_none().text_color(TEXT_FAINT).child(entry.modified.clone()))
                })
                // Row actions, shown on hover.
                .child(
                    div()
                        .w(px(48.0))
                        .flex_none()
                        .flex()
                        .justify_end()
                        .gap(px(2.0))
                        .invisible()
                        .group_hover("files-row", |s| s.visible())
                        .children((!is_dir).then(|| {
                            icon_button(ElementId::NamedInteger("files-row-edit".into(), idx as u64), TablerIcon::Terminal2, false)
                                .tooltip(|window, cx| gpui_kit::component::tooltip::Tooltip::new("Edit in a terminal (nano)").build(window, cx))
                                .on_click(move |_ev, _window, cx| {
                                    cx.stop_propagation();
                                    app_edit.update(cx, |this, cx| this.files_edit_in_terminal(&path_for_edit, cx));
                                })
                        }))
                        .child(
                            icon_button(ElementId::NamedInteger("files-row-delete".into(), idx as u64), TablerIcon::Trash, true)
                                .on_click(move |_ev, _window, cx| {
                                    cx.stop_propagation();
                                    app_del_toggle.update(cx, |this, cx| {
                                        this.toggle_file_delete_confirm(&name_for_delete, cx);
                                    });
                                }),
                        ),
                ),
        )
        .children(perms_open.map(|p| render_perms_editor(p, entry, app.clone())))
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

/// `ls -l` permissions, coloured like eza: read yellow, write red, execute
/// green, the special bits magenta; dashes recede.
fn perms_text(mode: &str) -> StyledText {
    let mut runs = Vec::new();
    for (i, c) in mode.char_indices() {
        let color = match c {
            'd' => hex_rgb(0x60a5fa),
            'l' => hex_rgb(0x38bdf8),
            'r' => hex_rgb(0xfacc15),
            'w' => hex_rgb(0xf87171),
            'x' => hex_rgb(0x4ade80),
            's' | 'S' | 't' | 'T' => hex_rgb(0xe879f9),
            _ => TEXT_FAINTER,
        };
        runs.push((i..i + c.len_utf8(), HighlightStyle { color: Some(color.into()), ..Default::default() }));
    }
    StyledText::new(mode.to_string()).with_highlights(runs)
}

/// The permission editor under a row: read/write/execute for owner, group
/// and others, the special bits, and the chmod it will run.
fn render_perms_editor(edit: &PermsEdit, entry: &FileEntry, app: Entity<CrowApp>) -> impl IntoElement {
    let mode = edit.mode;
    let changed = mode != edit.original;
    let symbolic = collector::mode_to_string(mode, entry.is_dir, entry.is_symlink);
    let who = [("OWNER", entry.owner.as_str(), 6u32), ("GROUP", entry.group.as_str(), 3), ("OTHERS", "everyone else", 0)];

    let bit = |id: String, label: &'static str, on: bool, color: Rgba, flip: u32| {
        let app = app.clone();
        div()
            .id(SharedString::from(id))
            .w(px(28.0))
            .h(px(20.0))
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .border_color(if on { color } else { BORDER_DEFAULT })
            .bg(if on { BG_CONTROL } else { hex_rgba(0, 0.0) })
            .font_family(FONT_MONO)
            .text_size(px(10.5))
            .font_weight(FontWeight::BOLD)
            .text_color(if on { color } else { TEXT_FAINTER })
            .cursor_pointer()
            .hover(|s| s.border_color(TEXT_SECONDARY))
            .on_click(move |_ev, _window, cx| app.update(cx, |this, cx| this.files_flip_perm_bit(flip, cx)))
            .child(if on { label } else { "-" })
    };

    let app_cancel = app.clone();
    let app_apply = app.clone();
    div()
        .relative()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(24.0))
        .pl(px(28.0))
        .pr(px(12.0))
        .py(px(10.0))
        .bg(BG_OVERLAY_PANEL)
        .border_t_1()
        .border_color(BORDER_PANEL)
        .font_family(FONT_MONO)
        .child(left_indicator(TEXT_SECONDARY))
        // rwx grid
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .children(who.into_iter().map(|(label, name, shift)| {
                    let m = (mode >> shift) & 0o7;
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(div().w(px(56.0)).text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(label))
                        .child(bit(format!("perm-{label}-r"), "r", m & 4 != 0, hex_rgb(0xfacc15), 4 << shift))
                        .child(bit(format!("perm-{label}-w"), "w", m & 2 != 0, hex_rgb(0xf87171), 2 << shift))
                        .child(bit(format!("perm-{label}-x"), "x", m & 1 != 0, hex_rgb(0x4ade80), 1 << shift))
                        .child(div().pl(px(6.0)).text_size(px(10.0)).text_color(TEXT_DIM).child(name.to_string()))
                })),
        )
        // special bits
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .children([("SETUID", 0o4000u32, "runs as its owner"), ("SETGID", 0o2000, if entry.is_dir { "new files get this group" } else { "runs as its group" }), ("STICKY", 0o1000, "only owners delete inside")].into_iter().map(|(label, b, what)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(bit(format!("perm-{label}"), "on", mode & b != 0, hex_rgb(0xe879f9), b))
                        .child(div().text_size(px(9.5)).font_weight(FontWeight::SEMIBOLD).text_color(TEXT_DIMMER).child(label))
                        .child(div().text_size(px(9.5)).text_color(TEXT_FAINT).child(what))
                })),
        )
        .child(div().flex_1())
        // result + actions
        .child(
            div()
                .flex()
                .flex_col()
                .items_end()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(10.0))
                        .child(div().text_size(px(18.0)).font_weight(FontWeight::BOLD).text_color(if changed { WARN } else { TEXT_PRIMARY }).child(format!("{mode:04o}")))
                        .child(div().text_size(px(12.0)).child(perms_text(&symbolic))),
                )
                .children((changed && mode & 0o002 != 0 && !entry.is_dir).then(|| div().text_size(px(9.5)).text_color(WARN).child("anyone on the server can change this file")))
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .child(
                            div()
                                .id("files-perms-cancel")
                                .px(px(8.0))
                                .py(px(3.0))
                                .border_1()
                                .border_color(BORDER_KEY)
                                .text_size(px(10.5))
                                .text_color(TEXT_TERTIARY)
                                .cursor_pointer()
                                .hover(|s| s.bg(BG_ROW_HOVER))
                                .on_click(move |_ev, _window, cx| {
                                    app_cancel.update(cx, |this, cx| {
                                        this.files.perms = None;
                                        cx.notify();
                                    })
                                })
                                .child("Cancel"),
                        )
                        .child(
                            div()
                                .id("files-perms-apply")
                                .px(px(9.0))
                                .py(px(3.0))
                                .border_1()
                                .border_color(if changed { OK } else { BORDER_DEFAULT })
                                .bg(if changed { OK_BG } else { hex_rgba(0, 0.0) })
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if changed { OK } else { TEXT_FAINTER })
                                .when(changed && !edit.saving, |d| {
                                    d.cursor_pointer()
                                        .hover(|s| s.bg(BG_ROW_HOVER))
                                        .on_click(move |_ev, _window, cx| app_apply.update(cx, |this, cx| this.files_apply_perms(cx)))
                                })
                                .child(if edit.saving { "applying…".to_string() } else { format!("chmod {mode:04o}") }),
                        ),
                ),
        )
}

/// The viewer: a file's text, read-only, highlighted like the config
/// editors, with a way out to a terminal editor.
fn render_preview(pv: &FilePreview, app: Entity<CrowApp>) -> impl IntoElement {
    let app_edit = app.clone();
    let app_close = app.clone();
    let path = pv.path.clone();
    let note = match &pv.result {
        None => "reading…".to_string(),
        Some(Err(_)) => String::new(),
        Some(Ok(t)) if t.truncated => format!("first {} KB · {} lines", collector::PREVIEW_LIMIT / 1024, t.lines),
        Some(Ok(t)) => format!("{} line{}", t.lines, if t.lines == 1 { "" } else { "s" }),
    };
    div()
        .w(relative(0.5))
        .flex_none()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(BORDER_PANEL)
        .bg(BG_RAIL)
        .font_family(FONT_MONO)
        .child(
            div()
                .h(px(32.0))
                .flex_none()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(10.0))
                .bg(BG_PANEL)
                .border_b_1()
                .border_color(BORDER_PANEL)
                .child(tabler_icon(TablerIcon::FileText).size(px(12.0)).text_color(TEXT_SECONDARY))
                .child(div().flex_1().min_w(px(0.0)).truncate().text_size(px(11.0)).font_weight(FontWeight::BOLD).text_color(TEXT_PRIMARY).child(path.clone()))
                .child(div().flex_none().text_size(px(10.0)).text_color(TEXT_FAINT).child(note))
                .child(
                    div()
                        .id("files-preview-edit")
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .bg(BG_CONTROL)
                        .border_1()
                        .border_color(BORDER_DEFAULT)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(TEXT_SECONDARY)
                        .cursor_pointer()
                        .hover(|s| s.bg(BG_ROW_HOVER).text_color(TEXT_PRIMARY))
                        .on_click(move |_ev, _window, cx| app_edit.update(cx, |this, cx| this.files_edit_in_terminal(&path, cx)))
                        .child(tabler_icon(TablerIcon::Terminal2).size(px(11.0)).text_color(TEXT_TERTIARY))
                        .child("EDIT IN TERMINAL"),
                )
                .child(icon_button("files-preview-close", TablerIcon::X, false).on_click(move |_ev, _window, cx| app_close.update(cx, |this, cx| this.files_close_preview(cx)))),
        )
        .child(match &pv.result {
            None => div().flex_1().p(px(14.0)).text_size(px(11.0)).text_color(TEXT_DIMMER).child("Reading the file…").into_any_element(),
            Some(Err(e)) => div().flex_1().p(px(14.0)).text_size(px(11.0)).text_color(CRIT_INK).child(e.clone()).into_any_element(),
            Some(Ok(t)) => div()
                .id("files-preview-scroll")
                .flex_1()
                .min_h(px(0.0))
                .overflow_scroll()
                .child(
                    div()
                        .flex()
                        .py(px(8.0))
                        .text_size(px(11.0))
                        .line_height(px(16.0))
                        .whitespace_nowrap()
                        .child(div().flex_none().pl(px(10.0)).pr(px(10.0)).text_align(TextAlign::Right).text_color(TEXT_FAINTER).child(t.gutter.clone()))
                        .child(div().flex_none().pr(px(16.0)).text_color(TEXT_SECONDARY).child(StyledText::new(t.text.clone()).with_highlights(t.runs.clone()))),
                )
                .into_any_element(),
        })
}
