//! HTTP handlers for agent sessions, per-user VMs, and the sandbox.
//!
//! JWT handling: claims are read with a dependency-free base64url split of the
//! token payload (`b64_json_sub`). Signature verification is expected to be
//! performed by the upstream gateway/integrator.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

use crate::keys;
use crate::models::{AgentSessionRow, ConversationSwitchBody, ExecBody, ModeBody, VmExecBody};
use crate::sandbox;
use crate::state::RateLimiter;
use crate::vm;
use crate::AgentService;

static MODE_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);
static EXEC_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Agent request failed".to_string())
}

fn unauthorized(msg: &str) -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, msg.to_string())
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()
        .map(|v| v.to_string())
        .filter(|v| v.starts_with("Bearer ") || v.starts_with("bearer "))
}

fn b64_decode(input: &str) -> Option<Vec<u8>> {
    fn value(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a') as u32 + 26),
            b'0'..=b'9' => Some((c - b'0') as u32 + 52),
            b'+' | b'-' => Some(62),
            b'/' | b'_' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for &byte in input.as_bytes() {
        if byte == b'=' || byte == b'\r' || byte == b'\n' { continue; }
        buffer = (buffer << 6) | value(byte)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

pub fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let payload_segment = token.split('.').nth(1)?;
    let decoded = b64_decode(payload_segment)?;
    serde_json::from_slice(&decoded).ok()
}

pub fn b64_json_sub(token: &str) -> Option<String> {
    jwt_payload(token)?
        .get("sub")
        .and_then(|s| s.as_str())
        .map(str::to_string)
}

pub fn is_admin_claims(claims: &serde_json::Value) -> bool {
    claims.get("role").and_then(|r| r.as_str()) == Some("admin")
}

/// Verifies the token signature with the SaaS JWT secret handed over by the
/// server at boot (`GB_AGENT_JWT_SECRET`).
fn verified_sub(raw: &str) -> Option<Uuid> {
    // `bearer_token` returns the whole header value; the signature covers the
    // JWT only, so drop the scheme prefix here.
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .unwrap_or(raw)
        .trim();
    let secret = std::env::var("GB_AGENT_JWT_SECRET").ok()?;
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    // Suite tokens are claim-light (no `aud`).
    validation.validate_aud = false;
    let data = jsonwebtoken::decode::<serde_json::Value>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| tracing::warn!("agent bearer rejected: {e}"))
    .ok()?;
    let sub = data.claims.get("sub")?.as_str()?;
    Uuid::parse_str(sub).ok()
}

fn caller_uuid(headers: &HeaderMap) -> Result<(String, Uuid, bool), (StatusCode, String)> {
    let token = bearer_token(headers).ok_or_else(|| unauthorized("Missing bearer token"))?;
    let payload = jwt_payload(&token).ok_or_else(|| unauthorized("Invalid token"))?;
    let sub = b64_json_sub(&token).ok_or_else(|| unauthorized("Token lacks sub claim"))?;
    // SECURITY: the payload segment is attacker-controlled. Every caller must
    // pass signature verification before acting on another user's VM — a
    // handcrafted `sub` used to reach any workspace.
    let user_id = match verified_sub(&token) {
        Some(verified) => verified,
        None if std::env::var("GB_AGENT_JWT_SECRET").is_err() => {
            tracing::warn!(
                "GB_AGENT_JWT_SECRET not set; falling back to unverified claims (tests only)"
            );
            Uuid::parse_str(&sub).unwrap_or_else(|_| {
                Uuid::new_v5(
                    &Uuid::NAMESPACE_URL,
                    format!("zitadel:{sub}").as_bytes(),
                )
            })
        }
        None => return Err(unauthorized("Invalid or unverified token")),
    };
    let admin = is_admin_claims(&payload);
    Ok((sub, user_id, admin))
}

pub fn authorize_session_access(
    headers: &HeaderMap,
    owner_user_id: &Uuid,
) -> Result<(), (StatusCode, String)> {
    let (_, user_id, admin) = caller_uuid(headers)?;
    if admin || user_id == *owner_user_id {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Not authorized".to_string()))
    }
}

// ─── Legacy toggle (kept for backward compat) ────────────────────────────

/// `POST /api/agent/sessions/mode` {session_id, enabled}
pub async fn set_agent_mode(
    State(state): State<Arc<AgentService>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(body): Json<ModeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (sub, user_id, _) = caller_uuid(&headers)?;
    if !MODE_LIMITER.check(&format!("mode:{sub}"), 10.0) {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()));
    }

    let bot_id = params
        .get("bot_id")
        .and_then(|b| Uuid::parse_str(b).ok())
        .unwrap_or_else(|| Uuid::nil());

    let response = if body.enabled {
        let row = vm::ensure_vm(&state, &user_id, &bot_id, Some(&body.session_id)).await?;
        serde_json::json!({ "status": "enabled", "item": row })
    } else {
        vm::stop_vm_for_user(&state, &user_id).await?;
        serde_json::json!({ "status": "disabled" })
    };
    Ok(Json(response))
}

/// `GET /api/agent/sessions/current?session_id=`
pub async fn current_session(
    State(state): State<Arc<AgentService>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (_, user_id, _) = caller_uuid(&headers)?;

    // Try per-user lookup first (#1580), fall back to session_id for compat.
    let row = match vm::get_user_vm(&state, &user_id)? {
        Some(r) => Some(r),
        None => {
            if let Some(sid) = params.get("session_id").filter(|s| !s.trim().is_empty()) {
                let clean = vm::sanitize_id(sid)?;
                let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                use crate::schema::agent_sessions::dsl::{agent_sessions, session_id};
                agent_sessions
                    .filter(session_id.eq(&clean))
                    .first::<AgentSessionRow>(&mut conn)
                    .optional()
                    .map_err(|e| internal(format!("agent_sessions select: {e}")))?
            } else {
                None
            }
        }
    };

    match row {
        None => Ok(Json(serde_json::json!({ "found": false }))),
        Some(row) => {
            authorize_session_access(&headers, &row.user_id)?;
            Ok(Json(serde_json::json!({ "found": true, "item": row })))
        }
    }
}

// ─── Per-user VM endpoints (#1580) ───────────────────────────────────────

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
    let (_, user_id, _) = caller_uuid(&headers)?;
    let session = vm::get_user_vm(&state, &user_id)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "No active VM".to_string()))?;
    if session.status != vm::STATUS_RUNNING {
        return Err((StatusCode::CONFLICT, format!("VM is {}", session.status)));
    }
    let conversation_id = params.get("conversation_id").cloned().unwrap_or_default();
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
    let (ok, stdout, stderr) = vm::exec_in_vm(
        &state, &session.vm_name,
        &["sh".into(), "-c".into(), format!("base64 -w0 '{}'", full_path)],
    ).await?;
    if !ok {
        return Err((StatusCode::NOT_FOUND, format!("File not found: {}", stderr)));
    }
    let encoded = stdout.trim().to_string();
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
    let (_, user_id, _) = caller_uuid(&headers)?;
    let session = vm::get_user_vm(&state, &user_id)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "No active VM".to_string()))?;
    if session.status != vm::STATUS_RUNNING {
        return Err((StatusCode::CONFLICT, format!("VM is {}", session.status)));
    }
    let conversation_id = params.get("conversation_id").cloned().unwrap_or_default();
    if conversation_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "conversation_id required".to_string(),
        ));
    }
    let clean_cid = vm::sanitize_id(&conversation_id)?;
    let files = vm::list_conversation_output(&state, &session.vm_name, &clean_cid).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "conversation_id": clean_cid,
        "files": files,
    })))
}

/// `POST /api/v1/sandbox/exec` — Bearer org key OR JWT.
pub async fn sandbox_exec(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Json(body): Json<ExecBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = bearer_token(&headers).ok_or_else(|| unauthorized("Missing credentials"))?;

    let (org_id, user_id, limiter_key) = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        if let Ok(ctx) = keys::authenticate_key(&auth_header, &mut conn) {
            (Some(ctx.org_id), None, format!("exec:key:{}", ctx.key_id))
        } else {
            let (sub, uid, _) = caller_uuid(&headers)?;
            (None, Some(uid), format!("exec:user:{sub}"))
        }
    };

    if !EXEC_LIMITER.check(&limiter_key, 5.0) {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()));
    }

    let result = sandbox::run_sandbox(&state, org_id, user_id, &body).await?;
    Ok(Json(result))
}
