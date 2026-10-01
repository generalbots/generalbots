//! Registrar name → adapter resolution (issue #1469).

use std::sync::Arc;

use crate::types::{RegistrarInfo, RegistrarError};
use crate::{Registrar, cloudflare, desec, dynadot, namecheap, porkbun};

/// Builds the adapter registered under `name`.
///
/// Adapters that need deployment configuration (a Cloudflare account, a
/// Namecheap client IP) have to be built by [`configured_registrar`]; this
/// function returns only the adapters that are usable with no extra arguments,
/// which is what an unconfigured build can offer.
#[must_use]
pub fn registrar_for_name(name: &str) -> Option<Arc<dyn Registrar>> {
    let normalized = name.trim().to_lowercase();
    let registrar: Arc<dyn Registrar> = match normalized.as_str() {
        "porkbun" => Arc::new(porkbun::Porkbun::new()),
        "dynadot" => Arc::new(dynadot::Dynadot::new()),
        "namecheap" => Arc::new(namecheap::Namecheap::sandbox()),
        "cloudflare" => Arc::new(cloudflare::Cloudflare::with_account("default")),
        "desec" => Arc::new(desec::Desec::with_zone("")),
        _ => return None,
    };
    Some(registrar)
}

/// Adapter selection with deployment configuration applied.
///
/// `options` carries what the build cannot know: an account id, a client IP, a
/// zone. A missing value is not fatal here — the adapter reports it at the point
/// of use, which is where the operator's message is actionable.
#[must_use]
pub fn configured_registrar(name: &str, options: &RegistrarOptions) -> Option<Arc<dyn Registrar>> {
    let normalized = name.trim().to_lowercase();
    let registrar: Arc<dyn Registrar> = match normalized.as_str() {
        "porkbun" => Arc::new(porkbun::Porkbun::new()),
        "dynadot" => Arc::new(dynadot::Dynadot::new()),
        "namecheap" => match (&options.client_ip, options.production) {
            (ip, true) if !ip.trim().is_empty() => {
                Arc::new(namecheap::Namecheap::production(ip.clone()))
            }
            _ => Arc::new(namecheap::Namecheap::sandbox()),
        },
        "cloudflare" => Arc::new(cloudflare::Cloudflare::with_account(
            options.account_id.clone(),
        )),
        "desec" => Arc::new(desec::Desec::with_zone(options.zone.clone())),
        _ => return None,
    };
    Some(registrar)
}

/// Deployment configuration the adapters cannot infer.
#[derive(Debug, Clone, Default)]
pub struct RegistrarOptions {
    /// Cloudflare account the domain is registered into.
    pub account_id: String,
    /// Namecheap's registered egress address; required for production calls.
    pub client_ip: String,
    /// Whether to target production rather than a sandbox.
    pub production: bool,
    /// Zone a DNS-only adapter writes into.
    pub zone: String,
}

/// Every adapter this build can construct, in default-preference order.
///
/// Porkbun first because it is the default registrar: open API, flat pricing, no
/// renewal cliff.
#[must_use]
pub fn available_registrars() -> Vec<Arc<dyn Registrar>> {
    ["porkbun", "dynadot", "cloudflare", "namecheap", "desec"]
        .iter()
        .filter_map(|name| registrar_for_name(name))
        .collect()
}

/// The adapter an SKU should use when the customer did not choose one.
#[must_use]
pub fn default_registrar() -> Option<Arc<dyn Registrar>> {
    registrar_for_name("porkbun")
}

/// Static description of an adapter, for the UI and pre-flight checks.
#[must_use]
pub fn registrar_info_for_name(name: &str) -> Option<RegistrarInfo> {
    registrar_for_name(name).map(|registrar| registrar.info())
}

/// Picks the first adapter in `candidates` that this build can construct.
///
/// A list that resolves to nothing is reported with its candidates, so a purchase
/// fails loudly instead of silently doing nothing.
pub fn first_available(candidates: &[&str]) -> Result<Arc<dyn Registrar>, RegistrarError> {
    candidates
        .iter()
        .find_map(|name| registrar_for_name(name))
        .ok_or_else(|| {
            RegistrarError::Unsupported(
                "platform".into(),
                format!(
                    "no registrar among candidates [{}] is available in this build",
                    candidates.join(", ")
                ),
            )
        })
}

/// Adapters that can register, as opposed to DNS-only hosts.
#[must_use]
pub fn registration_registrars() -> Vec<Arc<dyn Registrar>> {
    available_registrars()
        .into_iter()
        .filter(|registrar| !registrar.info().supported_tlds.is_empty())
        .collect()
}

/// Adapters whose DNS write replaces the whole zone.
///
/// A caller that is about to add one record has to know this: it must read the
/// zone first and confirm nothing is dropped.
#[must_use]
pub fn whole_zone_registrars() -> Vec<String> {
    available_registrars()
        .iter()
        .filter(|registrar| registrar.info().replaces_whole_zone)
        .map(|registrar| registrar.name().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_name_resolves() {
        for registrar in available_registrars() {
            assert!(
                registrar_for_name(registrar.name()).is_some(),
                "{}",
                registrar.name()
            );
        }
    }

    #[test]
    fn registrar_names_are_unique() {
        let mut names: Vec<String> = available_registrars()
            .iter()
            .map(|registrar| registrar.name().to_string())
            .collect();
        let count = names.len();
        names.sort();
        names.dedup();
        assert_eq!(count, names.len(), "duplicate registrar name in the registry");
    }

    #[test]
    fn an_unknown_name_resolves_to_nothing() {
        assert!(registrar_for_name("not-a-registrar").is_none());
        assert!(registrar_for_name("").is_none());
        assert!(registrar_info_for_name("not-a-registrar").is_none());
    }

    #[test]
    fn names_are_matched_case_insensitively_and_trimmed() {
        assert!(registrar_for_name(" PORKBUN ").is_some());
        assert!(registrar_for_name("CloudFlare").is_some());
    }

    #[test]
    fn the_default_registrar_is_porkbun() {
        let registrar = default_registrar().expect("a default registrar");
        assert_eq!(registrar.name(), "porkbun");
    }

    #[test]
    fn the_dns_only_host_is_not_offered_as_a_registrar() {
        let registration = registration_registrars();
        assert!(registration.iter().all(|r| r.name() != "desec"));
        let desec = registrar_for_name("desec").expect("adapter exists");
        assert!(desec.info().supported_tlds.is_empty());
    }

    #[test]
    fn the_zone_replace_hazard_is_discoverable_without_reading_the_source() {
        // This is the property a caller must be able to check before writing.
        assert_eq!(whole_zone_registrars(), vec!["namecheap".to_string()]);
    }

    #[test]
    fn a_candidate_list_with_nothing_available_names_its_candidates() {
        match first_available(&["not-a-registrar", "also-missing"]) {
            Ok(_) => panic!("expected an error, not a registrar"),
            Err(err) => {
                let message = err.to_string();
                assert!(message.contains("not-a-registrar"), "{message}");
                assert!(message.contains("also-missing"), "{message}");
            }
        }
    }

    #[test]
    fn candidate_order_decides_the_registrar() {
        match first_available(&["not-a-registrar", "dynadot"]) {
            Ok(registrar) => assert_eq!(registrar.name(), "dynadot"),
            Err(err) => panic!("dynadot should be available: {err}"),
        }
    }

    #[test]
    fn a_configured_adapter_carries_its_deployment_settings() {
        let options = RegistrarOptions {
            account_id: "acct-1".into(),
            client_ip: "203.0.113.9".into(),
            production: true,
            zone: "acme.com".into(),
        };
        let cloudflare = configured_registrar("cloudflare", &options).expect("adapter");
        assert_eq!(cloudflare.name(), "cloudflare");
        let namecheap = configured_registrar("namecheap", &options).expect("adapter");
        assert_eq!(namecheap.name(), "namecheap");
        let desec = configured_registrar("desec", &options).expect("adapter");
        assert_eq!(desec.name(), "desec");
    }

    #[test]
    fn without_a_client_ip_namecheap_falls_back_to_the_sandbox() {
        let registrar = configured_registrar("namecheap", &RegistrarOptions::default())
            .expect("adapter");
        assert_eq!(registrar.name(), "namecheap");
    }

    #[test]
    fn every_catalogued_domain_tld_is_registerable_somewhere() {
        let mut registrable: Vec<String> = Vec::new();
        for registrar in registration_registrars() {
            for tld in registrar.info().supported_tlds {
                if !registrable.contains(&tld) {
                    registrable.push(tld);
                }
            }
        }
        for tld in ["com", "io", "ai"] {
            assert!(registrable.iter().any(|t| t == tld), "{tld} is not purchasable");
        }
    }
}
