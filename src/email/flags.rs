use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct FlagRequest {
    pub email_ids: Vec<Uuid>,
    pub follow_up: String,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub flagged_count: usize,
}

/// Flag emails for follow-up
pub async fn flag_for_followup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FlagRequest>,
) -> Result<Json<FlagResponse>, StatusCode> {
    use crate::core::shared::schema::email_flags;
    

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let follow_up_date = calculate_followup_date(&req.follow_up);

    let mut flagged_count = 0;
    for email_id in &req.email_ids {
        diesel::insert_into(email_flags::table)
            .values((
                email_flags::email_id.eq(email_id),
                email_flags::follow_up_date.eq(follow_up_date),
                email_flags::flag_type.eq(&req.follow_up),
            ))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        flagged_count += 1;
    }

    Ok(Json(FlagResponse { flagged_count }))
}

/// Clear flag from email
pub async fn clear_flag(
    State(state): State<Arc<AppState>>,
    Json(email_id): Json<Uuid>,
) -> Result<StatusCode, StatusCode> {
    use crate::core::shared::schema::email_flags;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(email_flags::table.filter(email_flags::email_id.eq(email_id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

fn calculate_followup_date(preset: &str) -> Option<NaiveDate> {
    let now = Utc::now().date_naive();
    
    match preset {
        "today" => Some(now),
        "tomorrow" => Some(now + Duration::days(1)),
        "this-week" => Some(now + Duration::days(7 - now.weekday().num_days_from_monday() as i64)),
        "next-week" => Some(now + Duration::days(7)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_followup_date() {
        let today = calculate_followup_date("today");
        assert!(today.is_some());
        
        let tomorrow = calculate_followup_date("tomorrow");
        assert!(tomorrow.is_some());
    }
}
