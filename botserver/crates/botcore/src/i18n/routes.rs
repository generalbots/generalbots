//! Axum routes exposing the available locales, translation catalogues and the
//! caller's persisted locale preference.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::error::error_body;
use super::locale_meta::metadata_all;
use super::translations::get_translations_json;
use super::{available_locales, locale_from_headers, Locale};
use crate::shared::state::AppState;

pub fn configure_i18n_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/i18n/locales", get(handle_get_locales))
        .route(
            "/api/i18n/preference",
            get(handle_get_preference).post(handle_set_preference),
        )
        .route("/api/i18n/:locale", get(handle_get_translations))
}

/// `GET /api/i18n/locales` — the locales the catalogue ships.
///
/// `locales` stays a flat list of codes for the two clients that already read
/// it (`suite/js/i18n.js`, `cloud/js/i18n-lite.js`). `details` carries the
/// presentation metadata a picker needs (#2087/#2096): the native name, the
/// text direction and the CLDR date/number patterns. Both arrays describe the
/// same locales in the same order.
async fn handle_get_locales(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let locales = available_locales();
    let details = metadata_all(&locales);
    Json(serde_json::json!({
        "locales": locales,
        "details": details,
        "default": "en"
    }))
}

async fn handle_get_translations(
    State(_state): State<Arc<AppState>>,
    Path(locale_str): Path<String>,
) -> impl IntoResponse {
    let locale = Locale::new(&locale_str).unwrap_or_default();

    let translations = get_translations_json(&locale);

    Json(serde_json::json!({
        "locale": locale.to_string(),
        "translations": translations
    }))
}

#[derive(Debug, Deserialize)]
struct LocalePreferenceBody {
    locale: String,
}

#[derive(Debug, QueryableByName)]
struct PreferenceRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    value: Option<String>,
}

fn caller_id(headers: &HeaderMap) -> Option<Uuid> {
    crate::shared::tenant::user_id_from_claims(headers).and_then(|raw| Uuid::parse_str(&raw).ok())
}

/// `GET /api/i18n/preference` — the locale this user picked, so a new device or
/// a server-rendered email can use it instead of guessing from the browser.
async fn handle_get_preference(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let locale = locale_from_headers(&headers);

    let Some(user_id) = caller_id(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(error_body(&locale, "error-auth-token-missing")),
        );
    };

    let stored = tokio::task::spawn_blocking(move || {
        let mut conn = state.conn.get().ok()?;
        Some(locale_for_user(&mut conn, user_id).to_string())
    })
    .await
    .unwrap_or(None);

    (
        StatusCode::OK,
        Json(serde_json::json!({ "locale": stored })),
    )
}

/// The locale a user picked in the UI (`user_preferences` key `locale`).
///
/// Background work (emails, notifications, scheduled jobs) resolves the
/// recipient's preference here instead of using the process locale (#2094);
/// the default locale is returned when nothing is stored.
#[must_use]
pub fn locale_for_user(conn: &mut diesel::PgConnection, user_id: Uuid) -> Locale {
    let row: Option<PreferenceRow> = diesel::sql_query(
        "SELECT preference_value::text AS value FROM user_preferences \
         WHERE user_id = $1 AND preference_key = 'locale' LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result(conn)
    .ok();

    row.and_then(|r| r.value)
        .and_then(|raw| serde_json::from_str::<String>(&raw).ok())
        .and_then(|code| Locale::new(code.trim()))
        .unwrap_or_default()
}

/// `POST /api/i18n/preference` — persists the user's locale. Only a locale the
/// catalogue actually ships is accepted, so a typo cannot pin a user to a
/// language with no translations.
async fn handle_set_preference(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<LocalePreferenceBody>,
) -> impl IntoResponse {
    let locale = locale_from_headers(&headers);

    let Some(user_id) = caller_id(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(error_body(&locale, "error-auth-token-missing")),
        );
    };

    let Some(requested) = Locale::new(body.locale.trim()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(error_body(&locale, "error-locale-invalid")),
        );
    };

    let supported = available_locales()
        .iter()
        .any(|code| Locale::new(code).is_some_and(|available| available.matches(&requested)));

    if !supported {
        let mut body = error_body(&locale, "error-locale-unsupported");
        if let Some(map) = body.as_object_mut() {
            map.insert(
                "available".to_string(),
                serde_json::json!(available_locales()),
            );
        }

        return (StatusCode::BAD_REQUEST, Json(body));
    }

    let locale_code = requested.to_string();
    let value = serde_json::Value::String(locale_code.clone()).to_string();

    let saved = tokio::task::spawn_blocking(move || {
        let Ok(mut conn) = state.conn.get() else {
            return false;
        };
        diesel::sql_query(
            "INSERT INTO user_preferences (id, user_id, preference_key, preference_value, created_at, updated_at) \
             VALUES ($1, $2, 'locale', $3::jsonb, NOW(), NOW()) \
             ON CONFLICT (user_id, preference_key) \
             DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(value)
        .execute(&mut conn)
        .is_ok()
    })
    .await
    .unwrap_or(false);

    if !saved {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_body(&locale, "error-locale-persist-failed")),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "locale": locale_code })),
    )
}
