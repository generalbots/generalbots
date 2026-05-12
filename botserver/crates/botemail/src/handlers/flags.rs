use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AppState;
use crate::types::{FlagRequest, FlagResponse};

pub async fn flag_for_followup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FlagRequest>,
) -> Result<Json<FlagResponse>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let follow_up_date = calculate_followup_date(&req.follow_up);

    let mut flagged_count = 0;
    for email_id in &req.email_ids {
        diesel::sql_query(
            "INSERT INTO email_flags (email_id, follow_up_date, flag_type) VALUES ($1, $2, $3)"
        )
        .bind::<diesel::sql_types::Uuid, _>(email_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(follow_up_date)
        .bind::<diesel::sql_types::Text, _>(&req.follow_up)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        flagged_count += 1;
    }

    Ok(Json(FlagResponse { flagged_count }))
}

pub async fn clear_flag(
    State(state): State<Arc<AppState>>,
    Json(email_id): Json<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::sql_query("DELETE FROM email_flags WHERE email_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(email_id)
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
