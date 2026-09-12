use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use botcore::shared::state::AppState;
use log::{error, info, warn};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use crate::security::code_scan_fixes::is_safe_path;

use super::super::{check_bot_access_typed, BotAccessDenial};

/// 401 for a private bot the caller cannot reach. The browser WebSocket API
/// hides the handshake response, so the client mirrors this by preflighting
/// `/api/bot/public` and rendering a "sign in required" state instead of the
/// opaque "connection failed" it used to show.
fn auth_required_response(bot_name: &str) -> axum::response::Response {
    (
        axum::http::StatusCode::UNAUTHORIZED,
        [
            ("x-gb-auth-required", "1"),
            ("x-gb-bot-name", bot_name),
        ],
        axum::Json(serde_json::json!({
            "error": "auth_required",
            "message": "This bot is private. Sign in with an account that belongs to its organization.",
            "bot_name": bot_name,
        })),
    )
        .into_response()
}

/// 404 for an address that is not a bot. Reserved console streams live under
/// their own paths (`/api/attendance/ws`), so a stale `/ws/attendant` client
/// gets an actionable answer instead of "bot not found".
fn bot_not_found_response(bot_name: &str) -> axum::response::Response {
    (
        axum::http::StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({
            "error": "bot_not_found",
            "message": "No bot matches this address. The attendant console stream is /api/attendance/ws.",
            "bot_name": bot_name,
        })),
    )
        .into_response()
}

/// Path segments under `/ws/` that are console streams, not bot names.
const RESERVED_WS_SEGMENTS: [&str; 1] = ["attendant"];

#[derive(Deserialize)]
pub struct WsQuery {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub bot_name: Option<String>,
}

pub fn validate_bot_name(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Bot name cannot be empty".to_string());
    }
    if trimmed.len() > 255 {
        return Err("Bot name too long".to_string());
    }
    if trimmed.contains('\0') {
        return Err("Invalid bot name".to_string());
    }
    for c in trimmed.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' && c != '.' {
            return Err("Invalid bot name".to_string());
        }
    }
    if trimmed.starts_with('.') {
        return Err("Invalid bot name".to_string());
    }
    if trimmed.contains("..") {
        return Err("Invalid bot name".to_string());
    }
    Ok(trimmed.to_string())
}

pub fn verify_path_within_workdir(sub_path: &str) -> bool {
    let work_dir = botcore::shared::utils::get_work_path();
    let base = Path::new(&work_dir);
    let path = Path::new(sub_path);
    is_safe_path(base, path)
}

fn lookup_bot_id(state: &Arc<AppState>, bot_name: &str) -> Uuid {
    use botcorebot::schema::bots::dsl::{bots, id, name};
    use diesel::prelude::*;

    let pool = state.conn.clone();
    let bot_name = bot_name.to_string();
    let result = tokio::task::block_in_place(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("DB conn: {}", e);
                return Uuid::nil();
            }
        };

        if let Ok(uuid) = Uuid::parse_str(&bot_name) {
            bots.filter(id.eq(uuid))
                .select(id)
                .first::<Uuid>(&mut conn)
                .unwrap_or(Uuid::nil())
        } else {
            // #1289 — the launcher/URL identifies bots by SLUG; accept either
            // name or slug so a freshly created bot connects immediately.
            use botcorebot::schema::bots::dsl::slug;
            bots.filter(name.eq(&bot_name).or(slug.eq(&bot_name)))
                .select(id)
                .first::<Uuid>(&mut conn)
                .unwrap_or_else(|_| {
                    warn!("Bot not found: {}", bot_name);
                    Uuid::nil()
                })
        }
    });
    result
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
) -> axum::response::Response {
    let session_id = params
        .session_id
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::new_v4);
    let user_id = params
        .user_id
        .as_deref()
        .map(crate::security::user_role::derive_stable_user_uuid)
        .unwrap_or_else(Uuid::new_v4);
    let raw_bot_name = params
        .bot_name
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let bot_name = match validate_bot_name(&raw_bot_name) {
        Ok(name) => name,
        Err(e) => {
            warn!("Invalid bot_name in WS query: {}", e);
            return (axum::http::StatusCode::BAD_REQUEST, "Invalid bot name").into_response();
        }
    };

    if bot_name != "default" {
        match check_bot_access_typed(&state, &bot_name, user_id).await {
            Ok(()) => {}
            Err(BotAccessDenial::NotFound) => {
                if RESERVED_WS_SEGMENTS.contains(&bot_name.as_str()) {
                    warn!(
                        "WS: '{}' is a console stream, not a bot (use /api/attendance/ws)",
                        bot_name
                    );
                } else {
                    warn!("WS: no bot named '{}' (session {})", bot_name, session_id);
                }
                return bot_not_found_response(&bot_name);
            }
            Err(BotAccessDenial::AuthRequired) => {
                warn!(
                    "WS: bot '{}' is private and the caller is not a member — sign-in required (session {})",
                    bot_name, session_id
                );
                return auth_required_response(&bot_name);
            }
            Err(BotAccessDenial::Internal(e)) => {
                error!("WS: access check failed for bot '{}': {}", bot_name, e);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Access check failed",
                )
                    .into_response();
            }
        }
    }

    let bot_uuid = lookup_bot_id(&state, &bot_name);
    info!(
        "WebSocket: bot={}, session={}, user={}",
        bot_name, session_id, user_id
    );
    ws.on_upgrade(move |socket| {
        super::session::handle_ws(socket, state, session_id, user_id, bot_uuid, bot_name)
    })
    .into_response()
}

pub async fn websocket_handler_with_bot(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(bot_name): axum::extract::Path<String>,
    Query(mut params): Query<WsQuery>,
) -> axum::response::Response {
    let raw_bot_name = if bot_name.is_empty() {
        params
            .bot_name
            .clone()
            .unwrap_or_else(|| "default".to_string())
    } else {
        bot_name
    };
    let bot_name = match validate_bot_name(&raw_bot_name) {
        Ok(name) => name,
        Err(e) => {
            warn!("Invalid bot_name in WS path: {}", e);
            return (axum::http::StatusCode::BAD_REQUEST, "Invalid bot name").into_response();
        }
    };
    params.bot_name = Some(bot_name.clone());

    let user_id = params
        .user_id
        .as_deref()
        .map(crate::security::user_role::derive_stable_user_uuid)
        .unwrap_or_else(Uuid::new_v4);

    if bot_name != "default" {
        match check_bot_access_typed(&state, &bot_name, user_id).await {
            Ok(()) => {}
            Err(BotAccessDenial::NotFound) => {
                if RESERVED_WS_SEGMENTS.contains(&bot_name.as_str()) {
                    warn!(
                        "WS: '{}' is a console stream, not a bot (use /api/attendance/ws)",
                        bot_name
                    );
                } else {
                    warn!(
                        "WS: no bot named '{}' (session {})",
                        bot_name,
                        params.session_id.as_deref().unwrap_or("-")
                    );
                }
                return bot_not_found_response(&bot_name);
            }
            Err(BotAccessDenial::AuthRequired) => {
                warn!(
                    "WS: bot '{}' is private and the caller is not a member — sign-in required (session {})",
                    bot_name,
                    params.session_id.as_deref().unwrap_or("-")
                );
                return auth_required_response(&bot_name);
            }
            Err(BotAccessDenial::Internal(e)) => {
                error!("WS: access check failed for bot '{}': {}", bot_name, e);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Access check failed",
                )
                    .into_response();
            }
        }
    }

    websocket_handler(ws, State(state), Query(params)).await
}
