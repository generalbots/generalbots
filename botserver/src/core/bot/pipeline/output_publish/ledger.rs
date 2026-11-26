//! Durable mirror of the output-publish ledger (issue #2541).
//!
//! The in-memory ledger in [`super`] stays the fast path and the fallback when
//! the cache is unavailable. Committed entries are mirrored into Valkey/Redis
//! under `output_publish:{session_id}` with a bounded TTL, so a botserver
//! restart does not republish files. A cache outage degrades to the in-memory
//! ledger: the next turn re-uploads once, and Drive's deterministic key makes
//! that an overwrite, never a duplicate. No background task is started here.

use std::collections::HashMap;
use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::{snapshot, store, PublishedFile};

/// Default lifetime of a conversation's publish ledger, in days.
pub const DEFAULT_LEDGER_TTL_DAYS: u64 = 30;

/// Markdown link to an output file stored in the conversation's VM folder.
pub fn output_link(name: &str, conversation: Uuid) -> String {
    format!("[{name}](vmfile:{conversation}/output/{name})")
}

#[cfg(feature = "cache")]
fn redis_key(session_id: Uuid) -> String {
    format!("output_publish:{session_id}")
}

#[cfg(feature = "cache")]
fn ttl_secs() -> u64 {
    let days = std::env::var("GB_PUBLISH_LEDGER_TTL_DAYS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_LEDGER_TTL_DAYS);
    days.saturating_mul(24 * 60 * 60).max(60)
}

/// Loads the ledger for a conversation. Redis is the durable source when the
/// cache feature is enabled; the in-memory map is the fallback for a cache
/// outage, a corrupt payload or a build without the cache feature.
pub async fn load_durable(
    state: &Arc<AppState>,
    session_id: Uuid,
) -> HashMap<String, PublishedFile> {
    botlog::safe_debug!(
        "output publish: ledger lookup for {session_id} in {}",
        state.bucket_name
    );
    #[cfg(feature = "cache")]
    if let Some(ref cache) = state.cache {
        let mut conn = match cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(error) => {
                botlog::safe_debug!("output publish: cache conn failed for {session_id}: {error}");
                return snapshot(session_id);
            }
        };
        let raw: String = match redis::cmd("GET")
            .arg(redis_key(session_id))
            .query_async(&mut conn)
            .await
        {
            Ok(raw) => raw,
            Err(error) => {
                botlog::safe_debug!("output publish: ledger GET failed for {session_id}: {error}");
                return snapshot(session_id);
            }
        };
        return match serde_json::from_str::<HashMap<String, PublishedFile>>(&raw) {
            Ok(map) => map,
            Err(error) => {
                botlog::safe_warn!("output publish: corrupt ledger for {session_id}: {error}");
                snapshot(session_id)
            }
        };
    }
    snapshot(session_id)
}

/// Mirrors the committed ledger into Redis and the in-memory fallback.
/// Failures are logged, never fatal: the turn already answered the user.
pub async fn save_durable(
    state: &Arc<AppState>,
    session_id: Uuid,
    entries: &HashMap<String, PublishedFile>,
) {
    store(session_id, entries.clone());
    botlog::safe_debug!(
        "output publish: persisting ledger for {session_id} in {}",
        state.bucket_name
    );
    #[cfg(feature = "cache")]
    if let Some(ref cache) = state.cache {
        let payload = match serde_json::to_string(entries) {
            Ok(payload) => payload,
            Err(error) => {
                botlog::safe_warn!("output publish: ledger serialize failed: {error}");
                return;
            }
        };
        let mut conn = match cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(error) => {
                botlog::safe_warn!("output publish: cache conn failed for {session_id}: {error}");
                return;
            }
        };
        let result: Result<(), _> = redis::cmd("SET")
            .arg(redis_key(session_id))
            .arg(payload)
            .arg("EX")
            .arg(ttl_secs())
            .query_async(&mut conn)
            .await;
        if let Err(error) = result {
            botlog::safe_warn!("output publish: ledger SET failed for {session_id}: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "cache")]
    #[test]
    fn redis_key_is_session_scoped() {
        let id = Uuid::nil();
        assert_eq!(redis_key(id), format!("output_publish:{id}"));
    }

    #[cfg(feature = "cache")]
    #[test]
    fn ttl_has_a_floor() {
        std::env::remove_var("GB_PUBLISH_LEDGER_TTL_DAYS");
        assert!(ttl_secs() >= 60);
    }

    #[test]
    fn default_ttl_is_a_month() {
        assert_eq!(DEFAULT_LEDGER_TTL_DAYS, 30);
    }
}
