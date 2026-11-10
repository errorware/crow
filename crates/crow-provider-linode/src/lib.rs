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

use crow_provider_core::hosts::{
    Instance, InstanceId, InstanceStatus, ListInstances, PowerControl, ProvisionCatalog,
    ProvisionInstance, ProvisionOption, ProvisionSpec, Snapshot, SnapshotId, Snapshots,
};
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
    fn as_provision_instance(&self) -> Option<&dyn ProvisionInstance> {
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

#[derive(Deserialize)]
struct LinodeRegion {
    id: String,
    label: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Deserialize)]
struct LinodePrice {
    monthly: Option<f64>,
}

#[derive(Deserialize)]
struct LinodeType {
    id: String,
    label: String,
    vcpus: Option<u32>,
    memory: Option<u64>,
    disk: Option<u64>,
    successor: Option<String>,
    price: Option<LinodePrice>,
}

#[derive(Deserialize)]
struct LinodeImage {
    id: String,
    label: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    deprecated: bool,
}

impl Linode {
    fn fetch_regions(&self) -> Option<Vec<ProvisionOption>> {
        let p: Page<LinodeRegion> = self.get("/regions?page=1&page_size=500").ok()?;
        let mut list: Vec<ProvisionOption> = p
            .data
            .into_iter()
            .filter(|r| {
                (r.status.is_empty() || r.status == "ok")
                    && (r.capabilities.is_empty() || r.capabilities.iter().any(|c| c.eq_ignore_ascii_case("Linodes")))
            })
            .map(|r| {
                let lbl = if r.label.contains(&r.id) {
                    r.label
                } else {
                    format!("{} ({})", r.label, r.id)
                };
                ProvisionOption::new(r.id, lbl)
            })
            .collect();
        list.sort_by(|a, b| a.label.cmp(&b.label));
        (!list.is_empty()).then_some(list)
    }

    fn fetch_plans(&self) -> Option<Vec<ProvisionOption>> {
        let p: Page<LinodeType> = self.get("/linode/types?page=1&page_size=500").ok()?;
        let mut types: Vec<LinodeType> = p
            .data
            .into_iter()
            .filter(|t| t.successor.is_none())
            .collect();
        types.sort_by(|a, b| {
            let pa = a.price.as_ref().and_then(|p| p.monthly).unwrap_or(f64::MAX);
            let pb = b.price.as_ref().and_then(|p| p.monthly).unwrap_or(f64::MAX);
            pa.partial_cmp(&pb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.memory.cmp(&b.memory))
        });
        let list: Vec<ProvisionOption> = types
            .into_iter()
            .map(|t| {
                let vcpu = t.vcpus.unwrap_or(1);
                let ram_gb = t.memory.unwrap_or(1024) as f64 / 1024.0;
                let disk_gb = t.disk.unwrap_or(25600) / 1024;
                let ram_str = if (ram_gb - ram_gb.round()).abs() < 0.05 {
                    format!("{} GB", ram_gb.round() as u64)
                } else {
                    format!("{ram_gb:.1} GB")
                };
                let price_str = match t.price.as_ref().and_then(|p| p.monthly) {
                    Some(m) if m > 0.0 => format!(" · ${m:.0}/mo"),
                    _ => String::new(),
                };
                ProvisionOption::new(
                    t.id,
                    format!("{} ({vcpu} vCPU, {ram_str} RAM, {disk_gb} GB SSD{price_str})", t.label),
                )
            })
            .collect();
        (!list.is_empty()).then_some(list)
    }

    fn fetch_images(&self) -> Option<Vec<ProvisionOption>> {
        let p: Page<LinodeImage> = self.get("/images?page=1&page_size=500").ok()?;
        let mut imgs: Vec<LinodeImage> = p
            .data
            .into_iter()
            .filter(|i| !i.deprecated && (i.status.is_empty() || i.status == "available"))
            .collect();
        imgs.sort_by(|a, b| {
            let a_pub = a.id.starts_with("linode/");
            let b_pub = b.id.starts_with("linode/");
            b_pub.cmp(&a_pub).then_with(|| a.label.cmp(&b.label))
        });
        let list: Vec<ProvisionOption> = imgs
            .into_iter()
            .map(|i| ProvisionOption::new(i.id, i.label))
            .collect();
        (!list.is_empty()).then_some(list)
    }
}

impl ProvisionInstance for Linode {
    fn catalog(&self) -> ProvisionCatalog {
        let regions = self.fetch_regions().unwrap_or_else(|| {
            vec![
                ProvisionOption::new("us-east", "Newark, NJ (us-east)"),
                ProvisionOption::new("us-central", "Dallas, TX (us-central)"),
                ProvisionOption::new("us-west", "Fremont, CA (us-west)"),
                ProvisionOption::new("us-southeast", "Atlanta, GA (us-southeast)"),
                ProvisionOption::new("us-iad", "Washington, DC (us-iad)"),
                ProvisionOption::new("us-ord", "Chicago, IL (us-ord)"),
                ProvisionOption::new("us-sea", "Seattle, WA (us-sea)"),
                ProvisionOption::new("us-lax", "Los Angeles, CA (us-lax)"),
                ProvisionOption::new("us-mia", "Miami, FL (us-mia)"),
                ProvisionOption::new("ca-central", "Toronto, CA (ca-central)"),
                ProvisionOption::new("br-gru", "São Paulo, BR (br-gru)"),
                ProvisionOption::new("eu-central", "Frankfurt, DE (eu-central)"),
                ProvisionOption::new("de-fra-2", "Frankfurt 2, DE (de-fra-2)"),
                ProvisionOption::new("eu-west", "London, UK (eu-west)"),
                ProvisionOption::new("gb-lon", "London 2, UK (gb-lon)"),
                ProvisionOption::new("fr-par", "Paris, FR (fr-par)"),
                ProvisionOption::new("nl-ams", "Amsterdam, NL (nl-ams)"),
                ProvisionOption::new("se-sto", "Stockholm, SE (se-sto)"),
                ProvisionOption::new("es-mad", "Madrid, ES (es-mad)"),
                ProvisionOption::new("it-mil", "Milan, IT (it-mil)"),
                ProvisionOption::new("ap-south", "Singapore (ap-south)"),
                ProvisionOption::new("sg-sin-2", "Singapore 2 (sg-sin-2)"),
                ProvisionOption::new("ap-northeast", "Tokyo, JP (ap-northeast)"),
                ProvisionOption::new("jp-osa", "Osaka, JP (jp-osa)"),
                ProvisionOption::new("ap-west", "Mumbai, IN (ap-west)"),
                ProvisionOption::new("in-maa", "Chennai, IN (in-maa)"),
                ProvisionOption::new("id-cgk", "Jakarta, ID (id-cgk)"),
                ProvisionOption::new("ap-southeast", "Sydney, AU (ap-southeast)"),
                ProvisionOption::new("au-mel", "Melbourne, AU (au-mel)"),
            ]
        });
        let plans = self.fetch_plans().unwrap_or_else(|| {
            vec![
                ProvisionOption::new("g6-nanode-1", "Nanode 1GB (1 vCPU, 1 GB RAM, 25 GB SSD · $5/mo)"),
                ProvisionOption::new("g6-standard-1", "Linode 2GB (1 vCPU, 2 GB RAM, 50 GB SSD · $12/mo)"),
                ProvisionOption::new("g6-standard-2", "Linode 4GB (2 vCPU, 4 GB RAM, 80 GB SSD · $24/mo)"),
                ProvisionOption::new("g6-standard-4", "Linode 8GB (4 vCPU, 8 GB RAM, 160 GB SSD · $48/mo)"),
                ProvisionOption::new("g6-standard-6", "Linode 16GB (6 vCPU, 16 GB RAM, 320 GB SSD · $96/mo)"),
                ProvisionOption::new("g6-standard-8", "Linode 32GB (8 vCPU, 32 GB RAM, 640 GB SSD · $192/mo)"),
                ProvisionOption::new("g6-dedicated-2", "Dedicated 4GB (2 vCPU, 4 GB RAM, 80 GB SSD · $36/mo)"),
                ProvisionOption::new("g6-dedicated-4", "Dedicated 8GB (4 vCPU, 8 GB RAM, 160 GB SSD · $72/mo)"),
                ProvisionOption::new("g6-dedicated-8", "Dedicated 16GB (8 vCPU, 16 GB RAM, 320 GB SSD · $144/mo)"),
            ]
        });
        let images = self.fetch_images().unwrap_or_else(|| {
            vec![
                ProvisionOption::new("linode/ubuntu24.04", "Ubuntu 24.04 LTS"),
                ProvisionOption::new("linode/ubuntu22.04", "Ubuntu 22.04 LTS"),
                ProvisionOption::new("linode/debian12", "Debian 12 (Bookworm)"),
                ProvisionOption::new("linode/debian11", "Debian 11 (Bullseye)"),
                ProvisionOption::new("linode/almalinux9", "AlmaLinux 9"),
                ProvisionOption::new("linode/rocky9", "Rocky Linux 9"),
                ProvisionOption::new("linode/fedora40", "Fedora 40"),
                ProvisionOption::new("linode/arch", "Arch Linux"),
                ProvisionOption::new("linode/alpine3.20", "Alpine Linux 3.20"),
            ]
        });
        ProvisionCatalog { regions, plans, images }
    }

    fn provision(&self, spec: &ProvisionSpec) -> Result<Instance, ProviderError> {
        let mut body = json!({
            "label": spec.label,
            "region": spec.region,
            "type": spec.plan,
            "image": spec.image,
            "booted": true,
        });

        if !spec.ssh_public_key.trim().is_empty() {
            body["authorized_keys"] = json!([spec.ssh_public_key.trim()]);
        }

        if !spec.tags.is_empty() {
            body["tags"] = json!(spec.tags);
        }

        let res = self.post("/linode/instances", body)?;
        let linode_inst: LinodeInstance = serde_json::from_str(&res.body).map_err(|e| ProviderError::Unexpected(e.to_string()))?;
        Ok(Instance::from(linode_inst))
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

    #[test]
    fn provision_creates_instance_with_ssh_key() {
        let (l, http) = linode(
            RecordedHttp::new().on(Method::Post, &format!("{API}/linode/instances"), 200, INSTANCE),
        );
        let spec = ProvisionSpec {
            label: "assassin-01".into(),
            region: "de-fra-2".into(),
            plan: "g6-nanode-1".into(),
            image: "linode/ubuntu24.04".into(),
            ssh_public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI1234567890 root@test".into(),
            tags: vec!["prod".into()],
        };
        let inst = l.provision(&spec).unwrap();
        assert_eq!(inst.id.0, "123");
        assert_eq!(inst.label, "assassin-01");
        assert_eq!(inst.status, InstanceStatus::Running);

        let reqs = http.requests();
        assert_eq!(reqs.len(), 1);
        let body = reqs[0].body.as_ref().unwrap();
        assert_eq!(body["label"], "assassin-01");
        assert_eq!(body["authorized_keys"][0], "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI1234567890 root@test");
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
