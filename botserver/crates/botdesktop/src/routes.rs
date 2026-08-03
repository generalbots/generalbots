use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::{ConnectionConfig, DesktopSession};
use crate::proxy::{extract_client_ip, handle_ws_connection, tcp_health_check};
use crate::session_manager::{SessionError, SessionManager};
use crate::types::{
    ConnectionCreateRequest, ConnectionSummary, HealthCheckResponse, mask_ip,
};

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub session_manager: SessionManager,
    pub config: ConnectionConfig,
}

// ---------------------------------------------------------------------------
// POST /connections — create & register a proxy session
// ---------------------------------------------------------------------------

pub async fn create_connection(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Json(req): Json<ConnectionCreateRequest>,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&addr);
    let user_id = Uuid::nil(); // TODO: extract from auth middleware

    debug!(
        "create_connection: user={} target={}:{} client={}",
        user_id, req.target_host, req.target_port, mask_ip(&client_ip),
    );

    if req.target_host.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "target_host is required" })),
        );
    }
    if req.target_port == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "target_port must be between 1 and 65535" })),
        );
    }

    let session = DesktopSession::new(user_id, req.target_host, req.target_port, client_ip);

    match state.session_manager.register_session(session).await {
        Ok(session) => {
            info!("Connection created: {}", session.id);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "success": true,
                    "data": session.to_summary(),
                })),
            )
        }
        Err(SessionError::RateLimitExceeded { max, .. }) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": format!("maximum {max} concurrent connections per user"),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// POST /connect — create a persisted-style connection (maps to a proxy session)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub protocol: Option<String>,
    pub auth_type: Option<String>,
}

pub async fn handle_connect(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Json(req): Json<ConnectRequest>,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&addr);
    let user_id = Uuid::nil();

    if req.host.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "host is required" })),
        );
    }
    if req.port == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "port must be between 1 and 65535" })),
        );
    }

    let session = DesktopSession::new(user_id, req.host.clone(), req.port, client_ip);
    let protocol = req.protocol.unwrap_or_else(|| "rdp".to_string());

    match state.session_manager.register_session(session).await {
        Ok(session) => {
            info!("VDI connect created: {} -> {}:{}", req.name, req.host, req.port);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "success": true,
                    "data": session.to_summary(),
                    "name": req.name,
                    "protocol": protocol,
                })),
            )
        }
        Err(SessionError::RateLimitExceeded { max, .. }) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": format!("maximum {max} concurrent connections per user"),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })),
        ),
    }
}

// ---------------------------------------------------------------------------
// GET /connections — list all active connections
// ---------------------------------------------------------------------------

pub async fn list_connections(State(state): State<AppState>) -> impl IntoResponse {
    let sessions = state.session_manager.get_all_sessions().await;
    let summaries: Vec<ConnectionSummary> = sessions.iter().map(|s| s.to_summary()).collect();
    Json(serde_json::json!({ "success": true, "data": summaries }))
}

// ---------------------------------------------------------------------------
// GET /connections/{id} — single connection detail
// ---------------------------------------------------------------------------

pub async fn get_connection(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match state.session_manager.get_session(id).await {
        Some(s) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": s.to_summary() }))),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("connection {id} not found") })),
        ),
    }
}

// ---------------------------------------------------------------------------
// DELETE /connections/{id} — terminate a connection
// ---------------------------------------------------------------------------

pub async fn delete_connection(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match state.session_manager.remove_session(id).await {
        Some(_) => {
            info!("Connection deleted: {}", id);
            (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": "deleted" })))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("connection {id} not found") })),
        ),
    }
}

// ---------------------------------------------------------------------------
// GET /health — service health check
// ---------------------------------------------------------------------------

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let total = state.session_manager.total_count().await;
    let body = serde_json::json!({
        "status": "ok",
        "active_connections": total,
    });
    (StatusCode::OK, Json(body))
}

// ---------------------------------------------------------------------------
// POST /health/tcp — TCP port probe
// ---------------------------------------------------------------------------

pub async fn health_check_tcp(
    Json(req): Json<crate::types::HealthCheckRequest>,
) -> impl IntoResponse {
    let (reachable, latency_ms, err) = tcp_health_check(&req.host, req.port).await;
    let resp = HealthCheckResponse {
        reachable,
        latency_ms,
        error: err,
    };
    (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": resp })))
}

// ---------------------------------------------------------------------------
// GET /ws/{session_id} — WebSocket proxy endpoint
// ---------------------------------------------------------------------------

pub async fn ws_proxy_handler(
    ws: WebSocketUpgrade,
    axum::extract::Path(session_id): axum::extract::Path<Uuid>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> impl IntoResponse {
    let session = match state.session_manager.get_session(session_id).await {
        Some(s) => s,
        None => {
            warn!("WS proxy requested for unknown session: {}", session_id);
            return (StatusCode::NOT_FOUND, "session not found").into_response();
        }
    };

    let client_ip = extract_client_ip(&addr);
    debug!(
        "WS upgrade: session={} client={}",
        session_id, mask_ip(&client_ip),
    );

    ws.on_upgrade(move |socket| async move {
        handle_ws_connection(socket, session, state.session_manager).await;
    })
    .into_response()
}

// ---------------------------------------------------------------------------
// Axum router builder
// ---------------------------------------------------------------------------

const PREFIX: &str = "/api/desktop";

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route(&format!("{PREFIX}/connections"), axum::routing::post(create_connection))
        .route(&format!("{PREFIX}/connections"), axum::routing::get(list_connections))
        .route(&format!("{PREFIX}/connections/{{id}}"), axum::routing::get(get_connection))
        .route(&format!("{PREFIX}/connections/{{id}}"), axum::routing::delete(delete_connection))
        .route(&format!("{PREFIX}/connect"), axum::routing::post(handle_connect))
        .route(&format!("{PREFIX}/health"), axum::routing::get(health_check))
        .route(&format!("{PREFIX}/health/tcp"), axum::routing::post(health_check_tcp))
        .route(&format!("{PREFIX}/ws/proxy/{{session_id}}"), axum::routing::get(ws_proxy_handler))
        .with_state(state)
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let config = ConnectionConfig::default();
        Self {
            session_manager: SessionManager::new(config.clone()),
            config,
        }
    }
}

pub fn configure_routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route(&format!("{PREFIX}/connections"), axum::routing::post(create_connection))
        .route(&format!("{PREFIX}/connections"), axum::routing::get(list_connections))
        .route(&format!("{PREFIX}/connections/{{id}}"), axum::routing::get(get_connection))
        .route(&format!("{PREFIX}/connections/{{id}}"), axum::routing::delete(delete_connection))
        .route(&format!("{PREFIX}/connect"), axum::routing::post(handle_connect))
        .route(&format!("{PREFIX}/health"), axum::routing::get(health_check))
        .route(&format!("{PREFIX}/health/tcp"), axum::routing::post(health_check_tcp))
        .route(&format!("{PREFIX}/ws/proxy/{{session_id}}"), axum::routing::get(ws_proxy_handler))
}

use std::sync::Arc;

impl axum::extract::FromRef<Arc<AppState>> for AppState {
    fn from_ref(state: &Arc<AppState>) -> Self {
        state.as_ref().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_builds() {
        let state = AppState {
            session_manager: SessionManager::new(ConnectionConfig::default()),
            config: ConnectionConfig::default(),
        };
        let _app = router(state);
    }
}
