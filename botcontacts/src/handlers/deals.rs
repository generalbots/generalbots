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

fn get_bot_context(state: &CrateState) -> (Uuid, Uuid) {
    state.get_bot_context()
}

pub async fn create_lead_form(
    State(state): State<Arc<CrateState>>,
    Json(req): Json<CreateLeadForm>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let effective_org_id = if org_id == Uuid::nil() { bot_id } else { org_id };
    let id = Uuid::new_v4();
    let now = Utc::now();

    let title = req.title.or_else(|| {
        match (req.first_name.as_deref(), req.last_name.as_deref()) {
            (Some(first), Some(last)) => Some(format!("{first} {last}")),
            (Some(first), None) => Some(first.to_string()),
            (None, Some(last)) => Some(last.to_string()),
            (None, None) => Some("New Lead".to_string()),
        }
    }).unwrap_or_else(|| "New Lead".to_string());

    let lead = CrmDeal {
        id,
        org_id: effective_org_id,
        bot_id,
        contact_id: None,
        account_id: None,
        am_id: None,
        lead_id: None,
        title: Some(title),
        name: None,
        description: req.description,
        value: req.value,
        currency: Some("USD".to_string()),
        stage_id: None,
        stage: Some("new".to_string()),
        probability: 10,
        source: req.source.clone(),
        segment_id: None,
        department_id: None,
        expected_close_date: None,
        actual_close_date: None,
        period: None,
        deal_date: None,
        owner_id: None,
        lost_reason: None,
        won: None,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: Some(now),
        closed_at: None,
        notes: None,
    };

    diesel::insert_into(crm_deals::table)
        .values(&lead)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert lead error: {e}")))?;

    Ok(Json(lead))
}

pub async fn create_lead(
    State(state): State<Arc<CrateState>>,
    Json(req): Json<CreateLeadRequest>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let expected_close = req.expected_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let lead = CrmDeal {
        id,
        org_id,
        bot_id,
        contact_id: req.contact_id,
        account_id: req.account_id,
        am_id: None,
        lead_id: None,
        title: Some(req.title),
        name: None,
        description: req.description,
        value: req.value,
        currency: req.currency.or(Some("USD".to_string())),
        stage_id: None,
        stage: Some("new".to_string()),
        probability: 10,
        source: req.source,
        segment_id: None,
        department_id: None,
        expected_close_date: expected_close,
        actual_close_date: None,
        period: None,
        deal_date: None,
        owner_id: None,
        lost_reason: None,
        won: None,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: Some(now),
        closed_at: None,
        notes: None,
    };

    diesel::insert_into(crm_deals::table)
        .values(&lead)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(lead))
}

pub async fn list_leads(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmDeal>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
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
        q = q.filter(crm_deals::title.ilike(pattern));
    }

    if let Some(department_id) = query.department_id {
        q = q.filter(crm_deals::department_id.eq(department_id));
    }

    if let Some(source) = query.source {
        q = q.filter(crm_deals::source.eq(source));
    }

    let leads: Vec<CrmDeal> = q
        .order(crm_deals::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(leads))
}

pub async fn get_lead(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let lead: CrmDeal = crm_deals::table
        .filter(crm_deals::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Lead not found".to_string()))?;

    Ok(Json(lead))
}

pub async fn update_lead(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLeadRequest>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
        .set(crm_deals::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(title) = req.title {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::title.eq(title))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(stage) = req.stage {
        let probability = stage_probability(&stage);
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set((crm_deals::stage.eq(&stage), crm_deals::probability.eq(probability)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

        if stage == "won" || stage == "lost" {
            diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
                .set(crm_deals::closed_at.eq(Some(now)))
                .execute(&mut conn)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
        }
    }

    if let Some(lost_reason) = req.lost_reason {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::lost_reason.eq(lost_reason))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_lead(State(state), Path(id)).await
}

pub async fn update_lead_stage(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<LeadStageQuery>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();
    let stage = query.stage;

    let old_stage: Option<Option<String>> = crm_deals::table
        .filter(crm_deals::id.eq(id))
        .select(crm_deals::stage)
        .first(&mut conn)
        .ok();

    let old_stage_str = old_stage.flatten();
    let probability = stage_probability(&stage);

    diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
        .set((
            crm_deals::stage.eq(&stage),
            crm_deals::probability.eq(probability),
            crm_deals::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if stage == "won" || stage == "lost" || stage == "converted" {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::closed_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(old) = old_stage_str {
        if old != stage {
            let (_org_id, bot_id) = get_bot_context(&state);
            (state.trigger_deal_stage_change)(&mut conn, id, &old, &stage, bot_id);
        }
    }

    get_lead(State(state), Path(id)).await
}

pub async fn delete_lead(
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

pub async fn convert_lead_to_opportunity(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let lead: CrmDeal = crm_deals::table
        .filter(crm_deals::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Lead not found".to_string()))?;

    let opp_id = Uuid::new_v4();
    let now = Utc::now();

    let opportunity = CrmDeal {
        id: opp_id,
        org_id: lead.org_id,
        bot_id: lead.bot_id,
        lead_id: Some(lead.id),
        account_id: lead.account_id,
        contact_id: lead.contact_id,
        am_id: None,
        title: lead.title.clone(),
        name: lead.title.clone(),
        description: lead.description.clone(),
        value: lead.value,
        currency: lead.currency.clone(),
        stage_id: None,
        stage: Some("qualification".to_string()),
        probability: 25,
        source: lead.source.clone(),
        segment_id: None,
        department_id: None,
        expected_close_date: lead.expected_close_date,
        actual_close_date: None,
        period: None,
        deal_date: None,
        won: None,
        owner_id: lead.owner_id,
        lost_reason: None,
        closed_at: None,
        notes: None,
        tags: lead.tags.clone(),
        custom_fields: lead.custom_fields.clone(),
        created_at: now,
        updated_at: Some(now),
    };

    diesel::insert_into(crm_deals::table)
        .values(&opportunity)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
        .set((crm_deals::stage.eq("converted"), crm_deals::closed_at.eq(Some(now))))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    Ok(Json(opportunity))
}
