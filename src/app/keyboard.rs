use gpui_kit::*;

use super::CrowApp;
use crate::app::Screen;
use crate::vault::VaultStatus;
use crate::views::lock::SetupStep;
use crate::app::ClankerModalFocus;
use crate::keys::KeyGenFieldFocus;
use crate::views::lock::SetupState;
use crate::views::lock::LockFieldFocus;
use crate::views::lock::SetupFieldFocus;
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
}

impl CrowApp {
    /// Routes a key press to whichever surface owns the keyboard right now, in
    /// priority order: vault lock screen, vault setup, key-hub modals, settings
    /// dropdown, onboarding, log search, config search, then global shortcuts.
    pub(super) fn handle_key_down(&mut self, ev: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = ev.keystroke.key.to_lowercase();
        let k = KeyPress {
            ev,
            key: &key,
            is_mod: ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control,
            is_shift: ev.keystroke.modifiers.shift,
        };
        // Danger Zone confirm and Files new-folder prompt are native
        // gpui-component Input widgets — they own their own focus and
        // keyboard handling, so they have no entry here.
        let _handled = self.keys_lock_screen(&k, cx)
            || self.keys_vault_setup(&k, cx)
            || self.keys_modals(&k, cx)
            || self.keys_settings_dropdown(&k, cx)
            || self.keys_onboard(&k, cx)
            || self.keys_log_search(&k, cx)
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
                        self.setup_state.step = SetupStep::ConfigureCredentials;
                        if self.setup_state.totp_secret.is_empty() {
                            self.setup_state.totp_secret = crate::vault::generate_totp_secret();
                        }
                        self.caret.place(self.setup_state.password_input.chars().count());
                        self.caret.blink = true;
                        cx.notify();
                    }
                }
                SetupStep::ConfigureCredentials => {
                    self.caret.blink = true;
                    if ev.keystroke.key == "escape" {
                        self.setup_state.step = SetupStep::WarningNotice;
                        cx.notify();
                    } else if key == "tab" {
                        self.setup_state.active_focus = match self.setup_state.active_focus {
                            SetupFieldFocus::Password => SetupFieldFocus::ConfirmPassword,
                            SetupFieldFocus::ConfirmPassword => SetupFieldFocus::TotpConfirm,
                            SetupFieldFocus::TotpConfirm => SetupFieldFocus::Password,
                        };
                        let len = match self.setup_state.active_focus {
                            SetupFieldFocus::Password => self.setup_state.password_input.chars().count(),
                            SetupFieldFocus::ConfirmPassword => self.setup_state.confirm_input.chars().count(),
                            SetupFieldFocus::TotpConfirm => self.setup_state.totp_confirm_input.chars().count(),
                        };
                        self.caret.place(len);
                        cx.notify();
                    } else if key == "enter" {
                        self.submit_setup(cx);
                    } else {
                        let is_totp = self.setup_state.active_focus == SetupFieldFocus::TotpConfirm;
                        let text = match self.setup_state.active_focus {
                            SetupFieldFocus::Password => &mut self.setup_state.password_input,
                            SetupFieldFocus::ConfirmPassword => &mut self.setup_state.confirm_input,
                            SetupFieldFocus::TotpConfirm => &mut self.setup_state.totp_confirm_input,
                        };
                        let changed = crate::components::handle_text_key_event(text, &mut self.caret.cursor, &mut self.caret.selection, ev);
                        if is_totp {
                            self.setup_state.totp_confirm_input.retain(|c| c.is_ascii_digit());
                            if self.setup_state.totp_confirm_input.chars().count() > 6 {
                                let s: String = self.setup_state.totp_confirm_input.chars().take(6).collect();
                                self.setup_state.totp_confirm_input = s;
                                self.caret.cursor = self.caret.cursor.min(6);
                            }
                        }
                        if changed {
                            self.setup_state.error_message = None;
                            cx.notify();
                        }
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

        if let Some(ref mut clk) = self.clankers.editing {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.close_edit_clanker_modal(cx);
                return true;
            } else if key == "enter" {
                self.submit_edit_clanker(cx);
                return true;
            } else if key == "tab" {
                clk.focus = match clk.focus {
                    ClankerModalFocus::ApiKey => ClankerModalFocus::Model,
                    ClankerModalFocus::Model => ClankerModalFocus::BaseUrl,
                    ClankerModalFocus::BaseUrl => ClankerModalFocus::ApiKey,
                };
                let target_len = match clk.focus {
                    ClankerModalFocus::ApiKey => clk.api_key_input.chars().count(),
                    ClankerModalFocus::Model => clk.model_input.chars().count(),
                    ClankerModalFocus::BaseUrl => clk.base_url_input.chars().count(),
                };
                self.caret.place(target_len);
                cx.notify();
                return true;
            }

            let handled = match clk.focus {
                ClankerModalFocus::ApiKey => crate::components::handle_text_key_event(
                    &mut clk.api_key_input,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                ClankerModalFocus::Model => crate::components::handle_text_key_event(
                    &mut clk.model_input,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                ClankerModalFocus::BaseUrl => crate::components::handle_text_key_event(
                    &mut clk.base_url_input,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
            };
            if handled {
                clk.error_message = None;
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
                        .map(|f| matches!(&f.field_type, crow_config_core::schema::FieldType::Other(cow) if cow == "integer"))
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

    /// Onboarding wizard fields. Returns true when the key was consumed.
    fn keys_onboard(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, is_shift, .. } = *k;
        if self.screen == Screen::Onboard {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.set_screen(Screen::Fleet, cx);
                return true;
            } else if key == "enter" {
                self.onboard_next_step(cx);
                return true;
            } else if key == "tab" {
                let shift = is_shift;
                self.onboard_cycle_focus(shift, cx);
                return true;
            }

            let handled = match self.onboard_state.focus {
                OnboardFieldFocus::Host => crate::components::handle_text_key_event(
                    &mut self.onboard_state.host,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                OnboardFieldFocus::Port => {
                    let res = crate::components::handle_text_key_event(
                        &mut self.onboard_state.port,
                        &mut self.caret.cursor,
                        &mut self.caret.selection,
                        ev,
                    );
                    if res {
                        self.onboard_state.port.retain(|c| c.is_ascii_digit());
                        if self.onboard_state.port.len() > 5 {
                            self.onboard_state.port.truncate(5);
                            self.caret.cursor = self.caret.cursor.min(self.onboard_state.port.len());
                        }
                    }
                    res
                }
                OnboardFieldFocus::User => crate::components::handle_text_key_event(
                    &mut self.onboard_state.user,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                OnboardFieldFocus::Password => crate::components::handle_text_key_event(
                    &mut self.onboard_state.password,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                OnboardFieldFocus::Label => crate::components::handle_text_key_event(
                    &mut self.onboard_state.label,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                OnboardFieldFocus::Tags => crate::components::handle_text_key_event(
                    &mut self.onboard_state.tags,
                    &mut self.caret.cursor,
                    &mut self.caret.selection,
                    ev,
                ),
                OnboardFieldFocus::None => false,
            };

            if handled {
                self.onboard_state.error_message = None;
                cx.notify();
                return true;
            }
        }
        false
    }

    /// Logs screen journal search box. Returns true when the key was consumed.
    fn keys_log_search(&mut self, k: &KeyPress, cx: &mut Context<Self>) -> bool {
        let KeyPress { ev, key, .. } = *k;
        if self.screen == Screen::Server && self.active_view == "logs" && self.journal.search_focused {
            self.caret.blink = true;
            if ev.keystroke.key == "escape" {
                self.journal.search_focused = false;
                cx.notify();
                return true;
            } else if key == "enter" {
                self.run_journal_query(cx);
                return true;
            } else {
                let changed = crate::components::handle_text_key_event(
                    &mut self.journal.search,
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
        let KeyPress { ev, key, is_mod, is_shift, .. } = *k;
        if ev.keystroke.key == "escape" {
            if self.firewall.show_new_rule_modal {
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
        } else if key == "f" && is_mod && is_shift {
            self.set_screen(Screen::FleetSetup, cx);
        }
        false
    }
}
