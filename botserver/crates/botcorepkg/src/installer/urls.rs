use super::*;

/// Components whose upstream project ships **no Windows build** (#1966).
///
/// Their `3rdparty.toml` entry used to carry a `url_win` that answered 404
/// (Forgejo, the Forgejo runner, the retired table editor) or fell back to
/// the Linux archive (Roundcube). A Windows install must refuse them with a reason instead of
/// downloading a broken URL or the wrong platform's binary.
/// `cache` (Valkey): no Windows build exists. The desktop server provides a
/// built-in RESP cache on the loopback interface (`embedded_cache.rs`), so
/// the external binary is never needed. Without this guard the installer
/// downloads the Linux tarball and waits forever for a process that cannot
/// start.
const NO_WINDOWS_BUILD: &[&str] = &["alm", "alm_ci", "cache", "table_editor", "webmail"];

pub(super) fn get_thirdparty_config() -> &'static ThirdPartyConfig {
    THIRDPARTY_CONFIG.get_or_init(|| {
        let toml_str = include_str!("../../../../3rdparty.toml");
        match toml::from_str::<ThirdPartyConfig>(toml_str) {
            Ok(config) => config,
            Err(e) => {
                log::error!("CRITICAL: Failed to parse embedded 3rdparty.toml: {e}");
                ThirdPartyConfig {
                    components: HashMap::new(),
                    models: HashMap::new(),
                }
            }
        }
    })
}

/// Whether `name` can be installed on the current platform.
///
/// Only Windows is restricted today; Linux and macOS keep every component.
#[must_use]
pub fn component_available_here(name: &str) -> bool {
    !(cfg!(target_os = "windows") && NO_WINDOWS_BUILD.contains(&name))
}

/// Human-readable reason for [`component_available_here`] returning `false`.
#[must_use]
pub fn unavailable_reason(name: &str) -> Option<&'static str> {
    if component_available_here(name) {
        return None;
    }
    Some("upstream publishes no Windows build; use the container image or run this component on Linux")
}

/// The `3rdparty.toml` entry that provides component `name`: the `vault`
/// component is served by the OpenBao binary.
#[must_use]
pub fn download_entry(name: &str) -> &str {
    match name {
        "vault" => "openbao",
        other => other,
    }
}

pub fn get_component_url(name: &str) -> Option<String> {
    if !component_available_here(name) {
        log::warn!(
            "Component '{name}' has no build for this platform; refusing its download URL"
        );
        return None;
    }
    get_thirdparty_config()
        .components
        .get(download_entry(name))
        .map(|c| c.effective_url().to_string())
}

pub fn get_model_url(name: &str) -> Option<String> {
    get_thirdparty_config()
        .models
        .get(name)
        .map(|c| c.effective_url().to_string())
}

/// Checksum pinned for an asset URL (#2001).
///
/// The container installer fetches the file *inside* the container and cannot
/// hash it on the host, so it needs the pinned digest without a
/// `DownloadCache` instance.
#[must_use]
pub fn checksum_for_url(url: &str) -> Option<String> {
    let config = get_thirdparty_config();
    config
        .components
        .values()
        .chain(config.models.values())
        .find_map(|component| {
            component
                .sha256_for_url(url)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_only_components_are_listed_and_others_are_not() {
        for name in NO_WINDOWS_BUILD {
            assert!(
                super::get_thirdparty_config().components.contains_key(*name),
                "guarded component {name} must exist in 3rdparty.toml"
            );
        }
        assert!(!NO_WINDOWS_BUILD.contains(&"vault"));
        assert!(!NO_WINDOWS_BUILD.contains(&"timeseries_db"));
    }

    #[test]
    fn availability_matches_the_platform() {
        // Mirrors the guard so a future cfg change cannot silently accept a
        // component that has no Windows archive.
        let expected = !cfg!(target_os = "windows");
        for name in NO_WINDOWS_BUILD {
            assert_eq!(component_available_here(name), expected, "{name}");
            if cfg!(target_os = "windows") {
                assert!(unavailable_reason(name).is_some());
                assert!(get_component_url(name).is_none());
            }
        }
        assert!(component_available_here("vault"));
        assert_eq!(unavailable_reason("vault"), None);
    }

    #[test]
    fn the_vault_component_downloads_pinned_openbao() {
        let config = super::get_thirdparty_config();
        assert!(!config.components.contains_key("vault"));
        let entry = config.components.get(download_entry("vault"));
        let Some(entry) = entry else {
            panic!("3rdparty.toml must carry the openbao entry");
        };
        let url = get_component_url("vault").unwrap_or_default();
        assert!(url.contains("github.com/openbao/openbao/releases/download/"), "{url}");
        assert!(!url.contains("hashicorp"), "{url}");
        assert_eq!(checksum_for_url(&url).map(|d| d.len()), Some(64));
        assert!(entry.effective_url().contains("openbao_"));
        assert_eq!(download_entry("tables"), "tables");
    }
}
