//! Sandboxed code execution route (`POST /api/v1/sandbox/exec`).

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;

use crate::keys;
use crate::models::ExecBody;
use crate::sandbox as sandbox_service;
use crate::AgentService;

use super::auth::caller_uuid;
use super::{bearer_token, internal, unauthorized, EXEC_LIMITER};

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

    let result = sandbox_service::run_sandbox(&state, org_id, user_id, &body).await?;
    Ok(Json(result))
}
