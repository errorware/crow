use gpui_kit::Entity;
use gpui_kit::component::input::InputState;

use super::models::FileEntry;

/// Files screen state: the current directory listing and its prompts.
pub struct FilesState {
    pub current_path: String,
    pub entries: Vec<FileEntry>,
    pub error: Option<String>,
    pub pending_delete: Option<String>,
    pub new_folder_open: bool,
    pub new_folder_input: Option<Entity<InputState>>,
}

impl Default for FilesState {
    fn default() -> Self {
        Self {
            current_path: "/".to_string(),
            entries: Vec::new(),
            error: None,
            pending_delete: None,
            new_folder_open: false,
            new_folder_input: None,
        }
    }
}

impl FilesState {
    /// Moves to `path` and clears per-directory UI state; the caller reloads the listing.
    pub fn set_path(&mut self, path: &str) {
        self.current_path = path.to_string();
        self.pending_delete = None;
        self.error = None;
    }

    pub fn child_path(&self, name: &str) -> String {
        if self.current_path == "/" { format!("/{}", name) } else { format!("{}/{}", self.current_path, name) }
    }

    /// None when already at the root.
    pub fn parent_path(&self) -> Option<String> {
        if self.current_path == "/" {
            return None;
        }
        Some(
            std::path::Path::new(&self.current_path)
                .parent()
                .map(|p| {
                    let s = p.to_string_lossy().to_string();
                    if s.is_empty() { "/".to_string() } else { s }
                })
                .unwrap_or_else(|| "/".to_string()),
        )
    }

    pub fn toggle_delete_confirm(&mut self, name: &str) {
        self.pending_delete = if self.pending_delete.as_deref() == Some(name) { None } else { Some(name.to_string()) };
    }

    pub fn is_dir(&self, name: &str) -> bool {
        self.entries.iter().find(|e| e.name == name).map(|e| e.is_dir).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_navigation() {
        let mut st = FilesState::default();
        assert_eq!(st.parent_path(), None);
        assert_eq!(st.child_path("etc"), "/etc");
        st.set_path("/etc/ssh");
        assert_eq!(st.child_path("sshd_config"), "/etc/ssh/sshd_config");
        assert_eq!(st.parent_path().as_deref(), Some("/etc"));
        st.set_path("/etc");
        assert_eq!(st.parent_path().as_deref(), Some("/"));
    }
}
