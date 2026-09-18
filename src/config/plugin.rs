use crow_config_core::cst::{CstNode, SourceSpan, Span, SyntaxKind};
use crow_config_core::edit::{BindError, ConfigPlugin, EditError, EditOp, ParseError};
use crow_config_core::ir::{ConfigDocumentIr, FieldIr, RowIr, ShapeIr};
use crow_config_core::schema::{FieldType, PluginManifest, WidgetKind};
use std::sync::LazyLock;

pub const CROW_CONFIG_MANIFEST_TOML: &str = include_str!("manifest.toml");

pub static CROW_CONFIG_MANIFEST: LazyLock<PluginManifest> = LazyLock::new(|| {
    PluginManifest::from_toml_str(CROW_CONFIG_MANIFEST_TOML)
        .expect("Failed to parse embedded Crow settings manifest")
});

pub fn default_config_toml() -> &'static str {
    r#"# ==============================================================================
# CROW (Control, Reliability & Operations Workbench) — Configuration File
#
# This file is loaded by Crow and can be freely edited in any text editor.
# Changes take effect on next launch or when reloaded in Settings.
# ==============================================================================

[general]
theme = "obsidian_edge"
font_family = "JetBrains Mono"
font_size = 12
refresh_interval = 2
titlebar_latency = true
confirm_destructive = true
log_buffer_lines = 10000
notify_failures = true
auto_update_check = true

[connection]
strict_host_key_checking = true
agent_forwarding = false
control_master = true
keepalive_interval = 15
connect_timeout = 10
reconnect_backoff = "exponential"
ciphers = "chacha20-poly1305,aes256-gcm"
compression = false

[keys]
default_identity = "~/.ssh/id_ed25519"
auto_rotate_days = 90
enforce_ed25519_only = false
agent_integration = true

[security]
auto_lock_minutes = 15
zeroize_on_drop = true
"#
}

/// Lossless parser and schema plugin for Crow's config.toml
#[derive(Debug, Clone)]
pub struct CrowConfigPlugin {
    manifest: &'static PluginManifest,
}

impl CrowConfigPlugin {
    pub fn new() -> Self {
        Self {
            manifest: &CROW_CONFIG_MANIFEST,
        }
    }
}

impl Default for CrowConfigPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigPlugin for CrowConfigPlugin {
    fn manifest(&self) -> &PluginManifest {
        self.manifest
    }

    fn parse(&self, text: &str) -> Result<CstNode, ParseError> {
        let mut children = Vec::new();
        let mut byte_offset = 0;

        for line in text.split_inclusive('\n') {
            let line_len = line.len();
            let span = Span::new(byte_offset, byte_offset + line_len);
            byte_offset += line_len;

            let trimmed = line.trim();
            if trimmed.is_empty() {
                children.push(CstNode::token(SyntaxKind::BlankLine, line, span));
            } else if trimmed.starts_with('#') {
                children.push(CstNode::token(SyntaxKind::CommentLine, line, span));
            } else if trimmed.starts_with('[') && trimmed.contains(']') {
                let sec_name = trimmed.trim_matches(|c| c == '[' || c == ']').trim();
                let token = CstNode::token(SyntaxKind::Custom("section".into()), line, span);
                let rule = CstNode::rule(
                    SyntaxKind::Custom(format!("section:{}", sec_name)),
                    vec![token],
                    span,
                );
                children.push(rule);
            } else if let Some((key_part, rest)) = line.split_once('=') {
                let leading_ws_len = line.len() - line.trim_start().len();
                let key_trimmed = key_part.trim();

                let mut entry_children = Vec::new();
                let mut curr = span.start;

                if leading_ws_len > 0 {
                    let ws_str = &line[..leading_ws_len];
                    entry_children.push(CstNode::token(
                        SyntaxKind::Whitespace,
                        ws_str,
                        Span::new(curr, curr + leading_ws_len),
                    ));
                    curr += leading_ws_len;
                }

                entry_children.push(CstNode::token(
                    SyntaxKind::Key,
                    key_trimmed,
                    Span::new(curr, curr + key_trimmed.len()),
                ));
                curr += key_trimmed.len();

                let after_key = key_part.len() - leading_ws_len - key_trimmed.len();
                if after_key > 0 {
                    let ws_str = &key_part[key_part.len() - after_key..];
                    entry_children.push(CstNode::token(
                        SyntaxKind::Whitespace,
                        ws_str,
                        Span::new(curr, curr + after_key),
                    ));
                    curr += after_key;
                }

                // Equals sign
                entry_children.push(CstNode::token(
                    SyntaxKind::Custom("equals".into()),
                    "=",
                    Span::new(curr, curr + 1),
                ));
                curr += 1;

                // Value part
                let (val_part, comment_part) = if let Some((v, c)) = rest.split_once('#') {
                    (v, Some(format!("#{}", c)))
                } else {
                    (rest, None)
                };

                let val_leading_ws_len = val_part.len() - val_part.trim_start().len();
                if val_leading_ws_len > 0 {
                    let ws_str = &val_part[..val_leading_ws_len];
                    entry_children.push(CstNode::token(
                        SyntaxKind::Whitespace,
                        ws_str,
                        Span::new(curr, curr + val_leading_ws_len),
                    ));
                    curr += val_leading_ws_len;
                }

                let val_trimmed = val_part.trim_end_matches(|c| c == '\r' || c == '\n').trim();
                entry_children.push(CstNode::token(
                    SyntaxKind::Value,
                    val_trimmed,
                    Span::new(curr, curr + val_trimmed.len()),
                ));
                curr += val_trimmed.len();

                if let Some(comment) = comment_part {
                    let between_val_and_comment = val_part.len() - val_leading_ws_len - val_trimmed.len();
                    if between_val_and_comment > 0 {
                        let ws_str = &val_part[val_part.len() - between_val_and_comment..];
                        entry_children.push(CstNode::token(
                            SyntaxKind::Whitespace,
                            ws_str,
                            Span::new(curr, curr + between_val_and_comment),
                        ));
                        curr += between_val_and_comment;
                    }
                    entry_children.push(CstNode::token(
                        SyntaxKind::Comment,
                        &comment,
                        Span::new(curr, curr + comment.len()),
                    ));
                } else {
                    let trailing_nl_start = val_part.len() - (val_part.len() - val_part.trim_end_matches(|c| c == '\r' || c == '\n').len());
                    let nl_str = &val_part[trailing_nl_start..];
                    if !nl_str.is_empty() {
                        entry_children.push(CstNode::token(
                            SyntaxKind::Newline,
                            nl_str,
                            Span::new(curr, curr + nl_str.len()),
                        ));
                    }
                }

                children.push(CstNode::rule(SyntaxKind::Entry, entry_children, span));
            } else {
                children.push(CstNode::token(SyntaxKind::Line, line, span));
            }
        }

        let total_span = Span::new(0, text.len());
        Ok(CstNode::rule(SyntaxKind::Document, children, total_span))
    }

    fn to_ir(&self, cst: &CstNode) -> Result<ConfigDocumentIr, BindError> {
        let mut rows = Vec::new();
        let mut current_section = "general".to_string();
        let mut current_line = 1;

        for child in cst.children() {
            let line_no = current_line;
            current_line += 1;

            if let SyntaxKind::Custom(s) = child.kind() {
                if let Some(sec) = s.strip_prefix("section:") {
                    current_section = sec.to_string();
                    continue;
                }
            }

            if child.kind() == &SyntaxKind::Entry {
                let mut key_name = String::new();
                let mut raw_val = String::new();

                for token in child.children() {
                    match token.kind() {
                        SyntaxKind::Key => {
                            if let CstNode::Token { text, .. } = token {
                                key_name = text.clone();
                            }
                        }
                        SyntaxKind::Value => {
                            if let CstNode::Token { text, .. } = token {
                                raw_val = text.clone();
                            }
                        }
                        _ => {}
                    }
                }

                if !key_name.is_empty() {
                    let full_name = format!("{}.{}", current_section, key_name);
                    let row_id = full_name.clone();

                    let def = self.manifest.find_field(&full_name);
                    let field_type = def.map(|d| d.field_type.clone()).unwrap_or(FieldType::String);

                    let json_val = match &field_type {
                        FieldType::Bool => {
                            serde_json::Value::Bool(raw_val.parse::<bool>().unwrap_or(false))
                        }
                        FieldType::Other(cow) if cow == "integer" => {
                            if let Ok(n) = raw_val.parse::<i64>() {
                                serde_json::Value::Number(serde_json::Number::from(n))
                            } else {
                                serde_json::Value::String(raw_val.clone())
                            }
                        }
                        _ => {
                            let unquoted = raw_val.trim_matches('"').trim_matches('\'');
                            serde_json::Value::String(unquoted.to_string())
                        }
                    };

                    let mut field = FieldIr::new(full_name, field_type, json_val);
                    if let Some(d) = def {
                        field.help = d.help.clone();
                        field.options = d.options.clone();
                        field.docs_source = d.docs_source.clone();
                        field.valid = Some(true);
                    }

                    let mut row = RowIr::new(row_id, "key_value_list_row", SourceSpan::single_line(line_no));
                    row.fields.push(field);
                    rows.push(row);
                }
            }
        }

        Ok(ConfigDocumentIr {
            plugin_name: "crow_config".to_string(),
            shape: ShapeIr {
                kind: WidgetKind::KeyValueList,
                order_sensitive: false,
                order_note: Some("Configuration values are organized by section.".to_string()),
            },
            rows,
        })
    }

    fn apply_edit(&self, cst: &mut CstNode, op: &EditOp) -> Result<(), EditError> {
        match op {
            EditOp::UpdateField { row_id, field_name: _, new_value } => {
                let target_key = if let Some((_, k)) = row_id.split_once('.') {
                    k
                } else {
                    row_id.as_str()
                };

                let target_sec = row_id.split_once('.').map(|(s, _)| s).unwrap_or("general");
                let mut current_sec = "general".to_string();

                let new_val_str = match new_value {
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    _ => new_value.to_string(),
                };

                if let CstNode::Rule { children, .. } = cst {
                    for node in children.iter_mut() {
                        if let SyntaxKind::Custom(s) = node.kind() {
                            if let Some(sec) = s.strip_prefix("section:") {
                                current_sec = sec.to_string();
                            }
                        }

                        if current_sec == target_sec && node.kind() == &SyntaxKind::Entry {
                            if let CstNode::Rule { children: entry_children, .. } = node {
                                let mut is_matching_key = false;
                                for token in entry_children.iter() {
                                    if token.kind() == &SyntaxKind::Key {
                                        if let CstNode::Token { text, .. } = token {
                                            if text == target_key {
                                                is_matching_key = true;
                                                break;
                                            }
                                        }
                                    }
                                }

                                if is_matching_key {
                                    for token in entry_children.iter_mut() {
                                        if token.kind() == &SyntaxKind::Value {
                                            *token = CstNode::token(
                                                SyntaxKind::Value,
                                                &new_val_str,
                                                Span::default(),
                                            );
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Err(EditError::RowNotFound(row_id.clone()))
            }
            _ => Err(EditError::Unsupported(format!("{:?}", op))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_lossless_roundtrip() {
        let plugin = CrowConfigPlugin::new();
        let default_text = default_config_toml();
        let cst = plugin.parse(default_text).expect("Must parse default config");
        assert_eq!(cst.to_string_lossless(), default_text);
    }

    #[test]
    fn test_ir_binding() {
        let plugin = CrowConfigPlugin::new();
        let default_text = default_config_toml();
        let cst = plugin.parse(default_text).unwrap();
        let ir = plugin.to_ir(&cst).expect("Must bind to IR");

        assert_eq!(ir.plugin_name, "crow_config");
        assert!(!ir.rows.is_empty());

        let strict = ir.rows.iter().find(|r| r.row_id == "connection.strict_host_key_checking");
        assert!(strict.is_some(), "connection.strict_host_key_checking row must exist");
        let field = strict.unwrap().get_field("connection.strict_host_key_checking").unwrap();
        assert_eq!(field.value, serde_json::Value::Bool(true));
        assert!(field.help.is_some());
    }

    #[test]
    fn test_edit_field_preserves_comments() {
        let plugin = CrowConfigPlugin::new();
        let default_text = default_config_toml();
        let mut cst = plugin.parse(default_text).unwrap();

        let op = EditOp::UpdateField {
            row_id: "connection.agent_forwarding".to_string(),
            field_name: "connection.agent_forwarding".to_string(),
            new_value: serde_json::Value::Bool(true),
        };
        plugin.apply_edit(&mut cst, &op).expect("Edit must succeed");

        let serialized = cst.to_string_lossless();
        assert!(serialized.contains("agent_forwarding = true"));
        assert!(serialized.contains("# CROW (Control, Reliability & Operations Workbench) — Configuration File"));
    }
}
