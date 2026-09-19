use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::Local;
use super::{ConfigDiffLine, DiffKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigRevision {
    pub version: usize,
    pub timestamp: String,
    pub author: String,
    pub message: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct ConfigFileState {
    pub path: PathBuf,
    pub filename: String,
    pub baseline_content: String,
    pub current_content: String,
    pub revisions: Vec<ConfigRevision>,
    pub active_revision: usize,
}

impl ConfigFileState {
    pub fn new(path: PathBuf, filename: String, content: String) -> Self {
        let initial_rev = ConfigRevision {
            version: 1,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            author: "system".to_string(),
            message: "Initial baseline read from host disk".to_string(),
            content: content.clone(),
        };

        Self {
            path,
            filename,
            baseline_content: content.clone(),
            current_content: content,
            revisions: vec![initial_rev],
            active_revision: 1,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.current_content != self.baseline_content
    }

    pub fn update_content(&mut self, new_content: String) {
        self.current_content = new_content;
    }

    pub fn revert(&mut self) {
        self.current_content = self.baseline_content.clone();
    }

    pub fn stage_revision(&mut self, author: String, message: String) {
        let next_v = self.revisions.len() + 1;
        let rev = ConfigRevision {
            version: next_v,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            author,
            message,
            content: self.current_content.clone(),
        };
        self.revisions.push(rev);
        self.active_revision = next_v;
        self.baseline_content = self.current_content.clone();
    }

    pub fn rollback_to_revision(&mut self, version: usize) -> bool {
        if let Some(rev) = self.revisions.iter().find(|r| r.version == version) {
            self.current_content = rev.content.clone();
            self.active_revision = version;
            true
        } else {
            false
        }
    }

    pub fn diff(&self) -> Vec<ConfigDiffLine> {
        compute_unified_diff(&self.baseline_content, &self.current_content)
    }

    pub fn diff_stats(&self) -> (usize, usize) {
        let diff = self.diff();
        let add = diff.iter().filter(|l| l.kind == DiffKind::Addition).count();
        let del = diff.iter().filter(|l| l.kind == DiffKind::Deletion).count();
        (add, del)
    }

    pub fn last_author(&self) -> &str {
        self.revisions.last().map(|r| r.author.as_str()).unwrap_or("Nelson <nelson@errorware.net>")
    }

    pub fn latest_revision(&self) -> Option<&ConfigRevision> {
        self.revisions.last()
    }

    pub fn save_to_disk(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let write_res = std::fs::write(&self.path, &self.current_content);
        if write_res.is_err() {
            let _ = std::fs::create_dir_all("/tmp/crow-config");
            let fallback_path = format!("/tmp/crow-config/{}", self.filename);
            std::fs::write(&fallback_path, &self.current_content)
                .map_err(|e| format!("Failed to write to fallback {}: {}", fallback_path, e))?;
        }
        Ok(())
    }
}

pub fn compute_unified_diff(baseline: &str, current: &str) -> Vec<ConfigDiffLine> {
    if baseline == current {
        return Vec::new();
    }

    let orig_lines: Vec<&str> = baseline.lines().collect();
    let curr_lines: Vec<&str> = current.lines().collect();

    let mut diff_lines = Vec::new();
    let max_len = orig_lines.len().max(curr_lines.len());
    let mut in_hunk = false;

    for i in 0..max_len {
        let o = orig_lines.get(i);
        let c = curr_lines.get(i);

        match (o, c) {
            (Some(orig), Some(curr)) => {
                if orig != curr {
                    if !in_hunk {
                        diff_lines.push(ConfigDiffLine {
                            kind: DiffKind::Hunk,
                            text: format!("@@ line {} @@", i + 1),
                        });
                        in_hunk = true;
                    }
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Deletion,
                        text: format!("- {}", orig),
                    });
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Addition,
                        text: format!("+ {}", curr),
                    });
                } else {
                    in_hunk = false;
                }
            }
            (Some(orig), None) => {
                if !in_hunk {
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Hunk,
                        text: format!("@@ line {} @@", i + 1),
                    });
                    in_hunk = true;
                }
                diff_lines.push(ConfigDiffLine {
                    kind: DiffKind::Deletion,
                    text: format!("- {}", orig),
                });
            }
            (None, Some(curr)) => {
                if !in_hunk {
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Hunk,
                        text: format!("@@ line {} @@", i + 1),
                    });
                    in_hunk = true;
                }
                diff_lines.push(ConfigDiffLine {
                    kind: DiffKind::Addition,
                    text: format!("+ {}", curr),
                });
            }
            (None, None) => break,
        }
    }

    diff_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_diff_add_and_del() {
        let base = "worker_processes 1;\nevents {\n  worker_connections 1024;\n}\n";
        let curr = "worker_processes 4;\nevents {\n  worker_connections 2048;\n}\n# tuning\n";
        let diff = compute_unified_diff(base, curr);
        assert!(!diff.is_empty());
        assert!(diff.iter().any(|d| d.kind == DiffKind::Deletion && d.text.contains("worker_processes 1;")));
        assert!(diff.iter().any(|d| d.kind == DiffKind::Addition && d.text.contains("worker_processes 4;")));
    }

    #[test]
    fn test_config_file_state_lifecycle() {
        let mut state = ConfigFileState::new(
            PathBuf::from("/etc/nginx/nginx.conf"),
            "nginx.conf".to_string(),
            "server { listen 80; }".to_string(),
        );

        assert!(!state.is_modified());
        assert_eq!(state.diff_stats(), (0, 0));

        state.update_content("server { listen 443 ssl; }".to_string());
        assert!(state.is_modified());
        let (add, del) = state.diff_stats();
        assert_eq!(add, 1);
        assert_eq!(del, 1);

        state.revert();
        assert!(!state.is_modified());
        assert_eq!(state.current_content, "server { listen 80; }");

        state.update_content("server { listen 8080; }".to_string());
        state.stage_revision("Nelson".to_string(), "Changed to port 8080".to_string());
        assert_eq!(state.revisions.len(), 2);
        assert_eq!(state.active_revision, 2);
        assert!(!state.is_modified());
    }
}
