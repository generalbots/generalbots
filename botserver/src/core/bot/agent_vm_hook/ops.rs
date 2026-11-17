use super::*;

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
    if workspace_disabled(session_id) {
        return Err(workspace_disabled_error());
    }
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
    // #2239/#2264 — the workspace run is dispatched here: refuse it outright
    // when the chat's Agent/VM switch is off for this turn.
    if workspace_disabled(session_id) {
        botlog::safe_info!("workspace run refused for session {session_id}: Agent/VM is off");
        return Err(workspace_disabled_error());
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
    if workspace_disabled(session_id) {
        return None;
    }
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    botagent::vm::list_conversation_output(service, &row.vm_name, &session_id.to_string())
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
    if workspace_disabled(session_id) {
        return Err(crate::core::bot::pipeline::agent_turn::DISABLED_MESSAGE.to_string());
    }
    let service = SERVICE
        .get()
        .ok_or_else(|| "workspace unavailable".to_string())?;
    let row = botagent::vm::get_user_vm(service, &user_id)
        .map_err(|(_, e)| e)?
        .ok_or_else(|| "no workspace".to_string())?;
    if row.status != botagent::vm::STATUS_RUNNING {
        return Err("workspace is not running".to_string());
    }
    botagent::transfer::read_file(service, &row.vm_name, &session_id.to_string(), rel_path)
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
    if workspace_disabled(session_id) {
        return Err(crate::core::bot::pipeline::agent_turn::DISABLED_MESSAGE.to_string());
    }
    let service = SERVICE
        .get()
        .ok_or_else(|| "workspace unavailable".to_string())?;
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
    if workspace_disabled(session_id) {
        return None;
    }
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    botagent::transfer::list_folder(service, &row.vm_name, &session_id.to_string(), folder)
        .await
        .ok()
        .map(|files| files.into_iter().map(|f| (f.name, f.size)).collect())
}

/// Same listing as [`list_session_folder`] plus each file's modification
/// stamp (#2143). The turn-end publisher compares `(size, modified)` with the
/// copy it already sent to Drive, so an unchanged artifact is never uploaded
/// twice.
pub async fn list_session_folder_meta(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    folder: &str,
) -> Option<Vec<(String, u64, u64)>> {
    if workspace_disabled(session_id) {
        return None;
    }
    let service = SERVICE.get()?;
    let row = botagent::vm::get_user_vm(service, &user_id).ok()??;
    if row.status != botagent::vm::STATUS_RUNNING {
        return None;
    }
    botagent::transfer::list_folder(service, &row.vm_name, &session_id.to_string(), folder)
        .await
        .ok()
        .map(|files| {
            files
                .into_iter()
                .map(|f| (f.name, f.size, f.modified))
                .collect()
        })
}
