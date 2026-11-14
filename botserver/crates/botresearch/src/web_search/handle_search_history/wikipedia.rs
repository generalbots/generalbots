use super::super::*;

/// Search URL for the Wikipedia API.
///
/// The scheme and host are compile-time constants and are asserted after the
/// query string is attached, so request input can only ever become a
/// percent-encoded query parameter — it cannot choose the destination
/// (CodeQL rust/request-forgery).
pub(super) fn wikipedia_search_url(query: &str, max_results: usize) -> Result<url::Url, String> {
    const API: &str = "https://en.wikipedia.org/w/api.php";
    let mut url = url::Url::parse(API).map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair("action", "query")
        .append_pair("list", "search")
        .append_pair("srsearch", query)
        .append_pair("format", "json")
        .append_pair("formatversion", "2")
        .append_pair("srlimit", &max_results.min(20).to_string());
    if url.scheme() != "https" || url.host_str() != Some("en.wikipedia.org") {
        return Err("unexpected Wikipedia endpoint".to_string());
    }
    Ok(url)
}

/// Summary URL for a Wikipedia article title, built from a constant host.
pub(super) fn wikipedia_summary_url(title: &str) -> Result<url::Url, String> {
    const API: &str = "https://en.wikipedia.org/api/rest_v1/page/summary/";
    let base = url::Url::parse(API).map_err(|e| e.to_string())?;
    let segment = urlencoding::encode(&title.replace(' ', "_")).to_string();
    let url = base.join(&segment).map_err(|e| e.to_string())?;
    if url.scheme() != "https" || url.host_str() != Some("en.wikipedia.org") {
        return Err("unexpected Wikipedia endpoint".to_string());
    }
    Ok(url)
}

/// Wikipedia search: reliable, citable and reachable from a datacenter
/// without a key. Used when the SERP scrapers are challenge-gated.
pub(super) async fn search_wikipedia(
    query: &str,
    max_results: usize,
) -> Result<Vec<WebSearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("GeneralBots/1.0 (research)")
        .build()
        .map_err(|e| e.to_string())?;
    let url = wikipedia_search_url(query, max_results)?;
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let body = response.text().await.map_err(|e| e.to_string())?;
    let parsed: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    if let Some(items) = parsed
        .get("query")
        .and_then(|q| q.get("search"))
        .and_then(|s| s.as_array())
    {
        for item in items.iter().take(max_results) {
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
            if title.is_empty() {
                continue;
            }
            let snippet = item
                .get("snippet")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .replace("<span class=\"searchmatch\">", "")
                .replace("</span>", "");
            results.push(WebSearchResult {
                title: title.to_string(),
                url: format!(
                    "https://en.wikipedia.org/wiki/{}",
                    urlencoding::encode(&title.replace(' ', "_"))
                ),
                snippet,
                source: "en.wikipedia.org".to_string(),
                favicon: Some(
                    "https://www.google.com/s2/favicons?domain=en.wikipedia.org".to_string(),
                ),
                published_date: None,
            });
        }
    }
    // The search snippet is one sentence; the article summary carries the
    // numbers and definitions a factual answer needs. Enrich the top hits.
    for result in results.iter_mut().take(2) {
        let url = match wikipedia_summary_url(&result.title) {
            Ok(url) => url,
            Err(_) => continue,
        };
        let response = client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await;
        let extract = match response {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(value) => value
                    .get("extract")
                    .and_then(|e| e.as_str())
                    .map(|s| s.chars().take(900).collect::<String>()),
                Err(_) => None,
            },
            Err(_) => None,
        };
        if let Some(extract) = extract {
            if !extract.trim().is_empty() {
                result.snippet = format!("{} {}", result.snippet.trim(), extract.trim());
            }
        }
    }
    info!(
        "Wikipedia search for '{}' returned {} results",
        query,
        results.len()
    );
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::{wikipedia_search_url, wikipedia_summary_url};

    #[test]
    fn search_url_keeps_the_constant_host() {
        let url = wikipedia_search_url("population of Brazil & Chile", 5).expect("url");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("en.wikipedia.org"));
        assert_eq!(url.path(), "/w/api.php");
        let query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert!(query.contains(&(
            "srsearch".to_string(),
            "population of Brazil & Chile".to_string()
        )));
    }

    #[test]
    fn a_traversal_attempt_stays_a_query_parameter() {
        let url = wikipedia_search_url("../../etc/passwd", 3).expect("url");
        assert_eq!(url.host_str(), Some("en.wikipedia.org"));
        assert!(url.as_str().contains("srsearch=..%2F..%2Fetc%2Fpasswd"));
    }

    #[test]
    fn summary_url_cannot_leave_the_host() {
        let url = wikipedia_summary_url("../../evil.example/x").expect("url");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("en.wikipedia.org"));
    }
}
