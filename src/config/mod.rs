pub mod plugin;
pub mod crawler;
pub mod versioning;

pub use plugin::{default_config_toml, CrowConfigPlugin, CROW_CONFIG_MANIFEST};
pub use crawler::{crawl_machine_configs, detect_schema_kind, sample_config_content, DiscoveredConfigFile, SchemaKind};
pub use versioning::{compute_unified_diff, ConfigFileState, ConfigRevision};

use crow_config_core::cst::CstNode;
use crow_config_core::edit::{ConfigPlugin, EditError, EditOp};
use crow_config_core::ir::{ConfigDocumentIr, FieldIr};
use std::fs;
use std::path::PathBuf;

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
