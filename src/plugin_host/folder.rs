//! The plugins folder (ERR-177): which plugins are there, whether each passes
//! the checks, and whether you accepted exactly these files.
//!
//! Accepting pins a hash of every file in the folder. Any change, even one
//! byte or a new file, makes the plugin "changed" until it's reviewed again.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::check::{check_dir, Report};

/// Where user plugins live, one folder each.
pub fn plugins_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".config").join("crow").join("plugins")
}

/// What you decided about a plugin, kept in the vault by its hash.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pin {
    pub accepted: Option<String>,
    pub refused: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    /// Never reviewed.
    New,
    /// Accepted, and the files are the ones accepted.
    Accepted,
    /// Accepted once, but a file has changed since: review again.
    Changed,
    /// You refused these files.
    Refused,
    /// Fails the checks; can't be accepted.
    Invalid,
}

#[derive(Debug, Clone)]
pub struct Installed {
    /// The folder name.
    pub id: String,
    pub dir: PathBuf,
    pub report: Report,
    pub hash: String,
    pub trust: Trust,
}

impl Installed {
    pub fn name(&self) -> &str {
        self.report.manifest.as_ref().map_or(&self.id, |m| &m.plugin.name)
    }

    /// Accepted and unchanged: the only state in which anything of it may run.
    pub fn usable(&self) -> bool {
        self.trust == Trust::Accepted
    }
}

/// A hash over every file's path, length and bytes, in path order.
pub fn hash(r: &Report) -> String {
    let mut h = Sha256::new();
    for f in &r.files {
        h.update(f.rel.as_bytes());
        h.update([0]);
        h.update(f.len.to_le_bytes());
        h.update(std::fs::read(r.dir.join(&f.rel)).unwrap_or_default());
        h.update([0]);
    }
    let digest = h.finalize();
    format!("sha256:{}", digest.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

pub fn trust(report: &Report, hash: &str, pin: &Pin) -> Trust {
    if !report.ok() {
        Trust::Invalid
    } else if pin.accepted.as_deref() == Some(hash) {
        Trust::Accepted
    } else if pin.refused.as_deref() == Some(hash) {
        Trust::Refused
    } else if pin.accepted.is_some() {
        Trust::Changed
    } else {
        Trust::New
    }
}

/// Checks one plugin folder. Reads files; runs nothing.
pub fn load(dir: &Path, pin: &Pin) -> Installed {
    let report = check_dir(dir);
    let hash = hash(&report);
    let trust = trust(&report, &hash, pin);
    let id = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    Installed { id, dir: dir.to_path_buf(), report, hash, trust }
}

/// Every plugin folder in `root`, sorted by name. Files at the top level,
/// and hidden folders, are ignored.
pub fn scan(root: &Path, pins: impl Fn(&str) -> Pin) -> Vec<Installed> {
    let Ok(entries) = std::fs::read_dir(root) else { return Vec::new() };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir() || t.is_symlink()))
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    dirs.into_iter()
        .map(|d| {
            let id = d.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            load(&d, &pins(&id))
        })
        .collect()
}

/// Vault flag keys for a plugin's pin.
pub fn accepted_flag(id: &str) -> String {
    format!("user_plugin.{id}.accepted")
}

pub fn refused_flag(id: &str) -> String {
    format!("user_plugin.{id}.refused")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{load, scan, Pin, Trust};

    fn copy_fixture(case: &str, id: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("crow-plugin-folder-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugin_host/fixtures").join(id);
        let to = root.join(id);
        std::fs::create_dir_all(&to).unwrap();
        for e in std::fs::read_dir(&from).unwrap() {
            let e = e.unwrap();
            std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
        to
    }

    #[test]
    fn accepting_pins_the_exact_files() {
        let dir = copy_fixture("pin", "restic-check");
        let fresh = load(&dir, &Pin::default());
        assert_eq!(fresh.trust, Trust::New);
        assert!(fresh.hash.starts_with("sha256:") && fresh.hash.len() == 71);

        let pin = Pin { accepted: Some(fresh.hash.clone()), refused: None };
        assert_eq!(load(&dir, &pin).trust, Trust::Accepted);
        assert!(load(&dir, &pin).usable());

        // One byte in a script.
        let script = dir.join("status.sh");
        let mut body = std::fs::read(&script).unwrap();
        body.push(b'\n');
        std::fs::write(&script, body).unwrap();
        let changed = load(&dir, &pin);
        assert_eq!(changed.trust, Trust::Changed);
        assert!(!changed.usable());

        // A new file counts too.
        std::fs::write(&script, std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugin_host/fixtures/restic-check/status.sh")).unwrap()).unwrap();
        assert_eq!(load(&dir, &pin).trust, Trust::Accepted, "back to the accepted bytes");
        std::fs::write(dir.join("extra.txt"), "hi").unwrap();
        assert_eq!(load(&dir, &pin).trust, Trust::Changed);
    }

    #[test]
    fn refused_stays_refused_until_the_files_change() {
        let dir = copy_fixture("refuse", "restic-check");
        let h = load(&dir, &Pin::default()).hash;
        let pin = Pin { accepted: None, refused: Some(h) };
        assert_eq!(load(&dir, &pin).trust, Trust::Refused);
        std::fs::write(dir.join("extra.txt"), "hi").unwrap();
        assert_eq!(load(&dir, &pin).trust, Trust::New);
    }

    #[test]
    fn a_plugin_that_fails_the_checks_is_invalid_even_if_accepted() {
        let dir = copy_fixture("invalid", "restic-check");
        let h = load(&dir, &Pin::default()).hash;
        std::fs::write(dir.join("plugin.toml"), "format = 1\n").unwrap();
        let pin = Pin { accepted: Some(h), refused: None };
        assert_eq!(load(&dir, &pin).trust, Trust::Invalid);
    }

    #[test]
    fn scan_lists_folders_in_order_and_skips_the_rest() {
        let a = copy_fixture("scan", "restic-check");
        let root = a.parent().unwrap().to_path_buf();
        let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugin_host/fixtures/devsec-hardening");
        let b = root.join("devsec-hardening");
        std::fs::create_dir_all(&b).unwrap();
        for e in std::fs::read_dir(&from).unwrap() {
            let e = e.unwrap();
            std::fs::copy(e.path(), b.join(e.file_name())).unwrap();
        }
        std::fs::create_dir_all(root.join(".hidden")).unwrap();
        std::fs::write(root.join("loose.toml"), "x").unwrap();
        let found = scan(&root, |_| Pin::default());
        assert_eq!(found.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["devsec-hardening", "restic-check"]);
        assert!(found.iter().all(|p| p.trust == Trust::New), "{:?}", found.iter().map(|p| &p.report.refusals).collect::<Vec<_>>());
        assert_eq!(scan(&root.join("nope"), |_| Pin::default()).len(), 0);
    }
}
