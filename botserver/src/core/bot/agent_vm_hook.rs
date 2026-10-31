//! Agent-mode VM hook (#1167, #1580): always-on agent mode. Every
//! authenticated user gets a container provisioned on first message.
//! Switching conversations never terminates running jobs — work continues in
//! the background while the user moves between conversation folders.
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
                botlog::safe_warn!("conversation switch for {session_id}: {}", e.1);
            }
            Some(row)
        }
        Err((_, err)) => {
            log::warn!("auto-provision VM for user {user_id}: {err}");
            None
        }
    }
}

/// Delete the conversation folder when the user deletes the conversation
/// from history. Best-effort: a missing VM or service is not an error.
pub async fn delete_conversation_folder(user_id: uuid::Uuid, session_id: uuid::Uuid) {
    let Some(service) = SERVICE.get() else { return };
    if let Err((code, err)) = botagent::vm::delete_conversation_folder(
        service,
        &user_id,
        &session_id.to_string(),
    )
    .await
    {
        log::warn!("delete conversation folder for {session_id} failed ({code}): {err}");
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
