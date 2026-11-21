use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use botcore::shared::state::AppState;
use log::{error, warn};
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
    /// Suite/cloud token. When valid it determines the identity; the
    /// `user_id` query parameter alone is never trusted for an authenticated
    /// session (a stale client id used to scope Drive and files to a guest).
    /// The desktop shell sends the token as `token=`; serde alias accepts both.
    #[serde(alias = "token")]
    pub access_token: Option<String>,
}

/// Treats empty or whitespace-only strings as absent.
fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
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

/// The signed-in user behind the handshake. The cookie can hold a token from
/// an earlier run that no longer resolves while the query carries the fresh
/// one: each is tried in turn, so a stale cookie never turns the owner into
/// a guest (whose chat then finds none of the owner's connected agents).
///
/// Accepts a resolver closure so unit tests can substitute a fake without
/// touching the global session cache.
fn handshake_user_with<F>(cookie: Option<&str>, query: Option<&str>, resolver: F) -> Option<String>
where
    F: Fn(&str) -> Option<(String, Vec<String>)>,
{
    [cookie, query]
        .into_iter()
        .flatten()
        .filter(|t| !t.trim().is_empty())
        .find_map(|t| resolver(t))
        .map(|(id, _)| id)
}

fn handshake_user(cookie: Option<&str>, query: Option<&str>) -> Option<String> {
    handshake_user_with(cookie, query, crate::main_module::routes::anonymous_auth::resolve_token_user)
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let session_id = non_blank(params.session_id)
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::new_v4);
    let cookie_token = headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                let name = parts.next()?;
                let value = parts.next()?;
                if name == "gb-access-token" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        });
    let query_token = non_blank(params.access_token);
    let had_token = cookie_token.is_some() || query_token.is_some();
    let token_user = handshake_user(cookie_token.as_deref(), query_token.as_deref());
    let client_user_id = non_blank(params.user_id);

    let (user_id, identity_source) = if let Some(ref uid) = token_user {
        (crate::security::user_role::derive_stable_user_uuid(uid), "token")
    } else if let Some(ref cid) = client_user_id {
        (crate::security::user_role::derive_stable_user_uuid(cid), "client-supplied id")
    } else {
        (Uuid::new_v4(), "new guest")
    };

    if had_token && token_user.is_none() {
        botlog::safe_warn!(
            "WS handshake: token supplied but did not resolve (session {})",
            session_id
        );
    }
    botlog::safe_info!(
        "WS handshake: identity={identity_source}, user={user_id}, session={session_id}"
    );
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
                    botlog::safe_warn!(
                        "WS: '{}' is a console stream, not a bot (use /api/attendance/ws)",
                        bot_name
                    );
                } else {
                    botlog::safe_warn!("WS: no bot named '{}' (session {})", bot_name, session_id);
                }
                return bot_not_found_response(&bot_name);
            }
            Err(BotAccessDenial::AuthRequired) => {
                botlog::safe_warn!(
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
    botlog::safe_info!(
        "WebSocket: bot={}, session={}, user={}",
        bot_name,
        session_id,
        user_id
    );
    // A 50 MB attachment arrives as about 67 MB of base64 in one message; the
    // default limits (16 MiB per frame) dropped anything above about 11 MB.
    ws.max_message_size(96 * 1024 * 1024)
        .max_frame_size(96 * 1024 * 1024)
        .on_upgrade(move |socket| {
        super::session::handle_ws(socket, state, session_id, user_id, bot_uuid, bot_name)
    })
    .into_response()
}

pub async fn websocket_handler_with_bot(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(bot_name): axum::extract::Path<String>,
    Query(mut params): Query<WsQuery>,
    headers: axum::http::HeaderMap,
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

    // Normalise blank values before the access check; websocket_handler will
    // redo identity resolution from the cleaned params (identity is resolved
    // twice: once here for the access check, once inside websocket_handler).
    params.user_id = non_blank(params.user_id.take());
    params.access_token = non_blank(params.access_token.take());

    let cookie_token = headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|pair| {
                let mut parts = pair.trim().splitn(2, '=');
                let name = parts.next()?;
                let value = parts.next()?;
                if name == "gb-access-token" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        });
    let token_user = handshake_user(cookie_token.as_deref(), params.access_token.as_deref());
    let user_id = token_user
        .as_deref()
        .or(params.user_id.as_deref())
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
                    botlog::safe_warn!(
                        "WS: no bot named '{}' (session {})",
                        bot_name,
                        params.session_id.as_deref().unwrap_or("-")
                    );
                }
                return bot_not_found_response(&bot_name);
            }
            Err(BotAccessDenial::AuthRequired) => {
                botlog::safe_warn!(
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

    websocket_handler(ws, State(state), Query(params), headers).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_alias_deserialises() {
        let json = r#"{"token":"gb_abc123","session_id":"","user_id":""}"#;
        let parsed: WsQuery = serde_json::from_str(json).expect("deserialise WsQuery with token alias");
        assert_eq!(parsed.access_token.as_deref(), Some("gb_abc123"));
    }

    #[test]
    fn test_access_token_field_still_works() {
        let json = r#"{"access_token":"gb_xyz"}"#;
        let parsed: WsQuery = serde_json::from_str(json).expect("deserialise WsQuery");
        assert_eq!(parsed.access_token.as_deref(), Some("gb_xyz"));
    }

    #[test]
    fn test_non_blank_filters_empty() {
        assert!(non_blank(Some(String::new())).is_none());
        assert!(non_blank(Some("  ".to_string())).is_none());
        assert!(non_blank(None).is_none());
        assert_eq!(non_blank(Some("abc".to_string())).as_deref(), Some("abc"));
    }

    #[test]
    fn test_handshake_user_precedence() {
        // Stale cookie (returns None), valid query token -> query wins.
        let result = handshake_user_with(
            Some("stale_cookie"),
            Some("valid_query"),
            |token| {
                if token == "valid_query" {
                    Some(("user-123".to_string(), vec![]))
                } else {
                    None
                }
            },
        );
        assert_eq!(result.as_deref(), Some("user-123"));
    }

    #[test]
    fn test_handshake_user_cookie_wins_when_valid() {
        let result = handshake_user_with(
            Some("good_cookie"),
            Some("good_query"),
            |token| {
                if token == "good_cookie" {
                    Some(("cookie-user".to_string(), vec![]))
                } else if token == "good_query" {
                    Some(("query-user".to_string(), vec![]))
                } else {
                    None
                }
            },
        );
        assert_eq!(result.as_deref(), Some("cookie-user"));
    }

    #[test]
    fn test_handshake_user_empty_token_is_absent() {
        let result = handshake_user_with(Some(""), Some("   "), |_| {
            panic!("resolver should not be called for blank tokens");
        });
        assert!(result.is_none());
    }

    #[test]
    fn test_handshake_user_none_tokens() {
        let result = handshake_user_with(None, None, |_| panic!("should not be called"));
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_user_id_does_not_derive_guest() {
        let constant_guest = crate::security::user_role::derive_stable_user_uuid("");
        assert!(non_blank(Some(String::new())).is_none());
        let uid = crate::security::user_role::derive_stable_user_uuid("real-user");
        assert_ne!(uid, constant_guest);
    }
}
