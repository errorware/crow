//! Which editor a config file gets — the single source of truth behind both
//! the Config screen's "CROW UI" label and the editor it opens.
//!
//! Files a crow-config plugin understands are edited through the generic
//! structured editor; files that have a dedicated Crow screen open there;
//! everything else gets the plain-text editor.

use std::sync::OnceLock;

use crow_config_core::edit::{ConfigDocument, ConfigPlugin, EditOp};
use crow_config_core::ir::ConfigDocumentIr;
use crow_config_core::schema::RiskLevel;
use crow_config_schemas::{HostsPlugin, PgHbaPlugin, SshdPlugin};

use super::SchemaKind;
use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

/// crow-config plugins the Config screen renders with the generic editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructuredFormat {
    Hosts,
    Sshd,
    PgHba,
}

/// A Crow screen that owns a config file better than a file editor would.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DedicatedScreen {
    Cron,
    Firewall,
    Users,
    Settings,
}

impl DedicatedScreen {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Cron => "Cron",
            Self::Firewall => "Firewall",
            Self::Users => "Users",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConfigEditor {
    /// Generic editor driven by a crow-config plugin.
    Structured(StructuredFormat),
    /// Crow's own journald retention editor (no crow-config plugin yet).
    Journald,
    /// Managed on another Crow screen.
    Screen(DedicatedScreen),
    /// Plain-text editor.
    Text,
}

impl ConfigEditor {
    /// True when Crow offers more than plain text — what "CROW UI" means.
    pub fn is_crow_ui(&self) -> bool {
        !matches!(self, ConfigEditor::Text)
    }
}

/// The editor for a file of the given detected format.
pub fn editor_for(kind: Option<SchemaKind>) -> ConfigEditor {
    match kind {
        Some(SchemaKind::Hosts) => ConfigEditor::Structured(StructuredFormat::Hosts),
        Some(SchemaKind::Sshd) => ConfigEditor::Structured(StructuredFormat::Sshd),
        Some(SchemaKind::PgHba) => ConfigEditor::Structured(StructuredFormat::PgHba),
        Some(SchemaKind::Journald) => ConfigEditor::Journald,
        Some(SchemaKind::Cron) => ConfigEditor::Screen(DedicatedScreen::Cron),
        Some(SchemaKind::Ufw) => ConfigEditor::Screen(DedicatedScreen::Firewall),
        Some(SchemaKind::Accounts) => ConfigEditor::Screen(DedicatedScreen::Users),
        Some(SchemaKind::Crow) => ConfigEditor::Screen(DedicatedScreen::Settings),
        None => ConfigEditor::Text,
    }
}

/// The crow-config plugin for a structured format. Plugins are stateless, so
/// one shared instance each is enough.
pub fn plugin(format: StructuredFormat) -> &'static dyn ConfigPlugin {
    static HOSTS: OnceLock<HostsPlugin> = OnceLock::new();
    static SSHD: OnceLock<SshdPlugin> = OnceLock::new();
    static PG_HBA: OnceLock<PgHbaPlugin> = OnceLock::new();
    match format {
        StructuredFormat::Hosts => HOSTS.get_or_init(HostsPlugin::new),
        StructuredFormat::Sshd => SSHD.get_or_init(SshdPlugin::new),
        StructuredFormat::PgHba => PG_HBA.get_or_init(PgHbaPlugin::new),
    }
}

/// Parses `text` into the plugin's view model.
pub fn to_ir(format: StructuredFormat, text: &str) -> Result<ConfigDocumentIr, String> {
    let doc = ConfigDocument::parse(plugin(format), text).map_err(|e| e.to_string())?;
    doc.to_ir().map_err(|e| e.to_string())
}

/// Applies one edit to `text` and returns the new text. Only the edited
/// tokens change; comments and formatting elsewhere survive byte-for-byte.
pub fn apply_edit(format: StructuredFormat, text: &str, op: &EditOp) -> Result<String, String> {
    let mut doc = ConfigDocument::parse(plugin(format), text).map_err(|e| e.to_string())?;
    doc.apply_edit(op).map_err(|e| e.to_string())?;
    Ok(doc.serialize())
}

/// The `Match` criteria scoping each row (`None` = global), by row id. In
/// sshd_config every directive after a `Match` line belongs to that block.
/// crow-config doesn't model scopes yet (ERR-12), so Crow derives them.
pub fn sshd_match_scopes(ir: &ConfigDocumentIr) -> Vec<(String, Option<String>)> {
    let mut scope: Option<String> = None;
    ir.rows
        .iter()
        .map(|row| {
            if let Some(m) = row.fields.iter().find(|f| f.name.eq_ignore_ascii_case("match")) {
                scope = Some(m.value.as_str().unwrap_or_default().to_string());
            }
            (row.row_id.clone(), scope.clone())
        })
        .collect()
}

/// Outcome of running a plugin's file validators on a host before a write.
#[derive(Debug, PartialEq, Eq)]
pub enum Validation {
    /// Every applicable validator passed.
    Passed,
    /// No validator applies to a whole file, or none is installed on the host.
    Skipped(String),
}

/// Runs the plugin's file-level validators (those with a `{file}`
/// placeholder, e.g. `sshd -t -f {file}`) against `content` on `host`: the
/// content goes to a temp file there, the validator runs, the temp file is
/// removed. A validator whose command isn't installed (exit 127) is skipped.
pub fn validate_on_host(host: &dyn Host, format: StructuredFormat, content: &str) -> Result<Validation, String> {
    let validators: Vec<String> = plugin(format)
        .manifest()
        .validators
        .iter()
        .filter(|v| v.command.contains("{file}"))
        .map(|v| v.command.clone())
        .collect();
    if validators.is_empty() {
        return Ok(Validation::Skipped("no file validator for this format".into()));
    }
    let tmp = host
        .exec(&["mktemp", "/tmp/crow-validate.XXXXXX"], DEFAULT_TIMEOUT)
        .map_err(|e| format!("could not create a temp file for validation: {e}"))?
        .stdout
        .trim()
        .to_string();
    let result = (|| {
        host.exec_stdin(&["sh", "-c", "cat > \"$1\"", "crow-validate", &tmp], content.as_bytes(), DEFAULT_TIMEOUT)
            .map_err(|e| format!("could not stage content for validation: {e}"))?;
        let mut skipped = Vec::new();
        for template in &validators {
            // Check the tool exists before asking for root: a missing tool is
            // "skipped", not a sudo failure. sbin dirs hold e.g. sshd.
            let tool = template.split_whitespace().next().unwrap_or_default();
            let probe = format!("PATH=\"$PATH:/usr/sbin:/sbin\" command -v {}", shell_quote(tool));
            if host.exec(&["sh", "-c", &probe], DEFAULT_TIMEOUT).is_err() {
                skipped.push(template.replace("{file}", "<file>"));
                continue;
            }
            let command = format!("PATH=\"$PATH:/usr/sbin:/sbin\"; {}", template.replace("{file}", &shell_quote(&tmp)));
            // Validators often need root (e.g. `sshd -t` reads the host keys).
            match host.exec_privileged(&["sh", "-c", &command], &[], DEFAULT_TIMEOUT) {
                Ok(_) => {}
                Err(HostError::Failed { status: 127, .. }) => skipped.push(template.replace("{file}", "<file>")),
                Err(e) => return Err(format!("`{}` rejected the file: {e}", template.replace("{file}", "<file>"))),
            }
        }
        Ok(if skipped.is_empty() {
            Validation::Passed
        } else {
            Validation::Skipped(format!("not installed on this host: {}", skipped.join(", ")))
        })
    })();
    let _ = host.exec(&["rm", "-f", "--", &tmp], DEFAULT_TIMEOUT);
    result
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// A never-on-prod value: (field name, value, what it means).
pub type RiskFinding = (String, String, String);

/// `never_on_prod` values present in `text`.
pub fn never_on_prod_values(format: StructuredFormat, text: &str) -> Vec<RiskFinding> {
    let Ok(ir) = to_ir(format, text) else { return Vec::new() };
    let mut out = Vec::new();
    for field in ir.rows.iter().flat_map(|r| r.fields.iter()) {
        let Some(value) = field.value.as_str() else { continue };
        let risky = field.options.iter().flatten().find(|o| o.value == value && o.risk == Some(RiskLevel::NeverOnProd));
        if let Some(opt) = risky {
            out.push((field.name.clone(), value.to_string(), opt.label.clone()));
        }
    }
    out
}

/// `never_on_prod` values in `current` that `baseline` did not already have
/// (counting duplicates), i.e. the ones this change introduces.
pub fn new_never_on_prod_values(format: StructuredFormat, baseline: &str, current: &str) -> Vec<RiskFinding> {
    let mut before = never_on_prod_values(format, baseline);
    never_on_prod_values(format, current)
        .into_iter()
        .filter(|finding| match before.iter().position(|b| b.0 == finding.0 && b.1 == finding.1) {
            Some(i) => {
                before.remove(i);
                false
            }
            None => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::detect_schema_kind;
    use std::path::Path;

    #[test]
    fn crow_ui_label_matches_the_editor_that_opens() {
        let cases = [
            ("hosts", "/etc/hosts", true),
            ("sshd_config", "/etc/ssh/sshd_config", true),
            ("pg_hba.conf", "/etc/postgresql/16/main/pg_hba.conf", true),
            ("journald.conf", "/etc/systemd/journald.conf", true),
            ("crontab", "/etc/crontab", true),
            ("user.rules", "/etc/ufw/user.rules", true),
            ("before.rules", "/etc/ufw/before.rules", false),
            ("passwd", "/etc/passwd", true),
            // Not the main crontab: no structured editor, so plain text.
            ("anacrontab", "/etc/anacrontab", false),
            ("e2scrub_all", "/etc/cron.d/e2scrub_all", false),
            ("resolv.conf", "/etc/resolv.conf", false),
        ];
        for (name, path, crow_ui) in cases {
            let editor = editor_for(detect_schema_kind(name, Path::new(path)));
            assert_eq!(editor.is_crow_ui(), crow_ui, "{path} -> {editor:?}");
        }
    }

    #[test]
    fn structured_edit_changes_only_the_edited_value() {
        let text = "# local names\n127.0.0.1  localhost\n10.0.4.12  db-01  # primary\n";
        let ir = to_ir(StructuredFormat::Hosts, text).unwrap();
        let row = ir.rows.iter().find(|r| r.fields.iter().any(|f| f.value == "10.0.4.12")).unwrap();
        let op = EditOp::UpdateField {
            row_id: row.row_id.clone(),
            field_name: "address".into(),
            new_value: serde_json::json!("10.0.4.13"),
        };
        let out = apply_edit(StructuredFormat::Hosts, text, &op).unwrap();
        assert_eq!(out, "# local names\n127.0.0.1  localhost\n10.0.4.13  db-01  # primary\n");
    }

    #[test]
    fn sshd_rows_after_match_are_scoped() {
        let text = "PasswordAuthentication yes\nMatch User deploy\n\tPermitRootLogin yes\n";
        let ir = to_ir(StructuredFormat::Sshd, text).unwrap();
        let scopes = sshd_match_scopes(&ir);
        assert_eq!(scopes[0].1, None);
        assert_eq!(scopes.last().unwrap().1.as_deref(), Some("User deploy"));
    }

    #[test]
    fn validation_skips_formats_without_file_validators_and_missing_tools() {
        use crate::host::LocalHost;
        // hosts only declares a per-address validator, not a file one.
        assert!(matches!(validate_on_host(&LocalHost, StructuredFormat::Hosts, "127.0.0.1 localhost\n"), Ok(Validation::Skipped(_))));
        // sshd declares `sshd -t -f {file}`. Garbage must be rejected where sshd
        // exists, and a missing sshd must be reported as skipped — never passed.
        let sshd_installed = ["/usr/sbin/sshd", "/usr/bin/sshd", "/sbin/sshd"].iter().any(|p| std::path::Path::new(p).exists());
        let result = validate_on_host(&LocalHost, StructuredFormat::Sshd, "ThisIsNotADirective yes\n");
        if sshd_installed {
            assert!(result.is_err(), "sshd -t must reject an unknown directive: {result:?}");
        } else {
            assert!(matches!(result, Ok(Validation::Skipped(_))), "missing sshd must be skipped: {result:?}");
        }
        assert_eq!(shell_quote("/tmp/a'b"), "'/tmp/a'\\''b'");
    }

    #[test]
    fn only_newly_introduced_never_on_prod_values_are_flagged() {
        let baseline = "host all all 10.0.0.0/8 trust\n";
        let current = "host all all 10.0.0.0/8 trust\nhost all all 0.0.0.0/0 password\n";
        let new = new_never_on_prod_values(StructuredFormat::PgHba, baseline, current);
        assert_eq!(new.len(), 1);
        assert_eq!((new[0].0.as_str(), new[0].1.as_str()), ("method", "password"));
        assert!(new_never_on_prod_values(StructuredFormat::PgHba, current, current).is_empty());
    }
}
