use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::requests::*;
use crate::schema::crm_accounts;
use crate::CrateState;

fn get_bot_context(state: &CrateState) -> (Uuid, Uuid) {
    state.get_bot_context()
}

pub async fn create_account(
    State(state): State<Arc<CrateState>>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<CrmAccount>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let account = CrmAccount {
        id,
        org_id,
        bot_id,
        name: req.name,
        website: req.website,
        industry: req.industry,
        employees_count: req.employees_count,
        annual_revenue: None,
        phone: req.phone,
        email: req.email,
        address_line1: None,
        address_line2: None,
        city: None,
        state: None,
        postal_code: None,
        country: None,
        description: req.description,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        owner_id: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(crm_accounts::table)
        .values(&account)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(account))
}

pub async fn list_accounts(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmAccount>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = crm_accounts::table
        .filter(crm_accounts::org_id.eq(org_id))
        .filter(crm_accounts::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            crm_accounts::name.ilike(pattern.clone())
                .or(crm_accounts::industry.ilike(pattern)),
        );
    }

    let accounts: Vec<CrmAccount> = q
        .order(crm_accounts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(accounts))
}

pub async fn get_account(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmAccount>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let account: CrmAccount = crm_accounts::table
        .filter(crm_accounts::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    Ok(Json(account))
}

pub async fn delete_account(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(crm_accounts::table.filter(crm_accounts::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}
