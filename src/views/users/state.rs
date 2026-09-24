use super::{NewUserState, SystemUserRecord, UserFilterTab};

/// Users screen state: the account list plus modals, filters and toast.
pub struct UsersState {
    pub users: Vec<SystemUserRecord>,
    pub selected_for_ssh: Option<String>,
    pub selected_for_passwd: Option<String>,
    pub show_new_user_modal: bool,
    pub new_user: NewUserState,
    pub search_query: String,
    pub filter_tab: UserFilterTab,
    /// The account shown in the right-hand inspector.
    pub selected: Option<String>,
    /// Account awaiting a second click on DELETE.
    pub confirm_delete: Option<String>,
    pub toast: Option<String>,
    /// The command currently running on the server, if any.
    pub pending: Option<String>,
    /// Why the account list couldn't be read, if it couldn't.
    pub load_error: Option<String>,
}

impl UsersState {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            selected_for_ssh: None,
            selected_for_passwd: None,
            show_new_user_modal: false,
            new_user: NewUserState::default(),
            search_query: String::new(),
            filter_tab: UserFilterTab::Human,
            selected: None,
            confirm_delete: None,
            toast: None,
            pending: None,
            load_error: None,
        }
    }

    /// The accounts the table shows: filter tab, then search (name, full
    /// name, shell or group), in the order they were read.
    pub fn visible(&self) -> Vec<&SystemUserRecord> {
        let q = self.search_query.trim().to_lowercase();
        self.users
            .iter()
            .filter(|u| self.filter_tab.matches(u))
            .filter(|u| {
                q.is_empty()
                    || u.username.to_lowercase().contains(&q)
                    || u.gecos.to_lowercase().contains(&q)
                    || u.shell.to_lowercase().contains(&q)
                    || u.groups.iter().any(|g| g.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// The inspected account: the selection if it's still listed, else the
    /// first visible one.
    pub fn inspected(&self) -> Option<&SystemUserRecord> {
        let visible = self.visible();
        self.selected.as_ref().and_then(|s| visible.iter().find(|u| &u.username == s).copied()).or_else(|| visible.first().copied())
    }

    /// Supplementary groups that exist here (seen on some account), minus
    /// personal groups named after a user and the admin group (offered as
    /// its own toggle).
    pub fn group_choices(&self) -> Vec<String> {
        let mut groups: Vec<String> = self
            .users
            .iter()
            .flat_map(|u| u.groups.iter())
            .filter(|g| !self.users.iter().any(|u| &u.username == *g) && Some(g.as_str()) != self.sudo_group())
            .cloned()
            .collect();
        groups.sort();
        groups.dedup();
        groups
    }

    /// Shells to offer: the common ones plus any an account already uses.
    pub fn shell_choices(&self) -> Vec<String> {
        let mut shells: Vec<String> = ["/bin/bash", "/bin/sh", "/usr/sbin/nologin"].iter().map(|s| s.to_string()).collect();
        for u in &self.users {
            if !u.shell.is_empty() && !shells.contains(&u.shell) {
                shells.push(u.shell.clone());
            }
        }
        shells
    }

    pub fn find(&self, username: &str) -> Option<&SystemUserRecord> {
        self.users.iter().find(|u| u.username == username)
    }

    pub fn open_new_user_modal(&mut self) {
        self.show_new_user_modal = true;
        self.new_user = NewUserState::default();
    }

    /// The admin group this host uses for sudo rights, if it has one.
    pub fn sudo_group(&self) -> Option<&'static str> {
        let has = |g: &str| self.users.iter().any(|u| u.groups.iter().any(|x| x == g));
        if has("sudo") {
            Some("sudo")
        } else if has("wheel") {
            Some("wheel")
        } else {
            None
        }
    }
}
