use super::super::*;
use super::query::normalize_query;

/// Mojeek HTML search — a scraping-friendly engine used as a fallback when
/// DuckDuckGo serves its anti-bot challenge page (HTTP 202, zero results).
pub(super) async fn search_mojeek(
    query: &str,
    max_results: usize,
) -> Result<Vec<WebSearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://www.mojeek.com/search?q={}", normalize_query(query));
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    let mut pos = 0;
    while results.len() < max_results {
        let title_start = match html[pos..].find("<h2><a class=\"title\"") {
            Some(i) => pos + i,
            None => break,
        };
        let a_end = match html[title_start..].find("</a></h2>") {
            Some(i) => title_start + i,
            None => break,
        };
        let href_start = match html[title_start..].find("href=\"") {
            Some(i) => title_start + i + 6,
            None => break,
        };
        let href_end = match html[href_start..].find('"') {
            Some(i) => href_start + i,
            None => break,
        };
        let title_start_tag = match html[title_start..].find('>') {
            Some(i) => title_start + i + 1,
            None => break,
        };
        let url = html[href_start..href_end].to_string();
        let title = html[title_start_tag..a_end].trim().to_string();

        let mut snippet = String::new();
        let snip_start = match html[a_end..].find("<p class=\"s\">") {
            Some(i) => a_end + i + 13,
            None => a_end,
        };
        if snip_start != a_end {
            let snip_end = match html[snip_start..].find("</p>") {
                Some(i) => snip_start + i,
                None => snip_start,
            };
            snippet = html[snip_start..snip_end]
                .replace("<b>", "")
                .replace("</b>", "")
                .trim()
                .to_string();
        }

        results.push(WebSearchResult {
            title,
            url,
            snippet,
            source: "mojeek".to_string(),
            favicon: None,
            published_date: None,
        });
        pos = snip_start.max(a_end);
    }

    info!("Mojeek search for '{}' returned {} results", query, results.len());
    Ok(results)
}
