//! HTTP handlers for agent sessions and the sandbox entry point.
//!
//! JWT handling: botlib exposes no reusable decode utility (verified by
//! search), so claims are read with a dependency-free base64url split of the
//! token payload (`b64_json_sub`). Signature verification is expected to be
//! performed by the upstream gateway/integrator; `is_admin_claims` is an
//! isolated stub kept deliberately simple so it can be hardened in one place.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

use crate::keys;
use crate::models::{AgentSessionRow, ExecBody, ModeBody};
use crate::sandbox;
use crate::state::RateLimiter;
use crate::vm;
use crate::AgentService;

static MODE_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);
static EXEC_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Agent request failed".to_string(),
    )
}

fn unauthorized(msg: &str) -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, msg.to_string())
}

/// Extract a bearer token from the Authorization header.
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()
        .map(|v| v.to_string())
        .filter(|v| v.starts_with("Bearer ") || v.starts_with("bearer "))
}

/// Dependency-free base64 decoder accepting standard and URL-safe alphabets,
/// tolerant of missing padding.
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
        if byte == b'=' || byte == b'\r' || byte == b'\n' {
            continue;
        }
        buffer = (buffer << 6) | value(byte)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

/// Tiny JWT payload reader: splits on '.', base64url-decodes the payload and
/// parses it as JSON. No signature verification happens here.
pub fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let payload_segment = token.split('.').nth(1)?;
    let decoded = b64_decode(payload_segment)?;
    serde_json::from_slice(&decoded).ok()
}

/// `sub` claim of an unverified token payload.
pub fn b64_json_sub(token: &str) -> Option<String> {
    jwt_payload(token)
        .get("sub")
        .and_then(|s| s.as_str())
        .map(str::to_string)
}

/// Admin role check — isolated stub; integrators may replace with RBAC.
pub fn is_admin_claims(claims: &serde_json::Value) -> bool {
    claims.get("role").and_then(|r| r.as_str()) == Some("admin")
}

fn caller_uuid(headers: &HeaderMap) -> Result<(String, Uuid, bool), (StatusCode, String)> {
    let token = bearer_token(headers).ok_or_else(|| unauthorized("Missing bearer token"))?;
    let payload = jwt_payload(&token).ok_or_else(|| unauthorized("Invalid token"))?;
    let admin = is_admin_claims(&payload);
    let sub = b64_json_sub(&token).ok_or_else(|| unauthorized("Token lacks sub claim"))?;
    let user_id = Uuid::parse_str(&sub).unwrap_or_else(|_| {
        tracing::warn!("JWT sub '{sub}' is not a UUID; using nil uuid for agent session ownership");
        Uuid::nil()
    });
    Ok((sub, user_id, admin))
}

/// Ownership guard for session-scoped operations: admins pass, otherwise the
/// caller's `sub` must equal the row owner.
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

/// `POST /api/agent/sessions/mode` {session_id, enabled}
/// Optional query param `bot_id` seeds the owning bot on first provision.
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
        .unwrap_or_else(|| {
            tracing::info!("mode switch without bot_id for sub '{sub}'; defaulting to nil bot");
            Uuid::nil()
        });

    let response = if body.enabled {
        let row = vm::ensure_vm(&state, &body.session_id, &user_id, &bot_id).await?;
        serde_json::json!({ "status": "enabled", "item": row })
    } else {
        vm::stop_vm(&state, &body.session_id).await?;
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
    let session_id = params
        .get("session_id")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing session_id parameter".to_string()))?;
    let clean = vm::sanitize_session_id(session_id)?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    use crate::schema::agent_sessions::dsl::{agent_sessions, session_id as sid};
    let row: Option<AgentSessionRow> = agent_sessions
        .filter(sid.eq(&clean))
        .first(&mut conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select: {e}")))?;

    match row {
        None => Ok(Json(serde_json::json!({ "found": false }))),
        Some(row) => {
            authorize_session_access(&headers, &row.user_id)?;
            Ok(Json(serde_json::json!({ "found": true, "item": row })))
        }
    }
}

/// `POST /api/v1/sandbox/exec` — Bearer org key OR JWT.
pub async fn sandbox_exec(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Json(body): Json<ExecBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let auth_header = bearer_token(&headers).ok_or_else(|| unauthorized("Missing credentials"))?;

    // Org API key first; fall back to a JWT identity.
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
