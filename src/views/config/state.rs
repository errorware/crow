use std::collections::HashMap;

use super::cron_editor::{generate_crontab_content, CronJobDef};
use super::rules_editor::{generate_hba_conf, HbaRuleDef};
use crate::config::{ConfigFileState, DiscoveredConfigFile};
use crate::os_detect::DistroFamily;

/// Config screen state: discovered files, their versioned edit states, and the
/// structured editors (cron jobs, pg_hba rules) that render into those states.
pub struct ConfigsState {
    pub files: Vec<DiscoveredConfigFile>,
    /// Keyed by file name (e.g. "pg_hba.conf", "crontab", "user.rules").
    pub states: HashMap<String, ConfigFileState>,
    pub selected_file: String,
    pub search_query: String,
    pub search_focused: bool,
    pub show_history: bool,
    pub cron_jobs: Vec<CronJobDef>,
    pub hba_rules: Vec<HbaRuleDef>,
    /// Server the files were read from; staged changes are written back there.
    pub server_id: Option<String>,
    /// That server's distro family, which decided the crawl's path set.
    pub family: DistroFamily,
    /// Last failed write, shown on the pending-diff rail until the next save.
    pub save_error: Option<String>,
}

impl ConfigsState {
    pub fn new(
        files: Vec<DiscoveredConfigFile>,
        states: HashMap<String, ConfigFileState>,
        selected_file: String,
        cron_jobs: Vec<CronJobDef>,
        hba_rules: Vec<HbaRuleDef>,
    ) -> Self {
        Self {
            files,
            states,
            selected_file,
            search_query: String::new(),
            search_focused: false,
            show_history: false,
            cron_jobs,
            hba_rules,
            server_id: None,
            family: DistroFamily::Unknown,
            save_error: None,
        }
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
            st.baseline_content = text.clone();
            st.current_content = text;
        }
    }

    pub fn sync_hba(&mut self) {
        let content = generate_hba_conf(&self.hba_rules);
        if let Some(state) = self.states.get_mut("pg_hba.conf") {
            state.update_content(content);
        }
    }

    pub fn sync_cron(&mut self) {
        let content = generate_crontab_content(&self.cron_jobs);
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
            command: "/usr/local/bin/backup-sync.sh".to_string(),
            comment: Some("Nightly backup routine".to_string()),
            enabled: true,
            is_expanded: true,
        });
        self.sync_cron();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::cron_editor::default_cron_jobs;
    use std::path::PathBuf;

    fn with_crontab() -> ConfigsState {
        let jobs = default_cron_jobs();
        let mut states = HashMap::new();
        let content = generate_crontab_content(&jobs);
        states.insert("crontab".to_string(), ConfigFileState::new(PathBuf::from("/etc/crontab"), "crontab".into(), content));
        ConfigsState::new(Vec::new(), states, "crontab".into(), jobs, Vec::new())
    }

    #[test]
    fn cron_edits_flow_into_crontab_state() {
        let mut st = with_crontab();
        assert!(!st.states["crontab"].is_modified());
        st.add_cron_job();
        assert!(st.states["crontab"].is_modified());
        assert!(st.states["crontab"].current_content.contains("/usr/local/bin/backup-sync.sh"));
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
}
