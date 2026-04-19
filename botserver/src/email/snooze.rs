use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Datelike, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SnoozeRequest {
    pub email_ids: Vec<Uuid>,
    pub preset: String,
}

#[derive(Debug, Serialize)]
pub struct SnoozeResponse {
    pub snoozed_count: usize,
    pub snooze_until: DateTime<Utc>,
}

/// Snooze emails until a specific time
pub async fn snooze_emails(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SnoozeRequest>,
) -> Result<Json<SnoozeResponse>, StatusCode> {
    use crate::core::shared::schema::email_snooze;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let snooze_until = calculate_snooze_time(&req.preset);
    let snooze_until_naive = snooze_until.naive_utc();

    let mut snoozed_count = 0;
    for email_id in &req.email_ids {
        diesel::insert_into(email_snooze::table)
            .values((
                email_snooze::email_id.eq(email_id),
                email_snooze::snooze_until.eq(snooze_until_naive),
            ))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        snoozed_count += 1;
    }

    Ok(Json(SnoozeResponse {
        snoozed_count,
        snooze_until,
    }))
}

/// Get snoozed emails that are ready to be shown
pub async fn get_snoozed_emails(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Uuid>>, StatusCode> {
    use crate::core::shared::schema::email_snooze;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let now = Utc::now().naive_utc();
    
    let email_ids: Vec<Uuid> = email_snooze::table
        .filter(email_snooze::snooze_until.le(now))
        .select(email_snooze::email_id)
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete processed snoozes
    diesel::delete(email_snooze::table.filter(email_snooze::snooze_until.le(now)))
        .execute(&mut conn)
        .ok();

    Ok(Json(email_ids))
}

fn calculate_snooze_time(preset: &str) -> DateTime<Utc> {
    let now = Utc::now();
    
    match preset {
        "later-today" => {
            // 6 PM today
            let today = now.date_naive();
            today
                .and_hms_opt(18, 0, 0)
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or(now + Duration::hours(6))
        }
        "tomorrow" => {
            // 8 AM tomorrow
            let tomorrow = (now + Duration::days(1)).date_naive();
            tomorrow
                .and_hms_opt(8, 0, 0)
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or(now + Duration::days(1))
        }
        "this-weekend" => {
            // Saturday 9 AM
            let days_until_saturday = (6 - now.weekday().num_days_from_monday()) % 7;
            let saturday = (now + Duration::days(days_until_saturday as i64)).date_naive();
            saturday
                .and_hms_opt(9, 0, 0)
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or(now + Duration::days(days_until_saturday as i64))
        }
        "next-week" => {
            // Monday 8 AM next week
            let days_until_next_monday = (7 - now.weekday().num_days_from_monday() + 1) % 7 + 7;
            let next_monday = (now + Duration::days(days_until_next_monday as i64)).date_naive();
            next_monday
                .and_hms_opt(8, 0, 0)
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or(now + Duration::days(days_until_next_monday as i64))
        }
        _ => now + Duration::hours(1), // Default: 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_snooze_time() {
        let later_today = calculate_snooze_time("later-today");
        assert!(later_today > Utc::now());

        let tomorrow = calculate_snooze_time("tomorrow");
        assert!(tomorrow > Utc::now());
        
        let weekend = calculate_snooze_time("this-weekend");
        assert!(weekend > Utc::now());
    }
}
