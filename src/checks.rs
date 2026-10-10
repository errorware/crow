//! Checks from outside (ERR-148): what a server's users see, checked from
//! Crow's machine: an HTTP(S) answer (status, keyword, latency, the
//! certificate), a TCP port, a DNS name. Only while Crow is open.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// The vault flag that holds the checks.
pub const CHECKS_FLAG: &str = "checks.defs";
/// Results are kept this long.
pub const KEEP_SECS: i64 = 30 * 86_400;
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// `keyword`: must appear in the body; `status`: expected (else 2xx/3xx).
    Http { url: String, keyword: String, status: Option<u16> },
    Tcp { host: String, port: u16 },
    /// `expect`: an address the name must resolve to (empty: any).
    Dns { name: String, expect: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    /// The server it's attached to (its page, the map, its alerts).
    pub server_id: Option<String>,
    pub every_secs: i64,
    pub alert: bool,
}

impl Check {
    pub fn target(&self) -> String {
        match &self.kind {
            Kind::Http { url, keyword, status } => format!("{url}{}{}", status.map(|s| format!(" → {s}")).unwrap_or_default(), if keyword.is_empty() { String::new() } else { format!(" containing \"{keyword}\"") }),
            Kind::Tcp { host, port } => format!("tcp {host}:{port}"),
            Kind::Dns { name, expect } => format!("dns {name}{}", if expect.is_empty() { String::new() } else { format!(" → {expect}") }),
        }
    }

    pub fn alert_kind(&self) -> String {
        format!("check:{}", self.id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub ok: bool,
    pub latency_ms: Option<i64>,
    /// What was seen ("200 in 84 ms"), or what went wrong.
    pub detail: String,
    /// HTTPS: days until the certificate expires.
    pub cert_days: Option<i64>,
}

fn fail(detail: impl Into<String>, latency_ms: Option<i64>) -> Outcome {
    Outcome { ok: false, latency_ms, detail: detail.into(), cert_days: None }
}

/// Parses curl's `-w` line ("code time") and its `-v` certificate line.
pub fn parse_curl(write_out: &str, verbose: &str, now: i64) -> (Option<u16>, Option<i64>, Option<i64>) {
    let mut parts = write_out.split_whitespace();
    let code = parts.next().and_then(|c| c.parse().ok()).filter(|c| *c != 0);
    let ms = parts.next().and_then(|t| t.parse::<f64>().ok()).map(|t| (t * 1000.0).round() as i64);
    let cert = verbose.lines().find_map(|l| l.split("expire date: ").nth(1)).and_then(|d| chrono::NaiveDateTime::parse_from_str(d.trim().trim_end_matches("GMT").trim(), "%b %e %H:%M:%S %Y").ok()).map(|t| (t.and_utc().timestamp() - now) / 86_400);
    (code, ms, cert)
}

/// The last line curl printed about why it failed.
fn curl_error(stderr: &str) -> String {
    stderr.lines().rev().find(|l| l.starts_with("curl:")).map(|l| l.trim_start_matches("curl:").trim().to_string()).unwrap_or_else(|| "no answer".into())
}

pub fn run(check: &Check) -> Outcome {
    match &check.kind {
        Kind::Http { url, keyword, status } => {
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return fail("the URL must start with http:// or https://", None);
            }
            // Only the start of the body is kept, for the keyword.
            let out = std::process::Command::new("curl")
                .args(["-sS", "-v", "-L", "--max-redirs", "5", "--max-time", "10", "--max-filesize", "2000000", "-A", "Crow uptime check", "-w", "\n@@crow %{http_code} %{time_total}", "--", url])
                .output();
            let out = match out {
                Ok(o) => o,
                Err(e) => return fail(format!("couldn't run curl: {e}"), None),
            };
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let (body, write_out) = stdout.rsplit_once("@@crow ").unwrap_or((&stdout, ""));
            let (code, ms, cert_days) = parse_curl(write_out, &stderr, chrono::Utc::now().timestamp());
            let Some(code) = code else { return fail(curl_error(&stderr), ms) };
            let status_ok = match status {
                Some(s) => code == *s,
                None => (200..400).contains(&code),
            };
            let mut o = if !status_ok {
                fail(format!("HTTP {code}{}", status.map(|s| format!(" (expected {s})")).unwrap_or_default()), ms)
            } else if !keyword.is_empty() && !body.contains(keyword.as_str()) {
                fail(format!("HTTP {code}, but \"{keyword}\" isn't in the page"), ms)
            } else {
                Outcome { ok: true, latency_ms: ms, detail: format!("HTTP {code} in {} ms", ms.unwrap_or(0)), cert_days: None }
            };
            o.cert_days = cert_days;
            if let Some(d) = cert_days.filter(|d| *d < 0) {
                o.ok = false;
                o.detail = format!("the certificate expired {} days ago", -d);
            }
            o
        }
        Kind::Tcp { host, port } => {
            let start = Instant::now();
            let addrs: Vec<_> = match (host.as_str(), *port).to_socket_addrs() {
                Ok(a) => a.collect(),
                Err(e) => return fail(format!("can't resolve {host}: {e}"), None),
            };
            let mut last = String::from("no address");
            for a in addrs {
                match TcpStream::connect_timeout(&a, TIMEOUT) {
                    Ok(_) => {
                        let ms = start.elapsed().as_millis() as i64;
                        return Outcome { ok: true, latency_ms: Some(ms), detail: format!("{a} open in {ms} ms"), cert_days: None };
                    }
                    Err(e) => last = format!("{a}: {e}"),
                }
            }
            fail(last, Some(start.elapsed().as_millis() as i64))
        }
        Kind::Dns { name, expect } => {
            let start = Instant::now();
            match (name.as_str(), 0).to_socket_addrs() {
                Ok(a) => {
                    let ms = start.elapsed().as_millis() as i64;
                    let ips: Vec<String> = a.map(|a| a.ip().to_string()).collect();
                    if !expect.trim().is_empty() && !ips.iter().any(|i| i == expect.trim()) {
                        fail(format!("{name} resolves to {}, not {expect}", ips.join(", ")), Some(ms))
                    } else {
                        Outcome { ok: true, latency_ms: Some(ms), detail: format!("{name} → {}", ips.join(", ")), cert_days: None }
                    }
                }
                Err(e) => fail(format!("{name} doesn't resolve: {e}"), Some(start.elapsed().as_millis() as i64)),
            }
        }
    }
}

/// Up-time over results (percent), `None` without results.
pub fn uptime(results: &[(i64, bool, Option<i64>, String)]) -> Option<f64> {
    (!results.is_empty()).then(|| 100.0 * results.iter().filter(|r| r.1).count() as f64 / results.len() as f64)
}

/// Alert changes for a check: open after `failures` in a row (2 skips a
/// blip), resolved on the first success.
pub fn alert_changes(open: &[crate::metrics::alerts::Alert], check: &Check, alert_server: &str, outcome: &Outcome, failures_in_a_row: u32, now: i64) -> crate::metrics::alerts::AlertChanges {
    use crate::metrics::alerts::{Alert, AlertChanges};
    let kind = check.alert_kind();
    let existing = open.iter().find(|a| a.server_id == alert_server && a.kind == kind && a.resolved_at.is_none());
    let mut c = AlertChanges::default();
    if outcome.ok {
        if let Some(a) = existing {
            c.resolved.push(a.id.clone());
        }
    } else if failures_in_a_row >= 2 && check.alert {
        let detail = format!("check \"{}\" failing: {}", check.name, outcome.detail);
        match existing {
            Some(a) => c.updated.push((a.id.clone(), "CRIT".into(), detail, now)),
            None => c.opened.push(Alert { id: format!("alert-{alert_server}-{kind}-{now}"), server_id: alert_server.into(), kind, level: "CRIT".into(), detail, opened_at: now, last_seen: now, resolved_at: None, acknowledged_at: None }),
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::{alert_changes, parse_curl, run, uptime, Check, Kind, Outcome};

    fn check(kind: Kind) -> Check {
        Check { id: "c1".into(), name: "site".into(), kind, server_id: None, every_secs: 60, alert: true }
    }

    #[test]
    fn curl_output_is_read() {
        let now = chrono::NaiveDate::from_ymd_opt(2026, 10, 10).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let (code, ms, days) = parse_curl("200 0.084512", "*  expire date: Nov  9 10:00:00 2026 GMT\n", now);
        assert_eq!((code, ms, days), (Some(200), Some(85), Some(30)));
        assert_eq!(parse_curl("000 0.000", "", now).0, None, "no answer");
    }

    #[test]
    fn tcp_and_dns_checks_run_for_real() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(run(&check(Kind::Tcp { host: "127.0.0.1".into(), port })).ok);
        // Not the port just freed: a parallel test can bind it again.
        let closed = run(&check(Kind::Tcp { host: "127.0.0.1".into(), port: 1 }));
        assert!(!closed.ok, "{}", closed.detail);
        assert!(run(&check(Kind::Dns { name: "localhost".into(), expect: String::new() })).ok);
        assert!(!run(&check(Kind::Dns { name: "localhost".into(), expect: "10.9.9.9".into() })).ok);
        assert!(!run(&check(Kind::Http { url: "ftp://x".into(), keyword: String::new(), status: None })).ok);
    }

    #[test]
    fn alerts_wait_for_a_second_failure_and_resolve_on_success() {
        let c = check(Kind::Tcp { host: "x".into(), port: 1 });
        let bad = Outcome { ok: false, latency_ms: None, detail: "refused".into(), cert_days: None };
        assert!(alert_changes(&[], &c, "web-1", &bad, 1, 1).opened.is_empty(), "a blip");
        let opened = alert_changes(&[], &c, "web-1", &bad, 2, 2).opened;
        assert_eq!((opened.len(), opened[0].level.as_str()), (1, "CRIT"));
        let good = Outcome { ok: true, latency_ms: Some(5), detail: "open".into(), cert_days: None };
        assert_eq!(alert_changes(&opened, &c, "web-1", &good, 0, 3).resolved.len(), 1);
        assert_eq!(uptime(&[(1, true, None, String::new()), (2, false, None, String::new())]), Some(50.0));
    }

    /// HTTPS against a real site: CROW_CHECK_URL=https://example.com cargo test live_http_check -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_http_check() {
        let url = std::env::var("CROW_CHECK_URL").unwrap_or_else(|_| "https://example.com".into());
        let o = run(&check(Kind::Http { url, keyword: "Example".into(), status: None }));
        eprintln!("{o:?}");
        assert!(o.ok && o.cert_days.is_some());
    }
}
