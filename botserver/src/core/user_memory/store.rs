use uuid::Uuid;

use botcore::shared::utils::DbPool;
use botsettings::memory_store;

/// Entries the prompt may carry. Keeps the injected block small and bounded.
const MAX_PROMPT_ENTRIES: i64 = 24;
pub(super) const MAX_PROMPT_CHARS: usize = 2000;

#[derive(Debug, Clone)]
pub struct UserMemoryRow {
    pub memory_key: String,
    pub memory_value: String,
}

/// Memories that apply to this bot (its own entries plus the global ones),
/// opened with the person's key, newest first.
pub fn load_for_user(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
) -> Result<Vec<UserMemoryRow>, String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    memory_store::list(&mut conn, user_id, bot_id, MAX_PROMPT_ENTRIES)
        .map(|entries| {
            entries
                .into_iter()
                .map(|entry| UserMemoryRow {
                    memory_key: entry.key,
                    memory_value: entry.value,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

/// Whether memory is on for this person; on when the setting cannot be read.
pub fn is_enabled(pool: &DbPool, user_id: Uuid) -> bool {
    match pool.get() {
        Ok(mut conn) => memory_store::is_enabled(&mut conn, user_id),
        Err(error) => {
            log::warn!("memory switch could not be read: {error}");
            true
        }
    }
}

/// The stored entries as prompt lines, or an empty string when nothing is
/// stored or memory is switched off.
pub fn render_for_prompt(pool: &DbPool, user_id: Uuid, bot_id: Option<Uuid>) -> String {
    if !is_enabled(pool, user_id) {
        return String::new();
    }
    match load_for_user(pool, user_id, bot_id) {
        Ok(rows) => render_rows(rows),
        Err(error) => {
            log::warn!("user memory could not be read for the prompt: {error}");
            String::new()
        }
    }
}

/// Renders already-loaded rows, bounded by `MAX_PROMPT_CHARS`. Split out from
/// the database read so the prompt contract can be tested without a pool.
/// Keys the assistant made carry no meaning, so only the fact is shown.
pub fn render_rows(rows: Vec<UserMemoryRow>) -> String {
    let mut block = String::new();
    for row in rows {
        let line = if memory_store::is_entry_key(&row.memory_key) {
            format!("- {}\n", row.memory_value)
        } else {
            format!("- {}: {}\n", row.memory_key, row.memory_value)
        };
        if block.len() + line.len() > MAX_PROMPT_CHARS {
            block.push_str("- (more entries available in Settings › Memory)\n");
            break;
        }
        block.push_str(&line);
    }
    block
}

/// Persists the fact for this person, sealed, under its opaque key.
pub(super) fn store_memory(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    fact: &str,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let key = memory_store::fact_key(user_id, fact);
    memory_store::save(&mut conn, user_id, bot_id, &key, fact)
}

#[cfg(test)]
mod tests {
    use super::{render_rows, UserMemoryRow, MAX_PROMPT_CHARS};

    fn row(key: &str, value: &str) -> UserMemoryRow {
        UserMemoryRow {
            memory_key: key.to_string(),
            memory_value: value.to_string(),
        }
    }

    #[test]
    fn an_empty_store_adds_nothing_to_the_prompt() {
        assert!(render_rows(Vec::new()).is_empty());
    }

    #[test]
    fn stored_preferences_are_rendered_as_readable_lines() {
        let block = render_rows(vec![
            row("language", "Portuguese"),
            row("m-0123456789abcdef01234567", "Trabalho na Secretaria de Saúde"),
        ]);
        assert!(block.contains("- language: Portuguese"));
        assert!(block.contains("- Trabalho na Secretaria de Saúde\n"));
        assert!(!block.contains("m-0123"));
    }

    #[test]
    fn the_block_is_bounded_and_says_when_it_truncated() {
        let big = "x".repeat(500);
        let rows = (0..20).map(|i| row(&format!("k{i}"), &big)).collect();
        let block = render_rows(rows);
        assert!(
            block.len() <= MAX_PROMPT_CHARS + 80,
            "block must stay near the cap"
        );
        assert!(block.contains("(more entries available in Settings › Memory)"));
    }
}
