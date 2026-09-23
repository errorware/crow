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
            filter_tab: UserFilterTab::All,
            toast: None,
            pending: None,
            load_error: None,
        }
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
