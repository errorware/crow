//! Rollouts (ERR-142): one command or one config file across a group, in
//! stages: a canary first (the run pauses after it), then batches that run
//! side by side. Configs go through their format's checks on each server,
//! and a finished config rollout can be rolled back.

use std::time::Duration;

use crate::host::{Host, HostError};
use crate::views::fleet::run::{StepOutcome, StepResult};

/// A command may take a while (a package install, a migration).
pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffKind {
    Same,
    Added,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub text: String,
}

/// Lines longer than this are compared as-is but the table is capped:
/// config files are small, and a huge one is shown as "replaced".
const MAX_DIFF_CELLS: usize = 4_000_000;

/// A line diff by longest common subsequence: an inserted line is one
/// added line, not everything after it changed.
pub fn line_diff(old: &str, new: &str) -> Vec<DiffLine> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let line = |kind, t: &str| DiffLine { kind, text: t.to_string() };
    if a.len().saturating_mul(b.len()) > MAX_DIFF_CELLS {
        return a.iter().map(|t| line(DiffKind::Removed, t)).chain(b.iter().map(|t| line(DiffKind::Added, t))).collect();
    }
    // lcs[i][j]: the common length of a[i..] and b[j..].
    let (n, m) = (a.len(), b.len());
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            out.push(line(DiffKind::Same, a[i]));
            i += 1;
            j += 1;
        } else if i < n && (j == m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            // What goes, before what replaces it.
            out.push(line(DiffKind::Removed, a[i]));
            i += 1;
        } else {
            out.push(line(DiffKind::Added, b[j]));
            j += 1;
        }
    }
    out
}

/// (added, removed) lines.
pub fn diff_stats(diff: &[DiffLine]) -> (usize, usize) {
    (diff.iter().filter(|d| d.kind == DiffKind::Added).count(), diff.iter().filter(|d| d.kind == DiffKind::Removed).count())
}

/// The changed lines with `context` unchanged lines around them; `None`
/// stands for skipped unchanged lines.
pub fn hunks(diff: &[DiffLine], context: usize) -> Vec<Option<&DiffLine>> {
    let near_change = |i: usize| {
        let lo = i.saturating_sub(context);
        let hi = (i + context).min(diff.len().saturating_sub(1));
        (lo..=hi).any(|k| diff[k].kind != DiffKind::Same)
    };
    let mut out = Vec::new();
    let mut skipped = false;
    for (i, d) in diff.iter().enumerate() {
        if near_change(i) {
            out.push(Some(d));
            skipped = false;
        } else if !skipped {
            out.push(None);
            skipped = true;
        }
    }
    out
}

/// Each server's stage: the canary alone (stage 0) when there is one, then
/// batches of `batch` (at least 1).
pub fn stages(servers: usize, canary: bool, batch: usize) -> Vec<usize> {
    let batch = batch.max(1);
    (0..servers)
        .map(|i| match (canary, i) {
            (true, 0) => 0,
            (true, i) => 1 + (i - 1) / batch,
            (false, i) => i / batch,
        })
        .collect()
}

fn last_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

const EXIT_MARK: &str = "@@crow-exit ";

/// Runs `script` with `sh` (as root through sudo when `as_root`). The step
/// shows the last lines of output; a non-zero exit fails it with them.
/// The script's exit code is printed after its output, so a failure keeps
/// what it said.
pub fn run_command(host: &dyn Host, script: &str, as_root: bool) -> StepResult {
    let wrapped = format!("PATH=\"$PATH:/usr/sbin:/sbin\"; (\n{script}\n) 2>&1; echo \"{EXIT_MARK}$?\"");
    let argv = ["sh", "-c", wrapped.as_str()];
    let out = if as_root { host.exec_privileged(&argv, &[], COMMAND_TIMEOUT) } else { host.exec(&argv, COMMAND_TIMEOUT) };
    let stdout = match out {
        Ok(o) => o.stdout,
        Err(HostError::Failed { status, stderr }) => return Err(format!("couldn't run it (exit {status}): {}", last_lines(&stderr, 3))),
        Err(e) => return Err(e.to_string()),
    };
    let (output, code) = match stdout.rfind(EXIT_MARK) {
        Some(at) => (&stdout[..at], stdout[at + EXIT_MARK.len()..].trim().parse::<i32>().unwrap_or(-1)),
        None => (stdout.as_str(), -1),
    };
    let tail = last_lines(output, 5);
    match code {
        0 => Ok(StepOutcome::Done(if tail.is_empty() { "done (no output)".into() } else { tail })),
        -1 => Err(format!("ended without an exit code: {tail}")),
        n => Err(format!("exit {n}: {tail}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{diff_stats, hunks, line_diff, run_command, stages, DiffKind};
    use crate::host::LocalHost;
    use crate::views::fleet::run::StepOutcome;

    #[test]
    fn an_inserted_line_is_one_line() {
        let d = line_diff("a\nb\nc\nd\n", "a\nb\nNEW\nc\nd\n");
        assert_eq!(diff_stats(&d), (1, 0));
        assert_eq!(d.iter().find(|l| l.kind == DiffKind::Added).unwrap().text, "NEW");
        let d = line_diff("port 22\nroot yes\n", "port 22\nroot no\n");
        assert_eq!(diff_stats(&d), (1, 1));
        assert_eq!((d[1].kind, d[2].kind), (DiffKind::Removed, DiffKind::Added), "removed first");
        assert_eq!(diff_stats(&line_diff("x\n", "x\n")), (0, 0));
        assert_eq!(diff_stats(&line_diff("", "a\nb\n")), (2, 0));
    }

    #[test]
    fn hunks_keep_context_and_mark_skips() {
        let old: String = (1..=20).map(|i| format!("l{i}\n")).collect();
        let new = old.replace("l10\n", "L10\n");
        let d = line_diff(&old, &new);
        let h = hunks(&d, 2);
        assert_eq!(h.iter().filter(|x| x.is_none()).count(), 2, "skips before and after");
        assert_eq!(h.iter().flatten().count(), 6, "2 context + removed + added + 2 context");
    }

    #[test]
    fn stages_put_the_canary_alone_then_batches() {
        assert_eq!(stages(6, true, 2), [0, 1, 1, 2, 2, 3]);
        assert_eq!(stages(4, false, 2), [0, 0, 1, 1]);
        assert_eq!(stages(3, true, 0), [0, 1, 2], "a batch of 0 means 1");
        assert!(stages(0, true, 3).is_empty());
    }

    #[test]
    fn commands_report_their_output_and_failures() {
        assert_eq!(run_command(&LocalHost, "echo one\necho two", false), Ok(StepOutcome::Done("one\ntwo".into())));
        let err = run_command(&LocalHost, "echo checking; echo broken >&2; exit 3", false).unwrap_err();
        assert!(err.starts_with("exit 3") && err.contains("broken"), "{err}");
    }
}
