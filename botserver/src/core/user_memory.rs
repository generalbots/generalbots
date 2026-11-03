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

/// Imperative phrases that explicitly ask the assistant to remember a fact.
/// Kept prefix-only: "do you remember…" is a question, not a write.
const REMEMBER_PREFIXES: [&str; 12] = [
    "please remember that ",
    "please remember ",
    "remember that ",
    "remember ",
    "lembre-se que ",
    "lembre-se de que ",
    "lembre-se ",
    "lembre que ",
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

/// Writes a memory when the user turn explicitly asks for one. Returns the key
/// that was stored (for a log line), or None when the turn is not a request.
pub fn capture_from_message(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    text: &str,
) -> Option<String> {
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
    use super::{render_rows, UserMemoryRow, MAX_PROMPT_CHARS};

    fn row(key: &str, value: &str) -> UserMemoryRow {
        UserMemoryRow {
            memory_key: key.to_string(),
            memory_value: value.to_string(),
            bot_id: None,
        }
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
