use super::store::UserMemoryRow;

/// Imperative phrases that explicitly ask the assistant to forget a stored
/// fact. Longest prefixes first: `extract_forget_directive` takes the first
/// match, so "forget everything about " must win over "forget ".
const FORGET_PREFIXES: &[&str] = &[
    "please forget everything about ",
    "please forget everything ",
    "please forget that ",
    "please forget: ",
    "please forget ",
    "forget everything about ",
    "forget everything: ",
    "forget everything ",
    "forget all about ",
    "forget that ",
    "forget: ",
    "forget:",
    "forget about ",
    "forget ",
    "por favor esqueça tudo sobre ",
    "por favor esqueça tudo ",
    "por favor esqueça que ",
    "por favor esqueça ",
    "esqueça-se de que ",
    "esqueça-se que ",
    "esqueça que ",
    "esqueça: ",
    "esqueça ",
    "esqueca que ",
    "esqueca: ",
    "esqueca ",
    "apague da memória ",
    "apague da memoria ",
    "remova da memória ",
    "remova da memoria ",
    "delete my memory about ",
    "delete my memory of ",
    "remove my memory about ",
    "remove my memory of ",
];

/// Words that carry no identifying value when matching a forget request
/// against stored rows. Kept small: an unknown word is better than a wrong
/// deletion, and matching requires a distinctive token (four characters or
/// more) from the stored key/value.
const FORGET_STOPWORDS: &[&str] = &[
    "about",
    "preference",
    "preferences",
    "memory",
    "please",
    "remember",
    "forget",
    "everything",
    "tudo",
    "sobre",
    "minha",
    "minhas",
    "meus",
    "preferencia",
    "preferências",
    "preferencias",
    "memoria",
    "memória",
    "favor",
    "with",
    "from",
    "that",
    "this",
    "the",
    "and",
];

/// Imperative phrases that explicitly ask the assistant to remember a fact.
/// Kept prefix-only: "do you remember…" is a question, not a write.
const REMEMBER_PREFIXES: [&str; 20] = [
    "please remember that ",
    "please remember: ",
    "please remember:",
    "please remember ",
    "remember that ",
    "remember: ",
    "remember:",
    "remember ",
    "lembre-se de que ",
    "lembre-se que ",
    "lembre-se: ",
    "lembre-se:",
    "lembre-se ",
    "lembre que ",
    "lembre: ",
    "lembre:",
    "lembre ",
    "guarde que ",
    "guarde ",
    "anote que ",
];

/// Turns an explicit "remember …" turn into the fact to persist (#1642).
/// Returns None when the message is not an explicit memory request, so an
/// ordinary question never becomes a stored preference.
pub fn extract_remember_directive(text: &str) -> Option<String> {
    let original = text.trim();
    let lower = original.to_lowercase();
    let prefix = REMEMBER_PREFIXES
        .iter()
        .find(|prefix| lower.starts_with(**prefix))?;
    let fact: String = original
        .chars()
        .skip(prefix.chars().count())
        .collect::<String>()
        .trim()
        .trim_end_matches(|c| c == '.' || c == '!' || c == ';')
        .trim()
        .to_string();
    if fact.len() < 3 || fact.len() > 500 {
        return None;
    }
    Some(fact)
}

/// Stable, readable key derived from the fact so repeating it updates the same
/// row instead of piling up near-duplicates.
pub fn memory_key_for(fact: &str) -> String {
    let mut key = String::with_capacity(48);
    let mut last_dash = false;
    for ch in fact.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            key.push(ch);
            last_dash = false;
        } else if !last_dash {
            key.push('-');
            last_dash = true;
        }
        if key.len() >= 48 {
            break;
        }
    }
    let trimmed = key.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "note".to_string()
    } else {
        trimmed
    }
}

/// Turns an explicit "forget …" turn into the text to match against stored
/// memories (#1708). Returns None when the message is not a forget request.
pub fn extract_forget_directive(text: &str) -> Option<String> {
    let original = text.trim();
    let lower = original.to_lowercase();
    let prefix = FORGET_PREFIXES
        .iter()
        .find(|prefix| lower.starts_with(**prefix))?;
    let needle: String = original
        .chars()
        .skip(prefix.chars().count())
        .collect::<String>()
        .trim()
        .trim_end_matches(|c| c == '.' || c == '!' || c == ';')
        .trim()
        .to_string();
    if needle.is_empty() {
        return None;
    }
    Some(needle)
}

/// Distinctive lowercase tokens used to match a forget request against the
/// stored rows. Two-character tokens ("pt", "br") are dropped: they match too
/// much text. A whole-memory request ("forget everything") bypasses matching.
fn forget_tokens(needle: &str) -> Vec<String> {
    needle
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.len() >= 3 && !FORGET_STOPWORDS.contains(token))
        .map(str::to_string)
        .collect()
}

pub(super) fn select_forget_keys(rows: &[UserMemoryRow], needle: &str) -> Vec<String> {
    let lower = needle.to_lowercase();
    if lower.contains("everything") || lower.contains("tudo") {
        return rows.iter().map(|row| row.memory_key.clone()).collect();
    }
    let wanted = forget_tokens(needle);
    rows.iter()
        .filter(|row| {
            let haystack = format!("{} {}", row.memory_key, row.memory_value).to_lowercase();
            wanted
                .iter()
                .any(|token| token.len() >= 4 && haystack.contains(token.as_str()))
        })
        .map(|row| row.memory_key.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        extract_forget_directive, extract_remember_directive, memory_key_for, select_forget_keys,
    };
    use crate::core::user_memory::store::UserMemoryRow;

    fn row(key: &str, value: &str) -> UserMemoryRow {
        UserMemoryRow {
            memory_key: key.to_string(),
            memory_value: value.to_string(),
            bot_id: None,
        }
    }

    #[test]
    fn forget_requests_are_detected_and_strip_the_prefix() {
        assert_eq!(
            extract_forget_directive(
                "Forget my preference about writing in pt-BR and preferring tables."
            ),
            Some("my preference about writing in pt-BR and preferring tables".to_string())
        );
        assert_eq!(
            extract_forget_directive("esqueça que meu fuso é São Paulo"),
            Some("meu fuso é São Paulo".to_string())
        );
        assert!(extract_forget_directive("forget").is_none());
        assert!(extract_forget_directive("what do you remember?").is_none());
    }

    #[test]
    fn forget_matches_the_stored_row_by_distinctive_token() {
        let rows = vec![
            row(
                "i-write-in-pt-br-and-prefer-tables",
                "I write in pt-BR and prefer tables",
            ),
            row("favourite-drink", "espresso"),
        ];
        let keys = select_forget_keys(
            &rows,
            "my preference about writing in pt-BR and preferring tables",
        );
        assert_eq!(keys, vec!["i-write-in-pt-br-and-prefer-tables".to_string()]);
    }

    #[test]
    fn forget_everything_selects_every_row() {
        let rows = vec![row("a", "one"), row("b", "two")];
        let keys = select_forget_keys(&rows, "everything");
        assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn forget_without_a_matching_token_deletes_nothing() {
        let rows = vec![row("favourite-drink", "espresso")];
        assert!(select_forget_keys(&rows, "my shoe size").is_empty());
    }

    #[test]
    fn explicit_memory_requests_are_captured() {
        assert_eq!(
            extract_remember_directive("remember I prefer invoices as PDF"),
            Some("I prefer invoices as PDF".to_string())
        );
        assert_eq!(
            extract_remember_directive("Lembre-se que meu fuso é São Paulo."),
            Some("meu fuso é São Paulo".to_string())
        );
        // The colon form is the one people actually type; it used to be
        // ignored because the prefix required a space.
        assert_eq!(
            extract_remember_directive("Remember: I write in pt-BR and prefer tables."),
            Some("I write in pt-BR and prefer tables".to_string())
        );
        assert_eq!(
            extract_remember_directive("Lembre-se: prefiro tabelas."),
            Some("prefiro tabelas".to_string())
        );
    }

    #[test]
    fn questions_are_not_memories() {
        assert!(extract_remember_directive("do you remember my name?").is_none());
        assert!(extract_remember_directive("remember").is_none());
        assert!(extract_remember_directive("what is the weather?").is_none());
    }

    #[test]
    fn keys_are_stable_and_readable() {
        assert_eq!(
            memory_key_for("I prefer invoices as PDF"),
            "i-prefer-invoices-as-pdf"
        );
        assert_eq!(memory_key_for("   "), "note");
    }
}
