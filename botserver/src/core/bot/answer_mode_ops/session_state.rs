//! Per-session answer mode, cached in Redis when the `cache` feature is on.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::super::answer_mode::AnswerMode;

pub async fn get_answer_mode(state: &Arc<AppState>, session_id: &Uuid) -> AnswerMode {
    #[cfg(feature = "cache")]
    if let Some(ref cache) = state.cache {
        let key = format!("answer_mode:{session_id}");
        let mut conn = match cache.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                botlog::safe_debug!("answer_mode: cache conn failed for {session_id}: {e}");
                return AnswerMode::Default;
            }
        };
        let result: Result<String, _> = redis::cmd("GET").arg(&key).query_async(&mut conn).await;
        return match result {
            Ok(mode_str) => {
                let parsed = mode_str
                    .parse::<AnswerMode>()
                    .unwrap_or(AnswerMode::Default);
                botlog::safe_debug!(
                    "answer_mode: session {session_id} -> mode '{mode_str}' parsed={:?}",
                    parsed
                );
                parsed
            }
            Err(e) => {
                botlog::safe_debug!("answer_mode: cache GET failed for {session_id}: {e}");
                AnswerMode::Default
            }
        };
    }
    #[cfg(not(feature = "cache"))]
    botlog::safe_debug!("answer_mode: cache feature disabled, returning Default for {session_id}");
    AnswerMode::Default
}

pub async fn store_answer_mode(
    state: &Arc<AppState>,
    session_id: &Uuid,
    mode: &AnswerMode,
) -> Result<(), String> {
    #[cfg(feature = "cache")]
    if let Some(ref cache) = state.cache {
        let key = format!("answer_mode:{session_id}");
        let mut conn = cache
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis error: {e}"))?;
        redis::cmd("SET")
            .arg(&key)
            .arg(mode.as_str())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| format!("Redis SET error: {e}"))?;
        return Ok(());
    }
    log::warn!("SET ANSWER MODE: cache not available, mode change not persisted");
    Ok(())
}
