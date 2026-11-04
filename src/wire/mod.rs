//! Outbound Wire & Egress Traceability (`ERR-211`..`ERR-218`).
//!
//! Every network call Crow makes — SSH commands, SFTP/SCP transfers, interactive
//! PTY terminals, onboarding probes, Clanker AI requests, cloud provider APIs,
//! OSV.dev CVE queries, external checks, SMTP emails, GeoIP downloads, release
//! checks, and external plugins — passes through this module and is cross-checked
//! against the Linux kernel's `/proc` socket table (`proc_net`).

pub mod proc_net;
pub mod redact;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use serde::{Deserialize, Serialize};

pub const MAX_MEMORY_EVENTS: usize = 2_500;
pub const MAX_PAYLOAD_CHARS: usize = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WireCategory {
    Ssh,
    Terminal,
    Onboard,
    Ai,
    Provider,
    Osv,
    Check,
    Notify,
    GeoIp,
    Update,
    Plugin,
}

impl WireCategory {
    pub const ALL: [WireCategory; 11] = [
        Self::Ssh,
        Self::Terminal,
        Self::Onboard,
        Self::Ai,
        Self::Provider,
        Self::Osv,
        Self::Check,
        Self::Notify,
        Self::GeoIp,
        Self::Update,
        Self::Plugin,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Terminal => "terminal",
            Self::Onboard => "onboard",
            Self::Ai => "ai",
            Self::Provider => "provider",
            Self::Osv => "osv",
            Self::Check => "check",
            Self::Notify => "notify",
            Self::GeoIp => "geoip",
            Self::Update => "update",
            Self::Plugin => "plugin",
        }
    }

    pub fn from_id(s: &str) -> Self {
        match s {
            "terminal" => Self::Terminal,
            "onboard" => Self::Onboard,
            "ai" => Self::Ai,
            "provider" => Self::Provider,
            "osv" => Self::Osv,
            "check" => Self::Check,
            "notify" => Self::Notify,
            "geoip" => Self::GeoIp,
            "update" => Self::Update,
            "plugin" => Self::Plugin,
            _ => Self::Ssh,
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Ssh => "SSH",
            Self::Terminal => "PTY",
            Self::Onboard => "PROBE",
            Self::Ai => "CLANKER",
            Self::Provider => "CLOUD",
            Self::Osv => "OSV.DEV",
            Self::Check => "CHECK",
            Self::Notify => "SMTP",
            Self::GeoIp => "GEOIP",
            Self::Update => "UPDATE",
            Self::Plugin => "PLUGIN",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireStatus {
    Ok,
    Error,
    Timeout,
    Blocked,
}

impl WireStatus {
    pub fn id(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Timeout => "timeout",
            Self::Blocked => "blocked",
        }
    }

    pub fn from_id(s: &str) -> Self {
        match s {
            "error" | "failed" => Self::Error,
            "timeout" => Self::Timeout,
            "blocked" => Self::Blocked,
            _ => Self::Ok,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireEvent {
    pub seq: u64,
    pub started_at_ms: i64,
    pub duration_ms: u64,
    pub category: WireCategory,
    pub protocol: String,
    pub caller: String,
    pub target_label: String,
    pub remote_host: String,
    pub remote_port: u16,
    pub via_hop: Option<String>,
    pub status: WireStatus,
    pub bytes_tx: u64,
    pub bytes_rx: u64,
    pub summary: String,
    pub request_payload: String,
    pub response_payload: String,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WireCounters {
    pub total_calls: u64,
    pub in_flight: usize,
    pub failed_calls: u64,
    pub blocked_calls: u64,
    pub bytes_tx: u64,
    pub bytes_rx: u64,
    pub silenced: bool,
}

static NEXT_SEQ: AtomicU64 = AtomicU64::new(1);
static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static SILENCED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static CALLER_STACK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Runs `f` with `caller` pushed onto the current thread's Wire caller stack
/// (e.g. `"dokku.scan"`, `"overview.poll"`, `"containers.scan"`).
pub fn with_caller<R>(caller: impl Into<String>, f: impl FnOnce() -> R) -> R {
    CALLER_STACK.with(|stack| stack.borrow_mut().push(caller.into()));
    struct PopGuard;
    impl Drop for PopGuard {
        fn drop(&mut self) {
            CALLER_STACK.with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    }
    let _g = PopGuard;
    f()
}

pub fn current_caller(fallback: &str) -> String {
    CALLER_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    })
}

/// True when the operator has engaged Outbound Silence / Lockdown mode.
pub fn is_silenced() -> bool {
    SILENCED.load(Ordering::Relaxed)
}

pub fn set_silenced(silenced: bool) {
    SILENCED.store(silenced, Ordering::Relaxed);
}

type PersistSink = Arc<dyn Fn(WireEvent) + Send + Sync + 'static>;

struct Ledger {
    events: VecDeque<WireEvent>,
    total_calls: u64,
    failed_calls: u64,
    blocked_calls: u64,
    bytes_tx: u64,
    bytes_rx: u64,
    persist: Option<PersistSink>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            events: VecDeque::with_capacity(512),
            total_calls: 0,
            failed_calls: 0,
            blocked_calls: 0,
            bytes_tx: 0,
            bytes_rx: 0,
            persist: None,
        }
    }
}

fn ledger() -> &'static Mutex<Ledger> {
    static LEDGER: OnceLock<Mutex<Ledger>> = OnceLock::new();
    LEDGER.get_or_init(Default::default)
}

/// Seeds historical events loaded from the SQLite `wire_log` table (`ERR-215`).
pub fn seed_from_vault(mut historical: Vec<WireEvent>) {
    // `list_wire_events` returns newest first; push oldest first into the ring buffer.
    historical.reverse();
    if let Ok(mut l) = ledger().lock() {
        let mut max_seq = NEXT_SEQ.load(Ordering::Relaxed);
        for ev in historical {
            if ev.seq >= max_seq {
                max_seq = ev.seq + 1;
            }
            l.total_calls += 1;
            match ev.status {
                WireStatus::Error | WireStatus::Timeout => l.failed_calls += 1,
                WireStatus::Blocked => l.blocked_calls += 1,
                WireStatus::Ok => {}
            }
            l.bytes_tx = l.bytes_tx.saturating_add(ev.bytes_tx);
            l.bytes_rx = l.bytes_rx.saturating_add(ev.bytes_rx);
            if l.events.len() >= MAX_MEMORY_EVENTS {
                l.events.pop_front();
            }
            l.events.push_back(ev);
        }
        NEXT_SEQ.store(max_seq.max(1), Ordering::Relaxed);
    }
}

/// Installs the SQLite vault persistence callback (`ERR-215`).
pub fn init_persistence(sink: PersistSink) {
    if let Ok(mut l) = ledger().lock() {
        l.persist = Some(sink);
    }
}

fn cap_payload(raw: &str) -> String {
    let scrubbed = redact::scrub_payload(raw);
    if scrubbed.chars().count() <= MAX_PAYLOAD_CHARS {
        scrubbed
    } else {
        let mut s: String = scrubbed.chars().take(MAX_PAYLOAD_CHARS).collect();
        s.push_str("\n… [truncated]");
        s
    }
}

pub struct WireStart {
    pub category: WireCategory,
    pub protocol: &'static str,
    pub caller: String,
    pub target_label: String,
    pub remote_host: String,
    pub remote_port: u16,
    pub via_hop: Option<String>,
    pub summary: String,
    pub request_payload: String,
}

/// Handle for an in-flight outbound network call.
pub struct WireSpan {
    pub seq: u64,
    started_at_ms: i64,
    started: Instant,
    start: Option<WireStart>,
}

/// Checks if Outbound Silence mode is active; if so, records a `Blocked` event
/// and returns `Err`.
pub fn check_allowed(
    category: WireCategory,
    protocol: &'static str,
    caller: &str,
    target_label: &str,
    remote_host: &str,
    remote_port: u16,
) -> Result<(), String> {
    if is_silenced() {
        record_instant(
            WireStart {
                category,
                protocol,
                caller: caller.to_string(),
                target_label: target_label.to_string(),
                remote_host: remote_host.to_string(),
                remote_port,
                via_hop: None,
                summary: format!("{protocol} {remote_host}:{remote_port}"),
                request_payload: String::new(),
            },
            WireStatus::Blocked,
            0,
            0,
            0,
            "",
            Some("Outbound traffic is silenced (Wire Lockdown active).".into()),
        );
        return Err("Outbound traffic is silenced (Wire Lockdown active).".into());
    }
    Ok(())
}

/// Checks if Outbound Silence mode is active; if so, records a `Blocked` event
/// and returns `Err`. Otherwise begins an in-flight `WireSpan`.
pub fn begin_checked(start: WireStart) -> Result<WireSpan, String> {
    if is_silenced() {
        record_instant(
            start,
            WireStatus::Blocked,
            0,
            0,
            0,
            "",
            Some("Outbound traffic is silenced (Wire Lockdown active).".into()),
        );
        return Err("Outbound traffic is silenced (Wire Lockdown active).".into());
    }
    Ok(begin(start))
}

/// Begins an in-flight `WireSpan` that will record its `WireEvent` when
/// `finish_ok`, `finish_err`, or `Drop` runs.
pub fn begin(start: WireStart) -> WireSpan {
    let seq = NEXT_SEQ.fetch_add(1, Ordering::Relaxed);
    IN_FLIGHT.fetch_add(1, Ordering::Relaxed);
    WireSpan {
        seq,
        started_at_ms: chrono::Utc::now().timestamp_millis(),
        started: Instant::now(),
        start: Some(start),
    }
}

pub fn record_instant(
    start: WireStart,
    status: WireStatus,
    duration_ms: u64,
    bytes_tx: u64,
    bytes_rx: u64,
    response_payload: &str,
    error: Option<String>,
) -> u64 {
    let seq = NEXT_SEQ.fetch_add(1, Ordering::Relaxed);
    let now_ms = chrono::Utc::now().timestamp_millis();
    commit_event(WireEvent {
        seq,
        started_at_ms: now_ms,
        duration_ms,
        category: start.category,
        protocol: start.protocol.to_string(),
        caller: start.caller,
        target_label: start.target_label,
        remote_host: start.remote_host,
        remote_port: start.remote_port,
        via_hop: start.via_hop,
        status,
        bytes_tx,
        bytes_rx,
        summary: redact::scrub_payload(&start.summary),
        request_payload: cap_payload(&start.request_payload),
        response_payload: cap_payload(response_payload),
        error: error.map(|e| redact::scrub_payload(&e)),
    });
    seq
}

fn commit_event(mut ev: WireEvent) {
    // An app's environment coming back: keys yes, values never.
    if redact::dumps_environment(&ev.request_payload) || redact::dumps_environment(&ev.summary) {
        ev.response_payload = redact::scrub_environment(&ev.response_payload);
        ev.error = ev.error.map(|e| redact::scrub_environment(&e));
    }
    let mut persist_cb = None;
    if let Ok(mut l) = ledger().lock() {
        l.total_calls += 1;
        match ev.status {
            WireStatus::Error | WireStatus::Timeout => l.failed_calls += 1,
            WireStatus::Blocked => l.blocked_calls += 1,
            WireStatus::Ok => {}
        }
        l.bytes_tx = l.bytes_tx.saturating_add(ev.bytes_tx);
        l.bytes_rx = l.bytes_rx.saturating_add(ev.bytes_rx);
        if l.events.len() >= MAX_MEMORY_EVENTS {
            l.events.pop_front();
        }
        persist_cb = l.persist.clone();
        l.events.push_back(ev.clone());
    }
    if let Some(cb) = persist_cb {
        cb(ev);
    }
}

impl WireSpan {
    pub fn finish_ok(mut self, bytes_tx: u64, bytes_rx: u64, response_payload: &str) {
        let Some(start) = self.start.take() else { return };
        IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
        let duration_ms = self.started.elapsed().as_millis() as u64;
        commit_event(WireEvent {
            seq: self.seq,
            started_at_ms: self.started_at_ms,
            duration_ms,
            category: start.category,
            protocol: start.protocol.to_string(),
            caller: start.caller,
            target_label: start.target_label,
            remote_host: start.remote_host,
            remote_port: start.remote_port,
            via_hop: start.via_hop,
            status: WireStatus::Ok,
            bytes_tx,
            bytes_rx,
            summary: redact::scrub_payload(&start.summary),
            request_payload: cap_payload(&start.request_payload),
            response_payload: cap_payload(response_payload),
            error: None,
        });
    }

    pub fn finish_err(mut self, status: WireStatus, error: &str, response_payload: &str) {
        let Some(start) = self.start.take() else { return };
        IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
        let duration_ms = self.started.elapsed().as_millis() as u64;
        let tx = start.request_payload.len() as u64;
        let rx = response_payload.len() as u64;
        commit_event(WireEvent {
            seq: self.seq,
            started_at_ms: self.started_at_ms,
            duration_ms,
            category: start.category,
            protocol: start.protocol.to_string(),
            caller: start.caller,
            target_label: start.target_label,
            remote_host: start.remote_host,
            remote_port: start.remote_port,
            via_hop: start.via_hop,
            status,
            bytes_tx: tx,
            bytes_rx: rx,
            summary: redact::scrub_payload(&start.summary),
            request_payload: cap_payload(&start.request_payload),
            response_payload: cap_payload(response_payload),
            error: Some(redact::scrub_payload(error)),
        });
    }
}

impl Drop for WireSpan {
    fn drop(&mut self) {
        if let Some(start) = self.start.take() {
            IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
            let duration_ms = self.started.elapsed().as_millis() as u64;
            commit_event(WireEvent {
                seq: self.seq,
                started_at_ms: self.started_at_ms,
                duration_ms,
                category: start.category,
                protocol: start.protocol.to_string(),
                caller: start.caller,
                target_label: start.target_label,
                remote_host: start.remote_host,
                remote_port: start.remote_port,
                via_hop: start.via_hop,
                status: WireStatus::Error,
                bytes_tx: start.request_payload.len() as u64,
                bytes_rx: 0,
                summary: redact::scrub_payload(&start.summary),
                request_payload: cap_payload(&start.request_payload),
                response_payload: String::new(),
                error: Some("cancelled / dropped before completion".into()),
            });
        }
    }
}

/// Fast summary counters for the bottom status bar and telemetry cards.
pub fn counters() -> WireCounters {
    let silenced = is_silenced();
    let in_flight = IN_FLIGHT.load(Ordering::Relaxed);
    let Ok(l) = ledger().lock() else {
        return WireCounters { in_flight, silenced, ..Default::default() };
    };
    WireCounters {
        total_calls: l.total_calls,
        in_flight,
        failed_calls: l.failed_calls,
        blocked_calls: l.blocked_calls,
        bytes_tx: l.bytes_tx,
        bytes_rx: l.bytes_rx,
        silenced,
    }
}

/// Clones the current event ring buffer (newest first).
pub fn recent_events() -> Vec<WireEvent> {
    let Ok(l) = ledger().lock() else { return Vec::new() };
    l.events.iter().rev().cloned().collect()
}

/// Scans `/proc` for all live sockets in Crow's process tree and attributes
/// them against the current Wire ledger (`ERR-212`).
pub fn live_attributed_sockets() -> Vec<proc_net::AttributedSocket> {
    let events = recent_events();
    proc_net::scan_live_sockets(&events)
}

/// Clears the in-memory ring buffer and resets session counters.
pub fn clear_events() {
    if let Ok(mut l) = ledger().lock() {
        l.events.clear();
        l.total_calls = 0;
        l.failed_calls = 0;
        l.blocked_calls = 0;
        l.bytes_tx = 0;
        l.bytes_rx = 0;
    }
}

/// Serializes `events` to newline-delimited JSON (JSONL) for external audit tools.
pub fn format_jsonl(events: &[WireEvent]) -> String {
    let mut out = String::new();
    for ev in events {
        if let Ok(line) = serde_json::to_string(ev) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// Extracts `(host, port)` from an HTTP/HTTPS URL.
pub fn parse_url_endpoint(url: &str) -> (String, u16) {
    let (h, p, _) = parse_url_host_port(url);
    (h, p)
}

/// Extracts `(host, port, proto)` from an HTTP/HTTPS URL.
pub fn parse_url_host_port(url: &str) -> (String, u16, &'static str) {
    let trimmed = url.trim();
    let (proto, rest, default_port) = if let Some(r) = trimmed.strip_prefix("https://") {
        ("HTTPS", r, 443u16)
    } else if let Some(r) = trimmed.strip_prefix("http://") {
        ("HTTP", r, 80u16)
    } else {
        ("HTTPS", trimmed, 443u16)
    } ;
    let authority = rest.split('/').next().unwrap_or(rest);
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    if let Some((h, p)) = host_port.rsplit_once(':') {
        if let Ok(port) = p.parse::<u16>() {
            return (h.to_string(), port, proto);
        }
    }
    (host_port.to_string(), default_port, proto)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_stack_nests_and_restores() {
        assert_eq!(current_caller("default"), "default");
        with_caller("overview.poll", || {
            assert_eq!(current_caller("default"), "overview.poll");
            with_caller("dokku.scan", || {
                assert_eq!(current_caller("default"), "dokku.scan");
            });
            assert_eq!(current_caller("default"), "overview.poll");
        });
        assert_eq!(current_caller("default"), "default");
    }

    #[test]
    fn parses_url_host_and_port() {
        assert_eq!(
            parse_url_endpoint("https://api.osv.dev/v1/querybatch"),
            ("api.osv.dev".into(), 443)
        );
        assert_eq!(
            parse_url_endpoint("http://example.com:8080/health"),
            ("example.com".into(), 8080)
        );
    }
}
