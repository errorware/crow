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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConfigEditor {
    /// Generic editor driven by a crow-config plugin.
    Structured(StructuredFormat),
    /// Crow's own journald retention editor (no crow-config plugin yet).
    Journald,
    /// Managed on another Crow screen; not listed on the Config screen.
    Screen(DedicatedScreen),
    /// Plain-text editor.
    Text,
}

impl ConfigEditor {
    /// True when the file opens in a Crow editor on the Config screen —
    /// what "CROW UI" means.
    pub fn is_crow_ui(&self) -> bool {
        matches!(self, ConfigEditor::Structured(_) | ConfigEditor::Journald)
    }

    /// False for files another Crow screen owns (crontab, ufw's rules, the
    /// account databases, Crow's own config): they stay off the Config list.
    pub fn is_listed(&self) -> bool {
        !matches!(self, ConfigEditor::Screen(_))
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

/// One directive on the sshd settings sheet: a known one (from the plugin
/// manifest, set or not) or one Crow has no schema for.
#[derive(Clone, Debug)]
pub struct SheetRow {
    pub name: String,
    pub def: Option<crow_config_core::schema::FieldDef>,
    /// Where it's set in the file, if it is.
    pub row_id: Option<String>,
    /// The key as written in the file (case preserved), when set.
    pub field_name: Option<String>,
    pub value: Option<String>,
    pub line: Option<usize>,
    /// A later duplicate: sshd uses the first value, so this one is ignored.
    pub shadowed: bool,
}

impl SheetRow {
    /// The value sshd uses: the file's, else the plugin's default.
    pub fn effective(&self) -> Option<&str> {
        self.value.as_deref().or_else(|| self.def.as_ref().and_then(|d| d.default.as_deref()))
    }
}

/// sshd_config laid out for people: known directives grouped by what they
/// do (unset ones shown at their OpenSSH default), then unknown directives,
/// then Match blocks (read-only until ERR-12).
#[derive(Clone, Debug, Default)]
pub struct SshdSheet {
    pub sections: Vec<(String, Vec<SheetRow>)>,
    pub other: Vec<SheetRow>,
    pub scoped: Vec<(String, Vec<SheetRow>)>,
    /// Where new global directives go: after the last line outside any
    /// Match block (appending after a Match would scope them to it).
    pub insert_after: Option<String>,
}

pub fn sshd_sheet(ir: &ConfigDocumentIr) -> SshdSheet {
    use crow_config_core::ir::RowIr;
    let defs = plugin(StructuredFormat::Sshd).manifest().fields.clone();
    let scopes = sshd_match_scopes(ir);
    let scope_of = |row: &RowIr| scopes.iter().find(|(id, _)| *id == row.row_id).and_then(|(_, s)| s.clone());
    let as_sheet_row = |row: &RowIr, def: Option<crow_config_core::schema::FieldDef>, shadowed: bool| {
        let f = row.fields.first();
        SheetRow {
            name: f.map(|f| f.name.clone()).unwrap_or_default(),
            def,
            row_id: Some(row.row_id.clone()),
            field_name: f.map(|f| f.name.clone()),
            value: f.map(|f| match &f.value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(a) => a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "),
                other => other.to_string(),
            }),
            line: Some(row.source_span.start_line),
            shadowed,
        }
    };

    let mut sheet = SshdSheet::default();
    let mut seen: Vec<String> = Vec::new();
    let mut global_rows: Vec<&RowIr> = Vec::new();
    for row in ir.rows.iter().filter(|r| !r.fields.is_empty()) {
        match scope_of(row) {
            Some(scope) => {
                let is_match_line = row.fields.iter().any(|f| f.name.eq_ignore_ascii_case("match"));
                if is_match_line {
                    sheet.scoped.push((scope, Vec::new()));
                } else if let Some((_, rows)) = sheet.scoped.last_mut() {
                    let def = defs.iter().find(|d| row.fields.first().is_some_and(|f| f.name.eq_ignore_ascii_case(&d.name))).cloned();
                    rows.push(as_sheet_row(row, def, false));
                }
            }
            None => {
                global_rows.push(row);
                sheet.insert_after = Some(row.row_id.clone());
            }
        }
    }
    // Known directives, grouped in manifest order; the first setting wins.
    for def in &defs {
        let group = def.group.clone().unwrap_or_else(|| "Other settings".into());
        let found = global_rows.iter().find(|r| r.fields.first().is_some_and(|f| f.name.eq_ignore_ascii_case(&def.name)));
        let row = match found {
            Some(r) => {
                seen.push(r.row_id.clone());
                as_sheet_row(r, Some(def.clone()), false)
            }
            None => SheetRow { name: def.name.clone(), def: Some(def.clone()), row_id: None, field_name: None, value: None, line: None, shadowed: false },
        };
        match sheet.sections.iter_mut().find(|(g, _)| *g == group) {
            Some((_, rows)) => rows.push(row),
            None => sheet.sections.push((group, vec![row])),
        }
    }
    // Everything else outside Match blocks: unknown directives and ignored
    // duplicates of known ones.
    for row in global_rows.into_iter().filter(|r| !seen.contains(&r.row_id)) {
        let def = defs.iter().find(|d| row.fields.first().is_some_and(|f| f.name.eq_ignore_ascii_case(&d.name))).cloned();
        let shadowed = def.is_some();
        sheet.other.push(as_sheet_row(row, def, shadowed));
    }
    sheet
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
/// The format's validator commands that check a file (templates with
/// `{file}`), e.g. `sshd -t -f {file}`.
pub fn file_validators(format: StructuredFormat) -> Vec<String> {
    plugin(format)
        .manifest()
        .validators
        .iter()
        .filter(|v| v.command.contains("{file}"))
        .map(|v| v.command.clone())
        .collect()
}

pub fn validate_on_host(host: &dyn Host, format: StructuredFormat, content: &str) -> Result<Validation, String> {
    let validators = file_validators(format);
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
    fn sshd_sheet_groups_directives_and_shows_defaults() {
        let text = "Include /etc/ssh/sshd_config.d/*.conf\nPort 2222\nPermitRootLogin no\nUsePAM yes\nPermitRootLogin yes\nMatch User deploy\n    PasswordAuthentication yes\n";
        let ir = to_ir(StructuredFormat::Sshd, text).unwrap();
        let sheet = sshd_sheet(&ir);
        let find = |name: &str| sheet.sections.iter().flat_map(|(_, rows)| rows).find(|r| r.name.eq_ignore_ascii_case(name)).unwrap().clone();
        assert_eq!(find("Port").value.as_deref(), Some("2222"));
        assert_eq!(find("PermitRootLogin").value.as_deref(), Some("no"), "first value wins");
        let pw = find("PasswordAuthentication");
        assert_eq!((pw.value.as_deref(), pw.effective()), (None, Some("yes")), "unset globally: the OpenSSH default");
        assert!(sheet.sections.iter().any(|(g, _)| g == "Authentication"));
        let other: Vec<(&str, bool)> = sheet.other.iter().map(|r| (r.name.as_str(), r.shadowed)).collect();
        assert_eq!(other, [("Include", false), ("UsePAM", false), ("PermitRootLogin", true)]);
        assert_eq!(sheet.scoped.len(), 1);
        assert_eq!(sheet.scoped[0].1[0].name, "PasswordAuthentication");
        let pos = ir.rows.iter().position(|r| Some(&r.row_id) == sheet.insert_after.as_ref()).unwrap();
        assert!(ir.rows[pos].fields[0].name == "PermitRootLogin", "new directives go before the Match block");
    }

    #[test]
    fn crow_ui_label_matches_the_editor_that_opens() {
        let cases = [
            ("hosts", "/etc/hosts", true),
            ("sshd_config", "/etc/ssh/sshd_config", true),
            ("pg_hba.conf", "/etc/postgresql/16/main/pg_hba.conf", true),
            ("journald.conf", "/etc/systemd/journald.conf", true),
            // Owned by other screens: not CROW UI here, and not listed.
            ("crontab", "/etc/crontab", false),
            ("user.rules", "/etc/ufw/user.rules", false),
            ("before.rules", "/etc/ufw/before.rules", false),
            ("passwd", "/etc/passwd", false),
            // Not the main crontab: no structured editor, so plain text.
            ("anacrontab", "/etc/anacrontab", false),
            ("e2scrub_all", "/etc/cron.d/e2scrub_all", false),
            ("resolv.conf", "/etc/resolv.conf", false),
        ];
        for (name, path, crow_ui) in cases {
            let editor = editor_for(detect_schema_kind(name, Path::new(path)));
            assert_eq!(editor.is_crow_ui(), crow_ui, "{path} -> {editor:?}");
            let screen_owned = matches!(name, "crontab" | "user.rules" | "passwd");
            assert_eq!(editor.is_listed(), !screen_owned, "{path} -> {editor:?}");
        }
    }

    #[test]
    fn multi_field_documents_get_one_column_set_with_the_comment_last() {
        use crate::views::config::structured_editor::table_columns;
        let names = |cols: Option<Vec<_>>| cols.map(|c: Vec<crate::views::config::structured_editor::Column>| c.into_iter().map(|c| c.name).collect::<Vec<_>>());
        let hosts = to_ir(StructuredFormat::Hosts, "127.0.0.1 localhost # loop\n::1 localhost ip6-localhost\n").unwrap();
        assert_eq!(names(table_columns(&hosts)), Some(vec!["address".into(), "hostnames".into(), "comment".into()]));
        let pg = to_ir(StructuredFormat::PgHba, "local all postgres peer\nhost all all 10.0.0.0/8 scram-sha-256\n").unwrap();
        let cols = names(table_columns(&pg)).unwrap();
        assert_eq!(&cols[..2], ["type", "database"]);
        assert!(cols.contains(&"address".to_string()), "a column even though the local row has no address");
        // Key/value documents stay as labelled lines.
        let sshd = to_ir(StructuredFormat::Sshd, "PermitRootLogin no\nPort 22\n").unwrap();
        assert!(table_columns(&sshd).is_none());
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
