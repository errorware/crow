//! CVE lookups against OSV.dev (https://osv.dev), a free public vulnerability
//! database covering Debian and Ubuntu packages. Requests go through the
//! system `curl` (like `ssh`, no HTTP stack in Crow) and carry only package
//! names, versions and the distro release — never host names or addresses.
//!
//! What's reported is what an action fixes, not raw counts:
//! - fixed by upgrading: affects the installed version, not the pending one
//! - no fix yet: affects a package behind a running program with no update
//!
//! The kernel is left out: OSV records thousands of CVEs against `linux`,
//! served ~85 per 10-second page, so a version diff takes minutes. The
//! package manager already says what matters there (a kernel update from the
//! security pocket, reboot required), see `UpdatesReport`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use serde_json::{json, Value};

use crate::host::{Host, LocalHost};
use crate::views::overview::updates::{SourcePkg, UpdatesReport};

const BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";
const QUERY_URL: &str = "https://api.osv.dev/v1/query";
const VULN_URL: &str = "https://api.osv.dev/v1/vulns/";
/// OSV accepts up to 1000 queries per batch.
const BATCH_SIZE: usize = 1000;
/// Detail lookups (for severity and summary) are capped per refresh.
const MAX_DETAILS: usize = 60;
const DETAIL_SEPARATOR: &str = "@@crow-osv@@";

#[derive(Clone, Debug, PartialEq)]
pub struct VulnInfo {
    pub id: String,
    /// The upstream CVE id, when the record names one.
    pub cve: Option<String>,
    /// Distro priority (Ubuntu: negligible/low/medium/high/critical), if given.
    pub severity: Option<String>,
    pub summary: String,
}

impl VulnInfo {
    /// Sort key: most severe first.
    pub fn rank(&self) -> u8 {
        match self.severity.as_deref() {
            Some("critical") => 0,
            Some("high") => 1,
            Some("medium") => 2,
            Some("low") => 3,
            Some("negligible") => 4,
            _ => 5,
        }
    }

    pub fn label(&self) -> &str {
        self.cve.as_deref().unwrap_or(&self.id)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CveReport {
    /// Source package → vulnerability ids its pending upgrade fixes.
    pub fixed_by_upgrade: BTreeMap<String, Vec<String>>,
    /// Vulnerabilities in running programs' packages with no update yet.
    pub no_fix_yet: usize,
    /// Details for (up to MAX_DETAILS of) the fixable ids.
    pub details: HashMap<String, VulnInfo>,
    pub ecosystem: String,
}

impl CveReport {
    pub fn fixable_count(&self) -> usize {
        self.fixed_by_upgrade.values().map(Vec::len).sum::<usize>()
    }
}

fn curl_json(url: &str, body: Option<&str>) -> Result<Value, String> {
    let mut argv = vec!["curl", "-sS", "--fail-with-body", "--max-time", "30"];
    if body.is_some() {
        argv.extend(["-H", "Content-Type: application/json", "--data-binary", "@-"]);
    }
    argv.push(url);
    let out = LocalHost
        .exec_stdin(&argv, body.unwrap_or("").as_bytes(), Duration::from_secs(40))
        .map_err(|e| format!("OSV request failed: {e}"))?;
    serde_json::from_str(&out.stdout).map_err(|e| format!("OSV answered with something unexpected: {e}"))
}

/// The per-CVE records in a result (`UBUNTU-CVE-…`, `DEBIAN-CVE-…`). OSV
/// also returns the distro's advisories (USN, DSA, DLA), which bundle the
/// same CVEs; counting both would double every issue.
fn ids_of(result: &Value) -> Vec<String> {
    result["vulns"]
        .as_array()
        .map(|v| v.iter().filter_map(|x| x["id"].as_str()).filter(|id| id.contains("-CVE-")).map(str::to_string).collect())
        .unwrap_or_default()
}

/// Vulnerability ids affecting each (package, version), following pagination.
fn query_all(ecosystem: &str, pkgs: &[SourcePkg]) -> Result<HashMap<SourcePkg, BTreeSet<String>>, String> {
    let mut out = HashMap::new();
    for chunk in pkgs.chunks(BATCH_SIZE) {
        let queries: Vec<Value> = chunk.iter().map(|p| json!({"package": {"name": p.name, "ecosystem": ecosystem}, "version": p.version})).collect();
        let resp = curl_json(BATCH_URL, Some(&json!({ "queries": queries }).to_string()))?;
        let results = resp["results"].as_array().cloned().unwrap_or_default();
        for (pkg, result) in chunk.iter().zip(results) {
            let mut ids: BTreeSet<String> = ids_of(&result).into_iter().collect();
            let mut token = result["next_page_token"].as_str().map(str::to_string);
            let mut pages = 0;
            // Safety net: results this large are the kernel's, which is skipped.
            while let Some(t) = token.filter(|t| !t.is_empty() && pages < 3) {
                let page = curl_json(QUERY_URL, Some(&json!({"package": {"name": pkg.name, "ecosystem": ecosystem}, "version": pkg.version, "page_token": t}).to_string()))?;
                ids.extend(ids_of(&page));
                token = page["next_page_token"].as_str().map(str::to_string);
                pages += 1;
            }
            out.insert(pkg.clone(), ids);
        }
    }
    Ok(out)
}

/// Parses one `/v1/vulns/{id}` record.
pub fn parse_vuln(v: &Value) -> Option<VulnInfo> {
    let id = v["id"].as_str()?.to_string();
    let cve = v["upstream"].as_array().and_then(|u| u.iter().filter_map(Value::as_str).find(|s| s.starts_with("CVE-")).map(str::to_string))
        .or_else(|| v["aliases"].as_array().and_then(|a| a.iter().filter_map(Value::as_str).find(|s| s.starts_with("CVE-")).map(str::to_string)))
        .or_else(|| id.find("CVE-").map(|i| id[i..].to_string()));
    let severity = v["severity"].as_array().and_then(|s| s.iter().find(|x| x["type"] == "Ubuntu").and_then(|x| x["score"].as_str()).map(|s| s.to_lowercase()));
    let summary = v["summary"].as_str().filter(|s| !s.is_empty()).or_else(|| v["details"].as_str()).unwrap_or("").lines().next().unwrap_or("").chars().take(140).collect();
    Some(VulnInfo { id, cve, severity, summary })
}

/// Details for `ids`, in one curl run (one connection, one record per URL).
fn details(ids: &[String]) -> HashMap<String, VulnInfo> {
    if ids.is_empty() {
        return HashMap::new();
    }
    let urls: Vec<String> = ids.iter().map(|id| format!("{VULN_URL}{id}")).collect();
    let separator = format!("\n{DETAIL_SEPARATOR}\n");
    let mut argv = vec!["curl", "-sS", "--max-time", "60", "-w", separator.as_str()];
    argv.extend(urls.iter().map(String::as_str));
    let Ok(out) = LocalHost.exec(&argv, Duration::from_secs(70)) else { return HashMap::new() };
    out.stdout
        .split(DETAIL_SEPARATOR)
        .filter_map(|chunk| serde_json::from_str::<Value>(chunk.trim()).ok())
        .filter_map(|v| parse_vuln(&v))
        .map(|v| (v.id.clone(), v))
        .collect()
}

/// Kernel source packages (`linux`, `linux-signed-hwe-6.8`, ...), which OSV
/// can't answer quickly; linux-firmware is an ordinary package.
pub fn is_kernel_source(name: &str) -> bool {
    (name == "linux" || name.starts_with("linux-")) && !name.starts_with("linux-firmware")
}

/// Looks up the CVEs that matter for `report` (Debian/Ubuntu only).
pub fn lookup(report: &UpdatesReport) -> Result<CveReport, String> {
    let ecosystem = report.osv_ecosystem().ok_or_else(|| format!("CVE lookup covers Debian and Ubuntu; this server runs {}", if report.os_id.is_empty() { "an unknown OS" } else { &report.os_id }))?;

    // Pending upgrades as (source at installed, source at candidate). The
    // candidate's source version is only known when the binary shares it.
    let mut upgrades: BTreeMap<String, (SourcePkg, SourcePkg)> = BTreeMap::new();
    for u in &report.updates {
        let Some(src) = report.sources.get(&u.name) else { continue };
        if src.version == u.installed && !is_kernel_source(&src.name) {
            upgrades.entry(src.name.clone()).or_insert((src.clone(), SourcePkg { name: src.name.clone(), version: u.candidate.clone() }));
        }
    }
    let running: Vec<SourcePkg> = report.running.iter().filter(|p| !upgrades.contains_key(&p.name) && !is_kernel_source(&p.name)).cloned().collect();

    let mut wanted: BTreeSet<SourcePkg> = BTreeSet::new();
    for (installed, candidate) in upgrades.values() {
        wanted.insert(installed.clone());
        wanted.insert(candidate.clone());
    }
    wanted.extend(running.iter().cloned());
    let wanted: Vec<SourcePkg> = wanted.into_iter().collect();
    let found = query_all(&ecosystem, &wanted)?;
    let ids = |p: &SourcePkg| found.get(p).cloned().unwrap_or_default();

    let mut out = CveReport { ecosystem, ..Default::default() };
    for (name, (installed, candidate)) in &upgrades {
        let fixed: Vec<String> = ids(installed).difference(&ids(candidate)).cloned().collect();
        if !fixed.is_empty() {
            out.fixed_by_upgrade.insert(name.clone(), fixed);
        }
    }
    out.no_fix_yet = running.iter().map(|p| ids(p).len()).sum();

    let mut detail_ids: Vec<String> = out.fixed_by_upgrade.values().flatten().cloned().collect();
    detail_ids.sort();
    detail_ids.dedup();
    detail_ids.truncate(MAX_DETAILS);
    out.details = details(&detail_ids);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ubuntu_vuln_records() {
        let v: Value = serde_json::from_str(r#"{"id":"UBUNTU-CVE-2025-15467","summary":"","details":"Issue summary: A buffer overflow in X.\nMore.","upstream":["CVE-2025-15467"],"severity":[{"type":"CVSS_V3","score":"CVSS:3.1/AV:N"},{"type":"Ubuntu","score":"Medium"}]}"#).unwrap();
        let info = parse_vuln(&v).unwrap();
        assert_eq!(info.cve.as_deref(), Some("CVE-2025-15467"));
        assert_eq!(info.severity.as_deref(), Some("medium"));
        assert_eq!(info.summary, "Issue summary: A buffer overflow in X.");
        let debian: Value = serde_json::from_str(r#"{"id":"DEBIAN-CVE-2024-1234","details":"x"}"#).unwrap();
        let d = parse_vuln(&debian).unwrap();
        assert_eq!((d.cve.as_deref(), d.severity.as_deref()), (Some("CVE-2024-1234"), None));
        assert!(info.rank() < d.rank());
    }

    #[test]
    fn advisories_are_not_counted_twice() {
        let r: Value = serde_json::from_str(r#"{"vulns":[{"id":"UBUNTU-CVE-2026-1"},{"id":"USN-8287-2"},{"id":"DEBIAN-CVE-2026-2"},{"id":"DSA-5555-1"}]}"#).unwrap();
        assert_eq!(ids_of(&r), ["UBUNTU-CVE-2026-1", "DEBIAN-CVE-2026-2"]);
    }

    #[test]
    fn kernel_packages_are_recognised() {
        for k in ["linux", "linux-signed", "linux-hwe-6.8", "linux-meta", "linux-restricted-modules"] {
            assert!(is_kernel_source(k), "{k}");
        }
        assert!(!is_kernel_source("linux-firmware") && !is_kernel_source("openssl"));
    }

    #[test]
    fn unsupported_os_is_explained() {
        let r = UpdatesReport { os_id: "rocky".into(), os_version: "9".into(), ..Default::default() };
        assert!(lookup(&r).unwrap_err().contains("Debian and Ubuntu"));
    }

    /// Hits the real OSV API: `cargo test live_osv -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn live_osv() {
        use crate::views::overview::updates::{parse_updates, UPDATES_PROBE};
        let out = LocalHost.exec(&["sh", "-c", UPDATES_PROBE], std::time::Duration::from_secs(30)).unwrap();
        let report = parse_updates(&out.stdout);
        let started = std::time::Instant::now();
        let cves = lookup(&report).unwrap();
        println!("{} in {:?}", cves.ecosystem, started.elapsed());
        println!("fixed by upgrade: {} packages, {} vulns", cves.fixed_by_upgrade.len(), cves.fixed_by_upgrade.values().map(Vec::len).sum::<usize>());
        println!("no fix yet (running): {}", cves.no_fix_yet);
        let mut top: Vec<&VulnInfo> = cves.details.values().collect();
        top.sort_by_key(|v| v.rank());
        for v in top.iter().take(8) {
            println!("  {:9} {:16} {}", v.severity.as_deref().unwrap_or("?"), v.label(), v.summary);
        }
    }
}
