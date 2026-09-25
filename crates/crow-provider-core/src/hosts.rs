//! The `provider.hosts` contract: where servers come from.
//!
//! | capability        | trait            |
//! |-------------------|------------------|
//! | `instances.list`  | [`ListInstances`] |
//! | `instances.power` | [`PowerControl`]  |
//! | `snapshots`       | [`Snapshots`]     |

use std::collections::BTreeMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ProviderError;

/// The provider's id for an instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstanceId(pub String);

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Rebooting,
    Provisioning,
    /// Anything else, in the provider's own words.
    Other(String),
}

impl InstanceStatus {
    /// Whether a power action makes sense right now.
    pub fn is_settled(&self) -> bool {
        matches!(self, Self::Running | Self::Stopped)
    }
}

/// A server as the provider sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub id: InstanceId,
    pub label: String,
    pub status: InstanceStatus,
    /// Public addresses first.
    pub ipv4: Vec<IpAddr>,
    pub ipv6: Vec<IpAddr>,
    /// The provider's region code (e.g. `de-fra-2`, `de-fra1`). Crow maps it
    /// to a country and city with its region tables.
    pub region: Option<String>,
    /// Size or plan (e.g. `g6-standard-2`).
    pub plan: Option<String>,
    pub tags: Vec<String>,
    /// Provider-specific details worth showing, kept out of the contract.
    pub extra: BTreeMap<String, Value>,
}

impl Instance {
    /// Whether `ip` is one of the instance's addresses.
    pub fn has_ip(&self, ip: IpAddr) -> bool {
        self.ipv4.contains(&ip) || self.ipv6.contains(&ip)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SnapshotId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub label: Option<String>,
    /// e.g. "pending", "successful", in the provider's words.
    pub status: String,
    /// When it was taken, as the provider reports it (ISO 8601).
    pub created: Option<String>,
}

/// `instances.list`
pub trait ListInstances: Send + Sync {
    fn instances(&self) -> Result<Vec<Instance>, ProviderError>;

    fn instance(&self, id: &InstanceId) -> Result<Instance, ProviderError> {
        self.instances()?.into_iter().find(|i| &i.id == id).ok_or_else(|| ProviderError::NotFound(format!("instance {id}")))
    }
}

/// `instances.power`. Each call returns once the provider accepted the
/// request; the instance's status shows progress.
pub trait PowerControl: Send + Sync {
    fn boot(&self, id: &InstanceId) -> Result<(), ProviderError>;
    fn reboot(&self, id: &InstanceId) -> Result<(), ProviderError>;
    fn shutdown(&self, id: &InstanceId) -> Result<(), ProviderError>;
}

/// `snapshots`: whole-server restore points taken by the provider.
pub trait Snapshots: Send + Sync {
    fn snapshots(&self, id: &InstanceId) -> Result<Vec<Snapshot>, ProviderError>;
    /// Starts a snapshot and returns it (usually still pending).
    fn snapshot(&self, id: &InstanceId, label: &str) -> Result<Snapshot, ProviderError>;
}
