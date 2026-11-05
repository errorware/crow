//! UpCloud provider, API 1.3.
//!
//! - `instances.list`: `GET /server` for the servers and `GET /ip_address`
//!   for every address in the account (two calls, however many servers).
//! - `instances.power`: start, soft stop, soft restart.
//!
//! No snapshots: UpCloud backs up storage devices, not servers, so it
//! doesn't fit the `snapshots` contract yet. The manifest doesn't declare
//! it, and Crow doesn't offer it.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, LazyLock};

use serde::Deserialize;
use serde_json::{json, Value};

use crow_provider_core::hosts::{
    Instance, InstanceId, InstanceStatus, ListInstances, PowerControl, ProvisionCatalog,
    ProvisionInstance, ProvisionOption, ProvisionSpec,
};
use crow_provider_core::http::{self, Http, HttpRequest};
use crow_provider_core::{PluginManifest, Provider, ProviderError, ProviderFactory, ProviderSettings, SecretValue};

pub const API: &str = "https://api.upcloud.com/1.3";
/// Seconds a soft stop may take before UpCloud gives up (the server keeps running).
const SOFT_STOP_TIMEOUT: &str = "60";

pub const MANIFEST_TOML: &str = include_str!("manifest.toml");
pub static MANIFEST: LazyLock<PluginManifest> = LazyLock::new(|| PluginManifest::from_toml_str(MANIFEST_TOML).expect("upcloud manifest"));

pub const FACTORY: ProviderFactory = ProviderFactory { manifest: || &MANIFEST, build: |settings, http| Ok(Box::new(UpCloud::new(&settings, http)?)) };

enum Auth {
    Token(SecretValue),
    Basic(String, SecretValue),
}

pub struct UpCloud {
    auth: Auth,
    http: Arc<dyn Http>,
}

impl UpCloud {
    /// Uses the API token when set, else username and password.
    pub fn new(settings: &ProviderSettings, http: Arc<dyn Http>) -> Result<Self, ProviderError> {
        let auth = match settings.secret("api_token") {
            Some(token) => Auth::Token(token.clone()),
            None => match (settings.string("username"), settings.secret("password")) {
                (Some(user), Some(password)) => Auth::Basic(user.to_string(), password.clone()),
                (Some(_), None) => return Err(ProviderError::NotConfigured("password".into())),
                _ => return Err(ProviderError::NotConfigured("api_token (or username and password)".into())),
            },
        };
        Ok(Self { auth, http })
    }

    fn authed(&self, request: HttpRequest) -> HttpRequest {
        match &self.auth {
            Auth::Token(t) => request.bearer(t),
            Auth::Basic(user, password) => request.basic_auth(user, password),
        }
    }

    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, ProviderError> {
        http::decode(self.http.send(&self.authed(HttpRequest::get(format!("{API}{path}"))))?, error_message)
    }

    fn post(&self, path: &str, body: Value) -> Result<(), ProviderError> {
        http::check(self.http.send(&self.authed(HttpRequest::post(format!("{API}{path}"), body)))?, error_message).map(drop)
    }

    fn post_decode<T: serde::de::DeserializeOwned>(&self, path: &str, body: Value) -> Result<T, ProviderError> {
        http::decode(self.http.send(&self.authed(HttpRequest::post(format!("{API}{path}"), body)))?, error_message)
    }

    fn servers(&self) -> Result<Vec<UpServer>, ProviderError> {
        Ok(self.get::<ServerList>("/server")?.servers.server)
    }
}

/// UpCloud's error body: `{"error": {"error_code": "...", "error_message": "..."}}`.
fn error_message(v: &Value) -> Option<String> {
    v["error"]["error_message"].as_str().map(str::to_string)
}

#[derive(Deserialize)]
struct ServerList {
    servers: Servers,
}
#[derive(Deserialize)]
struct Servers {
    #[serde(default)]
    server: Vec<UpServer>,
}

#[derive(Deserialize)]
struct UpServer {
    uuid: String,
    title: Option<String>,
    hostname: Option<String>,
    state: String,
    zone: Option<String>,
    plan: Option<String>,
    #[serde(default)]
    tags: Tags,
}

#[derive(Deserialize, Default)]
struct Tags {
    #[serde(default)]
    tag: Vec<String>,
}

#[derive(Deserialize)]
struct IpList {
    ip_addresses: IpAddresses,
}
#[derive(Deserialize)]
struct IpAddresses {
    #[serde(default)]
    ip_address: Vec<UpIp>,
}

#[derive(Deserialize)]
struct UpIp {
    access: String,
    address: String,
    server: Option<String>,
}

fn status(s: &str) -> InstanceStatus {
    match s {
        "started" => InstanceStatus::Running,
        "stopped" => InstanceStatus::Stopped,
        other => InstanceStatus::Other(other.to_string()),
    }
}

fn instance(s: UpServer, ips: &HashMap<String, Vec<&UpIp>>) -> Instance {
    let mut addrs: Vec<(bool, IpAddr)> = ips
        .get(&s.uuid)
        .into_iter()
        .flatten()
        .filter(|ip| ip.access != "utility")
        .filter_map(|ip| Some((ip.access != "public", ip.address.parse().ok()?)))
        .collect();
    addrs.sort_by_key(|(private, _)| *private);
    let (ipv4, ipv6) = addrs.into_iter().map(|(_, ip)| ip).partition(IpAddr::is_ipv4);
    let label = s.title.filter(|t| !t.is_empty()).or(s.hostname.clone()).unwrap_or_else(|| s.uuid.clone());
    let extra = s.hostname.map(|h| [("hostname".to_string(), Value::String(h))].into()).unwrap_or_default();
    Instance { id: InstanceId(s.uuid), label, status: status(&s.state), ipv4, ipv6, region: s.zone, plan: s.plan, tags: s.tags.tag, extra }
}

impl Provider for UpCloud {
    fn manifest(&self) -> &PluginManifest {
        &MANIFEST
    }

    fn check(&self) -> Result<String, ProviderError> {
        let n = self.servers()?.len();
        Ok(format!("{n} server{}", if n == 1 { "" } else { "s" }))
    }

    fn as_list_instances(&self) -> Option<&dyn ListInstances> {
        Some(self)
    }
    fn as_power_control(&self) -> Option<&dyn PowerControl> {
        Some(self)
    }
    fn as_provision_instance(&self) -> Option<&dyn ProvisionInstance> {
        Some(self)
    }
}

impl ListInstances for UpCloud {
    fn instances(&self) -> Result<Vec<Instance>, ProviderError> {
        let servers = self.servers()?;
        let ips = self.get::<IpList>("/ip_address")?.ip_addresses.ip_address;
        let mut by_server: HashMap<String, Vec<&UpIp>> = HashMap::new();
        for ip in &ips {
            if let Some(server) = &ip.server {
                by_server.entry(server.clone()).or_default().push(ip);
            }
        }
        Ok(servers.into_iter().map(|s| instance(s, &by_server)).collect())
    }
}

impl PowerControl for UpCloud {
    fn boot(&self, id: &InstanceId) -> Result<(), ProviderError> {
        self.post(&format!("/server/{id}/start"), json!({}))
    }
    fn reboot(&self, id: &InstanceId) -> Result<(), ProviderError> {
        self.post(&format!("/server/{id}/restart"), json!({"restart_server": {"stop_type": "soft", "timeout": SOFT_STOP_TIMEOUT, "timeout_action": "ignore"}}))
    }
    fn shutdown(&self, id: &InstanceId) -> Result<(), ProviderError> {
        self.post(&format!("/server/{id}/stop"), json!({"stop_server": {"stop_type": "soft", "timeout": SOFT_STOP_TIMEOUT}}))
    }
}

#[derive(Deserialize)]
struct CreatedServerResponse {
    server: CreatedServer,
}

#[derive(Deserialize)]
struct CreatedServer {
    uuid: String,
    title: Option<String>,
    hostname: Option<String>,
    state: String,
    zone: Option<String>,
    plan: Option<String>,
    #[serde(default)]
    tags: Tags,
    #[serde(default)]
    ip_addresses: Option<IpAddressesField>,
}

#[derive(Deserialize)]
struct IpAddressesField {
    #[serde(default)]
    ip_address: Vec<UpIp>,
}

impl ProvisionInstance for UpCloud {
    fn catalog(&self) -> ProvisionCatalog {
        ProvisionCatalog {
            regions: vec![
                ProvisionOption::new("de-fra1", "Frankfurt 1 (de-fra1)"),
                ProvisionOption::new("fi-hel1", "Helsinki 1 (fi-hel1)"),
                ProvisionOption::new("fi-hel2", "Helsinki 2 (fi-hel2)"),
                ProvisionOption::new("nl-ams1", "Amsterdam 1 (nl-ams1)"),
                ProvisionOption::new("uk-lon1", "London 1 (uk-lon1)"),
                ProvisionOption::new("us-nyc1", "New York 1 (us-nyc1)"),
                ProvisionOption::new("us-chi1", "Chicago 1 (us-chi1)"),
                ProvisionOption::new("us-sjo1", "San Jose 1 (us-sjo1)"),
                ProvisionOption::new("sg-sin1", "Singapore 1 (sg-sin1)"),
                ProvisionOption::new("pl-waw1", "Warsaw 1 (pl-waw1)"),
                ProvisionOption::new("es-mad1", "Madrid 1 (es-mad1)"),
                ProvisionOption::new("se-sto1", "Stockholm 1 (se-sto1)"),
                ProvisionOption::new("au-syd1", "Sydney 1 (au-syd1)"),
            ],
            plans: vec![
                ProvisionOption::new("1xCPU-1GB", "1xCPU-1GB (1 vCPU, 1 GB RAM, 25 GB Storage)"),
                ProvisionOption::new("1xCPU-2GB", "1xCPU-2GB (1 vCPU, 2 GB RAM, 50 GB Storage)"),
                ProvisionOption::new("2xCPU-4GB", "2xCPU-4GB (2 vCPU, 4 GB RAM, 80 GB Storage)"),
                ProvisionOption::new("4xCPU-8GB", "4xCPU-8GB (4 vCPU, 8 GB RAM, 160 GB Storage)"),
                ProvisionOption::new("6xCPU-16GB", "6xCPU-16GB (6 vCPU, 16 GB RAM, 320 GB Storage)"),
            ],
            images: vec![
                ProvisionOption::new("01000000-0000-4000-8000-000030240200", "Ubuntu Server 24.04 LTS"),
                ProvisionOption::new("01000000-0000-4000-8000-000030220200", "Ubuntu Server 22.04 LTS"),
                ProvisionOption::new("01000000-0000-4000-8000-000020120200", "Debian GNU/Linux 12 (Bookworm)"),
                ProvisionOption::new("01000000-0000-4000-8000-000020110200", "Debian GNU/Linux 11 (Bullseye)"),
                ProvisionOption::new("01000000-0000-4000-8000-000060090200", "AlmaLinux 9"),
                ProvisionOption::new("01000000-0000-4000-8000-000050090200", "Rocky Linux 9"),
            ],
        }
    }

    fn provision(&self, spec: &ProvisionSpec) -> Result<Instance, ProviderError> {
        let size = match spec.plan.as_str() {
            "1xCPU-2GB" => 50,
            "2xCPU-4GB" => 80,
            "4xCPU-8GB" => 160,
            "6xCPU-16GB" => 320,
            _ => 25,
        };

        let hostname = format!("{}.example.com", spec.label.replace(' ', "-").to_lowercase());

        let mut server_obj = json!({
            "zone": spec.region,
            "title": spec.label,
            "hostname": hostname,
            "plan": spec.plan,
            "storage_devices": {
                "storage_device": [
                    {
                        "action": "clone",
                        "storage": spec.image,
                        "title": format!("{}-disk", spec.label),
                        "size": size,
                        "tier": "maxiops"
                    }
                ]
            }
        });

        if !spec.ssh_public_key.trim().is_empty() {
            server_obj["login_user"] = json!({
                "username": "root",
                "ssh_keys": {
                    "ssh_key": [
                        spec.ssh_public_key.trim()
                    ]
                }
            });
        }

        let body = json!({ "server": server_obj });
        let created: CreatedServerResponse = self.post_decode("/server", body)?;
        let s = created.server;
        let mut by_server: HashMap<String, Vec<&UpIp>> = HashMap::new();
        if let Some(ref ips) = s.ip_addresses {
            for ip in &ips.ip_address {
                by_server.entry(s.uuid.clone()).or_default().push(ip);
            }
        }

        let up_server = UpServer {
            uuid: s.uuid,
            title: s.title,
            hostname: s.hostname,
            state: s.state,
            zone: s.zone,
            plan: s.plan,
            tags: s.tags,
        };

        Ok(instance(up_server, &by_server))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crow_provider_core::http::Method;
    use crow_provider_core::testing::RecordedHttp;

    const SERVERS: &str = r#"{"servers": {"server": [
        {"uuid": "00798b85-efdc-41ca-8021-f6ef457b8531", "title": "web-01", "hostname": "web-01.example.com", "state": "started", "zone": "de-fra1", "plan": "1xCPU-1GB", "tags": {"tag": ["prod"]}},
        {"uuid": "009d64ef-31d1-4684-a26b-c86c955cbf46", "title": "", "hostname": "batch.example.com", "state": "maintenance", "zone": "fi-hel1", "plan": "2xCPU-4GB", "tags": {"tag": []}}
    ]}}"#;
    const IPS: &str = r#"{"ip_addresses": {"ip_address": [
        {"access": "private", "address": "10.1.2.3", "family": "IPv4", "server": "00798b85-efdc-41ca-8021-f6ef457b8531"},
        {"access": "public", "address": "94.237.1.2", "family": "IPv4", "server": "00798b85-efdc-41ca-8021-f6ef457b8531"},
        {"access": "public", "address": "2a04:3540:1000:310::1", "family": "IPv6", "server": "00798b85-efdc-41ca-8021-f6ef457b8531"},
        {"access": "utility", "address": "10.5.0.9", "family": "IPv4", "server": "00798b85-efdc-41ca-8021-f6ef457b8531"},
        {"access": "public", "address": "94.237.9.9", "family": "IPv4", "server": "009d64ef-31d1-4684-a26b-c86c955cbf46"},
        {"access": "public", "address": "94.237.7.7", "family": "IPv4"}
    ]}}"#;

    fn recorded() -> RecordedHttp {
        RecordedHttp::new().on(Method::Get, &format!("{API}/server"), 200, SERVERS).on(Method::Get, &format!("{API}/ip_address"), 200, IPS)
    }

    fn with_token(http: RecordedHttp) -> (UpCloud, Arc<RecordedHttp>) {
        let http = Arc::new(http);
        (UpCloud::new(&ProviderSettings::default().with_secret("api_token", "ucat_SECRET"), http.clone()).unwrap(), http)
    }

    #[test]
    fn declares_what_it_implements_and_no_snapshots() {
        let (u, _) = with_token(RecordedHttp::new());
        crow_provider_core::check_declarations(&u).unwrap();
        assert_eq!(MANIFEST.unknown_capabilities(), Vec::<&str>::new());
        assert!(u.as_snapshots().is_none(), "the UI must not offer snapshots for UpCloud");
    }

    #[test]
    fn provision_creates_server_with_ssh_key() {
        let created_json = r#"{"server": {
            "uuid": "00112233-4455-6677-8899-aabbccddeeff",
            "title": "app-01",
            "hostname": "app-01.example.com",
            "state": "started",
            "zone": "de-fra1",
            "plan": "1xCPU-1GB",
            "tags": {"tag": ["prod"]},
            "ip_addresses": {
                "ip_address": [
                    {"access": "public", "address": "94.237.10.10", "family": "IPv4"}
                ]
            }
        }}"#;
        let (u, http) = with_token(
            RecordedHttp::new().on(Method::Post, &format!("{API}/server"), 200, created_json)
        );

        let spec = ProvisionSpec {
            label: "app-01".into(),
            region: "de-fra1".into(),
            plan: "1xCPU-1GB".into(),
            image: "01000000-0000-4000-8000-000030240200".into(),
            ssh_public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI1234567890 root@test".into(),
            tags: vec!["prod".into()],
        };

        let inst = u.provision(&spec).unwrap();
        assert_eq!(inst.id.0, "00112233-4455-6677-8899-aabbccddeeff");
        assert_eq!(inst.label, "app-01");
        assert_eq!(inst.status, InstanceStatus::Running);
        assert_eq!(inst.ipv4[0].to_string(), "94.237.10.10");

        let reqs = http.requests();
        assert_eq!(reqs.len(), 1);
        let body = reqs[0].body.as_ref().unwrap();
        assert_eq!(body["server"]["title"], "app-01");
        assert_eq!(body["server"]["zone"], "de-fra1");
        assert_eq!(body["server"]["login_user"]["ssh_keys"]["ssh_key"][0], "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI1234567890 root@test");
    }

    #[test]
    fn lists_servers_with_their_addresses() {
        let (u, http) = with_token(recorded());
        let all = u.instances().unwrap();
        assert_eq!(all.len(), 2);
        let web = &all[0];
        assert_eq!((web.label.as_str(), &web.status, web.region.as_deref()), ("web-01", &InstanceStatus::Running, Some("de-fra1")));
        assert_eq!(web.ipv4.iter().map(ToString::to_string).collect::<Vec<_>>(), ["94.237.1.2", "10.1.2.3"], "public first, utility dropped");
        assert_eq!(web.ipv6[0].to_string(), "2a04:3540:1000:310::1");
        assert_eq!(web.tags, ["prod"]);
        assert_eq!(all[1].label, "batch.example.com", "empty title falls back to the hostname");
        assert_eq!(all[1].status, InstanceStatus::Other("maintenance".into()));
        assert_eq!(http.requests().len(), 2, "two calls however many servers");
        assert_eq!(http.requests()[0].headers[1].1.expose(), "Bearer ucat_SECRET");
    }

    #[test]
    fn falls_back_to_username_and_password() {
        let http = Arc::new(recorded());
        let settings = ProviderSettings::default().with_value("username", "crow-api").with_secret("password", "pw-SECRET");
        let u = UpCloud::new(&settings, http.clone()).unwrap();
        assert_eq!(u.check().unwrap(), "2 servers");
        let auth = http.requests()[0].headers[1].1.expose().to_string();
        assert_eq!(auth, format!("Basic {}", http::base64(b"crow-api:pw-SECRET")));
        assert!(!format!("{:?}", http.requests()).contains("SECRET"));
    }

    #[test]
    fn says_which_credentials_are_missing() {
        let http: Arc<dyn Http> = Arc::new(RecordedHttp::new());
        let err = |s: ProviderSettings| UpCloud::new(&s, http.clone()).err().unwrap();
        assert_eq!(err(ProviderSettings::default()), ProviderError::NotConfigured("api_token (or username and password)".into()));
        assert_eq!(err(ProviderSettings::default().with_value("username", "me")), ProviderError::NotConfigured("password".into()));
    }

    #[test]
    fn power_actions_are_soft_and_never_force() {
        let id = "00798b85-efdc-41ca-8021-f6ef457b8531";
        let (u, http) = with_token(
            RecordedHttp::new()
                .on(Method::Post, &format!("{API}/server/{id}/start"), 200, "{}")
                .on(Method::Post, &format!("{API}/server/{id}/restart"), 200, "{}")
                .on(Method::Post, &format!("{API}/server/{id}/stop"), 200, "{}"),
        );
        let id = InstanceId(id.into());
        u.boot(&id).unwrap();
        u.reboot(&id).unwrap();
        u.shutdown(&id).unwrap();
        let bodies: Vec<Value> = http.requests().into_iter().filter_map(|r| r.body).collect();
        assert_eq!(bodies[1]["restart_server"]["stop_type"], "soft");
        assert_eq!(bodies[1]["restart_server"]["timeout_action"], "ignore");
        assert_eq!(bodies[2]["stop_server"]["stop_type"], "soft");
    }

    #[test]
    fn errors_carry_upclouds_message() {
        let (u, _) = with_token(RecordedHttp::new().on(Method::Get, &format!("{API}/server"), 401, r#"{"error": {"error_code": "AUTHENTICATION_FAILED", "error_message": "Authentication failed using the given username and password."}}"#));
        assert_eq!(u.check().unwrap_err(), ProviderError::Auth("Authentication failed using the given username and password.".into()));
    }

    #[test]
    #[ignore = "needs a real UpCloud account; set CROW_UPCLOUD_TOKEN (or CROW_UPCLOUD_USER and CROW_UPCLOUD_PASSWORD)"]
    fn live_read_only() {
        let http = Arc::new(crow_provider_core::curl::CurlHttp::default());
        let settings = if let Ok(token) = std::env::var("CROW_UPCLOUD_TOKEN") {
            ProviderSettings::default().with_secret("api_token", token)
        } else if let (Ok(user), Ok(pass)) = (std::env::var("CROW_UPCLOUD_USER"), std::env::var("CROW_UPCLOUD_PASSWORD")) {
            ProviderSettings::default().with_value("username", user).with_secret("password", pass)
        } else {
            return;
        };
        let u = UpCloud::new(&settings, http).unwrap();
        println!("check: {}", u.check().unwrap());
        for i in u.instances().unwrap() {
            println!("{} {} {:?} {:?} {:?}", i.id, i.label, i.status, i.region, i.ipv4);
        }
    }
}
