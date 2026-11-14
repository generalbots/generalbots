use super::super::*;
use chrono::DateTime;

/// Public Google News RSS endpoint — no API key, returns dated article links.
/// Used as a news-oriented source when general SERP scrapers return index pages
/// or undated encyclopedia hits (#2146).
pub(super) async fn search_google_news(
    query: &str,
    max_results: usize,
) -> Result<Vec<WebSearchResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("GeneralBots/1.0 (research)")
        .build()
        .map_err(|e| e.to_string())?;

    let (hl, gl, ceid) = locale_for(query);
    let encoded = urlencoding::encode(query).to_string();
    let url = format!("https://news.google.com/rss/search?q={encoded}&hl={hl}&gl={gl}&ceid={ceid}");

    debug!("Searching Google News RSS: {query}");

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    let body = response.text().await.map_err(|e| e.to_string())?;
    info!(
        "Google News RSS HTTP {status} for '{query}': {} bytes",
        body.len()
    );

    let results = parse_rss(&body, max_results);
    info!(
        "Google News RSS for '{query}' returned {} results",
        results.len()
    );
    Ok(results)
}

fn locale_for(query: &str) -> (&'static str, &'static str, &'static str) {
    let lower = query.to_lowercase();
    let portuguese = lower.contains("notícia")
        || lower.contains("noticia")
        || lower.contains("notícias")
        || lower.contains("noticias")
        || lower.contains("últimas")
        || lower.contains("ultimas")
        || lower.contains("sobre")
        || lower.contains("política")
        || lower.contains("politica");
    if portuguese {
        ("pt-BR", "BR", "BR:pt-419")
    } else {
        ("en-US", "US", "US:en")
    }
}

fn parse_rss(xml: &str, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();
    let mut rest = xml;
    while results.len() < max_results {
        let Some(start) = rest.find("<item>") else {
            break;
        };
        rest = &rest[start + "<item>".len()..];
        let end = rest.find("</item>").unwrap_or(rest.len());
        let item = &rest[..end];
        rest = &rest[end..];

        let raw_title = tag_text(item, "title").unwrap_or_default();
        let link = extract_link(item);
        let pub_date = tag_text(item, "pubDate")
            .as_deref()
            .and_then(to_iso_date)
            .or_else(|| tag_text(item, "dc:date").as_deref().and_then(to_iso_date));
        let source_name = extract_source(item).unwrap_or_default();
        let snippet = tag_text(item, "description")
            .map(|d| strip_tags(&decode_entities(&d)))
            .unwrap_or_default();

        if link.is_empty() || raw_title.is_empty() {
            continue;
        }

        // Google News titles are often "Headline - Outlet"; prefer the outlet
        // from the <source> tag and keep the headline clean.
        let (title, outlet) = split_title_outlet(&raw_title, &source_name);
        let source = host_from_url(&link).unwrap_or_else(|| outlet.clone());
        let favicon = host_from_url(&link)
            .map(|h| format!("https://www.google.com/s2/favicons?domain={h}&sz=32"));

        results.push(WebSearchResult {
            title,
            url: link,
            snippet,
            source,
            favicon,
            published_date: pub_date,
        });
    }
    results
}

fn tag_text(xml: &str, tag: &str) -> Option<String> {
    // Handles <tag>text</tag> and <tag attr="...">text</tag>
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    let after_open = &xml[start + open.len()..];
    let gt = after_open.find('>')?;
    let content_start = start + open.len() + gt + 1;
    let close = format!("</{tag}>");
    let content_end = xml[content_start..].find(&close)? + content_start;
    let raw = &xml[content_start..content_end];
    let text = strip_cdata(raw);
    let decoded = decode_entities(text.trim());
    if decoded.is_empty() {
        None
    } else {
        Some(decoded)
    }
}

fn extract_link(item: &str) -> String {
    // Prefer <link>url</link>; Google News RSS also sometimes uses <guid>.
    if let Some(link) = tag_text(item, "link") {
        let link = link.trim().to_string();
        if link.starts_with("http") {
            return link;
        }
    }
    // Fallback: href inside the item
    if let Some(pos) = item.find("href=\"") {
        let rest = &item[pos + 6..];
        if let Some(end) = rest.find('"') {
            let link = decode_entities(&rest[..end]);
            if link.starts_with("http") {
                return link;
            }
        }
    }
    String::new()
}

fn extract_source(item: &str) -> Option<String> {
    // <source url="https://...">Outlet Name</source>
    let open = "<source";
    let start = item.find(open)?;
    let after = &item[start + open.len()..];
    let gt = after.find('>')?;
    let content_start = start + open.len() + gt + 1;
    let content_end = item[content_start..].find("</source>")? + content_start;
    let name = decode_entities(item[content_start..content_end].trim());
    if name.is_empty() { None } else { Some(name) }
}

fn split_title_outlet(raw: &str, source_name: &str) -> (String, String) {
    let raw = decode_entities(raw.trim());
    if !source_name.is_empty() {
        // Strip trailing " - Source" when it matches
        let suffix = format!(" - {}", source_name);
        if let Some(stripped) = raw.strip_suffix(&suffix) {
            return (stripped.trim().to_string(), source_name.to_string());
        }
        if let Some(pos) = raw.rfind(" - ") {
            let (head, tail) = raw.split_at(pos);
            let tail = tail.trim_start_matches(" - ").trim();
            if tail.eq_ignore_ascii_case(source_name) {
                return (head.trim().to_string(), source_name.to_string());
            }
        }
        return (raw, source_name.to_string());
    }
    if let Some(pos) = raw.rfind(" - ") {
        let (head, tail) = raw.split_at(pos);
        let tail = tail.trim_start_matches(" - ").trim().to_string();
        return (head.trim().to_string(), tail);
    }
    (raw, String::new())
}

fn host_from_url(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed.host_str().map(|h| h.to_string())
}

fn to_iso_date(raw: &str) -> Option<String> {
    // RFC2822 e.g. "Wed, 13 Nov 2026 01:02:03 GMT"
    if let Ok(dt) = DateTime::parse_from_rfc2822(raw.trim()) {
        return Some(dt.date_naive().format("%Y-%m-%d").to_string());
    }
    // ISO-like
    let trimmed = raw.trim();
    if trimmed.len() >= 10 {
        let candidate = &trimmed[..10];
        if candidate.as_bytes().get(4) == Some(&b'-') && candidate.as_bytes().get(7) == Some(&b'-')
        {
            return Some(candidate.to_string());
        }
    }
    None
}

fn strip_cdata(s: &str) -> &str {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("<![CDATA[") {
        inner.strip_suffix("]]>").unwrap_or(inner)
    } else {
        s
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
