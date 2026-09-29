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
            k if k.starts_with("provider.") => KindGroup::Power,
            k if k.starts_with("sshd.") => KindGroup::Ssh,
            "enroll" => KindGroup::Enroll,
            _ => KindGroup::Crow,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            KindGroup::Config => "config",
            KindGroup::Service => "service",
            KindGroup::Process => "process",
            KindGroup::Firewall => "firewall",
            KindGroup::User => "user",
            KindGroup::Power => "power",
            KindGroup::Ssh => "ssh",
            KindGroup::Enroll => "enroll",
            KindGroup::Crow => "crow",
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
pub fn audit_items(records: &[ChangeRecord], servers: &[ServerRecord]) -> Vec<AuditItem> {
    let mut items: Vec<AuditItem> = records.iter().filter_map(from_record).collect();
    items.extend(servers.iter().filter_map(|s| {
        Some(AuditItem {
            at: DateTime::parse_from_rfc3339(&s.created_at).ok()?.with_timezone(&Utc),
            server_id: s.id.clone(),
            server: s.name.clone(),
            group: KindGroup::Enroll,
            text: format!("enrolled {}@{}:{}", s.login_user, s.host, s.port),
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
        "success" | "applied" => Outcome::Done,
        "failed" => Outcome::Failed,
        _ => Outcome::Pending,
    };
    let group = KindGroup::of(&r.action_kind);
    let text = match r.action_kind.as_str() {
        "config.write" => format!("saved {}", r.target),
        "config.restore" => format!("restored an earlier version of {}", r.target),
        "process_kill" => format!("killed process {}", r.target),
        "purge" => format!("purged stored data for {}", r.target),
        "sshd.password_login_off" => "turned off SSH password login".to_string(),
        k if k.starts_with("service_") => format!("{} {}", k.trim_start_matches("service_"), r.target),
        k if k.starts_with("provider.") => format!("{} at the provider", k.trim_start_matches("provider.")),
        "firewall" => format!("ufw {}", r.target),
        "user" => format!("user {}", r.target),
        k => format!("{k} {}", r.target),
    };
    // Config records carry hashes; command records carry the command, and
    // a failure's after_state is the error.
    let detail = match (outcome, &r.after_state) {
        (Outcome::Failed, Some(err)) => Some(format!("{} · {err}", r.before_state)),
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
}
