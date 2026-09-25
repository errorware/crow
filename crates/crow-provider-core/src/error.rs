//! What can go wrong talking to a provider, in words an operator can act on.

/// A provider call's failure. Messages never contain credentials.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    #[error("not set up yet: {0} is missing")]
    NotConfigured(String),
    #[error("the provider rejected the credentials: {0}")]
    Auth(String),
    #[error("rate limited by the provider{}", .retry_after.map(|s| format!("; try again in {s}s")).unwrap_or_default())]
    RateLimited { retry_after: Option<u64> },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("couldn't reach the provider: {0}")]
    Network(String),
    #[error("the provider said ({status}): {message}")]
    Provider { status: u16, message: String },
    #[error("the provider answered with something unexpected: {0}")]
    Unexpected(String),
    #[error("{0}")]
    Unsupported(String),
}
