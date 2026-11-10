use axum::extract::Json;
use axum::http::{HeaderMap, StatusCode};
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

pub async fn get_financial(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] description: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] amount: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] category: String,
        #[diesel(sql_type = diesel::sql_types::Date)] entry_date: chrono::NaiveDate,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, kind, description, amount, category, entry_date FROM erp_financial WHERE branch_id = $1 ORDER BY entry_date DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "kind": r.kind, "description": r.description, "amount": r.amount.to_string(),
        "category": r.category, "date": r.entry_date.to_string(),
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_inventory(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] sku: String,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] quantity: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)] unit_cost: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] location: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, sku, name, quantity, unit_cost, location FROM erp_inventory WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "sku": r.sku, "name": r.name, "quantity": r.quantity,
        "unit_cost": r.unit_cost.to_string(), "location": r.location,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_procurement(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] supplier: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] items: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<chrono::Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, supplier, items, total, status, created_at FROM erp_procurement WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "supplier": r.supplier, "items": r.items,
        "total": r.total.to_string(), "status": r.status, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_branches(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] address: String,
        #[diesel(sql_type = diesel::sql_types::Text)] manager: String,
        #[diesel(sql_type = diesel::sql_types::Bool)] active: bool,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, address, manager, active FROM erp_branches WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "address": r.address, "manager": r.manager, "active": r.active,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anonymous_callers_resolve_to_the_nil_branch() {
        // No JWT header: the handler still runs, constrained to the global
        // branch. It must never default to an arbitrary tenant.
        assert_eq!(resolve_branch(&HeaderMap::new()), Uuid::nil());
    }

    #[test]
    fn an_invalid_bearer_token_does_not_pick_a_tenant() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer not-a-jwt".parse().expect("header"));
        assert_eq!(resolve_branch(&headers), Uuid::nil());
    }

    #[test]
    fn the_router_exposes_the_erp_surface() {
        let _router: axum::Router<()> = crate::routes::configure();
    }
}
