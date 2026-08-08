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
            let url = format!("https://html.duckduckgo.com/html/?q={}", percent_encode(&query));
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .unwrap_or_default();
            match client
                .get(&url)
                .header("User-Agent", user_agent())
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let html = resp.text().await.unwrap_or_default();
                    let results = parse_ddg_results(&html, max);
                    if results.is_empty() {
                        ok(json!({ "query": query, "results": [], "note": "No results parsed" }))
                    } else {
                        ok(json!({ "query": query, "results": results }))
                    }
                }
                Ok(resp) => err(format!("Search endpoint returned status {}", resp.status())),
                Err(e) => err(format!("Search request failed: {e}")),
            }
        })
    })
}

fn parse_ddg_results(html: &str, max: usize) -> Vec<Value> {
    let mut results = Vec::new();
    let marker = r#"class="result__a" href=""#;
    let mut search_from = 0;
    while results.len() < max {
        let Some(rel_start) = html[search_from..].find(marker) else { break };
        let abs_start = search_from + rel_start + marker.len();
        let Some(href_end) = html[abs_start..].find('"') else { break };
        let href = &html[abs_start..abs_start + href_end];
        let Some(title_open) = html[abs_start + href_end..].find('>') else { break };
        let title_start = abs_start + href_end + title_open + 1;
        let Some(title_end) = html[title_start..].find("</a>") else { break };
        let title = strip_html(&html[title_start..title_start + title_end]).trim().to_string();
        let snippet_start = title_start + title_end;
        let Some(snippet_marker) = html[snippet_start..].find(r#"class="result__snippet""#) else { break };
        let snip_abs = snippet_start + snippet_marker;
        let Some(snip_open) = html[snip_abs..].find('>') else { break };
        let snip_text_start = snip_abs + snip_open + 1;
        let Some(snip_end) = html[snip_text_start..].find("</a>") else { break };
        let snippet = strip_html(&html[snip_text_start..snip_text_start + snip_end]).trim().to_string();
        results.push(json!({
            "title": title,
            "url": decode_ddg_url(href),
            "snippet": snippet,
        }));
        search_from = snip_text_start + snip_end;
    }
    results
}

fn decode_ddg_url(href: &str) -> String {
    if let Some(idx) = href.find("uddg=") {
        let after = &href[idx + 5..];
        let end = after.find('&').unwrap_or(after.len());
        return urlencode::decode(after[..end]);
    }
    href.to_string()
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

mod urlencode {
    pub fn decode(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let hi = hex_val(bytes[i + 1]);
                let lo = hex_val(bytes[i + 2]);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi << 4) | lo);
                    i += 3;
                    continue;
                }
            }
            if bytes[i] == b'+' {
                out.push(b' ');
            } else {
                out.push(bytes[i]);
            }
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    fn hex_val(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }
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
    fn percent_encode_keeps_reserved_ascii() {
        assert_eq!(percent_encode("hello world"), "hello+world");
        assert_eq!(percent_encode("a-b.c~d"), "a-b.c~d");
        assert_eq!(percent_encode("café"), "caf%C3%A9");
        assert_eq!(percent_encode(""), "");
    }

    #[test]
    fn decode_ddg_url_handles_plain_and_encoded() {
        assert_eq!(decode_ddg_url("https://example.com/a?b=1"), "https://example.com/a?b=1");
        assert_eq!(decode_ddg_url("//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2F&rut=x"), "https://example.com/");
    }

    #[test]
    fn strip_html_removes_tags_and_scripts() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
        assert_eq!(strip_html("<script>var x = 1;</script>Keep"), "Keep");
        assert_eq!(strip_html("plain text"), "plain text");
    }

    #[test]
    fn parse_ddg_results_extracts_entries() {
        let html = r#"<a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2F">Example <b>Site</b></a>
            <a class="result__snippet" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2F">A useful snippet</a>
            <a class="result__a" href="https://other.org">Other</a>
            <a class="result__snippet" href="https://other.org">Snippet two</a>"#;
        let results = parse_ddg_results(html, 5);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["title"], "Example Site");
        assert_eq!(results[0]["url"], "https://example.com/");
        assert_eq!(results[0]["snippet"], "A useful snippet");
        assert_eq!(results[1]["title"], "Other");
    }

    #[test]
    fn parse_ddg_results_respects_max() {
        let html = format!(
            "{}",
            r#"<a class="result__a" href="https://a.org">A</a><a class="result__snippet" href="https://a.org">s1</a>
               <a class="result__a" href="https://b.org">B</a><a class="result__snippet" href="https://b.org">s2</a>"#
        );
        assert_eq!(parse_ddg_results(&html, 1).len(), 1);
        assert_eq!(parse_ddg_results("no markers here", 5).len(), 0);
    }

    #[test]
    fn urlencode_module_decodes_percent_and_plus() {
        assert_eq!(urlencode::decode("a%20b+c"), "a b c");
        assert_eq!(urlencode::decode("100%25"), "100%");
    }
}
