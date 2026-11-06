//! VM file listing, download, inline preview and raw responses.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::vm;
use crate::AgentService;

use super::auth::caller_uuid;
use super::unauthorized;

#[derive(diesel::QueryableByName)]
struct VmNameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    vm_name: String,
}

/// Desktop/single-user installs run without accounts, so the chat page holds
/// no bearer token and a plain download link cannot authenticate. In that mode
/// the conversation id carried by the link identifies the workspace: resolve
/// the VM from `agent_sessions.active_conversation_id` instead of inventing an
/// identity. Authenticated callers always take the normal per-user path.
fn desktop_vm_name(state: &Arc<AgentService>, conversation_id: &str) -> Option<String> {
    if conversation_id.is_empty() {
        return None;
    }
    let mut conn = state.pool().get().ok()?;
    let rows = diesel::sql_query(
        "SELECT vm_name FROM agent_sessions WHERE status = 'running' \
         AND active_conversation_id = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(conversation_id)
    .load::<VmNameRow>(&mut conn)
    .ok()?;
    rows.into_iter().next().map(|row| row.vm_name)
}

fn resolve_vm_name(
    state: &Arc<AgentService>,
    headers: &HeaderMap,
    conversation_id: &str,
) -> Result<String, (StatusCode, String)> {
    if let Ok((_, user_id, _)) = caller_uuid(headers) {
        let session = vm::get_user_vm(state, &user_id)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "No active VM".to_string()))?;
        if session.status != vm::STATUS_RUNNING {
            return Err((StatusCode::CONFLICT, format!("VM is {}", session.status)));
        }
        return Ok(session.vm_name);
    }
    // Single-user/desktop installs opt in with AGENT_ALLOW_UNREGISTERED_IDENTITIES=1
    // (the same flag that lets the chat provision a VM without accounts).
    let desktop = vm::allow_unregistered_identities();
    if desktop {
        if let Some(name) = desktop_vm_name(state, conversation_id) {
            return Ok(name);
        }
    }
    Err(unauthorized("Missing credentials"))
}

/// `GET /api/agent/vm/file?conversation_id=&path=` — fetch a file from the
/// user's VM (base64-encoded) for preview/download in chat.
pub async fn vm_file(
    State(state): State<Arc<AgentService>>,
    mut headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    // A file link must work as a plain URL: browsers cannot attach an
    // Authorization header to a download/navigation. Accept the same verified
    // token as a query parameter when the header is absent.
    if !headers.contains_key(axum::http::header::AUTHORIZATION) {
        if let Some(token) = params.get("access_token").filter(|t| !t.is_empty()) {
            if let Ok(value) =
                axum::http::HeaderValue::from_str(&format!("Bearer {token}"))
            {
                headers.insert(axum::http::header::AUTHORIZATION, value);
            }
        }
    }
    let conversation_id = params.get("conversation_id").cloned().unwrap_or_default();
    let vm_name = resolve_vm_name(&state, &headers, &conversation_id)?;
    let rel_path = params.get("path").cloned().unwrap_or_default();
    if conversation_id.is_empty() || rel_path.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "conversation_id and path required".to_string()));
    }
    // Basic path safety: no traversal, no absolute paths.
    if rel_path.contains("..") || rel_path.starts_with('/') || rel_path.contains('\\') {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_string()));
    }
    let clean_cid = vm::sanitize_id(&conversation_id)?;
    let full_path = format!("/opt/vibe/conversations/{}/{}", clean_cid, rel_path);
    // Execute base64 directly with the path as an argument: interpolating it
    // into a shell command broke on quotes, and the shell guard rejects `$`.
    let (ok, stdout, stderr) = vm::exec_in_vm(
        &state, &vm_name,
        &["base64".into(), "-w0".into(), full_path],
    ).await?;
    if !ok {
        return Err((StatusCode::NOT_FOUND, format!("File not found: {}", stderr)));
    }
    let encoded = stdout.trim().to_string();
    // `preview=1` renders Office packages (docx/xlsx/pptx) as a sanitized,
    // script-free HTML document (#1672) so the conversation previews them
    // inline instead of handing the file to an external viewer. Everything
    // goes through the same signed pipeline — same conversation scoping, same
    // path checks, same VM.
    if params.contains_key("preview") {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded.as_bytes())
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("decode failed: {error}")))?;
        let file_name: String = rel_path
            .rsplit('/')
            .next()
            .unwrap_or("file")
            .chars()
            .filter(|character| !matches!(character, '"' | '\r' | '\n' | '\\'))
            .collect();
        return match botcore::kb::document_processor::office_html(&file_name, &bytes) {
            Some(Ok(html)) => Ok((
                [
                    (
                        axum::http::header::CONTENT_TYPE,
                        "text/html; charset=utf-8".to_string(),
                    ),
                    (
                        axum::http::header::CONTENT_SECURITY_POLICY,
                        "default-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'".to_string(),
                    ),
                    (
                        axum::http::header::CONTENT_DISPOSITION,
                        format!("inline; filename=\"{file_name}.html\""),
                    ),
                    (
                        axum::http::header::CACHE_CONTROL,
                        "private, max-age=60".to_string(),
                    ),
                    (
                        axum::http::header::X_FRAME_OPTIONS,
                        "SAMEORIGIN".to_string(),
                    ),
                ],
                html,
            )
                .into_response()),
            Some(Err(why)) => Err((StatusCode::BAD_REQUEST, format!("Preview failed: {why}"))),
            None => Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                format!("{file_name} has no inline preview"),
            )),
        };
    }
    // `raw=1` serves the real bytes so a plain hyperlink downloads the file
    // itself (name, type and content), not a JSON envelope.
    if params.contains_key("raw") {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded.as_bytes())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("decode failed: {e}")))?;
        let file_name: String = rel_path
            .rsplit('/')
            .next()
            .unwrap_or("file")
            .chars()
            .filter(|c| !matches!(c, '"' | '\r' | '\n' | '\\'))
            .collect();
        let mime = match file_name.rsplit('.').next().unwrap_or("") {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "pdf" => "application/pdf",
            "txt" => "text/plain; charset=utf-8",
            "csv" => "text/csv; charset=utf-8",
            "json" => "application/json",
            "md" => "text/markdown; charset=utf-8",
            "zip" => "application/zip",
            "mp4" => "video/mp4",
            "mp3" => "audio/mpeg",
            // Office packages are what the agent produces most often; a real
            // MIME type lets the browser hand the file to its viewer instead
            // of treating every download as a generic octet-stream.
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "ppt" => "application/vnd.ms-powerpoint",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "doc" => "application/msword",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xls" => "application/vnd.ms-excel",
            _ => "application/octet-stream",
        };
        let disposition = format!("inline; filename=\"{file_name}\"");
        return Ok((
            [
                (axum::http::header::CONTENT_TYPE, mime.to_string()),
                (axum::http::header::CONTENT_DISPOSITION, disposition),
                (axum::http::header::CACHE_CONTROL, "private, max-age=60".to_string()),
                // The chat previews PDFs/images in a same-origin iframe; the
                // global DENY made that preview a blank broken-file box.
                (axum::http::header::X_FRAME_OPTIONS, "SAMEORIGIN".to_string()),
            ],
            bytes,
        )
            .into_response());
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "path": rel_path,
        "conversation_id": clean_cid,
        "content_base64": encoded,
    }))
    .into_response())
}

/// `GET /api/agent/vm/files?conversation_id=` — list files in the
/// conversation `output/` folder for chat attachment rendering.
pub async fn vm_files(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let conversation_id = params.get("conversation_id").cloned().unwrap_or_default();
    let vm_name = resolve_vm_name(&state, &headers, &conversation_id)?;
    if conversation_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "conversation_id required".to_string(),
        ));
    }
    let clean_cid = vm::sanitize_id(&conversation_id)?;
    let files = vm::list_conversation_output(&state, &vm_name, &clean_cid).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "conversation_id": clean_cid,
        "files": files,
    })))
}
