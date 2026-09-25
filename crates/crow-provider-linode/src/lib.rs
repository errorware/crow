//! Linode (Akamai Cloud) provider, API v4.
//!
//! - `instances.list`: `GET /linode/instances`, every page.
//! - `instances.power`: boot, reboot, shutdown.
//! - `snapshots`: Linode's manual backup snapshot. It needs the Backup
//!   service enabled on the instance, and there is one manual snapshot slot:
//!   a new one replaces the previous.

use std::net::IpAddr;
use std::sync::{Arc, LazyLock};

use serde::Deserialize;
use serde_json::{json, Value};

use crow_provider_core::hosts::{Instance, InstanceId, InstanceStatus, ListInstances, PowerControl, Snapshot, SnapshotId, Snapshots};
use crow_provider_core::http::{self, Http, HttpRequest};
use crow_provider_core::{PluginManifest, Provider, ProviderError, ProviderFactory, ProviderSettings, SecretValue};

pub const API: &str = "https://api.linode.com/v4";
const PAGE_SIZE: u32 = 500;

pub const MANIFEST_TOML: &str = include_str!("manifest.toml");
pub static MANIFEST: LazyLock<PluginManifest> = LazyLock::new(|| PluginManifest::from_toml_str(MANIFEST_TOML).expect("linode manifest"));

pub const FACTORY: ProviderFactory = ProviderFactory { manifest: || &MANIFEST, build: |settings, http| Ok(Box::new(Linode::new(&settings, http)?)) };

pub struct Linode {
    token: SecretValue,
    http: Arc<dyn Http>,
}

impl Linode {
    pub fn new(settings: &ProviderSettings, http: Arc<dyn Http>) -> Result<Self, ProviderError> {
        Ok(Self { token: settings.required_secret("api_token")?.clone(), http })
    }

    fn send(&self, request: HttpRequest) -> Result<http::HttpResponse, ProviderError> {
        http::check(self.http.send(&request.bearer(&self.token))?, error_message)
    }

    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        http::decode(self.http.send(&HttpRequest::get(format!("{API}{path}")).bearer(&self.token))?, error_message)
    }

    fn post(&self, path: &str, body: Value) -> Result<http::HttpResponse, ProviderError> {
        self.send(HttpRequest::post(format!("{API}{path}"), body))
    }

    fn page(&self, page: u32, size: u32) -> Result<Page<LinodeInstance>, ProviderError> {
        self.get(&format!("/linode/instances?page={page}&page_size={size}"))
    }
}

/// Linode's error body: `{"errors": [{"reason": "...", "field": "..."}]}`.
fn error_message(v: &Value) -> Option<String> {
    let reasons: Vec<&str> = v["errors"].as_array()?.iter().filter_map(|e| e["reason"].as_str()).collect();
    (!reasons.is_empty()).then(|| reasons.join("; "))
}

#[derive(Deserialize)]
struct Page<T> {
    data: Vec<T>,
    page: u32,
    pages: u32,
    results: u32,
}

#[derive(Deserialize)]
struct LinodeInstance {
    id: u64,
    label: String,
    status: String,
    #[serde(default)]
    ipv4: Vec<String>,
    ipv6: Option<String>,
    region: Option<String>,
    #[serde(rename = "type")]
    plan: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    image: Option<String>,
    created: Option<String>,
}

fn status(s: &str) -> InstanceStatus {
    match s {
        "running" => InstanceStatus::Running,
        "offline" | "stopped" => InstanceStatus::Stopped,
        "booting" => InstanceStatus::Starting,
        "shutting_down" => InstanceStatus::Stopping,
        "rebooting" => InstanceStatus::Rebooting,
        "provisioning" => InstanceStatus::Provisioning,
        other => InstanceStatus::Other(other.replace('_', " ")),
    }
}

impl From<LinodeInstance> for Instance {
    fn from(l: LinodeInstance) -> Self {
        // Public addresses first; Linode lists private (192.168.128.0/17) ones alongside.
        let mut ipv4: Vec<IpAddr> = l.ipv4.iter().filter_map(|s| s.parse().ok()).collect();
        ipv4.sort_by_key(|ip| matches!(ip, IpAddr::V4(v4) if v4.is_private()));
        let ipv6 = l.ipv6.as_deref().and_then(|s| s.split('/').next()?.parse().ok()).into_iter().collect();
        let extra = [("image", l.image), ("created", l.created)].into_iter().filter_map(|(k, v)| Some((k.to_string(), Value::String(v?)))).collect();
        Instance { id: InstanceId(l.id.to_string()), label: l.label, status: status(&l.status), ipv4, ipv6, region: l.region, plan: l.plan, tags: l.tags, extra }
    }
}

#[derive(Deserialize)]
struct Backup {
    id: u64,
    label: Option<String>,
    status: String,
    created: Option<String>,
}

impl From<Backup> for Snapshot {
    fn from(b: Backup) -> Self {
        Snapshot { id: SnapshotId(b.id.to_string()), label: b.label, status: b.status.replace('_', " "), created: b.created }
    }
}

#[derive(Deserialize)]
struct Backups {
    #[serde(default)]
    automatic: Vec<Backup>,
    snapshot: Option<SnapshotSlots>,
}

#[derive(Deserialize)]
struct SnapshotSlots {
    current: Option<Backup>,
    in_progress: Option<Backup>,
}

impl Provider for Linode {
    fn manifest(&self) -> &PluginManifest {
        &MANIFEST
    }

    fn check(&self) -> Result<String, ProviderError> {
        let n = self.page(1, 25)?.results;
        Ok(format!("{n} instance{}", if n == 1 { "" } else { "s" }))
    }

    fn as_list_instances(&self) -> Option<&dyn ListInstances> {
        Some(self)
    }
    fn as_power_control(&self) -> Option<&dyn PowerControl> {
        Some(self)
    }
    fn as_snapshots(&self) -> Option<&dyn Snapshots> {
        Some(self)
    }
}

impl ListInstances for Linode {
    fn instances(&self) -> Result<Vec<Instance>, ProviderError> {
        let mut out = Vec::new();
        let mut page = 1;
        loop {
            let p = self.page(page, PAGE_SIZE)?;
            out.extend(p.data.into_iter().map(Instance::from));
            if p.page >= p.pages {
                return Ok(out);
            }
            page = p.page + 1;
        }
    }

    fn instance(&self, id: &InstanceId) -> Result<Instance, ProviderError> {
        self.get::<LinodeInstance>(&format!("/linode/instances/{id}")).map(Instance::from)
    }
}

impl PowerControl for Linode {
    fn boot(&self, id: &InstanceId) -> Result<(), ProviderError> {
        self.post(&format!("/linode/instances/{id}/boot"), json!({})).map(drop)
    }
    fn reboot(&self, id: &InstanceId) -> Result<(), ProviderError> {
        self.post(&format!("/linode/instances/{id}/reboot"), json!({})).map(drop)
    }
    fn shutdown(&self, id: &InstanceId) -> Result<(), ProviderError> {
        self.post(&format!("/linode/instances/{id}/shutdown"), json!({})).map(drop)
    }
}

/// Linode's answer when the Backup service is off, made actionable.
fn backups_hint(e: ProviderError) -> ProviderError {
    match e {
        ProviderError::Provider { status: 400, message } if message.to_lowercase().contains("backups are not enabled") || message.to_lowercase().contains("backup service") => {
            ProviderError::Unsupported("Backups aren't enabled for this Linode. Turn on the Backup service for it in Linode's Cloud Manager (a paid add-on), then take snapshots from Crow.".into())
        }
        e => e,
    }
}

impl Snapshots for Linode {
    fn snapshots(&self, id: &InstanceId) -> Result<Vec<Snapshot>, ProviderError> {
        let b: Backups = self.get(&format!("/linode/instances/{id}/backups")).map_err(backups_hint)?;
        let manual = b.snapshot.into_iter().flat_map(|s| [s.in_progress, s.current]).flatten();
        Ok(manual.chain(b.automatic).map(Snapshot::from).collect())
    }

    fn snapshot(&self, id: &InstanceId, label: &str) -> Result<Snapshot, ProviderError> {
        let response = self.post(&format!("/linode/instances/{id}/backups"), json!({ "label": label })).map_err(backups_hint)?;
        serde_json::from_str::<Backup>(&response.body).map(Snapshot::from).map_err(|e| ProviderError::Unexpected(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crow_provider_core::http::Method;
    use crow_provider_core::testing::RecordedHttp;

    const INSTANCE: &str = r#"{"id": 123, "label": "assassin-01", "status": "running", "ipv4": ["192.168.139.4", "104.105.13.91"], "ipv6": "2a01:7e01::f03c:94ff:fe12:3456/128", "region": "de-fra-2", "type": "g6-nanode-1", "tags": ["prod"], "image": "linode/ubuntu24.04", "created": "2026-09-01T10:00:00"}"#;

    fn linode(http: RecordedHttp) -> (Linode, Arc<RecordedHttp>) {
        let http = Arc::new(http);
        let settings = ProviderSettings::default().with_secret("api_token", "lin-SECRET-token");
        (Linode::new(&settings, http.clone()).unwrap(), http)
    }

    fn page(page: u32, pages: u32, items: &[&str]) -> String {
        format!(r#"{{"data": [{}], "page": {page}, "pages": {pages}, "results": {}}}"#, items.join(","), items.len())
    }

    #[test]
    fn declares_what_it_implements() {
        let (l, _) = linode(RecordedHttp::new());
        crow_provider_core::check_declarations(&l).unwrap();
        assert_eq!(MANIFEST.unknown_capabilities(), Vec::<&str>::new());
    }

    #[test]
    fn needs_a_token() {
        let err = Linode::new(&ProviderSettings::default(), Arc::new(RecordedHttp::new())).err().unwrap();
        assert_eq!(err, ProviderError::NotConfigured("api_token".into()));
    }

    #[test]
    fn lists_every_page_and_maps_instances() {
        let other = INSTANCE.replace("123", "456").replace("assassin-01", "worker-05").replace("running", "offline");
        let (l, http) = linode(
            RecordedHttp::new()
                .on(Method::Get, &format!("{API}/linode/instances?page=1&page_size=500"), 200, &page(1, 2, &[INSTANCE]))
                .on(Method::Get, &format!("{API}/linode/instances?page=2&page_size=500"), 200, &page(2, 2, &[&other])),
        );
        let all = l.instances().unwrap();
        assert_eq!(all.len(), 2);
        let a = &all[0];
        assert_eq!((a.id.0.as_str(), a.label.as_str(), &a.status), ("123", "assassin-01", &InstanceStatus::Running));
        assert_eq!(a.ipv4[0].to_string(), "104.105.13.91", "public address first");
        assert_eq!(a.ipv6[0].to_string(), "2a01:7e01::f03c:94ff:fe12:3456");
        assert_eq!((a.region.as_deref(), a.plan.as_deref()), (Some("de-fra-2"), Some("g6-nanode-1")));
        assert_eq!(all[1].status, InstanceStatus::Stopped);

        let requests = http.requests();
        assert!(requests.iter().all(|r| r.headers.iter().any(|(k, v)| k == "Authorization" && v.expose() == "Bearer lin-SECRET-token")));
        assert!(!format!("{requests:?}").contains("SECRET"), "the token never shows in Debug");
    }

    #[test]
    fn check_reports_the_instance_count() {
        let (l, _) = linode(RecordedHttp::new().on(Method::Get, &format!("{API}/linode/instances?page=1&page_size=25"), 200, &page(1, 1, &[INSTANCE])));
        assert_eq!(l.check().unwrap(), "1 instance");
    }

    #[test]
    fn a_bad_token_is_an_auth_error_with_linodes_reason() {
        let (l, _) = linode(RecordedHttp::new().on(Method::Get, &format!("{API}/linode/instances?page=1&page_size=25"), 401, r#"{"errors": [{"reason": "Invalid Token"}]}"#));
        assert_eq!(l.check().unwrap_err(), ProviderError::Auth("Invalid Token".into()));
    }

    #[test]
    fn power_actions_post_to_the_instance() {
        let (l, http) = linode(
            RecordedHttp::new()
                .on(Method::Post, &format!("{API}/linode/instances/123/boot"), 200, "{}")
                .on(Method::Post, &format!("{API}/linode/instances/123/reboot"), 200, "{}")
                .on(Method::Post, &format!("{API}/linode/instances/123/shutdown"), 200, "{}"),
        );
        let id = InstanceId("123".into());
        l.boot(&id).unwrap();
        l.reboot(&id).unwrap();
        l.shutdown(&id).unwrap();
        assert_eq!(http.requests().len(), 3);
    }

    #[test]
    fn snapshots_list_manual_first_and_take_new_ones() {
        let backups = r#"{"automatic": [{"id": 1, "label": null, "status": "successful", "type": "auto", "created": "2026-09-24T02:00:00"}],
                          "snapshot": {"current": {"id": 2, "label": "before-sshd", "status": "successful", "type": "snapshot", "created": "2026-09-20T12:00:00"}, "in_progress": null}}"#;
        let (l, http) = linode(
            RecordedHttp::new()
                .on(Method::Get, &format!("{API}/linode/instances/123/backups"), 200, backups)
                .on(Method::Post, &format!("{API}/linode/instances/123/backups"), 200, r#"{"id": 3, "label": "crow-apply", "status": "pending", "type": "snapshot", "created": null}"#),
        );
        let id = InstanceId("123".into());
        let list = l.snapshots(&id).unwrap();
        assert_eq!(list.iter().map(|s| s.id.0.as_str()).collect::<Vec<_>>(), ["2", "1"]);
        let taken = l.snapshot(&id, "crow-apply").unwrap();
        assert_eq!((taken.id.0.as_str(), taken.status.as_str()), ("3", "pending"));
        assert_eq!(http.requests()[1].body, Some(json!({"label": "crow-apply"})));
    }

    #[test]
    fn backups_disabled_says_what_to_do() {
        let (l, _) = linode(RecordedHttp::new().on(Method::Post, &format!("{API}/linode/instances/123/backups"), 400, r#"{"errors": [{"reason": "Backups are not enabled for this Linode."}]}"#));
        let err = l.snapshot(&InstanceId("123".into()), "x").unwrap_err();
        assert!(matches!(&err, ProviderError::Unsupported(m) if m.contains("Backup service")), "{err}");
    }

    /// Against the real API: `CROW_LINODE_TOKEN=... cargo test -p crow-provider-linode -- --ignored`.
    /// Read-only: lists instances and snapshots, changes nothing.
    #[test]
    #[ignore]
    fn live_read_only() {
        let Ok(token) = std::env::var("CROW_LINODE_TOKEN") else { return };
        let l = Linode::new(&ProviderSettings::default().with_secret("api_token", token), Arc::new(crow_provider_core::curl::CurlHttp::default())).unwrap();
        println!("check: {}", l.check().unwrap());
        for i in l.instances().unwrap() {
            println!("{} {} {:?} {:?} {:?}", i.id, i.label, i.status, i.region, i.ipv4);
        }
    }
}
