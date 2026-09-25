//! The provider plugins compiled into this build (cargo features
//! `provider-*`), and how their data meets the rest of Crow.

use crow_provider_core::ProviderFactory;

/// Every compiled-in provider.
pub fn factories() -> Vec<ProviderFactory> {
    #[allow(unused_mut)]
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

/// The provider name the region tables use for plugin `name`, so an
/// instance's region code becomes a country and city.
pub fn region_provider(name: &str) -> Option<&'static str> {
    match name {
        "linode" => Some("Linode"),
        "upcloud" => Some("UpCloud"),
        _ => None,
    }
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
            assert!(region_provider(&m.plugin.name).is_some(), "{} has region tables", m.plugin.name);
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

    #[test]
    fn provider_regions_map_to_flags() {
        assert_eq!(crate::region::locate(region_provider("linode").unwrap(), "de-fra-2"), Some(("DE", "Frankfurt")));
        assert_eq!(crate::region::locate(region_provider("upcloud").unwrap(), "fi-hel1"), Some(("FI", "Helsinki")));
    }
}
