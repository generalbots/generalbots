use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::schema::attendance_webhooks;
use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceWebhook {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub webhook_url: String,
    pub events: Vec<String>,
    pub is_active: bool,
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

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    use diesel::prelude::*;
    use crate::core::shared::schema::bots;

    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = crate::core::bot::get_default_bot(&mut conn);
    
    let org_id = bots::table
        .filter(bots::id.eq(bot_id))
        .select(bots::org_id)
        .first::<Option<Uuid>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or(Uuid::nil());

    (org_id, bot_id)
}

pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AttendanceWebhook>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let webhooks: Vec<AttendanceWebhook> = attendance_webhooks::table
        .filter(attendance_webhooks::org_id.eq(org_id))
        .filter(attendance_webhooks::bot_id.eq(bot_id))
        .order(attendance_webhooks::created_at.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(webhooks))
}

pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<Json<AttendanceWebhook>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let webhook = AttendanceWebhook {
        id,
        org_id,
        bot_id,
        webhook_url: req.webhook_url,
        events: req.events,
        is_active: true,
        secret_key: req.secret_key,
        created_at: now,
        updated_at: Some(now),
    };

    diesel::insert_into(attendance_webhooks::table)
        .values(&webhook)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(webhook))
}

pub async fn get_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AttendanceWebhook>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let webhook: AttendanceWebhook = attendance_webhooks::table
        .filter(attendance_webhooks::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Webhook not found".to_string()))?;

    Ok(Json(webhook))
}

pub async fn update_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<AttendanceWebhook>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    if let Some(webhook_url) = req.webhook_url {
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::webhook_url.eq(webhook_url))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(events) = req.events {
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::events.eq(events))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(is_active) = req.is_active {
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::is_active.eq(is_active))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(secret_key) = req.secret_key {
        diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
            .set(attendance_webhooks::secret_key.eq(Some(secret_key)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    diesel::update(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
        .set(attendance_webhooks::updated_at.eq(Some(now)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_webhook(State(state), Path(id)).await
}

pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(attendance_webhooks::table.filter(attendance_webhooks::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let webhook: AttendanceWebhook = attendance_webhooks::table
        .filter(attendance_webhooks::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Webhook not found".to_string()))?;

    let payload = WebhookPayload {
        event: "test".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        bot_id: webhook.bot_id,
        data: serde_json::json!({ "message": "This is a test webhook" }),
    };

    let payload_json = serde_json::to_string(&payload).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {e}"))
    })?;

    let client = reqwest::Client::new();
    let mut request = client.post(&webhook.webhook_url);

    if let Some(ref secret) = webhook.secret_key {
        use std::time::Duration;
        
        let signature = calculate_hmac_signature(secret, &payload_json);
        request = request.header("X-Webhook-Signature", signature);
    }

    request = request
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(10))
        .body(payload_json);

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            
            Ok(Json(serde_json::json!({
                "success": status.is_success(),
                "status_code": status.as_u16(),
                "response": body
            })))
        }
        Err(e) => {
            log::error!("Webhook test failed: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

fn calculate_hmac_signature(secret: &str, payload: &str) -> String {
    use std::io::Write;
    
    let mut mac = hmac_sha256::HMAC::new(secret.as_bytes());
    mac.write_all(payload.as_bytes()).unwrap();
    format!("{:x}", mac.finalize())
}

pub fn emit_webhook_event(
    conn: &mut PgConnection,
    bot_id: Uuid,
    event: &str,
    data: serde_json::Value,
) {
    use crate::core::shared::schema::attendance_webhooks::dsl::*;

    let webhooks: Vec<(Uuid, String, Vec<String>, Option<String>)> = attendance_webhooks
        .filter(attendance_webhooks::bot_id.eq(bot_id))
        .filter(attendance_webhooks::is_active.eq(true))
        .select((id, webhook_url, events, secret_key))
        .load(conn)
        .unwrap_or_default();

    for (webhook_id, webhook_url, events, secret) in webhooks {
        if !events.contains(&event.to_string()) {
            continue;
        }

        let payload = WebhookPayload {
            event: event.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            bot_id,
            data: data.clone(),
        };

        let payload_json = serde_json::to_string(&payload).unwrap_or_default();

        let mut request = reqwest::Client::new()
            .post(&webhook_url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .body(payload_json.clone());

        if let Some(ref secret_key) = secret {
            let signature = calculate_hmac_signature(secret_key, &payload_json);
            request = request.header("X-Webhook-Signature", signature);
        }

        let webhook_url_clone = webhook_url.clone();
        
        tokio::spawn(async move {
            if let Err(e) = request.send().await {
                log::error!("Failed to emit webhook {}: {}", webhook_url_clone, e);
            } else {
                log::info!("Webhook emitted successfully: {} event={}", webhook_url_clone, event);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_generation() {
        let secret = "test-secret";
        let payload = r#"{"event":"test","data":{}}"#;
        
        let signature = calculate_hmac_signature(secret, payload);
        
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64);
    }
}
