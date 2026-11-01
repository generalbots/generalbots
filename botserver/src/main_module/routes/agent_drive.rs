//! `POST /api/agent/drive/save` — publish conversation output to Drive (#1584).
//!
//! The chat folder card calls this when the user clicks "Save to Drive". The
//! native tool does the same from the model side; both share
//! [`crate::core::bot::drive_bridge`].

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::Extension;
use botcore::middleware::AuthenticatedUser;
use botcore::shared::state::AppState;
use diesel::{OptionalExtension, RunQueryDsl};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::{agent_vm_hook, drive_bridge};

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
fn session_bot(
    state: &AppState,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, ApiError> {
    use diesel::sql_types::Uuid as SqlUuid;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = SqlUuid)]
        bot_id: Uuid,
        #[diesel(sql_type = SqlUuid)]
        user_id: Uuid,
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|_| err(StatusCode::SERVICE_UNAVAILABLE, "database unavailable"))?;
    let row: Option<Row> = diesel::sql_query(
        "SELECT bot_id, user_id FROM user_sessions WHERE id = $1 LIMIT 1",
    )
    .bind::<SqlUuid, _>(session_id)
    .get_result(&mut conn)
    .optional()
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed"))?;
    let row = row.ok_or_else(|| err(StatusCode::NOT_FOUND, "conversation not found"))?;
    if row.user_id != user_id {
        return Err(err(
            StatusCode::FORBIDDEN,
            "conversation belongs to another user",
        ));
    }
    Ok(row.bot_id)
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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<SaveOutputRequest>,
) -> Result<Json<Value>, ApiError> {
    let bot_uuid = session_bot(&state, req.conversation_id, user.user_id)?;
    let available = agent_vm_hook::list_session_folder(user.user_id, req.conversation_id, "output")
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
        match agent_vm_hook::read_session_file(user.user_id, req.conversation_id, &path).await {
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
