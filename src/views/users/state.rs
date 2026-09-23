use super::{default_system_users, NewUserState, SystemUserRecord, UserFilterTab, UserSshKeySummary};

/// Users screen state: the account list plus modals, filters and toast.
pub struct UsersState {
    pub users: Vec<SystemUserRecord>,
    pub selected_for_ssh: Option<String>,
    pub selected_for_passwd: Option<String>,
    pub show_new_user_modal: bool,
    pub new_user: NewUserState,
    pub search_query: String,
    pub filter_tab: UserFilterTab,
    pub toast: Option<String>,
}

impl UsersState {
    pub fn new() -> Self {
        Self {
            users: default_system_users(),
            selected_for_ssh: None,
            selected_for_passwd: None,
            show_new_user_modal: false,
            new_user: NewUserState::default(),
            search_query: String::new(),
            filter_tab: UserFilterTab::All,
            toast: None,
        }
    }

    fn find_mut(&mut self, username: &str) -> Option<&mut SystemUserRecord> {
        self.users.iter_mut().find(|u| u.username == username)
    }

    pub fn open_new_user_modal(&mut self) {
        self.show_new_user_modal = true;
        self.new_user = NewUserState::default();
    }

    pub fn toggle_group(&mut self, username: &str, group: &str) {
        let Some(user) = self.find_mut(username) else { return };
        let toast = if user.groups.iter().any(|g| g == group) {
            user.groups.retain(|g| g != group);
            format!("Executed: gpasswd -d {} {}", username, group)
        } else {
            user.groups.push(group.to_string());
            format!("Executed: usermod -aG {} {}", group, username)
        };
        self.toast = Some(toast);
    }

    pub fn toggle_lock(&mut self, username: &str) {
        let Some(user) = self.find_mut(username) else { return };
        user.is_locked = !user.is_locked;
        let toast = if user.is_locked {
            format!("Executed: passwd -l {} (Account Locked)", username)
        } else {
            format!("Executed: passwd -u {} (Account Unlocked)", username)
        };
        self.toast = Some(toast);
    }

    pub fn cycle_shell(&mut self, username: &str) {
        let Some(user) = self.find_mut(username) else { return };
        let next_shell = match user.shell.as_str() {
            "/bin/bash" => "/bin/zsh",
            "/bin/zsh" => "/usr/bin/fish",
            "/usr/bin/fish" => "/usr/sbin/nologin",
            _ => "/bin/bash",
        };
        user.shell = next_shell.to_string();
        self.toast = Some(format!("Executed: chsh -s {} {}", next_shell, username));
    }

    pub fn attach_key(&mut self, username: &str, key: UserSshKeySummary) {
        let Some(user) = self.find_mut(username) else { return };
        if !user.authorized_keys.iter().any(|k| k.id == key.id || k.fingerprint == key.fingerprint) {
            let key_name = key.name.clone();
            user.authorized_keys.push(key);
            self.toast = Some(format!("Authorized SSH Key '{}' for user {}", key_name, username));
        }
    }

    pub fn revoke_key(&mut self, username: &str, key_id: &str) {
        let Some(user) = self.find_mut(username) else { return };
        user.authorized_keys.retain(|k| k.id != key_id);
        self.toast = Some(format!("Revoked SSH Key from user {}", username));
    }

    pub fn submit_create_user(&mut self) {
        let username = self.new_user.username.trim().to_lowercase();
        if username.is_empty() {
            return;
        }
        if self.users.iter().any(|u| u.username == username) {
            self.toast = Some(format!("Error: user '{}' already exists", username));
            return;
        }
        let next_uid = self.users.iter().map(|u| u.uid).max().unwrap_or(1000).max(1000) + 1;
        let mut groups = vec![username.clone()];
        for g in &self.new_user.selected_groups {
            if !groups.contains(g) {
                groups.push(g.clone());
            }
        }
        let home_dir = if self.new_user.create_home {
            format!("/home/{}", username)
        } else {
            "/nonexistent".to_string()
        };

        self.users.push(SystemUserRecord {
            username: username.clone(),
            uid: next_uid,
            gid: next_uid,
            gecos: self.new_user.gecos.clone(),
            home_dir,
            shell: self.new_user.shell.clone(),
            primary_group: username.clone(),
            groups,
            is_system_user: false,
            is_locked: false,
            authorized_keys: vec![],
            last_login: None,
        });
        self.show_new_user_modal = false;
        self.toast = Some(format!("Provisioned user '{}' (UID {})", username, next_uid));
    }

    pub fn delete_user(&mut self, username: &str) {
        if username == "root" {
            self.toast = Some("Cannot delete root superuser account".to_string());
            return;
        }
        self.users.retain(|u| u.username != username);
        self.toast = Some(format!("Deleted user account '{}'", username));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user_assigns_next_uid_and_rejects_duplicates() {
        let mut st = UsersState::new();
        st.new_user.username = "  Deploy ".into();
        st.submit_create_user();
        let created = st.users.iter().find(|u| u.username == "deploy").expect("user created");
        assert!(created.uid > 1000);
        let count = st.users.len();
        st.submit_create_user();
        assert_eq!(st.users.len(), count);
        assert!(st.toast.as_deref().unwrap().contains("already exists"));
    }

    #[test]
    fn root_cannot_be_deleted() {
        let mut st = UsersState::new();
        st.delete_user("root");
        assert!(st.users.iter().any(|u| u.username == "root"));
    }
}
