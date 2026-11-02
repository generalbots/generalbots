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
}
