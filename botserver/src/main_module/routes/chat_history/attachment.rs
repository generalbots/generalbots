use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use botcore::shared::state::AppState;
use diesel::prelude::*;
use uuid::Uuid;

use super::HandlerError;
use crate::main_module::routes::anonymous_auth::{resolve_chat_user_uuid, resolve_token_user};

fn escape_preview_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// `GET /api/chat/attachments/preview?bot=&name=&access_token=`
///
/// User uploads are staged in the bot's Drive `inbox/`; the chat iframe
/// cannot attach an Authorization header, so the same suite token accepted by
/// `/api/agent/vm/file` is accepted as a query parameter. Office packages are
/// converted to sanitized, script-free HTML with `office_html`.
pub async fn handle_attachment_preview(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    request: axum::extract::Request,
) -> Result<axum::response::Response, HandlerError> {
    let token = params
        .get("access_token")
        .cloned()
        .or_else(|| {
            request
                .headers()
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .map(str::to_string)
        })
        .unwrap_or_default();
    let (user_raw, _roles) = resolve_token_user(&token).ok_or((
        StatusCode::UNAUTHORIZED,
        "Authentication required".to_string(),
    ))?;
    let user_uuid = resolve_chat_user_uuid(&user_raw);

    let name = params.get("name").cloned().unwrap_or_default();
    let bot_name = params.get("bot").cloned().unwrap_or_default();
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || bot_name.is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "invalid attachment name".to_string(),
        ));
    }

    let mut conn = state.conn.get().map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB pool: {error}"),
        )
    })?;
    let bot_id: Uuid = botschema::bots::table
        .filter(botschema::bots::name.eq(&bot_name))
        .select(botschema::bots::id)
        .first(&mut *conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Bot not found".to_string()))?;

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
    }
    let sessions = diesel::sql_query(
        "SELECT count(*) AS total FROM user_sessions WHERE user_id = $1 AND bot_id = $2",
    )
    // `user_sessions.user_id` stores the UUID as text.
    .bind::<diesel::sql_types::Text, _>(user_uuid.to_string())
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<CountRow>(&mut *conn)
    .map_err(|error| {
        log::error!("attachment preview session lookup failed: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not verify the conversation".to_string(),
        )
    })?
    .total;
    if sessions == 0 {
        return Err((StatusCode::FORBIDDEN, "No access to this bot".to_string()));
    }

    let location = botbasic_core::utils::bot_drive_location_for_name(&mut conn, &bot_name);
    let key = location.key_for(&format!("inbox/{name}"));
    let drive = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Drive is not configured".to_string(),
    ))?;
    let bytes = drive
        .get_object(&location.bucket, &key)
        .await
        .map_err(|error| {
            log::warn!("attachment preview: {key} not found: {error}");
            (StatusCode::NOT_FOUND, "Attachment not found".to_string())
        })?;

    let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
    let (content_type, body, sandboxed) = match extension.as_str() {
        "docx" | "xlsx" | "pptx" => {
            let html = botcore::kb::document_processor::office_html(&name, &bytes)
                .and_then(|result| result.ok())
                .ok_or((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "This Office file could not be converted for preview".to_string(),
                ))?;
            ("text/html; charset=utf-8", html.into_bytes(), true)
        }
        "txt" | "md" | "csv" | "json" | "log" => {
            // Plain-text uploads get the same preview surface as documents;
            // the body is escaped and capped so a large log cannot bloat the
            // conversation (#1693).
            let text: String = String::from_utf8_lossy(&bytes)
                .chars()
                .take(200_000)
                .collect();
            let escaped = escape_preview_text(&text);
            let html = format!(
                "<!doctype html><html><head><meta charset=\"utf-8\">\
                 <style>body{{margin:0;padding:12px;font:13px/1.5 ui-monospace,Menlo,monospace;\
                 white-space:pre-wrap;word-break:break-word;color:#0f172a;background:#fff}}</style>\
                 </head><body>{escaped}</body></html>"
            );
            ("text/html; charset=utf-8", html.into_bytes(), true)
        }
        "pdf" => ("application/pdf", bytes, false),
        "png" => ("image/png", bytes, false),
        "jpg" | "jpeg" => ("image/jpeg", bytes, false),
        "gif" => ("image/gif", bytes, false),
        "webp" => ("image/webp", bytes, false),
        _ => {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "No inline preview for this file type".to_string(),
            ))
        }
    };

    let csp = if sandboxed {
        "default-src 'none'; style-src 'unsafe-inline'; img-src data:; frame-ancestors 'self'"
    } else {
        "default-src 'none'; img-src 'self' data:; frame-ancestors 'self'"
    };
    axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, content_type)
        .header(axum::http::header::CACHE_CONTROL, "no-store")
        .header(axum::http::header::X_FRAME_OPTIONS, "SAMEORIGIN")
        .header(axum::http::header::CONTENT_SECURITY_POLICY, csp)
        .body(axum::body::Body::from(body))
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("response: {error}"),
            )
        })
}
