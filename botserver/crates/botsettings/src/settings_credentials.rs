use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use serde::Deserialize;

use botcore::shared::state::AppState;

use crate::settings_api::{get_conn, random_key, read_pref, resolve_user_id, write_pref};

pub async fn api_keys_list(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let keys = read_pref(&mut conn, user_id, "api_keys");
    let items = keys.as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return (StatusCode::OK, Html(r#"<div class="empty-state"><p>No API keys</p></div>"#.to_string()));
    }

    let mut html = String::new();
    for key in items {
        let name = key.get("name").and_then(|v| v.as_str()).unwrap_or("key");
        let prefix = key.get("prefix").and_then(|v| v.as_str()).unwrap_or("gb_••••");
        let id = key.get("id").and_then(|v| v.as_str()).unwrap_or("");
        html.push_str(&format!(
            r#"<div class="api-key-item" data-key-id="{id}"><span class="api-key-name">{name}</span><code class="api-key-value">{prefix}</code><button class="btn-icon" hx-delete="/api/user/api-keys/{id}" hx-target="closest .api-key-item" hx-swap="outerHTML" title="Revoke">×</button></div>"#
        ));
    }
    (StatusCode::OK, Html(html))
}

#[derive(Deserialize)]
pub struct ApiKeyForm {
    pub name: Option<String>,
}

pub async fn api_keys_create(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<ApiKeyForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let name = form.name.unwrap_or_else(|| "API Key".to_string());
    let mut existing = read_pref(&mut conn, user_id, "api_keys")
        .as_array()
        .cloned()
        .unwrap_or_default();
    let id = format!("ak_{}", random_key(12));
    let prefix = format!("gb_{}", random_key(8));
    existing.push(serde_json::json!({ "id": id, "name": name, "prefix": prefix, "created_at": chrono::Utc::now().to_rfc3339() }));
    write_pref(&mut conn, user_id, "api_keys", &serde_json::json!(existing));

    (StatusCode::OK, Html(format!("Created API key <code>{prefix}…</code>")))
}

/// DELETE /api/user/api-keys/{key_id}
///
/// Removes a stored API key from the caller's preferences (fix #832 — the
/// previous UI only showed an `alert('Revoke key')` placeholder).
pub async fn api_keys_delete(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let mut existing = read_pref(&mut conn, user_id, "api_keys")
        .as_array()
        .cloned()
        .unwrap_or_default();
    let before = existing.len();
    existing.retain(|k| k.get("id").and_then(|v| v.as_str()) != Some(key_id.as_str()));
    write_pref(&mut conn, user_id, "api_keys", &serde_json::json!(existing));

    if existing.len() == before {
        return (StatusCode::NOT_FOUND, Html("API key not found".to_string()));
    }
    (StatusCode::OK, Html(String::new()))
}

pub async fn webhooks_list(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let hooks = read_pref(&mut conn, user_id, "webhooks");
    let items = hooks.as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return (StatusCode::OK, Html(r#"<div class="empty-state"><p>No webhooks configured</p></div>"#.to_string()));
    }

    let mut html = String::new();
    for hook in items {
        let url = hook.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let events = hook.get("events").and_then(|v| v.as_str()).unwrap_or("all");
        let id = hook.get("id").and_then(|v| v.as_str()).unwrap_or("");
        html.push_str(&format!(
            r#"<div class="webhook-item" data-webhook-id="{id}"><span class="webhook-url">{url}</span><span class="webhook-events">{events}</span><button class="btn-icon" hx-delete="/api/user/webhooks/{id}" hx-target="closest .webhook-item" hx-swap="outerHTML" title="Remove">×</button></div>"#
        ));
    }
    (StatusCode::OK, Html(html))
}

#[derive(Deserialize)]
pub struct WebhookForm {
    pub url: Option<String>,
    pub events: Option<String>,
}

pub async fn webhooks_create(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<WebhookForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let url = form.url.unwrap_or_default();
    if url.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Html("Webhook URL is required".to_string()));
    }

    let mut existing = read_pref(&mut conn, user_id, "webhooks")
        .as_array()
        .cloned()
        .unwrap_or_default();
    existing.push(serde_json::json!({
        "id": format!("wh_{}", random_key(12)),
        "url": url,
        "events": form.events.unwrap_or_else(|| "all".to_string()),
        "created_at": chrono::Utc::now().to_rfc3339(),
    }));
    write_pref(&mut conn, user_id, "webhooks", &serde_json::json!(existing));

    (StatusCode::OK, Html("Webhook registered".to_string()))
}

/// DELETE /api/user/webhooks/{webhook_id}
pub async fn webhooks_delete(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(webhook_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let mut existing = read_pref(&mut conn, user_id, "webhooks")
        .as_array()
        .cloned()
        .unwrap_or_default();
    let before = existing.len();
    existing.retain(|h| h.get("id").and_then(|v| v.as_str()) != Some(webhook_id.as_str()));
    write_pref(&mut conn, user_id, "webhooks", &serde_json::json!(existing));

    if existing.len() == before {
        return (StatusCode::NOT_FOUND, Html("Webhook not found".to_string()));
    }
    (StatusCode::OK, Html(String::new()))
}
