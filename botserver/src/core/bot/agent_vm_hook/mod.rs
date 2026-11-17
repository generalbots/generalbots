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

/// Shared agent service, when agent mode has been initialised.
pub fn service() -> Option<&'static Arc<botagent::AgentService>> {
    SERVICE.get()
}

/// Central gate for the chat Agent/VM switch (#2239/#2264).
///
/// The pipeline records the switch per session for the current turn, so every
/// workspace entry point checks it here: no route — native tool, api catalog
/// command, Vibe run or bot script — can provision or drive a VM while the
/// switch is off. Paths outside a chat turn keep their previous behaviour
/// (the turn flag defaults to enabled).
fn workspace_disabled(session_id: uuid::Uuid) -> bool {
    !crate::core::bot::pipeline::agent_turn::is_enabled(session_id)
}

fn workspace_disabled_error() -> (u16, String) {
    (
        403,
        crate::core::bot::pipeline::agent_turn::DISABLED_MESSAGE.to_string(),
    )
}

/// Runs a shell script inside the user's conversation folder (#1613).
///
/// Used by conversation autotasks to install and inspect cron/systemd entries
/// that must keep running after the chat turn ends. Returns `None` when agent
/// mode is unavailable; the caller then records the job as not mirrored
/// instead of pretending it is scheduled.
pub async fn run_shell_in_conversation(
    user_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    conversation_id: uuid::Uuid,
    code: &str,
    timeout_ms: u64,
) -> Option<(bool, String, String)> {
    if workspace_disabled(conversation_id) {
        log::warn!(
            "autotask: workspace run refused for conversation {conversation_id}: Agent/VM is off"
        );
        return None;
    }
    let service = SERVICE.get()?;
    let conversation = conversation_id.to_string();
    let row = match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation)).await
    {
        Ok(row) => row,
        Err((status, error)) => {
            log::warn!("autotask: could not ensure the workspace ({status}): {error}");
            return None;
        }
    };
    match botagent::vm::exec_in_conversation(
        service,
        &row.vm_name,
        &conversation,
        "shell",
        code,
        timeout_ms,
        &[],
    )
    .await
    {
        Ok(outcome) => Some(outcome),
        Err((status, error)) => {
            log::warn!("autotask: workspace command failed ({status}): {error}");
            None
        }
    }
}

/// Installer variant: runs as root in the conversation's VM. Used by the
/// conversation-autotask installer, which needs `/etc/systemd/system` or a
/// crontab; generated programs keep running as `runner`.
pub async fn run_shell_in_conversation_as_root(
    user_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    conversation_id: uuid::Uuid,
    code: &str,
    timeout_ms: u64,
) -> Option<(bool, String, String)> {
    if workspace_disabled(conversation_id) {
        log::warn!(
            "autotask: root workspace run refused for conversation {conversation_id}: Agent/VM is off"
        );
        return None;
    }
    let service = SERVICE.get()?;
    let conversation = conversation_id.to_string();
    let row = match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation)).await
    {
        Ok(row) => row,
        Err((status, error)) => {
            log::warn!("autotask: could not ensure the workspace ({status}): {error}");
            return None;
        }
    };
    match botagent::vm::run_root_shell(service, &row.vm_name, code, timeout_ms).await {
        Ok(outcome) => Some(outcome),
        Err((status, error)) => {
            log::warn!("autotask: root install failed ({status}): {error}");
            None
        }
    }
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
    let Some(service) = SERVICE.get() else {
        return None;
    };
    let enabled = parsed
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

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
    // Unattributed sessions (guest/anonymous web chat) must never provision a
    // container — `Uuid::nil()` used to create `ag-u00000000-*` VMs on every
    // anonymous message and filled the production host disk (#1596).
    if user_id.is_nil() {
        return None;
    }
    // #2239 — the chat Agent/VM switch is off for this turn: never provision.
    if workspace_disabled(session_id) {
        botlog::safe_info!(
            "workspace provisioning skipped for session {session_id}: Agent/VM is off"
        );
        return None;
    }
    let service = SERVICE.get()?;
    let conversation_id = session_id.to_string();

    match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation_id)).await {
        Ok(row) => {
            if let Err(e) =
                botagent::vm::switch_conversation(service, &user_id, &conversation_id).await
            {
                botlog::safe_warn!("conversation switch for {session_id}: {}", e.1);
            }
            Some(row)
        }
        Err((_, err)) => {
            botlog::safe_warn!("auto-provision VM for user {user_id}: {err}");
            None
        }
    }
}

/// Delete the conversation folder when the user deletes the conversation
/// from history. Best-effort: a missing VM or service is not an error.
pub async fn delete_conversation_folder(user_id: uuid::Uuid, session_id: uuid::Uuid) {
    let Some(service) = SERVICE.get() else { return };
    if let Err((code, err)) =
        botagent::vm::delete_conversation_folder(service, &user_id, &session_id.to_string()).await
    {
        botlog::safe_warn!("delete conversation folder for {session_id} failed ({code}): {err}");
    }
}

mod ops;
mod wake;

pub use ops::WorkspaceError;
pub(crate) use ops::{
    ensure_agent_vm_result, exec_in_user_vm, exec_in_user_vm_result, exec_in_user_vm_with_timeout,
    list_session_folder, list_session_folder_meta, list_user_vm_output,
};
#[cfg(feature = "agent-vm")]
pub use ops::{read_session_file, user_vm_name, write_session_file};
pub(crate) use wake::{exec_with_wake_events, exec_with_wake_result};
