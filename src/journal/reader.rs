use crate::host::{host_for, Host, HostError, DEFAULT_TIMEOUT};
use crate::vault::ServerRecord;
use super::{parse_journal_json, JournalEntry, JournalQuery};

/// The unit filter value that selects kernel messages (dmesg).
pub const KERNEL_UNIT: &str = "kernel";

/// `journalctl` arguments for a query — built fresh from the filters so every
/// transport runs the same lookup.
pub fn journalctl_argv(query: &JournalQuery) -> Vec<String> {
    let mut argv: Vec<String> = ["journalctl", "-o", "json", "-n"].iter().map(|s| s.to_string()).collect();
    argv.push(query.limit.to_string());
    argv.push("--no-pager".into());
    // An exact window spans boots; otherwise this boot or the one before.
    if query.window.is_none() {
        argv.push("-b".into());
        argv.push(query.boot.offset().to_string());
    }
    if let Some(ref u) = query.unit {
        if u == KERNEL_UNIT {
            // Kernel messages have no systemd unit; they arrive by the kernel transport.
            argv.push("_TRANSPORT=kernel".into());
        } else if !u.is_empty() && u != "ALL" && u != "ALL UNITS" {
            argv.extend(["-u".to_string(), u.clone()]);
        }
    }
    if let Some(p) = query.priority {
        argv.extend(["-p".to_string(), (p as u8).to_string()]);
    }
    if let Some(pid) = query.pid {
        argv.push(format!("_PID={}", pid));
    }
    if let Some((from, to)) = query.window {
        argv.extend(["--since".to_string(), format!("@{from}"), "--until".to_string(), format!("@{to}")]);
    } else if let Some(since) = query.time_range.since_str() {
        argv.extend(["--since".to_string(), since.to_string()]);
    }
    if let Some(ref pattern) = query.grep {
        if !pattern.trim().is_empty() {
            argv.extend(["--grep".to_string(), pattern.clone()]);
        }
    }
    argv
}

/// Runs a journal lookup on `host`. Returns `None` only on a hard failure
/// (journalctl missing / host unreachable) — a query that legitimately matches
/// nothing still returns `Some(vec![])`. Note `journalctl --grep` exits 1 when
/// nothing matches; that is an empty result, not a failure.
pub fn read_journal(host: &dyn Host, query: &JournalQuery) -> Option<Vec<JournalEntry>> {
    let argv = journalctl_argv(query);
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let stdout = match host.exec(&argv, DEFAULT_TIMEOUT) {
        Ok(out) => out.stdout,
        Err(HostError::Failed { status: 1, .. }) if query.grep.is_some() => String::new(),
        Err(_) => return None,
    };
    Some(stdout.lines().map(str::trim).filter(|l| !l.is_empty()).filter_map(parse_journal_json).collect())
}

/// Reads the journal for a server over its transport. No matches, a failed
/// lookup and an unreachable server all return an empty list — never
/// stand-in entries.
pub fn read_journal_for_server(server: &ServerRecord, query: &JournalQuery) -> Vec<JournalEntry> {
    read_journal(host_for(server).as_ref(), query).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::JournalPriority;

    #[test]
    fn kernel_is_a_transport_not_a_unit() {
        let q = JournalQuery { unit: Some(KERNEL_UNIT.into()), ..Default::default() };
        let argv = journalctl_argv(&q);
        assert!(argv.contains(&"_TRANSPORT=kernel".to_string()) && !argv.contains(&"-u".to_string()));
        let q = JournalQuery { unit: Some("nginx.service".into()), priority: Some(JournalPriority::Warning), ..Default::default() };
        let argv = journalctl_argv(&q);
        assert!(argv.windows(2).any(|w| w == ["-u", "nginx.service"]));
        assert!(argv.windows(2).any(|w| w == ["-p", "4"]), "warning and worse");
    }
}
