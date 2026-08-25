use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{BindingsBody, CallLog, ChannelBinding};
use crate::schema::{call_logs, channel_bindings};
use crate::ChannelBindingsService;


fn b64_decode_flexible(input: &str) -> Option<Vec<u8>> {
    let mut normalized = String::with_capacity(input.len());
    for c in input.trim().chars() {
        match c {
            '-' => normalized.push('+'),
            '_' => normalized.push('/'),
            c if !c.is_whitespace() => normalized.push(c),
            _ => {}
        }
    }
    if normalized.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(normalized.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for b in normalized.bytes() {
        if b == b'=' {
            break;
        }
        let v = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

fn jwt_claims(headers: &HeaderMap) -> Option<Value> {
    let token = bearer_token(headers)?;
    let payload = token.split('.').nth(1)?;
    let decoded = b64_decode_flexible(payload)?;
    serde_json::from_slice(&decoded).ok()
}

fn require_jwt(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    match jwt_claims(headers) {
        Some(_) => Ok(()),
        None => Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string())),
    }
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
    require_jwt(&headers)?;
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
    require_jwt(&headers)?;
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
    require_jwt(&headers)?;
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
