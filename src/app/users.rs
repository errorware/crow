use gpui_kit::{AppContext, Context, Window};

use super::host_actions::HostCommand;
use super::CrowApp;
use crate::host::host_for;
use crate::vault::ServerRecord;
use crate::views::users::host_data::{self, read_users};
use crate::views::users::{SystemUserRecord, UserSshKeySummary};

// ==========================================
// User accounts: real data and actions on the active server
// ==========================================

fn load(srv: &ServerRecord) -> Result<Vec<SystemUserRecord>, String> {
    let host = host_for(srv);
    read_users(host.as_ref())
}

impl CrowApp {
    /// Reads the active server's accounts in the background.
    pub fn refresh_users(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { load(&srv) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.apply_users(result);
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_users(&mut self, result: Result<Vec<SystemUserRecord>, String>) {
        match result {
            Ok(users) => {
                self.users.users = users;
                self.users.load_error = None;
            }
            Err(e) => self.users.load_error = Some(e),
        }
    }

    /// Runs account commands as root, then re-reads the accounts.
    fn run_user_action(&mut self, target: &str, commands: Result<Vec<HostCommand>, String>, closes_modal: bool, cx: &mut Context<Self>) {
        let commands = match commands {
            Ok(c) => c,
            Err(e) => {
                self.users.toast = Some(format!("Not applied: {e}"));
                cx.notify();
                return;
            }
        };
        self.users.pending = Some(super::host_actions::describe_commands(&commands));
        cx.notify();
        self.run_host_action(
            "user",
            target,
            commands,
            load,
            move |this, users, result, summary, cx| {
                this.apply_users(users);
                this.users.pending = None;
                this.users.toast = Some(match &result {
                    Ok(()) => format!("Ran: {summary}"),
                    Err(e) => format!("Failed: {summary} — {e}"),
                });
                if closes_modal && result.is_ok() {
                    this.users.show_new_user_modal = false;
                }
                cx.notify();
            },
            cx,
        );
    }

    pub fn user_toggle_group(&mut self, user: &str, group: &str, cx: &mut Context<Self>) {
        let member = self.users.find(user).is_some_and(|u| u.groups.iter().any(|g| g == group));
        let cmd = if member { host_data::remove_from_group(user, group) } else { host_data::add_to_group(user, group) };
        self.run_user_action(user, cmd.map(|c| vec![HostCommand::new(c)]), false, cx);
    }

    pub fn user_toggle_lock(&mut self, user: &str, cx: &mut Context<Self>) {
        let locked = self.users.find(user).is_some_and(|u| u.is_locked);
        let cmd = host_data::set_locked(user, !locked);
        self.run_user_action(user, cmd.map(|c| vec![HostCommand::new(c)]), false, cx);
    }

    pub fn user_create(&mut self, cx: &mut Context<Self>) {
        let name = self.users.new_user.username.trim().to_lowercase();
        let cmd = host_data::create_user(&self.users.new_user, self.users.sudo_group());
        self.run_user_action(&name, cmd.map(|c| vec![HostCommand::new(c)]), true, cx);
    }

    /// Opens New User with a fresh form; its inputs are created on render.
    pub fn open_new_user_modal(&mut self, cx: &mut Context<Self>) {
        self.users.open_new_user_modal();
        self.new_user_inputs = None;
        cx.notify();
    }

    /// Creates the New User text inputs (they need the window), focused on
    /// the username; they write through to the form as you type.
    pub fn ensure_new_user_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.new_user_inputs.is_some() {
            return;
        }
        use gpui_kit::component::input::{InputEvent, InputState};
        let username = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. alice, deploy"));
        let gecos = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Alice Wonderland (optional)"));
        let sub_user = cx.subscribe(&username, |this, input, ev: &InputEvent, cx| match ev {
            InputEvent::Change => {
                this.users.new_user.username = input.read(cx).value().to_string();
                cx.notify();
            }
            InputEvent::PressEnter { .. } => this.user_create(cx),
            _ => {}
        });
        let sub_gecos = cx.subscribe(&gecos, |this, input, ev: &InputEvent, cx| match ev {
            InputEvent::Change => {
                this.users.new_user.gecos = input.read(cx).value().to_string();
                cx.notify();
            }
            InputEvent::PressEnter { .. } => this.user_create(cx),
            _ => {}
        });
        username.update(cx, |i, cx| i.focus(window, cx));
        self.new_user_inputs = Some(crate::views::users::NewUserInputs { username, gecos, _events: vec![sub_user, sub_gecos] });
    }

    pub fn select_user(&mut self, user: &str, cx: &mut Context<Self>) {
        self.users.selected = Some(user.to_string());
        self.users.confirm_delete = None;
        cx.notify();
    }

    pub fn set_users_filter(&mut self, tab: crate::views::users::UserFilterTab, cx: &mut Context<Self>) {
        self.users.filter_tab = tab;
        cx.notify();
    }

    pub fn user_set_shell(&mut self, user: &str, shell: &str, cx: &mut Context<Self>) {
        let cmd = host_data::set_shell(user, shell);
        self.run_user_action(user, cmd.map(|c| vec![HostCommand::new(c)]), false, cx);
    }

    /// First click arms DELETE for this account; the second deletes it.
    pub fn user_delete_clicked(&mut self, user: &str, cx: &mut Context<Self>) {
        if self.users.confirm_delete.as_deref() == Some(user) {
            self.users.confirm_delete = None;
            self.user_delete(user, cx);
        } else {
            self.users.confirm_delete = Some(user.to_string());
            cx.notify();
        }
    }

    pub fn user_delete(&mut self, user: &str, cx: &mut Context<Self>) {
        let cmd = host_data::delete_user(user);
        self.run_user_action(user, cmd.map(|c| vec![HostCommand::new(c)]), false, cx);
    }

    pub fn user_attach_key(&mut self, user: &str, key: UserSshKeySummary, cx: &mut Context<Self>) {
        let Some(home) = self.users.find(user).map(|u| u.home_dir.clone()) else { return };
        let cmd = host_data::authorize_key(user, &home, &key.key_preview);
        self.run_user_action(user, cmd.map(|c| vec![HostCommand::new(c)]), false, cx);
    }

    pub fn user_revoke_key(&mut self, user: &str, key_id: &str, cx: &mut Context<Self>) {
        let Some(u) = self.users.find(user) else { return };
        let Some(line) = u.authorized_keys.iter().find(|k| k.id == key_id).map(|k| k.key_preview.clone()) else { return };
        let cmd = host_data::revoke_key(&u.home_dir, &line);
        self.run_user_action(user, Ok(vec![HostCommand::new(cmd)]), false, cx);
    }
}
