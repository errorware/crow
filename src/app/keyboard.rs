use gpui_kit::*;

use super::{CrowApp, Screen};
use crate::keys::KeyGenFieldFocus;
use crate::vault::VaultStatus;
use crate::views::lock::{LockFieldFocus, SetupState, SetupStep};
use crate::views::onboard::OnboardFieldFocus;

// ==========================================
// Global keyboard routing
// ==========================================

/// One key press, pre-digested for the per-surface handlers below.
#[derive(Clone, Copy)]
struct KeyPress<'a> {
    ev: &'a KeyDownEvent,
    /// Lowercased key name.
    key: &'a str,
    /// Platform (⌘) or Control held.
    is_mod: bool,
    is_shift: bool,
    /// A text input inside the window has focus (the key is being typed
    /// into it), so unmodified shortcuts must stay out of the way (ERR-64).
    typing: bool,
}

impl CrowApp {
    /// Toggles the caret blink phase every 530ms while a text input is active.
    pub(super) fn spawn_cursor_blink(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |entity, cx| {
                loop {
                    cx.background_executor().timer(std::time::Duration::from_millis(530)).await;
                    let should_notify = entity.update(cx, |this, cx| {
                        if this.has_active_text_input() {
                            this.caret.blink = !this.caret.blink;
                            cx.notify();
                            true
                        } else {
                            false
                        }
                    });
                    if should_notify.is_err() {
                        break;
                    }
                }
            })
    }

    /// Routes a key press to whichever surface owns the keyboard right now, in
    /// priority order: vault lock screen, vault setup, key-hub modals, settings
    /// dropdown, onboarding, config search, then global shortcuts.
    pub(super) fn handle_key_down(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = ev.keystroke.key.to_lowercase();
        // The root keeps focus unless a gpui Input (or another focusable
        // child) took it; key events still bubble up to here.
        let typing = window.focused(cx).is_some_and(|f| f != self.focus_handle);
        let k = KeyPress {
            ev,
            key: &key,
            is_mod: ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control,
            is_shift: ev.keystroke.modifiers.shift,
            typing,
        };
        // Danger Zone confirm and Files new-folder prompt are native
        // gpui-component Input widgets — they own their own focus and
        // keyboard handling, so they have no entry here.
        let _handled = self.keys_lock_screen(&k, cx)
            || self.keys_vault_setup(&k, cx)
            || self.keys_modals(&k, cx)
            || self.keys_settings_dropdown(&k, cx)
            || self.keys_onboard(&k, cx)
            || self.keys_config_search(&k, cx)
            || self.keys_global_shortcuts(&k, cx);
    }

    /// Vault locked: keyboard input is dedicated to unlocking. Returns true when the key was consumed.
    fn keys_lock_screen(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, .. } = *k;
        if self.vault.status() == VaultStatus::Locked {
            self.caret.blink = true;
            if key == "tab" {
                self.lock_state.active_focus = match self.lock_state.active_focus {
                    LockFieldFocus::Password => LockFieldFocus::Totp,
                    LockFieldFocus::Totp => LockFieldFocus::Password,
                };
                let len = match self.lock_state.active_focus {
                    LockFieldFocus::Password => self.lock_state.password_input.chars().count(),
                    LockFieldFocus::Totp => self.lock_state.totp_input.chars().count(),
                };
                self.caret.place(len);
                cx.notify();
            } else if key == "enter" {
                self.submit_unlock(cx);
            } else {
                let is_totp = self.lock_state.active_focus == LockFieldFocus::Totp;
                let text = match self.lock_state.active_focus {
                    LockFieldFocus::Password => &mut self.lock_state.password_input,
                    LockFieldFocus::Totp => &mut self.lock_state.totp_input,
                };
                let changed = crate::components::handle_text_key_event(text, &mut self.caret.cursor, &mut self.caret.selection, ev);
                if is_totp {
                    self.lock_state.totp_input.retain(|c| c.is_ascii_digit());
                    if self.lock_state.totp_input.chars().count() > 6 {
                        let s: String = self.lock_state.totp_input.chars().take(6).collect();
                        self.lock_state.totp_input = s;
                        self.caret.cursor = self.caret.cursor.min(6);
                    }
                }
                if changed {
                    self.lock_state.error_message = None;
                    cx.notify();
                }
            }
            return true;
        }
        false
    }

    /// Vault setup wizard. Returns true when the key was consumed.
    fn keys_vault_setup(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, .. } = *k;
        if self.screen == Screen::VaultSetup {
            match self.setup_state.step {
                SetupStep::WarningNotice => {
                    if ev.keystroke.key == "escape" {
                        self.setup_state = SetupState::default();
                        self.set_screen(Screen::Settings, cx);
                    } else if key == "enter" {
                        self.enter_setup_credentials(cx);
                    }
                }
                SetupStep::ConfigureCredentials => {
                    // The fields are gpui inputs (Enter activates through
                    // them); only Escape, to go back, is handled here.
                    if ev.keystroke.key == "escape" {
                        self.setup_state.step = SetupStep::WarningNotice;
                        cx.notify();
                    } else {
                        return false;
                    }
                }
            }
            return true;
        }
        false
    }

    /// SSH key hub and Clankers edit modals. Returns true when the key was consumed.
    fn keys_modals(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, .. } = *k;
        if let Some(ref mut gen) = self.keys.gen_modal {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.close_key_gen_modal(cx);
            } else if key == "enter" {
                if gen.generated_public_key.is_some() {
                    self.close_key_gen_modal(cx);
                } else {
                    self.submit_key_generation(cx);
                }
            } else if key == "tab" {
                gen.active_focus = match gen.active_focus {
                    KeyGenFieldFocus::Name => KeyGenFieldFocus::Comment,
                    KeyGenFieldFocus::Comment => KeyGenFieldFocus::Directory,
                    KeyGenFieldFocus::Directory => KeyGenFieldFocus::Name,
                };
                self.caret.cursor = match gen.active_focus {
                    KeyGenFieldFocus::Name => gen.name_input.chars().count(),
                    KeyGenFieldFocus::Comment => gen.comment_input.chars().count(),
                    KeyGenFieldFocus::Directory => gen.custom_dir_input.chars().count(),
                };
                self.caret.selection = None;
                cx.notify();
            } else {
                let target = match gen.active_focus {
                    KeyGenFieldFocus::Name => &mut gen.name_input,
                    KeyGenFieldFocus::Comment => &mut gen.comment_input,
                    KeyGenFieldFocus::Directory => &mut gen.custom_dir_input,
                };
                if crate::components::handle_text_key_event(
                    target,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ) {
                    gen.error_message = None;
                    cx.notify();
                }
            }
            return true;
        }

        if let Some(ref mut grp) = self.keys.new_group_modal {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.close_new_group_modal(cx);
            } else if key == "enter" {
                self.submit_new_group(cx);
            } else if crate::components::handle_text_key_event(
                &mut grp.name_input,
                &mut self.caret.cursor,
                &mut self.caret.selection,
                ev,
            ) {
                grp.error_message = None;
                cx.notify();
            }
            return true;
        }

        if let Some(ref mut sp) = self.keys.add_scan_path_modal {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.close_add_scan_path_modal(cx);
            } else if key == "enter" {
                self.submit_add_scan_path(cx);
            } else if crate::components::handle_text_key_event(
                &mut sp.path_input,
                &mut self.caret.cursor,
                &mut self.caret.selection,
                ev,
            ) {
                sp.error_message = None;
                cx.notify();
            }
            return true;
        }

        if let Some(ref mut edit) = self.keys.edit_modal {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.close_edit_key_modal(cx);
            } else if key == "enter" {
                self.submit_edit_key(cx);
            } else if crate::components::handle_text_key_event(
                &mut edit.name_input,
                &mut self.caret.cursor,
                &mut self.caret.selection,
                ev,
            ) {
                edit.error_message = None;
                cx.notify();
            }
            return true;
        }

        false
    }

    /// Settings screen with a dropdown open: typing, escape, enter. Returns true when the key was consumed.
    fn keys_settings_dropdown(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, .. } = *k;
        if self.screen == Screen::Settings {
            if let Some(open_row_id) = self.settings.dropdown_open.clone() {
                self.caret.blink = true;
                if ev.keystroke.key == "escape" {
                    self.close_settings_dropdown(cx);
                    return true;
                } else if key == "enter" {
                    self.apply_settings_custom_input(&open_row_id, cx);
                    return true;
                } else if crate::components::handle_text_key_event(
                    &mut self.settings.custom_input,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ) {
                    let field_is_int = self
                        .config
                        .get_field(&open_row_id)
                        .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Integer))
                        .unwrap_or(false);
                    if field_is_int {
                        self.settings.custom_input.retain(|c| c.is_ascii_digit());
                        self.caret.cursor = self.caret.cursor.min(self.settings.custom_input.chars().count());
                    }
                    cx.notify();
                    return true;
                }
            }
        }
        false
    }

    /// Onboarding wizard keys. The text fields are real inputs that handle
    /// their own typing, paste and selection (and Enter); this only handles
    /// Escape, Tab between fields, and Enter when no field has focus.
    fn keys_onboard(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, is_shift, .. } = *k;
        if self.screen != Screen::Onboard {
            return false;
        }
        if ev.keystroke.key == "escape" {
            self.set_screen(Screen::Fleet, cx);
            true
        } else if key == "tab" {
            self.onboard_cycle_focus(is_shift, cx);
            true
        } else if key == "enter" && self.onboard_state.focus == OnboardFieldFocus::None {
            self.onboard_next_step(cx);
            true
        } else {
            false
        }
    }

    /// Config screen file search box. Returns true when the key was consumed.
    fn keys_config_search(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, .. } = *k;
        if self.screen == Screen::Server && (self.active_view == "config" || self.active_view == "configure") && self.configs.search_focused {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.configs.search_focused = false;
                self.configs.search_query.clear();
                cx.notify();
                return true;
            } else {
                let changed = crate::components::handle_text_key_event(
                    &mut self.configs.search_query,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                );
                if changed {
                    cx.notify();
                }
                return true;
            }
        }
        false
    }

    /// Global shortcuts (escape, palette, sidebar, screens). Returns true when the key was consumed.
    fn keys_global_shortcuts(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, is_mod, is_shift, typing } = *k;
        if typing && !is_mod {
            // Escape closes the form being typed into, and nothing else;
            // other plain keys belong to the input.
            if ev.keystroke.key == "escape" {
                if self.firewall.show_new_rule_modal {
                    self.close_new_firewall_rule_modal(cx);
                } else if self.users.show_new_user_modal {
                    self.users.show_new_user_modal = false;
                    cx.notify();
                } else if self.clanker_inputs.is_some() {
                    self.close_edit_clanker_modal(cx);
                } else if self.provider_inputs.is_some() {
                    self.close_provider_form(cx);
                } else if self.vault_form.open.is_some() {
                    self.open_vault_form(None, cx);
                }
            }
            return false;
        }
        if ev.keystroke.key == "escape" {
            if self.import.open {
                self.close_import(cx);
            } else if self.stance_panel_open {
                self.toggle_stance_panel(cx);
            } else if self.firewall.show_new_rule_modal {
                self.close_new_firewall_rule_modal(cx);
            } else if self.users.show_new_user_modal {
                self.users.show_new_user_modal = false; cx.notify();
            } else if self.show_about_modal {
                self.close_about_modal(cx);
            } else if self.menu_open {
                self.menu_open = false;
                cx.notify();
            } else if self.palette_open {
                self.palette_open = false;
                cx.notify();
            } else if self.screen != Screen::Server && self.screen != Screen::Fleet {
                self.set_screen(Screen::Fleet, cx);
            }
        } else if key == "l" && is_mod && is_shift {
            self.lock(cx);
        } else if key == "k" && is_mod {
            self.toggle_palette(cx);
        } else if key == "\\" && is_mod {
            self.sidebar_collapsed = !self.sidebar_collapsed;
            cx.notify();
        } else if key == "1" && is_mod {
            self.set_screen(Screen::Fleet, cx);
        } else if key == "2" && is_mod {
            self.set_screen(Screen::Server, cx);
            self.set_view("overview", cx);
        } else if key == "3" && is_mod {
            self.set_screen(Screen::Server, cx);
            self.set_view("config", cx);
        } else if key == "4" && is_mod {
            self.set_screen(Screen::Server, cx);
            self.set_view("logs", cx);
        } else if key == "," && is_mod {
            self.set_screen(Screen::Settings, cx);
        } else if key == "s" && is_mod && self.screen == Screen::Settings {
            self.save_config(cx);
        } else if key == "/" && is_mod && self.screen == Screen::Settings {
            self.open_config_file();
        } else if key == "n" && is_mod {
            self.start_onboarding(cx);
        } else if key == "/" && !is_mod && self.screen == Screen::Server && super::overview::TablePage::for_view(&self.active_view).is_some() {
            self.table_search_focus_pending = true;
            cx.notify();
        } else if key == "f" && is_mod && is_shift {
            self.set_screen(Screen::FleetSetup, cx);
        }
        false
    }
}
