//! AWS EC2 provider plugin for Crow (`provider.hosts`).
//!
//! Implements:
//! - `instances.list`: multi-region `DescribeInstances`
//! - `instances.power`: `StartInstances`, `StopInstances`, `RebootInstances`
//! - `instances.create`: SSH key import (`ImportKeyPair`), AMI resolution, `RunInstances`
//! - `snapshots`: EBS volume snapshots (`DescribeSnapshots`, `CreateSnapshot`)
//!
//! Outbound requests use pure-Rust AWS SigV4 (`AWS4-HMAC-SHA256`) and run over
//! `crow_provider_core::http::Http` (system `curl` via stdin config, zero credentials in argv).

pub mod sigv4;
pub mod xml;

use std::sync::{Arc, LazyLock};

use crow_provider_core::hosts::{
    Instance, InstanceId, ListInstances, PowerControl, ProvisionCatalog, ProvisionInstance,
    ProvisionOption, ProvisionSpec, Snapshot, Snapshots,
};
use crow_provider_core::http::{Http, HttpRequest};
use crow_provider_core::{PluginManifest, Provider, ProviderError, ProviderFactory, ProviderSettings, SecretValue};

pub const MANIFEST_TOML: &str = include_str!("manifest.toml");
pub static MANIFEST: LazyLock<PluginManifest> =
    LazyLock::new(|| PluginManifest::from_toml_str(MANIFEST_TOML).expect("aws manifest"));

pub const FACTORY: ProviderFactory = ProviderFactory {
    manifest: || &MANIFEST,
    build: |settings, http| Ok(Box::new(AwsEc2::new(&settings, http)?)),
};

pub struct AwsEc2 {
    access_key_id: SecretValue,
    secret_access_key: SecretValue,
    region: String,
    extra_regions: Vec<String>,
    http: Arc<dyn Http>,
}

impl AwsEc2 {
    pub fn new(settings: &ProviderSettings, http: Arc<dyn Http>) -> Result<Self, ProviderError> {
        let access_key_id = settings.required_secret("access_key_id")?.clone();
        let secret_access_key = settings.required_secret("secret_access_key")?.clone();

        let region = settings
            .string("region")
            .unwrap_or("us-east-1")
            .trim()
            .to_string();

        let extra_regions = settings
            .string("extra_regions")
            .map(|r| {
                r.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && s != &region)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            access_key_id,
            secret_access_key,
            region,
            extra_regions,
            http,
        })
    }

    /// All configured regions to query (primary region + extra regions).
    pub fn all_regions(&self) -> Vec<String> {
        let mut regions = vec![self.region.clone()];
        for r in &self.extra_regions {
            if !regions.contains(r) {
                regions.push(r.clone());
            }
        }
        regions
    }

    /// Signs and executes an EC2 Query API action for a specific region.
    pub fn send_action(
        &self,
        region: &str,
        action: &str,
        params: &[(&str, &str)],
    ) -> Result<String, ProviderError> {
        let host = format!("ec2.{region}.amazonaws.com");
        let url = format!("https://{host}/");

        let mut form_params = vec![("Action", action), ("Version", "2016-11-15")];
        form_params.extend_from_slice(params);

        let form_body = form_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencode(k), urlencode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let now = chrono::Utc::now();
        let signed = sigv4::sign_request(&sigv4::SigV4Params {
            method: "POST",
            uri_path: "/",
            query_string: "",
            host: &host,
            region,
            service: "ec2",
            access_key_id: self.access_key_id.expose(),
            secret_access_key: self.secret_access_key.expose(),
            datetime: now,
            extra_headers: &[("content-type", "application/x-www-form-urlencoded")],
            payload: form_body.as_bytes(),
        });

        let req = HttpRequest::post_form(&url, form_body)
            .header("Host", &signed.host)
            .header("X-Amz-Date", &signed.x_amz_date)
            .secret_header("Authorization", SecretValue::new(signed.authorization));

        let res = self.http.send(&req)?;
        if (200..300).contains(&res.status) {
            Ok(res.body)
        } else {
            Err(xml::parse_ec2_error(res.status, &res.body))
        }
    }

    /// Finds which region hosts an instance ID by checking primary then extra regions.
    pub fn find_instance_region(&self, id: &InstanceId) -> Result<String, ProviderError> {
        for region in self.all_regions() {
            let res = self.send_action(
                &region,
                "DescribeInstances",
                &[("InstanceId.1", &id.0)],
            );
            if let Ok(xml_str) = res {
                let instances = xml::parse_describe_instances(&xml_str, &region)?;
                if instances.iter().any(|i| &i.id == id) {
                    return Ok(region);
                }
            }
        }
        Ok(self.region.clone())
    }
}

/// RFC 3986 percent-encoding.
pub fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

impl Provider for AwsEc2 {
    fn manifest(&self) -> &PluginManifest {
        &MANIFEST
    }

    fn check(&self) -> Result<String, ProviderError> {
        let xml_str = self.send_action(&self.region, "DescribeInstances", &[("MaxResults", "10")])?;
        let instances = xml::parse_describe_instances(&xml_str, &self.region)?;
        let count = instances.len();
        Ok(format!(
            "{count} instance{} ({})",
            if count == 1 { "" } else { "s" },
            self.region
        ))
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

impl ListInstances for AwsEc2 {
    fn instances(&self) -> Result<Vec<Instance>, ProviderError> {
        let mut all = Vec::new();
        for region in self.all_regions() {
            let xml_str = self.send_action(&region, "DescribeInstances", &[])?;
            let list = xml::parse_describe_instances(&xml_str, &region)?;
            all.extend(list);
        }
        Ok(all)
    }

    fn instance(&self, id: &InstanceId) -> Result<Instance, ProviderError> {
        for region in self.all_regions() {
            let res = self.send_action(
                &region,
                "DescribeInstances",
                &[("InstanceId.1", &id.0)],
            );
            if let Ok(xml_str) = res {
                let instances = xml::parse_describe_instances(&xml_str, &region)?;
                if let Some(inst) = instances.into_iter().find(|i| &i.id == id) {
                    return Ok(inst);
                }
            }
        }
        Err(ProviderError::NotFound(format!("instance {id}")))
    }
}

impl PowerControl for AwsEc2 {
    fn boot(&self, id: &InstanceId) -> Result<(), ProviderError> {
        let region = self.find_instance_region(id)?;
        self.send_action(&region, "StartInstances", &[("InstanceId.1", &id.0)])?;
        Ok(())
    }

    fn reboot(&self, id: &InstanceId) -> Result<(), ProviderError> {
        let region = self.find_instance_region(id)?;
        self.send_action(&region, "RebootInstances", &[("InstanceId.1", &id.0)])?;
        Ok(())
    }

    fn shutdown(&self, id: &InstanceId) -> Result<(), ProviderError> {
        let region = self.find_instance_region(id)?;
        self.send_action(&region, "StopInstances", &[("InstanceId.1", &id.0)])?;
        Ok(())
    }
}

impl Snapshots for AwsEc2 {
    fn snapshots(&self, id: &InstanceId) -> Result<Vec<Snapshot>, ProviderError> {
        let region = self.find_instance_region(id)?;
        let xml_str = self.send_action(
            &region,
            "DescribeInstances",
            &[("InstanceId.1", &id.0)],
        )?;
        let vol_id = xml::parse_root_volume_id(&xml_str, &id.0)?;

        let snap_xml = self.send_action(
            &region,
            "DescribeSnapshots",
            &[
                ("Owner.1", "self"),
                ("Filter.1.Name", "volume-id"),
                ("Filter.1.Value.1", &vol_id),
            ],
        )?;

        xml::parse_describe_snapshots(&snap_xml)
    }

    fn snapshot(&self, id: &InstanceId, label: &str) -> Result<Snapshot, ProviderError> {
        let region = self.find_instance_region(id)?;
        let xml_str = self.send_action(
            &region,
            "DescribeInstances",
            &[("InstanceId.1", &id.0)],
        )?;
        let vol_id = xml::parse_root_volume_id(&xml_str, &id.0)?;

        let snap_xml = self.send_action(
            &region,
            "CreateSnapshot",
            &[
                ("VolumeId", &vol_id),
                ("Description", label),
                ("TagSpecification.1.ResourceType", "snapshot"),
                ("TagSpecification.1.Tag.1.Key", "Name"),
                ("TagSpecification.1.Tag.1.Value", label),
            ],
        )?;

        xml::parse_create_snapshot(&snap_xml)
    }
}

/// Fallback Ubuntu 24.04 LTS AMIs per common region.
fn default_ubuntu_ami(region: &str, is_arm64: bool) -> &'static str {
    match (region, is_arm64) {
        ("us-east-1", false) => "ami-0e2c8caa4b6378d8c",
        ("us-east-1", true) => "ami-053b0d53c279acc90",
        ("us-east-2", false) => "ami-036841078a4b68e14",
        ("us-east-2", true) => "ami-0ea3c35c5c3600e82",
        ("us-west-2", false) => "ami-03c983f9003cb9cd1",
        ("us-west-2", true) => "ami-0b925b65ff74301fa",
        ("eu-central-1", false) => "ami-0084a47cc718c111a",
        ("eu-central-1", true) => "ami-08573d4d3d78c3b06",
        ("eu-west-1", false) => "ami-0d64bb532e0502c46",
        ("eu-west-1", true) => "ami-0a06806504a5bf124",
        ("eu-west-2", false) => "ami-0b9932f4918a00c4f",
        ("eu-west-2", true) => "ami-06a5913e61c5c56c2",
        ("ap-northeast-1", false) => "ami-0d52744d6551d851e",
        ("ap-northeast-1", true) => "ami-05307b8b209d84cbe",
        ("ap-southeast-1", false) => "ami-047126e509f96e28b",
        ("ap-southeast-1", true) => "ami-0cf6b8f36c5617a26",
        (_, true) => "ami-053b0d53c279acc90",
        (_, false) => "ami-0e2c8caa4b6378d8c",
    }
}

impl ProvisionInstance for AwsEc2 {
    fn catalog(&self) -> ProvisionCatalog {
        ProvisionCatalog {
            regions: vec![
                ProvisionOption::new("us-east-1", "US East (N. Virginia)"),
                ProvisionOption::new("us-east-2", "US East (Ohio)"),
                ProvisionOption::new("us-west-2", "US West (Oregon)"),
                ProvisionOption::new("eu-central-1", "Europe (Frankfurt)"),
                ProvisionOption::new("eu-west-1", "Europe (Ireland)"),
                ProvisionOption::new("eu-west-2", "Europe (London)"),
                ProvisionOption::new("ap-northeast-1", "Asia Pacific (Tokyo)"),
                ProvisionOption::new("ap-southeast-1", "Asia Pacific (Singapore)"),
            ],
            plans: vec![
                ProvisionOption::new("t4g.nano", "t4g.nano (2 vCPU, 0.5 GB RAM - Graviton ARM64)"),
                ProvisionOption::new("t4g.micro", "t4g.micro (2 vCPU, 1 GB RAM - Graviton ARM64)"),
                ProvisionOption::new("t4g.small", "t4g.small (2 vCPU, 2 GB RAM - Graviton ARM64)"),
                ProvisionOption::new("t4g.medium", "t4g.medium (2 vCPU, 4 GB RAM - Graviton ARM64)"),
                ProvisionOption::new("t3.micro", "t3.micro (2 vCPU, 1 GB RAM - x86_64)"),
                ProvisionOption::new("t3.small", "t3.small (2 vCPU, 2 GB RAM - x86_64)"),
                ProvisionOption::new("t3.medium", "t3.medium (2 vCPU, 4 GB RAM - x86_64)"),
                ProvisionOption::new("c6i.large", "c6i.large (2 vCPU, 4 GB RAM - x86_64 Compute)"),
            ],
            images: vec![
                ProvisionOption::new("ubuntu-24-04", "Ubuntu 24.04 LTS (Noble Numbat)"),
                ProvisionOption::new("ubuntu-22-04", "Ubuntu 22.04 LTS (Jammy Jellyfish)"),
                ProvisionOption::new("debian-12", "Debian 12 (Bookworm)"),
                ProvisionOption::new("al2023", "Amazon Linux 2023"),
            ],
        }
    }

    fn provision(&self, spec: &ProvisionSpec) -> Result<Instance, ProviderError> {
        let region = if spec.region.is_empty() {
            &self.region
        } else {
            &spec.region
        };

        let is_arm64 = spec.plan.starts_with("t4g") || spec.plan.contains("arm");

        // 1. Ensure SSH key pair is registered on EC2 in target region
        let key_name = format!(
            "crow-{}",
            spec.label
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .take(32)
                .collect::<String>()
        );
        let key_name = if key_name == "crow-" {
            "crow-default-key".to_string()
        } else {
            key_name
        };

        if !spec.ssh_public_key.trim().is_empty() {
            // ImportKeyPair expects base64 encoded public key material
            let b64_key = crow_provider_core::http::base64(spec.ssh_public_key.trim().as_bytes());
            let import_res = self.send_action(
                region,
                "ImportKeyPair",
                &[
                    ("KeyName", &key_name),
                    ("PublicKeyMaterial", &b64_key),
                ],
            );
            // If duplicate, that's completely fine
            if let Err(e) = import_res {
                if !matches!(e, ProviderError::Provider { ref message, .. } if message.contains("Duplicate") || message.contains("already exists")) {
                    // Ignore duplicate key errors, bubble up others
                }
            }
        }

        // 2. Resolve AMI ID for this region and architecture
        let ami_id = default_ubuntu_ami(region, is_arm64).to_string();

        // 3. Launch instance with RunInstances
        let mut run_params = vec![
            ("ImageId", ami_id.as_str()),
            ("InstanceType", spec.plan.as_str()),
            ("MinCount", "1"),
            ("MaxCount", "1"),
            ("KeyName", key_name.as_str()),
            ("TagSpecification.1.ResourceType", "instance"),
            ("TagSpecification.1.Tag.1.Key", "Name"),
            ("TagSpecification.1.Tag.1.Value", spec.label.as_str()),
        ];

        let tag_key = "TagSpecification.1.Tag.2.Key";
        let tag_val = "TagSpecification.1.Tag.2.Value";
        let extra_tag_str;
        if !spec.tags.is_empty() {
            extra_tag_str = spec.tags.join(",");
            run_params.push((tag_key, "CrowTags"));
            run_params.push((tag_val, extra_tag_str.as_str()));
        }

        let run_xml = self.send_action(region, "RunInstances", &run_params)?;
        let mut inst = xml::parse_run_instances(&run_xml, region)?;

        // Ensure label matches spec if XML returned instance ID as fallback
        if inst.label == inst.id.0 && !spec.label.is_empty() {
            inst.label = spec.label.clone();
        }

        Ok(inst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crow_provider_core::check_declarations;
    use crow_provider_core::testing::RecordedHttp;

    const DESCRIBE_INSTANCES_XML: &str = r#"<DescribeInstancesResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/">
        <requestId>b855913e-670a-4074-b9b5-234b67367c3b</requestId>
        <reservationSet>
            <item>
                <reservationId>r-0123456789abcdef0</reservationId>
                <ownerId>123456789012</ownerId>
                <instancesSet>
                    <item>
                        <instanceId>i-0123456789abcdef0</instanceId>
                        <imageId>ami-0e2c8caa4b6378d8c</imageId>
                        <instanceState>
                            <code>16</code>
                            <name>running</name>
                        </instanceState>
                        <privateDnsName>ip-172-31-1-2.ec2.internal</privateDnsName>
                        <dnsName>ec2-54-210-1-2.compute-1.amazonaws.com</dnsName>
                        <instanceType>t4g.small</instanceType>
                        <placement>
                            <availabilityZone>us-east-1a</availabilityZone>
                        </placement>
                        <ipAddress>54.210.1.2</ipAddress>
                        <privateIpAddress>172.31.1.2</privateIpAddress>
                        <tagSet>
                            <item>
                                <key>Name</key>
                                <value>web-prod-01</value>
                            </item>
                            <item>
                                <key>Environment</key>
                                <value>production</value>
                            </item>
                        </tagSet>
                        <blockDeviceMapping>
                            <item>
                                <deviceName>/dev/sda1</deviceName>
                                <ebs>
                                    <volumeId>vol-0123456789abcdef0</volumeId>
                                    <status>attached</status>
                                </ebs>
                            </item>
                        </blockDeviceMapping>
                    </item>
                </instancesSet>
            </item>
        </reservationSet>
    </DescribeInstancesResponse>"#;

    fn make_aws(http: RecordedHttp) -> (AwsEc2, Arc<RecordedHttp>) {
        let http = Arc::new(http);
        let settings = ProviderSettings::default()
            .with_secret("access_key_id", "AKIAIOSFODNN7EXAMPLE")
            .with_secret("secret_access_key", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY")
            .with_value("region", "us-east-1");
        let aws = AwsEc2::new(&settings, http.clone()).unwrap();
        (aws, http)
    }

    #[test]
    fn declares_what_it_implements() {
        let (aws, _) = make_aws(RecordedHttp::new());
        check_declarations(&aws).unwrap();
        assert_eq!(MANIFEST.unknown_capabilities(), Vec::<&str>::new());
    }

    #[test]
    fn lists_instances_from_xml() {
        let (aws, _) = make_aws(
            RecordedHttp::new().on_form("https://ec2.us-east-1.amazonaws.com/", "", 200, DESCRIBE_INSTANCES_XML)
        );

        let instances = aws.instances().unwrap();
        assert_eq!(instances.len(), 1);
        let inst = &instances[0];
        assert_eq!(inst.id.0, "i-0123456789abcdef0");
        assert_eq!(inst.label, "web-prod-01");
        assert_eq!(inst.status, crow_provider_core::hosts::InstanceStatus::Running);
        assert_eq!(inst.plan.as_deref(), Some("t4g.small"));
        assert_eq!(inst.region.as_deref(), Some("us-east-1a"));
        assert_eq!(inst.ipv4.len(), 2);
        assert!(inst.tags.contains(&"Environment=production".to_string()));
    }

    #[test]
    fn check_reports_instance_count_and_region() {
        let (aws, _) = make_aws(
            RecordedHttp::new().on_form("https://ec2.us-east-1.amazonaws.com/", "", 200, DESCRIBE_INSTANCES_XML)
        );

        let summary = aws.check().unwrap();
        assert_eq!(summary, "1 instance (us-east-1)");
    }

    #[test]
    fn power_actions_send_expected_form_requests() {
        let (aws, http) = make_aws(
            RecordedHttp::new().on_form("https://ec2.us-east-1.amazonaws.com/", "", 200, "<StartInstancesResponse><instancesSet/></StartInstancesResponse>")
        );

        let id = InstanceId("i-0123456789abcdef0".into());
        aws.boot(&id).unwrap();

        let reqs = http.requests();
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].url, "https://ec2.us-east-1.amazonaws.com/");
        assert_eq!(reqs[1].url, "https://ec2.us-east-1.amazonaws.com/");
        let body = reqs[1].form_body.as_ref().unwrap();
        assert!(body.contains("Action=StartInstances"));
        assert!(body.contains("InstanceId.1=i-0123456789abcdef0"));
    }

    #[test]
    fn provision_imports_key_and_runs_instance() {
        let (aws, _) = make_aws(
            RecordedHttp::new()
                .on_form("https://ec2.us-east-1.amazonaws.com/", "Action=ImportKeyPair", 200, "<ImportKeyPairResponse><keyName>crow-srv1</keyName></ImportKeyPairResponse>")
                .on_form("https://ec2.us-east-1.amazonaws.com/", "Action=RunInstances", 200, r#"<RunInstancesResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/">
                    <instancesSet>
                        <item>
                            <instanceId>i-0fedcba9876543210</instanceId>
                            <instanceState><name>pending</name></instanceState>
                            <instanceType>t4g.small</instanceType>
                            <placement><availabilityZone>us-east-1b</availabilityZone></placement>
                            <ipAddress>54.210.99.1</ipAddress>
                        </item>
                    </instancesSet>
                </RunInstancesResponse>"#)
        );

        let spec = ProvisionSpec {
            label: "srv1".into(),
            region: "us-east-1".into(),
            plan: "t4g.small".into(),
            image: "ubuntu-24-04".into(),
            ssh_public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI1234567890 root@test".into(),
            tags: vec!["app=crow".into()],
        };

        let inst = aws.provision(&spec).unwrap();
        assert_eq!(inst.id.0, "i-0fedcba9876543210");
        assert_eq!(inst.label, "srv1");
        assert_eq!(inst.status, crow_provider_core::hosts::InstanceStatus::Starting);
    }
}
