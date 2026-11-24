use uuid::Uuid;

use botcore::shared::utils::DbPool;
use botsettings::memory_store;

use super::directives::select_forget_keys;
use super::store::UserMemoryRow;

/// Rows a forget request is matched against at most.
const FORGET_SCAN_LIMIT: i64 = 500;

/// i18n key of the reply to a forget request. Zero removed rows is a no-op,
/// so it must never read like a success (#1790).
pub(super) fn forget_acknowledgement(deleted: usize) -> &'static str {
    match deleted {
        0 => "memory-forgot-none",
        1 => "memory-forgot-one",
        _ => "memory-forgot-many",
    }
}

fn rows_to_match(pool: &DbPool, user_id: Uuid, bot_id: Option<Uuid>) -> Result<Vec<UserMemoryRow>, String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    memory_store::list(&mut conn, user_id, bot_id, FORGET_SCAN_LIMIT)
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

/// Deletes the stored memories that the request names; returns how many the
/// database removed.
pub(super) fn forget_memories_with_outcome(
    pool: &DbPool,
    user_id: Uuid,
    bot_id: Option<Uuid>,
    needle: &str,
) -> Result<usize, String> {
    let rows = rows_to_match(pool, user_id, bot_id)?;
    let keys = select_forget_keys(&rows, needle);
    if keys.is_empty() {
        return Ok(0);
    }
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let deleted = memory_store::delete_keys(&mut conn, user_id, bot_id, &keys)
        .map_err(|error| error.to_string())?;
    botlog::safe_info!("memory: deleted {deleted} row(s) for the forget request");
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::forget_acknowledgement;

    #[test]
    fn a_no_match_forget_is_not_reported_as_success() {
        assert_eq!(forget_acknowledgement(0), "memory-forgot-none");
    }

    #[test]
    fn a_successful_delete_reports_the_actual_count() {
        assert_eq!(forget_acknowledgement(1), "memory-forgot-one");
        assert_eq!(forget_acknowledgement(2), "memory-forgot-many");
    }
}
