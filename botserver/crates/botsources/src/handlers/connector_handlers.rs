use crate::connector_types::*;
use crate::connectors::engine::ConnectorEngine;
use crate::connectors::templates;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct ConnectorFilterParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn with_conn<F, T>(pool: Pool<ConnectionManager<PgConnection>>, f: F) -> Result<T, (StatusCode, String)>
where
    F: FnOnce(&mut PgConnection) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let mut conn = pool.get().map_err(|e| {
        log::error!("DB connection error: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed".to_string())
    })?;
    f(&mut conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn handle_list_connectors(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<ConnectorFilterParams>,
) -> Result<Json<Vec<ConnectorConfig>>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let limit = p.limit.unwrap_or(20).min(100);
    let offset = p.offset.unwrap_or(0);
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::list_connectors(conn, bot_id, limit, offset))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_get_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectorConfig>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::get_connector(conn, id))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_create_connector(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateConnectorRequest>,
) -> Result<Json<ConnectorConfig>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::create_connector(conn, req))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_update_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnectorRequest>,
) -> Result<Json<ConnectorConfig>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::update_connector(conn, id, req))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_delete_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::delete_connector(conn, id))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(serde_json::json!({"deleted": true})))
}

pub async fn handle_test_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::test_connection(conn, id))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(serde_json::json!({"status": "success", "message": result})))
}

pub async fn handle_sync_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SyncResult>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::sync_connector(conn, id))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_discover_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DiscoveredSchema>>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::discover_schema(conn, id, None))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_get_connector_logs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<SyncLog>>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let limit = p.limit.unwrap_or(20).min(100);
    let offset = p.offset.unwrap_or(0);
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, move |conn| ConnectorEngine::get_sync_logs(conn, id, limit, offset))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}

pub async fn handle_list_connector_templates(
) -> Result<Json<Vec<ConnectorTemplate>>, (StatusCode, String)> {
    Ok(Json(templates::get_all_templates()))
}

pub async fn handle_get_connector_template(
    Path(connector_type): Path<String>,
) -> Result<Json<ConnectorTemplate>, (StatusCode, String)> {
    templates::get_template(&connector_type)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Template not found: {connector_type}")))
        .map(Json)
}

pub async fn handle_install_connector_template(
    State(state): State<Arc<AppState>>,
    Path(connector_type): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ConnectorConfig>, (StatusCode, String)> {
    let template = templates::get_template(&connector_type)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Template not found: {connector_type}")))?;

    let bot_id = req.get("bot_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Uuid>().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "bot_id is required".to_string()))?;

    let name = req.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&template.name)
        .to_string();

    let auth_config: AuthConfig = serde_json::from_value(
        req.get("auth_config").cloned().unwrap_or(serde_json::json!({
            "auth_type": "api_key"
        }))
    ).unwrap_or(AuthConfig {
        auth_type: AuthType::ApiKey, api_key: None, api_key_header: None,
        username: None, password: None, oauth2_client_id: None,
        oauth2_client_secret: None, oauth2_token_url: None,
        oauth2_scopes: None, base_url: None, extra_headers: None,
    });

    let create_req = CreateConnectorRequest {
        name,
        connector_type: connector_type.clone(),
        description: Some(template.description),
        auth_config,
        endpoints: Some(template.default_endpoints),
        schedule: template.default_schedule,
        bot_id,
    };

    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        with_conn(pool, |conn| ConnectorEngine::create_connector(conn, create_req))
    }).await.map_err(|e| {
        log::error!("Task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })??;
    Ok(Json(result))
}
