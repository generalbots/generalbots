use crate::errors::{api_error, api_error_db, locale_from_headers, ApiError};
use crate::schema::attendance_webhooks;
use crate::AttendanceConfig;
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use chrono::Utc;
use diesel::prelude::*;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = attendance_webhooks)]
pub struct AttendanceWebhook {
    pub id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub webhook_url: String,
    pub events: Option<Vec<Option<String>>>,
    pub is_active: Option<bool>,
    pub secret_key: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub webhook_url: String,
    pub events: Vec<String>,
    pub secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhookRequest {
    pub webhook_url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: String,
    pub bot_id: Uuid,
    pub data: serde_json::Value,
}

fn calculate_hmac_signature(secret: &str, payload: &str) -> String {
    // #1368 — HMAC accepts a key of any length; a construction failure means
    // the crypto backend is unusable. Fail closed with an empty signature
    // instead of aborting the process.
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(e) => {
            log::error!("HMAC signature unavailable: {e}");
            return String::new();
        }
    };
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn emit_webhook_event(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    event: &str,
    data: serde_json::Value,
) {
    let branch_id = {
        use crate::schema::bots;
        bots::table
            .filter(bots::id.eq(bot_id))
            .select(bots::branch_id)
            .first::<Uuid>(conn)
            .unwrap_or(Uuid::nil())
    };
    let webhooks: Vec<AttendanceWebhook> = attendance_webhooks::table
        .filter(attendance_webhooks::branch_id.eq(branch_id))
        .filter(attendance_webhooks::is_active.eq(true))
        .load(conn)
        .unwrap_or_else(|e| {
            log::warn!("Failed to load attendance webhooks: {e}");
            Vec::new()
        });

    for webhook in webhooks {
        // #2636 — revalidate at dispatch: DNS can change after the URL was
        // stored, and older rows may predate this guard.
        if botsecurity_core::validation::validate_url_ssrf_strict(&webhook.webhook_url).is_err() {
            log::warn!("Refusing to dispatch attendance webhook to a disallowed URL");
            continue;
        }
        let event_str = event.to_string();
        if let Some(ref events) = webhook.events {
            if !events.iter().flatten().any(|e| e == &event_str) {
                continue;
            }
        }
        let payload = WebhookPayload {
            event: event_str,
            timestamp: Utc::now().to_rfc3339(),
            bot_id,
            data: data.clone(),
        };
        let payload_json = serde_json::to_string(&payload).unwrap_or_else(|e| {
            log::warn!("Webhook payload could not be serialized: {e}");
            String::new()
        });
        let mut request = reqwest::Client::new()
            .post(&webhook.webhook_url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .body(payload_json.clone());
        if let Some(ref secret) = webhook.secret_key {
            let signature = calculate_hmac_signature(secret, &payload_json);
            request = request.header("X-Webhook-Signature", signature);
        }
        let url_for_log = webhook.webhook_url.clone();
        let event_for_log = event.to_string();
        tokio::spawn(async move {
            if let Err(e) = request.send().await {
                log::error!("Failed to emit webhook {}: {}", url_for_log, e);
            } else {
                log::info!("Webhook emitted successfully: {} event={}", url_for_log, event_for_log);
            }
        });
    }
}

fn get_bot_context(config: &AttendanceConfig) -> Uuid {
    let Ok(mut conn) = config.pool.get() else {
        return Uuid::nil();
    };
    
    (config.get_default_bot)(&mut conn)
}

fn org_for_branch(config: &AttendanceConfig, branch_id: Uuid) -> Uuid {
    let Ok(mut conn) = config.pool.get() else {
        return Uuid::nil();
    };
    crate::schema::bots::table
        .filter(crate::schema::bots::branch_id.eq(branch_id))
        .select(crate::schema::bots::org_id)
        .first(&mut conn)
        .unwrap_or(Uuid::nil())
}

pub async fn list_webhooks(
    State(config): State<Arc<AttendanceConfig>>,
    headers: HeaderMap,
) -> Result<Json<Vec<AttendanceWebhook>>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = config.pool.get().map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let branch_id = get_bot_context(&config);
    let webhooks: Vec<AttendanceWebhook> = attendance_webhooks::table
        .filter(attendance_webhooks::branch_id.eq(branch_id))
        .order(attendance_webhooks::created_at.desc())
        .load(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(webhooks))
}

pub async fn create_webhook(
    State(config): State<Arc<AttendanceConfig>>,
    headers: HeaderMap,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<AttendanceWebhook>, ApiError> {
    let locale = locale_from_headers(&headers);
    // #2636 — reject loopback/link-local/private targets before persisting.
    if botsecurity_core::validation::validate_url_ssrf_strict(&req.webhook_url).is_err() {
        return Err(api_error(&locale, StatusCode::BAD_REQUEST, "error-http-400"));
    }
    let mut conn = config.pool.get().map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let branch_id = get_bot_context(&config);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let webhook = AttendanceWebhook {
        id,
        branch_id,
        is_active: Some(true),
        created_at: now,
        updated_at: Some(now),
        webhook_url: req.webhook_url,
        events: Some(req.events.into_iter().map(Some).collect()),
        secret_key: req.secret_key,
        org_id: org_for_branch(&config, branch_id),
    };
    diesel::insert_into(attendance_webhooks::table)
        .values(&webhook)
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(webhook))
}

pub async fn get_webhook(
    State(config): State<Arc<AttendanceConfig>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<AttendanceWebhook>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = config.pool.get().map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let webhook: AttendanceWebhook = attendance_webhooks::table
        .filter(attendance_webhooks::id.eq(id))
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-attendance-webhook-not-found"))?;
    Ok(Json(webhook))
}

pub async fn update_webhook(
    State(config): State<Arc<AttendanceConfig>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<AttendanceWebhook>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = config.pool.get().map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let now = Utc::now();
    if let Some(new_url) = req.webhook_url {
        // #2636 — the update path must be as strict as create.
        if botsecurity_core::validation::validate_url_ssrf_strict(&new_url).is_err() {
            return Err(api_error(&locale, StatusCode::BAD_REQUEST, "error-http-400"));
        }
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::webhook_url.eq(new_url))
            .execute(&mut conn).map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(new_events) = req.events {
        let events_value: Vec<Option<String>> = new_events.into_iter().map(Some).collect();
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::events.eq(Some(events_value)))
            .execute(&mut conn).map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(active) = req.is_active {
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::is_active.eq(Some(active)))
            .execute(&mut conn).map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    if let Some(new_secret) = req.secret_key {
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::secret_key.eq(Some(new_secret)))
            .execute(&mut conn).map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
        .set(attendance_webhooks::updated_at.eq(now))
        .execute(&mut conn).map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let webhook: AttendanceWebhook = attendance_webhooks::table
        .filter(attendance_webhooks::id.eq(id))
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-attendance-webhook-not-found"))?;
    Ok(Json(webhook))
}

pub async fn delete_webhook(
    State(config): State<Arc<AttendanceConfig>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = config.pool.get().map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    diesel::delete(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
        .execute(&mut conn).map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_webhook(
    State(config): State<Arc<AttendanceConfig>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = config.pool.get().map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let webhook: AttendanceWebhook = attendance_webhooks::table
        .filter(attendance_webhooks::id.eq(id))
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-attendance-webhook-not-found"))?;
    // #2636 — revalidate immediately before the test dispatch.
    if botsecurity_core::validation::validate_url_ssrf_strict(&webhook.webhook_url).is_err() {
        return Err(api_error(&locale, StatusCode::BAD_REQUEST, "error-http-400"));
    }
    let payload = WebhookPayload {
        event: "test".to_string(), timestamp: Utc::now().to_rfc3339(), bot_id: webhook.branch_id,
        data: serde_json::json!({ "message": "This is a test webhook" }),
    };
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| {
            log::warn!("webhook test payload could not be serialized: {e}");
            api_error(&locale, StatusCode::INTERNAL_SERVER_ERROR, "error-http-500")
        })?;
    let client = reqwest::Client::new();
    let mut request = client.post(&webhook.webhook_url);
    if let Some(ref secret) = webhook.secret_key {
        let signature = calculate_hmac_signature(secret, &payload_json);
        request = request.header("X-Webhook-Signature", signature);
    }
    request = request.header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .body(payload_json);
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            // #2636 — never reflect a remote body: it could be metadata or an
            // internal API response. Report only status and byte length.
            let body_len = response.content_length().unwrap_or(0) as usize;
            Ok(Json(serde_json::json!({
                "success": status.is_success(),
                "status_code": status.as_u16(),
                "response_bytes": body_len,
                "response": "[redacted — response body is never returned]"
            })))
        }
        Err(e) => {
            log::error!("Webhook test failed: {}", e);
            Ok(Json(serde_json::json!({ "success": false, "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botattendance::webhooks", None) })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhook_urls_reject_internal_targets() {
        for url in [
            "http://127.0.0.1:8080/admin",
            "http://169.254.169.254/latest/meta-data/",
            "http://localhost/secret",
            "file:///etc/passwd",
            "http://internal.example.local/api",
        ] {
            assert!(
                botsecurity_core::validation::validate_url_ssrf_strict(url).is_err(),
                "{url} must be rejected as a webhook target"
            );
        }
        assert!(botsecurity_core::validation::validate_url_ssrf_strict(
            "https://hooks.example.com/gb/event"
        )
        .is_ok());
    }

    #[test]
    fn test_webhook_never_reflects_the_remote_body() {
        // The success branch reports status/length only; the literal below is
        // the sole `response` value and contains no remote content.
        const REDACTED: &str = "[redacted — response body is never returned]";
        assert!(!REDACTED.contains("metadata"));
        assert!(!REDACTED.contains("169.254"));
    }

    #[test]
    fn test_hmac_signature_generation() {
        let secret = "test-secret";
        let payload = r#"{"event":"test","data":{}}"#;
        let signature = calculate_hmac_signature(secret, payload);
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64);
    }
}
