//! A server's distro and kernel, read when Crow reaches it and they aren't
//! known yet (enrolled before they could be read, or imported from a
//! provider), and reconnecting by hand.

use gpui_kit::*;

use super::CrowApp;
use crate::host::{host_for, transport_kind, Host, SshHost, TransportKind, DEFAULT_TIMEOUT};

/// A fact Crow doesn't have: empty, or a placeholder onboarding stored.
pub fn unknown_fact(v: &str) -> bool {
    matches!(v.trim(), "" | "-" | "—" | "unknown")
}

impl CrowApp {
    /// Reads the active server's distro and kernel in the background if
    /// either isn't known, once per server per run, and saves them.
    pub fn backfill_server_facts(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        if !(unknown_fact(&srv.os_distro) || unknown_fact(&srv.os_kernel)) || !self.server_facts_tried.insert(srv.id.clone()) {
            return;
        }
        cx.spawn(async move |entity, cx| {
            let target = srv.clone();
            let (distro, kernel) = cx
                .background_executor()
                .spawn(async move {
                    let host = host_for(&target);
                    let distro = crate::os_detect::detect_os_release(host.as_ref());
                    let kernel = host.exec(&["uname", "-r"], DEFAULT_TIMEOUT).ok().map(|o| o.stdout.trim().to_string()).filter(|k| !k.is_empty());
                    (distro, kernel)
                })
                .await;
            if distro.is_none() && kernel.is_none() {
                return;
            }
            let _ = entity.update(cx, |this, cx| {
                let Some(rec) = this.fleet.servers.iter_mut().find(|s| s.id == srv.id) else { return };
                if let Some(d) = distro.filter(|_| unknown_fact(&rec.os_distro)) {
                    rec.os_distro = d;
                }
                if let Some(k) = kernel.filter(|_| unknown_fact(&rec.os_kernel)) {
                    rec.os_kernel = k;
                }
                let updated = rec.clone();
                if let Ok(db) = this.vault.db().lock() {
                    let _ = db.upsert_server(&updated);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Drops the active server's SSH connection and opens a fresh one, then
    /// re-reads its pages. Says how it went in the status bar.
    pub fn reconnect_active_server(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        if transport_kind(&srv) != TransportKind::Ssh {
            return;
        }
        self.fleet.notice = Some(format!("Reconnecting to {}…", srv.name));
        cx.notify();
        cx.spawn(async move |entity, cx| {
            let target = srv.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    SshHost::for_server(&target).close_connection();
                    host_for(&target).exec(&["true"], DEFAULT_TIMEOUT)
                })
                .await;
            let _ = entity.update(cx, |this, cx| {
                this.fleet.notice = Some(match result {
                    Ok(_) => format!("{} is connected again", srv.name),
                    Err(e) => format!("Couldn't reconnect to {}: {e}", srv.name),
                });
                // Read the pages again, unless there are edits to keep.
                if !this.configs.has_unsaved_changes() {
                    this.configs.server_id = None;
                }
                this.reload_configs_for_active_server(cx);
                this.refresh_firewall_for_active_server(cx);
                this.refresh_overview_tables(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::unknown_fact;

    #[test]
    fn placeholders_are_unknown() {
        for v in ["", "-", "—", " — ", "unknown"] {
            assert!(unknown_fact(v), "{v:?}");
        }
        assert!(!unknown_fact("Ubuntu 24.04 LTS"));
    }
}
