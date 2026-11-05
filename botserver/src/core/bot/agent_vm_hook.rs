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
    let service = SERVICE.get()?;
    let conversation = conversation_id.to_string();
    let row = match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation)).await {
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
    let service = SERVICE.get()?;
    let conversation = conversation_id.to_string();
    let row = match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation)).await {
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
    // Unattributed sessions (guest/anonymous web chat) must never provision a
    // container — `Uuid::nil()` used to create `ag-u00000000-*` VMs on every
    // anonymous message and filled the production host disk (#1596).
    if user_id.is_nil() {
        return None;
    }
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
            botlog::safe_warn!("auto-provision VM for user {user_id}: {err}");
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
        botlog::safe_warn!("delete conversation folder for {session_id} failed ({code}): {err}");
    }
}

/// Concrete workspace failure: HTTP-ish status plus the reason text produced
/// by the VM subsystem. Threaded into chat so the user sees the real cause
/// (for example a sandbox host that cannot start containers) instead of a
/// dead-end "workspace is not available" phrase.
pub type WorkspaceError = (u16, String);

/// Execute code in the user's VM within the conversation folder. Returns
/// (success, stdout, stderr) or None if the service is absent.
pub async fn exec_in_user_vm(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
) -> Option<(bool, String, String)> {
    exec_in_user_vm_with_timeout(user_id, session_id, language, code, 60_000, &[]).await
}

/// Same as [`exec_in_user_vm`] with an explicit timeout (bounded by the
/// native tool to 120s) and optional third-party packages to install first.
pub async fn exec_in_user_vm_with_timeout(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Option<(bool, String, String)> {
    exec_in_user_vm_result(user_id, session_id, language, code, timeout_ms, packages)
        .await
        .ok()
        .flatten()
}

/// Detailed sibling of [`exec_in_user_vm_with_timeout`]. `Ok(None)` still
/// means "wake the workspace and retry"; `Err` carries the concrete failure
/// so callers can show the true reason.
pub async fn exec_in_user_vm_result(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Result<Option<(bool, String, String)>, WorkspaceError> {
    let Some(service) = SERVICE.get() else {
        return Ok(None);
    };
    let row = match botagent::vm::get_user_vm(service, &user_id) {
        Ok(Some(row)) => row,
        Ok(None) => return Ok(None),
        Err((_, error)) => {
            botlog::safe_warn!("workspace lookup for {user_id}: {error}");
            return Ok(None);
        }
    };
    if row.status != botagent::vm::STATUS_RUNNING {
        return Ok(None);
    }
    let conversation_id = session_id.to_string();
    match botagent::vm::exec_in_conversation(
        service,
        &row.vm_name,
        &conversation_id,
        language,
        code,
        timeout_ms,
        packages,
    )
    .await
    {
        Ok(outcome) => Ok(Some(outcome)),
        Err((status, error)) => Err((status.as_u16(), error)),
    }
}

/// Auto-provision with the failure reason threaded out;
/// [`ensure_agent_vm`] keeps the None form for callers that only mirror a
/// job. Guest sessions (nil user) never provision — the concrete error keeps
/// that truthful.
pub async fn ensure_agent_vm_result(
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
) -> Result<botagent::models::AgentSessionRow, WorkspaceError> {
    if user_id.is_nil() {
        return Err((
            403,
            "guest sessions have no workspace; sign in to use yours".to_string(),
        ));
    }
    let Some(service) = SERVICE.get() else {
        return Err((
            503,
            "the agent workspace subsystem is unavailable".to_string(),
        ));
    };
    let conversation_id = session_id.to_string();
    match botagent::vm::ensure_vm(service, &user_id, &bot_uuid, Some(&conversation_id)).await {
        Ok(row) => {
            if let Err(e) =
                botagent::vm::switch_conversation(service, &user_id, &conversation_id).await
            {
                botlog::safe_warn!("conversation switch for {session_id}: {}", e.1);
            }
            Ok(row)
        }
        Err((code, error)) => Err((code.as_u16(), error)),
    }
}

/// Executes a program, waking the workspace VM first when it is not running.
/// A stopped/idle workspace is normal; failing with "not available" after the
/// model already produced a program loses the whole turn.
#[cfg(feature = "agent-vm")]
pub async fn exec_with_wake_result(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Result<Option<(bool, String, String)>, WorkspaceError> {
    match exec_in_user_vm_result(
        user_id, session_id, language, code, timeout_ms, packages,
    )
    .await
    {
        Ok(Some(outcome)) => return Ok(Some(outcome)),
        Ok(None) => {}
        Err((status, error)) => {
            // The wake below still retries the run; keep the first failure
            // visible for diagnosis instead of discarding it.
            botlog::safe_warn!("workspace rapid try failed ({status}): {error}");
        }
    }
    let row = ensure_agent_vm_result(session_id, user_id, bot_uuid).await?;
    let Some(service) = SERVICE.get() else {
        return Err((
            503,
            "the agent workspace subsystem is unavailable".to_string(),
        ));
    };
    let conversation_id = session_id.to_string();
    match botagent::vm::exec_in_conversation(
        service,
        &row.vm_name,
        &conversation_id,
        language,
        code,
        timeout_ms,
        packages,
    )
    .await
    {
        Ok(outcome) => Ok(Some(outcome)),
        Err((status, error)) => Err((status.as_u16(), error)),
    }
}

/// Container name of the caller's running agent VM, if any. Used by the
/// terminal app to open a shell inside the user's own workspace.
#[cfg(feature = "agent-vm")]
pub async fn user_vm_name(user_id: uuid::Uuid) -> Option<String> {
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    Some(row.vm_name)
}

/// File names currently present in the conversation's `output/` folder.
pub async fn list_user_vm_output(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
) -> Option<Vec<String>> {
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    botagent::vm::list_conversation_output(
        service,
        &row.vm_name,
        &session_id.to_string(),
    )
    .await
    .ok()
}

/// Reads a file from the conversation folder (#1584 drive_fetch target check).
#[cfg(feature = "agent-vm")]
pub async fn read_session_file(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    rel_path: &str,
) -> Result<Vec<u8>, String> {
    let service = SERVICE.get().ok_or_else(|| "workspace unavailable".to_string())?;
    let row = botagent::vm::get_user_vm(service, &user_id)
        .map_err(|(_, e)| e)?
        .ok_or_else(|| "no workspace".to_string())?;
    if row.status != botagent::vm::STATUS_RUNNING {
        return Err("workspace is not running".to_string());
    }
    botagent::transfer::read_file(
        service,
        &row.vm_name,
        &session_id.to_string(),
        rel_path,
    )
    .await
    .map_err(|(_, e)| e)
}

/// Writes a file into the conversation folder (#1584 Drive → VM).
#[cfg(feature = "agent-vm")]
pub async fn write_session_file(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    rel_path: &str,
    bytes: &[u8],
) -> Result<usize, String> {
    let service = SERVICE.get().ok_or_else(|| "workspace unavailable".to_string())?;
    let row = botagent::vm::get_user_vm(service, &user_id)
        .map_err(|(_, e)| e)?
        .ok_or_else(|| "no workspace".to_string())?;
    if row.status != botagent::vm::STATUS_RUNNING {
        return Err("workspace is not running".to_string());
    }
    botagent::transfer::write_file(
        service,
        &row.vm_name,
        &session_id.to_string(),
        rel_path,
        bytes,
    )
    .await
    .map_err(|(_, e)| e)
}

/// Lists `output/` or `data/` with file sizes (#1584 folder card / save).
pub async fn list_session_folder(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    folder: &str,
) -> Option<Vec<(String, u64)>> {
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    botagent::transfer::list_folder(
        service,
        &row.vm_name,
        &session_id.to_string(),
        folder,
    )
    .await
    .ok()
    .map(|files| files.into_iter().map(|f| (f.name, f.size)).collect())
}
