//! `POST /api/agent/drive/save` — publish conversation output to Drive (#1584).
//!
//! The chat folder card calls this when the user clicks "Save to Drive". The
//! native tool does the same from the model side; both share
//! [`crate::core::bot::drive_bridge`].

use axum::extract::{Json, State};
use axum::http::{HeaderMap, StatusCode};
use botcore::shared::state::AppState;
use diesel::{OptionalExtension, RunQueryDsl};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::{agent_vm_hook, drive_bridge};

#[derive(Debug, Deserialize)]
pub struct ShareOutputRequest {
    pub conversation_id: Uuid,
    pub file: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveOutputRequest {
    pub conversation_id: Uuid,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub folder: Option<String>,
}

type ApiError = (StatusCode, Json<Value>);

fn err(code: StatusCode, message: &str) -> ApiError {
    (code, Json(json!({ "error": message })))
}

/// The bot a session belongs to, only when the caller owns that session.
/// Column types differ across deployments (uuid vs text), so both sides are
/// compared and returned as text and parsed here.
fn session_bot(
    state: &AppState,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, ApiError> {
    use diesel::sql_types::Text;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        bot_id: String,
        #[diesel(sql_type = Text)]
        user_id: String,
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|_| err(StatusCode::SERVICE_UNAVAILABLE, "database unavailable"))?;
    let row: Option<Row> = diesel::sql_query(
        "SELECT bot_id::text AS bot_id, user_id::text AS user_id \
         FROM user_sessions WHERE id::text = $1 LIMIT 1",
    )
    .bind::<Text, _>(session_id.to_string())
    .get_result(&mut conn)
    .optional()
    .map_err(|e| {
        log::error!("agent drive: session lookup failed: {e}");
        err(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed")
    })?;
    let row = row.ok_or_else(|| err(StatusCode::NOT_FOUND, "conversation not found"))?;
    let owner = Uuid::parse_str(row.user_id.trim())
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "bad session owner"))?;
    if owner != user_id {
        return Err(err(
            StatusCode::FORBIDDEN,
            "conversation belongs to another user",
        ));
    }
    Uuid::parse_str(row.bot_id.trim())
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "bad session bot"))
}

fn safe_name(raw: &str) -> String {
    let base = raw.rsplit('/').next().unwrap_or(raw).trim();
    let clean: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
        .collect();
    if clean.trim().is_empty() {
        "file".to_string()
    } else {
        clean
    }
}

pub async fn save_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SaveOutputRequest>,
) -> Result<Json<Value>, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "missing token"))?;
    let (claims_user, _) = crate::main_module::routes::anonymous_auth::resolve_token_user(token)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "invalid token"))?;
    let user_id = crate::security::user_role::derive_stable_user_uuid(&claims_user);
    let bot_uuid = session_bot(&state, req.conversation_id, user_id)?;
    let available = agent_vm_hook::list_session_folder(user_id, req.conversation_id, "output")
        .await
        .ok_or_else(|| err(StatusCode::CONFLICT, "workspace is not running"))?;
    if available.is_empty() {
        return Err(err(StatusCode::NOT_FOUND, "no files in output/"));
    }
    let wanted: Vec<String> = req.files.iter().map(|f| safe_name(f)).collect();
    let selected: Vec<String> = available
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| wanted.is_empty() || wanted.contains(name))
        .collect();
    if selected.is_empty() {
        return Err(err(StatusCode::NOT_FOUND, "named files are not in output/"));
    }

    let folder = match req.folder.as_deref().map(drive_bridge::slug) {
        Some(slug) if !slug.is_empty() => slug,
        _ => drive_bridge::conversation_title(&state, req.conversation_id).await,
    };

    let mut saved = Vec::new();
    let mut failed = Vec::new();
    for name in selected {
        let path = format!("output/{name}");
        match agent_vm_hook::read_session_file(user_id, req.conversation_id, &path).await {
            Ok(bytes) => match drive_bridge::save_object(&state, &bot_uuid, &folder, &name, bytes)
                .await
            {
                Some(drive_path) => saved.push(json!({ "name": name, "path": drive_path })),
                None => failed.push(name),
            },
            Err(e) => failed.push(format!("{name}: {e}")),
        }
    }

    if saved.is_empty() {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "could not save any file to Drive",
        ));
    }
    Ok(Json(json!({
        "success": true,
        "folder": folder,
        "saved": saved,
        "failed": failed,
    })))
}

/// `POST /api/agent/drive/share` — publishes one output file and returns a
/// public link (#1551 GB6-12). The video/file never leaves Drive: the VM copy
/// is saved under the conversation folder, then a token row is minted for that
/// exact object, so the link keeps working after the conversation is deleted.
pub async fn share_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ShareOutputRequest>,
) -> Result<Json<Value>, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "missing token"))?;
    let (claims_user, _) = crate::main_module::routes::anonymous_auth::resolve_token_user(token)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "invalid token"))?;
    let user_id = crate::security::user_role::derive_stable_user_uuid(&claims_user);
    let bot_uuid = session_bot(&state, req.conversation_id, user_id)?;
    let name = safe_name(&req.file);
    let bytes = agent_vm_hook::read_session_file(
        user_id,
        req.conversation_id,
        &format!("output/{name}"),
    )
    .await
    .map_err(|_| err(StatusCode::NOT_FOUND, "file not found in output/"))?;

    let folder = drive_bridge::conversation_title(&state, req.conversation_id).await;
    let stored = drive_bridge::save_object(&state, &bot_uuid, &folder, &name, bytes)
        .await
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, "could not save to Drive"))?;
    let (bucket, key) = stored
        .split_once('/')
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, "bad drive path"))?;

    use diesel::sql_types::Text;
    let share_token = Uuid::new_v4().simple().to_string();
    let expires = chrono::Utc::now() + chrono::Duration::days(7);
    let mut conn = state
        .conn
        .get()
        .map_err(|_| err(StatusCode::SERVICE_UNAVAILABLE, "database unavailable"))?;
    diesel::sql_query(
        "INSERT INTO drive_public_links (owner_id, bucket, path, key, scope, share_token, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<Text, _>(user_id.to_string())
    .bind::<Text, _>(bucket)
    .bind::<Text, _>(&name)
    .bind::<Text, _>(key)
    .bind::<Text, _>("bot")
    .bind::<Text, _>(&share_token)
    .bind::<diesel::sql_types::Timestamptz, _>(expires)
    .execute(&mut conn)
    .map_err(|e| {
        log::error!("agent drive: share link insert failed: {e}");
        err(StatusCode::INTERNAL_SERVER_ERROR, "could not create link")
    })?;

    Ok(Json(json!({
        "success": true,
        "token": share_token,
        "url": format!("/api/files/public/{share_token}"),
        "expires_at": expires.to_rfc3339(),
        "drive_path": stored,
    })))
}
