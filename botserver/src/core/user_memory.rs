//! Persisted user memory (GB6-16).
//!
//! Preferences edited in Settings → Memory live in `chat_preferences`; this module
//! is the server-side reader that puts them into the conversation prompt, so a
//! new session starts with what the user already taught the assistant instead
//! of repeating it.

use diesel::prelude::*;
use uuid::Uuid;

use botcore::shared::utils::DbPool;
use botschema::chat_preferences;

/// Entries the prompt may carry. Keeps the injected block small and bounded.
const MAX_PROMPT_ENTRIES: i64 = 24;
const MAX_PROMPT_CHARS: usize = 2000;

#[derive(Queryable)]
pub struct UserMemoryRow {
    pub memory_key: String,
    pub memory_value: String,
    pub bot_id: Option<Uuid>,
}

/// Memories that apply to this bot: its own entries plus the global ones.
pub fn load_for_user(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
) -> Result<Vec<UserMemoryRow>, String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let mut query = chat_preferences::table
        .filter(chat_preferences::user_id.eq(user_id))
        .into_boxed();
    if let Some(bot_id) = bot_id {
        query = query.filter(
            chat_preferences::bot_id
                .eq(bot_id)
                .or(chat_preferences::bot_id.is_null()),
        );
    }
    query
        .order(chat_preferences::bot_id.asc())
        .then_order_by(chat_preferences::updated_at.desc())
        .limit(MAX_PROMPT_ENTRIES)
        .select((
            chat_preferences::memory_key,
            chat_preferences::memory_value,
            chat_preferences::bot_id,
        ))
        .load::<UserMemoryRow>(&mut conn)
        .map_err(|error| error.to_string())
}

/// Prompt block for the assistant, or an empty string when nothing is stored.
pub fn render_for_prompt(pool: &DbPool, user_id: Uuid, bot_id: Option<Uuid>) -> String {
    let rows = match load_for_user(pool, user_id, bot_id) {
        Ok(rows) => rows,
        Err(error) => {
            log::warn!("user memory could not be read for the prompt: {error}");
            return String::new();
        }
    };
    render_rows(rows)
}

/// Renders already-loaded rows, bounded by `MAX_PROMPT_CHARS`. Split out from
/// the database read so the prompt contract can be tested without a pool.
pub fn render_rows(rows: Vec<UserMemoryRow>) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut block = String::from(
        "\n\n---\nPersisted user memory (the user can edit it in Settings → Memory):\n",
    );
    for row in rows {
        let line = format!("- {}: {}\n", row.memory_key, row.memory_value);
        if block.len() + line.len() > MAX_PROMPT_CHARS {
            block.push_str("- (more entries available in Settings)\n");
            break;
        }
        block.push_str(&line);
    }
    block
}

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
    "about", "preference", "preferences", "memory", "please", "remember",
    "forget", "everything", "tudo", "sobre", "minha", "minhas", "meus",
    "preferencia", "preferências", "preferencias", "memoria", "memória",
    "favor", "with", "from", "that", "this", "the", "and",
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

/// Persists the fact for this user/bot. UPDATE-then-INSERT works with the
/// COALESCE expression index from migration 6.8.2 (a plain ON CONFLICT cannot
/// target it).
fn store_memory(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let updated = diesel::sql_query(
        "UPDATE chat_preferences SET memory_value = $3, updated_at = NOW() \
         WHERE user_id = $1 AND memory_key = $4 \
           AND ((bot_id IS NULL AND $2::uuid IS NULL) OR bot_id = $2::uuid)",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(value)
    .bind::<diesel::sql_types::Text, _>(key)
    .execute(&mut conn)
    .map_err(|error| error.to_string())?;
    if updated > 0 {
        return Ok(());
    }
    diesel::sql_query(
        "INSERT INTO chat_preferences (user_id, bot_id, memory_key, memory_value) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .bind::<diesel::sql_types::Text, _>(value)
    .execute(&mut conn)
    .map_err(|error| error.to_string())?;
    Ok(())
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

fn select_forget_keys(rows: &[UserMemoryRow], needle: &str) -> Vec<String> {
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

/// Deletes the stored memories that the request names. Returns the keys that
/// were removed so the caller can log a truthful outcome instead of guessing.
pub fn forget_memories(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    needle: &str,
) -> Result<Vec<String>, String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let mut query = chat_preferences::table
        .filter(chat_preferences::user_id.eq(user_id))
        .into_boxed();
    if let Some(bot_id) = bot_id {
        query = query.filter(
            chat_preferences::bot_id
                .eq(bot_id)
                .or(chat_preferences::bot_id.is_null()),
        );
    }
    let rows = query
        .select((
            chat_preferences::memory_key,
            chat_preferences::memory_value,
            chat_preferences::bot_id,
        ))
        .load::<UserMemoryRow>(&mut conn)
        .map_err(|error| error.to_string())?;
    let keys = select_forget_keys(&rows, needle);
    if keys.is_empty() {
        return Ok(keys);
    }
    let keys_for_delete = keys.iter().map(String::as_str).collect::<Vec<_>>();
    let deleted = match bot_id {
        Some(bot_id) => diesel::delete(chat_preferences::table)
            .filter(chat_preferences::user_id.eq(user_id))
            .filter(chat_preferences::memory_key.eq_any(keys_for_delete.clone()))
            .filter(
                chat_preferences::bot_id
                    .eq(bot_id)
                    .or(chat_preferences::bot_id.is_null()),
            )
            .execute(&mut conn),
        None => diesel::delete(chat_preferences::table)
            .filter(chat_preferences::user_id.eq(user_id))
            .filter(chat_preferences::memory_key.eq_any(keys_for_delete.clone()))
            .execute(&mut conn),
    }
    .map_err(|error| error.to_string())?;
    log::info!("memory: deleted {deleted} row(s) for the forget request");
    Ok(keys)
}

/// Writes a memory when the user turn explicitly asks for one. Returns the key
/// that was stored (for a log line), or None when the turn is not a request.
pub fn capture_from_message(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    text: &str,
) -> Option<String> {
    // A forget request must delete, not silently no-op: the assistant used to
    // answer "removed" while the row stayed in chat_preferences (#1708).
    if let Some(needle) = extract_forget_directive(text) {
        match forget_memories(pool, user_id, bot_id, &needle) {
            Ok(keys) if keys.is_empty() => {
                log::info!("memory: no stored entry matched forget request for user {user_id}");
            }
            Ok(keys) => {
                log::info!(
                    "memory: forgot {:?} for user {user_id}",
                    keys
                );
            }
            Err(error) => {
                log::warn!("memory: could not forget '{needle}': {error}");
            }
        }
        return None;
    }
    let fact = extract_remember_directive(text)?;
    let key = memory_key_for(&fact);
    match store_memory(pool, user_id, bot_id, &key, &fact) {
        Ok(()) => {
            log::info!("memory: stored '{key}' for user {user_id}");
            Some(key)
        }
        Err(error) => {
            log::warn!("memory: could not store '{key}': {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{render_rows, select_forget_keys, UserMemoryRow, MAX_PROMPT_CHARS};

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
            super::extract_forget_directive(
                "Forget my preference about writing in pt-BR and preferring tables."
            ),
            Some("my preference about writing in pt-BR and preferring tables".to_string())
        );
        assert_eq!(
            super::extract_forget_directive("esqueça que meu fuso é São Paulo"),
            Some("meu fuso é São Paulo".to_string())
        );
        assert!(super::extract_forget_directive("forget").is_none());
        assert!(super::extract_forget_directive("what do you remember?").is_none());
    }

    #[test]
    fn forget_matches_the_stored_row_by_distinctive_token() {
        let rows = vec![
            row("i-write-in-pt-br-and-prefer-tables", "I write in pt-BR and prefer tables"),
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
    fn an_empty_store_adds_nothing_to_the_prompt() {
        assert!(render_rows(Vec::new()).is_empty());
    }

    #[test]
    fn stored_preferences_are_rendered_as_readable_lines() {
        let block = render_rows(vec![row("language", "Portuguese"), row("timezone", "America/Sao_Paulo")]);
        assert!(block.contains("- language: Portuguese"));
        assert!(block.contains("- timezone: America/Sao_Paulo"));
        assert!(block.contains("Settings → Memory"));
    }

    #[test]
    fn the_block_is_bounded_and_says_when_it_truncated() {
        let big = "x".repeat(500);
        let rows = (0..20).map(|i| row(&format!("k{i}"), &big)).collect();
        let block = render_rows(rows);
        assert!(block.len() <= MAX_PROMPT_CHARS + 80, "block must stay near the cap");
        assert!(block.contains("(more entries available in Settings)"));
    }

    #[test]
    fn explicit_memory_requests_are_captured() {
        assert_eq!(
            super::extract_remember_directive("remember I prefer invoices as PDF"),
            Some("I prefer invoices as PDF".to_string())
        );
        assert_eq!(
            super::extract_remember_directive("Lembre-se que meu fuso é São Paulo."),
            Some("meu fuso é São Paulo".to_string())
        );
        // The colon form is the one people actually type; it used to be
        // ignored because the prefix required a space.
        assert_eq!(
            super::extract_remember_directive("Remember: I write in pt-BR and prefer tables."),
            Some("I write in pt-BR and prefer tables".to_string())
        );
        assert_eq!(
            super::extract_remember_directive("Lembre-se: prefiro tabelas."),
            Some("prefiro tabelas".to_string())
        );
    }

    #[test]
    fn questions_are_not_memories() {
        assert!(super::extract_remember_directive("do you remember my name?").is_none());
        assert!(super::extract_remember_directive("remember").is_none());
        assert!(super::extract_remember_directive("what is the weather?").is_none());
    }

    #[test]
    fn keys_are_stable_and_readable() {
        assert_eq!(
            super::memory_key_for("I prefer invoices as PDF"),
            "i-prefer-invoices-as-pdf"
        );
        assert_eq!(super::memory_key_for("   "), "note");
    }
}
