use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use std::time::Duration;

use crate::ResearchState;

pub const ROUTE_RESEARCH_WEB_SEARCH: &str = "/api/ui/research/web/search";
pub const ROUTE_RESEARCH_WEB_SUMMARIZE: &str = "/api/ui/research/web/summarize";
pub const ROUTE_RESEARCH_WEB_DEEP: &str = "/api/ui/research/web/deep";
pub const ROUTE_RESEARCH_WEB_HISTORY: &str = "/api/ui/research/web/history";
pub const ROUTE_RESEARCH_WEB_INSTANT: &str = "/api/ui/research/web/instant";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchRequest {
    pub query: String,
    pub max_results: Option<usize>,
    pub region: Option<String>,
    pub safe_search: Option<bool>,
    pub time_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
    pub favicon: Option<String>,
    pub published_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub results: Vec<WebSearchResult>,
    pub query: String,
    pub total_results: usize,
    pub search_time_ms: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeRequest {
    pub query: String,
    pub results: Vec<WebSearchResult>,
    pub max_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeResponse {
    pub summary: String,
    pub citations: Vec<Citation>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearchRequest {
    pub query: String,
    pub depth: Option<usize>,
    pub max_sources: Option<usize>,
    pub follow_links: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearchResponse {
    pub answer: String,
    pub sources: Vec<WebSearchResult>,
    pub citations: Vec<Citation>,
    pub related_queries: Vec<String>,
    pub confidence: f32,
    pub research_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub id: String,
    pub query: String,
    pub results_count: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

pub fn configure_web_search_routes<S: ResearchState>() -> Router<Arc<S>> {
    Router::new()
        .route(ROUTE_RESEARCH_WEB_SEARCH, post(handle_web_search::<S>))
        .route(
            ROUTE_RESEARCH_WEB_SUMMARIZE,
            post(handle_summarize::<S>),
        )
        .route(ROUTE_RESEARCH_WEB_DEEP, post(handle_deep_research::<S>))
        .route(ROUTE_RESEARCH_WEB_HISTORY, get(handle_search_history::<S>))
        .route(ROUTE_RESEARCH_WEB_INSTANT, get(handle_instant_answer::<S>))
}

pub async fn handle_web_search<S: ResearchState>(
    State(_state): State<Arc<S>>,
    Json(payload): Json<WebSearchRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();

    if payload.query.trim().is_empty() {
        return Json(WebSearchResponse {
            results: Vec::new(),
            query: payload.query,
            total_results: 0,
            search_time_ms: 0,
            source: "none".to_string(),
        });
    }

    let max_results = payload.max_results.unwrap_or(10).min(25);
    let region = payload.region.as_deref().unwrap_or("wt-wt");

    let results = match search_duckduckgo(&payload.query, max_results, region).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("DuckDuckGo search failed: {}", e);
            Vec::new()
        }
    };

    let search_time_ms = start_time.elapsed().as_millis() as u64;

    Json(WebSearchResponse {
        total_results: results.len(),
        results,
        query: payload.query,
        search_time_ms,
        source: "duckduckgo".to_string(),
    })
}

pub async fn handle_summarize<S: ResearchState>(
    State(state): State<Arc<S>>,
    Json(payload): Json<SummarizeRequest>,
) -> impl IntoResponse {
    if payload.results.is_empty() {
        return Json(SummarizeResponse {
            summary: "No results to summarize.".to_string(),
            citations: Vec::new(),
            confidence: 0.0,
        });
    }

    let mut combined_text = String::new();
    let mut citations = Vec::new();

    for (idx, result) in payload.results.iter().enumerate() {
        let _ = writeln!(combined_text, "[{}] {}\n{}", idx + 1, result.title, result.snippet);
        citations.push(Citation {
            index: idx + 1,
            title: result.title.clone(),
            url: result.url.clone(),
            relevance: 1.0 - (idx as f32 * 0.1).min(0.5),
        });
    }

    let max_len = payload.max_length.unwrap_or(500);

    // Use the real LLM when available to synthesize the summary from the snippets.
    let summary = if let Some(llm) = state.llm_provider() {
        let prompt = format!(
            "Summarize the following search results about the query. Be concise and factual, \
             cite sources by their bracketed number [N] inline.\n\n{combined_text}"
        );
        match llm.generate_simple(&prompt).await {
            Ok(answer) => {
                let mut truncated = answer.trim().to_string();
                if truncated.chars().count() > max_len * 2 {
                    truncated = truncated.chars().take(max_len * 2).collect::<String>();
                    truncated.push_str("...");
                }
                truncated
            }
            Err(e) => {
                log::error!("Research summarize LLM call failed: {e}");
                let mut truncated = combined_text.chars().take(max_len).collect::<String>();
                if let Some(last_period) = truncated.rfind(". ") {
                    truncated.truncate(last_period + 1);
                }
                truncated
            }
        }
    } else {
        let mut truncated = combined_text.chars().take(max_len).collect::<String>();
        if let Some(last_period) = truncated.rfind(". ") {
            truncated.truncate(last_period + 1);
        }
        truncated
    };

    let confidence = (payload.results.len() as f32 / 10.0).min(1.0);

    Json(SummarizeResponse {
        summary,
        citations,
        confidence,
    })
}

pub async fn handle_deep_research<S: ResearchState>(
    State(state): State<Arc<S>>,
    Json(payload): Json<DeepResearchRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();

    if payload.query.trim().is_empty() {
        return Json(DeepResearchResponse {
            answer: "Please provide a research query.".to_string(),
            sources: Vec::new(),
            citations: Vec::new(),
            related_queries: Vec::new(),
            confidence: 0.0,
            research_time_ms: 0,
        });
    }

    let depth = payload.depth.unwrap_or(2).min(3);
    let max_sources = payload.max_sources.unwrap_or(10).min(20);

    let mut all_results: Vec<WebSearchResult> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    let initial_results = search_duckduckgo(&payload.query, max_sources, "wt-wt")
        .await
        .unwrap_or_default();

    for result in initial_results {
        if !seen_urls.contains(&result.url) {
            seen_urls.insert(result.url.clone());
            all_results.push(result);
        }
    }

    if depth > 1 {
        let related_queries = generate_related_queries(&payload.query);

        for rq in related_queries.iter().take(depth - 1) {
            if let Ok(more_results) = search_duckduckgo(rq, 5, "wt-wt").await {
                for result in more_results {
                    if !seen_urls.contains(&result.url) && all_results.len() < max_sources {
                        seen_urls.insert(result.url.clone());
                        all_results.push(result);
                    }
                }
            }
        }
    }

    let mut citations = Vec::new();
    let mut answer_parts: Vec<String> = Vec::new();

    for (idx, result) in all_results.iter().enumerate() {
        if idx < 5 {
            answer_parts.push(format!("\u{2022} {}", result.snippet));
        }
        citations.push(Citation {
            index: idx + 1,
            title: result.title.clone(),
            url: result.url.clone(),
            relevance: 1.0 - (idx as f32 * 0.05).min(0.5),
        });
    }

    // Use the real LLM when available to synthesize a deep research answer.
    let answer = if let Some(llm) = state.llm_provider() {
        if answer_parts.is_empty() {
            format!("No results found for: {}", payload.query)
        } else {
            let sources_block = all_results
                .iter()
                .enumerate()
                .map(|(i, r)| format!("[{}] {} — {}", i + 1, r.title, r.snippet))
                .collect::<Vec<_>>()
                .join("\n");
            let prompt = format!(
                "Perform deep research on the question. Synthesize a structured, factual answer \
                 using ONLY the sources below, citing them inline as [N].\n\n\
                 Question: {}\n\nSources:\n{}",
                payload.query, sources_block
            );
            match llm.generate_simple(&prompt).await {
                Ok(a) => a,
                Err(e) => {
                    log::error!("Research deep LLM call failed: {e}");
                    if answer_parts.is_empty() {
                        format!("No results found for: {}", payload.query)
                    } else {
                        format!(
                            "Based on {} sources about \"{}\":\n\n{}",
                            all_results.len(),
                            payload.query,
                            answer_parts.join("\n\n")
                        )
                    }
                }
            }
        }
    } else if answer_parts.is_empty() {
        format!("No results found for: {}", payload.query)
    } else {
        format!(
            "Based on {} sources about \"{}\":\n\n{}",
            all_results.len(),
            payload.query,
            answer_parts.join("\n\n")
        )
    };

    let related = generate_related_queries(&payload.query);

    let research_time_ms = start_time.elapsed().as_millis() as u64;
    let confidence = (citations.len() as f32 / 10.0).min(1.0);

    Json(DeepResearchResponse {
        answer,
        sources: all_results,
        citations,
        related_queries: related,
        confidence,
        research_time_ms,
    })
}

pub async fn handle_search_history<S: ResearchState>(
    State(state): State<Arc<S>>,
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
             WHERE query <> '' GROUP BY query ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
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
fn normalize_query(query: &str) -> String {
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

async fn search_duckduckgo(
    query: &str,
    max_results: usize,
    region: &str,
) -> Result<Vec<WebSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let encoded_query = normalize_query(query);
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}&kl={}",
        encoded_query, region
    );

    debug!("Searching DuckDuckGo: {}", query);

    let response = client.get(&url).send().await?;
    let html = response.text().await?;

    let results = parse_duckduckgo_html(&html, max_results);

    info!(
        "DuckDuckGo search for '{}' returned {} results",
        query,
        results.len()
    );

    Ok(results)
}

/// Public wrapper over the DuckDuckGo searcher so the LLM api-command catalog
/// can answer from the web over any channel (e.g. WhatsApp). Falls back to
/// Mojeek when DuckDuckGo returns no results (anti-bot challenge pages).
pub async fn search_web(
    query: &str,
    max_results: usize,
    region: &str,
) -> Result<Vec<WebSearchResult>, String> {
    let ddg = search_duckduckgo(query, max_results, region)
        .await
        .unwrap_or_default();
    if !ddg.is_empty() {
        return Ok(ddg);
    }
    search_mojeek(query, max_results).await
}

/// Mojeek HTML search — a scraping-friendly engine used as a fallback when
/// DuckDuckGo serves its anti-bot challenge page (HTTP 202, zero results).
async fn search_mojeek(
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

fn parse_duckduckgo_html(html: &str, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();

    let mut current_title = String::new();
    let mut current_url = String::new();
    let mut current_snippet = String::new();

    for line in html.lines() {
        let line = line.trim();

        if line.contains("class=\"result__a\"") {
            if let Some(href_start) = line.find("href=\"") {
                let start = href_start + 6;
                if let Some(href_end) = line[start..].find('"') {
                    let raw_url = &line[start..start + href_end];
                    current_url = decode_ddg_url(raw_url);
                }
            }

            if let Some(title_start) = line.find('>') {
                let after_tag = &line[title_start + 1..];
                if let Some(title_end) = after_tag.find('<') {
                    current_title = html_decode(&after_tag[..title_end]);
                }
            }
        }

        if line.contains("class=\"result__snippet\"") {
            if let Some(snippet_start) = line.find('>') {
                let after_tag = &line[snippet_start + 1..];
                let snippet_text = strip_html_inline(after_tag);
                current_snippet = html_decode(&snippet_text);
            }

            if !current_title.is_empty() && !current_url.is_empty() {
                let domain = extract_domain(&current_url);
                results.push(WebSearchResult {
                    title: current_title.clone(),
                    url: current_url.clone(),
                    snippet: current_snippet.clone(),
                    source: domain.clone(),
                    favicon: Some(format!(
                        "https://www.google.com/s2/favicons?domain={}",
                        domain
                    )),
                    published_date: None,
                });

                current_title.clear();
                current_url.clear();
                current_snippet.clear();

                if results.len() >= max_results {
                    break;
                }
            }
        }
    }

    if results.is_empty() {
        results = parse_duckduckgo_fallback(html, max_results);
    }

    results
}

fn parse_duckduckgo_fallback(html: &str, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();

    let parts: Vec<&str> = html.split("class=\"result ").collect();

    for part in parts.iter().skip(1).take(max_results) {
        let mut title = String::new();
        let mut url = String::new();
        let mut snippet = String::new();

        if let Some(a_start) = part.find("class=\"result__a\"") {
            let section = &part[a_start..];

            if let Some(href_pos) = section.find("href=\"") {
                let start = href_pos + 6;
                if let Some(end) = section[start..].find('"') {
                    url = decode_ddg_url(&section[start..start + end]);
                }
            }

            if let Some(text_start) = section.find('>') {
                let after = &section[text_start + 1..];
                if let Some(text_end) = after.find('<') {
                    title = html_decode(&after[..text_end]);
                }
            }
        }

        if let Some(snippet_start) = part.find("class=\"result__snippet\"") {
            let section = &part[snippet_start..];
            if let Some(text_start) = section.find('>') {
                let after = &section[text_start + 1..];
                let text = strip_html_inline(after);
                snippet = html_decode(&text);
                if let Some(end) = snippet.find("</") {
                    snippet.truncate(end);
                }
            }
        }

        if !title.is_empty() && !url.is_empty() {
            let domain = extract_domain(&url);
            results.push(WebSearchResult {
                title,
                url: url.clone(),
                snippet,
                source: domain.clone(),
                favicon: Some(format!(
                    "https://www.google.com/s2/favicons?domain={}",
                    domain
                )),
                published_date: None,
            });
        }
    }

    results
}

fn decode_ddg_url(raw_url: &str) -> String {
    if raw_url.starts_with("//duckduckgo.com/l/?uddg=") {
        let encoded_part = raw_url.trim_start_matches("//duckduckgo.com/l/?uddg=");
        if let Some(amp_pos) = encoded_part.find('&') {
            let url_part = &encoded_part[..amp_pos];
            return urlencoding::decode(url_part)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| raw_url.to_string());
        }
        return urlencoding::decode(encoded_part)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| raw_url.to_string());
    }

    if raw_url.starts_with("http") {
        return raw_url.to_string();
    }

    format!("https:{}", raw_url)
}

fn extract_domain(url: &str) -> String {
    let without_protocol = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    if let Some(slash_pos) = without_protocol.find('/') {
        without_protocol[..slash_pos].to_string()
    } else {
        without_protocol.to_string()
    }
}

fn strip_html_inline(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result.trim().to_string()
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn generate_related_queries(query: &str) -> Vec<String> {
    let base_words: Vec<&str> = query.split_whitespace().collect();

    let mut related = Vec::new();

    related.push(format!("what is {}", query));
    related.push(format!("{} explained", query));
    related.push(format!("{} examples", query));
    related.push(format!("how does {} work", query));
    related.push(format!("{} vs alternatives", query));

    if base_words.len() > 2 {
        let shortened: String = base_words[..2].join(" ");
        related.push(shortened);
    }

    related.into_iter().take(5).collect()
}

async fn get_instant_answer(query: &str) -> Option<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;

    let encoded = urlencoding::encode(query);
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        encoded
    );

    let response = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = response.json().await.ok()?;

    if let Some(abstract_text) = json.get("AbstractText").and_then(|v| v.as_str()) {
        if !abstract_text.is_empty() {
            let answer_type = json
                .get("Type")
                .and_then(|v| v.as_str())
                .unwrap_or("A")
                .to_string();
            return Some((abstract_text.to_string(), answer_type));
        }
    }

    if let Some(answer) = json.get("Answer").and_then(|v| v.as_str()) {
        if !answer.is_empty() {
            return Some((answer.to_string(), "answer".to_string()));
        }
    }

    if let Some(definition) = json.get("Definition").and_then(|v| v.as_str()) {
        if !definition.is_empty() {
            return Some((definition.to_string(), "definition".to_string()));
        }
    }

    None
}
