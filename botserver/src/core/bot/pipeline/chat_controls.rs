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
    use super::turn_watch::{bounded, REDIS_STEP_LIMIT};
    let cache = state.cache.as_ref()?;
    let raw = bounded("agent switch read", REDIS_STEP_LIMIT, async {
        let mut conn = cache.get_multiplexed_async_connection().await.ok()?;
        let raw: Option<String> = redis::cmd("GET")
            .arg(controls_key(user_id, session_id))
            .query_async(&mut conn)
            .await
            .ok()?;
        raw
    })
    .await
    .flatten()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
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
    use super::turn_watch::{bounded, REDIS_STEP_LIMIT};
    let Some(cache) = state.cache.as_ref() else {
        return;
    };
    bounded("agent switch write", REDIS_STEP_LIMIT, async {
        let Ok(mut conn) = cache.get_multiplexed_async_connection().await else {
            return;
        };
        let payload = serde_json::json!({ "agent_enabled": agent_enabled }).to_string();
        let _: Result<(), redis::RedisError> = redis::cmd("SET")
            .arg(controls_key(user_id, session_id))
            .arg(payload)
            .query_async(&mut conn)
            .await;
    })
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

/// The agent VM is an Incus container. On Windows that backend exists only when
/// an explicit WSL or remote wrapper is configured (`AGENT_INCUS_BIN`), and the
/// chat client sends the Agent switch as on by default, so without this check
/// every turn would try the VM, fail and log. The probe walks PATH, hence the
/// once-per-process cache.
#[cfg(all(windows, feature = "agent-vm"))]
fn agent_backend_available() -> bool {
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| botagent::agent_backend_unavailable().is_none())
}

/// Resolve the effective agent/VM toggle for this turn and persist explicit
/// payload values so the session is the single source of truth.
///
/// When the `agent-vm` feature is not compiled in, the agent is forced off
/// regardless of what the client sends. On Windows with `agent-vm` compiled
/// in, the backend availability check forces off when Incus is unreachable
/// (no WSL2 or no AGENT_INCUS_BIN wrapper).
pub async fn resolve_agent_enabled(
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    payload: Option<bool>,
) -> bool {
    // No agent-vm feature → no workspace, force off.
    if !cfg!(feature = "agent-vm") {
        return false;
    }

    #[cfg(all(windows, feature = "agent-vm"))]
    if !agent_backend_available() {
        return false;
    }

    if let Some(enabled) = payload {
        write_agent_enabled(state, user_id, session_id, enabled).await;
        return enabled;
    }
    read_agent_enabled(state, user_id, session_id)
        .await
        .unwrap_or(DEFAULT_AGENT_ENABLED)
}

/// Does the model-preference string name a subscription provider?
///
/// Pure parsing check with no platform dependency — used by the tests and
/// by [`subscription_implies_agent`].
#[cfg(feature = "agent-vm")]
fn is_subscription_choice(model_preference: &str) -> bool {
    use crate::core::ai_accounts::models::parse_choice;
    parse_choice(model_preference).is_some()
}

/// When the turn's `model_preference` explicitly names a subscription
/// provider (`sub:claude-code`, …), the agent is implicitly ON for this
/// turn even when the per-conversation switch is off — the person
/// deliberately chose it in the model menu.
///
/// Returns `true` only when both conditions hold:
/// 1. The preference is a subscription choice.
/// 2. The backend is available on this platform.
#[cfg(feature = "agent-vm")]
pub fn subscription_implies_agent(model_preference: &str) -> bool {
    if !is_subscription_choice(model_preference) {
        return false;
    }
    #[cfg(windows)]
    if !agent_backend_available() {
        return false;
    }
    true
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

    #[cfg(feature = "agent-vm")]
    mod subscription_override {
        use super::super::is_subscription_choice;

        #[test]
        fn explicit_sub_choice_is_recognised() {
            assert!(is_subscription_choice("sub:claude-code"));
            assert!(is_subscription_choice("sub:codex-cli/gpt-6.2-sol"));
            assert!(is_subscription_choice("sub:antigravity"));
        }

        #[test]
        fn non_subscription_preference_is_not_a_choice() {
            assert!(!is_subscription_choice("auto"));
            assert!(!is_subscription_choice("fast"));
            assert!(!is_subscription_choice("acct:openai/gpt-4o"));
            assert!(!is_subscription_choice(""));
        }
    }
}
