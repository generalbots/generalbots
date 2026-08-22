use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use diesel::RunQueryDsl;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

/// Resolves the caller's tenant branch from the server-minted JWT claims
/// (issue #734). Falls back to the global nil branch so anonymous/system
/// callers keep working, but every query is still constrained by the resolved
/// branch — a tenant can never see another tenant's rows.
fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

pub async fn list_connectors(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Pool error: {e}"),
        )
    })?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        endpoint: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        config: Option<serde_json::Value>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, kind, endpoint, status, config, created_at FROM integrations_connectors WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id, "name": r.name, "kind": r.kind, "endpoint": r.endpoint,
                "status": r.status, "config": r.config, "created_at": r.created_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn connect_connector(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed =
        Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Pool error: {e}"),
        )
    })?;
    let n = diesel::sql_query(
        "UPDATE integrations_connectors SET status = 'connected' WHERE id = $1 AND branch_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, "Connector not found".to_string()));
    }
    Ok(Json(serde_json::json!({"status": "connected"})))
}

pub async fn disconnect_connector(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed =
        Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Pool error: {e}"),
        )
    })?;
    let n = diesel::sql_query("UPDATE integrations_connectors SET status = 'disconnected' WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn).map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, "Connector not found".to_string()));
    }
    Ok(Json(serde_json::json!({"status": "disconnected"})))
}

pub async fn list_etl(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Pool error: {e}"),
        )
    })?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        source: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        target: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        schedule: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        last_run: Option<chrono::DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, source, target, schedule, status, last_run, created_at
         FROM integrations_etl_jobs WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "source": r.source, "target": r.target,
        "schedule": r.schedule, "status": r.status, "last_run": r.last_run, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}
