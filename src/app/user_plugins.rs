//! User plugins in Settings → Plugins (ERR-177): found in the plugins
//! folder, reviewed, then accepted or refused. Nothing of a plugin runs in
//! this stage; accepting only pins its files.

use gpui_kit::*;

use super::CrowApp;
use crate::plugin_host::folder::{self, Installed, Pin, Trust};
use crate::vault::{ChangeRecord, AUDIT_SERVER_ID};

#[derive(Default)]
pub struct UserPluginsState {
    pub installed: Vec<Installed>,
    pub scanning: bool,
    /// The plugin whose review is open.
    pub reviewing: Option<String>,
    /// ACCEPT was pressed once; the second press accepts.
    pub confirming: Option<String>,
    /// (ok, text) after accept, refuse, revoke or a failed folder.
    pub message: Option<(bool, String)>,
}

impl UserPluginsState {
    pub fn get(&self, id: &str) -> Option<&Installed> {
        self.installed.iter().find(|p| p.id == id)
    }
}

impl CrowApp {
    fn user_plugin_pin(&self, id: &str) -> Pin {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return Pin::default() };
        Pin { accepted: db.flag(&folder::accepted_flag(id)).filter(|s| !s.is_empty()), refused: db.flag(&folder::refused_flag(id)).filter(|s| !s.is_empty()) }
    }

    /// Reads the plugins folder in the background, then pins from the vault.
    pub fn scan_user_plugins(&mut self, cx: &mut Context<Self>) {
        if self.plugins.user.scanning {
            return;
        }
        self.plugins.user.scanning = true;
        cx.notify();
        cx.spawn(async move |entity, cx| {
            // Checked without pins first (files only), so the vault isn't
            // touched off the main thread.
            let found = cx.background_executor().spawn(async move { folder::scan(&folder::plugins_dir(), |_| Pin::default()) }).await;
            let _ = entity.update(cx, |this, cx| {
                let mut found = found;
                for p in &mut found {
                    p.trust = folder::trust(&p.report, &p.hash, &this.user_plugin_pin(&p.id));
                }
                this.plugins.user.installed = found;
                this.plugins.user.scanning = false;
                if let Some(id) = &this.plugins.user.reviewing {
                    if this.plugins.user.get(id).is_none() {
                        this.plugins.user.reviewing = None;
                    }
                }
                this.plugins.user.confirming = None;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn open_plugins_folder(&mut self, cx: &mut Context<Self>) {
        let dir = folder::plugins_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.plugins.user.message = Some((false, t!("app-user-plugins-couldnt-create", path = dir.display(), e = e)));
            cx.notify();
            return;
        }
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&dir).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }

    pub fn toggle_user_plugin_review(&mut self, id: &str, cx: &mut Context<Self>) {
        self.plugins.user.confirming = None;
        self.plugins.user.reviewing = if self.plugins.user.reviewing.as_deref() == Some(id) { None } else { Some(id.to_string()) };
        cx.notify();
    }

    /// First press arms; the second accepts, if the files are still the
    /// ones on screen.
    pub fn accept_user_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        if self.plugins.user.confirming.as_deref() != Some(id) {
            self.plugins.user.confirming = Some(id.to_string());
            cx.notify();
            return;
        }
        self.plugins.user.confirming = None;
        let Some(shown) = self.plugins.user.get(id).cloned() else { return };
        // Read again: what's accepted must be exactly what was reviewed.
        let now = folder::load(&shown.dir, &Pin::default());
        if now.hash != shown.hash || !now.report.ok() {
            self.plugins.user.message = Some((false, t!("app-user-plugins-changed-while-reviewing", name = shown.name())));
            self.scan_user_plugins(cx);
            return;
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&folder::accepted_flag(id), &now.hash);
            let _ = db.set_flag(&folder::refused_flag(id), "");
        }
        self.audit_user_plugin("plugin.accept", &now);
        self.plugins.user.message = Some((true, t!("app-user-plugins-accepted", name = now.name())));
        self.plugins.user.reviewing = None;
        self.scan_user_plugins(cx);
    }

    pub fn refuse_user_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(p) = self.plugins.user.get(id).cloned() else { return };
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&folder::refused_flag(id), &p.hash);
            let _ = db.set_flag(&folder::accepted_flag(id), "");
        }
        self.audit_user_plugin("plugin.refuse", &p);
        self.plugins.user.message = Some((true, t!("app-user-plugins-refused", name = p.name())));
        self.plugins.user.reviewing = None;
        self.plugins.user.confirming = None;
        self.scan_user_plugins(cx);
    }

    /// Takes back an acceptance: the plugin is new again.
    pub fn revoke_user_plugin(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.allowed(crate::team::Permission::Vault, None, cx) {
            return;
        }
        let Some(p) = self.plugins.user.get(id).cloned() else { return };
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.set_flag(&folder::accepted_flag(id), "");
        }
        self.audit_user_plugin("plugin.revoke", &p);
        self.plugins.user.message = Some((true, t!("app-user-plugins-revoked", name = p.name())));
        self.scan_user_plugins(cx);
    }

    fn audit_user_plugin(&self, kind: &str, p: &Installed) {
        let version = p.report.manifest.as_ref().map(|m| m.plugin.version.clone()).unwrap_or_default();
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.insert_change_record(&ChangeRecord {
                id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
                server_id: AUDIT_SERVER_ID.to_string(),
                server_name: "Crow".to_string(),
                action_kind: kind.to_string(),
                target: format!("{} {version} ({})", p.name(), p.id),
                before_state: p.hash.clone(),
                after_state: None,
                blast_radius: None,
                outcome: "success".to_string(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            });
        }
    }
}

/// Short words for a plugin's state, for its row.
pub fn trust_label(t: Trust) -> &'static str {
    match t {
        Trust::New => t!("app-user-plugins-state-new"),
        Trust::Accepted => t!("app-user-plugins-state-accepted"),
        Trust::Changed => t!("app-user-plugins-state-changed"),
        Trust::Refused => t!("app-user-plugins-state-refused"),
        Trust::Invalid => t!("app-user-plugins-state-invalid"),
    }
}
