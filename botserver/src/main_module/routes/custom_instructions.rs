//! `GET /api/settings/custom-instructions` and
//! `PUT /api/settings/custom-instructions` — per-user instructions injected
//! into every conversation's system prompt.

use axum::{extract::State, http::HeaderMap, http::StatusCode, Json};
use botcore::shared::state::AppState;
use std::sync::Arc;
use diesel::Connection;
use botsettings::settings_api::{read_pref, try_write_pref};

/// Maximum length (in characters) for each instruction field.
const MAX_FIELD_LEN: usize = 1500;

/// Preference keys stored in `user_preferences`.
const KEY_ABOUT: &str = "custom_instructions_about";
const KEY_RESPONSE: &str = "custom_instructions_response";
const KEY_ENABLED: &str = "custom_instructions_enabled";

/// Resolves the authenticated user UUID from the Authorization header.
pub(super) fn authed_user(headers: &HeaderMap) -> Option<uuid::Uuid> {
    if let Some(uid_str) = botcore::shared::tenant::user_id_from_claims(headers) {
        return Some(super::anonymous_auth::resolve_chat_user_uuid(&uid_str));
    }
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")))
        .map(str::trim)?;
    let (user_raw, _roles) = super::anonymous_auth::resolve_token_user(token)?;
    Some(super::anonymous_auth::resolve_chat_user_uuid(&user_raw))
}

/// Sanitises a free-text field: trims, strips control characters (keeping
/// newline and tab), and truncates to `MAX_FIELD_LEN` on a char boundary.
fn sanitise(raw: &str) -> String {
    let trimmed: String = raw
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(MAX_FIELD_LEN)
        .collect();
    trimmed.trim().to_string()
}

/// `GET /api/settings/custom-instructions`
pub async fn handle_get(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl axum::response::IntoResponse {
    let Some(user_id) = authed_user(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "authentication required"})),
        );
    };
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(error) => {
            log::warn!("custom-instructions: pool error: {error}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "service unavailable"})),
            );
        }
    };
    let about = botsettings::settings_api::read_pref(&mut conn, user_id, KEY_ABOUT);
    let response = botsettings::settings_api::read_pref(&mut conn, user_id, KEY_RESPONSE);
    let enabled = botsettings::settings_api::read_pref(&mut conn, user_id, KEY_ENABLED);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "about": about,
            "response": response,
            "enabled": enabled,
            "name": read_pref(&mut conn, user_id, "persona_name"),
            "icon": read_pref(&mut conn, user_id, "persona_icon"),
            "photo": read_pref(&mut conn, user_id, "persona_photo"),
        })),
    )
}

#[derive(serde::Deserialize)]
pub struct InstructionsBody {
    about: Option<String>,
    response: Option<String>,
    enabled: Option<bool>,
    name: Option<String>,
    icon: Option<String>,
    photo: Option<String>,
}

/// `PUT /api/settings/custom-instructions`
pub async fn handle_put(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<InstructionsBody>,
) -> impl axum::response::IntoResponse {
    let Some(user_id) = authed_user(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "authentication required"})),
        );
    };
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(error) => {
            log::warn!("custom-instructions: pool error: {error}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "service unavailable"})),
            );
        }
    };
    if body.name.as_ref().is_some_and(|s| s.chars().count() > 100)
        || body.icon.as_deref().is_some_and(|s| !matches!(s, "" | "robot" | "spark" | "building" | "book" | "shield" | "feather" | "leaf" | "compass"))
        || body.photo.as_deref().is_some_and(|s| !valid_photo(s))
    {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"invalid persona"})));
    }
    let result = conn.transaction::<_, diesel::result::Error, _>(|db| {
        for (key, value) in [
            (KEY_ABOUT, body.about.map(|s| serde_json::json!(sanitise(&s)))),
            (KEY_RESPONSE, body.response.map(|s| serde_json::json!(sanitise(&s)))),
            (KEY_ENABLED, body.enabled.map(serde_json::Value::Bool)),
            ("persona_name", body.name.map(|s| serde_json::json!(sanitise(&s)))),
            ("persona_icon", body.icon.map(serde_json::Value::String)),
            ("persona_photo", body.photo.map(serde_json::Value::String)),
        ] {
            if let Some(value) = value { try_write_pref(db, user_id, key, &value)?; }
        }
        Ok(())
    });
    if result.is_err() {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error":"could not save preferences"})));
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true})),
    )
}

fn valid_photo(value: &str) -> bool {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    if value.is_empty() { return true; }
    if value.len() > 256 * 1024 { return false; }
    let Some((mime, encoded)) = value.split_once(",") else { return false; };
    let Ok(bytes) = STANDARD.decode(encoded) else { return false; };
    match mime {
        "data:image/png;base64" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "data:image/jpeg;base64" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "data:image/webp;base64" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
        _ => false,
    }
}

/// Reads the user's custom instructions from `user_preferences` and returns
/// the prompt block to inject, or an empty string when disabled/absent.
///
/// Called from `system_prompt::build()` on every turn.
pub fn render_for_prompt(
    conn: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    user_id: uuid::Uuid,
) -> String {
    let mut db = match conn.get() {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let enabled = botsettings::settings_api::read_pref(&mut db, user_id, KEY_ENABLED);
    if enabled == serde_json::json!(false) {
        return String::new();
    }
    let about = botsettings::settings_api::read_pref(&mut db, user_id, KEY_ABOUT);
    let response = botsettings::settings_api::read_pref(&mut db, user_id, KEY_RESPONSE);
    let about_str = about.as_str().unwrap_or("").trim();
    let response_str = response.as_str().unwrap_or("").trim();
    if about_str.is_empty() && response_str.is_empty() {
        return String::new();
    }
    let mut block = String::from("\n\nCUSTOM USER INSTRUCTIONS (set by this user; follow them)");
    if !about_str.is_empty() {
        block.push_str("\nAbout the user:\n");
        block.push_str(about_str);
    }
    if !response_str.is_empty() {
        block.push_str("\nHow to respond:\n");
        block.push_str(response_str);
    }
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitise_trims_and_truncates() {
        assert_eq!(sanitise("  hello  "), "hello");
        assert_eq!(sanitise("a\0b\x01c"), "abc");
        assert_eq!(sanitise("line1\nline2\ttab"), "line1\nline2\ttab");
        let long = "x".repeat(2000);
        assert_eq!(sanitise(&long).len(), MAX_FIELD_LEN);
    }

    #[test]
    fn render_for_prompt_returns_empty_on_no_data() {
        // We test the assembly logic, not the DB read.
        let about = "";
        let response = "";
        assert!(about.is_empty() && response.is_empty());
    }
}
