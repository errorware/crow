//! The audit log (ERR-75): every change record across the fleet, plus
//! enrollments, as one newest-first list. GPUI-free.

use chrono::{DateTime, Local, Utc};

use crate::vault::{ChangeRecord, ServerRecord};

/// What kind of change, for filtering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KindGroup {
    Config,
    Service,
    Process,
    Firewall,
    User,
    Power,
    Ssh,
    Enroll,
    Crow,
}

impl KindGroup {
    pub const ALL: [KindGroup; 9] = [
        KindGroup::Config,
        KindGroup::Service,
        KindGroup::Process,
        KindGroup::Firewall,
        KindGroup::User,
        KindGroup::Power,
        KindGroup::Ssh,
        KindGroup::Enroll,
        KindGroup::Crow,
    ];

    pub fn of(action_kind: &str) -> Self {
        match action_kind {
            k if k.starts_with("config.") => KindGroup::Config,
            k if k.starts_with("service") => KindGroup::Service,
            k if k.starts_with("process") => KindGroup::Process,
            "firewall" => KindGroup::Firewall,
            "user" => KindGroup::User,
            k if k.starts_with("provider.") || k == "fleet.reboot" => KindGroup::Power,
            k if k.starts_with("sshd.") || k.starts_with("ssh.") => KindGroup::Ssh,
            "enroll" => KindGroup::Enroll,
            _ => KindGroup::Crow,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            KindGroup::Config => t!("audit-kind-config"),
            KindGroup::Service => t!("audit-kind-service"),
            KindGroup::Process => t!("audit-kind-process"),
            KindGroup::Firewall => t!("audit-kind-firewall"),
            KindGroup::User => t!("audit-kind-user"),
            KindGroup::Power => t!("audit-kind-power"),
            KindGroup::Ssh => t!("audit-kind-ssh"),
            KindGroup::Enroll => t!("audit-kind-enroll"),
            KindGroup::Crow => t!("audit-kind-crow"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    Failed,
    Pending,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditItem {
    pub at: DateTime<Utc>,
    pub server_id: String,
    pub server: String,
    pub group: KindGroup,
    /// One line: what happened.
    pub text: String,
    /// The exact command, hashes, or error, when there's more to say.
    pub detail: Option<String>,
    pub outcome: Outcome,
}

/// Change records and enrollments, newest first.
#[cfg(test)]
pub fn audit_items(records: &[ChangeRecord], servers: &[ServerRecord]) -> Vec<AuditItem> {
    audit_items_by(records, servers, &Default::default())
}

/// Like [`audit_items`], each change marked with the member who made it.
pub fn audit_items_by(records: &[ChangeRecord], servers: &[ServerRecord], actors: &std::collections::HashMap<String, String>) -> Vec<AuditItem> {
    let mut items: Vec<AuditItem> = records
        .iter()
        .filter_map(|r| {
            let mut item = from_record(r)?;
            if let Some(a) = actors.get(&r.id) {
                item.text = t!("audit-model-by", text = item.text, a = a);
            }
            Some(item)
        })
        .collect();
    items.extend(servers.iter().filter_map(|s| {
        Some(AuditItem {
            at: DateTime::parse_from_rfc3339(&s.created_at).ok()?.with_timezone(&Utc),
            server_id: s.id.clone(),
            server: s.name.clone(),
            group: KindGroup::Enroll,
            text: t!("audit-model-enrolled", login_user = s.login_user, host = s.host, port = s.port),
            detail: None,
            outcome: Outcome::Done,
        })
    }));
    items.sort_by(|a, b| b.at.cmp(&a.at));
    items
}

fn from_record(r: &ChangeRecord) -> Option<AuditItem> {
    let at = DateTime::parse_from_rfc3339(&r.started_at).ok()?.with_timezone(&Utc);
    let outcome = match r.outcome.as_str() {
        "success" | "applied" | "closed" => Outcome::Done,
        "failed" => Outcome::Failed,
        _ => Outcome::Pending,
    };
    let group = KindGroup::of(&r.action_kind);
    let text = match r.action_kind.as_str() {
        "config.write" => t!("audit-model-saved", target = r.target),
        "config.restore" => t!("audit-model-restored-an-earlier-version-of", target = r.target),
        "config.baseline" => t!("audit-model-baseline-for", target = r.target, before_state = r.before_state),
        "process_kill" => t!("audit-model-killed-process", target = r.target),
        "purge" => t!("audit-model-purged-stored-data-for", target = r.target),
        "sshd.password_login_off" => t!("audit-model-turned-off-ssh-password-login").to_string(),
        "ssh.host_key" => t!("audit-model-trusted-a-new-host-key", target = r.target),
        "ssh.key_switch" => r.before_state.clone(),
        "ssh.terminal" => t!("audit-model-opened-a-terminal").to_string(),
        k if k.starts_with("service_") => format!("{} {}", k.trim_start_matches("service_"), r.target),
        k if k.starts_with("provider.") => t!("audit-model-at-the-provider", k = k.trim_start_matches("provider.")),
        "fleet.reboot" => t!("audit-model-rolling-reboot", before_state = r.before_state),
        "firewall" => t!("audit-model-ufw", target = r.target),
        "user" => t!("audit-model-user", target = r.target),
        "people.offboard" => t!("audit-model-offboarded-on", target = r.target),
        "people.revoke_key" => t!("audit-model-key-revoked-on", target = r.target),
        "people.onboard" => t!("audit-model-account-set-up-on", target = r.target),
        "cert.renew" => t!("audit-model-certificate-renewed-on", target = r.target),
        "posture.fix" => t!("audit-model-hardening-fix-on", target = r.target),
        "posture.accept" | "posture.accepted" => t!("audit-model-risk-accepted-on", target = r.target),
        "rollout.command" => t!("audit-model-command-run-on", target = r.target),
        "rollout.config" => t!("audit-model-config-rolled-out-to", target = r.target),
        "rollout.rollback" => t!("audit-model-rollout-undone-on", target = r.target),
        "patch.apply" => t!("audit-model-updates-applied-on", target = r.target),
        "ssh.host_key_rotate" => t!("audit-model-host-keys-rotated-on", target = r.target),
        "ssh.revoke_sessions" => t!("audit-model-sessions-ended-on", target = r.target),
        "config.drift_restore" => t!("audit-model-brought-back-on", target = r.target),
        "file.chmod" => t!("audit-model-permissions-changed", target = r.target),
        "plugin.accept" => t!("audit-model-plugin-accepted", target = r.target),
        "plugin.refuse" => t!("audit-model-plugin-refused", target = r.target),
        "plugin.revoke" => t!("audit-model-plugin-revoked", target = r.target),
        k if k.starts_with("plugin.") => t!("audit-model-plugin-ran", target = r.target, action = k.rsplit('.').next().unwrap_or(k)),
        k => format!("{k} {}", r.target),
    };
    // Config records carry hashes; command records carry the command, and
    // a failure's after_state is the error.
    let detail = match (outcome, &r.after_state) {
        (Outcome::Failed, Some(err)) => Some(format!("{} · {err}", r.before_state)),
        // A terminal: "terminal closed after 12m 3s" once it ended.
        (Outcome::Done, Some(closed)) if r.action_kind == "ssh.terminal" => Some(closed.clone()),
        _ if r.before_state.is_empty() => None,
        _ if group == KindGroup::Config => r.after_state.as_ref().map(|a| format!("{} → {}", short_hash(&r.before_state), short_hash(a))),
        _ => Some(r.before_state.clone()),
    };
    Some(AuditItem { at, server_id: r.server_id.clone(), server: r.server_name.clone(), group, text, detail, outcome })
}

fn short_hash(s: &str) -> String {
    match s.strip_prefix("sha256:") {
        Some(h) => format!("sha256:{}", &h[..h.len().min(10)]),
        None => s.to_string(),
    }
}

/// "20:31" for today, "Sep 28 20:31" otherwise, in local time.
pub fn when_label(at: DateTime<Utc>, now: DateTime<Local>) -> String {
    let local = at.with_timezone(&Local);
    if local.date_naive() == now.date_naive() {
        local.format("%H:%M").to_string()
    } else {
        local.format("%b %d %H:%M").to_string()
    }
}

/// The audit screen's filters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditFilter {
    pub server_id: Option<String>,
    pub group: Option<KindGroup>,
}

impl AuditFilter {
    pub fn matches(&self, item: &AuditItem) -> bool {
        self.server_id.as_ref().is_none_or(|id| *id == item.server_id) && self.group.is_none_or(|g| g == item.group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(kind: &str, target: &str, before: &str, after: Option<&str>, outcome: &str, at: &str) -> ChangeRecord {
        ChangeRecord {
            id: at.into(),
            server_id: "s1".into(),
            server_name: "web-1".into(),
            action_kind: kind.into(),
            target: target.into(),
            before_state: before.into(),
            after_state: after.map(Into::into),
            blast_radius: None,
            outcome: outcome.into(),
            started_at: at.into(),
            completed_at: None,
        }
    }

    #[test]
    fn records_and_enrollments_merge_newest_first() {
        let records = [
            rec("config.write", "/etc/ssh/sshd_config", "sha256:aaaaaaaaaaaaaaaa", Some("sha256:bbbbbbbbbbbbbbbb"), "applied", "2026-09-29T10:00:00Z"),
            rec("firewall", "allow 443/tcp", "ufw allow 443/tcp", Some("ERROR: Could not load logging rules"), "failed", "2026-09-29T12:00:00Z"),
            rec("service_restart", "nginx.service", "systemctl restart nginx.service", None, "success", "2026-09-29T11:00:00Z"),
        ];
        let servers = [ServerRecord { id: "s1".into(), name: "web-1".into(), host: "10.0.0.5".into(), port: 22, login_user: "ops".into(), created_at: "2026-09-28T09:00:00Z".into(), ..Default::default() }];
        let items = audit_items(&records, &servers);
        let summary: Vec<_> = items.iter().map(|i| (i.group, i.text.as_str(), i.outcome)).collect();
        assert_eq!(summary, vec![
            (KindGroup::Firewall, "ufw allow 443/tcp", Outcome::Failed),
            (KindGroup::Service, "restart nginx.service", Outcome::Done),
            (KindGroup::Config, "saved /etc/ssh/sshd_config", Outcome::Done),
            (KindGroup::Enroll, "enrolled ops@10.0.0.5:22", Outcome::Done),
        ]);
        assert_eq!(items[0].detail.as_deref(), Some("ufw allow 443/tcp · ERROR: Could not load logging rules"));
        assert_eq!(items[2].detail.as_deref(), Some("sha256:aaaaaaaaaa → sha256:bbbbbbbbbb"));
    }

    #[test]
    fn filter_by_server_and_kind() {
        let items = audit_items(&[rec("firewall", "x", "", None, "success", "2026-09-29T12:00:00Z"), rec("user", "y", "", None, "success", "2026-09-29T12:00:00Z")], &[]);
        let f = AuditFilter { server_id: None, group: Some(KindGroup::User) };
        assert_eq!(items.iter().filter(|i| f.matches(i)).count(), 1);
        let f = AuditFilter { server_id: Some("other".into()), group: None };
        assert_eq!(items.iter().filter(|i| f.matches(i)).count(), 0);
    }

    #[test]
    fn terminals_show_as_opened_then_closed_with_how_long() {
        let open = from_record(&rec("ssh.terminal", "terminal", "terminal opened (what's typed isn't recorded)", None, "open", "2026-10-02T10:00:00Z")).unwrap();
        assert_eq!((open.text.as_str(), open.outcome, open.group), ("opened a terminal", Outcome::Pending, KindGroup::Ssh));
        let closed = from_record(&rec("ssh.terminal", "terminal", "terminal opened (what's typed isn't recorded)", Some("terminal closed after 12m 3s"), "closed", "2026-10-02T10:00:00Z")).unwrap();
        assert_eq!((closed.outcome, closed.detail.as_deref()), (Outcome::Done, Some("terminal closed after 12m 3s")));
    }
}
