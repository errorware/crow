//! Team (ERR-150): members, sign-in, and the permission check every
//! server-changing action goes through.

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::team::{self, Member, Permission, Role, Scope, TeamConfig};
use crate::vault::ServerRecord;

#[derive(Default)]
pub struct TeamState {
    pub config: TeamConfig,
    /// The member at the keyboard.
    pub active: Option<String>,
    /// Sign-in: whose passphrase is being typed.
    pub signing_in: Option<String>,
    pub passphrase: Option<Entity<InputState>>,
    pub new_name: Option<Entity<InputState>>,
    pub new_pass: Option<Entity<InputState>>,
    pub new_role: Option<Role>,
    pub new_scope_groups: Vec<String>,
    pub new_scope_envs: Vec<String>,
    pub approval: Option<Entity<InputState>>,
    pub message: Option<(bool, String)>,
    /// A refusal or sign-in note, shown on every screen.
    pub notice: Option<(bool, String)>,
    /// The approval passphrase is used once: emptied at the next render.
    pub clear_approval: bool,
}

impl CrowApp {
    pub fn load_team(&mut self) {
        self.team.config = self.vault.db().lock().ok().and_then(|db| db.flag(team::TEAM_FLAG)).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        crate::vault::set_current_actor(None);
    }

    fn save_team(&self) {
        if let (Ok(db), Ok(json)) = (self.vault.db().lock(), serde_json::to_string(&self.team.config)) {
            let _ = db.set_flag(team::TEAM_FLAG, &json);
        }
    }

    pub fn active_member(&self) -> Option<&Member> {
        self.team.active.as_ref().and_then(|id| self.team.config.find(id))
    }

    /// May the member at the keyboard do `perm` (on `server`)?
    pub fn may(&self, perm: Permission, server: Option<&ServerRecord>) -> Result<(), String> {
        self.team.config.may(self.active_member(), perm, server)
    }

    /// `may`, telling the user when it's refused.
    pub fn allowed(&mut self, perm: Permission, server: Option<&ServerRecord>, cx: &mut Context<Self>) -> bool {
        match self.may(perm, server) {
            Ok(()) => true,
            Err(why) => {
                self.team_notice(false, format!("Not allowed: {why}."), cx);
                false
            }
        }
    }

    /// Shows `text` for a few seconds, whatever screen is up.
    pub fn team_notice(&mut self, ok: bool, text: String, cx: &mut Context<Self>) {
        self.team.notice = Some((ok, text.clone()));
        cx.notify();
        cx.spawn(async move |entity, cx| {
            cx.background_executor().timer(std::time::Duration::from_secs(6)).await;
            let _ = entity.update(cx, |this, cx| {
                if this.team.notice.as_ref().is_some_and(|(_, t)| *t == text) {
                    this.team.notice = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn ensure_team_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let t = &mut self.team;
        if t.passphrase.is_none() {
            t.passphrase = Some(cx.new(|cx| InputState::new(window, cx).placeholder("passphrase").masked(true)));
            t.new_name = Some(cx.new(|cx| InputState::new(window, cx).placeholder("name, e.g. ana")));
            t.new_pass = Some(cx.new(|cx| InputState::new(window, cx).placeholder("their passphrase (8+ characters; they can change it later)").masked(true)));
            t.approval = Some(cx.new(|cx| InputState::new(window, cx).placeholder("a second member's passphrase").masked(true)));
        }
        if std::mem::take(&mut t.clear_approval) {
            if let Some(i) = &t.approval {
                i.update(cx, |x, cx| x.set_value("", window, cx));
            }
        }
    }

    pub fn choose_signin(&mut self, id: &str, cx: &mut Context<Self>) {
        self.team.signing_in = Some(id.to_string());
        self.team.message = None;
        cx.notify();
    }

    pub fn sign_in(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.team.signing_in.clone() else { return };
        let Some(m) = self.team.config.find(&id).cloned() else { return };
        let pass = self.team.passphrase.as_ref().map(|i| i.read(cx).value().to_string()).unwrap_or_default();
        if let Some(i) = &self.team.passphrase {
            i.update(cx, |x, cx| x.set_value("", window, cx));
        }
        if team::check_passphrase(&m, &pass) {
            self.team.active = Some(m.id.clone());
            self.team.signing_in = None;
            self.team.message = None;
            crate::vault::set_current_actor(Some(m.name.clone()));
            self.team_notice(true, format!("Signed in as {} ({}).", m.name, m.role.name()), cx);
        } else {
            self.team.message = Some((false, "That passphrase isn't right.".into()));
        }
        cx.notify();
    }

    pub fn sign_out(&mut self, cx: &mut Context<Self>) {
        self.team.active = None;
        crate::vault::set_current_actor(None);
        cx.notify();
    }

    pub fn set_new_role(&mut self, role: Role, cx: &mut Context<Self>) {
        self.team.new_role = Some(role);
        cx.notify();
    }

    pub fn toggle_new_scope(&mut self, group: Option<String>, env: Option<String>, cx: &mut Context<Self>) {
        if let Some(g) = group {
            if !self.team.new_scope_groups.contains(&g) {
                self.team.new_scope_groups.push(g);
            } else {
                self.team.new_scope_groups.retain(|x| *x != g);
            }
        }
        if let Some(e) = env {
            if !self.team.new_scope_envs.contains(&e) {
                self.team.new_scope_envs.push(e);
            } else {
                self.team.new_scope_envs.retain(|x| *x != e);
            }
        }
        cx.notify();
    }

    /// Adds a member. The first one must be an Admin (and is signed in).
    pub fn add_member(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.team.config.enabled() && !self.allowed(Permission::Team, None, cx) {
            return;
        }
        let name = self.team.new_name.as_ref().map(|i| i.read(cx).value().trim().to_string()).unwrap_or_default();
        let pass = self.team.new_pass.as_ref().map(|i| i.read(cx).value().to_string()).unwrap_or_default();
        let first = !self.team.config.enabled();
        let role = if first { Role::Admin } else { self.team.new_role.clone().unwrap_or(Role::Viewer) };
        if name.is_empty() || self.team.config.members.iter().any(|m| m.name.eq_ignore_ascii_case(&name)) {
            self.team.message = Some((false, "Give the member a name no one else has.".into()));
            cx.notify();
            return;
        }
        let verifier = match team::hash_passphrase(&pass) {
            Ok(v) => v,
            Err(e) => {
                self.team.message = Some((false, e));
                cx.notify();
                return;
            }
        };
        let scope = if !self.team.new_scope_envs.is_empty() {
            Scope::Envs(self.team.new_scope_envs.clone())
        } else if !self.team.new_scope_groups.is_empty() {
            Scope::Groups(self.team.new_scope_groups.clone())
        } else {
            Scope::Fleet
        };
        let m = Member { id: format!("m{}", chrono::Utc::now().timestamp_millis()), name: name.clone(), role: role.clone(), scope, verifier };
        self.team.config.members.push(m.clone());
        self.save_team();
        for i in [&self.team.new_name, &self.team.new_pass].into_iter().flatten() {
            i.update(cx, |x, cx| x.set_value("", window, cx));
        }
        self.team.new_scope_groups.clear();
        self.team.new_scope_envs.clear();
        if first {
            self.team.active = Some(m.id.clone());
            crate::vault::set_current_actor(Some(m.name.clone()));
            self.team.message = Some((true, format!("Team mode is on: {name} is the first Admin and is signed in. From now on Crow asks who's at the keyboard when it starts.")));
        } else {
            self.team.message = Some((true, format!("Added {name} as {}.", role.name())));
        }
        cx.notify();
    }

    pub fn remove_member(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(Permission::Team, None, cx) {
            return;
        }
        if self.team.config.last_admin(id) && self.team.config.members.len() > 1 {
            self.team.message = Some((false, "That's the last Admin: make someone else Admin first.".into()));
            cx.notify();
            return;
        }
        self.team.config.members.retain(|m| m.id != id);
        if self.team.active.as_deref() == Some(id) {
            self.sign_out(cx);
        }
        self.save_team();
        if !self.team.config.enabled() {
            self.team.message = Some((true, "No members left: team mode is off, and Crow is single-user again.".into()));
        }
        cx.notify();
    }

    pub fn toggle_approval(&mut self, cx: &mut Context<Self>) {
        if !self.allowed(Permission::Team, None, cx) {
            return;
        }
        self.team.config.require_approval = !self.team.config.require_approval;
        self.save_team();
        cx.notify();
    }

    /// With approval required: someone other than the member at the
    /// keyboard, allowed fleet runs, typed their passphrase.
    pub fn check_approval(&mut self, cx: &mut Context<Self>) -> Result<String, String> {
        if !self.team.config.enabled() || !self.team.config.require_approval {
            return Ok(String::new());
        }
        let pass = self.team.approval.as_ref().map(|i| i.read(cx).value().to_string()).unwrap_or_default();
        let me = self.team.active.clone().unwrap_or_default();
        self.team
            .config
            .members
            .iter()
            .find(|m| m.id != me && m.role.allows(Permission::RunFleet) && team::check_passphrase(m, &pass))
            .map(|m| m.name.clone())
            .ok_or_else(|| "needs a second member's approval: someone else allowed fleet runs types their passphrase".into())
    }
}
