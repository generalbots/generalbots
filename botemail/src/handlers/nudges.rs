use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AppState;
use crate::types::{NudgeCheckRequest, NudgesResponse};

pub async fn check_nudges(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<NudgeCheckRequest>,
) -> Result<Json<NudgesResponse>, StatusCode> {
    Ok(Json(NudgesResponse { nudges: vec![] }))
}

pub async fn dismiss_nudge(
    State(state): State<Arc<AppState>>,
    Json(email_id): Json<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::sql_query("UPDATE email_nudges SET dismissed = true WHERE email_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(email_id)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
