use diesel::prelude::*;
use uuid::Uuid;

use botcore::shared::utils::DbPool;
use botschema::chat_preferences;

use super::directives::{
    extract_forget_directive, extract_remember_directive, memory_key_for, select_forget_keys,
};
use super::store::{store_memory, UserMemoryRow};

/// Result of applying a forget request. `deleted` is what the database actually
/// removed; it can be lower than `keys.len()` when rows change concurrently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ForgetOutcome {
    pub keys: Vec<String>,
    pub deleted: usize,
}

impl ForgetOutcome {
    fn reply(&self) -> String {
        forget_acknowledgement(self.deleted)
    }
}

/// Assistant-facing text for a forget request. Zero removed rows is a no-op,
/// so it must never read like a success (#1790).
pub(super) fn forget_acknowledgement(deleted: usize) -> String {
    match deleted {
        0 => "No stored memory matched that request.".to_string(),
        1 => "Forgot 1 stored memory.".to_string(),
        count => format!("Forgot {count} stored memories."),
    }
}

/// Deletes the stored memories that the request names. Returns the keys that
/// the request matched so the caller can log a truthful outcome instead of
/// guessing; `forget_memories_with_outcome` also carries the actual delete
/// count for the chat acknowledgement.
pub fn forget_memories(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    needle: &str,
) -> Result<Vec<String>, String> {
    forget_memories_with_outcome(pool, user_id, bot_id, needle).map(|outcome| outcome.keys)
}

fn forget_memories_with_outcome(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    needle: &str,
) -> Result<ForgetOutcome, String> {
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
        return Ok(ForgetOutcome { keys, deleted: 0 });
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
    botlog::safe_info!("memory: deleted {deleted} row(s) for the forget request");
    Ok(ForgetOutcome { keys, deleted })
}

/// Writes a memory when the user turn explicitly asks for one. Returns the key
/// that was stored, or the truthful assistant-facing acknowledgement for a
/// forget request; None when the turn is not a request or the write failed.
pub fn capture_from_message(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    text: &str,
) -> Option<String> {
    // A forget request must delete, not silently no-op: the assistant used to
    // answer "removed" while the row stayed in chat_preferences (#1708). The
    // reply now reports a zero-row outcome instead of claiming success (#1790).
    if let Some(needle) = extract_forget_directive(text) {
        return match forget_memories_with_outcome(pool, user_id, bot_id, &needle) {
            Ok(outcome) if outcome.deleted == 0 => {
                botlog::safe_info!(
                    "memory: no stored entry matched forget request for user {user_id}"
                );
                Some(outcome.reply())
            }
            Ok(outcome) => {
                botlog::safe_info!(
                    "memory: forgot {} entries for user {user_id}",
                    outcome.deleted
                );
                Some(outcome.reply())
            }
            Err(error) => {
                botlog::safe_warn!("memory: could not forget entry: {error}");
                None
            }
        };
    }
    let fact = extract_remember_directive(text)?;
    let key = memory_key_for(&fact);
    match store_memory(pool, user_id, bot_id, &key, &fact) {
        Ok(()) => {
            botlog::safe_info!("memory: stored an entry for user {user_id}");
            Some(key)
        }
        Err(error) => {
            botlog::safe_warn!("memory: could not store entry: {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{forget_acknowledgement, ForgetOutcome};

    #[test]
    fn a_no_match_forget_is_not_reported_as_success() {
        let reply = forget_acknowledgement(0);
        assert_eq!(reply, "No stored memory matched that request.");
        let lower = reply.to_lowercase();
        for false_claim in ["forgot", "deleted", "removed", "success"] {
            assert!(!lower.contains(false_claim), "reply claimed {false_claim}");
        }
    }

    #[test]
    fn a_matched_row_that_was_not_deleted_is_not_reported_as_success() {
        let outcome = ForgetOutcome {
            keys: vec!["i-write-in-pt-br-and-prefer-tables".to_string()],
            deleted: 0,
        };
        assert_eq!(outcome.reply(), "No stored memory matched that request.");
    }

    #[test]
    fn a_successful_delete_reports_the_actual_count() {
        assert_eq!(forget_acknowledgement(1), "Forgot 1 stored memory.");
        assert_eq!(forget_acknowledgement(2), "Forgot 2 stored memories.");
    }
}
