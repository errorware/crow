use std::collections::{HashMap, HashSet};

use super::cron_editor::CronJobDef;
use crate::config::crontab::{parse_crontab, render_crontab};
use crate::config::{ConfigFileState, DiscoveredConfigFile};
use crate::os_detect::DistroFamily;

/// Config screen state: discovered files, their versioned edit states, and the
/// cron job model the Cron screen renders into the crontab state.
pub struct ConfigsState {
    pub files: Vec<DiscoveredConfigFile>,
    /// Files sshd_config pulls in with Include, as read from the server (ERR-12).
    pub sshd_includes: Vec<crate::config::plugins::SshdInclude>,
    /// Keyed by file name (e.g. "pg_hba.conf", "crontab", "user.rules").
    pub states: HashMap<String, ConfigFileState>,
    pub selected_file: String,
    pub search_query: String,
    pub search_focused: bool,
    pub show_history: bool,
    pub cron_jobs: Vec<CronJobDef>,
    /// The crontab text `cron_jobs` were parsed from; edits render back into it.
    pub cron_source: String,
    /// Server the files were read from; staged changes are written back there.
    pub server_id: Option<String>,
    /// That server's distro family, which decided the crawl's path set.
    pub family: DistroFamily,
    /// Last failed write, shown on the pending-diff rail until the next save.
    pub save_error: Option<String>,
    /// History couldn't be read or recorded (ERR-72); the file itself is fine.
    pub history_error: Option<String>,
    /// Whether history content is sealed (a data key was there at the last
    /// sync); `None` before the first sync.
    pub history_sealed: Option<bool>,
    /// Revision being previewed on the rail: (file, version) (ERR-73).
    pub preview_revision: Option<(String, usize)>,
    /// Baselines that apply to this server's files, by path (ERR-74).
    pub baselines: HashMap<String, AppliedBaseline>,
    /// Scope a pinned baseline gets here: the server's group, or the fleet.
    pub baseline_scope: String,
    /// Structured-editor UI: files switched to the plain-text view, the enum
    /// field whose options are open (row id, field), whether the "add
    /// directive" list is open, and the last rejected edit.
    pub text_mode: HashSet<String>,
    pub open_enum: Option<(String, String)>,
    pub adding_row: bool,
    pub edit_error: Option<String>,
    /// What the server's kernel runs now, per sysctl key (read from
    /// /proc/sys), and whether a read is under way.
    pub sysctl_live: HashMap<String, crate::config::sysctl_live::LiveValue>,
    pub sysctl_live_loading: bool,
}

/// A baseline in force for one of this server's files.
#[derive(Clone, Debug)]
pub struct AppliedBaseline {
    pub baseline: crate::vault::ConfigBaseline,
    /// Server the pinned revision came from.
    pub from_server: String,
    /// The baseline's text, when its content was kept and the vault is open.
    pub content: Option<String>,
    pub sha256: String,
}

impl AppliedBaseline {
    /// This server's copy of the file (`on_host`) against the baseline.
    pub fn drift(&self, path: &str, on_host: &str) -> crate::config::drift::Drift {
        use crate::config::drift::{compare, format_for_path, Drift};
        if crate::config::history::sha256_hex(on_host) == self.sha256 {
            return Drift::Identical;
        }
        match &self.content {
            Some(base) => compare(format_for_path(path), base, on_host),
            None => Drift::Differs(vec!["differs from the baseline (only its hash was kept)".into()]),
        }
    }
}

impl ConfigsState {
    pub fn new(
        files: Vec<DiscoveredConfigFile>,
        states: HashMap<String, ConfigFileState>,
        selected_file: String,
    ) -> Self {
        Self {
            files,
            sshd_includes: Vec::new(),
            states,
            selected_file,
            search_query: String::new(),
            search_focused: false,
            show_history: false,
            cron_jobs: Vec::new(),
            cron_source: String::new(),
            server_id: None,
            family: DistroFamily::Unknown,
            save_error: None,
            history_error: None,
            history_sealed: None,
            preview_revision: None,
            baselines: HashMap::new(),
            baseline_scope: crate::vault::BASELINE_FLEET.to_string(),
            text_mode: HashSet::new(),
            open_enum: None,
            adding_row: false,
            edit_error: None,
            sysctl_live: HashMap::new(),
            sysctl_live_loading: false,
        }
    }

    /// The crow-config format `file` is edited with, if it has one.
    pub fn structured_format(&self, file: &str) -> Option<crate::config::plugins::StructuredFormat> {
        use crate::config::plugins::{editor_for, ConfigEditor};
        let kind = self.files.iter().find(|f| f.name == file).and_then(|f| f.schema_kind)?;
        match editor_for(Some(kind)) {
            ConfigEditor::Structured(format) => Some(format),
            _ => None,
        }
    }

    /// The revision previewed for the selected file, if it can be shown.
    pub fn previewed_revision(&self) -> Option<&crate::config::ConfigRevision> {
        let (file, version) = self.preview_revision.as_ref()?;
        if *file != self.selected_file {
            return None;
        }
        self.states.get(file)?.revisions.iter().find(|r| r.version == *version && r.content.is_some())
    }

    /// Previews `version` of `file`, or stops previewing it if it already is.
    pub fn toggle_preview(&mut self, file: &str, version: usize) {
        let key = (file.to_string(), version);
        self.preview_revision = if self.preview_revision.as_ref() == Some(&key) { None } else { Some(key) };
    }

    /// True when any file has edits that have not been staged.
    pub fn has_unsaved_changes(&self) -> bool {
        self.states.values().any(|st| st.is_modified())
    }

    /// Marks a file as not writable back to its host, with the reason shown on save.
    pub fn block_writes(&mut self, file: &str, reason: String) {
        if let Some(st) = self.states.get_mut(file) {
            st.write_blocked.get_or_insert(reason);
        }
    }

    /// Sets a file's baseline and current content to `text` (both unmodified).
    /// With `create_at`, a missing file state is created at that path first.
    pub fn seed_baseline(&mut self, file: &str, text: String, create_at: Option<&str>) {
        if !self.states.contains_key(file) {
            let Some(path) = create_at else { return };
            let mut st = ConfigFileState::new(path.into(), file.to_string(), text.clone());
            st.write_blocked = Some(format!("{file} was not found on this server"));
            self.states.insert(file.to_string(), st);
        }
        if let Some(st) = self.states.get_mut(file) {
            // Generated by Crow, not read from the host: kept out of history.
            st.read_from_host = false;
            st.baseline_content = text.clone();
            st.current_content = text;
        }
    }

    /// Re-reads the Cron screen's jobs from crontab text.
    pub fn reload_cron_from(&mut self, text: &str) {
        self.cron_jobs = parse_crontab(text);
        self.cron_source = text.to_string();
    }

    pub fn sync_cron(&mut self) {
        let content = render_crontab(&self.cron_source, &self.cron_jobs);
        if let Some(st) = self.states.get_mut("crontab") {
            st.update_content(content);
        }
    }

    pub fn toggle_cron_job_enabled(&mut self, job_id: &str) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            job.enabled = !job.enabled;
            self.sync_cron();
        }
    }

    pub fn toggle_cron_job_expanded(&mut self, job_id: &str) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            job.is_expanded = !job.is_expanded;
        }
    }

    pub fn apply_cron_preset(
        &mut self,
        job_id: &str,
        minute: &str,
        hour: &str,
        day_of_month: &str,
        month: &str,
        day_of_week: &str,
    ) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            job.minute = minute.to_string();
            job.hour = hour.to_string();
            job.day_of_month = day_of_month.to_string();
            job.month = month.to_string();
            job.day_of_week = day_of_week.to_string();
            self.sync_cron();
        }
    }

    pub fn update_cron_field(
        &mut self,
        job_id: &str,
        field: &str,
        value: &str,
    ) {
        if let Some(job) = self.cron_jobs.iter_mut().find(|j| j.id == job_id) {
            match field {
                "minute" => job.minute = value.to_string(),
                "hour" => job.hour = value.to_string(),
                "day_of_month" | "dom" => job.day_of_month = value.to_string(),
                "month" | "mon" => job.month = value.to_string(),
                "day_of_week" | "dow" => job.day_of_week = value.to_string(),
                "user" => job.user = value.to_string(),
                "command" => job.command = value.to_string(),
                _ => {}
            }
            self.sync_cron();
        }
    }

    pub fn move_cron_job_up(&mut self, job_id: &str) {
        if let Some(idx) = self.cron_jobs.iter().position(|j| j.id == job_id) {
            if idx > 0 {
                self.cron_jobs.swap(idx, idx - 1);
                self.sync_cron();
            }
        }
    }

    pub fn move_cron_job_down(&mut self, job_id: &str) {
        if let Some(idx) = self.cron_jobs.iter().position(|j| j.id == job_id) {
            if idx + 1 < self.cron_jobs.len() {
                self.cron_jobs.swap(idx, idx + 1);
                self.sync_cron();
            }
        }
    }

    pub fn delete_cron_job(&mut self, job_id: &str) {
        self.cron_jobs.retain(|j| j.id != job_id);
        self.sync_cron();
    }

    pub fn add_cron_job(&mut self) {
        let new_id = format!(
            "cron_{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );
        self.cron_jobs.push(CronJobDef {
            id: new_id,
            minute: "0".to_string(),
            hour: "2".to_string(),
            day_of_month: "*".to_string(),
            month: "*".to_string(),
            day_of_week: "*".to_string(),
            user: "root".to_string(),
            // Starts disabled with a placeholder: staging it can't schedule
            // anything until a real command is filled in and it's enabled.
            command: "/path/to/command".to_string(),
            comment: Some("New job".to_string()),
            enabled: false,
            is_expanded: true,
        });
        self.sync_cron();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const CRONTAB: &str = "SHELL=/bin/sh\n# m h dom mon dow user\tcommand\n17 * * * *\troot\tcd / && run-parts /etc/cron.hourly\n25 6 * * *\troot\trun-parts /etc/cron.daily\n";

    fn with_crontab() -> ConfigsState {
        let mut states = HashMap::new();
        states.insert("crontab".to_string(), ConfigFileState::new(PathBuf::from("/etc/crontab"), "crontab".into(), CRONTAB.into()));
        let mut st = ConfigsState::new(Vec::new(), states, "crontab".into());
        st.reload_cron_from(CRONTAB);
        st
    }

    #[test]
    fn cron_edits_flow_into_crontab_state() {
        let mut st = with_crontab();
        assert!(!st.states["crontab"].is_modified());
        st.add_cron_job();
        assert!(st.states["crontab"].is_modified());
        assert!(st.states["crontab"].current_content.ends_with("# New job\n#0 2 * * *\troot\t/path/to/command\n"));
        assert!(st.states["crontab"].current_content.starts_with(CRONTAB), "existing lines untouched");
    }

    #[test]
    fn created_seed_files_are_write_blocked() {
        let mut st = with_crontab();
        st.seed_baseline("user.rules", "# rules\n".into(), Some("/etc/ufw/user.rules"));
        assert!(st.states["user.rules"].write_blocked.is_some());
        st.block_writes("crontab", "sample model".into());
        st.block_writes("crontab", "second reason is ignored".into());
        assert_eq!(st.states["crontab"].write_blocked.as_deref(), Some("sample model"));
    }

    #[test]
    fn cron_reorder_respects_bounds() {
        let mut st = with_crontab();
        let first = st.cron_jobs[0].id.clone();
        st.move_cron_job_up(&first);
        assert_eq!(st.cron_jobs[0].id, first);
        st.move_cron_job_down(&first);
        assert_eq!(st.cron_jobs[1].id, first);
    }

    #[test]
    fn previews_only_restorable_revisions_of_the_selected_file() {
        let mut st = with_crontab();
        let file = st.states.get_mut("crontab").unwrap();
        file.update_content("# edited\n".into());
        file.stage_revision("me".into(), "edit".into());
        file.revisions[0].content = None; // hash-only
        st.toggle_preview("crontab", 2);
        assert_eq!(st.previewed_revision().map(|r| r.version), Some(2));
        st.toggle_preview("crontab", 2);
        assert!(st.previewed_revision().is_none(), "second click closes it");
        st.toggle_preview("crontab", 1);
        assert!(st.previewed_revision().is_none(), "no content, nothing to preview");
        st.toggle_preview("crontab", 2);
        st.selected_file = "hosts".into();
        assert!(st.previewed_revision().is_none(), "belongs to another file");
    }
}
