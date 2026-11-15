//! `GET /api/appearance` and `PUT /api/appearance` — read and change the
//! installation accent colour after the setup wizard.

use axum::{extract::State, http::HeaderMap, http::StatusCode, Json};
use botcore::shared::state::AppState;
use std::sync::Arc;

fn caller_id(headers: &HeaderMap) -> Option<uuid::Uuid> {
    botcore::shared::tenant::user_id_from_claims(headers)
        .and_then(|raw| uuid::Uuid::parse_str(&raw).ok())
}

/// `GET /api/appearance` — `{"accent": "#rrggbb" | null}`.
pub async fn handle_get_appearance(headers: HeaderMap) -> impl axum::response::IntoResponse {
    if caller_id(&headers).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "authentication required"})),
        );
    }
    let accent = botcore::product::appearance::installation_accent();
    (StatusCode::OK, Json(serde_json::json!({ "accent": accent })))
}

#[derive(serde::Deserialize)]
pub struct AccentBody {
    accent: Option<String>,
}

/// `PUT /api/appearance` — body `{"accent": "#rrggbb"}` or `{"accent": null}`.
pub async fn handle_put_appearance(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<AccentBody>,
) -> impl axum::response::IntoResponse {
    if caller_id(&headers).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "authentication required"})),
        );
    }
    let Some(stack) = botcore::product::appearance::stack_path() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "stack path not configured"})),
        );
    };
    match body.accent {
        Some(raw) => match botcore::product::appearance::write_accent(&stack, &raw) {
            Ok(colour) => (
                StatusCode::OK,
                Json(serde_json::json!({ "accent": colour })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(
                    crate::security::error_sanitizer::log_and_sanitize_json(
                        &e, "appearance", None,
                    ),
                ),
            ),
        },
        None => match botcore::product::appearance::clear_accent(&stack) {
            Ok(()) => (
                StatusCode::OK,
                Json(serde_json::json!({ "accent": serde_json::Value::Null })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    crate::security::error_sanitizer::log_and_sanitize_json(
                        &e, "appearance", None,
                    ),
                ),
            ),
        },
    }
}
