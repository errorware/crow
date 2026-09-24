use std::collections::HashSet;

use crate::journal::retention::{JournalRetentionConfig, JournalTelemetry};
use crate::journal::{JournalBootScope, JournalEntry, JournalPriority, JournalQuery, JournalTimeRange};

/// Most recent "Crow did X" markers kept for the log stream.
const MAX_ACTION_MARKERS: usize = 50;

/// Logs screen state: fetched journal entries, the filters that shape the
/// next query, retention settings and the in-app action markers.
pub struct JournalState {
    pub entries: Vec<JournalEntry>,
    pub search: String,
    pub severity_filter: Option<JournalPriority>,
    pub unit_filter: Option<String>,
    pub pid_filter: Option<u32>,
    pub pid_kill_confirm: bool,
    pub live_tail: bool,
    pub time_range: JournalTimeRange,
    pub boot: JournalBootScope,
    pub limit: usize,
    pub dedupe: bool,
    pub collapsed_dupe_groups: HashSet<String>,
    pub action_markers: Vec<JournalEntry>,
    pub retention: JournalRetentionConfig,
    pub telemetry: JournalTelemetry,
    pub show_retention_modal: bool,
    /// Units seen logging most (from the last unfiltered read), for the
    /// unit filter chips.
    pub known_units: Vec<(String, usize)>,
    /// Bumped on every search keystroke; a query runs once typing pauses.
    pub search_generation: u64,
}

impl JournalState {
    pub fn new(entries: Vec<JournalEntry>, retention: JournalRetentionConfig, telemetry: JournalTelemetry) -> Self {
        Self {
            entries,
            search: String::new(),
            severity_filter: None,
            unit_filter: None,
            pid_filter: None,
            pid_kill_confirm: false,
            live_tail: true,
            time_range: JournalTimeRange::Live,
            boot: JournalBootScope::Current,
            limit: 200,
            dedupe: false,
            collapsed_dupe_groups: HashSet::new(),
            action_markers: Vec::new(),
            retention,
            telemetry,
            show_retention_modal: false,
            known_units: Vec::new(),
            search_generation: 0,
        }
    }

    /// The single source of truth for what a journal lookup should ask for — built
    /// fresh from current filter/UI state every time, so the live-tail poll and an
    /// explicit search never drift into two different notions of "the query."
    pub fn build_query(&self) -> JournalQuery {
        JournalQuery {
            limit: self.limit,
            unit: self.unit_filter.clone(),
            priority: self.severity_filter,
            pid: self.pid_filter,
            grep: if self.search.trim().is_empty() { None } else { Some(self.search.clone()) },
            time_range: self.time_range,
            boot: self.boot,
        }
    }

    /// Remembers which units log most, from a read with no unit filter.
    pub fn learn_units(&mut self, entries: &[JournalEntry]) {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        // Only real systemd units (they carry a type suffix: .service, .scope, ...);
        // a bare syslog identifier like "sudo" can't be passed to `journalctl -u`.
        for e in entries.iter().filter(|e| e.syslog_identifier != "kernel" && e.unit.contains('.')) {
            *counts.entry(e.unit.as_str()).or_default() += 1;
        }
        let mut units: Vec<(String, usize)> = counts.into_iter().map(|(u, n)| (u.to_string(), n)).collect();
        units.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        units.truncate(8);
        if !units.is_empty() {
            self.known_units = units;
        }
    }

    pub fn toggle_expanded(&mut self, id: &str) {
        let mut now_expanded = false;
        for entry in &mut self.entries {
            if entry.id == id {
                entry.is_expanded = !entry.is_expanded;
                now_expanded = entry.is_expanded;
            }
        }
        // Reading a log is incompatible with the tail silently replacing entries
        // out from under the reader every poll tick — pause the stream the moment
        // something is opened. Resuming is a deliberate action (the live-tail
        // toggle), not automatic, so the reader keeps control of when it moves again.
        if now_expanded {
            self.live_tail = false;
        }
    }

    /// Resets filters to a live tail of one unit — the "VIEW LOGS" handoff
    /// from the Service Manager panel.
    pub fn focus_unit(&mut self, unit: &str) {
        self.unit_filter = Some(unit.to_string());
        self.severity_filter = None;
        self.pid_filter = None;
        self.search.clear();
        self.time_range = JournalTimeRange::Live;
        self.boot = JournalBootScope::Current;
        self.live_tail = true;
    }

    pub fn toggle_dupe_group_collapsed(&mut self, key: &str) {
        if !self.collapsed_dupe_groups.remove(key) {
            self.collapsed_dupe_groups.insert(key.to_string());
        }
    }

    /// Records a lightweight "Crow did X" marker so the log stream can show cause
    /// and effect around an action taken through the app, without needing the full
    /// crow-history/semantic-diff subsystem this is standing in for ahead of time.
    pub fn push_marker(&mut self, text: String) {
        let now_usec = chrono::Local::now().timestamp_micros() as u64;
        let (timestamp_formatted, time_relative) = JournalEntry::format_time(now_usec);
        self.action_markers.push(JournalEntry {
            id: format!("crow-action_{}", now_usec),
            cursor: None,
            timestamp_usec: now_usec,
            timestamp_formatted,
            time_relative,
            priority: JournalPriority::Notice,
            unit: "crow-action".to_string(),
            syslog_identifier: "crow".to_string(),
            pid: None,
            message: text,
            fields: Vec::new(),
            is_expanded: false,
        });
        if self.action_markers.len() > MAX_ACTION_MARKERS {
            let excess = self.action_markers.len() - MAX_ACTION_MARKERS;
            self.action_markers.drain(0..excess);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> JournalState {
        JournalState::new(Vec::new(), JournalRetentionConfig::default(), JournalTelemetry::default())
    }

    #[test]
    fn query_reflects_filters_and_ignores_blank_search() {
        let mut st = state();
        st.search = "   ".into();
        st.unit_filter = Some("nginx.service".into());
        st.pid_filter = Some(42);
        let q = st.build_query();
        assert_eq!(q.grep, None);
        assert_eq!(q.unit.as_deref(), Some("nginx.service"));
        assert_eq!(q.pid, Some(42));
        assert_eq!(q.limit, 200);
    }

    #[test]
    fn focus_unit_resets_to_live_tail() {
        let mut st = state();
        st.live_tail = false;
        st.pid_filter = Some(7);
        st.search = "timeout".into();
        st.focus_unit("sshd.service");
        assert!(st.live_tail);
        assert_eq!(st.pid_filter, None);
        assert!(st.search.is_empty());
        assert_eq!(st.unit_filter.as_deref(), Some("sshd.service"));
    }

    #[test]
    fn learns_the_busiest_real_units() {
        let entry = |unit: &str, ident: &str| {
            let mut e = JournalEntry::from_marker_for_test(unit);
            e.syslog_identifier = ident.to_string();
            e
        };
        let entries = vec![
            entry("nginx.service", "nginx"),
            entry("nginx.service", "nginx"),
            entry("sshd.service", "sshd"),
            entry("sudo", "sudo"),
            entry("kernel", "kernel"),
        ];
        let mut st = state();
        st.learn_units(&entries);
        let units: Vec<&str> = st.known_units.iter().map(|(u, _)| u.as_str()).collect();
        assert_eq!(units, ["nginx.service", "sshd.service"], "busiest first; identifiers and kernel left out");
    }

    #[test]
    fn markers_are_capped() {
        let mut st = state();
        for i in 0..(MAX_ACTION_MARKERS + 10) {
            st.push_marker(format!("action {i}"));
        }
        assert_eq!(st.action_markers.len(), MAX_ACTION_MARKERS);
        assert_eq!(st.action_markers.last().unwrap().message, format!("action {}", MAX_ACTION_MARKERS + 9));
    }
}
