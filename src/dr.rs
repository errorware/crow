//! Disaster recovery plans (ERR-151): what matters on a server or group,
//! where its backups go, how fast it must come back (RTO) and how much
//! may be lost (RPO), the runbook, checks of all that against the
//! servers, and drills.

use serde::{Deserialize, Serialize};

use crate::host::{Host, DEFAULT_TIMEOUT};

pub const PLANS_FLAG: &str = "dr.plans";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Backup {
    /// A file or glob on the server; its newest match is the last backup.
    pub path: String,
    /// The systemd timer (or service) that makes it, if any.
    pub timer: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drill {
    pub started_at: i64,
    pub minutes: i64,
    pub met_rto: bool,
    pub notes: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plan {
    pub id: String,
    pub name: String,
    /// A server id, or "group:<name>".
    pub target: String,
    /// Paths and databases that matter, one per line.
    pub matters: String,
    pub backups: Vec<Backup>,
    pub rto_minutes: i64,
    pub rpo_minutes: i64,
    /// The runbook, as Markdown with commands.
    pub runbook: String,
    pub drills: Vec<Drill>,
    /// A drill in progress: when it started.
    pub drill_started: Option<i64>,
}

/// "path | timer" per line.
pub fn parse_backups(text: &str) -> Vec<Backup> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (path, timer) = l.split_once('|').unwrap_or((l, ""));
            Backup { path: path.trim().to_string(), timer: timer.trim().to_string() }
        })
        .collect()
}

pub fn format_backups(b: &[Backup]) -> String {
    b.iter().map(|b| if b.timer.is_empty() { b.path.clone() } else { format!("{} | {}", b.path, b.timer) }).collect::<Vec<_>>().join("\n")
}

/// One finding of a check against a server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub ok: bool,
    pub what: String,
}

fn ok(what: impl Into<String>) -> Finding {
    Finding { ok: true, what: what.into() }
}

fn bad(what: impl Into<String>) -> Finding {
    Finding { ok: false, what: what.into() }
}

/// The script that reads a plan's facts on a server: per backup, the
/// newest match's time; per timer, active or not and its last trigger;
/// per path that matters, whether it exists.
fn read_script() -> &'static str {
    r#"PATH="$PATH:/usr/sbin:/sbin"
mode=$1; shift
for a in "$@"; do
  case "$mode" in
    backup) n=$(ls -1td -- $a 2>/dev/null | head -1); if [ -n "$n" ]; then echo "$(stat -c %Y -- "$n") $n"; else echo "none"; fi ;;
    timer) echo "$(systemctl is-active -- "$a" 2>/dev/null) $(systemctl show -p LastTriggerUSec --value -- "$a" 2>/dev/null)" ;;
    path) if [ -e "$a" ]; then echo yes; else echo no; fi ;;
  esac
done"#
}

fn read_lines(host: &dyn Host, mode: &str, items: &[String]) -> Vec<String> {
    if items.is_empty() {
        return Vec::new();
    }
    let mut argv: Vec<&str> = vec!["sh", "-c", read_script(), "crow-dr", mode];
    argv.extend(items.iter().map(String::as_str));
    let out = host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT).or_else(|_| host.exec(&argv, DEFAULT_TIMEOUT)).map(|o| o.stdout).unwrap_or_default();
    out.lines().map(str::to_string).collect()
}

fn age(secs: i64) -> String {
    match secs {
        s if s < 3600 => format!("{} min", s / 60),
        s if s < 2 * 86_400 => format!("{} h", s / 3600),
        s => format!("{} days", s / 86_400),
    }
}

/// Checks a plan on one server; `baselines` counts the files pinned as
/// baselines for it, `snapshots` names the provider that can snapshot it.
pub fn check(host: &dyn Host, plan: &Plan, baselines: usize, snapshots: Option<&str>, now: i64) -> Vec<Finding> {
    let mut out = Vec::new();
    let paths: Vec<String> = plan.backups.iter().map(|b| b.path.clone()).filter(|p| !p.is_empty()).collect();
    for (b, line) in plan.backups.iter().filter(|b| !b.path.is_empty()).zip(read_lines(host, "backup", &paths)) {
        match line.split_once(' ').and_then(|(t, n)| t.parse::<i64>().ok().map(|t| (t, n.to_string()))) {
            Some((t, name)) => {
                let since = now - t;
                if plan.rpo_minutes > 0 && since > plan.rpo_minutes * 60 {
                    out.push(bad(format!("last backup {} is {} old: more than the RPO of {} min", name, age(since), plan.rpo_minutes)));
                } else {
                    out.push(ok(format!("last backup {} is {} old", name, age(since))));
                }
            }
            None => out.push(bad(format!("no backup found at {}", b.path))),
        }
    }
    let timers: Vec<String> = plan.backups.iter().map(|b| b.timer.clone()).filter(|t| !t.is_empty()).collect();
    for (t, line) in timers.iter().zip(read_lines(host, "timer", &timers)) {
        let (state, last) = line.split_once(' ').unwrap_or((line.as_str(), ""));
        let last = last.trim();
        if state == "active" {
            out.push(ok(format!("{t} is active{}", if last.is_empty() || last == "n/a" { String::new() } else { format!(", last ran {last}") })));
        } else {
            out.push(bad(format!("{t} isn't running ({}): backups won't be made", if state.is_empty() { "not found" } else { state })));
        }
    }
    let matters: Vec<String> = plan.matters.lines().map(str::trim).filter(|l| l.starts_with('/')).map(String::from).collect();
    for (p, line) in matters.iter().zip(read_lines(host, "path", &matters)) {
        if line.trim() == "yes" {
            out.push(ok(format!("{p} exists")));
        } else {
            out.push(bad(format!("{p} isn't there: the plan may be out of date")));
        }
    }
    out.push(if baselines > 0 { ok(format!("{baselines} config file{} pinned as baselines (rebuildable from Crow)", if baselines == 1 { "" } else { "s" })) } else { bad("no config file pinned as a baseline: pin the ones a rebuild needs") });
    if let Some(p) = snapshots {
        out.push(ok(format!("{p} can snapshot it (Danger Zone → SNAPSHOTS)")));
    }
    if plan.backups.is_empty() {
        out.push(bad("the plan names no backups"));
    }
    out
}

/// The plan as a document that's readable without Crow.
pub fn markdown(plan: &Plan, target: &str, findings: &[(String, Vec<Finding>)], now: &str) -> String {
    let mut md = format!("# Disaster recovery plan: {}\n\nFor {target}. Exported from Crow, {now}.\n\n", plan.name);
    md.push_str(&format!("- **RTO** (back within): {} min\n- **RPO** (data loss at most): {} min\n\n", plan.rto_minutes, plan.rpo_minutes));
    md.push_str("## What matters\n\n");
    for l in plan.matters.lines().map(str::trim).filter(|l| !l.is_empty()) {
        md.push_str(&format!("- `{l}`\n"));
    }
    md.push_str("\n## Backups\n\n");
    for b in &plan.backups {
        md.push_str(&format!("- `{}`{}\n", b.path, if b.timer.is_empty() { String::new() } else { format!(" (made by `{}`)", b.timer) }));
    }
    md.push_str("\n## Runbook\n\n");
    md.push_str(if plan.runbook.trim().is_empty() { "_No runbook written yet._" } else { plan.runbook.trim() });
    md.push_str("\n\n## Last check\n\n");
    if findings.is_empty() {
        md.push_str("_Not checked._\n");
    }
    for (server, f) in findings {
        md.push_str(&format!("### {server}\n\n"));
        for x in f {
            md.push_str(&format!("- {} {}\n", if x.ok { "✓" } else { "✗" }, x.what));
        }
        md.push('\n');
    }
    md.push_str("## Drills\n\n");
    if plan.drills.is_empty() {
        md.push_str("_None yet._\n");
    }
    for d in &plan.drills {
        use chrono::TimeZone;
        let when = chrono::Local.timestamp_opt(d.started_at, 0).single().map(|t| t.format("%Y-%m-%d").to_string()).unwrap_or_default();
        md.push_str(&format!("- {when}: {} min ({}){}\n", d.minutes, if d.met_rto { "met the RTO" } else { "missed the RTO" }, if d.notes.is_empty() { String::new() } else { format!(": {}", d.notes) }));
    }
    md
}

#[cfg(test)]
mod tests {
    use super::{check, format_backups, markdown, parse_backups, Drill, Plan};
    use crate::host::LocalHost;

    #[test]
    fn backups_parse_and_format() {
        let b = parse_backups("/var/backups/db/*.sql.gz | pg-dump.timer\n# comment\n/srv/files.tar\n");
        assert_eq!(b.len(), 2);
        assert_eq!((b[0].timer.as_str(), b[1].timer.as_str()), ("pg-dump.timer", ""));
        assert_eq!(parse_backups(&format_backups(&b)), b);
    }

    #[test]
    fn a_plan_is_checked_against_the_files_for_real() {
        let dir = std::env::temp_dir().join(format!("crow-dr-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("db-1.sql.gz"), "x").unwrap();
        let now = chrono::Utc::now().timestamp();
        let plan = Plan {
            name: "db".into(),
            matters: format!("{}\n/nonexistent/crow/data\n", dir.display()),
            backups: parse_backups(&format!("{}/*.sql.gz\n{}/*.tar", dir.display(), dir.display())),
            rpo_minutes: 60,
            rto_minutes: 30,
            ..Default::default()
        };
        let f = check(&LocalHost, &plan, 0, None, now);
        assert!(f[0].ok && f[0].what.contains("db-1.sql.gz"), "{f:?}");
        assert!(!f[1].ok && f[1].what.contains("no backup found"));
        assert!(f.iter().any(|x| x.ok && x.what.ends_with("exists")));
        assert!(f.iter().any(|x| !x.ok && x.what.contains("/nonexistent/crow/data")));
        // Two hours later the same backup is past the RPO.
        let later = check(&LocalHost, &plan, 2, None, now + 7200);
        assert!(!later[0].ok && later[0].what.contains("more than the RPO"));
        let md = markdown(&Plan { drills: vec![Drill { started_at: now, minutes: 25, met_rto: true, notes: "restored to a VM".into() }], ..plan }, "db-1", &[("db-1".into(), later)], "today");
        assert!(md.contains("**RTO**") && md.contains("25 min (met the RTO)") && md.contains("✗"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
