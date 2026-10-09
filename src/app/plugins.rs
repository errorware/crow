use std::collections::{HashMap, HashSet};

use gpui_kit::*;

use super::CrowApp;
use crate::plugins::{self, PluginStatus, CONTAINER_PLUGINS, INITIALIZED_FLAG};

/// Which plugins are on (ERR-138), and their last checked status.
#[derive(Default)]
pub struct PluginsState {
    pub enabled: HashSet<String>,
    pub status: HashMap<String, PluginStatus>,
    pub checking: HashSet<String>,
}

/// The first enabled set, for a vault from before plugins: whatever was
/// already in use stays on, so upgrading turns nothing off.
pub fn in_use(servers: &[crate::vault::ServerRecord], account_plugins: &[String]) -> HashSet<String> {
    let mut on = HashSet::new();
    if servers.iter().any(|s| crate::lab::multipass::vm_of(s).is_some()) {
        on.insert("multipass".to_string());
    }
    if servers.iter().any(|s| s.tags.iter().any(|t| t == "test-node")) {
        on.insert("podman".to_string());
        on.insert("distrobox".to_string());
        if servers.iter().any(|s| s.tags.iter().any(|t| t == "docker")) {
            on.insert("docker".to_string());
        }
    }
    for p in account_plugins {
        if plugins::get(p).is_some() {
            on.insert(p.clone());
        }
    }
    on
}

impl CrowApp {
    /// Reads which plugins are on; the first time, picks them from what's
    /// in use and saves that.
    pub fn load_plugins(&mut self) {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        if db.flag(INITIALIZED_FLAG).is_none() {
            let accounts: Vec<String> = self.providers.accounts.iter().map(|a| a.plugin.clone()).collect();
            let all: Vec<_> = self.fleet.servers.iter().chain(self.fleet.archived.iter()).cloned().collect();
            for id in in_use(&all, &accounts) {
                let _ = db.set_flag(&plugins::flag_key(&id), "1");
            }
            let _ = db.set_flag(INITIALIZED_FLAG, "1");
        }
        self.plugins.enabled = plugins::BUILTIN.iter().filter(|p| db.flag(&plugins::flag_key(p.id)).as_deref() == Some("1")).map(|p| p.id.to_string()).collect();
    }

    pub fn plugin_enabled(&self, id: &str) -> bool {
        self.plugins.enabled.contains(id)
    }

    /// Any container plugin: the lab (its window, its enroll button).
    pub fn lab_enabled(&self) -> bool {
        CONTAINER_PLUGINS.iter().any(|p| self.plugin_enabled(p))
    }

    /// What the lab is called given what's on, or `None` when nothing is.
    pub fn lab_label(&self) -> Option<&'static str> {
        match (self.plugin_enabled("multipass"), self.lab_enabled()) {
            (true, true) => Some("Local VMs & Containers"),
            (true, false) => Some("Multipass VMs"),
            (false, true) => Some("Local Lab Containers"),
            (false, false) => None,
        }
    }

    pub fn set_plugin_enabled(&mut self, id: &str, on: bool, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&plugins::flag_key(id), if on { "1" } else { "0" });
        }
        if id == "email" && on {
            // Start fresh: what opened while it was off isn't news.
            if let Ok(db) = self.vault.db().lock() {
                let _ = db.set_flag(crate::notify::email::STATE_FLAG, "{}");
            }
            self.email.dispatch = Default::default();
        }
        if on {
            self.plugins.enabled.insert(id.to_string());
            self.check_plugin(id, cx);
        } else {
            self.plugins.enabled.remove(id);
        }
        let name = plugins::get(id).map_or(id, |p| p.name);
        self.keys.toast = Some(format!("{name} {}", if on { "is on" } else { "is off" }));
        cx.notify();
    }

    /// Checks one plugin's status in the background.
    pub fn check_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if id == "email" {
            let status = match &self.email.saved {
                Some(s) => PluginStatus::Ready(format!("mails {}", s.to.join(", "))),
                None => PluginStatus::Problem { summary: "not set up yet".into(), steps: Vec::new() },
            };
            self.plugins.status.insert(id.to_string(), status);
            cx.notify();
            return;
        }
        if !self.plugins.checking.insert(id.to_string()) {
            return;
        }
        cx.notify();
        let id = id.to_string();
        cx.spawn(async move |entity, cx| {
            let job_id = id.clone();
            let status = cx.background_executor().spawn(async move { plugins::status(&job_id) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.plugins.checking.remove(&id);
                this.plugins.status.insert(id, status);
                cx.notify();
            });
        })
        .detach();
    }

    /// Checks every plugin (the Plugins page, when it opens or on RE-CHECK).
    pub fn check_all_plugins(&mut self, cx: &mut Context<Self>) {
        for p in plugins::BUILTIN {
            self.check_plugin(p.id, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::in_use;
    use crate::vault::ServerRecord;

    #[test]
    fn upgrading_keeps_on_what_was_in_use() {
        let vm = ServerRecord { provider_account: "multipass".into(), provider_instance: "lab".into(), ..Default::default() };
        let plain = ServerRecord::default();
        assert_eq!(in_use(&[plain.clone()], &[]).len(), 0, "nothing in use, nothing on");
        let on = in_use(&[vm, plain], &["linode".to_string(), "someone-else".to_string()]);
        assert!(on.contains("multipass") && on.contains("linode"));
        assert!(!on.contains("someone-else") && !on.contains("docker"));
        let container = ServerRecord { tags: vec!["test-node".into(), "podman".into()], ..Default::default() };
        let on = in_use(&[container], &[]);
        assert!(on.contains("podman") && on.contains("distrobox") && !on.contains("docker"));
    }
}
