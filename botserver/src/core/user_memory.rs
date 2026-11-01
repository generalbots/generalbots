//! Persisted user memory (GB6-16).
//!
//! Preferences edited in Settings → Memory live in `user_preferences`; this module
//! is the server-side reader that puts them into the conversation prompt, so a
//! new session starts with what the user already taught the assistant instead
//! of repeating it.

use diesel::prelude::*;
use uuid::Uuid;

use botcore::shared::utils::DbPool;
use botschema::user_preferences;

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
    let mut query = user_preferences::table
        .filter(user_preferences::user_id.eq(user_id))
        .into_boxed();
    if let Some(bot_id) = bot_id {
        query = query.filter(
            user_preferences::bot_id
                .eq(bot_id)
                .or(user_preferences::bot_id.is_null()),
        );
    }
    query
        .order(user_preferences::bot_id.asc())
        .then_order_by(user_preferences::updated_at.desc())
        .limit(MAX_PROMPT_ENTRIES)
        .select((
            user_preferences::memory_key,
            user_preferences::memory_value,
            user_preferences::bot_id,
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
