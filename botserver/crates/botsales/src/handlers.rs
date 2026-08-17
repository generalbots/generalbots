use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use diesel::RunQueryDsl;
use rust_decimal::Decimal;
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

pub async fn list_deals(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Uuid)] contact_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Numeric)] value: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] stage: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] probability: Decimal,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] closed_at: Option<chrono::DateTime<Utc>>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, title, contact_id, value, stage, status, probability, created_at, closed_at
         FROM sales_deals WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "title": r.title, "contact_id": r.contact_id, "value": r.value.to_string(),
        "stage": r.stage, "status": r.status, "probability": r.probability.to_string(),
        "created_at": r.created_at, "closed_at": r.closed_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_deal(headers: HeaderMap, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO sales_deals (id, title, contact_id, value, stage, status, probability, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, 'open', $6, $7, $8)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(item.get("title").and_then(|v| v.as_str()).unwrap_or(""))
    .bind::<diesel::sql_types::Uuid, _>(Uuid::parse_str(item.get("contact_id").and_then(|v| v.as_str()).unwrap_or(Uuid::nil().to_string().as_str())).unwrap_or(Uuid::nil()))
    .bind::<diesel::sql_types::Numeric, _>(Decimal::new((item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0) as i64, 2))
    .bind::<diesel::sql_types::Text, _>(item.get("stage").and_then(|v| v.as_str()).unwrap_or("lead"))
    .bind::<diesel::sql_types::Numeric, _>(Decimal::new((item.get("probability").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0) as i64, 2))
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({"item": {"id": id, "created_at": now, "status": "open"}})))
}

pub async fn update_deal(headers: HeaderMap, Path(id): Path<String>, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    if let Some(stage) = item.get("stage").and_then(|v| v.as_str()) {
        let n = diesel::sql_query("UPDATE sales_deals SET stage = $1 WHERE id = $2 AND branch_id = $3")
            .bind::<diesel::sql_types::Text, _>(stage)
            .bind::<diesel::sql_types::Uuid, _>(parsed)
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .execute(&mut conn).map_err(db::map_diesel_err)?;
        if n == 0 { return Err((StatusCode::NOT_FOUND, "Deal not found".to_string())); }
    }
    Ok(Json(serde_json::json!({"item": {"id": parsed, "status": "updated"}})))
}

pub async fn list_contacts(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] email: String,
        #[diesel(sql_type = diesel::sql_types::Text)] phone: String,
        #[diesel(sql_type = diesel::sql_types::Text)] company: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, email, phone, company, created_at FROM sales_contacts WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "email": r.email, "phone": r.phone, "company": r.company, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_activities(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] deal_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] description: String,
        #[diesel(sql_type = diesel::sql_types::Date)] activity_date: chrono::NaiveDate,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, deal_id, kind, description, activity_date FROM sales_activities WHERE branch_id = $1 ORDER BY activity_date DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "deal_id": r.deal_id, "kind": r.kind, "description": r.description, "date": r.activity_date.to_string(),
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

