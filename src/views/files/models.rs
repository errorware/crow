use gpui_kit::Rgba;
use crate::theme::*;

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size_bytes: u64,
    pub mode_str: String,
    pub owner: String,
    pub group: String,
    pub modified: String,
}

impl FileEntry {
    pub fn display_size(&self) -> String {
        if self.is_dir {
            return "—".to_string();
        }
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else if self.size_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    pub fn icon_color(&self) -> Rgba {
        if self.is_dir {
            hex_rgb(0x60a5fa)
        } else if self.is_symlink {
            hex_rgb(0x38bdf8)
        } else {
            TEXT_SECONDARY
        }
    }
}
