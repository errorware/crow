//! Email notifications (ERR-139): alerts mailed through the user's own SMTP
//! server. A critical alert goes out at once; warnings wait for one digest
//! (hourly by default); a resolved alert sends nothing. Each policy can be
//! changed. Mail only leaves while Crow is running: there's no agent.
//!
//! The dispatcher is pure: `plan` turns the open alerts into mail in an
//! outbox kept in the vault, and the sender drains the outbox, so a mail that
//! fails to send is retried rather than lost.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::metrics::alerts::Alert;

/// The vault flag that holds the settings (the password is an entry).
pub const SETTINGS_FLAG: &str = "notify.email.settings";
/// The vault flag that holds what's been mailed, the digest queue and the outbox.
pub const STATE_FLAG: &str = "notify.email.state";
/// The vault entry that holds the SMTP password, encrypted.
pub const PASSWORD_ENTRY: &str = "notify.email.password";
pub const PASSWORD_CATEGORY: &str = "notify";

/// How long a failed send waits before the outbox is tried again.
pub const RETRY_SECS: i64 = 300;
/// Mail older than this is dropped from the outbox unsent.
pub const OUTBOX_KEEP_SECS: i64 = 86_400;
pub const OUTBOX_MAX: usize = 50;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Security {
    /// TLS from the first byte (usually port 465).
    #[default]
    Tls,
    /// Plain, upgraded with STARTTLS; refused if the server can't (usually 587).
    StartTls,
    /// No encryption: only for a relay on this machine or your network.
    None,
}

impl Security {
    pub const ALL: [Security; 3] = [Security::Tls, Security::StartTls, Security::None];

    pub fn label(self) -> &'static str {
        match self {
            Security::Tls => "TLS",
            Security::StartTls => "STARTTLS",
            Security::None => "NONE",
        }
    }

    pub fn default_port(self) -> u16 {
        match self {
            Security::Tls => 465,
            Security::StartTls => 587,
            Security::None => 25,
        }
    }
}

/// When alerts of one level are mailed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delivery {
    Now,
    Digest,
    Off,
}

impl Delivery {
    pub const ALL: [Delivery; 3] = [Delivery::Now, Delivery::Digest, Delivery::Off];

    pub fn label(self) -> &'static str {
        match self {
            Delivery::Now => "AT ONCE",
            Delivery::Digest => "IN THE DIGEST",
            Delivery::Off => "NEVER",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmailSettings {
    pub host: String,
    pub port: u16,
    pub security: Security,
    /// Blank: the server takes mail without logging in.
    pub username: String,
    pub from: String,
    pub to: Vec<String>,
    pub crit: Delivery,
    pub warn: Delivery,
    pub digest_minutes: u32,
    /// Local hours (start, end) when digests wait; critical mail never does.
    pub quiet_hours: Option<(u8, u8)>,
}

impl Default for EmailSettings {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: Security::Tls.default_port(),
            security: Security::Tls,
            username: String::new(),
            from: String::new(),
            to: Vec::new(),
            crit: Delivery::Now,
            warn: Delivery::Digest,
            digest_minutes: 60,
            quiet_hours: None,
        }
    }
}

fn valid_address(a: &str) -> bool {
    a.parse::<lettre::Address>().is_ok()
}

impl EmailSettings {
    /// Why these settings can't send, if they can't.
    pub fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty() {
            return Err("the SMTP server is missing".into());
        }
        if self.host.chars().any(|c| c.is_whitespace() || c == '/' || c == ':') {
            return Err("the SMTP server is a host name only, like smtp.example.com".into());
        }
        if self.port == 0 {
            return Err("the port is missing".into());
        }
        if !valid_address(&self.from) {
            return Err(format!("\"{}\" isn't an email address (From)", self.from));
        }
        if self.to.is_empty() {
            return Err("no one to send to".into());
        }
        if let Some(bad) = self.to.iter().find(|a| !valid_address(a)) {
            return Err(format!("\"{bad}\" isn't an email address (To)"));
        }
        if !(5..=1440).contains(&self.digest_minutes) {
            return Err("the digest goes out every 5 to 1440 minutes".into());
        }
        if let Some((a, b)) = self.quiet_hours {
            if a > 23 || b > 23 || a == b {
                return Err("quiet hours are two different hours, 0 to 23".into());
            }
        }
        Ok(())
    }

    /// Whether `hour` (local, 0–23) is inside the quiet hours.
    pub fn is_quiet(&self, hour: u8) -> bool {
        match self.quiet_hours {
            None => false,
            Some((a, b)) if a < b => (a..b).contains(&hour),
            Some((a, b)) => hour >= a || hour < b,
        }
    }
}

/// "22-7", "22:00-07:00" or blank; `Err` says what's wrong.
pub fn parse_quiet_hours(s: &str) -> Result<Option<(u8, u8)>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let hour = |p: &str| -> Option<u8> {
        let p = p.trim();
        let h = p.strip_suffix(":00").unwrap_or(p);
        h.parse::<u8>().ok().filter(|h| *h < 24)
    };
    match s.split_once('-') {
        Some((a, b)) => match (hour(a), hour(b)) {
            (Some(a), Some(b)) if a != b => Ok(Some((a, b))),
            _ => Err("quiet hours look like 22-07 (from 22:00 to 07:00)".into()),
        },
        None => Err("quiet hours look like 22-07 (from 22:00 to 07:00)".into()),
    }
}

pub fn format_quiet_hours(q: Option<(u8, u8)>) -> String {
    q.map(|(a, b)| format!("{a:02}-{b:02}")).unwrap_or_default()
}

/// A comma, space or semicolon separated list of addresses.
pub fn parse_recipients(s: &str) -> Vec<String> {
    s.split([',', ';', ' ', '\n']).map(str::trim).filter(|a| !a.is_empty()).map(String::from).collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mail {
    pub subject: String,
    pub body: String,
    pub created_at: i64,
}

/// A warning waiting for the digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Queued {
    pub alert_id: String,
    pub server: String,
    pub level: String,
    pub detail: String,
    pub opened_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DispatchState {
    /// Set on the first plan: alerts already open then aren't news.
    pub primed: bool,
    /// The open alerts already handled, and the level they were handled at.
    pub seen: BTreeMap<String, String>,
    pub queue: Vec<Queued>,
    pub last_digest: i64,
    pub outbox: Vec<Mail>,
    pub last_attempt: i64,
    pub last_sent: Option<i64>,
    pub last_error: Option<String>,
}

/// What `plan` needs to name a server: its name and address.
pub type ServerNames = HashMap<String, (String, String)>;

fn server_label(names: &ServerNames, id: &str) -> String {
    match names.get(id) {
        Some((name, host)) if !host.is_empty() && host != name => format!("{name} ({host})"),
        Some((name, _)) => name.clone(),
        None => id.to_string(),
    }
}

fn short_name(names: &ServerNames, id: &str) -> String {
    names.get(id).map(|(n, _)| n.clone()).unwrap_or_else(|| id.to_string())
}

/// A local time for a mail body.
fn when(ts: i64) -> String {
    use chrono::TimeZone;
    chrono::Local.timestamp_opt(ts, 0).single().map(|t| t.format("%Y-%m-%d %H:%M %Z").to_string()).unwrap_or_default()
}

fn kind_label(kind: &str) -> String {
    match kind {
        "unreachable" => "unreachable".into(),
        "disk" => "disk".into(),
        "host_key_age" => "host key age".into(),
        k => k.strip_prefix("service:").map(|u| format!("service {u}")).unwrap_or_else(|| k.replace('_', " ")),
    }
}

/// The mail for one alert, sent at once.
pub fn alert_mail(a: &Alert, names: &ServerNames, now: i64) -> Mail {
    Mail {
        subject: format!("[Crow] {} {}: {}", a.level, short_name(names, &a.server_id), a.detail),
        body: format!(
            "{level} on {server}\n\n  {detail}\n\nWhat:   {kind}\nSince:  {since}\n\nOpen Crow to look into it or acknowledge it.\n\n-- \nSent by Crow, because email notifications are on (Settings > Plugins > Email).\n",
            level = a.level,
            server = server_label(names, &a.server_id),
            detail = a.detail,
            kind = kind_label(&a.kind),
            since = when(a.opened_at),
        ),
        created_at: now,
    }
}

/// The digest of the queued alerts; `open` says which still hold.
pub fn digest_mail(queue: &[Queued], open: &[Alert], names: &ServerNames, now: i64) -> Mail {
    let still = |q: &Queued| open.iter().any(|a| a.id == q.alert_id && a.resolved_at.is_none());
    let holding = queue.iter().filter(|q| still(q)).count();
    let n = queue.len();
    let mut body = format!(
        "{n} {} since the last digest; {holding} still open.\n\n",
        if n == 1 { "alert" } else { "alerts" }
    );
    let mut by_server: BTreeMap<String, Vec<&Queued>> = BTreeMap::new();
    for q in queue {
        by_server.entry(server_label(names, &q.server)).or_default().push(q);
    }
    for (server, items) in by_server {
        body.push_str(&format!("{server}\n"));
        for q in items {
            let state = if still(q) { "open" } else { "cleared" };
            body.push_str(&format!("  {:<4}  {}  ({}, since {})\n", q.level, q.detail, state, when(q.opened_at)));
        }
        body.push('\n');
    }
    body.push_str("Open Crow to look into them.\n\n-- \nSent by Crow, because email notifications are on (Settings > Plugins > Email).\n");
    Mail {
        subject: format!("[Crow] Digest: {n} {} ({holding} open)", if n == 1 { "alert" } else { "alerts" }),
        body,
        created_at: now,
    }
}

pub fn test_mail(now: i64) -> Mail {
    Mail {
        subject: "[Crow] Test: email notifications work".into(),
        body: format!(
            "This is a test from Crow, sent {}.\n\nIf you're reading it, critical alerts and digests will reach you here.\n\n-- \nSent by Crow (Settings > Plugins > Email).\n",
            when(now)
        ),
        created_at: now,
    }
}

/// Folds the open alerts into the state: new or escalated alerts become mail
/// or join the digest queue, a digest is written when it's due, and alerts
/// no longer open are forgotten. `hour` is the local hour, for quiet hours.
pub fn plan(state: &mut DispatchState, alerts: &[Alert], names: &ServerNames, settings: &EmailSettings, now: i64, hour: u8) {
    let open: Vec<&Alert> = alerts.iter().filter(|a| a.resolved_at.is_none()).collect();
    if !state.primed {
        state.seen = open.iter().map(|a| (a.id.clone(), a.level.clone())).collect();
        state.primed = true;
        state.last_digest = now;
        return;
    }
    for a in &open {
        let before = state.seen.get(&a.id).cloned();
        if before.as_deref() == Some(a.level.as_str()) {
            continue;
        }
        state.seen.insert(a.id.clone(), a.level.clone());
        // Acknowledged: someone already knows. Downgraded: not news.
        if a.acknowledged_at.is_some() || (before.as_deref() == Some("CRIT") && a.level != "CRIT") {
            continue;
        }
        let delivery = if a.level == "CRIT" { settings.crit } else { settings.warn };
        match delivery {
            Delivery::Now => state.outbox.push(alert_mail(a, names, now)),
            Delivery::Digest => {
                state.queue.retain(|q| q.alert_id != a.id);
                state.queue.push(Queued { alert_id: a.id.clone(), server: a.server_id.clone(), level: a.level.clone(), detail: a.detail.clone(), opened_at: a.opened_at });
            }
            Delivery::Off => {}
        }
    }
    state.seen.retain(|id, _| open.iter().any(|a| a.id == *id));
    let due = now - state.last_digest >= i64::from(settings.digest_minutes) * 60;
    if due && !settings.is_quiet(hour) {
        if !state.queue.is_empty() {
            let digest = digest_mail(&state.queue, alerts, names, now);
            state.outbox.push(digest);
            state.queue.clear();
        }
        state.last_digest = now;
    }
    state.outbox.retain(|m| now - m.created_at < OUTBOX_KEEP_SECS);
    let over = state.outbox.len().saturating_sub(OUTBOX_MAX);
    state.outbox.drain(..over);
}

/// Whether the outbox should be tried now.
pub fn send_due(state: &DispatchState, now: i64) -> bool {
    !state.outbox.is_empty() && (state.last_error.is_none() || now - state.last_attempt >= RETRY_SECS)
}

/// Strips what could end a header early.
fn one_line(s: &str) -> String {
    s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>().trim().to_string()
}

pub fn build_message(settings: &EmailSettings, mail: &Mail) -> Result<lettre::Message, String> {
    use lettre::message::{header::ContentType, Mailbox};
    let from: Mailbox = format!("Crow <{}>", settings.from.trim()).parse().map_err(|e| format!("From: {e}"))?;
    let mut builder = lettre::Message::builder().from(from).subject(one_line(&mail.subject)).header(ContentType::TEXT_PLAIN);
    for to in &settings.to {
        builder = builder.to(to.parse().map_err(|e| format!("To {to}: {e}"))?);
    }
    builder.body(mail.body.clone()).map_err(|e| e.to_string())
}

/// Plain words for what went wrong talking to the server.
fn explain(e: &lettre::transport::smtp::Error, settings: &EmailSettings) -> String {
    let text = e.to_string();
    let mut chain = text.clone();
    let mut src = std::error::Error::source(e);
    while let Some(s) = src {
        let more = s.to_string();
        if !chain.contains(&more) {
            chain.push_str(": ");
            chain.push_str(&more);
        }
        src = s.source();
    }
    let lower = chain.to_lowercase();
    if lower.contains("certificate") || lower.contains("invalidcertificate") {
        format!("{}'s TLS certificate isn't trusted ({chain})", settings.host)
    } else if lower.contains("connection refused") {
        format!("{}:{} refused the connection: is that the right port?", settings.host, settings.port)
    } else if lower.contains("timed out") || lower.contains("timeout") {
        format!("no answer from {}:{} (a firewall, or the wrong port for {})", settings.host, settings.port, settings.security.label())
    } else if lower.contains("failed to lookup") || lower.contains("name or service not known") || lower.contains("dns") {
        format!("can't find {}: check the server name", settings.host)
    } else if e.is_permanent() && (lower.contains("auth") || lower.contains("535") || lower.contains("credentials")) {
        format!("the server turned the login down ({chain})")
    } else if lower.contains("corrupt message") || lower.contains("received fatal alert") || lower.contains("wrong version") {
        format!("the TLS handshake failed: port {} on {} may want {} instead", settings.port, settings.host, if settings.security == Security::Tls { "STARTTLS (or NONE)" } else { "TLS" })
    } else {
        chain
    }
}

/// Sends each mail; returns how many went out before the first failure,
/// and that failure. Blocking: run it off the UI thread.
pub fn send(settings: &EmailSettings, password: Option<&str>, mails: &[Mail]) -> (usize, Option<String>) {
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{SmtpTransport, Transport};
    if let Err(e) = settings.validate() {
        return (0, Some(e));
    }
    let host = settings.host.trim();
    let builder = match settings.security {
        Security::Tls => SmtpTransport::relay(host),
        Security::StartTls => SmtpTransport::starttls_relay(host),
        Security::None => Ok(SmtpTransport::builder_dangerous(host)),
    };
    let mut builder = match builder {
        Ok(b) => b.port(settings.port).timeout(Some(Duration::from_secs(20))),
        Err(e) => return (0, Some(explain(&e, settings))),
    };
    if !settings.username.trim().is_empty() {
        builder = builder.credentials(Credentials::new(settings.username.trim().to_string(), password.unwrap_or_default().to_string()));
    }
    let transport = builder.build();
    let mut sent = 0;
    for mail in mails {
        let message = match build_message(settings, mail) {
            Ok(m) => m,
            Err(e) => return (sent, Some(e)),
        };
        if let Err(e) = transport.send(&message) {
            return (sent, Some(explain(&e, settings)));
        }
        sent += 1;
    }
    (sent, None)
}

#[cfg(test)]
mod tests {
    use super::{build_message, parse_quiet_hours, parse_recipients, plan, send_due, Delivery, DispatchState, EmailSettings, Mail, ServerNames, OUTBOX_MAX, RETRY_SECS};
    use crate::metrics::alerts::Alert;

    fn settings() -> EmailSettings {
        EmailSettings { host: "smtp.example.com".into(), from: "crow@example.com".into(), to: vec!["ops@example.com".into()], ..Default::default() }
    }

    fn alert(id: &str, level: &str) -> Alert {
        Alert { id: id.into(), server_id: "s1".into(), kind: "disk".into(), level: level.into(), detail: format!("{id} detail"), opened_at: 0, last_seen: 0, ..Default::default() }
    }

    fn names() -> ServerNames {
        [("s1".to_string(), ("web-01".to_string(), "10.0.0.5".to_string()))].into()
    }

    fn primed(alerts: &[Alert]) -> DispatchState {
        let mut st = DispatchState::default();
        plan(&mut st, alerts, &names(), &settings(), 1_000, 12);
        st
    }

    #[test]
    fn alerts_open_when_turned_on_are_not_news() {
        let st = primed(&[alert("a", "CRIT"), alert("b", "WARN")]);
        assert!(st.primed && st.outbox.is_empty() && st.queue.is_empty());
        assert_eq!(st.seen.len(), 2);
    }

    #[test]
    fn crit_goes_now_and_warn_waits_for_the_digest() {
        let mut st = primed(&[]);
        let open = [alert("c", "CRIT"), alert("w", "WARN")];
        plan(&mut st, &open, &names(), &settings(), 1_060, 12);
        assert_eq!(st.outbox.len(), 1);
        assert!(st.outbox[0].subject.starts_with("[Crow] CRIT web-01: c detail"), "{}", st.outbox[0].subject);
        assert!(st.outbox[0].body.contains("web-01 (10.0.0.5)"));
        assert_eq!(st.queue.len(), 1);
        // The same alerts on the next tick: nothing new.
        plan(&mut st, &open, &names(), &settings(), 1_120, 12);
        assert_eq!((st.outbox.len(), st.queue.len()), (1, 1));
        // An hour after priming the digest goes out, listing the warning.
        plan(&mut st, &open, &names(), &settings(), 1_000 + 3_600, 12);
        assert_eq!(st.outbox.len(), 2);
        let digest = &st.outbox[1];
        assert!(digest.subject.contains("Digest: 1 alert (1 open)"), "{}", digest.subject);
        assert!(digest.body.contains("w detail") && digest.body.contains("open"));
        assert!(st.queue.is_empty());
    }

    #[test]
    fn a_warning_that_turns_critical_is_mailed_and_resolved_ones_send_nothing() {
        let mut st = primed(&[alert("d", "WARN")]);
        plan(&mut st, &[alert("d", "CRIT")], &names(), &settings(), 1_060, 12);
        assert_eq!(st.outbox.len(), 1, "escalated");
        // Back down to WARN: not news. Then resolved: nothing, and forgotten.
        plan(&mut st, &[alert("d", "WARN")], &names(), &settings(), 1_120, 12);
        let resolved = Alert { resolved_at: Some(1_180), ..alert("d", "WARN") };
        plan(&mut st, &[resolved], &names(), &settings(), 1_180, 12);
        assert_eq!(st.outbox.len(), 1);
        assert!(st.seen.is_empty() && st.queue.is_empty());
    }

    #[test]
    fn acknowledged_and_off_alerts_are_not_mailed() {
        let mut st = primed(&[]);
        let acked = Alert { acknowledged_at: Some(1), ..alert("a", "CRIT") };
        let off = EmailSettings { warn: Delivery::Off, ..settings() };
        plan(&mut st, &[acked, alert("w", "WARN")], &names(), &off, 1_060, 12);
        assert!(st.outbox.is_empty() && st.queue.is_empty());
    }

    #[test]
    fn quiet_hours_hold_the_digest_but_not_critical_mail() {
        let quiet = EmailSettings { quiet_hours: Some((22, 7)), ..settings() };
        assert!(quiet.is_quiet(23) && quiet.is_quiet(3) && !quiet.is_quiet(7) && !quiet.is_quiet(12));
        let mut st = DispatchState::default();
        plan(&mut st, &[], &names(), &quiet, 0, 23);
        plan(&mut st, &[alert("w", "WARN"), alert("c", "CRIT")], &names(), &quiet, 7_200, 23);
        assert_eq!(st.outbox.len(), 1, "the CRIT, not the digest");
        assert_eq!(st.queue.len(), 1);
        plan(&mut st, &[alert("w", "WARN"), alert("c", "CRIT")], &names(), &quiet, 7_260, 7);
        assert_eq!(st.outbox.len(), 2, "the digest once quiet hours end");
    }

    #[test]
    fn the_outbox_is_bounded_and_retried_after_a_pause() {
        let mut st = primed(&[]);
        let many: Vec<_> = (0..OUTBOX_MAX + 10).map(|i| alert(&format!("c{i}"), "CRIT")).collect();
        plan(&mut st, &many, &names(), &settings(), 1_060, 12);
        assert_eq!(st.outbox.len(), OUTBOX_MAX);
        assert!(send_due(&st, 1_060));
        st.last_error = Some("refused".into());
        st.last_attempt = 1_060;
        assert!(!send_due(&st, 1_060 + RETRY_SECS - 1));
        assert!(send_due(&st, 1_060 + RETRY_SECS));
        // A day later, what never went out is dropped.
        plan(&mut st, &many, &names(), &settings(), 1_060 + 86_400, 12);
        assert!(st.outbox.is_empty());
    }

    #[test]
    fn settings_say_what_is_wrong() {
        assert!(settings().validate().is_ok());
        assert!(EmailSettings { host: "smtp://x".into(), ..settings() }.validate().unwrap_err().contains("host name"));
        assert!(EmailSettings { from: "nope".into(), ..settings() }.validate().unwrap_err().contains("From"));
        assert!(EmailSettings { to: vec![], ..settings() }.validate().unwrap_err().contains("no one"));
        assert!(EmailSettings { to: vec!["a@b.c".into(), "bad".into()], ..settings() }.validate().unwrap_err().contains("\"bad\""));
        assert_eq!(parse_recipients("a@b.c, d@e.f;g@h.i"), vec!["a@b.c", "d@e.f", "g@h.i"]);
        assert_eq!(parse_quiet_hours("22-07"), Ok(Some((22, 7))));
        assert_eq!(parse_quiet_hours(" 22:00 - 7:00 "), Ok(Some((22, 7))));
        assert_eq!(parse_quiet_hours(""), Ok(None));
        assert!(parse_quiet_hours("25-3").is_err() && parse_quiet_hours("9").is_err() && parse_quiet_hours("8-8").is_err());
    }

    #[test]
    fn a_subject_cannot_add_headers() {
        let mail = Mail { subject: "disk full\r\nBcc: evil@example.com".into(), body: "b".into(), created_at: 0 };
        let raw = String::from_utf8(build_message(&settings(), &mail).unwrap().formatted()).unwrap();
        assert!(!raw.lines().any(|l| l.starts_with("Bcc:")), "{raw}");
        assert!(raw.contains("From: Crow <crow@example.com>"));
    }
}

/// Sends a test mail through a real server. With a throwaway mailpit:
///   podman run -d --rm -p 127.0.0.1:1025:1025 -p 127.0.0.1:8025:8025 axllent/mailpit
///   CROW_SMTP=127.0.0.1:1025 cargo test live_email_send -- --ignored --nocapture
#[cfg(test)]
#[test]
#[ignore]
fn live_email_send() {
    let target = std::env::var("CROW_SMTP").unwrap_or_else(|_| "127.0.0.1:1025".into());
    let (host, port) = target.rsplit_once(':').unwrap();
    let settings = EmailSettings {
        host: host.into(),
        port: port.parse().unwrap(),
        security: Security::None,
        from: "crow@example.com".into(),
        to: vec!["ops@example.com".into(), "oncall@example.com".into()],
        ..Default::default()
    };
    let (sent, err) = send(&settings, None, &[test_mail(chrono::Utc::now().timestamp())]);
    eprintln!("sent {sent}, error {err:?}");
    assert_eq!((sent, err), (1, None));
    // TLS against a plain server: the error must be readable.
    let tls = EmailSettings { security: Security::Tls, ..settings };
    eprintln!("TLS against plain: {:?}", send(&tls, None, &[test_mail(0)]).1);
}
