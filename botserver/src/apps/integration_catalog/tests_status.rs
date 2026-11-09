//! #2048/#2049/#2050 — derived provider status tests.
//!
//! Split from `tests.rs` so that file stays under the size limit.

use super::types::Status;
use super::{provider_by_id, providers};

/// #2048 — the provider badge is derived from the adapter registry, so a seed
/// literal can never contradict what the registry actually executes.
#[test]
fn provider_status_is_derived_and_never_contradicts_the_registry() {
    for seed in super::providers::all() {
        let item = super::expand(seed);
        let implemented = item.actions.iter().filter(|action| action.implemented).count();
        assert_eq!(
            item.status,
            super::derived_status(seed, implemented, item.adapter_present),
            "{} status drifted from the derived value",
            seed.id
        );
        if item.status == Status::Built {
            assert!(
                item.adapter_present && item.adapter_action_count > 0,
                "{} renders built without an adapter",
                seed.id
            );
            assert!(
                item.actions.iter().all(|action| action.implemented),
                "{} renders built with unimplemented actions",
                seed.id
            );
        }
        if item.status == Status::Unsupported {
            assert!(
                !item.adapter_present && !item.llm_available && seed.module.is_none(),
                "{} renders unsupported although a surface exists",
                seed.id
            );
        }
        if item.adapter_present {
            assert_ne!(
                item.status,
                Status::Unsupported,
                "{} has an adapter but renders unsupported",
                seed.id
            );
        }
        assert_eq!(
            item.adapter_action_count > 0,
            item.adapter_present,
            "{} adapter flags disagree",
            seed.id
        );
    }
}

/// #2048 — the four seeds that were stale: their adapters are registered, so
/// none of them may render as unsupported.
#[cfg(feature = "integrations")]
#[test]
fn providers_with_a_live_adapter_are_not_unsupported() {
    let ids = [
        "eight_sleep",
        "linkedin_ads",
        "robinhood",
        "youtube_analytics",
    ];
    let found: Vec<_> = ids.iter().filter_map(|id| provider_by_id(id)).collect();
    assert_eq!(found.len(), ids.len(), "a catalog provider is missing");
    for provider in found {
        assert!(
            provider.adapter_present,
            "{} adapter must be registered",
            provider.id
        );
        assert_ne!(
            provider.status,
            Status::Unsupported,
            "{} has an adapter and must not render unsupported",
            provider.id
        );
    }
}

/// #2049/#2050 — Basecamp and Twitch claimed built while nothing was
/// registered. Both specs are wired now; each advertises fewer actions than
/// the catalog's shared template, so the honest badge is partial, not built.
#[cfg(feature = "integrations")]
#[test]
fn basecamp_and_twitch_are_wired_and_render_partial() {
    let ids = ["basecamp", "twitch"];
    let found: Vec<_> = ids.iter().filter_map(|id| provider_by_id(id)).collect();
    assert_eq!(found.len(), ids.len(), "a catalog provider is missing");
    for provider in found {
        let id = provider.id;
        assert!(provider.adapter_present, "{id} must have a registered adapter");
        assert_ne!(provider.status, Status::Built, "{id} cannot render built");
        assert_eq!(
            provider.status,
            Status::Partial,
            "{id} must render partial while its catalog template is wider than its adapter"
        );
        assert!(
            provider.actions.iter().any(|action| action.implemented),
            "{id} must have at least one implementable catalog action"
        );
        assert!(
            provider.actions.iter().any(|action| !action.implemented),
            "{id} must still report the unimplemented actions honestly"
        );
    }
}

/// #2051 — the guard that makes the badge honest: a seed may only render
/// `built` when every advertised action key is answered by the adapter. The
/// failure message names the provider and the unanswered keys, so a new seed
/// that reuses a 6-action template with a 2-action adapter fails review.
#[test]
fn no_provider_is_built_with_an_unanswered_action() {
    for seed in providers::all() {
        let item = super::expand(seed);
        let missing: Vec<&str> = item
            .actions
            .iter()
            .filter(|action| !action.implemented)
            .map(|action| action.name.as_str())
            .collect();
        if item.status == Status::Built {
            assert!(
                missing.is_empty(),
                "{} renders built but does not answer: {}",
                seed.id,
                missing.join(", ")
            );
        } else if !missing.is_empty() && item.adapter_present {
            // Partial is the honest badge; the payload already names the gap
            // per action (implemented:false), which is what the UI greys out.
            assert_eq!(item.status, Status::Partial, "{} status must be partial", seed.id);
        }
    }
}

/// #2051 — the derivation rule itself: fewer implemented actions than
/// advertised is partial, never built; zero adapter and no module is
/// unsupported.
#[test]
fn partial_coverage_never_renders_built() {
    let seeds = providers::all();
    let seed = match seeds.iter().find(|seed| seed.id == "amplitude") {
        Some(seed) => *seed,
        None => return,
    };
    let advertised = seed.actions.len();
    assert!(advertised >= 2, "amplitude template must have actions");
    assert_eq!(
        super::derived_status(seed, 2, true),
        Status::Partial,
        "2 of {advertised} actions must be partial"
    );
    assert_eq!(
        super::derived_status(seed, advertised, true),
        Status::Built,
        "full coverage must be built"
    );
    assert_eq!(
        super::derived_status(seed, 0, false),
        Status::Unsupported,
        "no adapter and no module must be unsupported"
    );
}

