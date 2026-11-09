//! Workspace wake + run helpers (#1167, #1580, #2046).
//!
//! Split from `agent_vm_hook.rs` so that file stays under the size limit. The
//! buffered path retries the run after provisioning; the streaming path is the
//! one the chat pipeline uses so `@@GB_TRACE@@` markers reach the run card.

use super::*;

pub(crate) async fn exec_with_wake_result(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Result<Option<(bool, String, String)>, WorkspaceError> {
    match exec_in_user_vm_result(user_id, session_id, language, code, timeout_ms, packages).await {
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

/// #2046 — streaming variant of [`exec_with_wake_result`]. Publishes the
/// program's stdout/stderr lines while it runs so trace markers become live
/// run-card rows. The rapid (already-running) path is skipped: a cold start
/// would have produced no events anyway, and the wake path is the one that
/// runs the program.
pub(crate) async fn exec_with_wake_events(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
    events: tokio::sync::mpsc::UnboundedSender<botagent::stream::ExecEvent>,
) -> Result<Option<(bool, String, String)>, WorkspaceError> {
    let row = ensure_agent_vm_result(session_id, user_id, bot_uuid).await?;
    let Some(service) = SERVICE.get() else {
        return Err((
            503,
            "the agent workspace subsystem is unavailable".to_string(),
        ));
    };
    let conversation_id = session_id.to_string();
    match botagent::vm::exec_in_conversation_streamed(
        service,
        &row.vm_name,
        &conversation_id,
        language,
        code,
        timeout_ms,
        packages,
        events,
    )
    .await
    {
        Ok(outcome) => Ok(Some(outcome)),
        Err((status, error)) => Err((status.as_u16(), error)),
    }
}

