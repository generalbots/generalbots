//! #2146 — source quality for the composer's Web switch.
//!
//! UAT-03 ("Notícias do Lula") produced a digest whose links were section
//! indexes (`/tudo-sobre/lula/`, `/politica/lula/`) with no per-item date. The
//! search hits were handed to the model verbatim, so a hub page became an
//! "article" and a hit without `published_date` became a line without a date.
//! This module turns raw hits into citable items before they reach the prompt:
//! indexes and dead URLs are removed, duplicates collapse, and every surviving
//! URL is probed once for its real publish date.

use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

use botresearch::web_search::WebSearchResult;

/// One bounded GET per hit is cheaper than the HEAD probe this replaced: the
/// first bytes of the article answer liveness *and* the publish date.
const PROBE_TIMEOUT: Duration = Duration::from_secs(6);
const PROBE_PREFIX_BYTES: usize = 180_000;
const USER_AGENT: &str = "Mozilla/5.0 (compatible; GeneralBots/1.0; +https://generalbo.ts)";

/// Path fragments that name a *section or tag index* instead of an article.
/// Citing one sends the reader to a page that changes every day, which is
/// exactly what the UAT marked as a miss.
const INDEX_MARKERS: &[&str] = &[
    "/tudo-sobre/",
    "/noticias-sobre/",
    "/ultimas-noticias/",
    "/ultimas/",
    "/tag/",
    "/tags/",
    "/topic/",
    "/topics/",
    "/secao/",
    "/secoes/",
    "/categoria/",
    "/category/",
    "/autor/",
    "/author/",
];

/// True when the URL addresses a hub (home page, tag, section, author) rather
/// than one article.
pub fn is_index_page(url: &str) -> bool {
    let path = match url.split_once("://") {
        Some((_, rest)) => rest.split_once('/').map(|(_, path)| path).unwrap_or(""),
        None => url.trim_start_matches('/'),
    };
    let path = format!("/{}", path.split(['?', '#']).next().unwrap_or("")).to_lowercase();
    if path.trim_matches('/').is_empty() {
        return true;
    }
    INDEX_MARKERS.iter().any(|marker| path.contains(marker))
}

/// Case- and punctuation-insensitive headline used to spot the same story
/// syndicated under two URLs, and duplicated hits from one provider.
fn headline_key(title: &str) -> String {
    title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.chars().count() > 2)
        .take(10)
        .collect::<Vec<_>>()
        .join(" ")
}

fn date_res() -> &'static [regex::Regex] {
    static RES: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    RES.get_or_init(|| {
        [
            r#"property=["']article:published_time["'][^>]*content=["']([^"']+)"#,
            r#"content=["']([^"']+)["'][^>]*property=["']article:published_time"#,
            r#"itemprop=["']datePublished["'][^>]*content=["']([^"']+)"#,
            r#"<time[^>]*datetime=["']([^"']+)"#,
            r#""datePublished"\s*:\s*"([^"]+)"#,
            r#""dateModified"\s*:\s*"([^"]+)"#,
        ]
        .iter()
        .filter_map(|pattern| regex::Regex::new(pattern).ok())
        .collect()
    })
    .as_slice()
}

/// `2026-11-12T18:03:00-03:00` and `12/11/2026` both become `2026-11-12`.
pub fn normalize_date(raw: &str) -> Option<String> {
    let value = raw.trim();
    if let Some((date, _)) = value.split_once('T') {
        if is_iso_date(date) {
            return Some(date.to_string());
        }
    }
    if is_iso_date(value) {
        return Some(value.to_string());
    }
    // dd/mm/yyyy (Brazilian outlets) — the only other shape worth accepting.
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() == 3 {
        let (day, month, year) = (parts[0], parts[1], parts[2]);
        let year = year.split(|c: char| !c.is_ascii_digit()).next()?;
        if !day.is_empty() && !month.is_empty() && year.len() == 4 {
            let day: u32 = day.parse().ok()?;
            let month: u32 = month.parse().ok()?;
            if (1..=31).contains(&day) && (1..=12).contains(&month) {
                return Some(format!("{year}-{month:02}-{day:02}"));
            }
        }
    }
    None
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// Publish date carried by the article markup (meta tags, JSON-LD, `<time>`).
fn date_from_markup(body: &str) -> Option<String> {
    for re in date_res() {
        if let Some(captures) = re.captures(body) {
            if let Some(date) = captures.get(1).and_then(|m| normalize_date(m.as_str())) {
                return Some(date);
            }
        }
    }
    None
}

/// `https://site/2026/11/12/slug` and `https://site/2026-11-12/slug`.
fn date_from_url(url: &str) -> Option<String> {
    let segments: Vec<&str> = url.split('/').collect();
    for (index, segment) in segments.iter().enumerate() {
        let is_year = segment.len() == 4 && segment.chars().all(|c| c.is_ascii_digit());
        if is_year {
            let (Some(month), Some(day)) = (segments.get(index + 1), segments.get(index + 2))
            else {
                continue;
            };
            let day_digits: String = day.chars().take(2).collect();
            let month_digits: String = month.chars().take(2).collect();
            let candidate = format!("{segment}-{month_digits}-{day_digits}");
            if day_digits.len() == 2 && month_digits.len() == 2 && is_iso_date(&candidate) {
                return Some(candidate);
            }
        }
        if let Some((date, _)) = segment.split_once('T') {
            if is_iso_date(date) {
                return Some(date.to_string());
            }
        }
    }
    None
}

/// A date written in the snippet ("12/11/2026", "há 2 horas" is not a date).
fn date_from_text(text: &str) -> Option<String> {
    static RE: OnceLock<Option<regex::Regex>> = OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\b(\d{1,2}/\d{1,2}/\d{4})\b").ok());
    let captures = re.as_ref()?.captures(text)?;
    normalize_date(captures.get(1)?.as_str())
}

/// True when the page declares itself a site (not an article) *and* the URL
/// ends in `/` — the shape of `/politica/lula/` section hubs that carry no
/// date. Wikipedia-style `/wiki/Title` pages stay: they are not ending in `/`.
fn is_hub_markup(body: &str) -> bool {
    static RE: OnceLock<Option<regex::Regex>> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r#"property=["']og:type["'][^>]*content=["']([^"']+)"#).ok()
    });
    let value = re
        .as_ref()
        .and_then(|re| re.captures(body))
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_lowercase())
        .unwrap_or_default();
    value == "website"
}

/// Outcome of the single probe request behind each hit.
enum Probe {
    Keep,
    Dead,
    Hub,
}

/// One bounded GET: `Dead` when the URL is definitively gone (404/410), `Hub`
/// when the page self-describes as a section index. A bot challenge or a
/// network error is not evidence that the article does not exist, so the hit
/// stays and simply carries no date.
async fn probe(client: &reqwest::Client, result: &mut WebSearchResult) -> Probe {
    let mut response = match client.get(&result.url).send().await {
        Ok(response) => response,
        Err(_) => return Probe::Keep,
    };
    let status = response.status().as_u16();
    if matches!(status, 404 | 410) {
        return Probe::Dead;
    }
    if !(200..300).contains(&status) {
        return Probe::Keep;
    }
    let mut body = String::new();
    while body.len() < PROBE_PREFIX_BYTES {
        match response.chunk().await {
            Ok(Some(chunk)) => body.push_str(&String::from_utf8_lossy(&chunk)),
            Ok(None) => break,
            Err(_) => break,
        }
    }
    let date = date_from_markup(&body)
        .or_else(|| date_from_url(&result.url))
        .or_else(|| date_from_text(&result.snippet));
    if date.is_none() && result.url.trim_end().ends_with('/') && is_hub_markup(&body) {
        return Probe::Hub;
    }
    if result.published_date.is_none() {
        result.published_date = date;
    }
    Probe::Keep
}

/// Removes index pages and repeated stories, then fills in the publish date of
/// each surviving hit. An empty answer means every hit was unusable.
pub async fn refine(results: Vec<WebSearchResult>) -> Vec<WebSearchResult> {
    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut seen_titles: HashSet<String> = HashSet::new();
    let mut kept: Vec<WebSearchResult> = Vec::new();
    for result in results {
        let url = result.url.trim().to_string();
        if url.is_empty() || is_index_page(&url) {
            log::info!("web context: skipped index or empty source {url}");
            continue;
        }
        let key = headline_key(&result.title);
        let is_duplicate_url = !seen_urls.insert(url.clone());
        let is_duplicate_title = !key.is_empty() && !seen_titles.insert(key);
        if is_duplicate_url || is_duplicate_title {
            log::info!("web context: dropped duplicate source {url}");
            continue;
        }
        kept.push(result);
    }
    if kept.is_empty() {
        return kept;
    }

    let client = match reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            log::debug!("web context: probe client unavailable: {error}");
            return kept;
        }
    };

    let probes = kept.into_iter().map(|mut result| {
        let client = client.clone();
        async move {
            let outcome = probe(&client, &mut result).await;
            (result, outcome)
        }
    });
    let probed = futures::future::join_all(probes).await;

    let mut out = Vec::new();
    for (result, outcome) in probed {
        match outcome {
            Probe::Keep => out.push(result),
            Probe::Dead => log::info!("web context: dropped dead source {}", result.url),
            Probe::Hub => log::info!("web context: dropped section page {}", result.url),
        }
    }
    out
}

/// Normalised publish date for citations (`30_citations.js` renders it next to
/// the outlet so every chip carries a date, not just the prose).
pub fn item_date(result: &WebSearchResult) -> Option<String> {
    result
        .published_date
        .as_deref()
        .and_then(normalize_date)
        .or_else(|| date_from_url(&result.url))
}

/// The grounding block. Each hit is printed with the fields the answer must
/// carry (headline, article URL, outlet, date) so the model never has to infer
/// them — and never fills a gap with an invented link or date.
pub fn prompt_block(results: &[WebSearchResult]) -> String {
    let mut block = String::from(
        "Fresh web search results are provided below. They are sufficient for \
         this turn: answer the user NOW from this material. Do not call \
         api.find, web.search or any other command to look for a search tool — \
         the search has already been done for you.\n\n\
         NEWS DIGEST CONTRACT\n\
         - List at least 5 items when 5 or more article sources are provided; \
         otherwise list every article source provided. Never invent items, \
         headlines, outlets, dates or URLs to reach a count.\n\
         - Order newest first when dates are available, one per topic. Never \
         repeat a headline or a URL.\n\
         - Every item is ONE line in this exact shape: \
         `- **[headline](ARTICLE_URL)** — outlet, published YYYY-MM-DD`.\n\
         - Use the ARTICLE_URL printed for that item, verbatim. Never cite a \
         section, tag, home or \"tudo sobre\" index page; those are not articles \
         and were already removed. Never invent a URL, an outlet or a date.\n\
         - When an item shows `published: not shown`, say the date is not shown \
         by that source instead of guessing one. Prefer items with real \
         publication dates for time-sensitive news requests.\n\
         - If the material does not answer the question, say clearly in one \
         line what is missing. Never pad the list with filler.\n\
         - Include a short summary in the language of the user's message and \
         never print raw JSON or the field names above.\n\n\
         SOURCES\n",
    );
    for (index, result) in results.iter().enumerate() {
        let date = item_date(result).unwrap_or_else(|| "not shown".to_string());
        block.push_str(&format!(
            "--- item {} ---\nheadline: {}\narticle_url: {}\noutlet: {}\npublished: {}\n{}\n\n",
            index + 1,
            result.title,
            result.url,
            result.source,
            date,
            result.snippet
        ));
    }
    block
}

#[cfg(test)]
mod web_source_tests {
    use super::{headline_key, is_index_page, normalize_date, prompt_block};
    use botresearch::web_search::WebSearchResult;

    fn result(title: &str, url: &str, date: Option<&str>) -> WebSearchResult {
        WebSearchResult {
            title: title.to_string(),
            url: url.to_string(),
            snippet: "snippet".to_string(),
            source: "example.com".to_string(),
            favicon: None,
            published_date: date.map(str::to_string),
        }
    }

    /// The UAT-03 links were all `/tudo-sobre/` style hubs.
    #[test]
    fn section_and_tag_pages_are_not_articles() {
        for hub in [
            "https://www.r7.com/tudo-sobre/lula/",
            "https://www.cnnbrasil.com.br/tudo-sobre/governo-lula/",
            "https://veja.abril.com.br/noticias-sobre/governo-lula/",
            "https://noticias.uol.com.br/",
            "https://www1.folha.uol.com.br/tag/lula",
        ] {
            assert!(is_index_page(hub), "hub not detected: {hub}");
        }
        for article in [
            "https://www1.folha.uol.com.br/poder/2026/11/lula-recebe-ministros.shtml",
            "https://g1.globo.com/politica/noticia/2026/11/12/lula-fala.ghtml",
        ] {
            assert!(!is_index_page(article), "article dropped: {article}");
        }
    }

    #[test]
    fn dates_are_normalised_to_iso() {
        assert_eq!(
            normalize_date("2026-11-12T18:03:00-03:00").as_deref(),
            Some("2026-11-12")
        );
        assert_eq!(normalize_date("2026-11-12").as_deref(), Some("2026-11-12"));
        assert_eq!(normalize_date("12/11/2026").as_deref(), Some("2026-11-12"));
        assert_eq!(normalize_date("ontem"), None);
        assert_eq!(normalize_date(""), None);
    }

    #[test]
    fn headlines_dedupe_across_punctuation() {
        assert_eq!(
            headline_key("Lula: fala sobre juros!"),
            headline_key("Lula fala sobre juros")
        );
        assert_ne!(
            headline_key("Lula fala sobre juros"),
            headline_key("Bolsonaro nega")
        );
    }

    #[test]
    fn the_block_carries_the_contract_and_every_field() {
        let block = prompt_block(&[
            result(
                "Lula recebe ministros",
                "https://g1.globo.com/politica/noticia/2026/11/12/lula-fala.ghtml",
                Some("2026-11-12"),
            ),
            result(
                "Outra notícia",
                "https://exemplo.com/2026/11/10/outra",
                None,
            ),
            result("Sem data", "https://exemplo.com/sem-data", None),
        ]);
        assert!(block.contains("NEWS DIGEST CONTRACT"));
        assert!(
            block.contains("never repeat a headline"),
            "contract missing"
        );
        assert!(block.contains(
            "article_url: https://g1.globo.com/politica/noticia/2026/11/12/lula-fala.ghtml"
        ));
        assert!(block.contains("published: 2026-11-12"));
        // The date is recovered from the URL when the provider gives none...
        assert!(block.contains("published: 2026-11-10"));
        // ...and a hit with no date at all is reported as such, never invented.
        assert!(block.contains("published: not shown"));
    }
}
