use gpui_kit::{Entity, HighlightStyle, SharedString};
use gpui_kit::component::input::InputState;

use super::models::FileEntry;

/// A file open in the viewer.
pub struct FilePreview {
    pub path: String,
    /// None while loading.
    pub result: Option<Result<PreviewText, String>>,
}

/// A file's text, laid out for the viewer once when it's read.
pub struct PreviewText {
    pub text: SharedString,
    pub runs: Vec<(std::ops::Range<usize>, HighlightStyle)>,
    /// Line numbers, one per line, to sit beside the text.
    pub gutter: SharedString,
    pub lines: usize,
    /// The file goes on past what's shown.
    pub truncated: bool,
}

impl PreviewText {
    pub fn new(raw: &str, truncated: bool, filename: &str) -> Self {
        // Tabs to spaces: the text element has no tab stops.
        let text = raw.replace('\t', "    ");
        let runs = crate::views::config::text_editor::highlight_runs(&text, filename);
        let lines = text.lines().count().max(1);
        let gutter = (1..=lines).map(|n| n.to_string()).collect::<Vec<_>>().join("\n");
        Self { text: text.into(), runs, gutter: gutter.into(), lines, truncated }
    }
}

/// The permission editor open under a row.
pub struct PermsEdit {
    pub name: String,
    pub original: u32,
    pub mode: u32,
    pub saving: bool,
}

/// Files screen state: the current directory listing and its prompts.
pub struct FilesState {
    pub current_path: String,
    pub entries: Vec<FileEntry>,
    pub error: Option<String>,
    pub pending_delete: Option<String>,
    pub new_folder_open: bool,
    pub new_folder_input: Option<Entity<InputState>>,
    pub preview: Option<FilePreview>,
    pub perms: Option<PermsEdit>,
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
            preview: None,
            perms: None,
        }
    }
}

impl FilesState {
    /// Moves to `path` and clears per-directory UI state; the caller reloads the listing.
    pub fn set_path(&mut self, path: &str) {
        self.current_path = path.to_string();
        self.pending_delete = None;
        self.perms = None;
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
