use axum::extract::{ConnectInfo, Extension, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::prelude::*;
use serde::Deserialize;
use tracing::{debug, info, warn};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::models::{ConnectionConfig, DesktopSession, SessionAuditEvent};
use crate::proxy::{extract_client_ip, handle_ws_connection, tcp_health_check};
use crate::session_manager::{SessionError, SessionManager};
use crate::types::{ConnectionCreateRequest, HealthCheckResponse, mask_ip};

// ---------------------------------------------------------------------------
// Authentication & authorization helpers
// ---------------------------------------------------------------------------

/// Rejects unauthenticated callers (anonymous users or the nil UUID).
/// Returns `Ok(user)` for authenticated callers, `Err(status)` otherwise.
fn require_authenticated(user: &AuthenticatedUser) -> Result<(), StatusCode> {
    if !user.is_authenticated() {
        warn!("Desktop proxy: rejected unauthenticated request (user={})", user.user_id);
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

/// Applies RBAC: a user may only reach sessions they own; admins may reach
/// any session.
fn can_access_session(user: &AuthenticatedUser, session: &DesktopSession) -> bool {
    user.is_admin() || session.is_owned_by(user.user_id)
}

/// Persists a session audit event into `desktop_connection_log`.
fn write_audit_event(pool: &DbPool, event: &SessionAuditEvent) {
    let pool = pool.clone();
    let event = event.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
        diesel::sql_query(
            "INSERT INTO desktop_connection_log \
             (connection_id, user_id, session_id, host, port, protocol, connected_at, \
              disconnected_at, bytes_transferred, disconnect_reason) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind::<diesel::sql_types::Uuid, _>(event.connection_id)
        .bind::<diesel::sql_types::Uuid, _>(event.user_id)
        .bind::<diesel::sql_types::Uuid, _>(event.session_id)
        .bind::<diesel::sql_types::Text, _>(&event.host)
        .bind::<diesel::sql_types::Int4, _>(event.port as i32)
        .bind::<diesel::sql_types::Text, _>(&event.protocol)
        .bind::<diesel::sql_types::Timestamptz, _>(event.connected_at)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(event.disconnected_at)
        .bind::<diesel::sql_types::BigInt, _>(event.bytes_transferred)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(event.disconnect_reason)
        .execute(&mut conn)
        .map_err(|e| format!("write desktop audit event: {e}"))?;
        Ok(())
    });
}

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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ConnectionCreateRequest>,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&addr);
    let user_id = user.user_id;

    if let Err(status) = require_authenticated(&user) {
        return (status, Json(serde_json::json!({ "success": false, "error": "Authentication required" })));
    }

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
    if !state.config.is_target_port_allowed(req.target_port) {
        warn!(
            "Desktop proxy: user {} attempted to proxy to disallowed port {} (allow-list: VNC 5900-5999, RDP 3389)",
            user_id, req.target_port
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "success": false, "error": "target port is not on the allowed list (VNC 5900-5999, RDP 3389)" })),
        );
    }

    // Tenant scoping: sessions are attributed to the authenticated user and
    // their organization/branch when available.
    let session = DesktopSession::with_scope(
        user_id,
        user.organization_id,
        None,
        req.target_host.clone(),
        req.target_port,
        client_ip,
    );

    match state.session_manager.register_session(session).await {
        Ok(session) => {
            info!(
                "Connection created: {} (user={}, target={}:{})",
                session.id, user_id, req.target_host, req.target_port,
            );
            // Audit: session start with actor + target (masked in logs).
            if let Some(pool) = state.pool.clone() {
                write_audit_event(
                    &pool,
                    &SessionAuditEvent {
                        connection_id: Uuid::new_v4(),
                        user_id,
                        session_id: session.id,
                        host: session.target_host.clone(),
                        port: session.target_port,
                        protocol: "vnc".to_string(),
                        connected_at: session.created_at,
                        disconnected_at: None,
                        bytes_transferred: 0,
                        disconnect_reason: None,
                    },
                );
            }
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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ConnectRequest>,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&addr);
    let user_id = user.user_id;

    if let Err(status) = require_authenticated(&user) {
        return (status, Json(serde_json::json!({ "success": false, "error": "Authentication required" })));
    }

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
    if !state.config.is_target_port_allowed(req.port) {
        warn!(
            "Desktop proxy: user {} attempted to proxy to disallowed port {} (allow-list: VNC 5900-5999, RDP 3389)",
            user_id, req.port
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "success": false, "error": "port is not on the allowed list (VNC 5900-5999, RDP 3389)" })),
        );
    }

    let session = DesktopSession::with_scope(
        user_id,
        user.organization_id,
        None,
        req.host.clone(),
        req.port,
        client_ip,
    );
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

    // Audit: session start for the VDI connect flow.
    if let Some(pool) = state.pool.clone() {
        let audit_session = session.clone();
        write_audit_event(
            &pool,
            &SessionAuditEvent {
                connection_id: Uuid::new_v4(),
                user_id,
                session_id: audit_session.id,
                host: audit_session.target_host.clone(),
                port: audit_session.target_port,
                protocol: protocol.clone(),
                connected_at: audit_session.created_at,
                disconnected_at: None,
                bytes_transferred: 0,
                disconnect_reason: None,
            },
        );
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

pub async fn list_connections(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    if let Err(status) = require_authenticated(&user) {
        return (status, Json(serde_json::json!({ "success": false, "error": "Authentication required" })));
    }

    // Session listing is scoped to the owning user (admins see all).
    let sessions = state.session_manager.get_all_sessions().await;
    let mut summaries: Vec<serde_json::Value> = sessions
        .iter()
        .filter(|s| user.is_admin() || s.is_owned_by(user.user_id))
        .map(|s| serde_json::json!(s.to_summary()))
        .collect();

    // Merge saved (persisted) connections so the grid shows them after restart.
    if let Some(pool) = state.pool.clone() {
        let pool_for_task = pool.clone();
        let user_for_task = user.user_id;
        let is_admin = user.is_admin();
        let result = tokio::task::spawn_blocking(move || -> Result<Vec<serde_json::Value>, String> {
            let mut conn = pool_for_task.get().map_err(|e| format!("db pool: {e}"))?;
            #[derive(diesel::QueryableByName)]
            struct SavedRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                user_id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                name: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                host: String,
                #[diesel(sql_type = diesel::sql_types::Int4)]
                port: i32,
                #[diesel(sql_type = diesel::sql_types::Text)]
                protocol: String,
            }
            let query = if is_admin {
                "SELECT id, user_id, name, host, port, protocol FROM desktop_connections ORDER BY name ASC"
            } else {
                "SELECT id, user_id, name, host, port, protocol FROM desktop_connections WHERE user_id = $1 ORDER BY name ASC"
            };
            let mut query_builder = diesel::sql_query(query);
            if !is_admin {
                query_builder = query_builder.bind::<diesel::sql_types::Uuid, _>(user_for_task);
            }
            let rows: Vec<SavedRow> = query_builder
                .load::<SavedRow>(&mut conn)
                .map_err(|e| format!("load saved vdi connections: {e}"))?;
            Ok(rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "user_id": r.user_id,
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
    Extension(user): Extension<AuthenticatedUser>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    if let Err(status) = require_authenticated(&user) {
        return (status, Json(serde_json::json!({ "success": false, "error": "Authentication required" })));
    }

    match state.session_manager.get_session(id).await {
        Some(s) if can_access_session(&user, &s) => {
            (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": s.to_summary() })))
        }
        Some(_) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "success": false, "error": "not authorized to view this connection" })),
        ),
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
    Extension(user): Extension<AuthenticatedUser>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    if let Err(status) = require_authenticated(&user) {
        return (status, Json(serde_json::json!({ "success": false, "error": "Authentication required" })));
    }

    // Ownership check before removal (kill switch for non-owners is denied).
    let authorized = match state.session_manager.get_session(id).await {
        Some(s) => can_access_session(&user, &s),
        None => true, // Allow the delete to fall through to DB removal below.
    };
    if !authorized {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "success": false, "error": "not authorized to delete this connection" })),
        );
    }

    let removed_session = state.session_manager.remove_session(id).await;
    let user_for_task = user.user_id;

    // Also remove the saved row if it exists (scoped to the owner).
    let mut db_deleted = false;
    if let Some(pool) = state.pool.clone() {
        let pool_for_task = pool.clone();
        let result = tokio::task::spawn_blocking(move || -> Result<bool, String> {
            let mut conn = pool_for_task.get().map_err(|e| format!("db pool: {e}"))?;
            let affected = diesel::sql_query(
                "DELETE FROM desktop_connections WHERE id = $1 AND (user_id = $2 OR $3)",
            )
            .bind::<diesel::sql_types::Uuid, _>(id)
            .bind::<diesel::sql_types::Uuid, _>(user_for_task)
            .bind::<diesel::sql_types::Bool, _>(user.is_admin())
            .execute(&mut conn)
            .map_err(|e| format!("delete saved vdi connection: {e}"))?;
            Ok(affected > 0)
        })
        .await;
        if let Ok(Ok(d)) = result {
            db_deleted = d;
        }
    }

    // Audit: session end with actor + reason.
    if let Some(removed) = &removed_session {
        if let Some(pool) = state.pool.clone() {
            write_audit_event(
                &pool,
                &SessionAuditEvent {
                    connection_id: Uuid::new_v4(),
                    user_id: user_for_task,
                    session_id: removed.id,
                    host: removed.target_host.clone(),
                    port: removed.target_port,
                    protocol: "vnc".to_string(),
                    connected_at: removed.created_at,
                    disconnected_at: Some(chrono::Utc::now()),
                    bytes_transferred: removed.bytes_sent + removed.bytes_received,
                    disconnect_reason: Some("deleted".to_string()),
                },
            );
        }
    }

    if removed_session.is_some() || db_deleted {
        info!("Connection deleted: {} by user {}", id, user_for_task);
        (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": "deleted" })))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "error": format!("connection {id} not found") })),
        )
    }
}

// ---------------------------------------------------------------------------
// POST /connections/kill — kill switch for the caller's active sessions
// ---------------------------------------------------------------------------

pub async fn kill_own_sessions(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    if let Err(status) = require_authenticated(&user) {
        return (status, Json(serde_json::json!({ "success": false, "error": "Authentication required" })));
    }

    let removed = state.session_manager.kill_sessions_for_user(user.user_id).await;
    info!(
        "Kill switch: user {} terminated {} active session(s)",
        user.user_id,
        removed.len()
    );

    // Audit each terminated session.
    if let Some(pool) = state.pool.clone() {
        let now = chrono::Utc::now();
        for s in &removed {
            write_audit_event(
                &pool,
                &SessionAuditEvent {
                    connection_id: Uuid::new_v4(),
                    user_id: user.user_id,
                    session_id: s.id,
                    host: s.target_host.clone(),
                    port: s.target_port,
                    protocol: "vnc".to_string(),
                    connected_at: s.created_at,
                    disconnected_at: Some(now),
                    bytes_transferred: s.bytes_sent + s.bytes_received,
                    disconnect_reason: Some("kill-switch".to_string()),
                },
            );
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "success": true, "data": { "terminated": removed.len() } })),
    )
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
    // Browser WebSocket handshakes cannot carry an Authorization header, so
    // the random 128-bit `session_id` acts as a capability token (same model
    // as the terminal WS). Sessions are only ever created by authenticated
    // users (`create_connection`/`handle_connect` reject anonymous callers),
    // so a valid session id already implies an authenticated owner.
    let session = match state.session_manager.get_session(session_id).await {
        Some(s) => s,
        None => {
            warn!("WS proxy requested for unknown session: {}", session_id);
            return (StatusCode::NOT_FOUND, "session not found").into_response();
        }
    };

    // WebSocket handshakes from the browser cannot send an Authorization
    // header, so the session id is the capability. Sessions are created only
    // by authenticated users and the owner check happens at creation time
    // (`create_connection`/`handle_connect`); the proxy itself validates the
    // session exists and is not expired.

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
        .route(&format!("{PREFIX}/connections/kill"), axum::routing::post(kill_own_sessions))
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
            pool: None,
        };
        let _app = router(state);
    }
}
