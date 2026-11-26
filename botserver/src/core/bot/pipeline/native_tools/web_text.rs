//! Text helpers for `web_fetch`: the web addresses a message names, and a
//! fetched page reduced to the readable text a model can quote.

use std::sync::OnceLock;

/// Most characters of page text handed to the model.
pub(super) const MAX_PAGE_CHARS: usize = 16_000;

/// Elements whose content is never page text.
const SKIPPED: [&str; 9] = [
    "script", "style", "noscript", "template", "svg", "iframe", "head", "canvas", "select",
];

/// Elements that start a new line in the readable text.
const BLOCKS: [&str; 24] = [
    "p", "div", "br", "li", "ul", "ol", "tr", "table", "section", "article", "main", "header",
    "footer", "nav", "aside", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "pre", "dd",
];

fn address_re() -> Option<&'static regex::Regex> {
    static RE: OnceLock<Option<regex::Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(
            r#"(?i)\b(?:https?://[^\s<>"'`]+|www\.[^\s<>"'`]+|(?:[a-z0-9-]+\.)+(?:com|org|net|gov|edu|mil|br|pt|io|info|dev|app|ai|eu|uk|us|int)(?:\.[a-z]{2})?(?:/[^\s<>"'`]*)?)"#,
        )
        .ok()
    })
    .as_ref()
}

/// The web addresses written in `text`, as `http(s)` URLs, in order and
/// without duplicates. A bare `www.` or domain name becomes `https://`.
pub(super) fn addresses_in(text: &str) -> Vec<String> {
    let Some(re) = address_re() else {
        return Vec::new();
    };
    let mut found: Vec<String> = Vec::new();
    for hit in re.find_iter(text) {
        let raw = hit
            .as_str()
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '}']);
        if hit.start() > 0 && text[..hit.start()].ends_with('@') {
            continue;
        }
        let candidate = if raw.to_ascii_lowercase().starts_with("http") {
            raw.to_string()
        } else {
            format!("https://{raw}")
        };
        let Ok(parsed) = url::Url::parse(&candidate) else {
            continue;
        };
        if parsed.host_str().is_none() || found.iter().any(|known| known == parsed.as_str()) {
            continue;
        }
        found.push(parsed.to_string());
    }
    found
}

/// The host of `address` without a leading `www.`, lower case.
pub(super) fn site_of(address: &str) -> Option<String> {
    let parsed = url::Url::parse(address).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    Some(host.strip_prefix("www.").unwrap_or(&host).to_string())
}

/// The page's title and readable text: scripts, styles and markup removed,
/// one line per block, blank runs collapsed, cut at [`MAX_PAGE_CHARS`].
pub(super) fn readable_text(html: &str) -> (String, String) {
    let document = scraper::Html::parse_document(html);
    let title = scraper::Selector::parse("title")
        .ok()
        .and_then(|selector| document.select(&selector).next())
        .map(|element| collapse(&element.text().collect::<String>()))
        .unwrap_or_else(|| { log::debug!("using default for page title"); Default::default() });
    let mut raw = String::new();
    for node in document.root_element().descendants() {
        match node.value() {
            scraper::Node::Element(element) if BLOCKS.contains(&element.name()) => raw.push('\n'),
            scraper::Node::Text(text) => {
                let hidden = node.ancestors().any(|ancestor| {
                    ancestor
                        .value()
                        .as_element()
                        .is_some_and(|element| SKIPPED.contains(&element.name()))
                });
                if !hidden {
                    raw.push_str(text);
                }
            }
            _ => {}
        }
    }
    (title, limit(&tidy(&raw)))
}

/// Plain text (or JSON, CSV) tidied and cut like page text.
pub(super) fn plain_text(body: &str) -> String {
    limit(&tidy(body))
}

fn collapse(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tidy(raw: &str) -> String {
    raw.lines()
        .map(collapse)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn limit(text: &str) -> String {
    match text.char_indices().nth(MAX_PAGE_CHARS) {
        Some((cut, _)) => format!("{}\n[...]", &text[..cut]),
        None => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_are_found_in_a_sentence() {
        let found = addresses_in(
            "Leia https://www.gov.br/pt-br/noticias?id=3, depois www.example.org e g1.globo.com/economia.",
        );
        assert_eq!(
            found,
            vec![
                "https://www.gov.br/pt-br/noticias?id=3".to_string(),
                "https://www.example.org/".to_string(),
                "https://g1.globo.com/economia".to_string(),
            ]
        );
    }

    #[test]
    fn file_names_and_mail_addresses_are_not_addresses() {
        assert!(addresses_in("abra relatorio.pdf e planilha.xlsx").is_empty());
        assert!(addresses_in("escreva para fulano@orgao.gov.br").is_empty());
        assert!(addresses_in("sem endereço nenhum").is_empty());
    }

    #[test]
    fn the_site_ignores_www_and_case() {
        assert_eq!(site_of("https://WWW.Example.org/a").as_deref(), Some("example.org"));
        assert_eq!(site_of("not a url"), None);
    }

    #[test]
    fn page_text_drops_scripts_and_keeps_blocks() {
        let html = "<html><head><title> Diário  Oficial </title><style>p{}</style></head>\
                    <body><script>var x = 1;</script><h1>Portaria 12</h1><p>Texto   da\n portaria.</p>\
                    <ul><li>um</li><li>dois</li></ul><noscript>ligue o JS</noscript></body></html>";
        let (title, text) = readable_text(html);
        assert_eq!(title, "Diário Oficial");
        assert_eq!(text, "Portaria 12\nTexto da\nportaria.\num\ndois");
    }

    #[test]
    fn long_text_is_cut() {
        let long = "a".repeat(MAX_PAGE_CHARS + 10);
        let cut = plain_text(&long);
        assert!(cut.ends_with("[...]"));
        assert_eq!(cut.chars().count(), MAX_PAGE_CHARS + "\n[...]".chars().count());
    }
}
