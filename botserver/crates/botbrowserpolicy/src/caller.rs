//! Verified-caller identity resolution for the browser-task API (#2877).
//!
//! Every task-id endpoint derives the caller's user id and tenant scope from
//! a signature-verified token only. Unverifiable tokens yield no identity, so
//! the request is refused before any ownership decision is taken.

use crate::jwt::{bearer_token, verified_claims};
use crate::ownership::TaskPrincipal;
use axum::http::HeaderMap;
use botsecurity_core::error_sanitizer::SafeErrorResponse;
use uuid::Uuid;

/// Full verified identity of the caller: `sub` plus the signed per-bot
/// workspace (`branch_id` -> `bot_id`) and `.gborg` tenant (`org_id`).
pub(crate) fn resolve_principal(
    headers: &HeaderMap,
) -> Result<TaskPrincipal, SafeErrorResponse> {
    let token = bearer_token(headers).ok_or_else(SafeErrorResponse::unauthorized)?;
    let claims = verified_claims(token).ok_or_else(SafeErrorResponse::unauthorized)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| SafeErrorResponse::new("unauthorized", "Invalid token subject"))?;
    Ok(TaskPrincipal {
        user_id,
        org_id: claims.org_id,
        bot_id: claims.branch_id,
    })
}

/// Verified user id for identity-only surfaces (browsing memory).
pub(crate) fn resolve_user_id(headers: &HeaderMap) -> Result<Uuid, SafeErrorResponse> {
    Ok(resolve_principal(headers)?.user_id)
}

/// SSE transport: EventSource cannot set headers, so the suite passes the
/// Bearer token as `?token=`. Header wins when both are present; a header
/// token that fails verification never falls back to the query parameter.
pub(crate) fn resolve_principal_sse(
    headers: &HeaderMap,
    token_param: &str,
) -> Result<TaskPrincipal, SafeErrorResponse> {
    let header_token = match bearer_token(headers) {
        Some(token) => token,
        None => "",
    };
    let token = if header_token.is_empty() {
        token_param.trim()
    } else {
        header_token
    };
    let claims = verified_claims(token).ok_or_else(SafeErrorResponse::unauthorized)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| SafeErrorResponse::new("unauthorized", "Invalid token subject"))?;
    Ok(TaskPrincipal {
        user_id,
        org_id: claims.org_id,
        bot_id: claims.branch_id,
    })
}
