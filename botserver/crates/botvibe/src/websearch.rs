use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult};
use serde_json::{json, Value};
use std::sync::Arc;

fn ok(data: Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: Value::Null, error: Some(msg), latency_ms: 0 }
}

fn user_agent() -> &'static str {
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
}

fn strip_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut skip_mode = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if skip_mode {
            if c == '<' && chars.peek() == Some(&'/') {
                skip_mode = false;
                in_tag = true;
            }
            continue;
        }
        match c {
            '<' => {
                let mut tag = String::with_capacity(8);
                let mut lookahead = chars.clone();
                for _ in 0..7 {
                    match lookahead.next() {
                        Some(nc) => tag.push(nc),
                        None => break,
                    }
                }
                if tag.starts_with("script") || tag.starts_with("style") {
                    skip_mode = true;
                } else {
                    in_tag = true;
                }
            }
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn websearch_tools() -> Vec<(String, ToolSchema, ToolHandler)> {
    vec![
        ("web/search".into(), ToolSchema::new("web/search", "Live web search; returns ranked results with URLs and snippets").with_parameters(json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "max_results": {"type": "integer", "description": "Maximum results (default 5)"}
            },
            "required": ["query"]
        })), web_search()),
        ("web/crawl".into(), ToolSchema::new("web/crawl", "Fetch a page and return its visible text").with_parameters(json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "Full http(s) URL"},
                "max_chars": {"type": "integer", "description": "Maximum characters (default 4000)"}
            },
            "required": ["url"]
        })), web_crawl()),
    ]
}

fn web_search() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if query.is_empty() {
                return err("query is required".into());
            }
            let max = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5).min(20) as usize;
            // #1592: the agent tool and the chat pipeline share one provider
            // chain (TinyFish → DuckDuckGo → Wikipedia → Mojeek) so they cannot
            // silently drift apart.
            match botresearch::web_search::search_web_with_source(&query, max, "wt-wt").await {
                Ok(outcome) => {
                    let results: Vec<Value> = outcome
                        .results
                        .iter()
                        .map(|result| {
                            json!({
                                "title": result.title,
                                "url": result.url,
                                "snippet": result.snippet,
                            })
                        })
                        .collect();
                    ok(json!({
                        "query": query,
                        "provider": outcome.provider,
                        "results": results,
                    }))
                }
                Err(error) => err(format!("Search failed: {error}")),
            }
        })
    })
}

fn web_crawl() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
                return err("url must be a full http(s) URL".into());
            }
            let max_chars = args.get("max_chars").and_then(|v| v.as_u64()).unwrap_or(4000) as usize;
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default();
            match client.get(&url).header("User-Agent", user_agent()).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let content_type = resp
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();
                    let body = resp.text().await.unwrap_or_default();
                    let mut text = if content_type.contains("text/html") {
                        strip_html(&body)
                    } else {
                        body.trim().to_string()
                    };
                    if text.chars().count() > max_chars {
                        text = text.chars().take(max_chars).collect::<String>();
                        text.push_str("\n…[truncated]");
                    }
                    ok(json!({
                        "url": url,
                        "content_type": content_type,
                        "text": text,
                    }))
                }
                Ok(resp) => err(format!("Fetch returned status {}", resp.status())),
                Err(e) => err(format!("Fetch request failed: {e}")),
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags_and_scripts() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
        assert_eq!(strip_html("<script>var x = 1;</script>Keep"), "Keep");
        assert_eq!(strip_html("plain text"), "plain text");
    }
}
