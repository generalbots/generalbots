use super::super::*;

/// DuckDuckGo's key-free HTML endpoints. A plain GET of the HTML page now
/// answers automated clients with HTTP 202 and an image challenge; the same
/// search sent as a form POST returns results, and the lite page is the
/// second try when the first one is challenged or comes back empty.
const HTML_ENDPOINT: &str = "https://html.duckduckgo.com/html/";
const LITE_ENDPOINT: &str = "https://lite.duckduckgo.com/lite/";

pub(super) async fn search_duckduckgo(
    query: &str,
    max_results: usize,
    region: &str,
) -> Result<Vec<WebSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/126.0 Safari/537.36",
        )
        .build()?;
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("q", query)
        .append_pair("kl", region)
        .finish();

    match post_form(&client, HTML_ENDPOINT, &body, query).await {
        Ok(Some(page)) => {
            let results = parse_duckduckgo_html(&page, max_results);
            if !results.is_empty() {
                return Ok(results);
            }
        }
        Ok(None) => {}
        Err(reason) => info!("DuckDuckGo html unavailable ({reason}); trying lite"),
    }

    let lite = post_form(&client, LITE_ENDPOINT, &body, query).await?;
    let results = lite
        .map(|page| parse_duckduckgo_lite(&page, max_results))
        .unwrap_or_else(|| {
            log::debug!("duckduckgo lite fetch failed, using empty result set");
            Vec::new()
        });
    info!(
        "DuckDuckGo lite for query_chars={} returned {} results",
        query.chars().count(),
        results.len()
    );
    Ok(results)
}

/// Posts the search form; `None` when DuckDuckGo answered with its challenge.
async fn post_form(
    client: &reqwest::Client,
    endpoint: &str,
    body: &str,
    query: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let origin = endpoint.trim_end_matches('/').rsplit_once('/').map_or(endpoint, |(base, _)| base);
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Origin", origin)
        .header("Referer", endpoint)
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Dest", "document")
        .body(body.to_string())
        .send()
        .await?;
    let status = response.status().as_u16();
    let page = response.text().await?;
    // The query can carry personal data: logs get its length only.
    let challenged = is_ddg_challenge(status, &page);
    info!(
        "DuckDuckGo {} HTTP {} for query_chars={}: {} bytes, challenged={}",
        if endpoint == LITE_ENDPOINT { "lite" } else { "html" },
        status,
        query.chars().count(),
        page.len(),
        challenged
    );
    Ok((!challenged).then_some(page))
}
