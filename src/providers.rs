//! The provider plugins compiled into this build (cargo features
//! `provider-*`), and how their data meets the rest of Crow.

use std::sync::Arc;

use crow_config_core::{PluginManifest, SecretValue, SettingsChange, SettingsDocument, SettingsEdit};
use crow_provider_core::curl::CurlHttp;
use crow_provider_core::{Provider, ProviderError, ProviderFactory, ProviderSettings};

use crate::vault::{provider_secret_id, MasterKey, ProviderAccount, VaultDb, PROVIDER_SECRET_CATEGORY};

/// Every compiled-in provider.
#[allow(unused_mut, clippy::vec_init_then_push)] // each push is behind its own feature
pub fn factories() -> Vec<ProviderFactory> {
    let mut out = Vec::new();
    #[cfg(feature = "provider-linode")]
    out.push(crow_provider_linode::FACTORY);
    #[cfg(feature = "provider-upcloud")]
    out.push(crow_provider_upcloud::FACTORY);
    out
}

/// The factory for plugin `name` (e.g. "linode"), if compiled in.
pub fn factory(name: &str) -> Option<ProviderFactory> {
    factories().into_iter().find(|f| (f.manifest)().plugin.name == name)
}

/// How plugin `name` is shown ("UpCloud"); the region tables use the same
/// name, so an instance's region code becomes a country and city with
/// `region::locate(&display_name(..), code)`.
pub fn display_name(name: &str) -> String {
    factory(name).map(|f| (f.manifest)().display_name().to_string()).unwrap_or_else(|| name.to_string())
}

/// A field's human name: its label, else its key.
pub fn field_label(manifest: &PluginManifest, key: &str) -> String {
    manifest.find_field(key).and_then(|f| f.label.clone()).unwrap_or_else(|| key.to_string())
}

/// Validates `edits` against the account's manifest and, only if every one
/// passes, saves them: plain values in the account row, secrets as encrypted
/// vault entries. Required fields must end up set.
pub fn save_form(db: &VaultDb, key: Option<&MasterKey>, account: &mut ProviderAccount, edits: Vec<SettingsEdit>) -> Result<(), String> {
    let manifest = factory(&account.plugin).ok_or_else(|| format!("{} isn't compiled into this build", account.plugin))?.manifest;
    let manifest = manifest();
    let secrets = db.provider_secret_keys(&account.id).map_err(|e| e.to_string())?;
    let mut doc = SettingsDocument::new(manifest, account.settings.clone(), secrets);
    let mut changes = Vec::new();
    for edit in edits {
        changes.push(doc.apply(edit).map_err(|e| e.to_string())?);
    }
    let missing = doc.missing_required();
    if !missing.is_empty() {
        let names: Vec<String> = missing.iter().map(|k| field_label(manifest, k)).collect();
        return Err(format!("{} {} required", names.join(", "), if names.len() == 1 { "is" } else { "are" }));
    }
    if changes.iter().any(|c| matches!(c, SettingsChange::StoreSecret { .. })) && key.is_none() {
        return Err("provider secrets are only stored encrypted, and no encryption key is available".into());
    }
    for change in &changes {
        match change {
            SettingsChange::Set { key, value } => {
                account.settings.insert(key.clone(), value.clone());
            }
            SettingsChange::Unset { key } => {
                account.settings.remove(key);
            }
            _ => {}
        }
    }
    db.upsert_provider_account(account).map_err(|e| e.to_string())?;
    for change in changes {
        match change {
            SettingsChange::StoreSecret { key: field, value } => {
                let name = format!("{} {}", account.label, field_label(manifest, &field));
                db.store_entry(key.expect("checked above"), &provider_secret_id(&account.id, &field), PROVIDER_SECRET_CATEGORY, &name, value.expose().as_bytes()).map_err(|e| e.to_string())?;
            }
            SettingsChange::DeleteSecret { key: field } => {
                db.delete_entry(&provider_secret_id(&account.id, &field)).map_err(|e| e.to_string())?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// What the provider needs to run: the account's plain settings plus its
/// secrets, decrypted with the vault key.
pub fn load_settings(db: &VaultDb, key: &MasterKey, account: &ProviderAccount) -> Result<ProviderSettings, String> {
    let mut settings = ProviderSettings { values: account.settings.clone().into_iter().collect(), ..Default::default() };
    for field in db.provider_secret_keys(&account.id).map_err(|e| e.to_string())? {
        let bytes = db.load_entry(key, &provider_secret_id(&account.id, &field)).map_err(|e| e.to_string())?;
        let text = String::from_utf8(bytes).map_err(|_| format!("the stored {field} isn't text"))?;
        settings.secrets.insert(field, SecretValue::new(text));
    }
    Ok(settings)
}

/// Builds the provider for `account`, talking HTTP through the system curl.
pub fn connect(account: &ProviderAccount, settings: ProviderSettings) -> Result<Box<dyn Provider>, ProviderError> {
    let f = factory(&account.plugin).ok_or_else(|| ProviderError::Unsupported(format!("{} isn't compiled into this build", account.plugin)))?;
    (f.build)(settings, Arc::new(CurlHttp::default()))
}

/// Something done to an instance at its provider (ERR-47).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderAction {
    Boot,
    Reboot,
    Shutdown,
    Snapshot,
}

impl ProviderAction {
    pub fn from_danger(action: &str) -> Option<Self> {
        match action {
            "boot" => Some(Self::Boot),
            "reboot" => Some(Self::Reboot),
            "poweroff" => Some(Self::Shutdown),
            "snapshot" => Some(Self::Snapshot),
            _ => None,
        }
    }

    pub fn verb(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::Reboot => "reboot",
            Self::Shutdown => "shutdown",
            Self::Snapshot => "snapshot",
        }
    }
}

/// Runs `action` on instance `id`, if the provider supports it.
pub fn run_action(provider: &dyn Provider, id: &str, action: ProviderAction) -> Result<String, String> {
    let id = crow_provider_core::hosts::InstanceId(id.to_string());
    let name = provider.manifest().display_name().to_string();
    let power = || provider.as_power_control().ok_or_else(|| format!("{name} can't power instances"));
    match action {
        ProviderAction::Boot => power()?.boot(&id).map(|()| format!("{name} accepted: boot {id}")),
        ProviderAction::Reboot => power()?.reboot(&id).map(|()| format!("{name} accepted: reboot {id}")),
        ProviderAction::Shutdown => power()?.shutdown(&id).map(|()| format!("{name} accepted: shut down {id}")),
        ProviderAction::Snapshot => {
            let snaps = provider.as_snapshots().ok_or_else(|| format!("{name} can't take snapshots"))?;
            let label = format!("crow-{}", chrono::Utc::now().format("%Y%m%d-%H%M"));
            snaps.snapshot(&id, &label).map(|s| format!("{name} snapshot {} ({}) started for {id}", s.id.0, s.status))
        }
    }
    .map_err(|e| e.to_string())
}

/// The instance's snapshots, as the provider lists them.
pub fn list_snapshots(provider: &dyn Provider, id: &str) -> Result<Vec<crow_provider_core::hosts::Snapshot>, String> {
    let name = provider.manifest().display_name().to_string();
    let snaps = provider.as_snapshots().ok_or_else(|| format!("{name} can't take snapshots"))?;
    snaps.snapshots(&crow_provider_core::hosts::InstanceId(id.to_string())).map_err(|e| e.to_string())
}

/// An instance a provider reported, and the enrolled server it is (if any).
#[derive(Clone, Debug)]
pub struct InstanceRow {
    pub account: String,
    pub provider_name: String,
    pub instance: crow_provider_core::hosts::Instance,
    /// The enrolled server's id, when it's already in the fleet.
    pub enrolled_as: Option<String>,
}

/// How one account's instances line up with the fleet.
#[derive(Clone, Debug, Default)]
pub struct Reconciled {
    pub rows: Vec<InstanceRow>,
    /// Servers to save: newly linked to their instance (matched by IP), or
    /// with a region learned from the provider.
    pub updates: Vec<crate::vault::ServerRecord>,
    /// Linked servers the provider no longer has (deleted or moved there).
    pub missing: Vec<String>,
    /// Linked servers whose address isn't one the provider lists any more:
    /// (server name, the instance's addresses).
    pub address_changed: Vec<(String, Vec<String>)>,
}

/// Matches `instances` from `account` against `servers`: first by an
/// existing link, then by IP. Manual region choices are never overwritten.
pub fn reconcile(servers: &[crate::vault::ServerRecord], account: &str, provider_name: &str, instances: &[crow_provider_core::hosts::Instance]) -> Reconciled {
    let mut out = Reconciled::default();
    for inst in instances {
        let linked = servers.iter().find(|s| s.provider_account == account && s.provider_instance == inst.id.0);
        let by_ip = || servers.iter().find(|s| s.host.trim().parse().is_ok_and(|ip| inst.has_ip(ip)));
        let server = linked.or_else(by_ip);
        // A linked server added by IP whose IP the instance no longer has.
        if let Some((s, ip)) = linked.and_then(|s| Some((s, s.host.trim().parse::<std::net::IpAddr>().ok()?))) {
            if !inst.has_ip(ip) {
                let addrs = inst.ipv4.iter().chain(&inst.ipv6).map(|ip| ip.to_string()).collect();
                out.address_changed.push((s.name.clone(), addrs));
            }
        }
        if let Some(s) = server {
            let mut updated = s.clone();
            updated.provider_account = account.to_string();
            updated.provider_instance = inst.id.0.clone();
            let code = inst.region.as_deref().unwrap_or_default();
            if let (true, Some((cc, city))) = (updated.region_source != "manual", crate::region::locate(provider_name, code)) {
                updated.region_country = cc.to_string();
                updated.region_city = city.to_string();
                updated.region_provider = provider_name.to_string();
                updated.region_code = code.to_string();
                updated.region_source = "provider".into();
            }
            if &updated != s {
                out.updates.push(updated);
            }
        }
        out.rows.push(InstanceRow { account: account.to_string(), provider_name: provider_name.to_string(), instance: inst.clone(), enrolled_as: server.map(|s| s.id.clone()) });
    }
    out.missing = servers
        .iter()
        .filter(|s| s.provider_account == account && !instances.iter().any(|i| i.id.0 == s.provider_instance))
        .map(|s| s.name.clone())
        .collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crow_config_core::{categories, PluginKind, SettingsDocument};

    #[test]
    fn compiled_in_providers_are_valid_host_providers() {
        let all = factories();
        assert!(!all.is_empty(), "default features compile providers in");
        for f in all {
            let m = (f.manifest)();
            assert_eq!(m.validate(), Ok(()), "{}", m.plugin.name);
            assert_eq!(m.plugin.kind, PluginKind::Provider);
            assert_eq!(m.plugin.category.as_deref(), Some(categories::PROVIDER_HOSTS));
            assert!(m.unknown_capabilities().is_empty(), "{}: {:?}", m.plugin.name, m.unknown_capabilities());
            assert!(crate::region::locate(m.display_name(), "de-fra1").is_some() || crate::region::locate(m.display_name(), "de-fra-2").is_some(), "{} has region tables under its display name", m.plugin.name);
        }
    }

    /// A provider's settings render through crow-config like any config
    /// file, with the token as set/not set, never its value.
    #[test]
    fn provider_settings_render_through_crow_config() {
        let linode = factory("linode").unwrap();
        let doc = SettingsDocument::new((linode.manifest)(), [], ["api_token".to_string()]);
        let ir = doc.to_ir();
        assert_eq!(ir.rows[0].row_id, "api_token");
        assert_eq!(ir.rows[0].fields[0].value, serde_json::json!({"set": true}));
        assert!(doc.missing_required().is_empty());
    }

    fn account(plugin: &str) -> ProviderAccount {
        ProviderAccount { id: plugin.into(), plugin: plugin.into(), label: plugin.into(), settings: Default::default(), last_check: None, last_check_ok: false, last_check_at: None }
    }

    #[test]
    fn saving_a_form_puts_secrets_in_the_vault_only() {
        let mut db = VaultDb::open_in_memory().unwrap();
        let key = db.init_vault("pw", None, None).unwrap();
        let mut acct = account("upcloud");
        let edits = vec![
            SettingsEdit::Set { key: "username".into(), value: serde_json::json!("crow-api") },
            SettingsEdit::SetSecret { key: "password".into(), value: SecretValue::new("pw-SECRET") },
        ];
        save_form(&db, Some(&key), &mut acct, edits).unwrap();

        let stored = &db.list_provider_accounts().unwrap()[0];
        assert_eq!(stored.settings["username"], "crow-api");
        assert!(!format!("{:?}", stored.settings).contains("SECRET"), "the row never holds the secret");
        let settings = load_settings(&db, &key, stored).unwrap();
        assert_eq!(settings.secret("password").unwrap().expose(), "pw-SECRET");
        assert!(connect(stored, settings).is_ok());
    }

    #[test]
    fn nothing_is_saved_when_an_edit_is_invalid_or_a_required_field_is_missing() {
        let mut db = VaultDb::open_in_memory().unwrap();
        let key = db.init_vault("pw", None, None).unwrap();
        let mut acct = account("linode");
        let err = save_form(&db, Some(&key), &mut acct, vec![]).unwrap_err();
        assert_eq!(err, "API token is required");
        let err = save_form(&db, Some(&key), &mut acct, vec![SettingsEdit::Set { key: "nope".into(), value: serde_json::json!(1) }]).unwrap_err();
        assert!(err.contains("nope"), "{err}");
        assert!(db.list_provider_accounts().unwrap().is_empty());
    }

    #[test]
    fn secrets_need_the_vault() {
        let db = VaultDb::open_in_memory().unwrap();
        let mut acct = account("linode");
        let err = save_form(&db, None, &mut acct, vec![SettingsEdit::SetSecret { key: "api_token".into(), value: SecretValue::new("t") }]).unwrap_err();
        assert!(err.contains("encryption key"), "{err}");
        assert!(db.list_provider_accounts().unwrap().is_empty());
    }

    fn instance(id: &str, ip: &str, region: &str) -> crow_provider_core::hosts::Instance {
        crow_provider_core::hosts::Instance {
            id: crow_provider_core::hosts::InstanceId(id.into()),
            label: format!("inst-{id}"),
            status: crow_provider_core::hosts::InstanceStatus::Running,
            ipv4: vec![ip.parse().unwrap()],
            ipv6: vec![],
            region: Some(region.into()),
            plan: None,
            tags: vec![],
            extra: Default::default(),
        }
    }

    fn server(id: &str, host: &str) -> crate::vault::ServerRecord {
        crate::vault::ServerRecord { id: id.into(), name: id.into(), host: host.into(), ..Default::default() }
    }

    #[test]
    fn reconcile_links_by_ip_learns_the_region_and_flags_missing() {
        let mut gone = server("old-box", "10.9.9.9");
        gone.provider_account = "linode".into();
        gone.provider_instance = "999".into();
        let mut manual = server("fra-box", "104.105.13.91");
        manual.region_source = "manual".into();
        manual.region_country = "NL".into();
        let servers = vec![server("web-01", "172.105.91.183"), manual, gone];
        let r = reconcile(&servers, "linode", "Linode", &[instance("1", "172.105.91.183", "de-fra-2"), instance("2", "104.105.13.91", "de-fra-2"), instance("3", "45.1.2.3", "us-east")]);

        assert_eq!(r.rows.iter().map(|r| r.enrolled_as.as_deref()).collect::<Vec<_>>(), [Some("web-01"), Some("fra-box"), None]);
        let web = r.updates.iter().find(|s| s.id == "web-01").unwrap();
        assert_eq!((web.provider_instance.as_str(), web.region_country.as_str(), web.region_source.as_str()), ("1", "DE", "provider"));
        let fra = r.updates.iter().find(|s| s.id == "fra-box").unwrap();
        assert_eq!((fra.region_country.as_str(), fra.region_source.as_str()), ("NL", "manual"), "a manual region is kept");
        assert_eq!(fra.provider_instance, "2", "but it's still linked");
        assert_eq!(r.missing, ["old-box"]);
    }

    #[test]
    fn reconcile_is_quiet_when_nothing_changed() {
        let mut s = server("web-01", "172.105.91.183");
        s.provider_account = "linode".into();
        s.provider_instance = "1".into();
        let first = reconcile(&[s], "linode", "Linode", &[instance("1", "172.105.91.183", "de-fra-2")]);
        let settled = first.updates[0].clone();
        let again = reconcile(&[settled], "linode", "Linode", &[instance("1", "172.105.91.183", "de-fra-2")]);
        assert!(again.updates.is_empty() && again.missing.is_empty());
    }

    #[test]
    fn reconcile_flags_a_linked_server_whose_address_changed() {
        let mut s = server("web-01", "172.105.91.183");
        s.provider_account = "linode".into();
        s.provider_instance = "1".into();
        let r = reconcile(&[s], "linode", "Linode", &[instance("1", "45.9.9.9", "de-fra-2")]);
        assert_eq!(r.address_changed, [("web-01".to_string(), vec!["45.9.9.9".to_string()])]);
        assert_eq!(r.rows[0].enrolled_as.as_deref(), Some("web-01"), "still the same instance");
    }

    #[test]
    fn actions_run_only_where_the_provider_supports_them() {
        use crow_provider_core::http::Method;
        use crow_provider_core::testing::RecordedHttp;
        let up = crow_provider_upcloud::API;
        let http = std::sync::Arc::new(RecordedHttp::new().on(Method::Post, &format!("{up}/server/abc/restart"), 200, "{}"));
        let settings = ProviderSettings::default().with_secret("api_token", "t");
        let upcloud = (factory("upcloud").unwrap().build)(settings, http.clone()).unwrap();
        assert_eq!(run_action(upcloud.as_ref(), "abc", ProviderAction::Reboot).unwrap(), "UpCloud accepted: reboot abc");
        assert_eq!(run_action(upcloud.as_ref(), "abc", ProviderAction::Snapshot).unwrap_err(), "UpCloud can't take snapshots");
        assert_eq!(http.requests().len(), 1, "nothing sent for the unsupported one");
    }

    #[test]
    fn snapshots_are_listed_through_the_provider() {
        use crow_provider_core::http::Method;
        use crow_provider_core::testing::RecordedHttp;
        let api = crow_provider_linode::API;
        let body = r#"{"automatic": [], "snapshot": {"current": {"id": 7, "label": "before-sshd", "status": "successful", "created": "2026-09-25T10:00:00"}, "in_progress": null}}"#;
        let http = std::sync::Arc::new(RecordedHttp::new().on(Method::Get, &format!("{api}/linode/instances/123/backups"), 200, body));
        let linode = (factory("linode").unwrap().build)(ProviderSettings::default().with_secret("api_token", "t"), http).unwrap();
        let list = list_snapshots(linode.as_ref(), "123").unwrap();
        assert_eq!((list[0].id.0.as_str(), list[0].label.as_deref()), ("7", Some("before-sshd")));
        let upcloud = (factory("upcloud").unwrap().build)(ProviderSettings::default().with_secret("api_token", "t"), std::sync::Arc::new(RecordedHttp::new())).unwrap();
        assert_eq!(list_snapshots(upcloud.as_ref(), "x").unwrap_err(), "UpCloud can't take snapshots");
    }

    #[test]
    fn provider_regions_map_to_flags() {
        assert_eq!(crate::region::locate(&display_name("linode"), "de-fra-2"), Some(("DE", "Frankfurt")));
        assert_eq!(crate::region::locate(&display_name("upcloud"), "fi-hel1"), Some(("FI", "Helsinki")));
    }
}
