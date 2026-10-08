//! Patching (ERR-141): applying the updates Crow reads, inside each
//! group's maintenance window, and rebooting in an order that keeps the
//! fleet reachable.
//!
//! Only the packages shown in the plan are upgraded (`--only-upgrade` and
//! friends with their names), so what runs is what was approved.

use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

use crate::host::Host;
use crate::views::overview::updates::{parse_updates, UpdatesReport, UPDATES_PROBE};
use crate::vault::ServerRecord;

/// Upgrades can take a while (a kernel, initramfs, many packages).
pub const APPLY_TIMEOUT: Duration = Duration::from_secs(1800);
/// How long a fleet scan's reading of a server stays fresh.
pub const SCAN_EVERY_SECS: i64 = 6 * 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Security,
    All,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Scope::Security => "security updates",
            Scope::All => "all updates",
        }
    }
}

/// What a fleet scan read from one server.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Summary {
    pub checked_at: i64,
    /// "apt", "dnf", "apk", or empty when none was found.
    pub manager: String,
    pub security: Vec<String>,
    pub other: Vec<String>,
    pub reboot_required: bool,
    /// "6.8.0-40 → 6.8.0-45" when a new kernel is pending.
    pub kernel: Option<String>,
    pub error: Option<String>,
}

impl Summary {
    pub fn from_report(r: &UpdatesReport, now: i64) -> Self {
        let mut security = Vec::new();
        let mut other = Vec::new();
        for u in &r.updates {
            if u.security {
                security.push(u.name.clone());
            } else {
                other.push(u.name.clone());
            }
        }
        // An index never downloaded: "nothing pending" would be a guess.
        let error = r.index_missing.then(|| format!("{}'s package index was never downloaded, so pending updates are unknown: run `{} update` there", r.manager, r.manager));
        Summary { checked_at: now, manager: r.manager.clone(), security, other, reboot_required: r.reboot_required, kernel: r.kernel_update().map(|(k, _)| k), error }
    }

    pub fn failed(error: String, now: i64) -> Self {
        Summary { checked_at: now, error: Some(error), ..Default::default() }
    }

    pub fn total(&self) -> usize {
        self.security.len() + self.other.len()
    }

    /// The packages a run with `scope` upgrades.
    pub fn packages(&self, scope: Scope) -> Vec<String> {
        match scope {
            Scope::Security => self.security.clone(),
            Scope::All => self.security.iter().chain(self.other.iter()).cloned().collect(),
        }
    }
}

/// A name a package manager could take as an option, or with shell
/// metacharacters, isn't a package name.
pub fn valid_package(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('-') && name.len() <= 200 && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._+:@~-".contains(&b))
}

/// The command that upgrades exactly `packages`, run as root. Package
/// names are passed as arguments, never pasted into the script.
pub fn apply_argv(manager: &str, packages: &[String]) -> Result<Vec<String>, String> {
    if packages.is_empty() {
        return Err("nothing to upgrade".into());
    }
    if let Some(bad) = packages.iter().find(|p| !valid_package(p)) {
        return Err(format!("{bad:?} isn't a package name"));
    }
    let script = match manager {
        // Refresh the lists first: the plan names packages, apt picks the
        // current candidate. confold keeps local config files.
        "apt" => "export DEBIAN_FRONTEND=noninteractive NEEDRESTART_MODE=l; apt-get update -q >/dev/null && apt-get install -y -q --only-upgrade -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold -- \"$@\"",
        "dnf" => "dnf -y -q upgrade -- \"$@\"",
        "apk" => "apk update -q && apk add -q --upgrade -- \"$@\"",
        "" => return Err("no package manager Crow knows (apt, dnf, apk)".into()),
        other => return Err(format!("{other} isn't a package manager Crow can patch with")),
    };
    let mut argv: Vec<String> = vec!["sh".into(), "-c".into(), format!("PATH=\"$PATH:/usr/sbin:/sbin\"; {script} 2>&1"), "crow-patch".into()];
    argv.extend(packages.iter().cloned());
    Ok(argv)
}

/// Reads the server's pending updates (as root when it can, to see every
/// process; otherwise as the user).
pub fn read_report(host: &dyn Host) -> Result<UpdatesReport, String> {
    let argv = ["sh", "-c", UPDATES_PROBE];
    host.exec_privileged(&argv, &[], crate::host::DEFAULT_TIMEOUT)
        .or_else(|_| host.exec(&argv, crate::host::DEFAULT_TIMEOUT))
        .map(|o| parse_updates(&o.stdout))
        .map_err(|e| e.to_string())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    pub upgraded: Vec<String>,
    /// Asked for but still pending afterwards (held back, failed).
    pub still_pending: Vec<String>,
    pub reboot_required: bool,
    /// The last lines the package manager printed.
    pub tail: String,
    pub after: Summary,
}

impl Applied {
    pub fn describe(&self) -> String {
        let mut s = format!("{} upgraded", self.upgraded.len());
        if !self.still_pending.is_empty() {
            s.push_str(&format!(", {} still pending ({})", self.still_pending.len(), self.still_pending.iter().take(4).cloned().collect::<Vec<_>>().join(", ")));
        }
        if self.reboot_required {
            s.push_str(" · reboot required");
        }
        s
    }
}

fn last_lines(text: &str, n: usize) -> String {
    // apt redraws its progress in place; only the finished line is news.
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with("(Reading database")).collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// Upgrades `packages` and reads what's pending afterwards. A package
/// manager that fails is the step's failure, with its last lines.
pub fn apply(host: &dyn Host, manager: &str, packages: &[String]) -> Result<Applied, String> {
    let argv = apply_argv(manager, packages)?;
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let output = host.exec_privileged(&argv, &[], APPLY_TIMEOUT).map_err(|e| match e {
        crate::host::HostError::Failed { stderr, status } => format!("{manager} failed (exit {status}): {}", last_lines(&stderr, 3)),
        other => other.to_string(),
    })?;
    let after = read_report(host)?;
    let now = chrono::Utc::now().timestamp();
    let pending: HashSet<&str> = after.updates.iter().map(|u| u.name.as_str()).collect();
    let (still, done): (Vec<String>, Vec<String>) = packages.iter().cloned().partition(|p| pending.contains(p.as_str()));
    Ok(Applied { upgraded: done, still_pending: still, reboot_required: after.reboot_required, tail: last_lines(&output.stdout, 6), after: Summary::from_report(&after, now) })
}

// ---------------------------------------------------------------------------
// Maintenance windows
// ---------------------------------------------------------------------------

/// When a group may be patched and rebooted: days of the week and local
/// hours. A window past midnight ("Sun 22-02") belongs to the day it starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    /// Bit 0 = Monday … bit 6 = Sunday.
    pub days: u8,
    pub start: u8,
    pub end: u8,
}

const DAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

fn day_index(s: &str) -> Option<usize> {
    let s = s.to_lowercase();
    DAYS.iter().position(|d| s.starts_with(d))
}

impl Window {
    /// "Sun 02-05", "Mon-Fri 22-06", "Sat,Sun 01-04", "daily 03-05".
    pub fn parse(s: &str) -> Result<Window, String> {
        let s = s.trim();
        let (days, hours) = s.rsplit_once(' ').ok_or("a window looks like \"Sun 02-05\" or \"Mon-Fri 22-06\"")?;
        let mut mask = 0u8;
        for part in days.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            if matches!(part.to_lowercase().as_str(), "daily" | "every" | "everyday" | "all") {
                mask = 0x7f;
                continue;
            }
            match part.split_once('-') {
                Some((a, b)) => {
                    let (a, b) = (day_index(a).ok_or(format!("{a:?} isn't a day"))?, day_index(b).ok_or(format!("{b:?} isn't a day"))?);
                    let mut d = a;
                    loop {
                        mask |= 1 << d;
                        if d == b {
                            break;
                        }
                        d = (d + 1) % 7;
                    }
                }
                None => mask |= 1 << day_index(part).ok_or(format!("{part:?} isn't a day"))?,
            }
        }
        if mask == 0 {
            return Err("no days in the window".into());
        }
        let (a, b) = hours.split_once('-').ok_or("hours look like 02-05")?;
        let hour = |h: &str| h.trim().trim_end_matches(":00").parse::<u8>().ok().filter(|h| *h <= 24);
        let (start, end) = (hour(a).ok_or("hours are 0 to 24")?, hour(b).ok_or("hours are 0 to 24")?);
        if start == end {
            return Err("the window's start and end are the same hour".into());
        }
        Ok(Window { days: mask, start: start % 24, end: if end == 24 { 0 } else { end } })
    }

    fn has_day(&self, weekday0: usize) -> bool {
        self.days & (1 << weekday0) != 0
    }

    /// Whether local time `(weekday from Monday = 0, hour)` is inside.
    pub fn contains_at(&self, weekday0: usize, hour: u8) -> bool {
        if self.start < self.end {
            self.has_day(weekday0) && (self.start..self.end).contains(&hour)
        } else {
            (self.has_day(weekday0) && hour >= self.start) || (self.has_day((weekday0 + 6) % 7) && hour < self.end)
        }
    }

    pub fn contains_now(&self) -> bool {
        let now = chrono::Local::now();
        self.contains_at(now.weekday().num_days_from_monday() as usize, now.hour() as u8)
    }

    pub fn describe(&self) -> String {
        let days = if self.days == 0x7f {
            "daily".to_string()
        } else {
            DAYS.iter().enumerate().filter(|(i, _)| self.has_day(*i)).map(|(_, d)| {
                let mut c = d.chars();
                c.next().map(|f| f.to_uppercase().chain(c).collect::<String>()).unwrap_or_default()
            }).collect::<Vec<_>>().join(",")
        };
        format!("{days} {:02}-{:02}", self.start, self.end)
    }
}

/// The vault flag that holds a group's window.
pub fn window_flag(group: &str) -> String {
    format!("patching.window.{group}")
}

/// The vault flag that holds the last fleet scan.
pub const SUMMARIES_FLAG: &str = "patching.summaries";

/// Why a server is left out of a run right now, if it is.
pub fn outside_window(group: &str, windows: &BTreeMap<String, Window>) -> Option<String> {
    let w = windows.get(group)?;
    (!w.contains_now()).then(|| format!("outside {group}'s maintenance window ({})", w.describe()))
}

// ---------------------------------------------------------------------------
// Reboot order
// ---------------------------------------------------------------------------

/// How many bastions sit between Crow and `s`.
fn depth(s: &ServerRecord, servers: &[ServerRecord]) -> usize {
    let mut d = 0;
    let mut at = s.jump_host_id.as_deref();
    while let Some(id) = at {
        d += 1;
        if d > crate::host::ssh::MAX_HOPS {
            break;
        }
        at = servers.iter().find(|o| o.id == id).and_then(|o| o.jump_host_id.as_deref());
    }
    d
}

/// Servers in the order they may be rebooted: the ones furthest behind
/// bastions first, so no bastion goes down while a server still needs it.
pub fn reboot_order(targets: Vec<ServerRecord>, all: &[ServerRecord]) -> Vec<ServerRecord> {
    let mut t = targets;
    t.sort_by(|a, b| depth(b, all).cmp(&depth(a, all)).then(a.name.cmp(&b.name)));
    t
}

// ---------------------------------------------------------------------------
// Services after a reboot
// ---------------------------------------------------------------------------

const SYSTEMD_STATE: &str = "command -v systemctl >/dev/null 2>&1 || { echo no-systemd; exit 0; }; systemctl is-system-running 2>/dev/null; echo @@failed; systemctl --failed --plain --no-legend --no-pager 2>/dev/null | awk '{print $1}'";

/// ("running" / "degraded" / "starting" …, failed units), or `None`
/// without systemd.
pub fn parse_system_state(out: &str) -> Option<(String, Vec<String>)> {
    let (state, failed) = out.split_once("@@failed").unwrap_or((out, ""));
    let state = state.trim().lines().last().unwrap_or("").trim().to_string();
    if state == "no-systemd" {
        return None;
    }
    let mut failed: Vec<String> = failed.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('●')).map(String::from).collect();
    failed.sort();
    Some((state, failed))
}

pub fn system_state(host: &dyn Host) -> Result<Option<(String, Vec<String>)>, String> {
    host.exec(&["sh", "-c", SYSTEMD_STATE], crate::host::DEFAULT_TIMEOUT).map(|o| parse_system_state(&o.stdout)).map_err(|e| e.to_string())
}

/// After a reboot: waits for systemd to finish starting, then fails if a
/// unit that was fine before the reboot has failed.
pub fn wait_for_services(host: &dyn Host, failed_before: &[String], timeout: Duration, poll: Duration) -> Result<String, String> {
    let started = std::time::Instant::now();
    loop {
        match system_state(host) {
            Ok(None) => return Ok("no systemd to wait for".into()),
            Ok(Some((state, failed))) if state != "starting" && state != "initializing" => {
                let new: Vec<&String> = failed.iter().filter(|f| !failed_before.contains(f)).collect();
                return if new.is_empty() {
                    Ok(format!("services up ({state})"))
                } else {
                    Err(format!("came back, but {} failed: {}", if new.len() == 1 { "a unit" } else { "units" }, new.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")))
                };
            }
            _ => {}
        }
        if started.elapsed() >= timeout {
            return Err(format!("came back, but systemd was still starting after {}s", timeout.as_secs()));
        }
        std::thread::sleep(poll);
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_argv, outside_window, parse_system_state, reboot_order, valid_package, Scope, Summary, Window};
    use crate::vault::ServerRecord;
    use crate::views::overview::updates::{PackageUpdate, UpdatesReport};
    use std::collections::BTreeMap;

    #[test]
    fn only_the_planned_packages_are_upgraded_and_names_stay_arguments() {
        let argv = apply_argv("apt", &["openssl".into(), "libssl3t64".into()]).unwrap();
        assert_eq!(&argv[..2], ["sh", "-c"]);
        assert!(argv[2].contains("--only-upgrade") && argv[2].contains("\"$@\"") && argv[2].contains("force-confold"));
        assert_eq!(&argv[3..], ["crow-patch", "openssl", "libssl3t64"]);
        assert!(apply_argv("dnf", &["kernel-core".into()]).unwrap()[2].contains("dnf -y -q upgrade -- \"$@\""));
        assert!(apply_argv("apk", &["musl".into()]).unwrap()[2].contains("apk add -q --upgrade"));
        assert!(apply_argv("apt", &["x; rm -rf /".into()]).is_err());
        assert!(apply_argv("apt", &["--purge".into()]).is_err());
        assert!(apply_argv("apt", &[]).is_err());
        assert!(apply_argv("pacman", &["x".into()]).unwrap_err().contains("pacman"));
        assert!(valid_package("libstdc++6") && valid_package("python3.12") && valid_package("1:openssl"));
    }

    #[test]
    fn a_summary_splits_security_from_the_rest() {
        let up = |n: &str, s: bool| PackageUpdate { name: n.into(), installed: "1".into(), candidate: "2".into(), security: s };
        let r = UpdatesReport { manager: "apt".into(), updates: vec![up("openssl", true), up("base-files", false), up("linux-image-generic", true)], reboot_required: true, ..Default::default() };
        let s = Summary::from_report(&r, 10);
        assert_eq!((s.security.len(), s.other.len(), s.total()), (2, 1, 3));
        assert_eq!(s.packages(Scope::Security), ["openssl", "linux-image-generic"]);
        assert_eq!(s.packages(Scope::All).len(), 3);
        assert!(s.kernel.is_some() && s.reboot_required);
    }

    #[test]
    fn windows_read_like_people_write_them() {
        let w = Window::parse("Sun 02-05").unwrap();
        assert!(w.contains_at(6, 2) && w.contains_at(6, 4) && !w.contains_at(6, 5) && !w.contains_at(5, 3));
        // Past midnight: Friday night into Saturday morning belongs to Friday.
        let n = Window::parse("Mon-Fri 22-06").unwrap();
        assert!(n.contains_at(0, 23) && n.contains_at(5, 3), "Saturday 03:00 is Friday night's window");
        assert!(!n.contains_at(0, 3), "Monday 03:00 would be Sunday night's, which isn't in it");
        assert!(!n.contains_at(2, 12));
        assert_eq!(n.describe(), "Mon,Tue,Wed,Thu,Fri 22-06");
        assert_eq!(Window::parse("daily 03-05").unwrap().describe(), "daily 03-05");
        assert!(Window::parse("Sat,Sun 01:00-04:00").unwrap().contains_at(5, 1));
        assert!(Window::parse("Fri-Mon 01-02").unwrap().contains_at(0, 1), "wraps the week");
        assert!(Window::parse("someday 01-02").is_err() && Window::parse("Sun 3-3").is_err() && Window::parse("Sun").is_err());
        let windows: BTreeMap<String, Window> = [("web".to_string(), Window { days: 0, start: 1, end: 2 })].into();
        assert!(outside_window("web", &windows).unwrap().contains("web's maintenance window"));
        assert_eq!(outside_window("db", &windows), None, "no window: any time");
    }

    #[test]
    fn bastions_reboot_after_the_servers_behind_them() {
        let s = |id: &str, jump: Option<&str>| ServerRecord { id: id.into(), name: id.into(), jump_host_id: jump.map(Into::into), ..Default::default() };
        let all = vec![s("edge", None), s("inner", Some("edge")), s("app", Some("inner")), s("web", None)];
        let order: Vec<String> = reboot_order(all.clone(), &all).into_iter().map(|s| s.id).collect();
        assert_eq!(order, ["app", "inner", "edge", "web"]);
    }

    #[test]
    fn systemd_state_and_failed_units_are_read() {
        assert_eq!(parse_system_state("degraded\n@@failed\nnginx.service\nfoo.mount\n"), Some(("degraded".into(), vec!["foo.mount".into(), "nginx.service".into()])));
        assert_eq!(parse_system_state("running\n@@failed\n"), Some(("running".into(), vec![])));
        assert_eq!(parse_system_state("no-systemd\n"), None);
    }
}

/// Applies real updates in throwaway containers with pending ones:
///   podman run -d --name crow-patch-deb debian:12.0 sleep 3600   (apt-get update inside first)
///   podman run -d --name crow-patch-apk alpine:3.19.0 sleep 3600 (apk update inside first)
///   cargo test live_patch_apply -- --ignored --nocapture
#[cfg(test)]
#[test]
#[ignore]
fn live_patch_apply() {
    for (name, scope) in [("crow-patch-deb", Scope::Security), ("crow-patch-apk", Scope::All)] {
        let host = crate::host::ContainerHost::new("podman", name);
        let before = read_report(&host).expect("probe");
        let s = Summary::from_report(&before, 0);
        let packages: Vec<String> = s.packages(scope).into_iter().take(5).collect();
        eprintln!("{name}: {} via {}, {} security, {} other; applying {packages:?}", before.updates.len(), s.manager, s.security.len(), s.other.len());
        assert!(!packages.is_empty(), "{name} has nothing pending: use an older image");
        let applied = apply(&host, &s.manager, &packages).expect("apply");
        eprintln!("{name}: {}\n{}", applied.describe(), applied.tail);
        assert_eq!(applied.upgraded.len(), packages.len(), "every planned package upgraded: {:?}", applied.still_pending);
        assert!(applied.after.total() < s.total());
    }
}

/// Reads systemd's state here (read-only) and in a container without systemd:
///   cargo test live_services_state -- --ignored --nocapture
#[cfg(test)]
#[test]
#[ignore]
fn live_services_state() {
    let here = system_state(&crate::host::LocalHost).expect("read");
    eprintln!("this machine: {here:?}");
    assert!(here.as_ref().is_some_and(|(s, _)| !s.is_empty()));
    let failed_now = here.map(|(_, f)| f).unwrap_or_default();
    eprintln!("{:?}", wait_for_services(&crate::host::LocalHost, &failed_now, Duration::from_secs(5), Duration::from_secs(1)));
    let c = crate::host::ContainerHost::new("podman", "crow-patch-deb");
    assert_eq!(wait_for_services(&c, &[], Duration::from_secs(5), Duration::from_secs(1)), Ok("no systemd to wait for".into()));
}
