//! Bastions (ERR-152): servers designated as jump hosts, which other
//! servers can then be reached through.

use gpui_kit::*;

use super::CrowApp;
use crate::vault::ServerRecord;

/// The tag that designates a bastion.
pub const BASTION_TAG: &str = "bastion";

/// A designated bastion, or one some server already goes through (servers
/// set up before designation keep working).
pub fn is_bastion(s: &ServerRecord, servers: &[ServerRecord]) -> bool {
    s.tags.iter().any(|t| t == BASTION_TAG) || !dependents(&s.id, servers).is_empty()
}

/// The servers that go through bastion `id` directly.
pub fn dependents<'a>(id: &str, servers: &'a [ServerRecord]) -> Vec<&'a ServerRecord> {
    servers.iter().filter(|s| s.jump_host_id.as_deref() == Some(id)).collect()
}

/// The bastions `server` may go through: designated ones, minus itself and
/// any that already sit behind it (that would loop).
pub fn choices<'a>(server_id: Option<&str>, servers: &'a [ServerRecord]) -> Vec<&'a ServerRecord> {
    let behind = |b: &ServerRecord| {
        let Some(me) = server_id else { return false };
        let mut at = Some(b);
        for _ in 0..crate::host::ssh::MAX_HOPS + 1 {
            match at {
                Some(s) if s.id == me => return true,
                Some(s) => at = s.jump_host_id.as_deref().and_then(|j| servers.iter().find(|o| o.id == j)),
                None => return false,
            }
        }
        true
    };
    let mut out: Vec<&ServerRecord> = servers.iter().filter(|b| is_bastion(b, servers) && !behind(b)).collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// "edge-1 → edge-2": the bastions on the way to a server behind `bastion_id`.
pub fn route(bastion_id: &str, servers: &[ServerRecord]) -> String {
    let mut names = Vec::new();
    let mut at = servers.iter().find(|s| s.id == bastion_id);
    while let Some(s) = at {
        if names.len() > crate::host::ssh::MAX_HOPS {
            break;
        }
        names.push(s.name.clone());
        at = s.jump_host_id.as_deref().and_then(|j| servers.iter().find(|o| o.id == j));
    }
    names.reverse();
    names.join(" → ")
}

impl CrowApp {
    /// Designates a server a bastion, or stops; refused while servers still
    /// go through it, naming them.
    pub fn set_bastion(&mut self, id: &str, on: bool, cx: &mut Context<Self>) {
        let Some(mut srv) = self.fleet.servers.iter().find(|s| s.id == id).cloned() else { return };
        if !on {
            let users: Vec<String> = dependents(id, &self.fleet.servers).iter().map(|s| s.name.clone()).collect();
            if !users.is_empty() {
                self.keys.toast = Some(format!("{} is still the bastion for {}: move them first", srv.name, users.join(", ")));
                cx.notify();
                return;
            }
        }
        srv.tags.retain(|t| t != BASTION_TAG);
        if on {
            srv.tags.push(BASTION_TAG.into());
        }
        if let Ok(db) = self.vault.db().lock() {
            let _ = db.upsert_server(&srv);
        }
        self.reload_servers();
        self.keys.toast = Some(if on { format!("{} is a bastion: servers can be added through it", srv.name) } else { format!("{} is no longer a bastion", srv.name) });
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::{choices, is_bastion, route, BASTION_TAG};
    use crate::vault::ServerRecord;

    fn s(id: &str, jump: Option<&str>, bastion: bool) -> ServerRecord {
        ServerRecord { id: id.into(), name: id.into(), jump_host_id: jump.map(Into::into), tags: if bastion { vec![BASTION_TAG.into()] } else { vec![] }, ..Default::default() }
    }

    #[test]
    fn only_designated_bastions_are_offered_and_never_in_a_loop() {
        let servers = vec![s("edge", None, true), s("inner", Some("edge"), true), s("app", Some("inner"), false), s("old-jump", None, false), s("db", Some("old-jump"), false), s("web", None, false)];
        assert!(is_bastion(&servers[3], &servers), "already used as a jump host: still a bastion");
        assert!(!is_bastion(&servers[5], &servers));
        let names = |v: Vec<&ServerRecord>| v.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(choices(None, &servers)), ["edge", "inner", "old-jump"]);
        // edge can't go through inner: inner is behind edge.
        assert_eq!(names(choices(Some("edge"), &servers)), ["old-jump"]);
        assert_eq!(route("inner", &servers), "edge → inner");
    }
}
