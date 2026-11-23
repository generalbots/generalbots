use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt, VerifiedJwt};
use chrono::Utc;
use diesel::prelude::*;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{BindingsBody, CallLog, ChannelBinding};
use crate::schema::{call_logs, channel_bindings};
use crate::ChannelBindingsService;

/// Verified claims of the request. `verify_saas_jwt` pins HS256 and checks the
/// signature, expiry and subject before any claim is exposed; a self-decoded
/// `alg:none` payload used to pass every binding/call gate (#2773).
fn verified_claims(headers: &HeaderMap) -> Option<VerifiedJwt> {
    verify_saas_jwt(bearer_token(headers)?).ok()
}

fn require_verified_jwt(headers: &HeaderMap) -> Result<VerifiedJwt, (StatusCode, String)> {
    verified_claims(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Valid JWT required".to_string(),
        )
    })
}

fn parse_bot_id(raw: &str) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(raw).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid bot identifier".to_string()))
}

fn clean_optional(value: Option<String>, max_len: usize) -> Result<Option<String>, (StatusCode, String)> {
    match value {
        Some(raw) => {
            let trimmed = raw.trim().to_string();
            if trimmed.chars().count() > max_len {
                return Err((StatusCode::BAD_REQUEST, format!("Field exceeds {max_len} characters")));
            }
            Ok(if trimmed.is_empty() { None } else { Some(trimmed) })
        }
        None => Ok(None),
    }
}

fn clean_domains(domains: Vec<String>) -> Vec<String> {
    domains
        .iter()
        .map(|d| d.trim().to_lowercase())
        .filter(|d| !d.is_empty())
        .collect()
}

/// `GET /api/channels/:bot_id/bindings` — current row or empty defaults.
pub async fn get_bindings(
    State(service): State<Arc<ChannelBindingsService>>,
    Path(bot_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, String)> {
    require_verified_jwt(&headers)?;
    let bot = parse_bot_id(&bot_id)?;
    let mut conn = service
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    let binding = channel_bindings::table
        .find(bot)
        .select(ChannelBinding::as_select())
        .first::<ChannelBinding>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query bindings: {e}")))?;

    match binding {
        Some(row) => Ok(Json(serde_json::json!({ "bindings": row }))),
        None => Ok(Json(
            serde_json::json!({ "bindings": ChannelBinding::empty(bot) }),
        )),
    }
}

/// `PUT /api/channels/:bot_id/bindings` — upsert the binding row.
pub async fn put_bindings(
    State(service): State<Arc<ChannelBindingsService>>,
    Path(bot_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<BindingsBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    require_verified_jwt(&headers)?;
    let bot = parse_bot_id(&bot_id)?;
    let phone_default = clean_optional(body.phone_default, 40)?;
    let whatsapp_number = clean_optional(body.whatsapp_number, 40)?;
    let telegram_username = clean_optional(body.telegram_username, 80)?;
    let domains = clean_domains(body.domains);
    let now = Utc::now();

    let row = ChannelBinding {
        bot_id: bot,
        phone_default: phone_default.clone(),
        whatsapp_number: whatsapp_number.clone(),
        telegram_username: telegram_username.clone(),
        domains: serde_json::json!(domains),
        updated_at: now,
    };

    let mut conn = service
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    diesel::insert_into(channel_bindings::table)
        .values(&row)
        .on_conflict(channel_bindings::bot_id)
        .do_update()
        .set((
            channel_bindings::phone_default.eq(phone_default),
            channel_bindings::whatsapp_number.eq(whatsapp_number),
            channel_bindings::telegram_username.eq(telegram_username),
            channel_bindings::domains.eq(serde_json::json!(domains)),
            channel_bindings::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Upsert bindings: {e}")))?;

    tracing::info!("Channel bindings saved for bot {bot}");
    Ok(Json(serde_json::json!({ "saved": true, "bindings": row })))
}

/// `GET /api/channels/:bot_id/calls` — last 50 call logs for the bot.
pub async fn list_calls(
    State(service): State<Arc<ChannelBindingsService>>,
    Path(bot_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, String)> {
    require_verified_jwt(&headers)?;
    let bot = parse_bot_id(&bot_id)?;
    let mut conn = service
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    let logs = call_logs::table
        .filter(call_logs::bot_id.eq(bot))
        .order(call_logs::created_at.desc())
        .limit(50)
        .select(CallLog::as_select())
        .load::<CallLog>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query calls: {e}")))?;

    Ok(Json(serde_json::json!({ "calls": logs })))
}

pub fn configure_routes() -> Router<Arc<ChannelBindingsService>> {
    Router::new()
        .route(
            "/api/channels/:bot_id/bindings",
            get(get_bindings).put(put_bindings),
        )
        .route("/api/channels/:bot_id/calls", get(list_calls))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(auth: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert("authorization", HeaderValue::from_str(auth).expect("header"));
        map
    }

    #[test]
    fn missing_token_is_unauthorized() {
        assert!(verified_claims(&HeaderMap::new()).is_none());
        assert_eq!(
            require_verified_jwt(&HeaderMap::new()).map(|_| ()).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn forged_admin_payload_is_denied() {
        // `alg:none` payload claiming an admin email and role; the hand-rolled
        // decoder accepted any decodable JSON and opened the bindings/calls
        // endpoints to it (#2773).
        let forged = "eyJhbGciOiJub25lIn0.eyJlbWFpbCI6ImFkbWluQGxvY2FsaG9zdCIsInJvbGUiOiJhZG1pbiIsInN1YiI6IngiLCJleHAiOjQxMDI0NDQ4MDB9.";
        let map = headers(&format!("Bearer {forged}"));
        assert!(verified_claims(&map).is_none());
        assert_eq!(
            require_verified_jwt(&map).map(|_| ()).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }
}
