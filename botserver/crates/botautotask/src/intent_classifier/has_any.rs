

/// Contains any of the substrings (case already normalized by caller).
pub(super) fn has_any(lower: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| lower.contains(p))
}
/// Contains the full phrase as a word boundary (phrase + whitespace or EOS).
/// Multi-word phrases fall back to substring matching (e.g. "quando eu disser").
pub(super) fn has_phrase(lower: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| {
        if p.contains(' ') {
            lower.contains(p)
        } else {
            lower.split_whitespace().any(|w| w == *p)
        }
    })
}
/// Deterministic resource-name suggestion without any LLM; slugs the first
/// meaningful words of the intent (PT or PT/EN mixture), e.g.
/// "fala todo dia que o comercial vendeu" -> "comercial_vendeu_diario".
pub(super) fn suggest_name(intent: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for word in intent.to_lowercase().split_whitespace() {
        if word.len() < 3 || word.chars().any(|c| !c.is_alphanumeric()) {
            continue;
        }
        let kept = word;
        words.push(kept.to_string());
    }
    let name = words.iter().take(3).cloned().collect::<Vec<_>>().join("_");
    if name.is_empty() {
        return String::new();
    }
    let trimmed: String = name.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
    let mut wrapped: Vec<char> = Vec::new();
    for c in trimmed.chars() {
        if wrapped.is_empty() {
            wrapped.push(c.to_ascii_lowercase());
        } else {
            wrapped.push(c);
        }
    }
    wrapped.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{has_any, has_phrase, suggest_name};

    #[test]
    fn single_words_need_a_word_boundary_and_phrases_do_not() {
        assert!(has_phrase("crm system", &["crm"]));
        assert!(!has_phrase("crmx system", &["crm"]));
        assert!(has_phrase("please create command now", &["create command"]));
        assert!(!has_phrase("createcommand", &["create command"]));
        assert!(has_any("ab crmx", &["crm"]));
        assert!(!has_any("ab crmx", &["xyz"]));
    }

    #[test]
    fn suggest_name_slugs_at_most_three_alphanumeric_words() {
        assert_eq!(suggest_name("Build CRM System Now"), "build_crm_system");
        assert_eq!(suggest_name("fala todo dia que o comercial vendeu"), "fala_todo_dia");
        assert_eq!(suggest_name("!!!"), "");
        assert_eq!(suggest_name(""), "");
    }
}
