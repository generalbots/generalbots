mod developer;
mod featured;
mod finance;
mod lifestyle;
mod llm;
mod productivity;
mod small_business;
mod social_messaging;
mod startups;

use super::types::{ActionTemplate, AuthProfile, Category, Priority, ProviderSeed, Strategy};

/// Everything a catalog provider declares beyond its identity, grouped so
/// [`provider`] keeps a short argument list.
///
/// #2048 — there is deliberately no `status` here: the badge is derived from
/// the adapter registry by `super::derived_status`, so a seed cannot advertise
/// a capability it does not have. `planned` is the single explicit exception.
pub(super) struct ProviderSpec {
    pub(super) category: Category,
    pub(super) strategy: Strategy,
    pub(super) planned: bool,
    pub(super) priority: Priority,
    pub(super) module: Option<&'static str>,
    pub(super) official_docs: Option<&'static str>,
    pub(super) auth: &'static AuthProfile,
    pub(super) actions: &'static [ActionTemplate],
}

pub(super) const fn provider(
    id: &'static str,
    name: &'static str,
    spec: ProviderSpec,
) -> ProviderSeed {
    ProviderSeed {
        id,
        name,
        category: spec.category,
        strategy: spec.strategy,
        planned: spec.planned,
        priority: spec.priority,
        module: spec.module,
        official_docs: spec.official_docs,
        auth: spec.auth,
        actions: spec.actions,
        llm_available: false,
    }
}

pub(crate) fn all() -> Vec<&'static ProviderSeed> {
    [
        featured::PROVIDERS,
        productivity::PROVIDERS,
        developer::PROVIDERS,
        startups::PROVIDERS,
        small_business::PROVIDERS,
        finance::PROVIDERS,
        social_messaging::PROVIDERS,
        lifestyle::PROVIDERS,
        llm::PROVIDERS,
    ]
    .into_iter()
    .flat_map(|group| group.iter())
    .collect()
}
