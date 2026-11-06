use super::super::*;
use super::duckduckgo::search_duckduckgo;
use super::mojeek::search_mojeek;
use super::query::clean_query;
use super::tinyfish::{search_tinyfish, tinyfish_api_key};
use super::wikipedia::search_wikipedia;

/// A search result plus the provider that actually answered (#1592).
///
/// The provider name is reported to callers so the chat UI and the research
/// routes can tell the user which backend supplied the sources.
#[derive(Debug, Clone)]
pub struct WebSearchOutcome {
    pub results: Vec<WebSearchResult>,
    pub provider: String,
}

/// Provider chain used by every web-search caller.
///
/// TinyFish first when configured; otherwise — or on timeout, HTTP error,
/// rate limit or empty results — the original DuckDuckGo → Wikipedia → Mojeek
/// chain runs unchanged.
pub async fn search_web_with_source(
    query: &str,
    max_results: usize,
    region: &str,
) -> Result<WebSearchOutcome, String> {
    let cleaned = clean_query(query);

    if tinyfish_api_key().is_some() {
        match search_tinyfish(&cleaned, max_results).await {
            Ok(results) if !results.is_empty() => {
                return Ok(WebSearchOutcome {
                    results,
                    provider: "tinyfish".to_string(),
                })
            }
            Ok(_) => info!("web search: TinyFish returned no results; using the fallback chain"),
            Err(reason) => info!("web search: TinyFish unavailable ({reason}); using the fallback chain"),
        }
    }

    let ddg = search_duckduckgo(&cleaned, max_results, region)
        .await
        .unwrap_or_default();
    if !ddg.is_empty() {
        return Ok(WebSearchOutcome {
            results: ddg,
            provider: "duckduckgo".to_string(),
        });
    }
    if let Ok(wiki) = search_wikipedia(&cleaned, max_results).await {
        if !wiki.is_empty() {
            return Ok(WebSearchOutcome {
                results: wiki,
                provider: "wikipedia".to_string(),
            });
        }
    }
    let mojeek = search_mojeek(&cleaned, max_results).await?;
    Ok(WebSearchOutcome {
        results: mojeek,
        provider: "mojeek".to_string(),
    })
}

/// Public wrapper used by the pipeline, the api-command catalog and the
/// research routes. Keeps the historical signature; callers that need the
/// answering provider use [`search_web_with_source`].
pub async fn search_web(
    query: &str,
    max_results: usize,
    region: &str,
) -> Result<Vec<WebSearchResult>, String> {
    search_web_with_source(query, max_results, region)
        .await
        .map(|outcome| outcome.results)
}
