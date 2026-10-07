//! TLS certificates about to expire (ERR-99): the certificates a server
//! serves, found where they usually live, their expiry read with openssl
//! over the server's own connection, and alerts from that.

use super::alerts::{Alert, AlertChanges};
use crate::host::{Host, DEFAULT_TIMEOUT};

/// Days left at which an alert opens, and at which it turns critical.
pub const CERT_WARN_DAYS: i64 = 14;
pub const CERT_CRIT_DAYS: i64 = 3;
/// How often each server's certificates are read.
pub const CERT_CHECK_EVERY_SECS: i64 = 6 * 3600;

/// One certificate on a server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertInfo {
    pub path: String,
    /// When it expires (unix seconds).
    pub expires: i64,
    /// Its common name, e.g. "example.com" (empty when it has none).
    pub name: String,
}

/// Prints `path<TAB>notAfter<TAB>subject` for Let's Encrypt's live
/// certificates and the certificate files nginx and Apache point at
/// (absolute paths, no variables), each once. Nothing without openssl.
const READ_CERTS: &str = r#"command -v openssl >/dev/null 2>&1 || exit 0
{ ls /etc/letsencrypt/live/*/cert.pem 2>/dev/null
  grep -rhE '^[[:space:]]*(ssl_certificate|SSLCertificateFile)[[:space:]]' /etc/nginx /etc/apache2 /etc/httpd 2>/dev/null | awk '{print $2}' | tr -d ';"'"'"
} | grep '^/' | grep -v '\$' | sort -u | while IFS= read -r f; do
  [ -r "$f" ] || continue
  e=$(openssl x509 -noout -enddate -in "$f" 2>/dev/null) || continue
  s=$(openssl x509 -noout -subject -nameopt RFC2253 -in "$f" 2>/dev/null)
  printf '%s\t%s\t%s\n' "$f" "${e#notAfter=}" "${s#subject=}"
done"#;

/// Parses openssl's `notAfter` ("Oct  7 12:00:00 2026 GMT").
fn parse_not_after(s: &str) -> Option<i64> {
    let s = s.trim().trim_end_matches("GMT").trim();
    chrono::NaiveDateTime::parse_from_str(s, "%b %e %H:%M:%S %Y").ok().map(|t| t.and_utc().timestamp())
}

/// The CN out of an RFC 2253 subject ("CN=example.com,O=Acme").
fn common_name(subject: &str) -> String {
    subject.split(',').find_map(|p| p.trim().strip_prefix("CN=")).unwrap_or_default().to_string()
}

fn parse(stdout: &str) -> Vec<CertInfo> {
    stdout
        .lines()
        .filter_map(|l| {
            let mut parts = l.splitn(3, '\t');
            let path = parts.next()?.to_string();
            let expires = parse_not_after(parts.next()?)?;
            Some(CertInfo { path, expires, name: common_name(parts.next().unwrap_or_default()) })
        })
        .collect()
}

/// The server's certificates, read as root when possible (Let's Encrypt
/// keeps its folder root-only), else as the login user. `None` when
/// neither could run, so alerts are left as they are.
pub fn read_certs(host: &dyn Host) -> Option<Vec<CertInfo>> {
    let argv = ["sh", "-c", READ_CERTS, "crow-certs"];
    host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT).or_else(|_| host.exec(&argv, DEFAULT_TIMEOUT)).ok().map(|o| parse(&o.stdout))
}

/// Alert changes for one server's certificates: one alert per certificate
/// file (`cert:<path>`), WARN within 14 days, CRIT within 3 or expired,
/// resolved once it's renewed or gone.
pub fn cert_changes(open: &[Alert], server: &str, certs: &[CertInfo], now: i64) -> AlertChanges {
    let mut changes = AlertChanges::default();
    let mut holding = Vec::new();
    for c in certs {
        let days = (c.expires - now).div_euclid(86_400);
        if days > CERT_WARN_DAYS {
            continue;
        }
        let level = if days <= CERT_CRIT_DAYS { "CRIT" } else { "WARN" };
        let what = if c.name.is_empty() { "certificate".to_string() } else { format!("certificate for {}", c.name) };
        let when = match days {
            d if d < 0 => format!("expired {} day{} ago", -d, if d == -1 { "" } else { "s" }),
            0 => "expires today".to_string(),
            1 => "expires tomorrow".to_string(),
            d => format!("expires in {d} days"),
        };
        let detail = format!("{what} {when} ({})", c.path);
        let kind = format!("cert:{}", c.path);
        match open.iter().find(|a| a.server_id == server && a.kind == kind && a.resolved_at.is_none()) {
            Some(a) => changes.updated.push((a.id.clone(), level.into(), detail, now)),
            None => changes.opened.push(Alert {
                id: format!("alert-{server}-{kind}-{now}"),
                server_id: server.into(),
                kind: kind.clone(),
                level: level.into(),
                detail,
                opened_at: now,
                last_seen: now,
                resolved_at: None,
                acknowledged_at: None,
            }),
        }
        holding.push(kind);
    }
    for a in open.iter().filter(|a| a.server_id == server && a.kind.starts_with("cert:") && a.resolved_at.is_none()) {
        if !holding.contains(&a.kind) {
            changes.resolved.push(a.id.clone());
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_scripts_output() {
        let out = "/etc/letsencrypt/live/example.com/cert.pem\tOct  7 12:00:00 2026 GMT\tCN=example.com\n/etc/ssl/certs/internal.pem\tJan 15 08:30:00 2027 GMT\tC=DE,O=Acme,CN=api.internal\n/etc/ssl/bad.pem\tnot a date\tCN=x\n";
        let certs = parse(out);
        assert_eq!(certs.len(), 2, "an unreadable date is skipped");
        assert_eq!(certs[0].name, "example.com");
        assert_eq!(certs[1].name, "api.internal");
        assert_eq!(certs[0].expires, chrono::NaiveDate::from_ymd_opt(2026, 10, 7).unwrap().and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp());
    }

    #[test]
    fn warn_then_crit_then_resolved_on_renewal() {
        let day = 86_400;
        let now = 10_000 * day;
        let cert = |days: i64| CertInfo { path: "/etc/letsencrypt/live/example.com/cert.pem".into(), expires: now + days * day + 3600, name: "example.com".into() };
        assert_eq!(cert_changes(&[], "s1", &[cert(30)], now), AlertChanges::default(), "a month left: nothing");
        let warn = cert_changes(&[], "s1", &[cert(9)], now);
        assert_eq!(warn.opened[0].level, "WARN");
        assert_eq!(warn.opened[0].detail, "certificate for example.com expires in 9 days (/etc/letsencrypt/live/example.com/cert.pem)");
        let crit = cert_changes(&warn.opened, "s1", &[cert(2)], now);
        assert_eq!(crit.updated[0].1, "CRIT");
        let expired = cert_changes(&warn.opened, "s1", &[cert(-3)], now);
        assert!(expired.updated[0].2.contains("expired 3 days ago"), "{}", expired.updated[0].2);
        assert_eq!(cert_changes(&warn.opened, "s1", &[cert(90)], now).resolved, vec![warn.opened[0].id.clone()], "renewed");
        assert_eq!(cert_changes(&warn.opened, "s1", &[], now).resolved.len(), 1, "the file is gone");
        assert!(cert_changes(&warn.opened, "s2", &[], now).resolved.is_empty(), "another server's alerts are untouched");
    }
}
