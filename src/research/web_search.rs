use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use std::time::Duration;

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

pub fn configure_web_search_routes() -> Router<Arc<AppState>> {
    use crate::core::urls::ApiUrls;

    Router::new()
        .route(ApiUrls::RESEARCH_WEB_SEARCH, post(handle_web_search))
        .route(ApiUrls::RESEARCH_WEB_SUMMARIZE, post(handle_summarize))
        .route(ApiUrls::RESEARCH_WEB_DEEP, post(handle_deep_research))
        .route(ApiUrls::RESEARCH_WEB_HISTORY, get(handle_search_history))
        .route(ApiUrls::RESEARCH_WEB_INSTANT, get(handle_instant_answer))
}

pub async fn handle_web_search(
    State(_state): State<Arc<AppState>>,
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
            error!("DuckDuckGo search failed: {}", e);
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

pub async fn handle_summarize(
    State(_state): State<Arc<AppState>>,
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
        let _ = writeln!(combined_text, "[{}] {}", idx + 1, result.snippet);
        citations.push(Citation {
            index: idx + 1,
            title: result.title.clone(),
            url: result.url.clone(),
            relevance: 1.0 - (idx as f32 * 0.1).min(0.5),
        });
    }

    let max_len = payload.max_length.unwrap_or(500);
    let summary = if combined_text.len() > max_len {
        let mut truncated = combined_text.chars().take(max_len).collect::<String>();
        if let Some(last_period) = truncated.rfind(". ") {
            truncated.truncate(last_period + 1);
        }
        truncated
    } else {
        combined_text
    };

    let confidence = (payload.results.len() as f32 / 10.0).min(1.0);

    Json(SummarizeResponse {
        summary,
        citations,
        confidence,
    })
}

pub async fn handle_deep_research(
    State(_state): State<Arc<AppState>>,
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
            answer_parts.push(format!("• {}", result.snippet));
        }
        citations.push(Citation {
            index: idx + 1,
            title: result.title.clone(),
            url: result.url.clone(),
            relevance: 1.0 - (idx as f32 * 0.05).min(0.5),
        });
    }

    let answer = if answer_parts.is_empty() {
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

pub async fn handle_search_history(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchHistoryQuery>,
) -> impl IntoResponse {
    let _page = params.page.unwrap_or(1).max(1);
    let _per_page = params.per_page.unwrap_or(20).min(100);

    let history: Vec<SearchHistoryEntry> = vec![
        SearchHistoryEntry {
            id: "1".to_string(),
            query: "Example search 1".to_string(),
            results_count: 10,
            timestamp: Utc::now(),
        },
        SearchHistoryEntry {
            id: "2".to_string(),
            query: "Example search 2".to_string(),
            results_count: 8,
            timestamp: Utc::now(),
        },
    ];

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

pub async fn handle_instant_answer(
    State(_state): State<Arc<AppState>>,
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

async fn search_duckduckgo(
    query: &str,
    max_results: usize,
    region: &str,
) -> Result<Vec<WebSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let encoded_query = urlencoding::encode(query);
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
