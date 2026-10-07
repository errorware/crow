use gpui_kit::*;

use super::CrowApp;
use crate::journal::{JournalBootScope, JournalPriority, JournalStorageMode, JournalTimeRange};
use crate::journal::reader::read_journal_for_server;
use crate::journal::retention::apply_journald_settings;

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
                    if this.journal.unit_filter.is_none() {
                        this.journal.learn_units(&entries);
                    }
                    this.journal.set_entries(entries);
                    cx.notify();
                });
            }).detach();
        }
    }

    /// Search typed into the box: query the server once typing pauses.
    pub fn schedule_journal_search(&mut self, cx: &mut Context<Self>) {
        self.journal.search_generation += 1;
        let generation = self.journal.search_generation;
        cx.spawn(async move |entity, cx| {
            cx.background_executor().timer(std::time::Duration::from_millis(350)).await;
            let _ = entity.update(cx, |this, cx| {
                if this.journal.search_generation == generation {
                    this.run_journal_query(cx);
                }
            });
        })
        .detach();
        cx.notify();
    }

    /// Runs one of the quick searches (JOURNAL_PRESETS) over the last 24h.
    pub fn apply_journal_preset(&mut self, idx: usize, cx: &mut Context<Self>) {
        let Some((_, unit, grep)) = crate::views::logs::JOURNAL_PRESETS.get(idx) else { return };
        self.journal.unit_filter = unit.map(str::to_string);
        self.journal.search = grep.to_string();
        self.journal.severity_filter = None;
        self.journal.pid_filter = None;
        self.journal.time_range = JournalTimeRange::Last24h;
        self.journal.live_tail = false;
        self.logs_search = None; // rebuilt showing the preset's pattern
        self.run_journal_query(cx);
    }

    pub fn toggle_journal_actions(&mut self, cx: &mut Context<Self>) {
        self.journal.show_actions = !self.journal.show_actions;
        cx.notify();
    }

    pub fn toggle_ai_panel(&mut self, cx: &mut Context<Self>) {
        self.journal.ai.open = !self.journal.ai.open;
        cx.notify();
    }

    /// The provider to ask: the default one if it has a key, else the first
    /// that does.
    fn ai_provider(&self) -> Option<crate::vault::ClankerProviderConfig> {
        self.ai_providers().map(|(p, _)| p)
    }

    /// The primary (or, with none chosen, the first with a key) and the
    /// backup, if one with a key is chosen.
    pub fn ai_providers(&self) -> Option<(crate::vault::ClankerProviderConfig, Option<crate::vault::ClankerProviderConfig>)> {
        let keyed = |p: &&crate::vault::ClankerProviderConfig| !p.api_key.trim().is_empty();
        let primary = self.clankers.providers.iter().filter(keyed).find(|p| p.is_default).or_else(|| self.clankers.providers.iter().find(keyed)).cloned()?;
        let backup = self.clankers.providers.iter().filter(keyed).find(|p| p.is_backup && p.id != primary.id).cloned();
        Some((primary, backup))
    }

    pub fn ai_provider_name(&self) -> Option<String> {
        self.ai_provider().map(|p| format!("{} · {}", p.display_name, p.model))
    }

    /// Sends the lines on screen (most recent first to be kept, capped) to
    /// the AI provider and shows its plain-English reading in the panel.
    pub fn explain_logs_with_ai(&mut self, cx: &mut Context<Self>) {
        let Some((provider, backup)) = self.ai_providers() else {
            self.journal.ai.answer = Some(Err("No AI provider has an API key. Add one in Settings → Clankers.".into()));
            cx.notify();
            return;
        };
        // Newest lines are the most relevant; keep as many as fit.
        let mut lines: Vec<String> = Vec::new();
        let mut size = 0;
        for e in self.journal.entries.iter().rev() {
            let line = format!("{} [{}] {}{}: {}", e.timestamp_formatted, e.priority.label(), e.unit, e.pid.map(|p| format!("[{p}]")).unwrap_or_default(), e.message);
            size += line.len() + 1;
            if size > crate::ai::MAX_LOG_CHARS {
                break;
            }
            lines.push(line);
        }
        lines.reverse();
        if lines.is_empty() {
            self.journal.ai.answer = Some(Err("There are no log lines on screen to explain.".into()));
            cx.notify();
            return;
        }
        let j = &self.journal;
        let context = format!(
            "These are the most recent {} lines matching: unit {}, priority {}, search {}, range {}.",
            lines.len(),
            j.unit_filter.as_deref().unwrap_or("any"),
            j.severity_filter.map(|p| format!("{} and worse", p.label())).unwrap_or_else(|| "any".into()),
            if j.search.is_empty() { "none".to_string() } else { format!("`{}`", j.search) },
            j.time_range.label(),
        );
        self.journal.ai = crate::views::logs::AiPanelState { open: true, loading: true, provider: format!("{} · {}", provider.display_name, provider.model), lines_sent: lines.len(), answer: None };
        cx.notify();
        let text = lines.join("\n");
        let db = self.vault.db();
        cx.spawn(async move |entity, cx| {
            let answer = cx.background_executor().spawn(async move { crate::ai::explain_logs_with_fallback(&provider, backup.as_ref(), &context, &text) }).await;
            if let Ok(a) = &answer {
                if let Ok(db) = db.lock() {
                    let _ = db.record_clanker_call(&a.provider_id);
                }
            }
            let _ = entity.update(cx, |this, cx| {
                this.journal.ai.loading = false;
                this.journal.ai.answer = Some(answer.map(|a| {
                    // Say when the backup answered, and why.
                    this.journal.ai.provider = a.provider_label.clone();
                    match a.primary_failed {
                        Some(why) => format!("Answered by the backup, {}: the primary failed ({why}).\n\n{}", a.provider_label, a.text),
                        None => a.text,
                    }
                }));
                this.refresh_clankers(cx);
            });
        })
        .detach();
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

    /// CLEAR, or SHOW ALL once cleared.
    pub fn clear_journal(&mut self, cx: &mut Context<Self>) {
        if self.journal.cleared_after.take().is_some() {
            self.journal.export_note = None;
            self.run_journal_query(cx);
        } else {
            self.journal.clear();
            self.journal.export_note = None;
            cx.notify();
        }
    }

    /// EXPORT: copies the lines on screen (after filters) to the clipboard.
    pub fn export_journal(&mut self, text: String, lines: usize, cx: &mut Context<Self>) {
        if lines == 0 {
            self.journal.export_note = Some("Nothing to export".into());
        } else {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.journal.export_note = Some(format!("Copied {lines} line{}", if lines == 1 { "" } else { "s" }));
        }
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
        // Only the settings that differ from the saved file are written, in place.
        if let Some(state) = self.configs.states.get_mut("journald.conf") {
            let content = apply_journald_settings(&state.baseline_content, &self.journal.retention);
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
