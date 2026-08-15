use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::prelude::*;
use serde::Deserialize;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::{ConnectionConfig, DesktopSession};
use crate::proxy::{extract_client_ip, handle_ws_connection, tcp_health_check};
use crate::session_manager::{SessionError, SessionManager};
use crate::types::{ConnectionCreateRequest, HealthCheckResponse, mask_ip};

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub session_manager: SessionManager,
    pub config: ConnectionConfig,
    pub pool: Option<DbPool>,
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
    let protocol = req.protocol.unwrap_or_else(|| "vnc".to_string());
    let auth_type = req.auth_type.clone().unwrap_or_else(|| "password".to_string());
    let conn_name = req.name.clone();
    let target_host = req.host.clone();
    let target_port = req.port;
    let proto = protocol.clone();

    // Persist the saved connection so it survives restarts (#vdi persistence).
    if let Some(pool) = state.pool.clone() {
        let pool_for_task = pool.clone();
        let host_for_task = target_host.clone();
        let name_for_task = conn_name.clone();
        let auth_for_task = auth_type.clone();
        let proto_for_task = proto.clone();
        let now = chrono::Utc::now();
        let log_name = name_for_task.clone();
        let log_host = host_for_task.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
            let mut conn = pool_for_task.get().map_err(|e| format!("db pool: {e}"))?;
            diesel::sql_query(
                "INSERT INTO desktop_connections \
                 (id, user_id, name, host, port, protocol, auth_type, auto_connect, created_at, updated_at, last_used_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $8, $8) \
                 ON CONFLICT DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Text, _>(&name_for_task)
            .bind::<diesel::sql_types::Text, _>(&host_for_task)
            .bind::<diesel::sql_types::Int4, _>(target_port as i32)
            .bind::<diesel::sql_types::Text, _>(&proto_for_task)
            .bind::<diesel::sql_types::Text, _>(&auth_for_task)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)
            .map_err(|e| format!("persist vdi connection: {e}"))?;
            Ok(())
        })
        .await;
        if let Err(e) = result {
            tracing::warn!("Failed to persist VDI connection {} -> {}: {}", log_name, log_host, e);
        }
    }

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
    let mut summaries: Vec<serde_json::Value> =
        sessions.iter().map(|s| serde_json::json!(s.to_summary())).collect();

    // Merge saved (persisted) connections so the grid shows them after restart.
    if let Some(pool) = state.pool.clone() {
        let result = tokio::task::spawn_blocking(move || -> Result<Vec<serde_json::Value>, String> {
            let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
            #[derive(diesel::QueryableByName)]
            struct SavedRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                name: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                host: String,
                #[diesel(sql_type = diesel::sql_types::Int4)]
                port: i32,
                #[diesel(sql_type = diesel::sql_types::Text)]
                protocol: String,
            }
            let rows: Vec<SavedRow> = diesel::sql_query(
                "SELECT id, name, host, port, protocol FROM desktop_connections ORDER BY name ASC",
            )
            .load::<SavedRow>(&mut conn)
            .map_err(|e| format!("load saved vdi connections: {e}"))?;
            Ok(rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "target_host": r.host,
                        "target_port": r.port,
                        "status": "saved",
                        "name": r.name,
                        "protocol": r.protocol,
                        "saved": true,
                    })
                })
                .collect())
        })
        .await;
        if let Ok(Ok(mut saved)) = result {
            summaries.append(&mut saved);
        }
    }

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
    let removed_session = state.session_manager.remove_session(id).await;

    // Also remove the saved row if it exists.
    let mut db_deleted = false;
    if let Some(pool) = state.pool.clone() {
        let result = tokio::task::spawn_blocking(move || -> Result<bool, String> {
            let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
            let affected = diesel::sql_query("DELETE FROM desktop_connections WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(&mut conn)
                .map_err(|e| format!("delete saved vdi connection: {e}"))?;
            Ok(affected > 0)
        })
        .await;
        if let Ok(Ok(d)) = result {
            db_deleted = d;
        }
    }

    if removed_session.is_some() || db_deleted {
        info!("Connection deleted: {}", id);
        (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": "deleted" })))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("connection {id} not found") })),
        )
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
        Self::new(None)
    }
}

impl AppState {
    pub fn new(pool: Option<DbPool>) -> Self {
        let config = ConnectionConfig::default();
        Self {
            session_manager: SessionManager::new(config.clone()),
            config,
            pool,
        }
    }
}

/// Read the default VDI connection from Vault (`secret/gbo/vdi`).
/// Never hardcodes infrastructure addresses in the repository; the host is
/// provisioned at deploy time as Vault fields `default-host`/`default-port`/
/// `default-name`.
fn vdi_default_connection() -> Option<(String, u16, String)> {
    let manager = botcoresecrets::SecretsManager::get_clone().ok()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => {
                rt.block_on(manager.get_secret(botcoresecrets::SecretPaths::VDI))
            }
            Err(e) => {
                warn!("vdi: failed to create runtime: {e}");
                return;
            }
        };
        let _ = tx.send(result);
    });
    let secrets = rx.recv().ok()?.ok()?;

    let host = secrets.get("default-host").filter(|h| !h.is_empty())?.clone();
    let port = secrets
        .get("default-port")
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5900);
    let name = secrets
        .get("default-name")
        .filter(|n| !n.is_empty())
        .cloned()
        .unwrap_or_else(|| "Default Desktop".to_string());
    Some((host, port, name))
}

/// Seed the default VDI connection into `desktop_connections` from Vault.
/// Idempotent (`ON CONFLICT DO NOTHING`); no-op when the Vault path is absent.
pub fn seed_default_connection(pool: &DbPool) {
    let Some((host, port, name)) = vdi_default_connection() else {
        return;
    };

    let result = (|| -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
        diesel::sql_query(
            "INSERT INTO desktop_connections \
             (id, user_id, name, host, port, protocol, auth_type, auto_connect, created_at, updated_at, last_used_at) \
             VALUES ($1, $2, $3, $4, $5, 'vnc', 'password', true, $6, $6, $6) \
             ON CONFLICT DO NOTHING",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(Uuid::nil())
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>(&host)
        .bind::<diesel::sql_types::Int4, _>(port as i32)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut conn)
        .map_err(|e| format!("seed default vdi connection: {e}"))?;
        Ok(())
    })();
    if let Err(e) = result {
        tracing::warn!("Failed to seed default VDI connection for {host}: {e}");
    } else {
        info!("Seeded default VDI connection from Vault secret/gbo/vdi");
    }
}

pub fn configure_routes(pool: Option<DbPool>) -> axum::Router<()> {
    if let Some(p) = &pool {
        seed_default_connection(p);
    }
    router(AppState::new(pool))
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
