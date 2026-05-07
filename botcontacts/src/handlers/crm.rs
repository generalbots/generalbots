use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::requests::*;
use crate::schema::{crm_deals, crm_activities, crm_pipeline_stages, crm_contacts, crm_accounts};
use crate::CrateState;

pub async fn list_deals(
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
        q = q.filter(crm_deals::title.ilike(pattern.clone()).or(crm_deals::name.ilike(pattern)));
    }

    if let Some(department_id) = query.department_id {
        q = q.filter(crm_deals::department_id.eq(department_id));
    }

    if let Some(source) = query.source {
        q = q.filter(crm_deals::source.eq(source));
    }

    if let Some(owner_id) = query.owner_id {
        q = q.filter(crm_deals::owner_id.eq(owner_id));
    }

    if let Some(status) = query.status {
        match status.as_str() {
            "open" => { q = q.filter(crm_deals::closed_at.is_null()); }
            "closed" => { q = q.filter(crm_deals::closed_at.is_not_null()); }
            "won" => { q = q.filter(crm_deals::won.eq(true)); }
            "lost" => { q = q.filter(crm_deals::won.eq(false)); }
            _ => {}
        }
    }

    let deals: Vec<CrmDeal> = q
        .order(crm_deals::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(deals))
}

pub async fn create_deal(
    State(state): State<Arc<CrateState>>,
    Json(req): Json<CreateDealRequest>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();
    let id = Uuid::new_v4();
    let now = Utc::now();

    let expected_close = req.expected_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let stage = req.stage.unwrap_or_else(|| "new".to_string());
    let probability = stage_probability(&stage);

    let deal = CrmDeal {
        id,
        org_id,
        bot_id,
        contact_id: req.contact_id,
        account_id: req.account_id,
        am_id: None,
        lead_id: None,
        owner_id: req.owner_id,
        title: req.title,
        name: req.name,
        description: req.description,
        value: req.value,
        currency: req.currency.or(Some("USD".to_string())),
        stage_id: None,
        stage: Some(stage.clone()),
        probability,
        source: req.source,
        segment_id: None,
        department_id: req.department_id,
        expected_close_date: expected_close,
        actual_close_date: None,
        period: None,
        deal_date: None,
        lost_reason: None,
        won: if stage == "won" { Some(true) } else if stage == "lost" { Some(false) } else { None },
        tags: req.tags.unwrap_or_default(),
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: Some(now),
        closed_at: if stage == "won" || stage == "lost" { Some(now) } else { None },
        notes: req.notes,
    };

    diesel::insert_into(crm_deals::table)
        .values(&deal)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert deal error: {e}")))?;

    Ok(Json(deal))
}

pub async fn get_deal(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmDeal>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let deal: CrmDeal = crm_deals::table
        .filter(crm_deals::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Deal not found".to_string()))?;

    Ok(Json(deal))
}

pub async fn update_deal(
    State(state): State<Arc<CrateState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDealRequest>,
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

    if let Some(name) = req.name {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(value) = req.value {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::value.eq(value))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(currency) = req.currency {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::currency.eq(currency))
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

    if let Some(department_id) = req.department_id {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::department_id.eq(department_id))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(owner_id) = req.owner_id {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::owner_id.eq(owner_id))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(lost_reason) = req.lost_reason {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::lost_reason.eq(lost_reason))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(won) = req.won {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set((crm_deals::won.eq(won), crm_deals::closed_at.eq(Some(now))))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(notes) = req.notes {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::notes.eq(notes))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(tags) = req.tags {
        diesel::update(crm_deals::table.filter(crm_deals::id.eq(id)))
            .set(crm_deals::tags.eq(tags))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_deal(State(state), Path(id)).await
}

pub async fn delete_deal(
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

pub async fn create_activity(
    State(state): State<Arc<CrateState>>,
    Json(req): Json<CreateActivityRequest>,
) -> Result<Json<CrmActivity>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();
    let id = Uuid::new_v4();
    let now = Utc::now();

    let due_date = req.due_date
        .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
        .map(|d| d.with_timezone(&Utc));

    let activity = CrmActivity {
        id,
        org_id,
        bot_id,
        contact_id: req.contact_id,
        lead_id: req.lead_id,
        opportunity_id: req.opportunity_id,
        account_id: req.account_id,
        activity_type: req.activity_type,
        subject: req.subject,
        description: req.description,
        due_date,
        completed_at: None,
        outcome: None,
        owner_id: None,
        created_at: now,
    };

    diesel::insert_into(crm_activities::table)
        .values(&activity)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(activity))
}

pub async fn list_activities(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmActivity>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let activities: Vec<CrmActivity> = crm_activities::table
        .filter(crm_activities::org_id.eq(org_id))
        .filter(crm_activities::bot_id.eq(bot_id))
        .order(crm_activities::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(activities))
}

pub async fn get_pipeline_stages(
    State(state): State<Arc<CrateState>>,
) -> Result<Json<Vec<CrmPipelineStage>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();

    let stages: Vec<CrmPipelineStage> = crm_pipeline_stages::table
        .filter(crm_pipeline_stages::org_id.eq(org_id))
        .filter(crm_pipeline_stages::bot_id.eq(bot_id))
        .order(crm_pipeline_stages::stage_order.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(stages))
}

pub async fn get_crm_stats(
    State(state): State<Arc<CrateState>>,
) -> Result<Json<CrmStats>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = state.get_bot_context();

    let total_contacts: i64 = crm_contacts::table
        .filter(crm_contacts::org_id.eq(org_id))
        .filter(crm_contacts::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_accounts: i64 = crm_accounts::table
        .filter(crm_accounts::org_id.eq(org_id))
        .filter(crm_accounts::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_leads: i64 = crm_deals::table
        .filter(crm_deals::org_id.eq(org_id))
        .filter(crm_deals::bot_id.eq(bot_id))
        .filter(crm_deals::closed_at.is_null())
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_opportunities: i64 = crm_deals::table
        .filter(crm_deals::org_id.eq(org_id))
        .filter(crm_deals::bot_id.eq(bot_id))
        .filter(crm_deals::won.is_null())
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let won_this_month: i64 = crm_deals::table
        .filter(crm_deals::org_id.eq(org_id))
        .filter(crm_deals::bot_id.eq(bot_id))
        .filter(crm_deals::won.eq(Some(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let stats = CrmStats {
        total_contacts,
        total_accounts,
        total_leads,
        total_opportunities,
        total_campaigns: 0,
        pipeline_value: 0.0,
        won_this_month,
        conversion_rate: if total_leads > 0 {
            (won_this_month as f64 / total_leads as f64) * 100.0
        } else {
            0.0
        },
    };

    Ok(Json(stats))
}
