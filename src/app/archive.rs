//! Archiving servers (ERR-32): a server leaves the active fleet but is never
//! deleted, and the data Crow stored *about* it is purged once the archive
//! window closes. The record — address, key, jump host, env, role, group,
//! tags, host-key fingerprint and OS facts — always stays.

use chrono::{Duration, Utc};
use gpui_kit::{Context, Task};

use super::poll::seed_metrics;
use super::{CrowApp, Screen};
use crate::host::{host_for, update_directory};
use crate::vault::{ChangeRecord, PurgeOutcome, ServerRecord, AUDIT_SERVER_ID};

/// The config field holding the purge window, in days ("never" = keep forever).
pub const PURGE_DAYS_FIELD: &str = "servers.archive_purge_days";

/// What the window is when the setting is absent, empty or unreadable.
pub const DEFAULT_PURGE_DAYS: i64 = 90;

/// How long an archived server keeps its stored data. `None` = never purge.
/// Anything unreadable keeps the default rather than silently switching
/// purging off.
pub fn purge_window_days(config: &crate::config::CrowConfigManager) -> Option<i64> {
    let raw = config
        .get_field(PURGE_DAYS_FIELD)
        .and_then(|f| f.value.as_str().map(|v| v.trim().to_string()))
        .unwrap_or_default();
    if raw == "never" {
        return None;
    }
    Some(raw.parse::<i64>().ok().filter(|days| *days > 0).unwrap_or(DEFAULT_PURGE_DAYS))
}

/// Days left before `srv`'s data is purged, or `None` when purging is off.
pub fn days_until_purge(srv: &ServerRecord, window_days: Option<i64>) -> Option<i64> {
    let window = window_days?;
    let archived = srv.archived_at.as_ref()?;
    let at = chrono::DateTime::parse_from_rfc3339(archived).ok()?.with_timezone(&Utc);
    Some((at + Duration::days(window) - Utc::now()).num_days())
}

/// How an archived server's countdown reads in the Archived tab.
pub fn purge_status_text(srv: &ServerRecord, window_days: Option<i64>) -> String {
    if srv.purged_at.is_some() {
        return "stored data purged".to_string();
    }
    match days_until_purge(srv, window_days) {
        Some(days) if days > 1 => format!("data purged in {days} days"),
        Some(1) => "data purged in 1 day".to_string(),
        Some(0) => "data purged today".to_string(),
        Some(_) => "purge due".to_string(),
        None => "purging is off · data kept".to_string(),
    }
}

/// The archive date as the Archived tab shows it.
pub fn archived_on_text(srv: &ServerRecord) -> String {
    srv.archived_at
        .as_deref()
        .and_then(|a| chrono::DateTime::parse_from_rfc3339(a).ok())
        .map(|a| a.format("%d %b %Y").to_string())
        .unwrap_or_else(|| "—".to_string())
}

/// The date an archive taken now would be purged, as text.
pub fn purge_due_text(window_days: Option<i64>) -> String {
    match window_days {
        Some(days) => (Utc::now() + Duration::days(days)).format("%d %b %Y").to_string(),
        None => "never — purging is switched off".to_string(),
    }
}

/// The host-key rotation policy in days (ERR-98); `None` when it's off.
pub fn host_key_max_age_days(config: &crate::config::CrowConfigManager) -> Option<i64> {
    let raw = config.get_field("servers.host_key_max_age_days").and_then(|f| f.value.as_str().map(|v| v.trim().to_string())).unwrap_or_default();
    if raw == "never" {
        return None;
    }
    Some(raw.parse::<i64>().ok().filter(|d| *d > 0).unwrap_or(365))
}

impl CrowApp {
    pub fn archive_purge_days(&self) -> Option<i64> {
        purge_window_days(&self.config)
    }

    /// The date an archive taken now would be purged, for the confirm dialog.
    pub fn purge_due_text(&self) -> String {
        purge_due_text(self.archive_purge_days())
    }

    /// Opens the archive confirmation for the active server — the Danger Zone's
    /// entry point, so a server can be archived from its own page.
    pub fn request_archive_active_server(&mut self, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.active_server() else { return };
        self.fleet.pending_archive = Some(srv.id);
        cx.notify();
    }

    /// Takes a server out of the fleet without deleting it. The record stays;
    /// its stored data is purged once the window closes.
    pub fn archive_server(&mut self, server_id: &str, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.servers.iter().find(|s| s.id == server_id || s.name == server_id).cloned() else { return };

        let archived = match self.vault.db().lock() {
            Ok(db) => match db.archive_server(&srv.id) {
                Ok(()) => db.list_archived_servers().unwrap_or_default(),
                Err(e) => {
                    self.fleet.notice = Some(format!("Could not archive {}: {e}", srv.name));
                    self.fleet.pending_archive = None;
                    cx.notify();
                    return;
                }
            },
            Err(_) => {
                self.fleet.notice = Some("Could not archive: the vault is unavailable".to_string());
                self.fleet.pending_archive = None;
                cx.notify();
                return;
            }
        };

        // Stop talking to it: release the multiplexed SSH connection, then drop
        // it from `fleet.servers` — which is what the poll loop snapshots, so
        // polling stops with it — along with its metrics, buffers and tab.
        // Host calls block, so closing the connection goes to the background.
        let closing = srv.clone();
        cx.spawn(async move |_entity, cx| {
            cx.background_executor().spawn(async move { host_for(&closing).close_connection() }).await;
        })
        .detach();
        self.fleet.servers.retain(|s| s.id != srv.id);
        self.fleet.archived = archived;
        self.fleet.pending_archive = None;
        self.forget_server_runtime(&srv, cx);
        update_directory(&self.fleet.servers, &self.keys.enrolled);
        self.fleet.notice = Some(format!("{} archived · its data is purged {}", srv.name, self.purge_due_text()));
        cx.notify();
    }

    /// Puts an archived server back in the fleet. Whatever the purge took is
    /// gone; the record and its facts come back either way.
    pub fn restore_archived_server(&mut self, server_id: &str, cx: &mut Context<Self>) {
        let Some(srv) = self.fleet.archived.iter().find(|s| s.id == server_id).cloned() else { return };

        let (servers, archived) = match self.vault.db().lock() {
            Ok(db) => {
                if let Err(e) = db.restore_server(&srv.id) {
                    self.fleet.notice = Some(format!("Could not restore {}: {e}", srv.name));
                    cx.notify();
                    return;
                }
                (db.list_servers().unwrap_or_default(), db.list_archived_servers().unwrap_or_default())
            }
            Err(_) => return,
        };

        self.fleet.servers = servers;
        self.fleet.archived = archived;
        // Seed one sample so the table and stat strip have something to show
        // before the next poll tick reaches the restored server.
        let (metrics, buffers) = seed_metrics(std::slice::from_ref(&srv));
        self.fleet.metrics_store.extend(metrics);
        self.fleet.buffered_stores.extend(buffers);

        self.fleet.notice = Some(format!("{} restored to the fleet", srv.name));
        self.fleet.page = crate::views::fleet::state::FleetPage::Active;
        // Opens a tab for it; then stay on the Fleet screen it was restored from.
        self.switch_tab(&srv.id, cx);
        self.screen = Screen::Fleet;
        update_directory(&self.fleet.servers, &self.keys.enrolled);
        cx.notify();
    }

    /// Drops everything Crow holds in memory for `srv`. Configs loaded from it
    /// are released so whichever server becomes active reloads them.
    fn forget_server_runtime(&mut self, srv: &ServerRecord, cx: &mut Context<Self>) {
        self.fleet.metrics_store.remove(&srv.id);
        self.fleet.metrics_store.remove(&srv.name);
        self.fleet.buffered_stores.remove(&srv.id);
        self.fleet.buffered_stores.remove(&srv.name);
        if self.configs.server_id.as_deref() == Some(srv.id.as_str()) {
            self.configs.server_id = None;
        }
        if let Some(pos) = self.fleet.tabs.iter().position(|t| t.id == srv.id || t.name == srv.name) {
            self.fleet.tabs.remove(pos);
        }
        if self.fleet.active_tab_id == srv.id || self.fleet.active_tab_id == srv.name {
            match self.fleet.tabs.last().map(|t| t.id.clone()) {
                Some(id) => self.switch_tab(&id, cx),
                None => {
                    self.fleet.active_tab_id.clear();
                    self.screen = Screen::Fleet;
                }
            }
        }
    }

    /// Deletes the stored data of archived servers whose window has closed.
    /// Runs at startup and once a day; each purge leaves an audit record.
    /// Returns how many servers were purged.
    pub fn purge_due_archives(&mut self, cx: &mut Context<Self>) -> usize {
        let Some(days) = self.archive_purge_days() else { return 0 };
        let cutoff = (Utc::now() - Duration::days(days)).to_rfc3339();
        let due = match self.vault.db().lock() {
            Ok(db) => db.servers_due_for_purge(&cutoff).unwrap_or_default(),
            Err(_) => return 0,
        };

        let mut purged = 0;
        for srv in due {
            let outcome = match self.vault.db().lock() {
                Ok(db) => db.purge_server_data(&srv.id, &srv.name),
                Err(_) => continue,
            };
            match outcome {
                Ok(outcome) => {
                    purged += 1;
                    self.record_purge_audit(&srv, &outcome);
                }
                Err(e) => self.fleet.notice = Some(format!("Could not purge {}'s data: {e}", srv.name)),
            }
        }

        if purged > 0 {
            if let Ok(db) = self.vault.db().lock() {
                self.fleet.archived = db.list_archived_servers().unwrap_or_default();
            }
            cx.notify();
        }
        purged
    }

    /// The audit trail of one purge. Recorded under Crow's own scope, not the
    /// purged server's, so the record survives the purge that wrote it.
    fn record_purge_audit(&self, srv: &ServerRecord, outcome: &PurgeOutcome) {
        let db = self.vault.db();
        let Ok(db) = db.lock() else { return };
        let now = Utc::now().to_rfc3339();
        let _ = db.insert_change_record(&ChangeRecord {
            id: format!("chg_{}", chrono::Local::now().timestamp_micros()),
            server_id: AUDIT_SERVER_ID.to_string(),
            server_name: "Crow".to_string(),
            action_kind: "purge".to_string(),
            target: srv.name.clone(),
            before_state: format!(
                "archived {} · removed {} change record(s) and {} key attachment(s)",
                srv.archived_at.clone().unwrap_or_default(),
                outcome.change_records,
                outcome.key_attachments
            ),
            after_state: None,
            blast_radius: None,
            outcome: "success".to_string(),
            started_at: now.clone(),
            completed_at: Some(now),
        });
    }

    /// The audit log across the fleet (ERR-75), newest first: the newest
    /// `limit` change records, plus enrollments of active and archived servers.
    pub fn audit_items(&self, limit: usize) -> Vec<crate::views::audit::model::AuditItem> {
        let (records, actors) = self.vault.db().lock().ok().map(|db| (db.list_all_change_records(limit).unwrap_or_default(), db.change_record_actors().unwrap_or_default())).unwrap_or_default();
        let servers: Vec<ServerRecord> = self.fleet.servers.iter().chain(&self.fleet.archived).cloned().collect();
        crate::views::audit::model::audit_items_by(&records, &servers, &actors)
    }

    /// FLEET ALERTS' lines and the last time Crow wasn't watching (ERR-85).
    pub fn fleet_alert_panel(&self) -> (Vec<crate::views::fleet::alert_lines::AlertLine>, Option<(i64, i64)>) {
        use crate::views::fleet::state::FleetHealth;
        let now = chrono::Utc::now().timestamp();
        let down_now: Vec<(String, String)> = self
            .fleet
            .servers
            .iter()
            .filter_map(|s| match self.fleet.health(s) {
                FleetHealth::Down { label, detail } => Some((s.id.clone(), if detail.is_empty() { label.to_lowercase() } else { format!("{} · {detail}", label.to_lowercase()) })),
                _ => None,
            })
            .collect();
        let (stored, sessions) = self
            .vault
            .db()
            .lock()
            .map(|db| (db.list_alerts(now - 86_400).unwrap_or_default(), db.list_watch_sessions(now - 7 * 86_400).unwrap_or_default()))
            .unwrap_or_default();
        (
            crate::views::fleet::alert_lines::alert_lines(&stored, &down_now, &self.fleet.servers, now),
            crate::views::fleet::alert_lines::watch_gap(&sessions, self.history.watch_started),
        )
    }

    /// Acknowledges one alert, or every open one.
    pub fn acknowledge_alerts(&mut self, id: Option<&str>, cx: &mut Context<Self>) {
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.acknowledge_alerts(id, chrono::Utc::now().timestamp());
        }
        cx.notify();
    }

    /// Recent purge audit records, newest first — the Archived tab's log.
    pub fn recent_purge_audit(&self) -> Vec<ChangeRecord> {
        self.vault
            .db()
            .lock()
            .ok()
            .and_then(|db| db.list_change_records(AUDIT_SERVER_ID, 20).ok())
            .unwrap_or_default()
    }

    /// Purges due archives a beat after startup, then once a day.
    pub(super) fn spawn_archive_purge(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |entity, cx| {
            loop {
                // Wait before the first sweep so the app is fully built; the
                // entity cannot be updated while `CrowApp::new` is still running.
                cx.background_executor().timer(std::time::Duration::from_secs(3)).await;
                if entity
                    .update(cx, |this, cx| {
                        this.purge_due_archives(cx);
                    })
                    .is_err()
                {
                    break;
                }
                cx.background_executor().timer(std::time::Duration::from_secs(24 * 60 * 60)).await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srv(archived_at: Option<&str>, purged_at: Option<&str>) -> ServerRecord {
        ServerRecord {
            id: "s1".into(),
            name: "s1".into(),
            archived_at: archived_at.map(str::to_string),
            purged_at: purged_at.map(str::to_string),
            ..ServerRecord::default()
        }
    }

    #[test]
    fn countdown_reads_in_days_and_handles_purged_and_off() {
        let recently = (Utc::now() - Duration::days(27)).to_rfc3339();
        let s = srv(Some(&recently), None);

        // 27 days into a 90-day window leaves about 63. Whole days truncate, so
        // allow for the sub-day drift between building the date and reading it.
        let days = days_until_purge(&s, Some(90)).expect("a window gives a countdown");
        assert!((62..=63).contains(&days), "expected about 63 days left, got {days}");
        assert_eq!(purge_status_text(&s, Some(90)), format!("data purged in {days} days"));

        let long_ago = (Utc::now() - Duration::days(200)).to_rfc3339();
        assert_eq!(purge_status_text(&srv(Some(&long_ago), None), Some(30)), "purge due");
        assert_eq!(purge_status_text(&s, None), "purging is off · data kept");
        assert_eq!(purge_status_text(&srv(Some(&recently), Some(&recently)), Some(90)), "stored data purged");
        assert_eq!(days_until_purge(&srv(None, None), Some(90)), None, "an active server has no countdown");
        assert_eq!(archived_on_text(&srv(None, None)), "—");
        assert!(!archived_on_text(&s).starts_with('—'));
    }

    #[test]
    fn window_falls_back_to_ninety_days_and_reads_never() {
        for (value, expected) in [("30", Some(30)), ("never", None), ("", Some(90)), ("junk", Some(90)), ("0", Some(90))] {
            // Built from text, not the user's config.toml: set straight in the
            // file, since the editor rejects an empty or junk value.
            let text = crate::config::default_config_toml().replace("archive_purge_days = \"90\"", &format!("archive_purge_days = {value:?}"));
            assert!(text.contains(&format!("archive_purge_days = {value:?}")));
            let config = crate::config::CrowConfigManager::from_text(std::path::PathBuf::new(), text);
            assert_eq!(purge_window_days(&config), expected, "value {value:?}");
        }
    }
}
