//! Agent-mode VM hook (#1167, #1580): always-on agent mode. Every
//! authenticated user gets a VM provisioned on first message. Conversation
//! switching stops previous jobs automatically.
use std::sync::{Arc, OnceLock};

static SERVICE: OnceLock<Arc<botagent::AgentService>> = OnceLock::new();

pub fn init(pool: botagent::state::DbPool) -> bool {
    init_with(Arc::new(botagent::AgentService::new(pool)))
}

pub fn init_with(service: Arc<botagent::AgentService>) -> bool {
    SERVICE.set(service).is_ok()
}

/// Handles the legacy `{ "type": "agent_mode" }` control frame for backward
/// compatibility. New clients no longer need to send this — agent mode is
/// always on.
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
    let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

    if !enabled {
        return Some("Agent mode is always active. Your workspace remains available.".to_string());
    }

    let conversation_id = session_id.to_string();
    match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation_id)).await {
        Ok(row) => Some(format!(
            "Agent workspace '{}' is {}. Conversation folder ready.",
            row.vm_name, row.status
        )),
        Err((code, err)) => {
            log::error!("agent_mode ensure_vm failed ({code}): {err}");
            Some("Could not start the agent workspace. Please try again shortly.".to_string())
        }
    }
}

/// Auto-provision the user's VM and switch to the current conversation
/// folder. Called from the pipeline on every message (#1580). Returns the
/// VM info string to inject into the system prompt, or None if the agent
/// subsystem is unavailable.
pub async fn ensure_agent_vm(
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
) -> Option<botagent::models::AgentSessionRow> {
    let service = SERVICE.get()?;
    let conversation_id = session_id.to_string();

    match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation_id)).await {
        Ok(row) => {
            if let Err(e) = botagent::vm::switch_conversation(service, &user_id, &conversation_id).await {
                log::warn!("conversation switch for {session_id}: {}", e.1);
            }
            Some(row)
        }
        Err((_, err)) => {
            log::warn!("auto-provision VM for user {user_id}: {err}");
            None
        }
    }
}

/// Execute code in the user's VM within the conversation folder. Returns
/// (success, stdout, stderr) or None if the service is absent.
pub async fn exec_in_user_vm(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
) -> Option<(bool, String, String)> {
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    let conversation_id = session_id.to_string();
    botagent::vm::exec_in_conversation(
        service, &row.vm_name, &conversation_id,
        language, code, 60_000,
    ).await.ok()
}
