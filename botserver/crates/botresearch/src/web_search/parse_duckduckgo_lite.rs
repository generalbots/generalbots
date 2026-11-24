use super::*;

/// The page DuckDuckGo serves an automated client instead of results: HTTP
/// 202 with an image challenge ("anomaly" modal) and no result markup.
pub(super) fn is_ddg_challenge(status: u16, html: &str) -> bool {
    status == 202 || html.contains("anomaly-modal")
}

/// Whether a result address is a DuckDuckGo ad redirect rather than a page.
fn is_ad(url: &str) -> bool {
    url.contains("duckduckgo.com/y.js") || url.contains("ad_domain=")
}

/// Parses `lite.duckduckgo.com/lite/`: each result is a table row with a
/// `result-link` anchor, followed by a row whose `result-snippet` cell holds
/// the abstract. Attributes mix single and double quotes.
pub(super) fn parse_duckduckgo_lite(html: &str, max_results: usize) -> Vec<WebSearchResult> {
    let mut results = Vec::new();
    let mut rest = html;

    while results.len() < max_results {
        let Some(marker) = rest.find("result-link") else {
            break;
        };
        let anchor_start = rest[..marker].rfind("<a ").unwrap_or(marker);
        let after = &rest[anchor_start..];
        let Some(open_end) = after.find('>') else {
            break;
        };
        let open_tag = &after[..open_end];
        let Some(close) = after[open_end..].find("</a>") else {
            break;
        };
        let title = html_decode(&strip_html_inline(&after[open_end + 1..open_end + close]));
        let remainder = &after[open_end + close + 4..];
        let block_end = remainder.find("result-link").unwrap_or(remainder.len());
        let block = &remainder[..block_end];

        let snippet = block
            .find("result-snippet")
            .and_then(|at| {
                let cell = &block[at..];
                let gt = cell.find('>')?;
                let text = &cell[gt + 1..];
                let end = text.find("</td>").unwrap_or(text.len());
                Some(html_decode(&strip_html_inline(&text[..end])))
            })
            .unwrap_or_default();

        let url = quoted_attr(open_tag, "href")
            .map(|raw| decode_ddg_url(&html_decode(&raw)))
            .unwrap_or_default();
        if !title.trim().is_empty() && url.starts_with("http") && !is_ad(&url) {
            let domain = extract_domain(&url);
            results.push(WebSearchResult {
                title: title.trim().to_string(),
                url,
                snippet: snippet.trim().to_string(),
                source: domain.clone(),
                favicon: Some(format!("https://www.google.com/s2/favicons?domain={domain}")),
                published_date: None,
            });
        }
        rest = remainder;
    }

    results
}

/// An attribute value in double or single quotes.
fn quoted_attr(tag: &str, name: &str) -> Option<String> {
    let at = tag.find(&format!("{name}="))? + name.len() + 1;
    let quote = tag[at..].chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let start = at + 1;
    let end = tag[start..].find(quote)?;
    Some(tag[start..start + end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors lite.duckduckgo.com: an ad row, then two organic results.
    const LITE: &str = r#"
<table>
  <tr class='result-sponsored'>
    <td><a rel="nofollow" href="https://duckduckgo.com/y.js?ad_domain=shop.example&amp;u3=x" class='result-link'>Buy now</a></td>
  </tr>
  <tr>
    <td valign="top">1.&nbsp;</td>
    <td>
      <a rel="nofollow" href="https://www.python.org/downloads/" class='result-link'>Download <b>Python</b> | Python.org</a>
    </td>
  </tr>
  <tr>
    <td>&nbsp;&nbsp;&nbsp;</td>
    <td class='result-snippet'>
      The official home of the <b>Python</b> Programming Language&#x27;s releases.
    </td>
  </tr>
  <tr><td><span class='link-text'>www.python.org/downloads/</span></td></tr>
  <tr>
    <td valign="top">2.&nbsp;</td>
    <td><a rel="nofollow" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fen.wikipedia.org%2Fwiki%2FPython&amp;rut=1" class='result-link'>Python - Wikipedia</a></td>
  </tr>
</table>
"#;

    #[test]
    fn parses_lite_results_and_skips_ads() {
        let results = parse_duckduckgo_lite(LITE, 8);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Download Python | Python.org");
        assert_eq!(results[0].url, "https://www.python.org/downloads/");
        assert_eq!(results[0].source, "www.python.org");
        assert_eq!(
            results[0].snippet,
            "The official home of the Python Programming Language's releases."
        );
        assert_eq!(results[1].url, "https://en.wikipedia.org/wiki/Python");
        assert!(results[1].snippet.is_empty());
    }

    #[test]
    fn lite_respects_the_result_limit() {
        assert_eq!(parse_duckduckgo_lite(LITE, 1).len(), 1);
    }

    #[test]
    fn detects_the_challenge_page() {
        assert!(is_ddg_challenge(202, ""));
        assert!(is_ddg_challenge(200, "<div class=\"anomaly-modal__box\">"));
        assert!(!is_ddg_challenge(200, LITE));
        assert!(parse_duckduckgo_lite("<div class=\"anomaly-modal\"></div>", 8).is_empty());
    }
}
