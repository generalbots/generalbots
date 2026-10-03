//! Provider name → adapter resolution (issue #1470).
//!
//! The cloud API used to `match` on exactly the two providers that happened to
//! be in the default feature set, which left `runpod.rs` and `vultr.rs` as
//! complete but unreachable code. Resolution now lives here, so adding an
//! adapter is one `#[cfg]` line in `lib.rs` plus one arm below, and a name that
//! has no compiled adapter reports itself instead of failing at the `match`.

use std::sync::Arc;

use crate::{ComputeProvider, ProviderError, ProviderInfo};

/// Builds the adapter registered under `name`.
///
/// Returns `None` for a name this build does not include, which lets a caller
/// walk a fallback chain instead of failing outright.
#[must_use]
pub fn provider_for_name(name: &str) -> Option<Arc<dyn ComputeProvider>> {
    let normalized = name.trim().to_lowercase();
    let provider: Arc<dyn ComputeProvider> = match normalized.as_str() {
        #[cfg(feature = "providers-vast")]
        "vast" | "vastai" | "vast.ai" => Arc::new(crate::vast::VastAiProvider::new()),
        #[cfg(feature = "providers-contabo")]
        "contabo" => Arc::new(crate::contabo::ContaboProvider::new()),
        #[cfg(feature = "providers-runpod")]
        "runpod" => Arc::new(crate::runpod::RunPodProvider::new()),
        #[cfg(feature = "providers-vultr")]
        "vultr" => Arc::new(crate::vultr::VultrProvider::new()),
        #[cfg(feature = "providers-hetzner")]
        "hetzner" => Arc::new(crate::hetzner::HetznerProvider::new()),
        #[cfg(feature = "providers-ovh")]
        "ovh" | "ovhcloud" => Arc::new(crate::ovh::OvhProvider::new()),
        #[cfg(feature = "providers-digitalocean")]
        "digitalocean" | "do" => Arc::new(crate::digitalocean::DigitalOceanProvider::new()),
        #[cfg(feature = "providers-oracle")]
        "oracle" | "oci" => Arc::new(crate::oracle::OracleProvider::new()),
        _ => return None,
    };
    Some(provider)
}

/// Every adapter compiled into this build, cheapest-CPU-first.
#[must_use]
pub fn available_providers() -> Vec<Arc<dyn ComputeProvider>> {
    // `cfg!` rather than `#[cfg]` so the list stays one expression; every entry
    // is a compile-time constant either way.
    let catalog: [(&str, bool); 8] = [
        ("hetzner", cfg!(feature = "providers-hetzner")),
        ("digitalocean", cfg!(feature = "providers-digitalocean")),
        ("oracle", cfg!(feature = "providers-oracle")),
        ("vultr", cfg!(feature = "providers-vultr")),
        ("contabo", cfg!(feature = "providers-contabo")),
        ("vast", cfg!(feature = "providers-vast")),
        ("runpod", cfg!(feature = "providers-runpod")),
        ("ovh", cfg!(feature = "providers-ovh")),
    ];
    catalog
        .iter()
        .filter(|(_, compiled)| *compiled)
        .filter_map(|(name, _)| provider_for_name(name))
        .collect()
}

/// Static description of one adapter, for callers that only need the catalog.
#[must_use]
pub fn provider_info_for_name(name: &str) -> Option<ProviderInfo> {
    provider_for_name(name).map(|provider| provider.info())
}

/// The provider an SKU should use when the request does not name one.
///
/// Flat-rate hosts come before the GPU marketplaces, whose per-bid pricing is
/// frequently higher than a plain VPS for CPU work.
#[must_use]
pub fn default_provider_for(gpu_type: Option<&str>) -> Option<Arc<dyn ComputeProvider>> {
    let cpu_first = ["hetzner", "digitalocean", "oracle", "vultr", "contabo", "vast"];
    let gpu_first = ["ovh", "vast", "runpod", "contabo"];
    let preferred: &[&str] = if gpu_type.is_some() { &gpu_first } else { &cpu_first };
    preferred.iter().find_map(|name| provider_for_name(name))
}

/// Builds the first adapter in `candidates` that this build includes.
///
/// A candidate list that resolves to nothing is reported as an error naming the
/// candidates, so a SKU with no reachable provider fails loudly instead of
/// silently succeeding.
pub fn first_available(candidates: &[&str]) -> Result<Arc<dyn ComputeProvider>, ProviderError> {
    candidates
        .iter()
        .find_map(|name| provider_for_name(name))
        .ok_or_else(|| {
            ProviderError::NotFound(format!(
                "no compiled compute provider among candidates [{}]",
                candidates.join(", ")
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_name_resolves() {
        for name in available_providers().iter().map(|p| p.name().to_string()) {
            assert!(provider_for_name(&name).is_some(), "{name}");
        }
    }

    #[test]
    fn provider_names_are_unique() {
        let mut names: Vec<String> = available_providers().iter().map(|p| p.name().into()).collect();
        let count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(count, names.len(), "duplicate provider name in registry");
    }

    #[test]
    fn an_unknown_name_resolves_to_nothing_rather_than_panicking() {
        assert!(provider_for_name("not-a-provider").is_none());
        assert!(provider_for_name("").is_none());
        assert!(provider_info_for_name("not-a-provider").is_none());
    }

    #[test]
    fn every_adapter_reports_a_non_empty_catalog() {
        for provider in available_providers() {
            let info = provider.info();
            assert!(!info.name.is_empty(), "provider name is empty");
            assert!(!info.display_name.is_empty(), "{} display name", info.name);
            assert!(!info.regions.is_empty(), "{} has no regions", info.name);
        }
    }

    #[test]
    fn a_sku_with_no_reachable_provider_reports_its_candidates() {
        match first_available(&["not-a-provider", "also-missing"]) {
            Ok(_) => panic!("expected an error, not a provider"),
            Err(err) => {
                let message = err.to_string();
                assert!(message.contains("not-a-provider"), "{message}");
                assert!(message.contains("also-missing"), "{message}");
            }
        }
    }

    #[test]
    fn candidate_order_decides_the_provider() {
        match first_available(&["not-a-provider", "vast"]) {
            Ok(provider) => assert_eq!(provider.name(), "vast"),
            Err(err) => panic!("vast should be compiled: {err}"),
        }
    }

    #[test]
    fn the_default_provider_matches_the_workload() {
        if available_providers().is_empty() {
            return;
        }
        let cpu = default_provider_for(None).map(|p| p.name().to_string());
        let gpu = default_provider_for(Some("A100")).map(|p| p.name().to_string());
        assert!(cpu.is_some(), "no default CPU provider");
        assert!(gpu.is_some(), "no default GPU provider");
        assert_ne!(cpu, gpu, "a GPU request must not default to a CPU-first host");
    }
}
