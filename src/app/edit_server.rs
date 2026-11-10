//! Edit server (ERR-166): a server's name, environment, role, group, tags,
//! and how Crow reaches it (login user, key, direct or through a bastion),
//! changed after it was added. A change to how Crow reaches it is tried
//! first and saved only if Crow can still log in. The address and port
//! aren't here: a new address has a new host key to pin, which is what
//! adding the server again does.

use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::CrowApp;
use crate::host::{host_for, transport_kind, Host, TransportKind, DEFAULT_TIMEOUT};
use crate::vault::{ChangeRecord, ServerRecord};

/// Tags Crow sets itself: kept as they are, never shown for editing.
const OWN_TAGS: [&str; 6] = ["bastion", "test-node", "local", "podman", "docker", "distrobox"];

pub struct EditServer {
    pub server_id: String,
    pub name: Entity<InputState>,
    pub role: Entity<InputState>,
    pub tags: Entity<InputState>,
    pub login: Entity<InputState>,
    pub env: String,
    pub group: String,
    pub key_id: Option<String>,
    /// The bastion it goes through; `None` = direct.
    pub jump: Option<String>,
    /// Enrolled in the background `crow --agent` monitor (ERR-209).
    pub agent_enrolled: bool,
    /// Trying the new way in before saving.
    pub testing: bool,
    pub error: Option<String>,
}

/// The tags a user sees and edits: the server's, minus Crow's own.
pub fn user_tags(tags: &[String]) -> String {
    tags.iter().filter(|t| !OWN_TAGS.contains(&t.as_str())).cloned().collect::<Vec<_>>().join(", ")
}

/// `typed` (comma separated) with Crow's own tags from `old` kept.
pub fn merge_tags(old: &[String], typed: &str) -> Vec<String> {
    let mut out: Vec<String> = old.iter().filter(|t| OWN_TAGS.contains(&t.as_str())).cloned().collect();
    for t in typed.split(',').map(str::trim).filter(|t| !t.is_empty() && !t.chars().any(char::is_control)) {
        if !out.iter().any(|o| o == t) {
            out.push(t.to_string());
        }
    }
    out
}

/// What's wrong with `new` as an edit of `old` among `servers`, if anything.
pub fn check(new: &ServerRecord, servers: &[ServerRecord]) -> Result<(), String> {
    let name = new.name.trim();
    if name.is_empty() {
        return Err(t!("app-edit-server-needs-a-name").to_string());
    }
    if name.chars().any(char::is_control) {
        return Err(t!("app-edit-server-name-control").to_string());
    }
    if servers.iter().any(|s| s.id != new.id && s.archived_at.is_none() && s.name.eq_ignore_ascii_case(name)) {
        return Err(t!("app-edit-server-name-taken", name = name));
    }
    // A lab container is reached by its name: renaming it would lose it.
    let old = servers.iter().find(|s| s.id == new.id);
    if old.is_some_and(|o| transport_kind(o) == TransportKind::Container && o.name != new.name) {
        return Err(t!("app-edit-server-container-name").to_string());
    }
    if new.login_user.trim().is_empty() || new.login_user.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(t!("app-edit-server-login-user").to_string());
    }
    if let Some(j) = &new.jump_host_id {
        if !crate::app::bastions::choices(Some(&new.id), servers).iter().any(|b| &b.id == j) {
            return Err(t!("app-edit-server-not-a-bastion").to_string());
        }
    }
    Ok(())
}

/// Whether the edit changes how Crow reaches the server.
pub fn reach_changed(old: &ServerRecord, new: &ServerRecord) -> bool {
    old.login_user != new.login_user || old.key_id != new.key_id || old.jump_host_id != new.jump_host_id
}

/// One line per changed field, for the audit log: `role: web → api`.
pub fn describe(old: &ServerRecord, new: &ServerRecord, servers: &[ServerRecord]) -> String {
    let route = |j: &Option<String>| j.as_ref().map_or_else(|| "direct".to_string(), |j| servers.iter().find(|s| &s.id == j).map_or(j.clone(), |s| s.name.clone()));
    let pairs = [
        ("name", old.name.clone(), new.name.clone()),
        ("env", old.env.clone(), new.env.clone()),
        ("role", old.role.clone(), new.role.clone()),
        ("group", old.group_name.clone(), new.group_name.clone()),
        ("tags", old.tags.join(","), new.tags.join(",")),
        ("login", old.login_user.clone(), new.login_user.clone()),
        ("key", old.key_id.clone().unwrap_or_default(), new.key_id.clone().unwrap_or_default()),
        ("route", route(&old.jump_host_id), route(&new.jump_host_id)),
        ("agent", old.agent_installed.to_string(), new.agent_installed.to_string()),
    ];
    pairs.iter().filter(|(_, a, b)| a != b).map(|(k, a, b)| format!("{k}: {a} → {b}")).collect::<Vec<_>>().join("; ")
}

impl CrowApp {
    pub fn open_edit_server(&mut self, server_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(s) = self.fleet.servers.iter().find(|s| s.id == server_id).cloned() else { return };
        let mut input = |value: String, placeholder: String, cx: &mut Context<Self>| cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).default_value(value));
        let name = input(s.name.clone(), t!("app-edit-server-name-placeholder").to_string(), cx);
        let role = input(s.role.clone(), t!("app-edit-server-role-placeholder").to_string(), cx);
        let tags = input(user_tags(&s.tags), t!("app-edit-server-tags-placeholder").to_string(), cx);
        let login = input(s.login_user.clone(), "root".to_string(), cx);
        self.edit_server = Some(EditServer { server_id: s.id, name, role, tags, login, env: s.env, group: s.group_name, key_id: s.key_id, jump: s.jump_host_id, agent_enrolled: s.agent_installed, testing: false, error: None });
        cx.notify();
    }

    pub fn close_edit_server(&mut self, cx: &mut Context<Self>) {
        if self.edit_server.as_ref().is_some_and(|e| e.testing) {
            return;
        }
        self.edit_server = None;
        cx.notify();
    }

    /// Sets a picked value (environment, group, key, route).
    pub fn edit_server_pick(&mut self, f: impl FnOnce(&mut EditServer), cx: &mut Context<Self>) {
        if let Some(e) = self.edit_server.as_mut() {
            f(e);
            e.error = None;
        }
        cx.notify();
    }

    /// SAVE: checks the edit; when it changes how Crow reaches the server,
    /// logs in that way first and saves only if that works.
    pub fn save_edit_server(&mut self, cx: &mut Context<Self>) {
        let Some(e) = self.edit_server.as_ref().filter(|e| !e.testing) else { return };
        let Some(old) = self.fleet.servers.iter().find(|s| s.id == e.server_id).cloned() else { return };
        let mut new = old.clone();
        new.name = e.name.read(cx).value().trim().to_string();
        new.role = e.role.read(cx).value().trim().to_string();
        new.tags = merge_tags(&old.tags, &e.tags.read(cx).value());
        new.env = e.env.clone();
        new.group_name = e.group.clone();
        new.agent_installed = e.agent_enrolled;
        if transport_kind(&old) == TransportKind::Ssh {
            new.login_user = e.login.read(cx).value().trim().to_string();
            new.key_id = e.key_id.clone();
            new.jump_host_id = e.jump.clone();
        }
        if let Err(why) = check(&new, &self.fleet.servers) {
            if let Some(e) = self.edit_server.as_mut() {
                e.error = Some(why);
            }
            cx.notify();
            return;
        }
        if new == old {
            self.edit_server = None;
            cx.notify();
            return;
        }
        if !reach_changed(&old, &new) {
            self.commit_edit_server(old, new, cx);
            return;
        }
        // Try the new way in on its own connection before anything is saved.
        if let Some(e) = self.edit_server.as_mut() {
            e.testing = true;
            e.error = None;
        }
        cx.notify();
        let mut directory = self.fleet.servers.clone();
        if let Some(s) = directory.iter_mut().find(|s| s.id == new.id) {
            *s = new.clone();
        }
        crate::host::update_directory(&directory, &self.keys.enrolled);
        cx.spawn(async move |entity, cx| {
            let target = new.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    crate::host::SshHost::for_server(&target).close_connection();
                    host_for(&target).exec(&["true"], DEFAULT_TIMEOUT).map(|_| ())
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                if let Some(e) = this.edit_server.as_mut() {
                    e.testing = false;
                }
                match result {
                    Ok(()) => this.commit_edit_server(old, new, cx),
                    Err(err) => {
                        // Back to the way in that works.
                        this.sync_ssh_directory();
                        crate::host::SshHost::for_server(&new).close_connection();
                        if let Some(e) = this.edit_server.as_mut() {
                            e.error = Some(t!("app-edit-server-couldnt-log-in", e = err));
                        }
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    fn commit_edit_server(&mut self, old: ServerRecord, new: ServerRecord, cx: &mut Context<Self>) {
        let what = describe(&old, &new, &self.fleet.servers);
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.upsert_server(&new);
            let now = chrono::Utc::now().to_rfc3339();
            let _ = db.insert_change_record(&ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: new.id.clone(),
                server_name: new.name.clone(),
                action_kind: "server.edit".into(),
                target: new.name.clone(),
                before_state: what,
                after_state: None,
                blast_radius: None,
                outcome: "success".into(),
                started_at: now.clone(),
                completed_at: Some(now),
            });
        }
        self.reload_servers();
        // Tabs carry the name they were opened with.
        for t in self.fleet.tabs.iter_mut().filter(|t| t.id == new.id) {
            t.name = new.name.clone();
        }
        // A new way in: the next command opens a fresh connection.
        if reach_changed(&old, &new) {
            crate::host::SshHost::for_server(&old).close_connection();
        }
        self.edit_server = None;
        self.fleet.notice = Some(t!("app-edit-server-saved", name = new.name));
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::{check, describe, merge_tags, reach_changed, user_tags};
    use crate::vault::ServerRecord;

    fn s(id: &str, name: &str) -> ServerRecord {
        ServerRecord { id: id.into(), name: name.into(), login_user: "root".into(), ..Default::default() }
    }

    #[test]
    fn crows_own_tags_stay_and_hide() {
        let old: Vec<String> = ["bastion", "web", "test-node", "podman"].map(String::from).to_vec();
        assert_eq!(user_tags(&old), "web");
        assert_eq!(merge_tags(&old, " api, web,, api ,"), ["bastion", "test-node", "podman", "api", "web"]);
        assert_eq!(merge_tags(&old, ""), ["bastion", "test-node", "podman"]);
    }

    #[test]
    fn edits_are_checked() {
        let mut bastion = s("b", "edge");
        bastion.tags = vec!["bastion".into()];
        let servers = vec![s("a", "web-1"), s("c", "db-1"), bastion];
        let mut e = s("a", "  ");
        assert!(check(&e, &servers).is_err(), "empty name");
        e.name = "DB-1".into();
        assert!(check(&e, &servers).is_err(), "taken, any case");
        e.name = "web-2".into();
        assert!(check(&e, &servers).is_ok());
        e.login_user = "ro ot".into();
        assert!(check(&e, &servers).is_err(), "login with a space");
        e.login_user = "deploy".into();
        e.jump_host_id = Some("c".into());
        assert!(check(&e, &servers).is_err(), "db-1 isn't a bastion");
        e.jump_host_id = Some("b".into());
        assert!(check(&e, &servers).is_ok());
        // A lab container keeps its name: it's how Crow finds it.
        let mut lab = s("l", "crow-dr-1");
        lab.tags = vec!["test-node".into(), "podman".into()];
        let servers = vec![lab.clone()];
        let mut renamed = lab.clone();
        renamed.name = "dr-primary".into();
        assert!(check(&renamed, &servers).is_err());
        renamed.name = lab.name.clone();
        renamed.env = "PROD".into();
        assert!(check(&renamed, &servers).is_ok(), "everything else can change");
    }

    #[test]
    fn what_changed_is_said() {
        let old = s("a", "web-1");
        let mut new = old.clone();
        new.role = "api".into();
        assert!(!reach_changed(&old, &new));
        new.jump_host_id = Some("b".into());
        assert!(reach_changed(&old, &new));
        let servers = vec![old.clone(), s("b", "edge")];
        assert_eq!(describe(&old, &new, &servers), "role:  → api; route: direct → edge");
    }
}
