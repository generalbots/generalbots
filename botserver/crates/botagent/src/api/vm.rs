//! Per-user VM status, conversation switching and code execution routes.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;

use crate::models::{ConversationSwitchBody, VmExecBody};
use crate::vm;
use crate::AgentService;

use super::auth::caller_uuid;
use super::EXEC_LIMITER;

/// `GET /api/agent/vm/status` — current VM status for authenticated user.
pub async fn user_vm_status(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (_, user_id, _) = caller_uuid(&headers)?;
    match vm::get_user_vm(&state, &user_id)? {
        None => Ok(Json(serde_json::json!({ "has_vm": false }))),
        Some(row) => Ok(Json(serde_json::json!({
            "has_vm": true,
            "vm_name": row.vm_name,
            "status": row.status,
            "active_conversation_id": row.active_conversation_id,
            "last_active_at": row.last_active_at.to_rfc3339(),
        }))),
    }
}

/// `POST /api/agent/conversations/switch` — switch active conversation,
/// killing previous jobs and ensuring the new folder exists.
pub async fn switch_conversation(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Json(body): Json<ConversationSwitchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (_, user_id, _) = caller_uuid(&headers)?;
    let row = vm::switch_conversation(&state, &user_id, &body.conversation_id).await?;
    Ok(Json(serde_json::json!({
        "status": "switched",
        "active_conversation_id": row.active_conversation_id,
        "vm_name": row.vm_name,
    })))
}

/// `POST /api/agent/vm/exec` — execute code in the user's VM, scoped to the
/// conversation folder. The LLM generates programs on-the-fly and the output
/// is returned for preview (#1580).
pub async fn vm_exec(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Json(body): Json<VmExecBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (sub, user_id, _) = caller_uuid(&headers)?;
    if !EXEC_LIMITER.check(&format!("vmexec:{sub}"), 5.0) {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()));
    }

    let session = vm::get_user_vm(&state, &user_id)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "No active VM".to_string()))?;

    if session.status != vm::STATUS_RUNNING {
        return Err((StatusCode::CONFLICT, format!("VM is {}", session.status)));
    }

    let timeout = body.timeout_ms.unwrap_or(30_000).min(120_000);
    let (ok, stdout, stderr) = vm::exec_in_conversation(
        &state, &session.vm_name, &body.conversation_id,
        &body.language, &body.code, timeout, &body.packages,
    ).await?;

    let files = vm::list_conversation_output(&state, &session.vm_name, &body.conversation_id)
        .await
        .unwrap_or_default();
    let attachments: Vec<serde_json::Value> = files
        .iter()
        .map(|name| {
            serde_json::json!({
                "name": name,
                "path": format!("output/{name}"),
                "link": format!(
                    "[{name}](vmfile:{}/output/{name})",
                    body.conversation_id
                ),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "success": ok,
        "stdout": stdout.chars().take(20_000).collect::<String>(),
        "stderr": stderr.chars().take(20_000).collect::<String>(),
        "conversation_id": body.conversation_id,
        "files": files,
        "attachments": attachments,
    })))
}
