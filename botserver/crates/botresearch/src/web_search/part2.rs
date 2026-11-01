use super::*;

pub async fn handle_search_history<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<SearchHistoryQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct HistoryRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        pub query: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        pub n: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        pub created_at: chrono::DateTime<Utc>,
    }

    let conn = state.db_pool().clone();
    let branch_id = botsecurity_core::tenant::branch_from_claims(&headers).unwrap_or_else(uuid::Uuid::nil);
    let history: Vec<SearchHistoryEntry> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT query, COUNT(*) AS n, MAX(created_at) AS created_at FROM research_searches \
             WHERE query <> '' AND branch_id = $1 GROUP BY query ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::BigInt, _>(per_page as i64)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .load::<HistoryRow>(&mut db_conn)
        .map(|rows| {
            rows.into_iter()
                .enumerate()
                .map(|(i, r)| SearchHistoryEntry {
                    id: format!("{i}"),
                    query: r.query,
                    results_count: r.n as usize,
                    timestamp: r.created_at,
                })
                .collect()
        })
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"search-history\">");

    if history.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No search history yet</p>");
        html.push_str("</div>");
    } else {
        for entry in &history {
            html.push_str("<div class=\"history-item\" data-id=\"");
            html.push_str(&html_escape(&entry.id));
            html.push_str("\">");
            html.push_str("<span class=\"history-query\">");
            html.push_str(&html_escape(&entry.query));
            html.push_str("</span>");
            html.push_str("<span class=\"history-count\">");
            html.push_str(&entry.results_count.to_string());
            html.push_str(" results</span>");
            html.push_str("</div>");
        }
    }

    html.push_str("</div>");
    Html(html)
}
pub async fn handle_instant_answer<S: ResearchState>(
    State(_state): State<Arc<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();

    if query.is_empty() {
        return Json(serde_json::json!({
            "answer": null,
            "type": "none"
        }));
    }

    if let Some(answer) = get_instant_answer(&query).await {
        Json(serde_json::json!({
            "answer": answer.0,
            "type": answer.1,
            "source": "duckduckgo"
        }))
    } else {
        Json(serde_json::json!({
            "answer": null,
            "type": "none"
        }))
    }
}
/// Normalizes a search query for scraping-friendly engines: strips accents
/// (DuckDuckGo/Mojeek serve anti-bot challenge pages for %20-encoded or
/// accented queries) and joins words with '+' like a form submission.
pub(super) fn normalize_query(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    for c in query.chars() {
        let mapped = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'Ç' => 'C',
            'Ñ' => 'N',
            _ => c,
        };
        if mapped.is_whitespace() {
            if !out.ends_with('+') {
                out.push('+');
            }
        } else {
            out.push(mapped);
        }
    }
    out.trim_end_matches('+').to_string()
}
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

    let encoded_query = normalize_query(query);
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}&kl={}",
        encoded_query, region
    );

    debug!("Searching DuckDuckGo: {}", query);

    let response = client
        .get(&url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Dest", "document")
        .send()
        .await?;
    let status = response.status();
    let html = response.text().await?;
    info!(
        "DuckDuckGo HTTP {} for '{}': {} bytes, result__a markers: {}",
        status,
        query,
        html.len(),
        html.matches("result__a").count()
    );

    let results = parse_duckduckgo_html(&html, max_results);

    info!(
        "DuckDuckGo search for '{}' returned {} results",
        query,
        results.len()
    );

    Ok(results)
}
/// Strips conversational scaffolding ("What is ... ? Cite your sources") so
/// the engine sees the topic, not the question wrapper.
pub fn clean_query(query: &str) -> String {
    let mut text = query.trim().to_string();
    let mut changed = true;
    while changed {
        changed = false;
        let lower = text.to_lowercase();
        for prefix in [
            "what is the ", "what are the ", "what is ", "what are ", "who is ", "who are ",
            "how many ", "how much ", "when did ", "when was ", "where is ", "where are ",
            "tell me ", "can you tell me ", "could you tell me ", "do you know ", "please ",
            "search the web for ", "search web for ", "search for ", "search the web ",
            "search the internet for ", "look up ", "google ",
        ] {
            if lower.starts_with(prefix) {
                text = text[prefix.len()..].to_string();
                changed = true;
                break;
            }
        }
        if changed {
            continue;
        }
        let lower = text.to_lowercase();
        for phrase in [
            " cite your sources", " cite sources", " give sources", " with sources",
            " according to the latest official estimate", " according to official sources",
        ] {
            if let Some(at) = lower.find(phrase) {
                text = format!("{}{}", &text[..at], &text[at + phrase.len()..]);
                changed = true;
                break;
            }
        }
    }
    let text = text
        .trim()
        .trim_end_matches(|c: char| matches!(c, '?' | '.' | '!' | ','))
        .trim()
        .to_string();
    if text.is_empty() {
        query.trim().to_string()
    } else {
        text
    }
}

/// Search URL for the Wikipedia API.
///
/// The scheme and host are compile-time constants and are asserted after the
/// query string is attached, so request input can only ever become a
/// percent-encoded query parameter — it cannot choose the destination
/// (CodeQL rust/request-forgery).
pub(super) fn wikipedia_search_url(
    query: &str,
    max_results: usize,
) -> Result<url::Url, String> {
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
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| e.to_string())?;
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

/// A search result plus the provider that actually answered (#1592).
///
/// The provider name is reported to callers so the chat UI and the research
/// routes can tell the user which backend supplied the sources.
#[derive(Debug, Clone)]
pub struct WebSearchOutcome {
    pub results: Vec<WebSearchResult>,
    pub provider: String,
}

/// TinySearch is free but metered: 30 requests/minute and 500 requests/hour
/// per account. The limiter is process-wide so parallel conversations cannot
/// exceed the quota together.
const TINYFISH_PER_MINUTE: usize = 30;
const TINYFISH_PER_HOUR: usize = 500;
const TINYFISH_ENDPOINT: &str = "https://api.search.tinyfish.ai";
const TINYFISH_TIMEOUT: Duration = Duration::from_secs(10);

struct TinyFishLimiter {
    minute: std::collections::VecDeque<std::time::Instant>,
    hour: std::collections::VecDeque<std::time::Instant>,
}

impl TinyFishLimiter {
    fn new() -> Self {
        Self {
            minute: std::collections::VecDeque::new(),
            hour: std::collections::VecDeque::new(),
        }
    }

    /// Drops timestamps that fell out of each window, then reserves a slot.
    fn try_acquire(&mut self) -> bool {
        let now = std::time::Instant::now();
        while self
            .minute
            .front()
            .is_some_and(|at| now.duration_since(*at).as_secs() >= 60)
        {
            self.minute.pop_front();
        }
        while self
            .hour
            .front()
            .is_some_and(|at| now.duration_since(*at).as_secs() >= 3600)
        {
            self.hour.pop_front();
        }
        if self.minute.len() >= TINYFISH_PER_MINUTE || self.hour.len() >= TINYFISH_PER_HOUR {
            return false;
        }
        self.minute.push_back(now);
        self.hour.push_back(now);
        true
    }
}

fn tinyfish_limiter() -> &'static std::sync::Mutex<TinyFishLimiter> {
    static LIMITER: std::sync::OnceLock<std::sync::Mutex<TinyFishLimiter>> =
        std::sync::OnceLock::new();
    LIMITER.get_or_init(|| std::sync::Mutex::new(TinyFishLimiter::new()))
}

/// Reads the key at call time. A missing key preserves the previous behaviour
/// exactly: the scraper chain runs as if TinyFish did not exist.
fn tinyfish_api_key() -> Option<String> {
    std::env::var("TINYFISH_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Parses the structured TinySearch response. Exposed for unit tests; a
/// malformed payload yields no results so the caller falls back.
pub(super) fn parse_tinyfish_response(body: &str, max_results: usize) -> Vec<WebSearchResult> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let Some(items) = value.get("results").and_then(|r| r.as_array()) else {
        return Vec::new();
    };
    let mut results = Vec::new();
    for item in items.iter().take(max_results) {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").trim();
        let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").trim();
        if title.is_empty() || url.is_empty() {
            continue;
        }
        let snippet = item
            .get("snippet")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let domain = item
            .get("site_name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| extract_domain(url));
        results.push(WebSearchResult {
            title: title.to_string(),
            url: url.to_string(),
            snippet,
            source: domain.clone(),
            favicon: Some(format!(
                "https://www.google.com/s2/favicons?domain={domain}"
            )),
            published_date: item
                .get("date")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        });
    }
    results
}

/// TinyFish TinySearch — the default provider when a key is configured.
pub(super) async fn search_tinyfish(
    query: &str,
    max_results: usize,
) -> Result<Vec<WebSearchResult>, String> {
    let key = tinyfish_api_key().ok_or_else(|| "TINYFISH_API_KEY is not set".to_string())?;
    {
        let mut limiter = tinyfish_limiter()
            .lock()
            .map_err(|_| "TinyFish limiter poisoned".to_string())?;
        if !limiter.try_acquire() {
            return Err("TinyFish rate limit reached (30/min, 500/h)".to_string());
        }
    }
    let client = reqwest::Client::builder()
        .timeout(TINYFISH_TIMEOUT)
        .build()
        .map_err(|error| format!("TinyFish client: {error}"))?;
    let url = format!(
        "{TINYFISH_ENDPOINT}?query={}",
        urlencoding::encode(query)
    );
    let response = client
        .get(&url)
        .header("X-API-Key", key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("TinyFish request failed: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("TinyFish returned HTTP {status}"));
    }
    let body = response
        .text()
        .await
        .map_err(|error| format!("TinyFish body: {error}"))?;
    let results = parse_tinyfish_response(&body, max_results);
    if results.is_empty() {
        return Err("TinyFish returned no usable results".to_string());
    }
    info!("TinyFish search for '{}' returned {} results", query, results.len());
    Ok(results)
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

#[cfg(test)]
mod tinyfish_tests {
    use super::{
        parse_tinyfish_response, TinyFishLimiter, TINYFISH_PER_HOUR, TINYFISH_PER_MINUTE,
    };

    #[test]
    fn parses_structured_results_and_keeps_the_site_name() {
        let body = r#"{
            "query": "web automation",
            "total_results": 2,
            "results": [
                {"position": 1, "site_name": "example.com", "title": "Example",
                 "snippet": "A snippet", "url": "https://example.com/page", "date": "2026-10-01"},
                {"position": 2, "site_name": "", "title": "Second",
                 "snippet": "", "url": "https://other.org/x"}
            ]
        }"#;
        let results = parse_tinyfish_response(body, 5);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Example");
        assert_eq!(results[0].url, "https://example.com/page");
        assert_eq!(results[0].source, "example.com");
        assert_eq!(results[0].published_date.as_deref(), Some("2026-10-01"));
        // Empty site_name falls back to the URL host.
        assert_eq!(results[1].source, "other.org");
    }

    #[test]
    fn malformed_payload_yields_no_results_so_the_chain_falls_back() {
        assert!(parse_tinyfish_response("not json", 5).is_empty());
        assert!(parse_tinyfish_response("{}", 5).is_empty());
        assert!(parse_tinyfish_response(r#"{"results":"nope"}"#, 5).is_empty());
    }

    #[test]
    fn parser_respects_max_results_and_skips_incomplete_rows() {
        let body = r#"{"results":[
            {"title":"A","url":"https://a.org"},
            {"title":"","url":"https://b.org"},
            {"title":"C","url":""},
            {"title":"D","url":"https://d.org"}
        ]}"#;
        let results = parse_tinyfish_response(body, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "A");
    }

    #[test]
    fn limiter_allows_the_minute_budget_then_refuses() {
        let mut limiter = TinyFishLimiter::new();
        for _ in 0..TINYFISH_PER_MINUTE {
            assert!(limiter.try_acquire());
        }
        assert!(!limiter.try_acquire(), "the 31st request in a minute must be refused");
        assert!(TINYFISH_PER_MINUTE < TINYFISH_PER_HOUR);
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::{wikipedia_search_url, wikipedia_summary_url};

    #[test]
    fn search_url_keeps_the_constant_host() {
        let url = wikipedia_search_url("population of Brazil & Chile", 5).expect("url");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("en.wikipedia.org"));
        assert_eq!(url.path(), "/w/api.php");
        let query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert!(query.contains(&("srsearch".to_string(), "population of Brazil & Chile".to_string())));
    }

    #[test]
    fn a_traversal_attempt_stays_a_query_parameter() {
        let url = wikipedia_search_url("../../etc/passwd", 3).expect("url");
        assert_eq!(url.host_str(), Some("en.wikipedia.org"));
        assert!(url.as_str().contains("srsearch=..%2F..%2Fetc%2Fpasswd"));
    }

    #[test]
    fn clean_query_strips_the_search_instruction() {
        assert_eq!(
            super::clean_query("search web for computers virus recent"),
            "computers virus recent"
        );
        assert_eq!(
            super::clean_query("Search the web for the latest virus news."),
            "the latest virus news"
        );
        assert_eq!(super::clean_query("google ransomware 2026"), "ransomware 2026");
    }

    #[test]
    fn summary_url_cannot_leave_the_host() {
        let url = wikipedia_summary_url("../../evil.example/x").expect("url");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("en.wikipedia.org"));
    }
}
