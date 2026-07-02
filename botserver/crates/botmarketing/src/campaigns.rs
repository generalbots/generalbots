use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::marketing_campaigns;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = marketing_campaigns)]
pub struct CrmCampaign {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub name: String,
    pub campaign_type: String,
    pub status: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub budget: Option<bigdecimal::BigDecimal>,
    pub metrics: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub campaign_type: Option<String>,
    pub scheduled_at: Option<String>,
    pub budget: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub campaign_type: Option<String>,
    pub scheduled_at: Option<String>,
    pub budget: Option<f64>,
}

pub async fn list_campaigns(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CrmCampaign>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (branch_id, _) = state.get_bot_context();

    let campaigns: Vec<CrmCampaign> = marketing_campaigns::table
        .filter(marketing_campaigns::branch_id.eq(branch_id))
        .order(marketing_campaigns::created_at.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(campaigns))
}

pub async fn get_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmCampaign>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    Ok(Json(campaign))
}

pub async fn create_campaign(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<Json<CrmCampaign>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (branch_id, _) = state.get_bot_context();
    let id = Uuid::new_v4();
    let now = Utc::now();

    let starts_at = req.scheduled_at.and_then(|s| {
        DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
    });

    let campaign = CrmCampaign {
        id,
        branch_id,
        name: req.name,
        campaign_type: req.campaign_type.unwrap_or_else(|| "email".to_string()),
        status: Some("draft".to_string()),
        starts_at,
        ends_at: None,
        budget: req.budget.map(|b| bigdecimal::BigDecimal::try_from(b).unwrap_or_default()),
        metrics: Some(serde_json::json!({
            "sent": 0,
            "delivered": 0,
            "failed": 0,
            "opened": 0,
            "clicked": 0,
            "replied": 0
        })),
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(marketing_campaigns::table)
        .values(&campaign)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(campaign))
}

pub async fn update_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> Result<Json<CrmCampaign>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
        .set(marketing_campaigns::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(name) = req.name {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(status) = req.status {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::status.eq(Some(status)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(campaign_type) = req.campaign_type {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::campaign_type.eq(campaign_type))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(scheduled) = req.scheduled_at {
        let dt = DateTime::parse_from_rfc3339(&scheduled)
            .ok()
            .map(|d| d.with_timezone(&Utc));
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::starts_at.eq(dt))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(budget) = req.budget {
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
            .set(marketing_campaigns::budget.eq(bigdecimal::BigDecimal::try_from(budget).unwrap_or_default()))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_campaign(State(state), Path(id)).await
}

pub async fn delete_campaign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

#[derive(Debug, Deserialize)]
pub struct SendCampaignRequest {
    pub list_id: Option<Uuid>,
    pub contact_ids: Option<Vec<Uuid>>,
    pub template_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignSendResult {
    pub campaign_id: Uuid,
    pub total_recipients: i32,
    pub sent: i32,
    pub failed: i32,
    pub pending: i32,
}


pub async fn send_campaign(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Stub — campaign execution will be implemented in future
    let _ = (&state, campaign_id);
    Err((StatusCode::NOT_IMPLEMENTED, "Campaign execution not implemented yet".to_string()))
}
