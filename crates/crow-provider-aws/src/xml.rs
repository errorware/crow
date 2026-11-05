//! EC2 XML response parsing using roxmltree.

use std::collections::BTreeMap;
use std::net::IpAddr;

use crow_provider_core::hosts::{Instance, InstanceId, InstanceStatus, Snapshot, SnapshotId};
use crow_provider_core::ProviderError;
use roxmltree::{Document, Node};

/// Parses AWS EC2 error XML:
/// `<Response><Errors><Error><Code>...</Code><Message>...</Message></Error></Errors></Response>`
pub fn parse_ec2_error(status: u16, xml: &str) -> ProviderError {
    if let Ok(doc) = Document::parse(xml) {
        if let Some(err_node) = doc.descendants().find(|n| n.has_tag_name("Error")) {
            let code = err_node
                .children()
                .find(|n| n.has_tag_name("Code"))
                .and_then(|n| n.text())
                .unwrap_or("");
            let message = err_node
                .children()
                .find(|n| n.has_tag_name("Message"))
                .and_then(|n| n.text())
                .unwrap_or(xml);

            return match code {
                "AuthFailure" | "SignatureDoesNotMatch" | "InvalidClientTokenId" | "UnauthorizedOperation" => {
                    ProviderError::Auth(format!("{code}: {message}"))
                }
                "RequestLimitExceeded" => ProviderError::RateLimited { retry_after: None },
                "InvalidInstanceID.NotFound" | "InvalidSnapshot.NotFound" => {
                    ProviderError::NotFound(message.to_string())
                }
                _ => ProviderError::Provider {
                    status,
                    message: format!("{code}: {message}"),
                },
            };
        }
    }

    ProviderError::Provider {
        status,
        message: xml.to_string(),
    }
}

/// Helper to get direct text child of a node.
fn child_text<'a>(node: &Node<'a, 'a>, tag: &str) -> Option<&'a str> {
    node.children().find(|n| n.has_tag_name(tag)).and_then(|n| n.text())
}

/// Map EC2 state string to InstanceStatus.
pub fn map_instance_state(state: &str) -> InstanceStatus {
    match state {
        "running" => InstanceStatus::Running,
        "stopped" => InstanceStatus::Stopped,
        "pending" => InstanceStatus::Starting,
        "stopping" => InstanceStatus::Stopping,
        "shutting-down" => InstanceStatus::Stopping,
        "rebooting" => InstanceStatus::Rebooting,
        other => InstanceStatus::Other(other.to_string()),
    }
}

/// Helper to parse a single `<item>` node representing an EC2 instance.
pub fn parse_instance_node(node: &Node, default_region: &str) -> Option<Instance> {
    let raw_id = child_text(node, "instanceId")?;
    let id = InstanceId(raw_id.to_string());
    let plan = child_text(node, "instanceType").map(|s| s.to_string());

    let state = node
        .children()
        .find(|n| n.has_tag_name("instanceState"))
        .and_then(|s| child_text(&s, "name"))
        .unwrap_or("unknown");

    let status = map_instance_state(state);

    let az = node
        .children()
        .find(|n| n.has_tag_name("placement"))
        .and_then(|p| child_text(&p, "availabilityZone"));

    let region = az.map(|z| z.to_string()).or_else(|| {
        if !default_region.is_empty() {
            Some(default_region.to_string())
        } else {
            None
        }
    });

    let mut label = raw_id.to_string();
    let mut tags = Vec::new();

    if let Some(tag_set) = node.children().find(|n| n.has_tag_name("tagSet")) {
        for tag_item in tag_set.children().filter(|n| n.has_tag_name("item")) {
            let key = child_text(&tag_item, "key").unwrap_or("");
            let val = child_text(&tag_item, "value").unwrap_or("");
            if key.eq_ignore_ascii_case("name") && !val.is_empty() {
                label = val.to_string();
            } else if !key.is_empty() {
                tags.push(format!("{key}={val}"));
            }
        }
    }

    let mut ipv4 = Vec::new();
    if let Some(pub_ip) = child_text(node, "ipAddress") {
        if let Ok(ip) = pub_ip.parse::<IpAddr>() {
            ipv4.push(ip);
        }
    }
    if let Some(priv_ip) = child_text(node, "privateIpAddress") {
        if let Ok(ip) = priv_ip.parse::<IpAddr>() {
            if !ipv4.contains(&ip) {
                ipv4.push(ip);
            }
        }
    }

    let mut ipv6 = Vec::new();
    if let Some(nic_set) = node.children().find(|n| n.has_tag_name("networkInterfaceSet")) {
        for nic in nic_set.children().filter(|n| n.has_tag_name("item")) {
            if let Some(ipv6_set) = nic.children().find(|n| n.has_tag_name("ipv6AddressesSet")) {
                for ip_item in ipv6_set.children().filter(|n| n.has_tag_name("item")) {
                    if let Some(addr) = child_text(&ip_item, "ipv6Address") {
                        if let Ok(ip) = addr.parse::<IpAddr>() {
                            if !ipv6.contains(&ip) {
                                ipv6.push(ip);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut extra = BTreeMap::new();
    if let Some(zone) = az {
        extra.insert("availability_zone".to_string(), serde_json::Value::String(zone.to_string()));
    }

    Some(Instance {
        id,
        label,
        status,
        ipv4,
        ipv6,
        region,
        plan,
        tags,
        extra,
    })
}

/// Parses EC2 `<DescribeInstancesResponse>` XML.
pub fn parse_describe_instances(xml: &str, default_region: &str) -> Result<Vec<Instance>, ProviderError> {
    let doc = Document::parse(xml).map_err(|e| ProviderError::Unexpected(e.to_string()))?;
    let mut instances = Vec::new();

    for res in doc.descendants().filter(|n| n.has_tag_name("instancesSet")) {
        for item in res.children().filter(|n| n.has_tag_name("item")) {
            if let Some(inst) = parse_instance_node(&item, default_region) {
                // Filter out terminated instances so the fleet view stays clean
                if !matches!(inst.status, InstanceStatus::Other(ref s) if s == "terminated") {
                    instances.push(inst);
                }
            }
        }
    }

    Ok(instances)
}

/// Extracts the root volume ID for a given instance from `<DescribeInstancesResponse>`.
pub fn parse_root_volume_id(xml: &str, instance_id: &str) -> Result<String, ProviderError> {
    let doc = Document::parse(xml).map_err(|e| ProviderError::Unexpected(e.to_string()))?;

    for inst_node in doc.descendants().filter(|n| n.has_tag_name("item")) {
        if child_text(&inst_node, "instanceId") == Some(instance_id) {
            let root_dev = child_text(&inst_node, "rootDeviceName").unwrap_or("/dev/xvda");
            if let Some(bdm) = inst_node.children().find(|n| n.has_tag_name("blockDeviceMapping")) {
                for dev_item in bdm.children().filter(|n| n.has_tag_name("item")) {
                    let dev_name = child_text(&dev_item, "deviceName").unwrap_or("");
                    if dev_name == root_dev || dev_name.contains("sda") || dev_name.contains("xvda") {
                        if let Some(ebs) = dev_item.children().find(|n| n.has_tag_name("ebs")) {
                            if let Some(vol_id) = child_text(&ebs, "volumeId") {
                                return Ok(vol_id.to_string());
                            }
                        }
                    }
                }
                // Fallback to any attached EBS volume
                for dev_item in bdm.children().filter(|n| n.has_tag_name("item")) {
                    if let Some(ebs) = dev_item.children().find(|n| n.has_tag_name("ebs")) {
                        if let Some(vol_id) = child_text(&ebs, "volumeId") {
                            return Ok(vol_id.to_string());
                        }
                    }
                }
            }
        }
    }

    Err(ProviderError::NotFound(format!("No EBS volume found for instance {instance_id}")))
}

/// Parses `<DescribeSnapshotsResponse>`.
pub fn parse_describe_snapshots(xml: &str) -> Result<Vec<Snapshot>, ProviderError> {
    let doc = Document::parse(xml).map_err(|e| ProviderError::Unexpected(e.to_string()))?;
    let mut snapshots = Vec::new();

    for set in doc.descendants().filter(|n| n.has_tag_name("snapshotSet")) {
        for item in set.children().filter(|n| n.has_tag_name("item")) {
            let id = child_text(&item, "snapshotId").unwrap_or("").to_string();
            if id.is_empty() {
                continue;
            }

            let status = child_text(&item, "status").unwrap_or("completed").to_string();

            let mut label = child_text(&item, "description").map(|s| s.to_string());
            if label.as_deref().unwrap_or("").is_empty() {
                if let Some(tag_set) = item.children().find(|n| n.has_tag_name("tagSet")) {
                    for tag_item in tag_set.children().filter(|n| n.has_tag_name("item")) {
                        let key = child_text(&tag_item, "key").unwrap_or("");
                        let val = child_text(&tag_item, "value").unwrap_or("");
                        if key.eq_ignore_ascii_case("name") && !val.is_empty() {
                            label = Some(val.to_string());
                            break;
                        }
                    }
                }
            }

            let created = child_text(&item, "startTime").map(|s| s.to_string());

            snapshots.push(Snapshot {
                id: SnapshotId(id),
                label,
                status,
                created,
            });
        }
    }

    Ok(snapshots)
}

/// Parses `<CreateSnapshotResponse>`.
pub fn parse_create_snapshot(xml: &str) -> Result<Snapshot, ProviderError> {
    let doc = Document::parse(xml).map_err(|e| ProviderError::Unexpected(e.to_string()))?;
    let root = doc.root_element();
    let id = child_text(&root, "snapshotId")
        .ok_or_else(|| ProviderError::Unexpected("Missing snapshotId in CreateSnapshotResponse".into()))?
        .to_string();

    let label = child_text(&root, "description").map(|s| s.to_string());
    let status = child_text(&root, "status").unwrap_or("pending").to_string();
    let created = child_text(&root, "startTime").map(|s| s.to_string());

    Ok(Snapshot {
        id: SnapshotId(id),
        label,
        status,
        created,
    })
}

/// Parses `<RunInstancesResponse>` into an `Instance`.
pub fn parse_run_instances(xml: &str, default_region: &str) -> Result<Instance, ProviderError> {
    let doc = Document::parse(xml).map_err(|e| ProviderError::Unexpected(e.to_string()))?;
    for set in doc.descendants().filter(|n| n.has_tag_name("instancesSet")) {
        for item in set.children().filter(|n| n.has_tag_name("item")) {
            if let Some(inst) = parse_instance_node(&item, default_region) {
                return Ok(inst);
            }
        }
    }
    Err(ProviderError::Unexpected("No instance returned in RunInstancesResponse".into()))
}
