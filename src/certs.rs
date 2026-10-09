//! Certificate renewal (ERR-146): what renews a server's certificates
//! (certbot or acme.sh), whether its automatic renewal works, and a
//! renewal from Crow: dry run, renew, reload the web server, confirm.

use crate::host::{Host, HostError, DEFAULT_TIMEOUT};

/// Renewals talk to the CA: give them time.
pub const RENEW_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// One round trip: the tools, certbot's certificates, the automatic
/// renewal's state and its last log lines.
pub const INVENTORY: &str = r#"PATH="$PATH:/usr/sbin:/sbin:/snap/bin"
echo "@@tools"
command -v certbot >/dev/null 2>&1 && echo certbot
for a in /root/.acme.sh/acme.sh "$HOME/.acme.sh/acme.sh"; do [ -x "$a" ] && { echo "acme.sh $a"; break; }; done
command -v nginx >/dev/null 2>&1 && echo nginx
{ command -v apachectl || command -v apache2ctl; } >/dev/null 2>&1 && echo apache
echo "@@certificates"
command -v certbot >/dev/null 2>&1 && certbot certificates 2>/dev/null
echo "@@timer"
for t in certbot.timer snap.certbot.renew.timer; do
  if systemctl list-unit-files "$t" >/dev/null 2>&1 && systemctl list-unit-files "$t" 2>/dev/null | grep -q "$t"; then
    echo "$t $(systemctl is-enabled "$t" 2>/dev/null) $(systemctl is-active "$t" 2>/dev/null)"
  fi
done
[ -f /etc/cron.d/certbot ] && echo "cron /etc/cron.d/certbot"
crontab -l 2>/dev/null | grep -q 'acme.sh' && echo "cron acme.sh"
echo "@@log"
tail -n 200 /var/log/letsencrypt/letsencrypt.log 2>/dev/null | grep -E 'ERROR|Failed|failed|Congratulations|No renewals were attempted|not due' | tail -n 3
true"#;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManagedCert {
    /// certbot's certificate name.
    pub name: String,
    pub domains: Vec<String>,
    /// "2026-12-01 10:00:00+00:00 (VALID: 52 days)" as certbot prints it.
    pub expiry: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    pub certbot: bool,
    /// The acme.sh script's path, when it's there.
    pub acme_sh: Option<String>,
    pub nginx: bool,
    pub apache: bool,
    pub certs: Vec<ManagedCert>,
    /// What renews automatically: "certbot.timer enabled active", "cron …".
    pub auto: Vec<String>,
    /// certbot's last log lines about renewals.
    pub log: Vec<String>,
}

impl Inventory {
    /// Why automatic renewal won't happen, if it won't.
    pub fn auto_renew_problem(&self) -> Option<String> {
        if !self.certbot && self.acme_sh.is_none() {
            return None;
        }
        if self.certbot && !self.certs.is_empty() {
            let timer_ok = self.auto.iter().any(|a| a.contains(".timer") && a.ends_with(" active") && !a.contains(" disabled "));
            let cron_ok = self.auto.iter().any(|a| a.starts_with("cron /etc/cron.d/certbot"));
            if !timer_ok && !cron_ok {
                let state = self.auto.iter().find(|a| a.contains(".timer")).cloned();
                return Some(match state {
                    Some(t) => format!("certbot's timer isn't running ({t}): certificates won't renew on their own"),
                    None => "certbot is installed but nothing runs it (no timer, no cron): certificates won't renew on their own".into(),
                });
            }
            if let Some(last) = self.log.last() {
                if last.contains("ERROR") || last.to_lowercase().contains("failed") {
                    return Some(format!("certbot's last renewal failed: {last}"));
                }
            }
        }
        if self.acme_sh.is_some() && !self.certbot && !self.auto.iter().any(|a| a == "cron acme.sh") {
            return Some("acme.sh is installed but its cron job is missing: certificates won't renew on their own".into());
        }
        None
    }
}

/// Parses `certbot certificates`.
pub fn parse_certbot_certificates(text: &str) -> Vec<ManagedCert> {
    let mut out: Vec<ManagedCert> = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(v) = line.strip_prefix("Certificate Name:") {
            out.push(ManagedCert { name: v.trim().to_string(), ..Default::default() });
        } else if let Some(c) = out.last_mut() {
            if let Some(v) = line.strip_prefix("Domains:") {
                c.domains = v.split_whitespace().map(String::from).collect();
            } else if let Some(v) = line.strip_prefix("Expiry Date:") {
                c.expiry = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("Certificate Path:") {
                c.path = v.trim().to_string();
            }
        }
    }
    out
}

pub fn parse_inventory(out: &str) -> Inventory {
    let mut inv = Inventory::default();
    let mut section = "";
    let mut certs_text = String::new();
    for line in out.lines() {
        if let Some(s) = line.strip_prefix("@@") {
            section = match s.trim() {
                "tools" => "tools",
                "certificates" => "certificates",
                "timer" => "timer",
                _ => "log",
            };
            continue;
        }
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        match section {
            "tools" => match l.split_once(' ') {
                Some(("acme.sh", path)) => inv.acme_sh = Some(path.to_string()),
                _ if l == "certbot" => inv.certbot = true,
                _ if l == "nginx" => inv.nginx = true,
                _ if l == "apache" => inv.apache = true,
                _ => {}
            },
            "certificates" => {
                certs_text.push_str(line);
                certs_text.push('\n');
            }
            "timer" => inv.auto.push(l.to_string()),
            _ => inv.log.push(l.to_string()),
        }
    }
    inv.certs = parse_certbot_certificates(&certs_text);
    inv
}

pub fn read_inventory(host: &dyn Host) -> Result<Inventory, String> {
    let argv = ["sh", "-c", INVENTORY];
    host.exec_privileged(&argv, &[], DEFAULT_TIMEOUT).or_else(|_| host.exec(&argv, DEFAULT_TIMEOUT)).map(|o| parse_inventory(&o.stdout)).map_err(|e| e.to_string())
}

fn root(host: &dyn Host, script: &str, timeout: std::time::Duration) -> Result<String, String> {
    let full = format!("PATH=\"$PATH:/usr/sbin:/sbin:/snap/bin\"; ( {script} ) 2>&1");
    host.exec_privileged(&["sh", "-c", &full], &[], timeout).map(|o| o.stdout).map_err(|e| match e {
        HostError::Failed { stderr, status } => format!("exit {status}: {}", stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim()),
        other => other.to_string(),
    })
}

fn tail(text: &str) -> String {
    text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim().to_string()
}

/// Which certificate a renewal is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Renewal {
    Certbot { name: String },
    AcmeSh { script: String, domain: String },
}

/// Renews one certificate, step by step (the log says what happened):
/// certbot's dry run first, the renewal, the web servers' config checks
/// and reloads, then `confirm` re-reads the certificate's expiry.
pub fn renew(host: &dyn Host, what: &Renewal, nginx: bool, apache: bool, confirm: &dyn Fn() -> Option<String>) -> Result<Vec<String>, Vec<String>> {
    let mut log = Vec::new();
    match what {
        Renewal::Certbot { name } => {
            if name.is_empty() || name.starts_with('-') || name.contains(char::is_whitespace) {
                return Err(vec![format!("✕ {name:?} isn't a certificate name.")]);
            }
            let q = crate::host::ssh::shell_quote(name);
            match root(host, &format!("certbot renew --cert-name {q} --dry-run --non-interactive"), RENEW_TIMEOUT) {
                Ok(out) => log.push(format!("✓ Dry run passed: {}", tail(&out))),
                Err(e) => {
                    log.push(format!("✕ The dry run failed, so nothing was renewed: {e}"));
                    return Err(log);
                }
            }
            match root(host, &format!("certbot renew --cert-name {q} --non-interactive"), RENEW_TIMEOUT) {
                Ok(out) if out.contains("not due for renewal") || out.contains("not yet due") => {
                    log.push("= Not due yet: certbot renews within 30 days of expiry, and Let's Encrypt limits how often a certificate can be issued.".into());
                    return Ok(log);
                }
                Ok(out) => log.push(format!("✓ Renewed: {}", tail(&out))),
                Err(e) => {
                    log.push(format!("✕ Renewal failed: {e}"));
                    return Err(log);
                }
            }
        }
        Renewal::AcmeSh { script, domain } => {
            if domain.is_empty() || domain.starts_with('-') || domain.contains(char::is_whitespace) {
                return Err(vec![format!("✕ {domain:?} isn't a domain.")]);
            }
            log.push("· acme.sh has no dry run: renewing directly.".into());
            match root(host, &format!("{} --renew -d {}", crate::host::ssh::shell_quote(script), crate::host::ssh::shell_quote(domain)), RENEW_TIMEOUT) {
                Ok(out) if out.contains("Skip, Next renewal time") => {
                    log.push("= Not due yet (acme.sh renews 60 days after issue).".into());
                    return Ok(log);
                }
                Ok(out) => log.push(format!("✓ Renewed: {}", tail(&out))),
                Err(e) => {
                    log.push(format!("✕ Renewal failed: {e}"));
                    return Err(log);
                }
            }
        }
    }
    // The new certificate is only served after a reload; a broken config
    // must not be reloaded.
    if nginx {
        match root(host, "nginx -t && (systemctl reload nginx || nginx -s reload)", crate::host::DEFAULT_TIMEOUT) {
            Ok(_) => log.push("✓ nginx config checked and reloaded.".into()),
            Err(e) => {
                log.push(format!("✕ nginx wasn't reloaded ({e}): it still serves the old certificate until it is."));
                return Err(log);
            }
        }
    }
    if apache {
        match root(host, "(apachectl configtest || apache2ctl configtest) && (systemctl reload apache2 || systemctl reload httpd || apachectl graceful)", crate::host::DEFAULT_TIMEOUT) {
            Ok(_) => log.push("✓ Apache config checked and reloaded.".into()),
            Err(e) => {
                log.push(format!("✕ Apache wasn't reloaded ({e}): it still serves the old certificate until it is."));
                return Err(log);
            }
        }
    }
    match confirm() {
        Some(expiry) => log.push(format!("✓ The certificate now expires {expiry}.")),
        None => log.push("· Couldn't re-read the certificate to confirm its new expiry.".into()),
    }
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::{parse_inventory, Inventory};

    const CERTBOT: &str = "Found the following certs:\n  Certificate Name: example.com\n    Serial Number: 4a\n    Key Type: ECDSA\n    Domains: example.com www.example.com\n    Expiry Date: 2026-12-01 10:00:00+00:00 (VALID: 52 days)\n    Certificate Path: /etc/letsencrypt/live/example.com/fullchain.pem\n    Private Key Path: /etc/letsencrypt/live/example.com/privkey.pem\n";

    #[test]
    fn the_inventory_reads_tools_certificates_and_the_timer() {
        let out = format!("@@tools\ncertbot\nnginx\n@@certificates\n{CERTBOT}@@timer\ncertbot.timer enabled active\n@@log\n2026-10-09 Congratulations, all renewals succeeded\n");
        let inv = parse_inventory(&out);
        assert!(inv.certbot && inv.nginx && !inv.apache && inv.acme_sh.is_none());
        assert_eq!(inv.certs.len(), 1);
        assert_eq!(inv.certs[0].domains, ["example.com", "www.example.com"]);
        assert!(inv.certs[0].expiry.contains("VALID: 52 days"));
        assert_eq!(inv.auto_renew_problem(), None);
    }

    #[test]
    fn broken_auto_renewal_is_spotted() {
        let base = |auto: Vec<&str>, log: Vec<&str>| Inventory { certbot: true, certs: vec![Default::default()], auto: auto.into_iter().map(String::from).collect(), log: log.into_iter().map(String::from).collect(), ..Default::default() };
        assert!(base(vec!["certbot.timer disabled inactive"], vec![]).auto_renew_problem().unwrap().contains("timer isn't running"));
        assert!(base(vec![], vec![]).auto_renew_problem().unwrap().contains("nothing runs it"));
        assert_eq!(base(vec!["cron /etc/cron.d/certbot"], vec![]).auto_renew_problem(), None);
        assert!(base(vec!["certbot.timer enabled active"], vec!["2026-10-08 ERROR:certbot.renewal:All renewals failed"]).auto_renew_problem().unwrap().contains("last renewal failed"));
        let acme = Inventory { acme_sh: Some("/root/.acme.sh/acme.sh".into()), ..Default::default() };
        assert!(acme.auto_renew_problem().unwrap().contains("cron job is missing"));
        assert_eq!(Inventory::default().auto_renew_problem(), None, "nothing installed: nothing to say");
    }
}

/// Renews a certificate end to end in a throwaway container running nginx
/// with a stand-in certbot (it issues self-signed certificates):
///   cargo test live_cert_renewal -- --ignored --nocapture
#[cfg(test)]
#[test]
#[ignore]
fn live_cert_renewal() {
    let host = crate::host::ContainerHost::new("podman", "crow-certs");
    let inv = read_inventory(&host).expect("inventory");
    eprintln!("{inv:?}");
    assert!(inv.certbot && inv.nginx && inv.certs.len() == 1);
    eprintln!("auto-renewal: {:?}", inv.auto_renew_problem());
    let before = crate::metrics::certs::read_certs(&host).unwrap();
    let path = inv.certs[0].path.replace("fullchain.pem", "cert.pem");
    let expiry_of = |certs: &[crate::metrics::certs::CertInfo]| certs.iter().find(|c| c.path.contains("example.test")).map(|c| c.expires).unwrap();
    let confirm = || crate::metrics::certs::read_certs(&host).map(|c| chrono::DateTime::from_timestamp(expiry_of(&c), 0).unwrap().format("%Y-%m-%d").to_string());
    let log = renew(&host, &Renewal::Certbot { name: inv.certs[0].name.clone() }, inv.nginx, inv.apache, &confirm).expect("renewed");
    for l in &log {
        eprintln!("{l}");
    }
    let after = crate::metrics::certs::read_certs(&host).unwrap();
    assert!(expiry_of(&after) - expiry_of(&before) > 60 * 86_400, "{path}: the new certificate lasts ~85 days longer");
}
