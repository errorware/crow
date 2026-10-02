pub mod plugin;
pub mod crawler;
pub mod plugins;
pub mod crontab;
pub mod versioning;
pub mod history;
pub mod drift;
pub mod push;
pub mod syntax;

pub use plugin::{default_config_toml, CrowConfigPlugin, APPEARANCE_DEFAULTS, SERVERS_DEFAULTS, CROW_CONFIG_MANIFEST};
pub use crawler::{crawl_all_configs, crawl_configs, detect_schema_kind, load_config_file_states, DiscoveredConfigFile, SchemaKind};
pub use versioning::{compute_unified_diff, ConfigFileState, ConfigRevision};
pub use syntax::{highlight_config_line, SyntaxToken};

use crow_config_core::cst::CstNode;
use crow_config_core::edit::{ConfigPlugin, EditError, EditOp};
use crow_config_core::ir::{ConfigDocumentIr, FieldIr};
use std::fs;
use std::path::PathBuf;

/// Settings some code actually reads (ERR-69). Only these are shown in
/// Settings; a test checks each one is read outside the config and
/// settings views, so nothing is shown that does nothing.
pub const WIRED_SETTINGS: &[&str] = &[
    "general.refresh_interval",
    "connection.connect_timeout",
    "connection.keepalive_interval",
    "connection.control_master",
    "security.auto_lock_minutes",
    "servers.archive_purge_days",
    "appearance.fleet_background",
    "appearance.fleet_background_opacity",
    "appearance.fleet_background_blur",
    "appearance.terminal_font_size",
    "servers.geoip_regions",
];

/// Adds `line` right after `[section]`'s header when the section exists but
/// doesn't set `key` yet. Other text is kept as is.
pub fn add_missing_key(text: &str, section: &str, key: &str, line: &str) -> String {
    let header = format!("[{section}]");
    let mut out = Vec::new();
    let mut lines = text.lines().peekable();
    let mut done = false;
    while let Some(l) = lines.next() {
        out.push(l.to_string());
        if !done && l.trim() == header {
            // Does the section already set it (before the next header)?
            let rest: Vec<&str> = lines.clone().take_while(|n| !n.trim_start().starts_with('[')).collect();
            let has = rest.iter().any(|n| n.split('=').next().is_some_and(|k| k.trim() == key));
            if !has {
                out.push(line.to_string());
            }
            done = true;
        }
    }
    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffKind {
    Hunk,
    Context,
    Addition,
    Deletion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigDiffLine {
    pub kind: DiffKind,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct CrowConfigManager {
    pub path: PathBuf,
    pub baseline_text: String,
    pub baseline_ir: ConfigDocumentIr,
    pub plugin: CrowConfigPlugin,
    pub cst: CstNode,
    pub ir: ConfigDocumentIr,
}

impl CrowConfigManager {
    pub fn config_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".config").join("crow").join("config.toml")
        } else if let Some(dir) = dirs::config_dir() {
            dir.join("crow").join("config.toml")
        } else {
            PathBuf::from(".crow_config.toml")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let plugin = CrowConfigPlugin::new();

        let text = if path.exists() {
            fs::read_to_string(&path).unwrap_or_else(|_| default_config_toml().to_string())
        } else {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let default_text = default_config_toml().to_string();
            let _ = fs::write(&path, &default_text);
            default_text
        };

        // Config files from before Personalisation get its section (written
        // out with the next save), and files from before archiving get theirs.
        let text = if text.lines().any(|l| l.trim() == "[appearance]") {
            text
        } else {
            format!("{}\n{}", text.trim_end(), plugin::APPEARANCE_DEFAULTS)
        };
        // Keys added to an existing section later go inside it: a second
        // [appearance] header would make the file invalid TOML.
        let text = add_missing_key(&text, "appearance", "terminal_font_size", "terminal_font_size = 13");
        let text = if text.lines().any(|l| l.trim() == "[servers]") {
            text
        } else {
            format!("{}\n{}", text.trim_end(), plugin::SERVERS_DEFAULTS)
        };
        let text = add_missing_key(&text, "servers", "geoip_regions", "geoip_regions = false");
        let cst = plugin.parse(&text).unwrap_or_else(|_| {
            plugin.parse(default_config_toml()).expect("Default config must parse")
        });
        let ir = plugin.to_ir(&cst).expect("CST must bind to IR");

        Self {
            path,
            baseline_text: text,
            baseline_ir: ir.clone(),
            plugin,
            cst,
            ir,
        }
    }

    pub fn serialize(&self) -> String {
        self.cst.to_string_lossless()
    }

    pub fn update_field(&mut self, row_id: &str, new_value: serde_json::Value) -> Result<(), EditError> {
        let op = EditOp::UpdateField {
            row_id: row_id.to_string(),
            field_name: row_id.to_string(),
            new_value,
        };
        self.plugin.apply_edit(&mut self.cst, &op)?;
        if let Ok(new_ir) = self.plugin.to_ir(&self.cst) {
            self.ir = new_ir;
        }
        Ok(())
    }

    /// A saved setting as a whole number (settings apply once saved).
    pub fn saved_int(&self, row_id: &str) -> Option<i64> {
        let v = &self.baseline_ir.rows.iter().find(|r| r.row_id == row_id)?.get_field(row_id)?.value;
        v.as_i64().or_else(|| v.as_str()?.trim().parse().ok())
    }

    /// A saved setting as a yes/no.
    pub fn saved_bool(&self, row_id: &str) -> Option<bool> {
        let v = &self.baseline_ir.rows.iter().find(|r| r.row_id == row_id)?.get_field(row_id)?.value;
        v.as_bool().or_else(|| match v.as_str()?.trim() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        })
    }

    pub fn get_field(&self, row_id: &str) -> Option<&FieldIr> {
        for row in &self.ir.rows {
            if row.row_id == row_id {
                return row.get_field(row_id);
            }
        }
        None
    }

    pub fn is_field_changed(&self, row_id: &str) -> bool {
        let curr_val = self.get_field(row_id).map(|f| &f.value);
        let base_val = self
            .baseline_ir
            .rows
            .iter()
            .find(|r| r.row_id == row_id)
            .and_then(|r| r.get_field(row_id))
            .map(|f| &f.value);

        curr_val != base_val
    }

    pub fn changed_count_for_section(&self, sec_prefix: &str) -> usize {
        let prefix = format!("{}.", sec_prefix);
        self.ir
            .rows
            .iter()
            .filter(|r| r.row_id.starts_with(&prefix) && self.is_field_changed(&r.row_id))
            .count()
    }

    pub fn total_changed_count(&self) -> usize {
        self.ir
            .rows
            .iter()
            .filter(|r| self.is_field_changed(&r.row_id))
            .count()
    }

    pub fn reset_field(&mut self, row_id: &str) -> Result<(), EditError> {
        if let Some(base_val) = self
            .baseline_ir
            .rows
            .iter()
            .find(|r| r.row_id == row_id)
            .and_then(|r| r.get_field(row_id))
            .map(|f| f.value.clone())
        {
            self.update_field(row_id, base_val)?;
        }
        Ok(())
    }

    pub fn reset_section(&mut self, sec_prefix: &str) -> Result<(), EditError> {
        let prefix = format!("{}.", sec_prefix);
        let to_reset: Vec<(String, serde_json::Value)> = self
            .baseline_ir
            .rows
            .iter()
            .filter(|r| r.row_id.starts_with(&prefix))
            .filter_map(|r| r.get_field(&r.row_id).map(|f| (r.row_id.clone(), f.value.clone())))
            .collect();

        for (row_id, val) in to_reset {
            let _ = self.update_field(&row_id, val);
        }
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        let serialized = self.serialize();
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, &serialized)?;
        self.baseline_text = serialized;
        self.baseline_ir = self.ir.clone();
        Ok(())
    }

    pub fn generate_diff(&self) -> Vec<ConfigDiffLine> {
        let mut diff_lines = Vec::new();
        let curr_serialized = self.cst.to_string_lossless();
        let orig_lines: Vec<&str> = self.baseline_text.lines().collect();
        let curr_lines: Vec<&str> = curr_serialized.lines().collect();

        let mut current_sec = "";
        let max_lines = orig_lines.len().max(curr_lines.len());

        for i in 0..max_lines {
            let orig = orig_lines.get(i).copied();
            let curr = curr_lines.get(i).copied();

            if let Some(line) = curr.or(orig) {
                if line.trim().starts_with('[') && line.trim().ends_with(']') {
                    current_sec = line.trim();
                }
            }

            match (orig, curr) {
                (Some(o), Some(c)) if o != c => {
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Hunk,
                        text: format!("@@ {} @@", current_sec),
                    });
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Deletion,
                        text: format!("- {}", o.trim()),
                    });
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Addition,
                        text: format!("+ {}", c.trim()),
                    });
                }
                (None, Some(c)) if !c.trim().is_empty() => {
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Addition,
                        text: format!("+ {}", c.trim()),
                    });
                }
                (Some(o), None) if !o.trim().is_empty() => {
                    diff_lines.push(ConfigDiffLine {
                        kind: DiffKind::Deletion,
                        text: format!("- {}", o.trim()),
                    });
                }
                _ => {}
            }
        }

        diff_lines
    }
}

#[cfg(test)]
mod wired_tests {
    use super::*;

    /// Every shown setting is read by code outside the config module and
    /// the Settings views; otherwise it would be shown doing nothing.
    #[test]
    fn every_wired_setting_is_read_somewhere() {
        fn walk(dir: &std::path::Path, out: &mut String) {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                let s = p.to_string_lossy();
                if s.contains("/src/config") || s.contains("/views/settings") {
                    continue;
                }
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push_str(&std::fs::read_to_string(&p).unwrap());
                }
            }
        }
        let mut code = String::new();
        walk(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut code);
        for key in WIRED_SETTINGS {
            assert!(code.contains(&format!("\"{key}\"")), "{key} is shown in Settings but no code reads it");
        }
        let manifest: Vec<&str> = CROW_CONFIG_MANIFEST.fields.iter().map(|f| f.name.as_str()).collect();
        for key in WIRED_SETTINGS {
            assert!(manifest.contains(key), "{key} isn't in the settings manifest");
        }
    }

    #[test]
    fn missing_keys_go_inside_their_existing_section() {
        let old = "[general]\ntheme = \"x\"\n\n[appearance]\nfleet_background = \"\"\n\n[servers]\narchive_purge_days = \"90\"\n";
        let new = add_missing_key(old, "appearance", "terminal_font_size", "terminal_font_size = 13");
        assert_eq!(new, old.replace("[appearance]\n", "[appearance]\nterminal_font_size = 13\n"));
        assert_eq!(new.matches("[appearance]").count(), 1, "still valid TOML");
        assert_eq!(add_missing_key(&new, "appearance", "terminal_font_size", "terminal_font_size = 13"), new, "added once");
        assert_eq!(add_missing_key("[general]\n", "appearance", "k", "k = 1"), "[general]\n", "no section, nothing added");
    }
}
