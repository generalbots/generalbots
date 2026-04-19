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

use crate::core::shared::schema::marketing_lists;
use crate::core::shared::state::AppState;
use crate::core::bot::get_default_bot;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = marketing_lists)]
pub struct MarketingList {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub list_type: String,
    pub query_text: Option<String>,
    pub contact_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateListRequest {
    pub name: String,
    pub list_type: String,
    pub query_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListRequest {
    pub name: Option<String>,
    pub list_type: Option<String>,
    pub query_text: Option<String>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    use diesel::prelude::*;
    use crate::core::shared::schema::bots;

    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    
    let org_id = bots::table
        .filter(bots::id.eq(bot_id))
        .select(bots::org_id)
        .first::<Option<Uuid>>(&mut conn)
        .unwrap_or(None)
        .unwrap_or(Uuid::nil());

    (org_id, bot_id)
}

pub async fn list_lists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<MarketingList>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let lists: Vec<MarketingList> = marketing_lists::table
        .filter(marketing_lists::org_id.eq(org_id))
        .filter(marketing_lists::bot_id.eq(bot_id))
        .order(marketing_lists::created_at.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(lists))
}

pub async fn get_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketingList>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let list: MarketingList = marketing_lists::table
        .filter(marketing_lists::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "List not found".to_string()))?;

    Ok(Json(list))
}

pub async fn create_list(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateListRequest>,
) -> Result<Json<MarketingList>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let list = MarketingList {
        id,
        org_id,
        bot_id,
        name: req.name,
        list_type: req.list_type,
        query_text: req.query_text,
        contact_count: Some(0),
        created_at: now,
        updated_at: Some(now),
    };

    diesel::insert_into(marketing_lists::table)
        .values(&list)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(list))
}

pub async fn update_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateListRequest>,
) -> Result<Json<MarketingList>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    if let Some(name) = req.name {
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set(marketing_lists::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(list_type) = req.list_type {
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set(marketing_lists::list_type.eq(list_type))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(query_text) = req.query_text {
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set(marketing_lists::query_text.eq(Some(query_text)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
        .set(marketing_lists::updated_at.eq(Some(now)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_list(State(state), Path(id)).await
}

pub async fn delete_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(marketing_lists::table.filter(marketing_lists::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn refresh_marketing_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    use crate::core::shared::schema::crm_contacts;

    let list: MarketingList = marketing_lists::table
        .filter(marketing_lists::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "List not found".to_string()))?;

    let (org_id, bot_id) = get_bot_context(&state);

    let query_text = list.query_text.as_deref().unwrap_or("");
    let list_type = list.list_type.as_str();

    let contact_count: i64 = if list_type == "dynamic" && !query_text.is_empty() {
        let query_lower = query_text.to_lowercase();
        
        if query_lower.contains("status=") {
            let status = query_lower
                .split("status=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("active");
            
            crm_contacts::table
                .filter(crm_contacts::org_id.eq(org_id))
                .filter(crm_contacts::bot_id.eq(bot_id))
                .filter(crm_contacts::status.eq(status))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0)
        } else if query_lower.contains("company=") {
            let company = query_lower
                .split("company=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("");
            
            if !company.is_empty() {
                crm_contacts::table
                    .filter(crm_contacts::org_id.eq(org_id))
                    .filter(crm_contacts::bot_id.eq(bot_id))
                    .filter(crm_contacts::company.ilike(format!("%{company}%")))
                    .count()
                    .get_result(&mut conn)
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            let pattern = format!("%{query_text}%");
            crm_contacts::table
                .filter(crm_contacts::org_id.eq(org_id))
                .filter(crm_contacts::bot_id.eq(bot_id))
                .filter(
                    crm_contacts::first_name.ilike(pattern.clone())
                        .or(crm_contacts::last_name.ilike(pattern.clone()))
                        .or(crm_contacts::email.ilike(pattern.clone()))
                        .or(crm_contacts::company.ilike(pattern)),
                )
                .count()
                .get_result(&mut conn)
                .unwrap_or(0)
        }
    } else {
        crm_contacts::table
            .filter(crm_contacts::org_id.eq(org_id))
            .filter(crm_contacts::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    };

    diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
        .set((
            marketing_lists::contact_count.eq(Some(contact_count as i32)),
            marketing_lists::updated_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "refreshed",
        "list_id": id,
        "contact_count": contact_count
    })))
}
