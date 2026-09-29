//! The History view's data (ERR-86), built from the vault and kept until
//! the server, the range, or the history changes, so hovering doesn't
//! re-query.

use gpui_kit::*;

use super::CrowApp;
use crate::metrics::chart::{self, Range, Series, Span};

pub struct HistoryCache {
    pub server_id: String,
    pub range: Range,
    /// The history tick it was built at.
    pub tick: u64,
    pub from: i64,
    pub to: i64,
    pub cpu: Series,
    pub mem: Series,
    pub disk: Series,
    pub disk_label: String,
    pub load: Series,
    pub gaps: Vec<Span>,
    pub alerts: Vec<(f32, f32, String, String)>,
    /// (level, "detail · 20:50 → now") for the alerts in range.
    pub alert_labels: Vec<(String, String)>,
}

impl CrowApp {
    /// The active server's history for the chosen range, rebuilt when stale.
    pub fn history_cache(&mut self) -> Option<&HistoryCache> {
        let srv = self.fleet.active_server()?;
        let fresh = self.history_view.as_ref().is_some_and(|c| c.server_id == srv.id && c.range == self.history_range && c.tick == self.history.last_tick);
        if !fresh {
            let now = chrono::Utc::now().timestamp();
            let (range, bucket) = (self.history_range, self.history_range.bucket_secs());
            let (from, to) = (now - range.secs(), now);
            let db = self.vault.db();
            let db = db.lock().ok()?;
            let rows = db.list_history(&srv.id, from).unwrap_or_default();
            let sessions = db.list_watch_sessions(from).unwrap_or_default();
            let alerts = db.list_alerts(from).unwrap_or_default();
            drop(db);
            let fmt = |t: i64| chrono::DateTime::from_timestamp(t, 0).map(|d| d.with_timezone(&chrono::Local).format("%b %d %H:%M").to_string()).unwrap_or_default();
            let mount = self.fleet.metrics_store.get(&srv.id).map(|m| m.disk_mount.clone()).unwrap_or_else(|| "/".into());
            self.history_view = Some(HistoryCache {
                server_id: srv.id.clone(),
                range,
                tick: self.history.last_tick,
                from,
                to,
                cpu: chart::series(&rows, |r| r.cpu, from, to, bucket),
                mem: chart::series(&rows, |r| r.mem, from, to, bucket),
                disk: chart::series(&rows, |r| r.disk, from, to, bucket),
                disk_label: format!("DISK {mount}"),
                load: chart::series(&rows, |r| r.load1, from, to, bucket),
                gaps: chart::gaps(&rows, &sessions, from, to, bucket),
                alerts: chart::alert_spans(&alerts, &srv.id, from, to, now),
                alert_labels: alerts
                    .iter()
                    .filter(|a| a.server_id == srv.id)
                    .map(|a| (a.level.clone(), format!("{} · {} → {}", a.detail, fmt(a.opened_at), a.resolved_at.map(fmt).unwrap_or_else(|| "now".into()))))
                    .collect(),
            });
        }
        self.history_view.as_ref()
    }

    pub fn set_history_range(&mut self, range: Range, cx: &mut Context<Self>) {
        self.history_range = range;
        self.history_hover = None;
        cx.notify();
    }

    pub fn set_history_hover(&mut self, x: Option<f32>, cx: &mut Context<Self>) {
        if self.history_hover != x {
            self.history_hover = x;
            cx.notify();
        }
    }
}
