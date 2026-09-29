use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::CrowApp;
use crate::keys::{
    expand_tilde,
    AddScanPathModalState,
    EditKeyModalState,
    KeyGenModalState,
    NewGroupModalState,
};
use crate::vault::SshKeyRecord;

pub struct KeyGenInputs {
    pub name: Entity<InputState>,
    pub comment: Entity<InputState>,
    pub custom_dir: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

pub struct NewGroupInputs {
    pub name: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

pub struct AddScanPathInputs {
    pub path: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

pub struct EditKeyInputs {
    pub name: Entity<InputState>,
    pub _events: Vec<Subscription>,
}

pub enum KeyModalInputs {
    Gen(KeyGenInputs),
    NewGroup(NewGroupInputs),
    AddScanPath(AddScanPathInputs),
    Edit(EditKeyInputs),
}

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
        self.key_inputs = None;
        cx.notify();
    }

    pub fn close_key_gen_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.gen_modal = None;
        self.key_inputs = None;
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
        self.key_inputs = None;
        cx.notify();
    }

    pub fn close_new_group_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.new_group_modal = None;
        self.key_inputs = None;
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

            let id = name.to_lowercase().replace(' ', "-");
            let group = crate::keys::SshKeyGroup {
                id: id.clone(),
                name: name.to_string(),
                color: state.color_input.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            let db = self.vault.db();
            if let Ok(db_guard) = db.lock() {
                let _ = db_guard.add_key_group(&group.id, &group.name, &group.color);
            }
            self.keys.new_group_modal = None;
            self.key_inputs = None;
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
        self.key_inputs = None;
        cx.notify();
    }

    pub fn close_add_scan_path_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.add_scan_path_modal = None;
        self.key_inputs = None;
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
            self.key_inputs = None;
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
            self.keys.edit_modal = Some(EditKeyModalState {
                key_id: key.id.clone(),
                name_input: key.name.clone(),
                group_id: key.group_id.clone(),
                attached_servers: key.attached_servers.clone(),
                error_message: None,
            });
            self.key_inputs = None;
            cx.notify();
        }
    }

    pub fn close_edit_key_modal(&mut self, cx: &mut Context<Self>) {
        self.keys.edit_modal = None;
        self.key_inputs = None;
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
            self.key_inputs = None;
            self.refresh_keys(cx);
        }
    }

    pub fn toggle_server_attachment(&mut self, server_id: &str, cx: &mut Context<Self>) {
        if let Some(ref mut state) = self.keys.edit_modal {
            if let Some(pos) = state.attached_servers.iter().position(|id| id == server_id) {
                state.attached_servers.remove(pos);
            } else {
                state.attached_servers.push(server_id.to_string());
            }
            cx.notify();
        }
    }

    pub fn clear_key_toast(&mut self, cx: &mut Context<Self>) {
        self.keys.toast = None;
        cx.notify();
    }

    pub fn copy_text_with_toast(&mut self, text: &str, toast: &str, cx: &mut Context<Self>) {
        cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(text.to_string()));
        self.keys.toast = Some(toast.to_string());
        cx.notify();
    }

    pub fn ensure_key_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref gen) = self.keys.gen_modal {
            if !matches!(self.key_inputs, Some(KeyModalInputs::Gen(_))) {
                let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. id_ed25519_bastion").default_value(&gen.name_input));
                let comment = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. nelson@crow").default_value(&gen.comment_input));
                let custom_dir = cx.new(|cx| InputState::new(window, cx).placeholder("~/.ssh").default_value(&gen.custom_dir_input));
                let mut events = Vec::new();
                for (input, field_idx) in [(&name, 0), (&comment, 1), (&custom_dir, 2)] {
                    events.push(cx.subscribe(input, move |this, input, ev: &InputEvent, cx| match ev {
                        InputEvent::Change => {
                            let val = input.read(cx).value().to_string();
                            if let Some(ref mut g) = this.keys.gen_modal {
                                match field_idx {
                                    0 => g.name_input = val,
                                    1 => g.comment_input = val,
                                    2 => g.custom_dir_input = val,
                                    _ => {}
                                }
                                g.error_message = None;
                            }
                            cx.notify();
                        }
                        InputEvent::PressEnter { .. } => {
                            this.submit_key_generation(cx);
                        }
                        _ => {}
                    }));
                }
                self.key_inputs = Some(KeyModalInputs::Gen(KeyGenInputs { name, comment, custom_dir, _events: events }));
            }
        } else if let Some(ref grp) = self.keys.new_group_modal {
            if !matches!(self.key_inputs, Some(KeyModalInputs::NewGroup(_))) {
                let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Staging Fleet, Edge Bastions").default_value(&grp.name_input));
                let events = vec![cx.subscribe(&name, |this, input, ev: &InputEvent, cx| match ev {
                    InputEvent::Change => {
                        if let Some(ref mut g) = this.keys.new_group_modal {
                            g.name_input = input.read(cx).value().to_string();
                            g.error_message = None;
                        }
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        this.submit_new_group(cx);
                    }
                    _ => {}
                })];
                self.key_inputs = Some(KeyModalInputs::NewGroup(NewGroupInputs { name, _events: events }));
            }
        } else if let Some(ref sp) = self.keys.add_scan_path_modal {
            if !matches!(self.key_inputs, Some(KeyModalInputs::AddScanPath(_))) {
                let path = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. ~/work-keys or /etc/ssh").default_value(&sp.path_input));
                let events = vec![cx.subscribe(&path, |this, input, ev: &InputEvent, cx| match ev {
                    InputEvent::Change => {
                        if let Some(ref mut s) = this.keys.add_scan_path_modal {
                            s.path_input = input.read(cx).value().to_string();
                            s.error_message = None;
                        }
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        this.submit_add_scan_path(cx);
                    }
                    _ => {}
                })];
                self.key_inputs = Some(KeyModalInputs::AddScanPath(AddScanPathInputs { path, _events: events }));
            }
        } else if let Some(ref edit) = self.keys.edit_modal {
            if !matches!(self.key_inputs, Some(KeyModalInputs::Edit(_))) {
                let name = cx.new(|cx| InputState::new(window, cx).placeholder("Enter key name…").default_value(&edit.name_input));
                let events = vec![cx.subscribe(&name, |this, input, ev: &InputEvent, cx| match ev {
                    InputEvent::Change => {
                        if let Some(ref mut e) = this.keys.edit_modal {
                            e.name_input = input.read(cx).value().to_string();
                            e.error_message = None;
                        }
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        this.submit_edit_key(cx);
                    }
                    _ => {}
                })];
                self.key_inputs = Some(KeyModalInputs::Edit(EditKeyInputs { name, _events: events }));
            }
        } else {
            self.key_inputs = None;
        }
    }
}
