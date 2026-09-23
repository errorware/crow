//! Lossless editing of the system crontab (/etc/crontab format:
//! `m h dom mon dow user command`).
//!
//! Jobs are parsed from the real file and remember which line they came from
//! (`id = "line-N"`). Rendering rewrites only job lines that changed, drops
//! deleted ones, places reordered jobs into the original job slots, and
//! appends new jobs; comments, blank lines, environment lines (SHELL=,
//! PATH=, MAILTO=) and `@reboot`-style entries are kept byte-for-byte.

use crate::views::config::cron_editor::CronJobDef;

const LINE_ID_PREFIX: &str = "line-";

/// Splits a job line into its 7 fields: 5 schedule fields, user, command.
fn split_job(line: &str) -> Option<[String; 7]> {
    let mut rest = line.trim_start();
    let mut fields: Vec<String> = Vec::with_capacity(7);
    for _ in 0..6 {
        let end = rest.find(char::is_whitespace)?;
        fields.push(rest[..end].to_string());
        rest = rest[end..].trim_start();
    }
    if rest.is_empty() {
        return None;
    }
    fields.push(rest.trim_end().to_string());
    let schedule_ok = fields[..5].iter().all(|f| f.chars().all(|c| c.is_ascii_alphanumeric() || "*/,-".contains(c)));
    // The first field must look like a minute spec, not a word from a comment.
    let first_ok = fields[0].chars().next().is_some_and(|c| c.is_ascii_digit() || c == '*');
    (schedule_ok && first_ok).then(|| fields.try_into().ok()).flatten()
}

fn is_env_line(line: &str) -> bool {
    let t = line.trim_start();
    match t.split_once('=') {
        Some((name, _)) => !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
        None => false,
    }
}

/// A commented-out job (`#17 * * * * root cmd` or `# DISABLED: ...`).
fn disabled_job(line: &str) -> Option<[String; 7]> {
    let body = line.trim_start().strip_prefix('#')?;
    let body = body.trim_start();
    let body = body.strip_prefix("DISABLED:").unwrap_or(body);
    split_job(body)
}

/// Parses the jobs in a system crontab. A plain comment directly above a job
/// becomes that job's description.
pub fn parse_crontab(text: &str) -> Vec<CronJobDef> {
    let mut jobs = Vec::new();
    let mut pending_comment: Option<String> = None;
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        let (fields, enabled) = if let Some(f) = (!trimmed.starts_with('#') && !is_env_line(line)).then(|| split_job(line)).flatten() {
            (f, true)
        } else if let Some(f) = disabled_job(line) {
            (f, false)
        } else {
            pending_comment = trimmed
                .strip_prefix('#')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty() && !c.starts_with("m h dom"));
            continue;
        };
        let [minute, hour, dom, month, dow, user, command] = fields;
        let mut job = CronJobDef::new(format!("{LINE_ID_PREFIX}{i}"), enabled, minute, hour, dom, month, dow, user, command, pending_comment.take());
        job.is_expanded = false;
        jobs.push(job);
    }
    jobs
}

fn format_job(job: &CronJobDef) -> String {
    let line = format!(
        "{} {} {} {} {}\t{}\t{}",
        job.minute, job.hour, job.day_of_month, job.month, job.day_of_week, job.user, job.command
    );
    if job.enabled { line } else { format!("#{line}") }
}

fn same_job(a: &CronJobDef, b: &CronJobDef) -> bool {
    a.enabled == b.enabled
        && a.minute == b.minute
        && a.hour == b.hour
        && a.day_of_month == b.day_of_month
        && a.month == b.month
        && a.day_of_week == b.day_of_week
        && a.user == b.user
        && a.command == b.command
}

/// Renders `jobs` back into `source` (the text they were parsed from).
pub fn render_crontab(source: &str, jobs: &[CronJobDef]) -> String {
    let original = parse_crontab(source);
    let line_of = |id: &str| id.strip_prefix(LINE_ID_PREFIX).and_then(|n| n.parse::<usize>().ok());
    // Slots of original jobs that still exist, in file order…
    let kept_slots: Vec<usize> = original.iter().filter(|o| jobs.iter().any(|j| j.id == o.id)).filter_map(|o| line_of(&o.id)).collect();
    // …filled with the surviving jobs in their current (possibly reordered) order.
    let existing: Vec<&CronJobDef> = jobs.iter().filter(|j| original.iter().any(|o| o.id == j.id)).collect();
    let slot_job: std::collections::HashMap<usize, &CronJobDef> = kept_slots.iter().copied().zip(existing).collect();
    let all_slots: std::collections::HashSet<usize> = original.iter().filter_map(|o| line_of(&o.id)).collect();

    let mut out = String::new();
    for (i, line) in source.split_inclusive('\n').enumerate() {
        let newline = if line.ends_with('\n') { "\n" } else { "" };
        match slot_job.get(&i) {
            Some(job) => {
                let unchanged = line_of(&job.id) == Some(i) && original.iter().any(|o| o.id == job.id && same_job(o, job));
                if unchanged {
                    out.push_str(line);
                } else {
                    out.push_str(&format_job(job));
                    out.push_str(newline);
                }
            }
            None if all_slots.contains(&i) => {} // deleted job
            None => out.push_str(line),
        }
    }
    let new_jobs: Vec<&CronJobDef> = jobs.iter().filter(|j| !original.iter().any(|o| o.id == j.id)).collect();
    if !new_jobs.is_empty() && !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    for job in new_jobs {
        if let Some(c) = &job.comment {
            out.push_str(&format!("# {c}\n"));
        }
        out.push_str(&format_job(job));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEBIAN: &str = "# /etc/crontab: system-wide crontab\nSHELL=/bin/sh\nPATH=/usr/local/sbin:/usr/local/bin:/sbin:/bin:/usr/sbin:/usr/bin\n\n# Example of job definition:\n# m h dom mon dow user\tcommand\n17 *\t* * *\troot\tcd / && run-parts --report /etc/cron.hourly\n25 6\t* * *\troot\ttest -x /usr/sbin/anacron || { cd / && run-parts --report /etc/cron.daily; }\n# weekly\n47 6\t* * 7\troot\ttest -x /usr/sbin/anacron || { cd / && run-parts --report /etc/cron.weekly; }\n@reboot root /usr/local/bin/warmup\n#52 6\t1 * *\troot\tmonthly-job\n";

    #[test]
    fn parses_real_crontab_jobs_and_skips_env_and_special_lines() {
        let jobs = parse_crontab(DEBIAN);
        assert_eq!(jobs.len(), 4);
        assert_eq!(jobs[0].id, "line-6");
        assert_eq!((jobs[0].minute.as_str(), jobs[0].hour.as_str(), jobs[0].user.as_str()), ("17", "*", "root"));
        assert_eq!(jobs[1].command, "test -x /usr/sbin/anacron || { cd / && run-parts --report /etc/cron.daily; }");
        assert_eq!(jobs[2].comment.as_deref(), Some("weekly"));
        assert!(!jobs[3].enabled);
        assert_eq!(jobs[3].command, "monthly-job");
    }

    #[test]
    fn unedited_render_is_byte_identical() {
        assert_eq!(render_crontab(DEBIAN, &parse_crontab(DEBIAN)), DEBIAN);
    }

    #[test]
    fn edit_changes_only_that_line() {
        let mut jobs = parse_crontab(DEBIAN);
        jobs[1].hour = "7".into();
        let out = render_crontab(DEBIAN, &jobs);
        let (a, b): (Vec<&str>, Vec<&str>) = (DEBIAN.lines().collect(), out.lines().collect());
        assert_eq!(a.len(), b.len());
        let changed: Vec<usize> = (0..a.len()).filter(|&i| a[i] != b[i]).collect();
        assert_eq!(changed, vec![7]);
        assert!(b[7].starts_with("25 7 * * *\troot\t"));
    }

    #[test]
    fn delete_reorder_disable_and_add() {
        let mut jobs = parse_crontab(DEBIAN);
        jobs.remove(0); // delete hourly
        jobs.swap(0, 1); // weekly before daily
        jobs[0].enabled = false;
        jobs.push(CronJobDef::new("cron_new", true, "0", "2", "*", "*", "*", "root", "/usr/local/bin/backup", Some("Nightly backup".into())));
        let out = render_crontab(DEBIAN, &jobs);
        assert!(!out.contains("cron.hourly"));
        assert!(out.contains("SHELL=/bin/sh\n") && out.contains("@reboot root /usr/local/bin/warmup\n"));
        let weekly = out.find("cron.weekly").unwrap();
        let daily = out.find("cron.daily").unwrap();
        assert!(weekly < daily, "reordered into the original slots");
        assert!(out.contains("#47 6 * * 7\troot\t"));
        assert!(out.ends_with("# Nightly backup\n0 2 * * *\troot\t/usr/local/bin/backup\n"));
        // And it parses back to the same jobs.
        let reparsed = parse_crontab(&out);
        assert_eq!(reparsed.len(), 4);
    }

    /// Round-trips this machine's real crontab when it has one.
    #[test]
    fn real_system_crontab_round_trips() {
        let Ok(text) = std::fs::read_to_string("/etc/crontab") else { return };
        let jobs = parse_crontab(&text);
        assert_eq!(render_crontab(&text, &jobs), text);
        if let Some(first) = jobs.first() {
            let mut edited = jobs.clone();
            edited[0].minute = if first.minute == "0" { "1".into() } else { "0".into() };
            let out = render_crontab(&text, &edited);
            let changed = text.lines().zip(out.lines()).filter(|(a, b)| a != b).count();
            assert_eq!(changed, 1, "exactly one line changes");
        }
    }
}
