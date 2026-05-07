use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::CrmActivity;
use crate::requests::ListQuery;
use crate::schema::crm_activities;
use crate::CrateState;

fn get_bot_context(state: &CrateState) -> (Uuid, Uuid) {
    state.get_bot_context()
}

pub async fn list_activities(
    State(state): State<Arc<CrateState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<CrmActivity>>, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
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
