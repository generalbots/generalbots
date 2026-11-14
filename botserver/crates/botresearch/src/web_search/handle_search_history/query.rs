/// Normalizes a search query for scraping-friendly engines: strips accents
/// (DuckDuckGo/Mojeek serve anti-bot challenge pages for %20-encoded or
/// accented queries) and joins words with '+' like a form submission.
pub(super) fn normalize_query(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    for c in query.chars() {
        let mapped = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'Ç' => 'C',
            'Ñ' => 'N',
            _ => c,
        };
        if mapped.is_whitespace() {
            if !out.ends_with('+') {
                out.push('+');
            }
        } else {
            out.push(mapped);
        }
    }
    out.trim_end_matches('+').to_string()
}

/// Strips conversational scaffolding ("What is ... ? Cite your sources") so
/// the engine sees the topic, not the question wrapper.
pub fn clean_query(query: &str) -> String {
    let mut text = query.trim().to_string();
    let mut changed = true;
    while changed {
        changed = false;
        let lower = text.to_lowercase();
        for prefix in [
            "what is the ",
            "what are the ",
            "what is ",
            "what are ",
            "who is ",
            "who are ",
            "how many ",
            "how much ",
            "when did ",
            "when was ",
            "where is ",
            "where are ",
            "tell me ",
            "can you tell me ",
            "could you tell me ",
            "do you know ",
            "please ",
            "search the web for ",
            "search web for ",
            "search for ",
            "search the web ",
            "search the internet for ",
            "look up ",
            "google ",
        ] {
            if lower.starts_with(prefix) {
                text = text[prefix.len()..].to_string();
                changed = true;
                break;
            }
        }
        if changed {
            continue;
        }
        let lower = text.to_lowercase();
        for phrase in [
            " cite your sources",
            " cite sources",
            " give sources",
            " with sources",
            " according to the latest official estimate",
            " according to official sources",
        ] {
            if let Some(at) = lower.find(phrase) {
                text = format!("{}{}", &text[..at], &text[at + phrase.len()..]);
                changed = true;
                break;
            }
        }
    }
    let text = text
        .trim()
        .trim_end_matches(|c: char| matches!(c, '?' | '.' | '!' | ','))
        .trim()
        .to_string();
    if text.is_empty() {
        query.trim().to_string()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn clean_query_strips_the_search_instruction() {
        assert_eq!(
            super::clean_query("search web for computers virus recent"),
            "computers virus recent"
        );
        assert_eq!(
            super::clean_query("Search the web for the latest virus news."),
            "the latest virus news"
        );
        assert_eq!(
            super::clean_query("google ransomware 2026"),
            "ransomware 2026"
        );
    }
}
