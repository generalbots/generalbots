use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;
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

pub async fn list_templates(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] description: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] version: String,
        #[diesel(sql_type = diesel::sql_types::Text)] author: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, description, kind, version, author, created_at FROM app_templates WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "description": r.description, "kind": r.kind,
        "version": r.version, "author": r.author, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn preview_template(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
    }
    let row: Option<Row> = diesel::sql_query("SELECT name FROM app_templates WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .get_result(&mut conn).optional().map_err(db::map_diesel_err)?;
    let name = row.ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?.name;
    Ok(Json(serde_json::json!({"preview": {"id": id, "name": name, "files": [], "config": {}}})))
}

pub async fn deploy_template(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let n = diesel::sql_query("SELECT 1 FROM app_templates WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, "Template not found".to_string()));
    }
    let deploy_id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO app_template_deploys (id, template_id, status, target, deployed_at, branch_id)
         VALUES ($1, $2, 'deployed', 'production', $3, $4)",
    )
    .bind::<diesel::sql_types::Uuid, _>(deploy_id)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({"result": {"id": deploy_id, "template_id": id, "status": "deployed", "deployed_at": now, "target": "production"}})))
}