//! Session-scoped chat control state (#2239).
//!
//! `agent_enabled` (and other composer toggles) must have a single source of
//! truth for the conversation: when a turn carries an explicit value we persist
//! it, and when it does not we read the stored session value. This keeps the
//! Agent/VM toggle sticky across turns and across clients that omit the flag.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

const DEFAULT_AGENT_ENABLED: bool = true;

fn controls_key(user_id: Uuid, session_id: Uuid) -> String {
    format!("chat_controls:{user_id}:{session_id}")
}

#[cfg(feature = "cache")]
async fn read_agent_enabled(
    state: &Arc<AppState>,
    user_id: Uuid,
    session_id: Uuid,
) -> Option<bool> {
    let cache = state.cache.as_ref()?;
    let mut conn = cache.get_multiplexed_async_connection().await.ok()?;
    let raw: Option<String> = redis::cmd("GET")
        .arg(controls_key(user_id, session_id))
        .query_async(&mut conn)
        .await
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(raw.as_deref()?).ok()?;
    value.get("agent_enabled").and_then(|v| v.as_bool())
}

#[cfg(not(feature = "cache"))]
async fn read_agent_enabled(
    _state: &Arc<AppState>,
    _user_id: Uuid,
    _session_id: Uuid,
) -> Option<bool> {
    None
}

#[cfg(feature = "cache")]
async fn write_agent_enabled(
    state: &Arc<AppState>,
    user_id: Uuid,
    session_id: Uuid,
    agent_enabled: bool,
) {
    let Some(cache) = state.cache.as_ref() else {
        return;
    };
    let Ok(mut conn) = cache.get_multiplexed_async_connection().await else {
        return;
    };
    let payload = serde_json::json!({ "agent_enabled": agent_enabled }).to_string();
    let _: Result<(), redis::RedisError> = redis::cmd("SET")
        .arg(controls_key(user_id, session_id))
        .arg(payload)
        .query_async(&mut conn)
        .await;
}

#[cfg(not(feature = "cache"))]
async fn write_agent_enabled(
    _state: &Arc<AppState>,
    _user_id: Uuid,
    _session_id: Uuid,
    _agent_enabled: bool,
) {
}

/// Resolve the effective agent/VM toggle for this turn and persist explicit
/// payload values so the session is the single source of truth.
pub async fn resolve_agent_enabled(
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    payload: Option<bool>,
) -> bool {
    if let Some(enabled) = payload {
        write_agent_enabled(state, user_id, session_id, enabled).await;
        return enabled;
    }
    read_agent_enabled(state, user_id, session_id)
        .await
        .unwrap_or(DEFAULT_AGENT_ENABLED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_key_is_session_scoped() {
        let user = Uuid::nil();
        let session = Uuid::nil();
        assert_eq!(controls_key(user, session), "chat_controls:00000000-0000-0000-0000-000000000000:00000000-0000-0000-0000-000000000000");
    }
}
