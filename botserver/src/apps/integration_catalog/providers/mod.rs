mod developer;
mod featured;
mod finance;
mod lifestyle;
mod productivity;
mod small_business;
mod social_messaging;
mod startups;

use super::types::{
    ActionTemplate, AuthProfile, Category, Priority, ProviderSeed, Status, Strategy,
};

pub(super) const fn provider(
    id: &'static str,
    name: &'static str,
    category: Category,
    strategy: Strategy,
    status: Status,
    priority: Priority,
    module: Option<&'static str>,
    official_docs: Option<&'static str>,
    auth: &'static AuthProfile,
    actions: &'static [ActionTemplate],
) -> ProviderSeed {
    ProviderSeed {
        id,
        name,
        category,
        strategy,
        status,
        priority,
        module,
        official_docs,
        auth,
        actions,
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
    ]
    .into_iter()
    .flat_map(|group| group.iter())
    .collect()
}
