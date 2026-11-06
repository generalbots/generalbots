use super::super::*;

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
pub(super) fn tinyfish_api_key() -> Option<String> {
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
