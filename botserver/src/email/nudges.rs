use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct NudgeCheckRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Nudge {
    pub email_id: Uuid,
    pub from: String,
    pub subject: String,
    pub days_ago: i64,
}

#[derive(Debug, Serialize)]
pub struct NudgesResponse {
    pub nudges: Vec<Nudge>,
}

/// Check for emails that need follow-up nudges
pub async fn check_nudges(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<NudgeCheckRequest>,
) -> Result<Json<NudgesResponse>, StatusCode> {
    // Simple implementation - can be enhanced with actual email tracking
    let nudges = vec![];
    
    Ok(Json(NudgesResponse { nudges }))
}

/// Dismiss a nudge
pub async fn dismiss_nudge(
    State(state): State<Arc<AppState>>,
    Json(email_id): Json<Uuid>,
) -> Result<StatusCode, StatusCode> {
    use crate::core::shared::schema::email_nudges;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::update(email_nudges::table.filter(email_nudges::email_id.eq(email_id)))
        .set(email_nudges::dismissed.eq(true))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
