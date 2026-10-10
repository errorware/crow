//! Pushing a baseline to one server (ERR-79), as a fleet-runner step.

use std::sync::Mutex;

use super::drift::{compare, format_for_path, Drift};
use super::history::{self, SOURCE_CROW};
use super::plugins;
use crate::host::{Host, DEFAULT_TIMEOUT};
use crate::vault::{ChangeRecord, MasterKey, VaultDb};
use crate::views::fleet::run::{StepOutcome, StepResult};

/// Where the step writes: which server, which file, and what the baseline says.
pub struct PushTarget<'a> {
    pub server_id: &'a str,
    pub server_name: &'a str,
    pub path: &'a str,
    pub baseline: &'a str,
    pub author: &'a str,
    /// Who Crow logs in as there: a sudoers push must leave them sudo.
    pub login_user: &'a str,
    /// The revision's message, and what the change record says it was part of.
    pub message: &'a str,
    pub context: &'a str,
}

/// Re-reads `path` on the host (it may have changed since Crow last looked),
/// skips it if it already means the same as the baseline, else checks the
/// baseline with the format's host validator, writes it atomically, and
/// records the revision and a change record.
pub fn push_baseline(host: &dyn Host, db: &Mutex<VaultDb>, key: Option<&MasterKey>, t: &PushTarget) -> StepResult {
    push_inner(host, db, key, t, false, &mut None)
}

/// What a file was before a push replaced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Before {
    /// It didn't exist: the push created it, and these folders on the way
    /// (outermost first).
    Missing { created_dirs: Vec<String> },
    Content(String),
}

/// [`push_baseline`] that may also create the file, returning what was
/// there before when it wrote (for a rollback).
pub fn push_file(host: &dyn Host, db: &Mutex<VaultDb>, key: Option<&MasterKey>, t: &PushTarget) -> (StepResult, Option<Before>) {
    let mut previous = None;
    let result = push_inner(host, db, key, t, true, &mut previous);
    (result, previous)
}

/// Puts a file back as it was: its old content, or gone if it was created.
pub fn restore(host: &dyn Host, db: &Mutex<VaultDb>, key: Option<&MasterKey>, t: &PushTarget, before: &Before) -> StepResult {
    match before {
        Before::Content(_) => push_baseline(host, db, key, t),
        Before::Missing { created_dirs } => {
            let argv = ["rm", "-f", "--", t.path];
            host.exec(&argv, DEFAULT_TIMEOUT).or_else(|_| host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT)).map_err(|e| format!("couldn't remove {}: {e}", t.path))?;
            remove_dirs(host, created_dirs);
            Ok(StepOutcome::Done(if created_dirs.is_empty() { "removed (it didn't exist before)".into() } else { format!("removed, with {} (none of it existed before)", created_dirs.join(", ")) }))
        }
    }
}

fn exists(host: &dyn Host, path: &str) -> bool {
    host.exec(&["test", "-e", path], DEFAULT_TIMEOUT).is_ok() || host.exec_privileged(&["test", "-e", path], &[], DEFAULT_TIMEOUT).is_ok()
}

/// The folders on the way to `path` that don't exist yet, outermost first.
fn missing_dirs(host: &dyn Host, path: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let mut at = std::path::Path::new(path).parent();
    while let Some(dir) = at.filter(|d| !d.as_os_str().is_empty() && *d != std::path::Path::new("/")) {
        let d = dir.to_string_lossy().to_string();
        if exists(host, &d) {
            break;
        }
        missing.push(d);
        at = dir.parent();
    }
    missing.reverse();
    missing
}

/// Removes folders a push created, innermost first, only while empty.
fn remove_dirs(host: &dyn Host, dirs: &[String]) {
    for d in dirs.iter().rev() {
        let argv = ["rmdir", "--", d.as_str()];
        let _ = host.exec(&argv, DEFAULT_TIMEOUT).or_else(|_| host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT));
    }
}

fn push_inner(host: &dyn Host, db: &Mutex<VaultDb>, key: Option<&MasterKey>, t: &PushTarget, may_create: bool, previous: &mut Option<Before>) -> StepResult {
    let read = host.read_file(t.path).or_else(|_| host.exec_privileged(&["cat", "--", t.path], &[], DEFAULT_TIMEOUT).map(|o| o.stdout));
    let (current, before) = match read {
        Ok(c) => (c.clone(), Before::Content(c)),
        Err(_) if may_create && !exists(host, t.path) => (String::new(), Before::Missing { created_dirs: Vec::new() }),
        Err(e) => return Err(format!("couldn't read {}: {e}", t.path)),
    };
    let format = format_for_path(t.path);
    let differences = match compare(format, t.baseline, &current) {
        Drift::Identical | Drift::Cosmetic => return Ok(StepOutcome::Skipped("already matches the baseline".into())),
        Drift::Differs(d) => d.len(),
    };
    // A new file may need its folders: created (as root when needed), and
    // remembered so a rollback (or a failed write) takes them away again.
    let mut before = before;
    if let Before::Missing { created_dirs } = &mut before {
        *created_dirs = missing_dirs(host, t.path);
        if let Some(parent) = created_dirs.last() {
            let argv = ["mkdir", "-p", "--", parent.as_str()];
            host.exec(&argv, DEFAULT_TIMEOUT).or_else(|_| host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT)).map_err(|e| format!("not written: couldn't create {parent}: {e}"))?;
        }
    }
    let written = (|| -> Result<(), String> {
        if format == Some(plugins::StructuredFormat::Nginx) {
            super::nginx::install_nginx(host, t.path, t.baseline).map_err(|e| format!("not written: {e}"))?;
        } else if format == Some(plugins::StructuredFormat::Sudoers) {
            // visudo and the keep-sudo guard run in the install itself.
            super::sudoers::install_sudoers(host, t.path, t.baseline, t.login_user).map_err(|e| format!("not written: {e}"))?;
        } else {
            if let Some(f) = format {
                plugins::validate_on_host(host, f, t.path, t.baseline).map_err(|e| format!("not written: the host's validator refused the baseline: {e}"))?;
            }
            if format == Some(plugins::StructuredFormat::Fstab) {
                super::fstab::check(host, &current, t.baseline).map_err(|e| format!("not written: {e}"))?;
            }
            host.write_file_privileged(t.path, t.baseline).map_err(|e| format!("not written: {e}"))?;
            if format == Some(plugins::StructuredFormat::Fstab) {
                super::fstab::reload_systemd(host);
            }
        }
        Ok(())
    })();
    if let Err(e) = written {
        if let Before::Missing { created_dirs } = &before {
            remove_dirs(host, created_dirs);
        }
        return Err(e);
    }

    *previous = Some(before);
    // Written. Recording can still fail; say so without calling the push a failure.
    let recorded = db.lock().map_err(|_| "the vault is busy".to_string()).and_then(|db| {
        let rev = history::record(&db, key, t.server_id, t.path, t.baseline, t.author, t.message, SOURCE_CROW).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        db.insert_change_record(&ChangeRecord {
            id: format!("chg-{}", rev.id.trim_start_matches("cfgrev-")),
            server_id: t.server_id.into(),
            server_name: t.server_name.into(),
            action_kind: "config.write".into(),
            target: t.path.into(),
            before_state: format!("sha256:{}", history::sha256_hex(&current)),
            after_state: Some(format!("sha256:{}", rev.sha256)),
            blast_radius: Some(t.context.into()),
            outcome: "applied".into(),
            started_at: now.clone(),
            completed_at: Some(now),
        })
        .map_err(|e| e.to_string())
    });
    let what = format!("written ({differences} difference{})", if differences == 1 { "" } else { "s" });
    Ok(StepOutcome::Done(match recorded {
        Ok(()) => what,
        Err(e) => format!("{what}; history not recorded: {e}"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::LocalHost;

    fn target<'a>(path: &'a str, baseline: &'a str) -> PushTarget<'a> {
        PushTarget { server_id: "s1", server_name: "web-1", path, baseline, author: "me@box", login_user: "deploy", message: "Pushed the baseline", context: "fleet run: push baseline" }
    }

    #[test]
    fn pushes_drifted_files_and_skips_matching_ones() {
        let dir = std::env::temp_dir().join(format!("crow-push-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("app.conf");
        let path = file.to_str().unwrap();
        let db = Mutex::new(VaultDb::open_in_memory().unwrap());
        let key = crate::vault::generate_data_key();

        std::fs::write(&file, "workers = 2\n").unwrap();
        let r = push_baseline(&LocalHost, &db, Some(&key), &target(path, "workers = 8\n"));
        assert_eq!(r, Ok(StepOutcome::Done("written (2 differences)".into())));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "workers = 8\n");
        let db_ = db.lock().unwrap();
        let revs = db_.list_config_revisions("s1", path).unwrap();
        assert_eq!((revs.len(), revs[0].message.as_str()), (1, "Pushed the baseline"));
        assert_eq!(db_.list_change_records("s1", 5).unwrap()[0].action_kind, "config.write");
        drop(db_);

        // The previous content comes back for a rollback.
        std::fs::write(&file, "workers = 3\n").unwrap();
        let (r, previous) = push_file(&LocalHost, &db, Some(&key), &target(path, "workers = 8\n"));
        assert!(r.is_ok());
        assert_eq!(previous, Some(Before::Content("workers = 3\n".into())));
        // A file that isn't there is created, and restoring removes it again.
        let fresh = dir.join("new.conf");
        let fresh_path = fresh.to_str().unwrap();
        let (r, previous) = push_file(&LocalHost, &db, Some(&key), &target(fresh_path, "x = 1\n"));
        assert!(r.is_ok() && previous == Some(Before::Missing { created_dirs: Vec::new() }), "{r:?}");
        assert_eq!(std::fs::read_to_string(&fresh).unwrap(), "x = 1\n");
        assert!(restore(&LocalHost, &db, Some(&key), &target(fresh_path, ""), &Before::Missing { created_dirs: Vec::new() }).is_ok());
        // A new file in folders that don't exist yet: they're created, and a
        // rollback takes the file and those folders (only) away.
        let nested = dir.join("app.d/conf.d/new.conf");
        let nested_path = nested.to_str().unwrap();
        let (r, previous) = push_file(&LocalHost, &db, Some(&key), &target(nested_path, "y = 2\n"));
        let made = vec![dir.join("app.d").display().to_string(), dir.join("app.d/conf.d").display().to_string()];
        assert!(r.is_ok() && previous == Some(Before::Missing { created_dirs: made.clone() }), "{r:?} {previous:?}");
        assert_eq!(std::fs::read_to_string(&nested).unwrap(), "y = 2\n");
        assert!(restore(&LocalHost, &db, Some(&key), &target(nested_path, ""), &previous.unwrap()).is_ok());
        assert!(!dir.join("app.d").exists() && dir.exists(), "the folders it made are gone, the rest stays");
        assert!(!fresh.exists());
        // Trailing whitespace only: nothing to do, nothing written.
        std::fs::write(&file, "workers = 8   \n").unwrap();
        let r = push_baseline(&LocalHost, &db, Some(&key), &target(path, "workers = 8\n"));
        assert_eq!(r, Ok(StepOutcome::Skipped("already matches the baseline".into())));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "workers = 8   \n");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unreadable_file_fails_the_step() {
        let db = Mutex::new(VaultDb::open_in_memory().unwrap());
        let r = push_baseline(&LocalHost, &db, None, &target("/nonexistent/crow/app.conf", "x\n"));
        assert!(r.unwrap_err().starts_with("couldn't read"));
    }
}
