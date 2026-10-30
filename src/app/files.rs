use gpui_kit::*;
use gpui_kit::component::input::InputState;

use super::CrowApp;
use crate::views::files::collector::{change_mode, create_directory, delete_entry, edit_then_exit_command, list_directory_for_server, read_preview};
use crate::views::files::state::{FilePreview, FilesEditor, PermsEdit, PreviewText};

// ==========================================
// Files — a literal directory browser on top of the server layer
// ==========================================

impl CrowApp {
    pub fn load_file_listing(&mut self, cx: &mut Context<Self>) {
        if let Some(srv) = self.fleet.active_server() {
            match list_directory_for_server(&srv, &self.files.current_path) {
                Ok(entries) => {
                    self.files.entries = entries;
                }
                Err(e) => {
                    self.files.entries.clear();
                    self.files.error = Some(e);
                }
            }
        } else {
            self.files.entries.clear();
        }
        cx.notify();
    }

    pub fn navigate_files_to(&mut self, path: &str, cx: &mut Context<Self>) {
        self.files.set_path(path);
        self.load_file_listing(cx);
    }

    pub fn files_go_into(&mut self, name: &str, cx: &mut Context<Self>) {
        let new_path = self.files.child_path(name);
        self.navigate_files_to(&new_path, cx);
    }

    pub fn files_go_up(&mut self, cx: &mut Context<Self>) {
        if let Some(parent) = self.files.parent_path() {
            self.navigate_files_to(&parent, cx);
        }
    }

    pub fn toggle_new_folder_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.files.new_folder_open = !self.files.new_folder_open;
        self.files.new_folder_input = if self.files.new_folder_open {
            Some(cx.new(|cx| InputState::new(window, cx).placeholder("new-folder-name")))
        } else {
            None
        };
        cx.notify();
    }

    pub fn create_new_folder(&mut self, cx: &mut Context<Self>) {
        if let Some(srv) = self.fleet.active_server() {
            let name = self.files.new_folder_input.as_ref()
                .map(|s| s.read(cx).value().trim().to_string())
                .unwrap_or_default();
            if !name.is_empty() {
                match create_directory(&srv, &self.files.current_path, &name) {
                    Ok(()) => {
                        self.files.new_folder_open = false;
                        self.files.new_folder_input = None;
                        self.load_file_listing(cx);
                        return;
                    }
                    Err(e) => self.files.error = Some(e),
                }
            }
        }
        cx.notify();
    }

    pub fn toggle_file_delete_confirm(&mut self, name: &str, cx: &mut Context<Self>) {
        self.files.toggle_delete_confirm(name);
        cx.notify();
    }

    pub fn execute_file_delete(&mut self, cx: &mut Context<Self>) {
        if let Some(name) = self.files.pending_delete.take() {
            if let Some(srv) = self.fleet.active_server() {
                let is_dir = self.files.is_dir(&name);
                match delete_entry(&srv, &self.files.current_path, &name, is_dir) {
                    Ok(()) => {
                        self.load_file_listing(cx);
                        return;
                    }
                    Err(e) => self.files.error = Some(e),
                }
            }
        }
        cx.notify();
    }

    /// Opens a file in the viewer, read in the background.
    pub fn files_open_preview(&mut self, name: &str, cx: &mut Context<Self>) {
        let path = self.files.child_path(name);
        self.files_preview_path(path, cx);
    }

    /// Reads `path` into the viewer.
    fn files_preview_path(&mut self, path: String, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        self.files.preview = Some(FilePreview { path: path.clone(), result: None });
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let p = path.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    let name = p.rsplit('/').next().unwrap_or_default().to_string();
                    read_preview(&srv, &p).map(|(text, more)| PreviewText::new(&text, more, &name))
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                // Only if the viewer still shows that file.
                if let Some(pv) = this.files.preview.as_mut().filter(|pv| pv.path == path) {
                    pv.result = Some(result);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn files_close_preview(&mut self, cx: &mut Context<Self>) {
        self.files_close_editor(cx);
        self.files.preview = None;
        cx.notify();
    }

    /// Opens (or closes) the permission editor under a row.
    pub fn files_toggle_perms(&mut self, name: &str, cx: &mut Context<Self>) {
        if self.files.perms.as_ref().is_some_and(|p| p.name == name) {
            self.files.perms = None;
        } else if let Some(e) = self.files.entries.iter().find(|e| e.name == name) {
            self.files.perms = Some(PermsEdit { name: name.to_string(), original: e.mode, mode: e.mode, saving: false });
            self.files.pending_delete = None;
        }
        cx.notify();
    }

    pub fn files_flip_perm_bit(&mut self, bit: u32, cx: &mut Context<Self>) {
        if let Some(p) = self.files.perms.as_mut() {
            p.mode ^= bit;
        }
        cx.notify();
    }

    /// chmod's the file to the editor's bits, and records it in the audit log.
    pub fn files_apply_perms(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        let Some(edit) = self.files.perms.as_ref().filter(|p| !p.saving && p.mode != p.original) else { return };
        let (path, before, after) = (self.files.child_path(&edit.name), edit.original, edit.mode);
        if let Some(edit) = self.files.perms.as_mut() {
            edit.saving = true;
        }
        let db = self.vault.db();
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let started = chrono::Utc::now().to_rfc3339();
            let (s, p) = (srv.clone(), path.clone());
            let result = cx.background_executor().spawn(async move { change_mode(&s, &p, after) }).await;
            if let Ok(db) = db.lock() {
                let _ = db.insert_change_record(&crate::vault::ChangeRecord {
                    id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                    server_id: srv.id.clone(),
                    server_name: srv.name.clone(),
                    action_kind: "file.chmod".into(),
                    target: path.clone(),
                    before_state: format!("{before:04o}"),
                    after_state: Some(format!("{after:04o}")),
                    blast_radius: None,
                    outcome: if result.is_ok() { "success" } else { "failed" }.into(),
                    started_at: started,
                    completed_at: Some(chrono::Utc::now().to_rfc3339()),
                });
            }
            let _ = entity.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.files.perms = None;
                        this.load_file_listing(cx);
                    }
                    Err(e) => {
                        if let Some(p) = this.files.perms.as_mut() {
                            p.saving = false;
                        }
                        this.files.error = Some(t!("app-files-chmod", after = format!("{:04o}", after), path = path, e = e));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Opens a terminal under the file's viewer with the file in nano (or
    /// vim/vi), through sudoedit when the login can't write it (ERR-188).
    pub fn files_edit_in_terminal(&mut self, path: &str, cx: &mut Context<Self>) {
        let Some(cmd) = edit_then_exit_command(path) else {
            self.files.error = Some(t!("app-files-that-name-has-control-characters").into());
            cx.notify();
            return;
        };
        let Some(srv) = self.fleet.active_server() else { return };
        // One editor at a time; the viewer shows the file being edited.
        let split = self.files.editor.as_ref().map_or(0.5, |e| e.split);
        self.files_close_editor(cx);
        if self.files.preview.as_ref().is_none_or(|p| p.path != path) {
            self.files_preview_path(path.to_string(), cx);
        }
        let Some(pane) = self.terminal_detached_pane(cmd.into_bytes(), cx) else { return };
        self.files.editor = Some(FilesEditor { pane, server_id: srv.id, path: path.to_string(), split, dragging: false, bounds: Default::default() });
        cx.notify();
    }

    /// Closes the terminal under the viewer (its session ends) and reads
    /// the file again: it may have changed.
    pub fn files_close_editor(&mut self, cx: &mut Context<Self>) {
        let Some(ed) = self.files.editor.take() else { return };
        if let Some(ws) = self.terminals.get_mut(&ed.server_id) {
            ws.panes.remove(&ed.pane);
        }
        if self.files.preview.as_ref().is_some_and(|p| p.path == ed.path) {
            self.files_preview_path(ed.path, cx);
        }
        cx.notify();
    }

    /// The editor terminal's frame and whether it has focus, taking focus
    /// when it asked for it. `None` with no editor open.
    pub fn files_editor_snapshot(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<(Option<crate::terminal::session::Frame>, bool)> {
        let ed = self.files.editor.as_ref()?;
        let pane = self.terminals.get_mut(&ed.server_id)?.panes.get_mut(&ed.pane)?;
        if pane.focus_pending {
            window.focus(&pane.focus, cx);
            pane.focus_pending = false;
        }
        Some((pane.session.as_ref().map(|s| s.frame()), pane.focus.is_focused(window)))
    }

    /// Dragging the divider between the viewer and the terminal.
    pub fn files_editor_drag(&mut self, y: Pixels, cx: &mut Context<Self>) {
        let Some(ed) = self.files.editor.as_mut().filter(|e| e.dragging) else { return };
        let Some(b) = ed.bounds.get() else { return };
        if b.size.height > px(0.0) {
            ed.split = ((y - b.origin.y) / b.size.height).clamp(0.15, 0.85);
            cx.notify();
        }
    }

    pub fn files_editor_set_dragging(&mut self, dragging: bool, cx: &mut Context<Self>) {
        if let Some(ed) = self.files.editor.as_mut() {
            ed.dragging = dragging;
            cx.notify();
        }
    }
}
