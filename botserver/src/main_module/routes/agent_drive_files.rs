//! The conversation copies in Drive, for the shell's file panel and the Drive
//! tab's "Arquivos das conversas".
//!
//! - `GET /api/agent/drive/conversations` lists `{bot}.gbdrive/conversations/`
//!   of every bot the caller talked to, only the folders of the caller's own
//!   conversations, newest conversation first.
//! - `GET /api/agent/drive/file?path=conversations/<folder>/<file>&raw=1`
//!   serves one of those files; `&preview=1` renders an Office package as the
//!   same sanitized HTML the workspace file preview uses.
//!
//! The bot's Drive location comes from `bot_drive_location_for` (an org bot
//! lives under its `{org}.gborg` bucket). A folder no session of the caller
//! explains is answered as not found.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderName, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use botcore::shared::state::AppState;
use diesel::RunQueryDsl;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::bot::drive_bridge;

mod folders;
use folders::{group, owner_of, validate_path, OwnedFolder, OwnedSession, CONVERSATIONS};

/// Sessions of the caller looked at, newest first.
const MAX_SESSIONS: i64 = 2000;
/// Files whose modification time is looked up in one listing.
const MAX_LISTED_FILES: usize = 500;
/// Largest file served (100 MB).
const MAX_SERVED_BYTES: u64 = 100 * 1024 * 1024;

fn refuse(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({ "error": code }))).into_response()
}

fn internal(error: &(impl std::fmt::Display + ?Sized)) -> Response {
    let body = crate::security::error_sanitizer::log_and_sanitize_json(error, "agent drive files", None);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": body }))).into_response()
}

fn caller(headers: &HeaderMap) -> Result<Uuid, Response> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| refuse(StatusCode::UNAUTHORIZED, "token_missing"))?;
    let (claims_user, _) = crate::main_module::routes::anonymous_auth::resolve_token_user(token)
        .ok_or_else(|| refuse(StatusCode::UNAUTHORIZED, "token_invalid"))?;
    Ok(crate::security::user_role::derive_stable_user_uuid(&claims_user))
}

/// The caller's sessions, grouped by bot.
fn owned_sessions(state: &AppState, user_id: Uuid) -> Result<BTreeMap<Uuid, Vec<OwnedSession>>, Response> {
    use diesel::sql_types::{BigInt, Text};
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        bot_id: String,
        #[diesel(sql_type = Text)]
        title: String,
        #[diesel(sql_type = BigInt)]
        last_activity: i64,
    }
    let mut conn = state.conn.get().map_err(|e| internal(&e))?;
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id::text AS id, bot_id::text AS bot_id, COALESCE(title, '') AS title, \
         EXTRACT(EPOCH FROM last_activity)::bigint AS last_activity \
         FROM user_sessions WHERE user_id::text = $1 ORDER BY last_activity DESC LIMIT $2",
    )
    .bind::<Text, _>(user_id.to_string())
    .bind::<BigInt, _>(MAX_SESSIONS)
    .load(&mut conn)
    .map_err(|e| internal(&e))?;
    let mut by_bot: BTreeMap<Uuid, Vec<OwnedSession>> = BTreeMap::new();
    for row in rows {
        let (Ok(id), Ok(bot_id)) = (Uuid::parse_str(row.id.trim()), Uuid::parse_str(row.bot_id.trim())) else {
            continue;
        };
        by_bot.entry(bot_id).or_default().push(OwnedSession {
            id,
            bot_id,
            title: row.title,
            last_activity: row.last_activity,
        });
    }
    Ok(by_bot)
}

fn location(state: &AppState, bot_id: Uuid) -> Result<botbasic_core::utils::BotDriveLocation, Response> {
    let mut conn = state.conn.get().map_err(|e| internal(&e))?;
    Ok(botbasic_core::utils::bot_drive_location_for(&mut conn, bot_id))
}

/// RFC 3339 when the store's date can be read, the store's text otherwise.
fn normalized_time(raw: Option<String>) -> Option<String> {
    let raw = raw?;
    let parsed = chrono::DateTime::parse_from_rfc3339(&raw).or_else(|_| chrono::DateTime::parse_from_rfc2822(&raw));
    Some(parsed.map(|time| time.with_timezone(&chrono::Utc).to_rfc3339()).unwrap_or(raw))
}

/// `GET /api/agent/drive/conversations`.
pub async fn list_conversation_folders(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let user_id = match caller(&headers) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };
    let by_bot = match owned_sessions(&state, user_id) {
        Ok(by_bot) => by_bot,
        Err(response) => return response,
    };
    let Some(drive) = state.drive.clone() else {
        return refuse(StatusCode::SERVICE_UNAVAILABLE, "drive_unavailable");
    };
    let mut found: Vec<(String, String, OwnedFolder)> = Vec::new();
    for (bot_id, sessions) in &by_bot {
        let place = match location(&state, *bot_id) {
            Ok(place) => place,
            Err(response) => return response,
        };
        let prefix = place.key_for(&format!("{CONVERSATIONS}/"));
        let objects = match drive.list_objects_with_metadata(&place.bucket, Some(&prefix)).await {
            Ok(objects) => objects,
            Err(e) => {
                botlog::safe_warn!("agent drive files: listing conversations of bot {bot_id} failed: {e}");
                continue;
            }
        };
        let relative: Vec<(String, u64)> = objects
            .into_iter()
            .filter_map(|object| object.key.strip_prefix(&prefix).map(|rest| (rest.to_string(), object.size)))
            .collect();
        for folder in group(&relative, sessions) {
            found.push((place.bucket.clone(), place.key_prefix.clone(), folder));
        }
    }
    found.sort_by(|left, right| right.2.owner.last_activity.cmp(&left.2.owner.last_activity));
    let mut budget = MAX_LISTED_FILES;
    let mut out: Vec<Value> = Vec::with_capacity(found.len());
    for (bucket, key_prefix, folder) in found {
        let mut files: Vec<(Option<String>, Value)> = Vec::with_capacity(folder.files.len());
        for file in folder.files {
            let modified = if budget > 0 {
                budget -= 1;
                let key = format!("{key_prefix}{}", file.path);
                match drive.get_object_metadata(&bucket, &key).await {
                    Ok(Some(meta)) => normalized_time(meta.last_modified),
                    _ => None,
                }
            } else {
                None
            };
            let entry = json!({ "name": file.name, "path": file.path, "size": file.size, "modified": modified });
            files.push((modified, entry));
        }
        // RFC 3339 in UTC sorts as text; unknown times go last.
        files.sort_by(|left, right| right.0.cmp(&left.0));
        out.push(json!({
            "folder": folder.folder,
            "conversation_id": folder.owner.conversation_id,
            "title": folder.owner.title,
            "files": files.into_iter().map(|(_, entry)| entry).collect::<Vec<Value>>(),
        }));
    }
    Json(json!({ "folders": out })).into_response()
}

/// `filename*=` value of RFC 6266 / 5987 (UTF-8, percent-encoded).
fn disposition(name: &str) -> String {
    format!("inline; filename*=UTF-8''{}", urlencoding::encode(name))
}

/// Types a browser would run scripts in when opened directly.
fn is_active_content(name: &str) -> bool {
    let extension = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(extension.as_str(), "html" | "htm" | "svg" | "xml" | "xhtml")
}

/// `GET /api/agent/drive/file?path=…&raw=1` (or `&preview=1`).
pub async fn conversation_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let user_id = match caller(&headers) {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };
    let Some(path) = params.get("path").map(String::as_str) else {
        return refuse(StatusCode::BAD_REQUEST, "path_missing");
    };
    let Ok(folder) = validate_path(path) else {
        return refuse(StatusCode::BAD_REQUEST, "invalid_path");
    };
    let by_bot = match owned_sessions(&state, user_id) {
        Ok(by_bot) => by_bot,
        Err(response) => return response,
    };
    let Some(bot_id) = by_bot
        .iter()
        .find(|(_, sessions)| owner_of(folder, sessions).is_some())
        .map(|(bot_id, _)| *bot_id)
    else {
        return refuse(StatusCode::NOT_FOUND, "not_found");
    };
    let Some(drive) = state.drive.clone() else {
        return refuse(StatusCode::SERVICE_UNAVAILABLE, "drive_unavailable");
    };
    let place = match location(&state, bot_id) {
        Ok(place) => place,
        Err(response) => return response,
    };
    let key = place.key_for(path);
    match drive.get_object_metadata(&place.bucket, &key).await {
        Ok(Some(meta)) if meta.size > MAX_SERVED_BYTES => return refuse(StatusCode::PAYLOAD_TOO_LARGE, "too_large"),
        Ok(Some(_)) => {}
        _ => return refuse(StatusCode::NOT_FOUND, "not_found"),
    }
    let bytes = match drive.get_object(&place.bucket, &key).await {
        Ok(bytes) => bytes,
        Err(e) => {
            botlog::safe_warn!("agent drive files: reading a conversation file failed: {e}");
            return refuse(StatusCode::NOT_FOUND, "not_found");
        }
    };
    let name = path.rsplit('/').next().unwrap_or("file").to_string();
    if params.contains_key("preview") {
        return match botcore::kb::document_processor::office_html(&name, &bytes) {
            Some(Ok(html)) => (
                [
                    (header::CONTENT_TYPE, "text/html; charset=utf-8".to_string()),
                    (
                        header::CONTENT_SECURITY_POLICY,
                        "default-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'".to_string(),
                    ),
                    (header::CONTENT_DISPOSITION, disposition(&format!("{name}.html"))),
                    (header::CACHE_CONTROL, "private, max-age=60".to_string()),
                    (header::X_FRAME_OPTIONS, "SAMEORIGIN".to_string()),
                ],
                html,
            )
                .into_response(),
            Some(Err(why)) => {
                botlog::safe_warn!("agent drive files: preview failed: {why}");
                refuse(StatusCode::BAD_REQUEST, "preview_failed")
            }
            None => refuse(StatusCode::UNSUPPORTED_MEDIA_TYPE, "no_preview"),
        };
    }
    let mut extra: Vec<(HeaderName, String)> = vec![
        (header::CONTENT_TYPE, drive_bridge::content_type(&name).to_string()),
        (header::CONTENT_DISPOSITION, disposition(&name)),
        (header::CACHE_CONTROL, "private, max-age=60".to_string()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
    ];
    if is_active_content(&name) {
        extra.push((header::CONTENT_SECURITY_POLICY, "sandbox; default-src 'none'; style-src 'unsafe-inline'; img-src data:".to_string()));
    }
    let mut response = bytes.into_response();
    for (header_name, value) in extra {
        if let Ok(value) = value.parse() {
            response.headers_mut().insert(header_name, value);
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::{disposition, is_active_content, normalized_time};

    #[test]
    fn the_file_name_is_sent_percent_encoded() {
        assert_eq!(disposition("memorando vacinação.docx"), "inline; filename*=UTF-8''memorando%20vacina%C3%A7%C3%A3o.docx");
        assert_eq!(disposition("a\"b\r\n.txt"), "inline; filename*=UTF-8''a%22b%0D%0A.txt");
    }

    #[test]
    fn pages_that_could_run_scripts_are_sandboxed() {
        assert!(is_active_content("x.HTML"));
        assert!(is_active_content("d.svg"));
        assert!(!is_active_content("r.pdf"));
    }

    #[test]
    fn store_times_become_rfc3339() {
        assert_eq!(
            normalized_time(Some("Tue, 24 Nov 2026 10:00:00 GMT".to_string())).as_deref(),
            Some("2026-11-24T10:00:00+00:00")
        );
        assert_eq!(normalized_time(None), None);
    }
}
