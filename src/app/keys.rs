use gpui_kit::*;

use super::CrowApp;
use crate::keys::{
    copy_to_clipboard_system,
    expand_tilde,
    AddScanPathModalState,
    EditKeyModalState,
    KeyGenModalState,
    NewGroupModalState,
};
use crate::vault::SshKeyRecord;

// ==========================================
// SSH Key Management Hub
// ==========================================

impl CrowApp {
    pub fn refresh_keys(&mut self, cx: &mut Context<Self>) {
        if let Ok(db) = self.vault.db().lock() {
            self.keys.reload(&db);
        }
        self.sync_ssh_directory();
        cx.notify();
    }

    pub fn set_key_group_filter(&mut self, group_id: Option<String>, cx: &mut Context<Self>) {
        self.keys.group_filter = group_id;
        cx.notify();
    }

    pub fn import_discovered_key(&mut self, fingerprint: &str, group_id: Option<&str>, cx: &mut Context<Self>) {
        let key_opt = self.keys.discovered.iter().find(|k| k.fingerprint == fingerprint).cloned();
        if let Some(disc) = key_opt {
            let target_group = group_id.unwrap_or("fleet");
            let id = format!("key-{}", &fingerprint.replace("SHA256:", "").chars().take(12).collect::<String>());
            let record = SshKeyRecord {
                id,
                name: disc.suggested_name,
                group_id: target_group.to_string(),
                public_key: disc.public_key,
                fingerprint: disc.fingerprint,
                algorithm: disc.algorithm,
                comment: disc.comment,
                private_key_path: if disc.has_private_key { Some(disc.file_path) } else { None },
                attached_servers: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.upsert_ssh_key(&record);
            }
            self.keys.toast = Some(format!("Enrolled '{}' into group '{}'", record.name, target_group));
            self.refresh_keys(cx);
        }
    }

    pub fn delete_enrolled_key(&mut self, key_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.delete_ssh_key(key_id);
        }
        self.keys.toast = Some("Removed key from memory (file preserved)".to_string());
        self.refresh_keys(cx);
    }

    pub fn open_key_gen_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.gen_modal = Some(KeyGenModalState::default());
        self.caret.place(0);
        self.caret.blink = true;
        cx.notify();
    }

    pub fn close_key_gen_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.gen_modal = None;
        self.refresh_keys(cx);
    }

    pub fn submit_key_generation(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.keys.gen_modal {
            let name = state.name_input.trim();
            if name.is_empty() {
                state.error_message = Some("Key name cannot be empty".to_string());
                cx.notify();
                return;
            }

            let target_dir = expand_tilde(state.custom_dir_input.trim());
            let comment = if state.comment_input.trim().is_empty() {
                None
            } else {
                Some(state.comment_input.trim())
            };

            match crate::keys::generate_keypair(
                name,
                state.algo,
                comment,
                &state.group_id,
                &target_dir,
                None,
            ) {
                Ok((record, pub_key_openssh, priv_path, _pub_path)) => {
                    let db = self.vault.db();
                    if let Ok(db_guard) = db.lock() {
                        let _ = db_guard.upsert_ssh_key(&record);
                    }
                    state.error_message = None;
                    state.generated_public_key = Some(pub_key_openssh);
                    state.generated_priv_path = Some(priv_path.display().to_string());
                    state.generated_fingerprint = Some(record.fingerprint);
                    cx.notify();
                }
                Err(err) => {
                    state.error_message = Some(err);
                    cx.notify();
                }
            }
        }
    }

    pub fn open_new_group_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.new_group_modal = Some(NewGroupModalState {
            name_input: String::new(),
            color_input: "#4ade80".to_string(),
            error_message: None,
        });
        self.caret.place(0);
        self.caret.blink = true;
        cx.notify();
    }

    pub fn close_new_group_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.new_group_modal = None;
        cx.notify();
    }

    pub fn submit_new_group(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.keys.new_group_modal {
            let name = state.name_input.trim();
            if name.is_empty() {
                state.error_message = Some("Group name cannot be empty".to_string());
                cx.notify();
                return;
            }
            let slug = name.to_lowercase().replace(' ', "-").replace('_', "-");
            let color = if state.color_input.is_empty() { "#60a5fa" } else { &state.color_input };

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.add_key_group(&slug, name, color);
            }
            self.keys.new_group_modal = None;
            self.refresh_keys(cx);
        }
    }

    pub fn delete_key_group(&mut self, group_id: &str, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.delete_key_group(group_id);
        }
        if self.keys.group_filter.as_deref() == Some(group_id) {
            self.keys.group_filter = None;
        }
        self.refresh_keys(cx);
    }

    pub fn open_add_scan_path_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.add_scan_path_modal = Some(AddScanPathModalState::default());
        self.caret.place(0);
        self.caret.blink = true;
        cx.notify();
    }

    pub fn close_add_scan_path_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.add_scan_path_modal = None;
        cx.notify();
    }

    pub fn submit_add_scan_path(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.keys.add_scan_path_modal {
            let path = state.path_input.trim();
            if path.is_empty() {
                state.error_message = Some("Path cannot be empty".to_string());
                cx.notify();
                return;
            }
            let expanded = expand_tilde(path);
            if !expanded.exists() || !expanded.is_dir() {
                state.error_message = Some(format!("Directory does not exist: {}", expanded.display()));
                cx.notify();
                return;
            }

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.add_scan_path(path);
            }
            self.keys.add_scan_path_modal = None;
            self.refresh_keys(cx);
        }
    }

    pub fn remove_scan_path(&mut self, id: i64, cx: &mut Context<Self>) {
        let db = self.vault.db();
        if let Ok(db_guard) = db.lock() {
            let _ = db_guard.remove_scan_path(id);
        }
        self.refresh_keys(cx);
    }

    pub fn open_edit_key_modal(&mut self, key_id: &str, cx: &mut Context<Self>) {
        if let Some(key) = self.keys.enrolled.iter().find(|k| k.id == key_id) {
            let name_len = key.name.chars().count();
            self.keys.edit_modal = Some(EditKeyModalState {
                key_id: key.id.clone(),
                name_input: key.name.clone(),
                group_id: key.group_id.clone(),
                attached_servers: key.attached_servers.clone(),
                error_message: None,
            });
            self.caret.place(name_len);
            self.caret.blink = true;
            cx.notify();
        }
    }

    pub fn close_edit_key_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.edit_modal = None;
        cx.notify();
    }

    pub fn submit_edit_key(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.keys.edit_modal {
            let name = state.name_input.trim();
            if name.is_empty() {
                state.error_message = Some("Key name cannot be empty".to_string());
                cx.notify();
                return;
            }

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.update_ssh_key_name_and_group(&state.key_id, name, &state.group_id);
                let _ = db_guard.update_ssh_key_attached_servers(&state.key_id, &state.attached_servers);
            }
            self.keys.edit_modal = None;
            self.refresh_keys(cx);
        }
    }

    pub fn toggle_edit_key_server(&mut self, server_id: &str, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.keys.edit_modal {
            if let Some(idx) = state.attached_servers.iter().position(|s| s == server_id) {
                state.attached_servers.remove(idx);
            } else {
                state.attached_servers.push(server_id.to_string());
            }
            cx.notify();
        }
    }

    pub fn copy_text_with_toast(&mut self, text: &str, toast: &str, cx: &mut Context<Self>) {
        copy_to_clipboard_system(text);
        self.keys.toast = Some(toast.to_string());
        cx.notify();
    }

    pub fn clear_key_toast(&mut self, cx: &mut Context<Self>) {
        self.keys.toast = None;
        cx.notify();
    }
}
