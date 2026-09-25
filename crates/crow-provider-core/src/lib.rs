//! Contracts for Crow's provider plugins.
//!
//! A provider talks to an outside service on the user's behalf. Its
//! manifest (a crow-config [`PluginManifest`]) names the **category** it
//! implements and the **capabilities** it supports; this crate defines what
//! those mean in code:
//!
//! - [`hosts`]: the `provider.hosts` contract (Linode, UpCloud, ...):
//!   list instances, power them, snapshot them.
//!
//! Every provider implements [`Provider`], and hands out the category traits
//! it supports through the `as_*` accessors. Crow offers an action only
//! when the accessor returns `Some`, and [`check_declarations`] keeps the
//! manifest and the code in agreement.
//!
//! # Design rules
//!
//! - **Sync.** Crow runs provider calls on its background executor, like
//!   every other command. No async runtime.
//! - **Data-shaped.** Plain structs in and out, serde-friendly, no lifetimes
//!   or HTTP-library types, so providers could later run out of process
//!   (WASM, JSON-RPC) behind the same traits.
//! - **HTTP is injected.** Providers build [`http::HttpRequest`]s and send
//!   them through the [`http::Http`] they're given. Crow's implementation
//!   uses the system `curl` with the whole request, credentials included,
//!   on stdin, so tokens never appear in argv, and no TLS stack is
//!   compiled in. Tests use recorded responses instead.
//! - **Secrets stay secret.** Credentials arrive as [`SecretValue`]s and go
//!   into requests as secret headers; `Debug` of a request prints
//!   `[secret]`, and no error message carries one.

pub mod error;
pub mod hosts;
pub mod curl;
pub mod http;
pub mod settings;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

use std::sync::Arc;

pub use crow_config_core::{PluginManifest, SecretValue};
pub use error::ProviderError;
pub use settings::ProviderSettings;

use hosts::{ListInstances, PowerControl, Snapshots};

/// Capability tags, as written in manifests.
pub mod capabilities {
    pub const INSTANCES_LIST: &str = "instances.list";
    pub const INSTANCES_POWER: &str = "instances.power";
    pub const SNAPSHOTS: &str = "snapshots";
}

/// A configured provider account.
pub trait Provider: Send + Sync {
    fn manifest(&self) -> &PluginManifest;

    /// A cheap authenticated call, for "Test connection". Returns a short
    /// human summary such as "3 instances".
    fn check(&self) -> Result<String, ProviderError>;

    fn as_list_instances(&self) -> Option<&dyn ListInstances> {
        None
    }
    fn as_power_control(&self) -> Option<&dyn PowerControl> {
        None
    }
    fn as_snapshots(&self) -> Option<&dyn Snapshots> {
        None
    }
}

/// The capabilities `provider` actually implements.
pub fn implemented_capabilities(provider: &dyn Provider) -> Vec<&'static str> {
    [
        (provider.as_list_instances().is_some(), capabilities::INSTANCES_LIST),
        (provider.as_power_control().is_some(), capabilities::INSTANCES_POWER),
        (provider.as_snapshots().is_some(), capabilities::SNAPSHOTS),
    ]
    .into_iter()
    .filter_map(|(has, cap)| has.then_some(cap))
    .collect()
}

/// Checks that the manifest declares exactly the capabilities the code
/// implements, so the UI never offers an action that isn't there, nor hides
/// one that is. Every provider crate runs this in its tests.
pub fn check_declarations(provider: &dyn Provider) -> Result<(), String> {
    let manifest = provider.manifest();
    manifest.validate().map_err(|e| e.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "))?;
    let implemented = implemented_capabilities(provider);
    let declared: Vec<&str> = manifest.plugin.capabilities.iter().map(String::as_str).collect();
    let undeclared: Vec<&str> = implemented.iter().copied().filter(|c| !declared.contains(c)).collect();
    let unimplemented: Vec<&str> = declared.iter().copied().filter(|c| !implemented.contains(c)).collect();
    if undeclared.is_empty() && unimplemented.is_empty() {
        Ok(())
    } else {
        Err(format!("{}: implemented but not declared {undeclared:?}; declared but not implemented {unimplemented:?}", manifest.plugin.name))
    }
}

/// Builds a provider account from its settings and the HTTP to use.
pub type BuildProvider = fn(ProviderSettings, Arc<dyn http::Http>) -> Result<Box<dyn Provider>, ProviderError>;

/// How Crow builds a provider: its manifest (for Settings and capability
/// checks before any account exists) and a constructor.
#[derive(Clone, Copy)]
pub struct ProviderFactory {
    pub manifest: fn() -> &'static PluginManifest,
    pub build: BuildProvider,
}
