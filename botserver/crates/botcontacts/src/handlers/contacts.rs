use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::requests::*;
use crate::schema::crm_contacts;
use crate::scope::branch_from_jwt;
use crate::CrateState;

fn get_bot_context(state: &CrateState) -> Uuid {
    state.get_bot_context()
}

pub async fn create_contact(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Json(req): Json<CreateContactRequest>,
) -> Result<Json<CrmContact>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| get_bot_context(&state));
    let id = Uuid::new_v4();
    let now = Utc::now();

    let contact = CrmContact {
        id,
        org_id: branch_id,
        bot_id: state.bot_for_branch(branch_id),
        branch_id,
        first_name: req.first_name,
        last_name: req.last_name,
        email: req.email,
        phone: req.phone,
        mobile: req.mobile,
        company: req.company,
        job_title: req.job_title,
        source: req.source,
        status: Some("active".to_string()),
        tags: req.tags,
        custom_fields: Some(serde_json::json!({})),
        notes: req.notes,
        owner_id: None,
        pass_hash: None,
        created_at: now,
        updated_at: now,
        address_line1: req.address_line1,
        address_line2: None,
        city: req.city,
        state: req.state,
        postal_code: req.postal_code,
        country: req.country,
    };

    diesel::insert_into(crm_contacts::table)
        .values(&contact)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    (state.trigger_contact_change)(&mut conn, id, "created", branch_id);

    Ok(Json(contact))
}

pub async fn list_contacts(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmContact>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| get_bot_context(&state));
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
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
                .or(crm_contacts::company.ilike(pattern)),
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
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<CrmContact>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| get_bot_context(&state));

    let contact: CrmContact = crm_contacts::table
        .filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Contact not found".to_string()))?;

    Ok(Json(contact))
}

pub async fn update_contact(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContactRequest>,
) -> Result<Json<CrmContact>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| get_bot_context(&state));

    let now = Utc::now();

    diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
        .set(crm_contacts::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(first_name) = req.first_name {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
            .set(crm_contacts::first_name.eq(first_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(last_name) = req.last_name {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
            .set(crm_contacts::last_name.eq(last_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(email) = req.email {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
            .set(crm_contacts::email.eq(email))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(phone) = req.phone {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
            .set(crm_contacts::phone.eq(phone))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(status) = req.status {
        diesel::update(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
            .set(crm_contacts::status.eq(status))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_contact(State(state), headers, Path(id)).await
}

pub async fn delete_contact(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| get_bot_context(&state));

    diesel::delete(crm_contacts::table.filter(crm_contacts::id.eq(id))
                .filter(crm_contacts::branch_id.eq(branch_id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}
