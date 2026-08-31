//! #1247 — Web Push subscription backend.
//!
//! The Settings app's `togglePushNotifications()` calls
//! `POST /api/notifications/push/register` and
//! `POST /api/notifications/push/unregister` with the browser's
//! PushSubscription object. Previously no backend existed, so the calls
//! 404'd and the toggle silently failed. These endpoints persist/remove the
//! subscription (scoped to the authenticated user) so a future push-delivery
//! service can fan out notifications from them.

use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Extension, Json, Router,
};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

use crate::security::auth_api::types::AuthenticatedUser;
use botcore::shared::utils::DbPool;

/// Idempotent schema; the table is created on first use so the module works
/// in environments where the diesel migration pipeline has not run.
const PUSH_SUBSCRIPTIONS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS push_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    endpoint TEXT NOT NULL UNIQUE,
    p256dh TEXT NOT NULL DEFAULT '',
    auth TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct PushSubscriptionRequest {
    pub endpoint: String,
    #[serde(default)]
    pub expiration_time: Option<f64>,
    #[serde(default)]
    pub keys: Option<PushSubscriptionKeys>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushSubscriptionKeys {
    #[serde(default)]
    pub p256dh: String,
    #[serde(default)]
    pub auth: String,
}

#[derive(Clone)]
pub struct PushState {
    pub pool: DbPool,
}

pub fn configure_push_routes(pool: DbPool) -> Router {
    if let Ok(mut conn) = pool.get() {
        if let Err(e) = diesel::sql_query(PUSH_SUBSCRIPTIONS_SCHEMA).execute(&mut conn) {
            log::error!("push_subscriptions schema setup failed: {e}");
        }
    }
    Router::new()
        .route(
            "/api/notifications/push/register",
            post(register_push_subscription),
        )
        .route(
            "/api/notifications/push/unregister",
            post(unregister_push_subscription),
        )
        .with_state(Arc::new(PushState { pool }))
}

async fn register_push_subscription(
    State(state): State<Arc<PushState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<PushSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let endpoint = req.endpoint.trim();
    if endpoint.is_empty() || !endpoint.starts_with("https://") {
        return Err((
            StatusCode::BAD_REQUEST,
            "invalid push endpoint".to_string(),
        ));
    }
    let p256dh = req
        .keys
        .as_ref()
        .map(|k| k.p256dh.trim().to_string())
        .unwrap_or_default();
    let auth = req
        .keys
        .as_ref()
        .map(|k| k.auth.trim().to_string())
        .unwrap_or_default();
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("db: {e}")))?;
    diesel::sql_query(
        "INSERT INTO push_subscriptions (user_id, endpoint, p256dh, auth, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, NOW(), NOW()) \
         ON CONFLICT (endpoint) DO UPDATE SET \
             user_id = EXCLUDED.user_id, p256dh = EXCLUDED.p256dh, \
             auth = EXCLUDED.auth, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(user.user_id)
    .bind::<diesel::sql_types::Text, _>(endpoint)
    .bind::<diesel::sql_types::Text, _>(&p256dh)
    .bind::<diesel::sql_types::Text, _>(&auth)
    .execute(&mut conn)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("save push subscription: {e}"),
        )
    })?;
    Ok(Json(serde_json::json!({ "registered": true })))
}

async fn unregister_push_subscription(
    State(state): State<Arc<PushState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<PushSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("db: {e}")))?;
    diesel::sql_query(
        "DELETE FROM push_subscriptions WHERE endpoint = $1 AND user_id = $2",
    )
    .bind::<diesel::sql_types::Text, _>(&req.endpoint)
    .bind::<diesel::sql_types::Uuid, _>(user.user_id)
    .execute(&mut conn)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("delete push subscription: {e}"),
        )
    })?;
    Ok(Json(serde_json::json!({ "unregistered": true })))
}
