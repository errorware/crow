//! New versions of Crow (ERR-88): a daily check, a notice next to the
//! version, and an opt-in verified install. See `crate::update`.

use gpui_kit::*;

use super::CrowApp;
use crate::update::{self, Release};

/// app_flags key: the release tag the user chose to skip.
const SKIPPED: &str = "updates.skipped";

#[derive(Default)]
pub struct UpdateState {
    /// A newer release worth offering.
    pub available: Option<Release>,
    pub installing: bool,
    /// Installed and waiting for a restart.
    pub installed: bool,
    /// What happened last (install result, or why it didn't).
    pub note: Option<String>,
}

impl CrowApp {
    pub fn update_check_enabled(&self) -> bool {
        self.config.saved_bool("general.update_check").unwrap_or(true)
    }

    pub fn update_prereleases(&self) -> bool {
        self.config.saved_bool("general.update_prereleases").unwrap_or(false)
    }

    pub fn update_auto_install(&self) -> bool {
        self.config.saved_bool("general.update_auto_install").unwrap_or(false)
    }

    /// Checks shortly after launch, then daily, while checking is on.
    pub(super) fn spawn_update_check(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |entity, cx| {
            cx.background_executor().timer(std::time::Duration::from_secs(10)).await;
            loop {
                let Ok(wanted) = entity.update(cx, |this, _| this.update_check_enabled().then(|| (this.update_prereleases(), this.vault.db().lock().ok().and_then(|db| db.flag(SKIPPED))))) else { break };
                if let Some((include_pre, skipped)) = wanted {
                    let releases = cx.background_executor().spawn(async move { update::fetch_releases() }).await;
                    let found = update::newest(update::CURRENT, &releases, include_pre, skipped.as_deref());
                    let install = entity
                        .update(cx, |this, cx| {
                            let install = found.is_some() && this.update_auto_install() && !this.update.installed && !this.update.installing;
                            this.update.available = found;
                            cx.notify();
                            install
                        })
                        .unwrap_or(false);
                    if install {
                        let _ = entity.update(cx, |this, cx| this.install_update(cx));
                    }
                }
                cx.background_executor().timer(std::time::Duration::from_secs(24 * 60 * 60)).await;
            }
        })
    }

    /// Downloads, verifies and installs the offered release in the background.
    pub fn install_update(&mut self, cx: &mut Context<Self>) {
        let Some(release) = self.update.available.clone() else { return };
        if self.update.installing {
            return;
        }
        self.update.installing = true;
        self.update.note = Some(format!("Downloading and verifying Crow {}…", release.version));
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { update::install(&release) }).await;
            let _ = entity.update(cx, |this, cx| {
                this.update.installing = false;
                this.update.installed = result.is_ok();
                this.update.note = Some(result.unwrap_or_else(|e| e));
                cx.notify();
            });
        })
        .detach();
    }

    pub fn skip_update(&mut self, cx: &mut Context<Self>) {
        if let Some(r) = self.update.available.take() {
            if let Ok(db) = self.vault.db().lock() {
                let _ = db.set_flag(SKIPPED, &r.tag);
            }
        }
        self.update.note = None;
        cx.notify();
    }

    pub fn open_release_notes(&mut self, _cx: &mut Context<Self>) {
        if let Some(r) = &self.update.available {
            if r.url.starts_with("https://github.com/") {
                super::terminal::open_url(&r.url);
            }
        }
    }

    /// Starts the newly installed Crow and quits this one. Only offered once
    /// an install succeeded; never done on its own.
    pub fn restart_after_update(&mut self, cx: &mut Context<Self>) {
        match update::restart_into_installed() {
            Ok(()) => cx.quit(),
            Err(e) => {
                self.update.note = Some(format!("Couldn't start the new version: {e}"));
                cx.notify();
            }
        }
    }
}
