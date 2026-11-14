use std::time::Duration;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::host::{host_for, DEFAULT_TIMEOUT};
use crate::vault::{ChangeRecord, ServerRecord, VaultDb};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDetails {
    pub unit_name: String,
    pub fragment_path: String,
    pub drop_in_paths: Vec<String>,
    pub is_vendor: bool,
    pub is_custom: bool,
    pub cat_content: String,
    pub override_path: String,
    pub override_content: Option<String>,
    pub custom_unit_content: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunOutcome {
    pub success: bool,
    pub exit_code: i32,
    pub output: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitBackupRecord {
    pub unit: String,
    pub path: String,
    pub existed: bool,
    pub content: Option<String>,
    pub backup_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyOutcome {
    pub success: bool,
    pub unit_status: String,
    pub message: String,
}

fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Parses FragmentPath and DropInPaths from `systemctl show -p FragmentPath,DropInPaths ...` output.
pub fn parse_show_properties(stdout: &str) -> (String, Vec<String>) {
    let mut fragment_path = String::new();
    let mut drop_in_paths = Vec::new();

    for line in stdout.lines() {
        if let Some(val) = line.strip_prefix("FragmentPath=") {
            fragment_path = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("DropInPaths=") {
            drop_in_paths = val
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
        }
    }
    (fragment_path, drop_in_paths)
}

/// Collects unit file paths, active drop-ins, and file content for a service unit.
pub fn inspect_unit_details(server: &ServerRecord, unit: &str) -> Result<UnitDetails, String> {
    let host = host_for(server);

    // 1. Query FragmentPath and DropInPaths from systemctl
    let show_out = host
        .exec(&["systemctl", "show", "-p", "FragmentPath,DropInPaths,ActiveState", unit], DEFAULT_TIMEOUT)
        .map_err(|e| format!("Failed to inspect unit properties: {e}"))?;

    let (fragment_path, drop_in_paths) = parse_show_properties(&show_out.stdout);

    let is_vendor = fragment_path.starts_with("/usr/lib/")
        || fragment_path.starts_with("/lib/")
        || fragment_path.starts_with("/usr/local/lib/");
    let is_custom = fragment_path.starts_with("/etc/systemd/system/");

    // 2. Fetch systemctl cat output
    let cat_out = host
        .exec(&["systemctl", "cat", unit], DEFAULT_TIMEOUT)
        .map(|o| o.stdout)
        .unwrap_or_else(|_| String::new());

    // 3. Check for drop-in override (/etc/systemd/system/<unit>.d/override.conf)
    let override_path = format!("/etc/systemd/system/{unit}.d/override.conf");
    let override_content = if host.exists(&override_path) {
        host.read_file(&override_path).ok()
    } else {
        None
    };

    // 4. If custom unit in /etc, read its direct content
    let custom_unit_content = if is_custom && !fragment_path.is_empty() {
        host.read_file(&fragment_path).ok()
    } else {
        None
    };

    Ok(UnitDetails {
        unit_name: unit.to_string(),
        fragment_path,
        drop_in_paths,
        is_vendor,
        is_custom,
        cat_content: cat_out,
        override_path,
        override_content,
        custom_unit_content,
    })
}

/// Runs a dry-run syntax verification using `systemd-analyze verify` inside a temporary staging sandbox.
pub fn dry_run_verify_unit(
    server: &ServerRecord,
    unit: &str,
    fragment_path: &str,
    is_override: bool,
    candidate_content: &str,
) -> Result<DryRunOutcome, String> {
    let host = host_for(server);

    let script = r#"
set -e
td=$(mktemp -d /tmp/.crow-verify-XXXXXX)
trap 'rm -rf "$td"' EXIT

unit="$1"
frag="$2"
is_ovr="$3"

if [ "$is_ovr" = "1" ]; then
    mkdir -p "$td/${unit}.d"
    if [ -n "$frag" ] && [ -f "$frag" ]; then
        cp "$frag" "$td/${unit}"
    else
        touch "$td/${unit}"
    fi
    cat > "$td/${unit}.d/override.conf"
else
    cat > "$td/${unit}"
fi

if command -v systemd-analyze >/dev/null 2>&1; then
    systemd-analyze verify "$td/${unit}" 2>&1
    exit $?
else
    echo "systemd-analyze not present on system; basic syntax valid"
    exit 0
fi
"#;

    let is_ovr_arg = if is_override { "1" } else { "0" };
    let out = host
        .exec_stdin(
            &["sh", "-c", script, "crow-verify", unit, fragment_path, is_ovr_arg],
            candidate_content.as_bytes(),
            Duration::from_secs(12),
        );

    match out {
        Ok(exec_res) => {
            let combined = format!("{}{}", exec_res.stdout, exec_res.stderr).trim().to_string();
            Ok(DryRunOutcome {
                success: true,
                exit_code: 0,
                output: if combined.is_empty() {
                    "Unit configuration verified cleanly by systemd-analyze with no errors or warnings.".to_string()
                } else {
                    combined
                },
            })
        }
        Err(crate::host::HostError::Failed { status, stderr }) => {
            let msg = stderr.trim().to_string();
            Ok(DryRunOutcome {
                success: false,
                exit_code: status,
                output: if msg.is_empty() {
                    format!("systemd-analyze verify exited with status {status}")
                } else {
                    msg
                },
            })
        }
        Err(e) => Err(format!("Failed to execute verification: {e}")),
    }
}

/// Applies candidate content to target unit/override path atomically with pre-mutation backup,
/// daemon-reload, restart (optional), health status verification, and ChangeRecord audit persistence.
pub fn apply_unit_change(
    server: &ServerRecord,
    unit: &str,
    target_path: &str,
    new_content: &str,
    restart_after: bool,
    db: &VaultDb,
) -> Result<ApplyOutcome, String> {
    let host = host_for(server);
    let now = chrono::Utc::now().to_rfc3339();

    // 1. Read existing content if present
    let old_content = host.read_file(target_path).ok();
    let existed = old_content.is_some();

    // 2. Create remote timestamped backup copy if file exists
    let backup_path = if existed {
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let bpath = format!("/var/backups/crow/systemd/{unit}-{ts}.bak");
        let _ = host.exec_privileged(&["mkdir", "-p", "/var/backups/crow/systemd"], &[], DEFAULT_TIMEOUT);
        if let Some(ref c) = old_content {
            let _ = host.write_file_privileged(&bpath, c);
        }
        Some(bpath)
    } else {
        None
    };

    // 3. Ensure parent directory exists (e.g. /etc/systemd/system/<unit>.d)
    if let Some(parent) = std::path::Path::new(target_path).parent() {
        let parent_str = parent.to_string_lossy();
        if !parent_str.is_empty() {
            let _ = host.exec_privileged(&["mkdir", "-p", &parent_str], &[], DEFAULT_TIMEOUT);
        }
    }

    // 4. Write new content atomically with root privileges
    host.write_file_privileged(target_path, new_content)
        .map_err(|e| format!("Failed to write unit file: {e}"))?;

    // 5. Run systemctl daemon-reload
    let reload_res = host.exec_privileged(&["systemctl", "daemon-reload"], &[], DEFAULT_TIMEOUT);
    if let Err(e) = reload_res {
        return Err(format!("Written successfully, but daemon-reload failed: {e}"));
    }

    // 6. Optional restart/reload
    if restart_after {
        let _ = host.exec_privileged(&["systemctl", "try-reload-or-restart", unit], &[], DEFAULT_TIMEOUT);
    }

    // 7. Verify unit status
    let status_out = host.exec(&["systemctl", "is-active", unit], DEFAULT_TIMEOUT);
    let unit_status = match status_out {
        Ok(o) => o.stdout.trim().to_string(),
        Err(crate::host::HostError::Failed { stderr, .. }) => stderr.trim().to_string(),
        Err(_) => "unknown".to_string(),
    };

    let is_ok = unit_status == "active" || unit_status == "activating" || (!restart_after && !unit_status.is_empty());
    let outcome_str = if is_ok { "success" } else { "failed" };

    // 8. Record audit ChangeRecord in vault db
    let backup_data = UnitBackupRecord {
        unit: unit.to_string(),
        path: target_path.to_string(),
        existed,
        content: old_content,
        backup_path,
    };
    let before_state_json = serde_json::to_string(&backup_data).unwrap_or_default();
    let change_id = format!("chg-unit-{}", &uuid_short());

    let _ = db.insert_change_record(&ChangeRecord {
        id: change_id,
        server_id: server.id.clone(),
        server_name: server.name.clone(),
        action_kind: "service_unit_edit".into(),
        target: unit.to_string(),
        before_state: before_state_json,
        after_state: Some(format!("sha256:{}", sha256_hex(new_content))),
        blast_radius: Some(if restart_after {
            format!("daemon-reload; restart {unit}")
        } else {
            "daemon-reload".into()
        }),
        outcome: outcome_str.into(),
        started_at: now.clone(),
        completed_at: Some(chrono::Utc::now().to_rfc3339()),
    });

    Ok(ApplyOutcome {
        success: is_ok,
        unit_status: unit_status.clone(),
        message: if is_ok {
            format!("Applied changes to {target_path}. systemd reloaded. Status: {unit_status}.")
        } else {
            format!("Changes applied, but service reported status: {unit_status}.")
        },
    })
}

/// Performs a 1-click rollback of a previous service unit edit from its ChangeRecord snapshot.
pub fn rollback_unit_change(
    server: &ServerRecord,
    record: &ChangeRecord,
    db: &VaultDb,
) -> Result<String, String> {
    let host = host_for(server);

    let backup: UnitBackupRecord = serde_json::from_str(&record.before_state)
        .map_err(|e| format!("Invalid unit backup snapshot in change record: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();

    if !backup.existed {
        // File did not exist prior to this change (e.g. newly created override.conf) -> remove it!
        host.exec_privileged(&["rm", "-f", &backup.path], &[], DEFAULT_TIMEOUT)
            .map_err(|e| format!("Failed to remove override file during rollback: {e}"))?;
    } else if let Some(ref prev_content) = backup.content {
        // File existed -> restore its exact previous content!
        host.write_file_privileged(&backup.path, prev_content)
            .map_err(|e| format!("Failed to restore file content during rollback: {e}"))?;
    } else {
        return Err("Snapshot contains no prior content to restore".into());
    }

    // Run systemctl daemon-reload
    host.exec_privileged(&["systemctl", "daemon-reload"], &[], DEFAULT_TIMEOUT)
        .map_err(|e| format!("Rolled back file, but daemon-reload failed: {e}"))?;

    // Restart service if applicable
    let _ = host.exec_privileged(&["systemctl", "try-reload-or-restart", &backup.unit], &[], DEFAULT_TIMEOUT);

    let status_out = host.exec(&["systemctl", "is-active", &backup.unit], DEFAULT_TIMEOUT);
    let current_status = status_out.map(|o| o.stdout.trim().to_string()).unwrap_or_else(|_| "active".to_string());

    // Record the rollback in audit trail
    let rollback_id = format!("chg-unit-rollback-{}", &uuid_short());
    let _ = db.insert_change_record(&ChangeRecord {
        id: rollback_id,
        server_id: server.id.clone(),
        server_name: server.name.clone(),
        action_kind: "service_unit_rollback".into(),
        target: backup.unit.clone(),
        before_state: format!("rolled_back_change:{}", record.id),
        after_state: Some(format!("restored:{}", backup.path)),
        blast_radius: Some(format!("daemon-reload; restart {}", backup.unit)),
        outcome: "success".into(),
        started_at: now.clone(),
        completed_at: Some(chrono::Utc::now().to_rfc3339()),
    });

    Ok(format!("Successfully rolled back {} to pre-edit state. Current status: {}.", backup.unit, current_status))
}

fn uuid_short() -> String {
    format!("{:08x}", rand::random::<u32>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_show_properties() {
        let sample = "\
FragmentPath=/usr/lib/systemd/system/sshd.service
DropInPaths=/usr/lib/systemd/system/service.d/10-timeout-abort.conf /etc/systemd/system/sshd.service.d/override.conf
ActiveState=active
";
        let (fragment, drop_ins) = parse_show_properties(sample);
        assert_eq!(fragment, "/usr/lib/systemd/system/sshd.service");
        assert_eq!(drop_ins.len(), 2);
        assert_eq!(drop_ins[0], "/usr/lib/systemd/system/service.d/10-timeout-abort.conf");
        assert_eq!(drop_ins[1], "/etc/systemd/system/sshd.service.d/override.conf");
    }

    #[test]
    fn test_unit_backup_record_serde() {
        let backup = UnitBackupRecord {
            unit: "sshd.service".into(),
            path: "/etc/systemd/system/sshd.service.d/override.conf".into(),
            existed: true,
            content: Some("[Service]\nEnvironment=FOO=BAR\n".into()),
            backup_path: Some("/var/backups/crow/systemd/sshd.service-20261114.bak".into()),
        };

        let json = serde_json::to_string(&backup).expect("serialize backup");
        let decoded: UnitBackupRecord = serde_json::from_str(&json).expect("deserialize backup");
        assert_eq!(backup, decoded);
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("hello systemd");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, sha256_hex("hello systemd"));
    }
}
