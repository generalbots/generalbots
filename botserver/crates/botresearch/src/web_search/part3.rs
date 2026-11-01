use super::*;

/// Pulls one attribute value out of an opening tag (`href="..."`).
fn attr_value(tag: &str, name: &str) -> Option<String> {
    let marker = format!("{name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_string())
}

/// DuckDuckGo's HTML puts the title on the line *after* the anchor tag, so a
/// line-oriented parser never saw a title and returned zero results. This
/// scans anchors by position: href from the opening tag, title from the text
/// inside `result__a`, snippet from the following `result__snippet` anchor.
pub(super) fn parse_duckduckgo_html(html: &str, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();
    let mut rest = html;

    while results.len() < max_results {
        let Some(marker) = rest.find("result__a") else {
            break;
        };
        let anchor_start = rest[..marker].rfind("<a ").unwrap_or(marker);
        let after = &rest[anchor_start..];
        let Some(open_end) = after.find('>') else {
            break;
        };
        let open_tag = &after[..open_end];
        let Some(raw_url) = attr_value(open_tag, "href") else {
            rest = &after[open_end + 1..];
            continue;
        };
        let url = decode_ddg_url(&raw_url);
        let Some(close) = after[open_end..].find("</a>") else {
            break;
        };
        let title = html_decode(&strip_html_inline(
            &after[open_end + 1..open_end + close],
        ));
        let remainder = &after[open_end + close + 4..];

        let snippet = remainder
            .find("result__snippet")
            .and_then(|at| {
                let anchor = &remainder[at..];
                let gt = anchor.find('>')?;
                let text = &anchor[gt + 1..];
                let end = text.find("</a>").unwrap_or(text.len());
                Some(html_decode(&strip_html_inline(&text[..end])))
            })
            .unwrap_or_default();

        if !title.trim().is_empty() && !url.trim().is_empty() {
            let domain = extract_domain(&url);
            results.push(WebSearchResult {
                title: title.trim().to_string(),
                url,
                snippet: snippet.trim().to_string(),
                source: domain.clone(),
                favicon: Some(format!(
                    "https://www.google.com/s2/favicons?domain={}",
                    domain
                )),
                published_date: None,
            });
        }
        rest = remainder;
    }

    if results.is_empty() {
        results = parse_duckduckgo_fallback(html, max_results);
    }

    results
}
pub(super) fn parse_duckduckgo_fallback(html: &str, max_results: usize) -> Vec<WebSearchResult> {
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
pub(super) fn decode_ddg_url(raw_url: &str) -> String {
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
pub(super) fn extract_domain(url: &str) -> String {
    let without_protocol = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    if let Some(slash_pos) = without_protocol.find('/') {
        without_protocol[..slash_pos].to_string()
    } else {
        without_protocol.to_string()
    }
}
pub(super) fn strip_html_inline(s: &str) -> String {
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
pub(super) fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
}
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub(super) fn generate_related_queries(query: &str) -> Vec<String> {
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
pub(super) async fn get_instant_answer(query: &str) -> Option<(String, String)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors html.duckduckgo.com: the title lives on the line after `<a>`.
    const HTML: &str = r#"
<div class="result results_links">
  <h2 class="result__title">
    <a rel="nofollow" class="result__a"
       href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fstory&amp;rut=abc">
      Example <b>Story</b>
    </a>
  </h2>
  <a class="result__snippet" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com">
    A <b>short</b> snippet about the story.
  </a>
</div>
"#;

    #[test]
    fn parses_multiline_anchor_title_and_snippet() {
        let results = parse_duckduckgo_html(HTML, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com/story");
        assert!(results[0].title.contains("Example"));
        assert!(results[0].title.contains("Story"));
        assert!(results[0].snippet.contains("snippet"));
        assert_eq!(results[0].source, "example.com");
    }

    #[test]
    fn multi_line_opening_tag_still_yields_href() {
        let html = "<a rel=\"nofollow\" class=\"result__a\"\nhref=\"https://plain.example/page\">Plain title</a>";
        let results = parse_duckduckgo_html(html, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://plain.example/page");
        assert_eq!(results[0].title, "Plain title");
    }
}
