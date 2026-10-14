//! Plugins' badges (ERR-182): the status each accepted plugin with a badge
//! last read for each server, for the identity bar, the fleet table and
//! the fleet map to show. Kept here so those views read it without new
//! arguments, like the connection states in `host`.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use super::run::State;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeView {
    pub plugin: String,
    /// The plugin's label, plus "· <date>" of its last applied run.
    pub text: String,
    pub state: State,
}

fn store() -> &'static RwLock<HashMap<String, Vec<BadgeView>>> {
    static S: OnceLock<RwLock<HashMap<String, Vec<BadgeView>>>> = OnceLock::new();
    S.get_or_init(Default::default)
}

/// Sets `plugin`'s badge on `server_id`, or clears it.
pub fn set(server_id: &str, plugin: &str, badge: Option<BadgeView>) {
    if let Ok(mut s) = store().write() {
        let list = s.entry(server_id.to_string()).or_default();
        list.retain(|b| b.plugin != plugin);
        if let Some(b) = badge {
            list.push(b);
            list.sort_by(|a, b| a.plugin.cmp(&b.plugin));
        }
    }
}

/// Drops every badge of plugins not in `keep` (taken back, changed, removed).
pub fn retain_plugins(keep: &[String]) {
    if let Ok(mut s) = store().write() {
        for list in s.values_mut() {
            list.retain(|b| keep.contains(&b.plugin));
        }
    }
}

pub fn for_server(server_id: &str) -> Vec<BadgeView> {
    store().read().ok().and_then(|s| s.get(server_id).cloned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{for_server, retain_plugins, set, BadgeView};
    use crate::plugin_host::run::State;

    #[test]
    fn set_replace_and_clear() {
        let b = |p: &str, t: &str| BadgeView { plugin: p.into(), text: t.into(), state: State::Ok };
        set("srv-badge-test", "zz", Some(b("zz", "one")));
        set("srv-badge-test", "aa", Some(b("aa", "two")));
        set("srv-badge-test", "zz", Some(b("zz", "three")));
        assert_eq!(for_server("srv-badge-test").iter().map(|b| b.text.as_str()).collect::<Vec<_>>(), vec!["two", "three"]);
        retain_plugins(&["zz".to_string()]);
        assert_eq!(for_server("srv-badge-test").len(), 1);
        set("srv-badge-test", "zz", None);
        assert!(for_server("srv-badge-test").is_empty());
    }
}
