//! A provider account's settings, as handed to its constructor: plain values
//! from Crow's settings store and secrets from the vault.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{ProviderError, SecretValue};

#[derive(Debug, Clone, Default)]
pub struct ProviderSettings {
    pub values: BTreeMap<String, Value>,
    pub secrets: BTreeMap<String, SecretValue>,
}

impl ProviderSettings {
    pub fn with_value(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn with_secret(mut self, key: &str, value: impl Into<String>) -> Self {
        self.secrets.insert(key.into(), SecretValue::new(value));
        self
    }

    /// A text setting, if set and not blank.
    pub fn string(&self, key: &str) -> Option<&str> {
        self.values.get(key)?.as_str().map(str::trim).filter(|s| !s.is_empty())
    }

    pub fn required_string(&self, key: &str) -> Result<&str, ProviderError> {
        self.string(key).ok_or_else(|| ProviderError::NotConfigured(key.into()))
    }

    pub fn secret(&self, key: &str) -> Option<&SecretValue> {
        self.secrets.get(key).filter(|s| !s.is_empty())
    }

    pub fn required_secret(&self, key: &str) -> Result<&SecretValue, ProviderError> {
        self.secret(key).ok_or_else(|| ProviderError::NotConfigured(key.into()))
    }
}
