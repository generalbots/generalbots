use diesel::prelude::*;
use uuid::Uuid;

use botcore::shared::utils::DbPool;
use botschema::chat_preferences;

use super::directives::{
    extract_forget_directive, extract_remember_directive, memory_key_for, select_forget_keys,
};
use super::store::{store_memory, UserMemoryRow};

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
                log::info!("memory: forgot {} entries for user {user_id}", keys.len());
            }
            Err(error) => {
                log::warn!("memory: could not forget entry: {error}");
            }
        }
        return None;
    }
    let fact = extract_remember_directive(text)?;
    let key = memory_key_for(&fact);
    match store_memory(pool, user_id, bot_id, &key, &fact) {
        Ok(()) => {
            log::info!("memory: stored an entry for user {user_id}");
            Some(key)
        }
        Err(error) => {
            log::warn!("memory: could not store entry: {error}");
            None
        }
    }
}
