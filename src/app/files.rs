use gpui_kit::*;
use gpui_kit::component::input::InputState;

use super::CrowApp;
use crate::views::files::collector::{create_directory, delete_entry, list_directory_for_server};

// ==========================================
// Files — a literal directory browser on top of the server layer
// ==========================================

impl CrowApp {
    pub fn load_file_listing(&mut self, cx: &mut Context<Self>) {
        if let Some(srv) = self.fleet.active_server() {
            let (entries, is_sim) = list_directory_for_server(&srv, &self.files.current_path);
            self.files.entries = entries;
            self.files.is_simulated = is_sim;
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
}
