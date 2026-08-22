mod actions;
mod auth;
mod providers;
mod routes;
mod types;

pub use types::{CatalogResponse, LlmAction, LlmProviderSummary, ProviderItem};

use axum::Router;

use types::{
    ActionTemplate, CatalogTotals, Category, CategorySummary, ProviderAction, ProviderSeed,
};

pub fn register<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    routes::register(router)
}

/// Truthful implementation flag (#950): an action is implemented only when
/// the provider seed is LLM-available AND a live adapter in the
/// botintegrations registry advertises the exact action key. The catalog can
/// therefore never advertise an action that would fail with
/// `action_not_available`.
fn action_is_implemented(seed: &ProviderSeed, action: &ActionTemplate) -> bool {
    seed.llm_available && registry_action_names(seed.id).contains(&action.key)
}

/// Action keys backed by a live adapter, keyed by provider id.
#[cfg(feature = "integrations")]
fn registry_action_names(provider_id: &str) -> Vec<&'static str> {
    botintegrations::providers::implemented_action_names(provider_id).to_vec()
}

/// Without the `integrations` feature no adapter is compiled in, so no
/// catalog action may advertise itself as implemented.
#[cfg(not(feature = "integrations"))]
fn registry_action_names(provider_id: &str) -> Vec<&'static str> {
    log::debug!("adapter registry unavailable in this build for {provider_id}");
    Vec::new()
}

fn expand(seed: &ProviderSeed) -> ProviderItem {
    let actions: Vec<ProviderAction> = seed
        .actions
        .iter()
        .map(|action| ProviderAction {
            name: format!("{}.{}", seed.id, action.key),
            verb: action.verb,
            label: action.label,
            summary: action.summary,
            params: action.params,
            risk: action.risk,
            requires_approval: action.requires_approval,
            surfaces: action.surfaces,
            implemented: action_is_implemented(seed, action),
        })
        .collect();
    ProviderItem {
        id: seed.id,
        name: seed.name,
        category: seed.category,
        strategy: seed.strategy,
        status: seed.status,
        priority: seed.priority,
        module: seed.module,
        official_docs: seed.official_docs,
        auth: *seed.auth,
        action_count: actions.len(),
        actions,
        llm_available: seed.llm_available,
    }
}

fn category_matches(seed: &ProviderSeed, filter: &str) -> bool {
    seed.category.as_str().eq_ignore_ascii_case(filter)
        || seed.category.label().eq_ignore_ascii_case(filter)
}

fn query_matches(seed: &ProviderSeed, query: &str) -> bool {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() {
        return true;
    }
    let provider_haystack = format!(
        "{} {} {}",
        seed.id,
        seed.name.to_lowercase(),
        seed.category.as_str()
    );
    provider_haystack.contains(&normalized)
        || seed.actions.iter().any(|action| {
            let action_haystack = format!(
                "{}.{} {} {} {}",
                seed.id,
                action.key,
                action.verb,
                action.label.to_lowercase(),
                action.summary.to_lowercase()
            );
            action_haystack.contains(&normalized)
        })
}

fn matching_seeds(
    query: Option<&str>,
    category: Option<&str>,
    status: Option<&str>,
) -> Vec<&'static ProviderSeed> {
    providers::all()
        .into_iter()
        .filter(|seed| query.is_none_or(|value| query_matches(seed, value)))
        .filter(|seed| category.is_none_or(|value| category_matches(seed, value)))
        .filter(|seed| status.is_none_or(|value| seed.status.as_str().eq_ignore_ascii_case(value)))
        .collect()
}

pub fn search(
    query: Option<&str>,
    category: Option<&str>,
    status: Option<&str>,
) -> CatalogResponse {
    let providers: Vec<ProviderItem> = matching_seeds(query, category, status)
        .into_iter()
        .map(expand)
        .collect();
    let provider_count = providers.len();
    let action_count = providers.iter().map(|provider| provider.action_count).sum();
    let implemented_actions = providers
        .iter()
        .flat_map(|provider| provider.actions.iter())
        .filter(|action| action.implemented)
        .count();
    let categories = Category::ALL
        .into_iter()
        .filter_map(|catalog_category| {
            let count = providers
                .iter()
                .filter(|provider| provider.category == catalog_category)
                .count();
            (count > 0).then_some(CategorySummary {
                id: catalog_category.as_str(),
                label: catalog_category.label(),
                count,
            })
        })
        .collect();
    CatalogResponse {
        categories,
        totals: CatalogTotals {
            providers: provider_count,
            actions: action_count,
            implemented_actions,
        },
        provider_count,
        action_count,
        providers,
    }
}

pub fn provider_by_id(id: &str) -> Option<ProviderItem> {
    providers::all()
        .into_iter()
        .find(|seed| seed.id.eq_ignore_ascii_case(id.trim()))
        .map(expand)
}

pub fn llm_search(
    query: Option<&str>,
    category: Option<&str>,
    status: Option<&str>,
) -> Vec<LlmProviderSummary> {
    matching_seeds(query, category, status)
        .into_iter()
        .map(|seed| LlmProviderSummary {
            id: seed.id,
            name: seed.name,
            category: seed.category,
            status: seed.status,
            action_count: seed.actions.len(),
            llm_available: seed.llm_available,
        })
        .collect()
}

pub fn llm_actions(provider_id: &str) -> Option<Vec<LlmAction>> {
    providers::all()
        .into_iter()
        .find(|seed| seed.id.eq_ignore_ascii_case(provider_id.trim()))
        .map(|seed| {
            seed.actions
                .iter()
                .map(|action| LlmAction {
                    name: format!("{}.{}", seed.id, action.key),
                    summary: action.summary,
                    params: action.params,
                    risk: action.risk,
                    requires_approval: action.requires_approval,
                    implemented: action_is_implemented(seed, action),
                })
                .collect()
        })
}

#[cfg(test)]
mod tests;
