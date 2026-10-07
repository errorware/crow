//! fstab on the Config screen (ERR-22): a bad fstab can stop a server from
//! booting, so a save is checked against the file it replaces first.
//!
//! `findmnt --verify` is the check, but it reports errors on working files
//! too (Fedora and Bazzite's /home → /var/home trips its "wrong order"), so
//! only errors the edit introduces block a save.

use std::collections::BTreeSet;

use crow_config_schemas::{fstab_mounts, FstabMount};

use crate::host::{Host, DEFAULT_TIMEOUT};

/// Mount points a server can't boot without.
const ESSENTIAL: [&str; 5] = ["/", "/boot", "/boot/efi", "/usr", "/var"];

/// Essential mount points `old` has and `new` doesn't.
pub fn dropped_essentials(old: &str, new: &str) -> Vec<String> {
    let has = |mounts: &[FstabMount], mp: &str| mounts.iter().any(|m| m.mountpoint == mp);
    let (old, new) = (fstab_mounts(old), fstab_mounts(new));
    ESSENTIAL.iter().filter(|mp| has(&old, mp) && !has(&new, mp)).map(|mp| mp.to_string()).collect()
}

/// The `[E]` lines of `findmnt --verify` output, as "target: message".
fn verify_errors(output: &str) -> BTreeSet<String> {
    let mut target = String::new();
    let mut errors = BTreeSet::new();
    for line in output.lines() {
        let t = line.trim();
        if let Some(msg) = t.strip_prefix("[E]") {
            errors.insert(format!("{target}: {}", msg.trim()));
        } else if !line.starts_with(' ') && !t.is_empty() && !t.contains("parse error") {
            target = t.to_string();
        }
    }
    errors
}

/// Runs `findmnt --verify` on `content` on the host, from a temp file.
/// `None` when findmnt isn't there.
fn verify_on_host(host: &dyn Host, content: &str) -> Option<BTreeSet<String>> {
    host.exec(&["sh", "-c", "command -v findmnt"], DEFAULT_TIMEOUT).ok()?;
    let tmp = host.exec(&["mktemp", "/tmp/crow-fstab.XXXXXX"], DEFAULT_TIMEOUT).ok()?.stdout.trim().to_string();
    let staged = host.exec_stdin(&["sh", "-c", "cat > \"$1\"", "crow-fstab", &tmp], content.as_bytes(), DEFAULT_TIMEOUT);
    // findmnt exits non-zero when it finds anything; its report is what
    // counts, so it's always collected as output.
    let report = staged.ok().and_then(|_| host.exec(&["sh", "-c", "findmnt --verify --tab-file \"$1\" 2>&1; true", "crow-fstab", &tmp], DEFAULT_TIMEOUT).ok()).map(|o| o.stdout);
    let _ = host.exec(&["rm", "-f", "--", &tmp], DEFAULT_TIMEOUT);
    report.map(|r| verify_errors(&r))
}

/// Checks `new` before it replaces `old` on the host: refuses dropping an
/// essential mount, and errors `findmnt --verify` finds in `new` that it
/// doesn't find in `old`.
pub fn check(host: &dyn Host, old: &str, new: &str) -> Result<(), String> {
    let dropped = dropped_essentials(old, new);
    if !dropped.is_empty() {
        return Err(format!("this removes the entry for {}, which the server needs to boot", dropped.join(", ")));
    }
    if let (Some(before), Some(after)) = (verify_on_host(host, old), verify_on_host(host, new)) {
        // A missing mount folder on a nofail/noauto mount can't stop a boot
        // (and systemd creates it), so it doesn't block.
        let mounts = fstab_mounts(new);
        let optional = |target: &str| mounts.iter().any(|m| m.mountpoint == target && (m.has_option("nofail") || m.has_option("noauto")));
        let introduced: Vec<&String> = after
            .difference(&before)
            .filter(|e| !(e.contains("required target: No such file or directory") && e.split(": ").next().is_some_and(optional)))
            .collect();
        if !introduced.is_empty() {
            return Err(format!("findmnt --verify finds new errors: {}", introduced.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; ")));
        }
    }
    Ok(())
}

/// After a write, systemd regenerates its mount units from fstab.
pub fn reload_systemd(host: &dyn Host) {
    let _ = host.exec_privileged(&["sh", "-c", "command -v systemctl >/dev/null 2>&1 && systemctl daemon-reload || true"], &[], DEFAULT_TIMEOUT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn essentials_can_be_edited_but_not_dropped() {
        let old = "UUID=a / ext4 defaults 0 1\nUUID=b /boot ext4 defaults 0 2\n/dev/sdb1 /data xfs defaults 0 2\n";
        assert!(dropped_essentials(old, "UUID=a / ext4 defaults 0 1\nUUID=b /boot ext4 defaults 0 2\n").is_empty(), "/data isn't essential");
        assert!(dropped_essentials(old, "UUID=c / xfs defaults 0 1\nUUID=b /boot ext4 noatime 0 2\n").is_empty(), "changed, not dropped");
        assert_eq!(dropped_essentials(old, "UUID=b /boot ext4 defaults 0 2\n"), vec!["/"]);
        assert_eq!(dropped_essentials(old, "#UUID=a / ext4 defaults 0 1\nUUID=b /boot ext4 defaults 0 2\n"), vec!["/"], "commented out is dropped");
    }

    #[test]
    fn reads_findmnt_errors_by_target() {
        // As findmnt 2.41 prints it.
        let out = "\n0 parse errors, 2 errors, 3 warnings\n/mnt/nowhere\n   [E] unreachable on boot required target: No such file or directory\n   [E] unreachable on boot required source: UUID=dead\n/\n   [W] ext9 seems unsupported by the current kernel\n";
        let errors: Vec<String> = verify_errors(out).into_iter().collect();
        assert_eq!(errors, vec![
            "/mnt/nowhere: unreachable on boot required source: UUID=dead".to_string(),
            "/mnt/nowhere: unreachable on boot required target: No such file or directory".to_string(),
        ]);
    }
}

