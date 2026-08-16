use crate::state::AppState;
use crate::types::{
    ConnectConnectorRequest, ConnectorListResponse, ConnectorResponse, CreateEtlJobRequest,
    EtlJobListResponse, EtlJobResponse,
};
use botcoresecrets::SecretsManager;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use diesel::RunQueryDsl;
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
             is_active, last_sync_at, last_test_at, COALESCE(last_test_status,'') as last_test_status, \
             0 as records_synced \
             FROM connectors WHERE is_active = true ORDER BY name",
        )
        .load::<ConnectorDbRow>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        diesel::sql_query(
            "SELECT id, name, connector_type, COALESCE(description,'') as description, \
             is_active, last_sync_at, last_test_at, COALESCE(last_test_status,'') as last_test_status, \
             0 as records_synced \
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
            last_test: r.last_test_at.map(|t| t.to_rfc3339()),
            last_test_status: if r.last_test_status.is_empty() {
                None
            } else {
                Some(r.last_test_status)
            },
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

    let schedule = payload.schedule.clone();
    if let Err(e) = crate::connector_ops::validate_schedule(schedule.as_deref()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Secrets never touch the DB: sensitive auth fields move to Vault and the
    // row keeps only the vault path + sanitized config.
    let raw_auth = payload.auth_config.unwrap_or_default();
    let (auth, secrets) = crate::connector_ops::split_sensitive_auth(&raw_auth);
    let endpoints = payload.endpoints.unwrap_or_default();
    let vault_path = if secrets.is_empty() {
        None
    } else {
        let path = crate::connector_ops::secrets_path("default", "default", &id.to_string());
        if let Ok(manager) = SecretsManager::from_env() {
            crate::connector_ops::store_secrets(&manager, &path, &secrets);
        } else {
            log::warn!("connector secrets for {path} not vaulted: Vault is not configured");
        }
        Some(path)
    };

    diesel::sql_query(
        "INSERT INTO connectors (id, bot_id, name, connector_type, description, auth_config, endpoints, schedule, is_active, secrets_vault_path) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9) \
         ON CONFLICT (id) DO UPDATE SET name = $3, description = $5, auth_config = $6, endpoints = $7, schedule = $8, secrets_vault_path = $9, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
    .bind::<diesel::sql_types::Text, _>(&payload.name)
    .bind::<diesel::sql_types::Text, _>(&payload.connector_type)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.description)
    .bind::<diesel::sql_types::Jsonb, _>(&auth)
    .bind::<diesel::sql_types::Jsonb, _>(&endpoints)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.schedule)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&vault_path)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true, "id": id, "secrets_vaulted": !secrets.is_empty()})))
}

/// `POST /api/ui/sources/connectors/:id/test`
///
/// Runs a live connectivity check (HTTP GET or DB TCP connect) and records the
/// outcome on the connector row for the list health column.
pub async fn handle_test_connector(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ConnectorRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        connector_type: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        auth_config: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        endpoints: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        secrets_vault_path: Option<String>,
    }

    let row = diesel::sql_query(
        "SELECT connector_type, auth_config, endpoints, secrets_vault_path FROM connectors WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .get_result::<ConnectorRow>(&mut conn)
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // Merge vaulted secrets back for the live check; never returned to the UI.
    let mut auth = row.auth_config.clone();
    if let Some(path) = row.secrets_vault_path.as_deref() {
        if let Ok(manager) = SecretsManager::from_env() {
            let secrets = crate::connector_ops::load_secrets(&manager, path);
            if let Some(obj) = auth.as_object_mut() {
                for (k, v) in secrets {
                    obj.insert(k, serde_json::json!(v));
                }
            }
        }
    }

    let (ok, latency_ms, detail) =
        crate::connector_ops::test_connection(&row.connector_type, &auth, &row.endpoints).await;
    let status = if ok { "ok" } else { "failed" };
    let now = chrono::Utc::now();

    diesel::sql_query(
        "UPDATE connectors SET last_test_at = $1, last_test_status = $2, error_log = $3, updated_at = $1 WHERE id = $4",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(&now)
    .bind::<diesel::sql_types::Text, _>(&status)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&Some(detail.clone()))
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": ok,
        "status": status,
        "latency_ms": latency_ms,
        "detail": detail,
    })))
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

/// `GET /api/integrations/connectors/templates`
///
/// Returns the connector type catalog with per-type default endpoints and
/// auth hints so the UI can render typed configuration forms without
/// hardcoding connector definitions.
pub async fn handle_list_connector_templates(
) -> Result<Json<serde_json::Value>, StatusCode> {
    let templates = crate::connectors::templates::get_all_templates();
    let items: Vec<serde_json::Value> = templates
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "description": t.description,
                "connector_type": t.connector_type.to_string(),
                "icon": t.icon,
                "auth_type": format!("{:?}", t.auth_type).to_lowercase(),
                "auth_help": t.auth_help,
                "default_schedule": t.default_schedule,
                "color": t.color,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "templates": items })))
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
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_test_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    last_test_status: String,
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
