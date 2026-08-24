//! Agent-mode WS hook (issue #1167): handles the `agent_mode` control frame
//! emitted by the chat switcher. No-ops when the `agent-vm` feature is off,
//! leaving the frame unconsumed for legacy pipelines.
use std::sync::{Arc, OnceLock};

static SERVICE: OnceLock<Arc<botagent::AgentService>> = OnceLock::new();

pub fn init(pool: botagent::state::DbPool) -> bool {
    init_with(Arc::new(botagent::AgentService::new(pool)))
}

/// Registers an already-built service so the router state and this hook
/// share one instance.
pub fn init_with(service: Arc<botagent::AgentService>) -> bool {
    SERVICE.set(service).is_ok()
}

/// Handles `{ "type": "agent_mode", "enabled": bool, "session_id": "..." }`.
/// Returns the human-facing confirmation to send back, or `None` when the
/// frame is not an agent_mode request or the service is absent.
pub async fn handle_frame(
    parsed: &serde_json::Value,
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
) -> Option<String> {
    if parsed.get("type").and_then(|v| v.as_str()) != Some("agent_mode") {
        return None;
    }
    let Some(service) = SERVICE.get() else { return None };
    let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let raw_session = parsed
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| session_id.to_string());

    if !enabled {
        return Some("Agent mode disabled. This conversation returned to chat mode.".to_string());
    }

    match botagent::vm::ensure_vm(service, &raw_session, &user_id, &bot_uuid).await {
        Ok(row) => Some(format!(
            "Agent workspace '{}' is {}. Snapshots are available from the header menu.",
            row.vm_name, row.status
        )),
        Err((code, err)) => {
            log::error!("agent_mode ensure_vm failed ({code}): {err}");
            Some("Could not start the agent workspace. Please try again shortly.".to_string())
        }
    }
}

