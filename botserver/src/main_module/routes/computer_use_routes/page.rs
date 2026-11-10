//! Page fetch/extract for browser computer-use actions (#1644).

use botsecurity_core::validation::{fetch_user_url, FetchPolicy};
use std::time::Duration;

pub(super) struct PageContent {
    pub(super) url: String,
    pub(super) title: String,
    pub(super) text: String,
}

/// Fetches a page server-side and returns a bounded, readable extract. This is
/// what lets the assistant cite a real page title even when the client iframe
/// cannot read a cross-origin document.
///
/// The URL comes from the approved computer-use action (i.e. from the LLM or
/// the user), so it goes through the shared SSRF guard (#2131): only public
/// http(s) targets are reachable, redirects are re-validated per hop, and the
/// body is capped. The error text is generic — the rejected host never reaches
/// the caller.
pub(super) async fn fetch_page(url: &str) -> Result<PageContent, String> {
    let policy = FetchPolicy::default()
        .with_timeout(Duration::from_secs(20))
        .with_user_agent("GeneralBots-ComputerUse/1.0")
        .with_max_bytes(2_000_000);
    let response = match fetch_user_url(url, &policy).await {
        Ok(response) => response,
        Err(err) => {
            log::warn!(
                "computer_use: refused or failed page fetch ({}): {}",
                err.kind().as_str(),
                err.detail()
            );
            return Err(err.to_string());
        }
    };
    if !response.is_success() {
        return Err(format!("the page answered HTTP {}", response.status));
    }
    let html = response.text();
    Ok(PageContent {
        url: response.final_url,
        title: extract_tag(&html, "title").unwrap_or_default(),
        text: html_to_text(&html, 12_000),
    })
}

fn extract_tag(html: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"(?is)<{tag}[^>]*>(.*?)</{tag}>");
    let re = regex::Regex::new(&pattern).ok()?;
    let captures = re.captures(html)?;
    let value = captures.get(1)?.as_str().trim();
    if value.is_empty() {
        None
    } else {
        Some(decode_entities(value))
    }
}

fn html_to_text(html: &str, limit: usize) -> String {
    let mut out = String::with_capacity(limit.min(html.len()));
    let mut tag = String::new();
    let mut in_tag = false;
    let mut skip = false;
    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                let closing = tag.starts_with('/');
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                match name.as_str() {
                    "script" | "style" | "noscript" => skip = !closing,
                    "p" | "br" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "td" => {
                        out.push(' ')
                    }
                    _ => {}
                }
                in_tag = false;
                tag.clear();
            } else {
                tag.push(ch);
            }
        } else if ch == '<' {
            in_tag = true;
            tag.clear();
        } else if !skip {
            out.push(ch);
        }
        if out.len() >= limit {
            break;
        }
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    decode_entities(&collapsed)
}

fn decode_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}
