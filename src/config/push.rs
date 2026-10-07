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
}

/// Re-reads `path` on the host (it may have changed since Crow last looked),
/// skips it if it already means the same as the baseline, else checks the
/// baseline with the format's host validator, writes it atomically, and
/// records the revision and a change record.
pub fn push_baseline(host: &dyn Host, db: &Mutex<VaultDb>, key: Option<&MasterKey>, t: &PushTarget) -> StepResult {
    let current = host
        .read_file(t.path)
        .or_else(|_| host.exec_privileged(&["cat", "--", t.path], &[], DEFAULT_TIMEOUT).map(|o| o.stdout))
        .map_err(|e| format!("couldn't read {}: {e}", t.path))?;
    let format = format_for_path(t.path);
    let differences = match compare(format, t.baseline, &current) {
        Drift::Identical | Drift::Cosmetic => return Ok(StepOutcome::Skipped("already matches the baseline".into())),
        Drift::Differs(d) => d.len(),
    };
    if format == Some(plugins::StructuredFormat::Sudoers) {
        // visudo and the keep-sudo guard run in the install itself.
        super::sudoers::install_sudoers(host, t.path, t.baseline, t.login_user).map_err(|e| format!("not written: {e}"))?;
    } else {
        if let Some(f) = format {
            plugins::validate_on_host(host, f, t.baseline).map_err(|e| format!("not written: the host's validator refused the baseline: {e}"))?;
        }
        host.write_file_privileged(t.path, t.baseline).map_err(|e| format!("not written: {e}"))?;
    }

    // Written. Recording can still fail; say so without calling the push a failure.
    let recorded = db.lock().map_err(|_| "the vault is busy".to_string()).and_then(|db| {
        let rev = history::record(&db, key, t.server_id, t.path, t.baseline, t.author, "Pushed the baseline", SOURCE_CROW).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        db.insert_change_record(&ChangeRecord {
            id: format!("chg-{}", rev.id.trim_start_matches("cfgrev-")),
            server_id: t.server_id.into(),
            server_name: t.server_name.into(),
            action_kind: "config.write".into(),
            target: t.path.into(),
            before_state: format!("sha256:{}", history::sha256_hex(&current)),
            after_state: Some(format!("sha256:{}", rev.sha256)),
            blast_radius: Some("fleet run: push baseline".into()),
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
        PushTarget { server_id: "s1", server_name: "web-1", path, baseline, author: "me@box", login_user: "deploy" }
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
