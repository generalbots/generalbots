//! Workspace tab persistence for the suite chat (issue #1168).
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::anonymous_auth::{resolve_chat_user_uuid, resolve_token_user};

type HandlerError = (StatusCode, String);

const MAX_TABS: usize = 20;
const MAX_TAB_JSON_BYTES: usize = 32 * 1024;

// Canonical definition lives in `botschema` (#1497).
pub use botschema::user_workspace_tabs;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = user_workspace_tabs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct WorkspaceTabsRow {
    user_id: Uuid,
    tabs: serde_json::Value,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TabsBody {
    tabs: serde_json::Value,
}

/// Resolves the tab owner the same way chat history does: the chat sends its
/// `gb_` suite session token, not a JWT, so a JWT-only parser rejected every
/// restore/save with 401. `resolve_token_user` validates both the cached
/// suite session token and the signed cloud JWT.
fn requester(headers: &HeaderMap) -> Result<Uuid, HandlerError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    let (user_raw, _roles) = resolve_token_user(token).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        )
    })?;
    Ok(resolve_chat_user_uuid(&user_raw))
}

pub async fn get_tabs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_id = requester(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
    let row = user_workspace_tabs::dsl::user_workspace_tabs
        .find(user_id)
        .select(WorkspaceTabsRow::as_select())
        .get_result::<WorkspaceTabsRow>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("tabs lookup: {e}")))?;
    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "tabs": r.tabs,
            "updated_at": r.updated_at.to_rfc3339(),
        }))),
        None => Ok(Json(serde_json::json!({ "tabs": [], "updated_at": null }))),
    }
}

pub async fn put_tabs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<TabsBody>,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_id = requester(&headers)?;
    if serde_json::to_string(&body.tabs)
        .map(|s| s.len() > MAX_TAB_JSON_BYTES)
        .unwrap_or(true)
    {
        return Err((StatusCode::BAD_REQUEST, "tabs payload too large".to_string()));
    }
    if body.tabs.as_array().map(|a| a.len()).unwrap_or(0) > MAX_TABS {
        return Err((StatusCode::BAD_REQUEST, "too many tabs".to_string()));
    }
    let row = WorkspaceTabsRow {
        user_id,
        tabs: body.tabs.clone(),
        updated_at: Utc::now(),
    };
    let mut conn = state
        .conn
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
    diesel::insert_into(user_workspace_tabs::dsl::user_workspace_tabs)
        .values(&row)
        .on_conflict(user_workspace_tabs::dsl::user_id)
        .do_update()
        .set((
            user_workspace_tabs::dsl::tabs.eq(row.tabs.clone()),
            user_workspace_tabs::dsl::updated_at.eq(row.updated_at),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("tabs save: {e}")))?;
    Ok(Json(serde_json::json!({
        "saved": true,
        "updated_at": row.updated_at.to_rfc3339(),
    })))
}

pub fn configure_workspace_tabs_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/user/workspace/tabs", get(get_tabs).put(put_tabs))
}
