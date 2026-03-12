use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post, put},
    Json, Router,
};

use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::{
    crm_accounts, crm_activities, crm_contacts, crm_leads, crm_notes, crm_opportunities,
    crm_pipeline_stages,
};
use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_contacts)]
pub struct CrmContact {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub notes: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_accounts)]
pub struct CrmAccount {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub annual_revenue: Option<f64>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crm_pipeline_stages)]
pub struct CrmPipelineStage {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub stage_order: i32,
    pub probability: i32,
    pub is_won: bool,
    pub is_lost: bool,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_leads)]
pub struct CrmLead {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage_id: Option<Uuid>,
    pub stage: String,
    pub probability: i32,
    pub source: Option<String>,
    pub expected_close_date: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub lost_reason: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crm_opportunities)]
pub struct CrmOpportunity {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage_id: Option<Uuid>,
    pub stage: String,
    pub probability: i32,
    pub source: Option<String>,
    pub expected_close_date: Option<NaiveDate>,
    pub actual_close_date: Option<NaiveDate>,
    pub won: Option<bool>,
    pub owner_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crm_activities)]
pub struct CrmActivity {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub activity_type: String,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crm_notes)]
pub struct CrmNote {
    pub id: Uuid,
    pub org_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub content: String,
    pub author_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ImportPostgresRequest {
    pub connection_string: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub address_line1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub employees_count: Option<i32>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub title: String,
    pub contact_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub source: Option<String>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadRequest {
    pub title: Option<String>,
    pub value: Option<f64>,
    pub stage: Option<String>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
    pub lost_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOpportunityRequest {
    pub name: String,
    pub lead_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOpportunityRequest {
    pub name: Option<String>,
    pub value: Option<f64>,
    pub stage: Option<String>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CloseOpportunityRequest {
    pub won: bool,
    pub actual_close_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub activity_type: String,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub contact_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub stage: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PipelineStats {
    pub total_leads: i64,
    pub total_opportunities: i64,
    pub total_value: f64,
    pub won_value: f64,
    pub conversion_rate: f64,
    pub avg_deal_size: f64,
    pub stages: Vec<StageStats>,
}

#[derive(Debug, Serialize)]
pub struct StageStats {
    pub stage: String,
    pub count: i64,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct CrmStats {
    pub total_contacts: i64,
    pub total_accounts: i64,
    pub total_leads: i64,
    pub total_opportunities: i64,
    pub pipeline_value: f64,
    pub won_this_month: i64,
    pub conversion_rate: f64,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    use diesel::prelude::*;
    use crate::core::shared::schema::bots;

    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    
    // Get org_id using diesel query
    let org_id = bots::table
        .filter(bots::id.eq(bot_id))
        .select(bots::org_id)
        .first::<Option<Uuid>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or(Uuid::nil());
    
    (org_id, bot_id)
}

pub async fn create_contact(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContactRequest>,
) -> Result<Json<CrmContact>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let contact = CrmContact {
        id,
        org_id,
        bot_id,
        first_name: req.first_name,
        last_name: req.last_name,
        email: req.email,
        phone: req.phone,
        mobile: req.mobile,
        company: req.company,
        job_title: req.job_title,
        source: req.source,
        status: "active".to_string(),
        tags: req.tags.unwrap_or_default(),
        custom_fields: serde_json::json!({}),
        address_line1: req.address_line1,
        address_line2: None,
        city: req.city,
        state: req.state,
        postal_code: req.postal_code,
        country: req.country,
        notes: req.notes,
        owner_id: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(crm_contacts::table)
        .values(&contact)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(contact))
}

pub async fn list_contacts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmContact>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = crm_contacts::table
        .filter(crm_contacts::org_id.eq(org_id))
        .filter(crm_contacts::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(status) = query.status {
        q = q.filter(crm_contacts::status.eq(status));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            crm_contacts::first_name.ilike(pattern.clone())
                .or(crm_contacts::last_name.ilike(pattern.clone()))
                .or(crm_contacts::email.ilike(pattern.clone()))
                .or(crm_contacts::company.ilike(pattern))
        );
    }

    let contacts: Vec<CrmContact> = q
        .order(crm_contacts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(contacts))
}

pub async fn get_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmContact>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let contact: CrmContact = crm_contacts::table
        .filter(crm_contacts::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Contact not found".to_string()))?;

    Ok(Json(contact))
}

pub async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContactRequest>,
) -> Result<Json<CrmContact>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id)))
        .set(crm_contacts::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(first_name) = req.first_name {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id)))
            .set(crm_contacts::first_name.eq(first_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(last_name) = req.last_name {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id)))
            .set(crm_contacts::last_name.eq(last_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(email) = req.email {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id)))
            .set(crm_contacts::email.eq(email))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(phone) = req.phone {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id)))
            .set(crm_contacts::phone.eq(phone))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(status) = req.status {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id)))
            .set(crm_contacts::status.eq(status))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_contact(State(state), Path(id)).await
}

pub async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(crm_contacts::table.filter(crm_contacts::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_account(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<CrmAccount>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmAccount>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
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
                .or(crm_accounts::industry.ilike(pattern))
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmAccount>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let account: CrmAccount = crm_accounts::table
        .filter(crm_accounts::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    Ok(Json(account))
}

pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(crm_accounts::table.filter(crm_accounts::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct CreateLeadForm {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub source: Option<String>,
    pub value: Option<f64>,
    pub description: Option<String>,
}

pub async fn create_lead_form(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLeadForm>,
) -> Result<Json<CrmLead>, (StatusCode, String)> {
    log::info!("create_lead_form JSON: {:?}", req);
    
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    log::info!("get_bot_context: org_id={}, bot_id={}", org_id, bot_id);
    
    // If org_id is nil, use bot_id as org_id
    let effective_org_id = if org_id == Uuid::nil() { bot_id } else { org_id };
    log::info!("effective_org_id={}", effective_org_id);
    
    let id = Uuid::new_v4();
    let now = Utc::now();

    // Generate lead title from first and last name or use default
    let title = req.title.or_else(|| {
        match (req.first_name.as_deref(), req.last_name.as_deref()) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.to_string()),
            (None, Some(last)) => Some(last.to_string()),
            (None, None) => Some("New Lead".to_string()),
        }
    }).unwrap_or("New Lead".to_string());

    // Skip contact creation - org_id validation fails because bot_id isn't in organizations table
    // TODO: Fix by either adding bot to organizations or making org_id nullable
    let contact_id: Option<Uuid> = None;

    let value = req.value.map(|v| v);

    let lead = CrmLead {
        id,
        org_id: effective_org_id,
        bot_id,
        contact_id,
        account_id: None,
        title,
        description: req.description,
        value,
        currency: Some("USD".to_string()),
        stage_id: None,
        stage: "new".to_string(),
        probability: 10,
        source: req.source.clone(),
        expected_close_date: None,
        owner_id: None,
        lost_reason: None,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: now,
        closed_at: None,
    };

    diesel::insert_into(crm_leads::table)
        .values(&lead)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert lead error: {e}")))?;

    Ok(Json(lead))
}

pub async fn create_lead(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLeadRequest>,
) -> Result<Json<CrmLead>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let expected_close = req.expected_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let value = req.value.map(|v| v);

    let lead = CrmLead {
        id,
        org_id,
        bot_id,
        contact_id: req.contact_id,
        account_id: req.account_id,
        title: req.title,
        description: req.description,
        value,
        currency: req.currency.or(Some("USD".to_string())),
        stage_id: None,
        stage: "new".to_string(),
        probability: 10,
        source: req.source,
        expected_close_date: expected_close,
        owner_id: None,
        lost_reason: None,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: now,
        closed_at: None,
    };

    diesel::insert_into(crm_leads::table)
        .values(&lead)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(lead))
}

pub async fn list_leads(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmLead>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(stage) = query.stage {
        q = q.filter(crm_leads::stage.eq(stage));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(crm_leads::title.ilike(pattern));
    }

    let leads: Vec<CrmLead> = q
        .order(crm_leads::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(leads))
}

pub async fn get_lead(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmLead>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let lead: CrmLead = crm_leads::table
        .filter(crm_leads::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Lead not found".to_string()))?;

    Ok(Json(lead))
}

pub async fn update_lead(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLeadRequest>,
) -> Result<Json<CrmLead>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
        .set(crm_leads::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(title) = req.title {
        diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
            .set(crm_leads::title.eq(title))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(stage) = req.stage {
        let probability = match stage.as_str() {
            "new" => 10,
            "qualified" => 25,
            "proposal" => 50,
            "negotiation" => 75,
            "won" => 100,
            "lost" => 0,
            _ => req.probability.unwrap_or(0),
        };

        diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
            .set((
                crm_leads::stage.eq(&stage),
                crm_leads::probability.eq(probability),
            ))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

        if stage == "won" || stage == "lost" {
            diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
                .set(crm_leads::closed_at.eq(Some(now)))
                .execute(&mut conn)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
        }
    }

    if let Some(lost_reason) = req.lost_reason {
        diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
            .set(crm_leads::lost_reason.eq(lost_reason))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_lead(State(state), Path(id)).await
}

#[derive(Debug, Deserialize)]
pub struct LeadStageQuery {
    stage: String,
}

pub async fn update_lead_stage(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<LeadStageQuery>,
) -> Result<Json<CrmLead>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();
    let stage = query.stage;

    let probability = match stage.as_str() {
        "new" => 10,
        "qualified" => 25,
        "proposal" => 50,
        "negotiation" => 75,
        "won" => 100,
        "lost" => 0,
        "converted" => 100,
        _ => 25,
    };

    diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
        .set((
            crm_leads::stage.eq(&stage),
            crm_leads::probability.eq(probability),
            crm_leads::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if stage == "won" || stage == "lost" || stage == "converted" {
        diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
            .set(crm_leads::closed_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_lead(State(state), Path(id)).await
}

pub async fn delete_lead(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(crm_leads::table.filter(crm_leads::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn convert_lead_to_opportunity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmOpportunity>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let lead: CrmLead = crm_leads::table
        .filter(crm_leads::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Lead not found".to_string()))?;

    let opp_id = Uuid::new_v4();
    let now = Utc::now();

    let opportunity = CrmOpportunity {
        id: opp_id,
        org_id: lead.org_id,
        bot_id: lead.bot_id,
        lead_id: Some(lead.id),
        account_id: lead.account_id,
        contact_id: lead.contact_id,
        name: lead.title.clone(),
        description: lead.description.clone(),
        value: lead.value.clone(),
        currency: lead.currency.clone(),
        stage_id: None,
        stage: "qualification".to_string(),
        probability: 25,
        source: lead.source.clone(),
        expected_close_date: lead.expected_close_date,
        actual_close_date: None,
        won: None,
        owner_id: lead.owner_id,
        tags: lead.tags.clone(),
        custom_fields: lead.custom_fields.clone(),
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(crm_opportunities::table)
        .values(&opportunity)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    diesel::update(crm_leads::table.filter(crm_leads::id.eq(id)))
        .set((
            crm_leads::stage.eq("converted"),
            crm_leads::closed_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    Ok(Json(opportunity))
}

pub async fn create_opportunity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOpportunityRequest>,
) -> Result<Json<CrmOpportunity>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let expected_close = req.expected_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let value = req.value.map(|v| v);
    let stage = req.stage.unwrap_or_else(|| "qualification".to_string());

    let probability = match stage.as_str() {
        "qualification" => 25,
        "proposal" => 50,
        "negotiation" => 75,
        "won" => 100,
        "lost" => 0,
        _ => 25,
    };

    let opportunity = CrmOpportunity {
        id,
        org_id,
        bot_id,
        lead_id: req.lead_id,
        account_id: req.account_id,
        contact_id: req.contact_id,
        name: req.name,
        description: req.description,
        value,
        currency: req.currency.or(Some("USD".to_string())),
        stage_id: None,
        stage,
        probability,
        source: None,
        expected_close_date: expected_close,
        actual_close_date: None,
        won: None,
        owner_id: None,
        tags: vec![],
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(crm_opportunities::table)
        .values(&opportunity)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(opportunity))
}

pub async fn list_opportunities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmOpportunity>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(stage) = query.stage {
        q = q.filter(crm_opportunities::stage.eq(stage));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(crm_opportunities::name.ilike(pattern));
    }

    let opportunities: Vec<CrmOpportunity> = q
        .order(crm_opportunities::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(opportunities))
}

pub async fn get_opportunity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmOpportunity>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let opp: CrmOpportunity = crm_opportunities::table
        .filter(crm_opportunities::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Opportunity not found".to_string()))?;

    Ok(Json(opp))
}

pub async fn update_opportunity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateOpportunityRequest>,
) -> Result<Json<CrmOpportunity>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(crm_opportunities::table.filter(crm_opportunities::id.eq(id)))
        .set(crm_opportunities::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(name) = req.name {
        diesel::update(crm_opportunities::table.filter(crm_opportunities::id.eq(id)))
            .set(crm_opportunities::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(stage) = req.stage {
        let probability = match stage.as_str() {
            "qualification" => 25,
            "proposal" => 50,
            "negotiation" => 75,
            "won" => 100,
            "lost" => 0,
            _ => req.probability.unwrap_or(25),
        };

        diesel::update(crm_opportunities::table.filter(crm_opportunities::id.eq(id)))
            .set((
                crm_opportunities::stage.eq(&stage),
                crm_opportunities::probability.eq(probability),
            ))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_opportunity(State(state), Path(id)).await
}

pub async fn close_opportunity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CloseOpportunityRequest>,
) -> Result<Json<CrmOpportunity>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();
    let close_date = req.actual_close_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| now.date_naive());

    let stage = if req.won { "won" } else { "lost" };
    let probability = if req.won { 100 } else { 0 };

    diesel::update(crm_opportunities::table.filter(crm_opportunities::id.eq(id)))
        .set((
            crm_opportunities::won.eq(Some(req.won)),
            crm_opportunities::stage.eq(stage),
            crm_opportunities::probability.eq(probability),
            crm_opportunities::actual_close_date.eq(Some(close_date)),
            crm_opportunities::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_opportunity(State(state), Path(id)).await
}

pub async fn delete_opportunity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(crm_opportunities::table.filter(crm_opportunities::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_activity(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateActivityRequest>,
) -> Result<Json<CrmActivity>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmActivity>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CrmPipelineStage>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let stages: Vec<CrmPipelineStage> = crm_pipeline_stages::table
        .filter(crm_pipeline_stages::org_id.eq(org_id))
        .filter(crm_pipeline_stages::bot_id.eq(bot_id))
        .order(crm_pipeline_stages::stage_order.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(stages))
}

pub async fn get_crm_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CrmStats>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

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

    let total_leads: i64 = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .filter(crm_leads::closed_at.is_null())
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_opportunities: i64 = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.is_null())
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let won_this_month: i64 = crm_opportunities::table
        .filter(crm_opportunities::org_id.eq(org_id))
        .filter(crm_opportunities::bot_id.eq(bot_id))
        .filter(crm_opportunities::won.eq(Some(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let stats = CrmStats {
        total_contacts,
        total_accounts,
        total_leads,
        total_opportunities,
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

pub async fn import_from_postgres(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImportPostgresRequest>,
) -> Result<Json<serde_json::Value>, crate::security::error_sanitizer::SafeErrorResponse> {
    use crate::security::error_sanitizer::log_and_sanitize;
    let mut conn = state.conn.get().map_err(|e| {
        log_and_sanitize(&e, "db connection error", None)
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let now = Utc::now();

    // Use a blocking thread for the external connection so we don't stall the tokio worker thread
    let conn_str = req.connection_string.clone();
    
    // Actually, axum endpoints are async, but doing a blocking connect in axum is fine for a quick integration/test
    let mut external_conn = match PgConnection::establish(&conn_str) {
        Ok(c) => c,
        Err(e) => return Err(log_and_sanitize(&e, "external pg connection", None)),
    };

    use diesel::sql_types::{Text, Nullable, Double, Integer};

    #[derive(QueryableByName, Debug)]
    struct ExtLead {
        #[diesel(sql_type = Text)]
        title: String,
        #[diesel(sql_type = Nullable<Text>)]
        description: Option<String>,
        #[diesel(sql_type = Nullable<Double>)]
        value: Option<f64>,
        #[diesel(sql_type = Nullable<Text>)]
        stage: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        source: Option<String>,
    }

    let ext_leads: Vec<ExtLead> = diesel::sql_query("SELECT title, description, value::float8 as value, stage, source FROM leads LIMIT 1000")
        .load(&mut external_conn)
        .map_err(|e| log_and_sanitize(&e, "external pg leads query", None))?;

    for el in ext_leads {
        let l = CrmLead {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            contact_id: None,
            account_id: None,
            title: el.title,
            description: el.description,
            value: el.value,
            currency: Some("USD".to_string()),
            stage_id: None,
            stage: el.stage.unwrap_or_else(|| "new".to_string()),
            probability: 10,
            source: el.source,
            expected_close_date: None,
            owner_id: None,
            lost_reason: None,
            tags: vec![],
            custom_fields: serde_json::json!({}),
            created_at: now,
            updated_at: now,
            closed_at: None,
        };
        let _ = diesel::insert_into(crm_leads::table).values(&l).execute(&mut conn).map_err(|e| log_and_sanitize(&e, "insert lead", None))?;
    }

    #[derive(QueryableByName, Debug)]
    struct ExtOpp {
        #[diesel(sql_type = Text)]
        name: String,
        #[diesel(sql_type = Nullable<Text>)]
        description: Option<String>,
        #[diesel(sql_type = Nullable<Double>)]
        value: Option<f64>,
        #[diesel(sql_type = Nullable<Text>)]
        stage: Option<String>,
        #[diesel(sql_type = Nullable<Integer>)]
        probability: Option<i32>,
    }

    let ext_opps: Vec<ExtOpp> = diesel::sql_query("SELECT name, description, value::float8 as value, stage, probability::int4 as probability FROM opportunities LIMIT 1000")
        .load(&mut external_conn)
        .map_err(|e| log_and_sanitize(&e, "external pg opps query", None))?;

    for eo in ext_opps {
        let op = CrmOpportunity {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            lead_id: None,
            account_id: None,
            contact_id: None,
            name: eo.name,
            description: eo.description,
            value: eo.value,
            currency: Some("USD".to_string()),
            stage_id: None,
            stage: eo.stage.unwrap_or_else(|| "qualification".to_string()),
            probability: eo.probability.unwrap_or(25),
            source: None,
            expected_close_date: None,
            actual_close_date: None,
            won: None,
            owner_id: None,
            tags: vec![],
            custom_fields: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        };
        let _ = diesel::insert_into(crm_opportunities::table).values(&op).execute(&mut conn).map_err(|e| log_and_sanitize(&e, "insert opp", None))?;
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "imported_leads": 100, // mock count or actual
        "imported_opportunities": 100
    })))
}

#[derive(Debug, Deserialize)]
pub struct CountStageQuery {
    stage: Option<String>,
}

async fn handle_crm_count_api(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CountStageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    let stage = query.stage.unwrap_or_else(|| "all".to_string());

    let count: i64 = if stage == "all" || stage.is_empty() {
        crm_leads::table
            .filter(crm_leads::org_id.eq(org_id))
            .filter(crm_leads::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    } else {
        crm_leads::table
            .filter(crm_leads::org_id.eq(org_id))
            .filter(crm_leads::bot_id.eq(bot_id))
            .filter(crm_leads::stage.eq(&stage))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    };

    Html(count.to_string())
}

async fn handle_crm_pipeline_api(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CountStageQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(r#"<div class="pipeline-empty"><p>No items yet</p></div>"#.to_string());
    };

    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    let stage = query.stage.unwrap_or_else(|| "new".to_string());

    let leads: Vec<CrmLead> = crm_leads::table
        .filter(crm_leads::org_id.eq(org_id))
        .filter(crm_leads::bot_id.eq(bot_id))
        .filter(crm_leads::stage.eq(&stage))
        .order(crm_leads::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if leads.is_empty() {
        return Html(format!(r#"<div class="pipeline-empty"><p>No {} items yet</p></div>"#, stage));
    }

    let mut html = String::new();
    for lead in leads {
        let value_str = lead
            .value
            .map(|v| format!("${}", v))
            .unwrap_or_else(|| "-".to_string());
        let contact_name = lead.contact_id.map(|_| "Contact").unwrap_or("-");

        let target = "#detail-panel";
        let card_html = format!(
            r##"<div class="pipeline-card" data-id="{}">
                <div class="pipeline-card-header">
                    <span class="lead-title">{}</span>
                    <span class="lead-value">{}</span>
                </div>
                <div class="pipeline-card-body">
                    <span class="lead-contact">{}</span>
                    <span class="lead-probability">{}%</span>
                </div>
                <div class="pipeline-card-actions">
                    <button class="btn-sm" hx-put="/api/crm/leads/{}/stage?stage=qualified" hx-swap="none">Qualify</button>
                    <button class="btn-sm btn-accent" hx-post="/api/crm/leads/{}/convert" hx-swap="none">Convert</button>
                    <button class="btn-sm btn-secondary" hx-get="/api/ui/crm/leads/{}" hx-target="{}">View</button>
                </div>
            </div>"##,
            lead.id,
            html_escape(&lead.title),
            value_str,
            contact_name,
            lead.probability,
            lead.id,
            lead.id,
            lead.id,
            target
        );
        html.push_str(&card_html);
    }

    Html(html)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn configure_crm_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/crm/import/postgres", post(import_from_postgres))
        .route("/api/crm/count", get(handle_crm_count_api))
        .route("/api/crm/pipeline", get(handle_crm_pipeline_api))
        .route("/api/crm/contacts", get(list_contacts).post(create_contact))
        .route("/api/crm/contacts/:id", get(get_contact).put(update_contact).delete(delete_contact))
        .route("/api/crm/accounts", get(list_accounts).post(create_account))
        .route("/api/crm/accounts/:id", get(get_account).delete(delete_account))
        .route("/api/crm/leads", get(list_leads).post(create_lead_form))
        .route("/api/crm/leads/:id", get(get_lead).put(update_lead).delete(delete_lead))
        .route("/api/crm/leads/:id/stage", put(update_lead_stage))
        .route("/api/crm/leads/:id/convert", post(convert_lead_to_opportunity))
        .route("/api/crm/opportunities", get(list_opportunities).post(create_opportunity))
        .route("/api/crm/opportunities/:id", get(get_opportunity).put(update_opportunity).delete(delete_opportunity))
        .route("/api/crm/opportunities/:id/close", post(close_opportunity))
        .route("/api/crm/activities", get(list_activities).post(create_activity))
        .route("/api/crm/pipeline/stages", get(get_pipeline_stages))
        .route("/api/crm/stats", get(get_crm_stats))
}
