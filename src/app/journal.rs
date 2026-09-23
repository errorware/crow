use gpui_kit::*;

use super::CrowApp;
use crate::journal::{JournalBootScope, JournalPriority, JournalStorageMode, JournalTimeRange};
use crate::journal::reader::read_journal_for_server;
use crate::journal::retention::generate_journald_conf;

// ==========================================
// Systemd Journal Log Explorer & Retention
// ==========================================

impl CrowApp {
    pub fn toggle_journal_expanded(&mut self, id: &str, cx: &mut Context<Self>) {
        self.journal.toggle_expanded(id);
        cx.notify();
    }

    /// Runs the current query against the active server right now, regardless of
    /// live-tail state — this is what "search" actually means; it reaches into real
    /// journal history instead of only re-filtering whatever happened to be cached.
    pub fn run_journal_query(&mut self, cx: &mut Context<Self>) {
        if let Some(active_srv) = self.fleet.active_server() {
            let query = self.journal.build_query();
            cx.spawn(async move |entity, cx| {
                let entries = cx.background_executor().spawn(async move {
                    read_journal_for_server(&active_srv, &query)
                }).await;
                let _ = entity.update(cx, |this, cx| {
                    this.journal.entries = entries;
                    cx.notify();
                });
            }).detach();
        }
    }

    pub fn set_journal_pid_filter(&mut self, pid: Option<u32>, cx: &mut Context<Self>) {
        self.journal.pid_filter = pid;
        self.journal.pid_kill_confirm = false;
        self.run_journal_query(cx);
    }

    pub fn toggle_journal_pid_kill_confirm(&mut self, cx: &mut Context<Self>) {
        self.journal.pid_kill_confirm = !self.journal.pid_kill_confirm;
        cx.notify();
    }

    pub fn execute_journal_pid_kill(&mut self, cx: &mut Context<Self>) {
        if let Some(pid) = self.journal.pid_filter {
            self.execute_process_kill(pid, cx);
        }
        self.journal.pid_kill_confirm = false;
        cx.notify();
    }

    pub fn set_journal_severity(&mut self, prio: Option<JournalPriority>, cx: &mut Context<Self>) {
        self.journal.severity_filter = prio;
        self.run_journal_query(cx);
    }

    pub fn set_journal_unit(&mut self, unit: Option<String>, cx: &mut Context<Self>) {
        self.journal.unit_filter = unit;
        self.run_journal_query(cx);
    }

    /// Jumps the Logs screen to a specific unit's stream — the "VIEW LOGS" handoff
    /// from the Service Manager panel.
    pub fn jump_to_service_logs(&mut self, unit: &str, cx: &mut Context<Self>) {
        self.journal.focus_unit(unit);
        self.set_view("logs", cx);
        self.run_journal_query(cx);
    }

    pub fn set_journal_time_range(&mut self, range: JournalTimeRange, cx: &mut Context<Self>) {
        self.journal.time_range = range;
        self.journal.live_tail = range == JournalTimeRange::Live;
        self.run_journal_query(cx);
    }

    pub fn set_journal_boot(&mut self, boot: JournalBootScope, cx: &mut Context<Self>) {
        self.journal.boot = boot;
        if boot != JournalBootScope::Current {
            self.journal.live_tail = false;
        }
        self.run_journal_query(cx);
    }

    pub fn load_more_journal(&mut self, cx: &mut Context<Self>) {
        self.journal.limit += 200;
        self.run_journal_query(cx);
    }

    pub fn toggle_journal_dedupe(&mut self, cx: &mut Context<Self>) {
        self.journal.dedupe = !self.journal.dedupe;
        cx.notify();
    }

    pub fn toggle_journal_dupe_group_collapsed(&mut self, key: &str, cx: &mut Context<Self>) {
        self.journal.toggle_dupe_group_collapsed(key);
        cx.notify();
    }

    pub fn push_journal_action_marker(&mut self, text: String) {
        self.journal.push_marker(text);
    }

    pub fn toggle_journal_live_tail(&mut self, cx: &mut Context<Self>) {
        self.journal.live_tail = !self.journal.live_tail;
        if self.journal.live_tail {
            self.journal.time_range = JournalTimeRange::Live;
            self.journal.boot = JournalBootScope::Current;
            self.run_journal_query(cx);
        } else {
            cx.notify();
        }
    }

    pub fn clear_journal(&mut self, cx: &mut Context<Self>) {
        self.journal.entries.clear();
        cx.notify();
    }

    pub fn toggle_journal_retention_modal(&mut self, cx: &mut Context<Self>) {
        self.journal.show_retention_modal = !self.journal.show_retention_modal;
        cx.notify();
    }

    pub fn set_journal_quota(&mut self, quota_mb: u64, cx: &mut Context<Self>) {
        self.journal.retention.system_max_use_mb = quota_mb;
        self.journal.telemetry.estimated_retained_days = quota_mb as f32 / self.journal.telemetry.daily_burn_rate_mb.max(1.0);
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn sync_journald_to_config_state(&mut self) {
        let content = generate_journald_conf(&self.journal.retention);
        if let Some(state) = self.configs.states.get_mut("journald.conf") {
            state.update_content(content);
        }
    }

    pub fn set_journal_retention_days(&mut self, days: u32, cx: &mut Context<Self>) {
        self.journal.retention.max_retention_days = days;
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn set_journal_keep_free(&mut self, mb: u64, cx: &mut Context<Self>) {
        self.journal.retention.system_keep_free_mb = mb;
        self.sync_journald_to_config_state();
        cx.notify();
    }

    pub fn set_journal_storage_mode(&mut self, mode: JournalStorageMode, cx: &mut Context<Self>) {
        self.journal.retention.storage = mode;
        self.journal.telemetry.is_volatile_warning = mode != JournalStorageMode::Persistent;
        self.sync_journald_to_config_state();
        cx.notify();
    }
}
