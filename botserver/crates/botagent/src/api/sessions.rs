//! Legacy per-session mode toggle and current-session lookup.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AgentSessionRow, ModeBody};
use crate::vm;
use crate::AgentService;

use super::auth::{authorize_session_access, caller_uuid};
use super::{internal, MODE_LIMITER};

/// `POST /api/agent/sessions/mode` {session_id, enabled}
pub async fn set_agent_mode(
    State(state): State<Arc<AgentService>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(body): Json<ModeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (sub, user_id, _) = caller_uuid(&headers)?;
    if !MODE_LIMITER.check(&format!("mode:{sub}"), 10.0) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        ));
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
                let mut conn = state
                    .pool()
                    .get()
                    .map_err(|e| internal(format!("DB pool: {e}")))?;
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
