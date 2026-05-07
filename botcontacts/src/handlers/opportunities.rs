use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::requests::*;
use crate::schema::crm_deals;
use crate::CrateState;

pub async fn create_opportunity(
    State(state): State<Arc<CrateState>>,
    Json(req): Json<CreateOpportunityRequest>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();
    let id = Uuid::new_v4();
    let now = Utc::now();

    let expected_close = req.expected_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let stage = req.stage.unwrap_or_else(|| "qualification".to_string());
    let probability = stage_probability(&stage);

    let opportunity = CrmDeal {
        id,
        org_id,
        bot_id,
        lead_id: req.lead_id,
        account_id: req.account_id,
        contact_id: req.contact_id,
        am_id: None,
        title: None,
        name: Some(req.name),
        description: req.description,
        value: req.value,
        currency: req.currency.or(Some("USD".to_string())),
        stage_id: None,
        stage: Some(stage),
        probability,
        source: None,
        segment_id: None,
        department_id: None,
        expected_close_date: expected_close,
        actual_close_date: None,
        period: None,
        deal_date: None,
        won: None,
        owner_id: None,
        lost_reason: None,
        closed_at: None,
        notes: None,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: Some(now),
    };

    diesel::insert_into(crm_deals::table)
        .values(&opportunity)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(opportunity))
}

pub async fn list_opportunities(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmDeal>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = crm_deals::table
        .filter(crm_deals::org_id.eq(org_id))
        .filter(crm_deals::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(stage) = query.stage {
        q = q.filter(crm_deals::stage.eq(stage));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(crm_deals::name.ilike(pattern));
    }

    if let Some(department_id) = query.department_id {
        q = q.filter(crm_deals::department_id.eq(department_id));
    }

    if let Some(source) = query.source {
        q = q.filter(crm_deals::source.eq(source));
    }

    let opportunities: Vec<CrmDeal> = q
        .order(crm_deals::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(opportunities))
}

pub async fn get_opportunity(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let opp: CrmDeal = crm_deals::table
        .filter(crm_deals::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Opportunity not found".to_string()))?;

    Ok(Json(opp))
}

pub async fn update_opportunity(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateOpportunityRequest>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
        .set(crm_deals::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(name) = req.name {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(stage) = req.stage {
        let probability = stage_probability(&stage);
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set((crm_deals::stage.eq(&stage), crm_deals::probability.eq(probability)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_opportunity(State(state), Path(id)).await
}

pub async fn close_opportunity(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CloseOpportunityRequest>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();
    let close_date = req.actual_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| now.date_naive());

    let stage = if req.won { "won" } else { "lost" };
    let probability = if req.won { 100 } else { 0 };

    diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
        .set((
            crm_deals::won.eq(Some(req.won)),
            crm_deals::stage.eq(stage),
            crm_deals::probability.eq(probability),
            crm_deals::actual_close_date.eq(Some(close_date)),
            crm_deals::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_opportunity(State(state), Path(id)).await
}

pub async fn delete_opportunity(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(crm_deals::table.filter(crm_deals::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}
