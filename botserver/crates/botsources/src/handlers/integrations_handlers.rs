use crate::state::AppState;
use crate::types::{
    ConnectConnectorRequest, ConnectorListResponse, ConnectorResponse, CreateEtlJobRequest,
    EtlJobListResponse, EtlJobResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ConnectorQuery {
    pub connected: Option<bool>,
}

pub async fn handle_list_connectors(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ConnectorQuery>,
) -> Result<Json<ConnectorListResponse>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let rows = if query.connected.unwrap_or(false) {
        diesel::sql_query(
            "SELECT id, name, connector_type, COALESCE(description,'') as description, \
             is_active, last_sync_at, 0 as records_synced \
             FROM connectors WHERE is_active = true ORDER BY name",
        )
        .load::<ConnectorDbRow>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        diesel::sql_query(
            "SELECT id, name, connector_type, COALESCE(description,'') as description, \
             is_active, last_sync_at, 0 as records_synced \
             FROM connectors ORDER BY name",
        )
        .load::<ConnectorDbRow>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let connectors: Vec<ConnectorResponse> = rows
        .into_iter()
        .map(|r| ConnectorResponse {
            id: r.id,
            name: r.name,
            connector_type: r.connector_type,
            description: r.description,
            connected: r.is_active,
            active: r.is_active,
            last_sync: r.last_sync_at.map(|t| t.to_rfc3339()),
            records_synced: r.records_synced,
        })
        .collect();

    Ok(Json(ConnectorListResponse { connectors }))
}

pub async fn handle_connect_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConnectConnectorRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let auth = payload.auth_config.unwrap_or_default();
    let endpoints = payload.endpoints.unwrap_or_default();

    diesel::sql_query(
        "INSERT INTO connectors (id, bot_id, name, connector_type, description, auth_config, endpoints, schedule, is_active) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true) \
         ON CONFLICT (id) DO UPDATE SET name = $3, description = $5, auth_config = $6, endpoints = $7, schedule = $8, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
    .bind::<diesel::sql_types::Text, _>(&payload.name)
    .bind::<diesel::sql_types::Text, _>(&payload.connector_type)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.description)
    .bind::<diesel::sql_types::Jsonb, _>(&auth)
    .bind::<diesel::sql_types::Jsonb, _>(&endpoints)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.schedule)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

pub async fn handle_sync_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let now = chrono::Utc::now();
    diesel::sql_query(
        "UPDATE connectors SET last_sync_at = $1, last_sync_status = 'success', updated_at = $1 WHERE id = $2",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(&now)
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn handle_disconnect_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    diesel::sql_query("DELETE FROM connectors WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn handle_list_etl_jobs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<EtlJobListResponse>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let rows = diesel::sql_query(
        "SELECT e.id, e.name, \
         COALESCE(sc.name, '') as source_name, \
         COALESCE(dc.name, '') as dest_name, \
         e.schedule, e.last_run_at, e.status \
         FROM etl_jobs e \
         LEFT JOIN connectors sc ON e.source_connector_id = sc.id \
         LEFT JOIN connectors dc ON e.destination_connector_id = dc.id \
         ORDER BY e.created_at DESC",
    )
    .load::<EtlJobDbRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jobs: Vec<EtlJobResponse> = rows
        .into_iter()
        .map(|r| EtlJobResponse {
            id: r.id,
            name: r.name,
            source: r.source_name,
            destination: r.dest_name,
            schedule: r.schedule,
            last_run: r.last_run_at.map(|t| t.to_rfc3339()),
            status: r.status,
        })
        .collect();

    Ok(Json(EtlJobListResponse { jobs }))
}

pub async fn handle_create_etl_job(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEtlJobRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let id = Uuid::new_v4();
    let transform: serde_json::Value =
        payload.transform.map(|t| serde_json::from_str(&t).unwrap_or_default()).unwrap_or_default();

    diesel::sql_query(
        "INSERT INTO etl_jobs (id, bot_id, name, source_connector_id, destination_connector_id, \
         transform_config, schedule, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
    .bind::<diesel::sql_types::Text, _>(&payload.name)
    .bind::<diesel::sql_types::Uuid, _>(&payload.source)
    .bind::<diesel::sql_types::Uuid, _>(&payload.destination)
    .bind::<diesel::sql_types::Jsonb, _>(&transform)
    .bind::<diesel::sql_types::Text, _>(&payload.schedule)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

pub async fn handle_run_etl_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let now = chrono::Utc::now();
    diesel::sql_query(
        "UPDATE etl_jobs SET last_run_at = $1, last_run_status = 'running', \
         run_count = run_count + 1, status = 'running', updated_at = $1 WHERE id = $2",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(&now)
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Trigger VIBE ETL execution via the existing vibe module
    trigger_vibe_etl(&state, id);

    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn handle_delete_etl_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    diesel::sql_query("DELETE FROM etl_jobs WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

fn trigger_vibe_etl(_state: &Arc<AppState>, _job_id: Uuid) {
    // Placeholder: VIBE agent will pick up the ETL job and execute
    // In production, this sends a message to the VIBE agent loop
    // which handles data extraction, transformation, and loading
    log::info!("VIBE ETL triggered for job {}", _job_id);
}

// --- Diesel queryable rows ---

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ConnectorDbRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    connector_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    description: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    records_synced: i64,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct EtlJobDbRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    source_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    dest_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    schedule: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
}
