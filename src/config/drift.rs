//! Drift from a baseline (ERR-74): does a server's copy of a file mean the
//! same as the known-good one?
//!
//! Where crow-config has a plugin, both files are compared by meaning: their
//! rows (comments, blank lines and `comment` fields left out), in order where
//! the format says order matters. A reworded comment isn't drift; a changed
//! value is. Other files are compared as text, ignoring trailing whitespace.

use super::history::open_content;
use super::plugins::{editor_for, to_ir, ConfigEditor, StructuredFormat};
use crate::vault::{ConfigBaseline, MasterKey, ServerRecord, VaultDb, VaultError, BASELINE_FLEET};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Drift {
    /// Byte-for-byte the baseline.
    Identical,
    /// Different text, same meaning (comments, spacing, layout).
    Cosmetic,
    /// Means something else; one line per difference.
    Differs(Vec<String>),
}

impl Drift {
    pub fn is_drift(&self) -> bool {
        matches!(self, Drift::Differs(_))
    }
}

pub fn compare(format: Option<StructuredFormat>, baseline: &str, actual: &str) -> Drift {
    if baseline == actual {
        return Drift::Identical;
    }
    let semantic = format.and_then(|f| Some((meaningful_rows(f, baseline)?, meaningful_rows(f, actual)?)));
    let diffs = match semantic {
        Some(((base, ordered), (act, _))) => row_differences(base, act, ordered),
        // Plain text, or a file the plugin can't parse: compare lines.
        None => {
            let lines = |t: &str| t.lines().map(|l| l.trim_end().to_string()).filter(|l| !l.is_empty()).collect::<Vec<_>>();
            row_differences(lines(baseline), lines(actual), true)
        }
    };
    if diffs.is_empty() { Drift::Cosmetic } else { Drift::Differs(diffs) }
}

/// The file's rows that carry meaning, each as "field=value …", and whether
/// their order matters. `None` if the plugin can't read the file.
fn meaningful_rows(format: StructuredFormat, text: &str) -> Option<(Vec<String>, bool)> {
    let ir = to_ir(format, text).ok()?;
    let rows = ir
        .rows
        .iter()
        .filter(|r| r.is_comment_only != Some(true) && r.is_blank != Some(true))
        .map(|r| {
            r.fields
                .iter()
                .filter(|f| f.name != "comment")
                .map(|f| match &f.value {
                    serde_json::Value::String(s) => s.clone(),
                    v => v.to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|r| !r.trim().is_empty())
        .collect();
    Some((rows, ir.shape.order_sensitive))
}

/// "missing: …" for baseline rows the actual file lacks, "extra: …" for rows
/// only it has; "moved: …" when both have a row but in a different place
/// and order matters.
fn row_differences(mut base: Vec<String>, mut act: Vec<String>, ordered: bool) -> Vec<String> {
    let mut out = Vec::new();
    let missing: Vec<String> = base.iter().filter(|r| !act.contains(r)).cloned().collect();
    let extra: Vec<String> = act.iter().filter(|r| !base.contains(r)).cloned().collect();
    out.extend(missing.iter().map(|r| format!("missing: {r}")));
    out.extend(extra.iter().map(|r| format!("extra: {r}")));
    if out.is_empty() && ordered {
        base.retain(|r| !missing.contains(r));
        act.retain(|r| !extra.contains(r));
        if base != act {
            out.push("same lines in a different order (order matters in this file)".into());
        }
    }
    out
}

/// The crow-config format for a file at `path`, if it has one.
pub fn format_for_path(path: &str) -> Option<StructuredFormat> {
    let p = std::path::Path::new(path);
    let name = p.file_name()?.to_string_lossy();
    match editor_for(super::crawler::detect_schema_kind(&name, p)) {
        ConfigEditor::Structured(f) => Some(f),
        _ => None,
    }
}

/// The baseline for `path` on a server in `group`: the group's own, else
/// the fleet's.
pub fn baseline_for<'a>(baselines: &'a [ConfigBaseline], path: &str, group: &str) -> Option<&'a ConfigBaseline> {
    let group = group.trim();
    baselines
        .iter()
        .find(|b| b.path == path && !group.is_empty() && b.scope == group)
        .or_else(|| baselines.iter().find(|b| b.path == path && b.scope == BASELINE_FLEET))
}

/// One server's copy of a baselined file, compared with the baseline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriftEntry {
    pub server_id: String,
    pub path: String,
    pub scope: String,
    pub drift: Drift,
    /// When that copy was last read from the server (RFC 3339).
    pub as_of: String,
}

/// Every server's last-read copy of every baselined file, against its
/// baseline. Uses recorded history only: no server is contacted, so this is
/// as current as the last time each server's configs were loaded.
pub fn fleet_drift(db: &VaultDb, key: Option<&MasterKey>, servers: &[ServerRecord]) -> Result<Vec<DriftEntry>, VaultError> {
    let baselines = db.list_config_baselines()?;
    let mut paths: Vec<&str> = baselines.iter().map(|b| b.path.as_str()).collect();
    paths.dedup();
    let mut out = Vec::new();
    for path in paths {
        for rev in db.latest_config_revisions_for_path(path)? {
            let Some(server) = servers.iter().find(|s| s.id == rev.server_id) else { continue };
            let Some(b) = baseline_for(&baselines, path, &server.group_name) else { continue };
            let Some(base) = db.get_config_revision(&b.revision_id)? else { continue };
            let drift = if base.sha256 == rev.sha256 {
                Drift::Identical
            } else {
                match (open_content(&base, key), open_content(&rev, key)) {
                    (Some(b), Some(a)) => compare(format_for_path(path), &b, &a),
                    _ => Drift::Differs(vec!["differs from the baseline (only hashes kept, so Crow can't say how)".into()]),
                }
            };
            out.push(DriftEntry { server_id: rev.server_id, path: path.to_string(), scope: b.scope.clone(), drift, as_of: rev.created_at });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_and_spacing_are_not_drift() {
        let base = "# my hosts\n127.0.0.1 localhost\n10.0.0.5 db\n";
        let reworded = "# hosts file, managed\n127.0.0.1   localhost\n\n10.0.0.5 db   # the database\n";
        assert_eq!(compare(Some(StructuredFormat::Hosts), base, reworded), Drift::Cosmetic);
        assert_eq!(compare(Some(StructuredFormat::Hosts), base, base), Drift::Identical);
    }

    #[test]
    fn changed_values_are_drift() {
        let base = "Port 22\nPasswordAuthentication no\n";
        let drifted = "Port 22\nPasswordAuthentication yes\n";
        match compare(Some(StructuredFormat::Sshd), base, drifted) {
            Drift::Differs(d) => {
                assert!(d.iter().any(|l| l.starts_with("missing:") && l.contains("no")), "{d:?}");
                assert!(d.iter().any(|l| l.starts_with("extra:") && l.contains("yes")), "{d:?}");
            }
            other => panic!("expected drift, got {other:?}"),
        }
    }

    #[test]
    fn plain_text_ignores_trailing_space_and_blank_lines_only() {
        assert_eq!(compare(None, "a=1\nb=2\n", "a=1  \n\nb=2"), Drift::Cosmetic);
        assert!(compare(None, "a=1\n", "a=2\n").is_drift());
        assert!(compare(None, "a\nb\n", "b\na\n").is_drift(), "plain text: order counts");
    }

    #[test]
    fn fleet_drift_uses_group_baseline_before_fleet_baseline() {
        use crate::config::history::{record, SOURCE_CROW, SOURCE_OBSERVED};
        let db = VaultDb::open_in_memory().unwrap();
        let key = crate::vault::generate_data_key();
        let path = "/etc/ssh/sshd_config";
        let srv = |id: &str, group: &str| ServerRecord { id: id.into(), name: id.into(), group_name: group.into(), ..Default::default() };
        let servers = [srv("web1", "web"), srv("web2", "web"), srv("db1", "db"), srv("new", "")];

        let good = record(&db, Some(&key), "web1", path, "Port 22\nPasswordAuthentication no\n", "me", "hardened", SOURCE_CROW).unwrap();
        db.set_config_baseline(path, BASELINE_FLEET, &good.id, "me").unwrap();
        let db_good = record(&db, Some(&key), "db1", path, "Port 2222\n", "me", "db port", SOURCE_CROW).unwrap();
        db.set_config_baseline(path, "db", &db_good.id, "me").unwrap();
        record(&db, Some(&key), "web2", path, "# reworded\nPort 22\nPasswordAuthentication no\n", "on the host", "seen", SOURCE_OBSERVED).unwrap();
        record(&db, Some(&key), "new", path, "Port 22\nPasswordAuthentication yes\n", "on the host", "seen", SOURCE_OBSERVED).unwrap();

        let mut by_server: Vec<(String, String, bool)> = fleet_drift(&db, Some(&key), &servers).unwrap().into_iter().map(|e| (e.server_id, e.scope, e.drift.is_drift())).collect();
        by_server.sort();
        assert_eq!(by_server, vec![
            ("db1".into(), "db".into(), false),
            ("new".into(), BASELINE_FLEET.into(), true),
            ("web1".into(), BASELINE_FLEET.into(), false),
            ("web2".into(), BASELINE_FLEET.into(), false),
        ]);
    }
}
