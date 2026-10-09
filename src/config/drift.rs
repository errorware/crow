//! Drift from a baseline (ERR-74): does a server's copy of a file mean the
//! same as the known-good one?
//!
//! Where crow-config has a plugin, both files are compared by meaning: their
//! rows (comments, blank lines and `comment` fields left out), in order where
//! the format says order matters. A reworded comment isn't drift; a changed
//! value is. Other files are compared as text, ignoring trailing whitespace.

use crate::config::history::open_content;
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

/// How often the baselined files are read again on every server.
pub const DRIFT_EVERY_SECS: i64 = 15 * 60;

/// The alert kind for a drifted file.
pub fn alert_kind(path: &str) -> String {
    format!("drift:{path}")
}

/// The baselined paths that apply to a server in `group`.
pub fn paths_for(baselines: &[ConfigBaseline], group: &str) -> Vec<String> {
    let mut paths: Vec<String> = baselines.iter().filter(|b| baseline_for(baselines, &b.path, group).is_some_and(|x| x.scope == b.scope)).map(|b| b.path.clone()).collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Reads each of `paths` on `host` and records a revision of the ones that
/// changed since Crow last saw them (as "observed": changed on the
/// server). A file that isn't there is skipped; one that can't be read is
/// reported. Returns the paths it could read.
pub fn read_and_record(host: &dyn crate::host::Host, db: &std::sync::Mutex<VaultDb>, key: Option<&MasterKey>, server_id: &str, paths: &[String]) -> (Vec<String>, Vec<(String, String)>) {
    use crate::host::DEFAULT_TIMEOUT;
    let mut read = Vec::new();
    let mut failed = Vec::new();
    for path in paths {
        let content = host.read_file(path).or_else(|_| host.exec_privileged(&["cat", "--", path], &[], DEFAULT_TIMEOUT).map(|o| o.stdout));
        let content = match content {
            Ok(c) => c,
            Err(e) => {
                let gone = host.exec(&["test", "-e", path], DEFAULT_TIMEOUT).is_err() && host.exec_privileged(&["test", "-e", path], &[], DEFAULT_TIMEOUT).is_err();
                if !gone {
                    failed.push((path.clone(), e.to_string()));
                }
                continue;
            }
        };
        if let Ok(db) = db.lock() {
            let latest = db.list_config_revisions(server_id, path).ok().and_then(|r| r.last().map(|r| r.sha256.clone()));
            if latest.as_deref() != Some(super::history::sha256_hex(&content).as_str()) {
                let _ = super::history::record(&db, key, server_id, path, &content, "on the host", "Changed on the server", super::history::SOURCE_OBSERVED);
            }
        }
        read.push(path.clone());
    }
    (read, failed)
}

/// Alerts for drift (ERR-143): a WARN per drifted file on the servers
/// just checked (`checked`: server id → paths read), resolved once the
/// file matches its baseline again. Files not read this round are left
/// as they are.
pub fn alert_changes(open: &[crate::metrics::alerts::Alert], entries: &[DriftEntry], checked: &std::collections::HashMap<String, Vec<String>>, now: i64) -> crate::metrics::alerts::AlertChanges {
    use crate::metrics::alerts::{Alert, AlertChanges};
    let mut changes = AlertChanges::default();
    for e in entries {
        if !checked.get(&e.server_id).is_some_and(|paths| paths.contains(&e.path)) {
            continue;
        }
        let kind = alert_kind(&e.path);
        let existing = open.iter().find(|a| a.server_id == e.server_id && a.kind == kind && a.resolved_at.is_none());
        match (&e.drift, existing) {
            (Drift::Differs(d), existing) => {
                let scope = if e.scope == BASELINE_FLEET { "fleet".to_string() } else { format!("{} group", e.scope) };
                let first = d.first().cloned().unwrap_or_default();
                let more = if d.len() > 1 { format!(" (+{} more)", d.len() - 1) } else { String::new() };
                let detail = format!("{} drifted from the {scope} baseline: {first}{more}", e.path);
                match existing {
                    Some(a) => changes.updated.push((a.id.clone(), "WARN".into(), detail, now)),
                    None => changes.opened.push(Alert {
                        id: format!("alert-{}-{kind}-{now}", e.server_id),
                        server_id: e.server_id.clone(),
                        kind,
                        level: "WARN".into(),
                        detail,
                        opened_at: now,
                        last_seen: now,
                        resolved_at: None,
                        acknowledged_at: None,
                    }),
                }
            }
            (_, Some(a)) => changes.resolved.push(a.id.clone()),
            (_, None) => {}
        }
    }
    // A baseline that was removed: its alerts go too.
    for a in open.iter().filter(|a| a.resolved_at.is_none() && a.kind.starts_with("drift:")) {
        let path = &a.kind["drift:".len()..];
        let still_baselined = entries.iter().any(|e| e.server_id == a.server_id && e.path == path);
        if !still_baselined && checked.contains_key(&a.server_id) {
            changes.resolved.push(a.id.clone());
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drifted_files_raise_alerts_and_matching_ones_resolve_them() {
        use std::collections::HashMap;
        let entry = |srv: &str, drift: Drift| DriftEntry { server_id: srv.into(), path: "/etc/ssh/sshd_config".into(), scope: "web".into(), drift, as_of: String::new() };
        let checked: HashMap<String, Vec<String>> = [("a".to_string(), vec!["/etc/ssh/sshd_config".to_string()]), ("b".to_string(), vec!["/etc/ssh/sshd_config".to_string()])].into();
        let entries = vec![entry("a", Drift::Differs(vec!["missing: PasswordAuthentication no".into(), "extra: x".into()])), entry("b", Drift::Cosmetic), entry("c", Drift::Differs(vec!["x".into()]))];
        let c = alert_changes(&[], &entries, &checked, 100);
        assert_eq!(c.opened.len(), 1, "c wasn't read this round");
        assert_eq!(c.opened[0].kind, "drift:/etc/ssh/sshd_config");
        assert!(c.opened[0].detail.contains("web group baseline: missing: PasswordAuthentication no (+1 more)"), "{}", c.opened[0].detail);
        // b's old alert resolves now that it matches.
        let old = crate::metrics::alerts::Alert { id: "x".into(), server_id: "b".into(), kind: "drift:/etc/ssh/sshd_config".into(), level: "WARN".into(), ..Default::default() };
        let c = alert_changes(&[old], &entries, &checked, 100);
        assert_eq!(c.resolved, vec!["x".to_string()]);
    }

    #[test]
    fn a_servers_group_baseline_hides_the_fleet_one_for_that_path() {
        let b = |path: &str, scope: &str| ConfigBaseline { path: path.into(), scope: scope.into(), revision_id: String::new(), set_by: String::new(), set_at: String::new() };
        let baselines = vec![b("/etc/a", BASELINE_FLEET), b("/etc/a", "web"), b("/etc/b", "db")];
        assert_eq!(paths_for(&baselines, "web"), ["/etc/a"]);
        assert_eq!(paths_for(&baselines, "db"), ["/etc/a", "/etc/b"]);
        assert_eq!(paths_for(&baselines, ""), ["/etc/a"]);
    }

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

    /// The whole drift loop against two real servers (throwaway containers
    /// whose /etc/crow-demo.conf differ), with a real vault key:
    ///   podman run -d --name crow-drift-1 alpine sleep 7200   (and crow-drift-2)
    ///   cargo test live_drift_loop -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_drift_loop() {
        use crate::host::ContainerHost;
        use std::collections::HashMap;
        let path = "/etc/crow-demo.conf".to_string();
        let (h1, h2) = (ContainerHost::new("podman", "crow-drift-1"), ContainerHost::new("podman", "crow-drift-2"));
        use crate::host::Host;
        h1.exec(&["sh", "-c", "printf 'workers = 8\\ntimeout = 30\\nPermitRootLogin no\\n' > /etc/crow-demo.conf"], crate::host::DEFAULT_TIMEOUT).unwrap();
        h2.exec(&["sh", "-c", "printf 'workers = 8\\ntimeout = 90\\nPermitRootLogin yes\\n' > /etc/crow-demo.conf"], crate::host::DEFAULT_TIMEOUT).unwrap();
        let db = std::sync::Mutex::new(VaultDb::open_in_memory().unwrap());
        let key = crate::vault::generate_data_key();
        let srv = |id: &str| ServerRecord { id: id.into(), name: id.into(), group_name: "web".into(), login_user: "root".into(), ..Default::default() };
        let servers = [srv("d1"), srv("d2")];
        // Pin d1's copy as the web baseline.
        read_and_record(&h1, &db, Some(&key), "d1", &[path.clone()]);
        let rev = db.lock().unwrap().list_config_revisions("d1", &path).unwrap().pop().unwrap();
        db.lock().unwrap().set_config_baseline(&path, "web", &rev.id, "test").unwrap();
        // A check reads d2 and finds the drift, with what changed.
        let (read, failed) = read_and_record(&h2, &db, Some(&key), "d2", &[path.clone()]);
        assert!(failed.is_empty() && read == [path.clone()]);
        let entries = fleet_drift(&db.lock().unwrap(), Some(&key), &servers).unwrap();
        let d2 = entries.iter().find(|e| e.server_id == "d2").unwrap();
        eprintln!("d2: {:?}", d2.drift);
        assert!(matches!(&d2.drift, Drift::Differs(d) if d.iter().any(|l| l.contains("PermitRootLogin yes"))));
        let checked: HashMap<String, Vec<String>> = [("d2".to_string(), vec![path.clone()])].into();
        let changes = alert_changes(&[], &entries, &checked, 1);
        eprintln!("alert: {}", changes.opened[0].detail);
        // The search finds the setting on both.
        let files: Vec<_> = db.lock().unwrap().latest_config_revisions().unwrap().into_iter().map(|r| { let c = crate::config::history::open_content(&r, Some(&key)); (r.server_id.clone(), r.server_id, r.path, c) }).collect();
        let hits = crate::app::drift::search(&files, "permitroot");
        assert_eq!(hits.len(), 2, "{hits:?}");
        // Bring d2 back: the baseline is written there, and it matches again.
        let base = crate::config::history::open_content(&rev, Some(&key)).unwrap();
        let t = crate::config::push::PushTarget { server_id: "d2", server_name: "d2", path: &path, baseline: &base, author: "test", login_user: "root", message: "Brought back to the baseline", context: "drift: bring back" };
        eprintln!("bring back: {:?}", crate::config::push::push_baseline(&h2, &db, Some(&key), &t));
        assert_eq!(h2.exec(&["cat", &path], crate::host::DEFAULT_TIMEOUT).unwrap().stdout, base);
        let entries = fleet_drift(&db.lock().unwrap(), Some(&key), &servers).unwrap();
        assert!(!entries.iter().any(|e| e.drift.is_drift()));
        let open = changes.opened.clone();
        assert_eq!(alert_changes(&open, &entries, &checked, 2).resolved.len(), 1, "the alert resolves");
    }
}
