use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use crate::core::shared::schema::warmup_schedules;
use crate::core::shared::state::AppState;
use crate::marketing::lists::MarketingList;

/// Standard warmup schedule from industry best practices
/// Day ranges and max emails per day
pub const WARMUP_SCHEDULE: [(u32, u32, u32); 8] = [
    (1, 2, 50),      // Days 1-2: 50 emails/day
    (3, 4, 100),     // Days 3-4: 100 emails/day
    (5, 7, 500),     // Days 5-7: 500 emails/day
    (8, 10, 1000),   // Days 8-10: 1,000 emails/day
    (11, 14, 5000),  // Days 11-14: 5,000 emails/day
    (15, 21, 10000), // Days 15-21: 10,000 emails/day
    (22, 28, 50000), // Days 22-28: 50,000 emails/day
    (29, u32::MAX, u32::MAX), // Day 29+: unlimited
];

/// Warmup schedule status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarmupStatus {
    Active,
    Paused,
    Completed,
}

impl WarmupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
        }
    }
}

impl From<&str> for WarmupStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            _ => Self::Active,
        }
    }
}

/// Reason for pausing warmup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PausedReason {
    HighBounceRate { rate: f64 },
    HighComplaintRate { rate: f64 },
    ManualPause,
    Other(String),
}

impl PausedReason {
    pub fn as_str(&self) -> String {
        match self {
            Self::HighBounceRate { rate } => format!("High bounce rate: {:.1}%", rate * 100.0),
            Self::HighComplaintRate { rate } => format!("High complaint rate: {:.1}%", rate * 100.0),
            Self::ManualPause => "Manually paused".to_string(),
            Self::Other(s) => s.clone(),
        }
    }
}

/// Warmup schedule for an IP address
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = warmup_schedules)]
pub struct WarmupSchedule {
    pub id: Uuid,
    pub org_id: Uuid,
    pub ip: String,
    pub started_at: DateTime<Utc>,
    pub current_day: u32,
    pub daily_limit: u32,
    pub status: String,
    pub paused_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Engine for managing warmup schedules
pub struct WarmupEngine;

impl WarmupEngine {
    /// Get the daily email limit for an IP on a given day of warmup
    pub fn get_daily_limit(day: u32) -> u32 {
        for (start_day, end_day, limit) in WARMUP_SCHEDULE.iter() {
            if day >= *start_day && day <= *end_day {
                return *limit;
            }
        }
        u32::MAX // Unlimited after day 28
    }

    /// Calculate current day in warmup schedule
    pub fn calculate_current_day(started_at: DateTime<Utc>) -> u32 {
        let now = Utc::now();
        let duration = now.signed_duration_since(started_at);
        let days = duration.num_days();
        if days < 0 {
            return 1;
        }
        (days as u32) + 1
    }

    /// Get the daily limit for an IP based on its warmup schedule
    pub async fn get_ip_daily_limit(
        state: &AppState,
        ip: &IpAddr,
    ) -> Result<u32, diesel::result::Error> {
        use crate::core::shared::schema::warmup_schedules::dsl::*;

        let mut conn = state.conn.get()?;
        let ip_str = ip.to_string();

        let schedule: Option<WarmupSchedule> = warmup_schedules
            .filter(ip.eq(&ip_str))
            .filter(status.eq("active"))
            .first(&mut conn)
            .optional()?;

        match schedule {
            Some(sched) => {
                let current_day = Self::calculate_current_day(sched.started_at);
                Ok(Self::get_daily_limit(current_day))
            }
            None => Ok(u32::MAX), // No warmup schedule = unlimited
        }
    }

    /// Start a new warmup schedule for an IP
    pub async fn start_warmup(
        state: &AppState,
        org_id_val: Uuid,
        ip_val: &IpAddr,
    ) -> Result<WarmupSchedule, diesel::result::Error> {
        use crate::core::shared::schema::warmup_schedules::dsl::*;

        let mut conn = state.conn.get()?;
        let ip_str = ip_val.to_string();
        let now = Utc::now();

        // Check if already exists
        let existing: Option<WarmupSchedule> = warmup_schedules
            .filter(ip.eq(&ip_str))
            .first(&mut conn)
            .optional()?;

        if let Some(mut sched) = existing {
            // Reset the schedule
            sched.started_at = now;
            sched.current_day = 1;
            sched.daily_limit = Self::get_daily_limit(1);
            sched.status = WarmupStatus::Active.as_str().to_string();
            sched.paused_reason = None;
            sched.updated_at = Some(now);

            diesel::update(warmup_schedules.filter(id.eq(sched.id)))
                .set(&sched)
                .execute(&mut conn)?;

            return Ok(sched);
        }

        // Create new schedule
        let new_schedule = WarmupSchedule {
            id: Uuid::new_v4(),
            org_id: org_id_val,
            ip: ip_str,
            started_at: now,
            current_day: 1,
            daily_limit: Self::get_daily_limit(1),
            status: WarmupStatus::Active.as_str().to_string(),
            paused_reason: None,
            created_at: now,
            updated_at: None,
        };

        diesel::insert_into(warmup_schedules)
            .values(&new_schedule)
            .execute(&mut conn)?;

        Ok(new_schedule)
    }

    /// Pause warmup due to issues
    pub async fn pause_warmup(
        state: &AppState,
        ip_val: &IpAddr,
        reason: PausedReason,
    ) -> Result<(), diesel::result::Error> {
        use crate::core::shared::schema::warmup_schedules::dsl::*;

        let mut conn = state.conn.get()?;
        let ip_str = ip_val.to_string();
        let now = Utc::now();

        diesel::update(warmup_schedules.filter(ip.eq(&ip_str)))
            .set((
                status.eq(WarmupStatus::Paused.as_str()),
                paused_reason.eq(Some(reason.as_str())),
                updated_at.eq(Some(now)),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Resume warmup at same volume
    pub async fn resume_warmup(
        state: &AppState,
        ip_val: &IpAddr,
    ) -> Result<(), diesel::result::Error> {
        use crate::core::shared::schema::warmup_schedules::dsl::*;

        let mut conn = state.conn.get()?;
        let ip_str = ip_val.to_string();
        let now = Utc::now();

        diesel::update(warmup_schedules.filter(ip.eq(&ip_str)))
            .set((
                status.eq(WarmupStatus::Active.as_str()),
                paused_reason.eq(None::<String>),
                updated_at.eq(Some(now)),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Get engaged subscribers for warmup sends (opened in last 90 days)
    pub async fn get_engaged_subscribers(
        state: &AppState,
        list_id: Uuid,
        limit: usize,
    ) -> Result<Vec<String>, diesel::result::Error> {
        use crate::core::shared::schema::marketing_contacts;
        use crate::core::shared::schema::marketing_email_opens;
        use diesel::dsl::sql;

        let mut conn = state.conn.get()?;
        let ninety_days_ago = Utc::now() - chrono::Duration::days(90);

        // Get contacts who opened emails in the last 90 days
        let engaged: Vec<String> = marketing_contacts::table
            .inner_join(
                marketing_email_opens::table.on(
                    marketing_contacts::email.eq(marketing_email_opens::email)
                )
            )
            .filter(marketing_contacts::list_id.eq(list_id))
            .filter(marketing_email_opens::opened_at.gt(ninety_days_ago))
            .select(marketing_contacts::email)
            .distinct()
            .limit(limit as i64)
            .load(&mut conn)?;

        Ok(engaged)
    }

    /// Check if bounce rate exceeds threshold (3%)
    pub fn should_pause_for_bounces(sent: u32, bounces: u32) -> bool {
        if sent == 0 {
            return false;
        }
        let bounce_rate = bounces as f64 / sent as f64;
        bounce_rate > 0.03 // 3% threshold
    }

    /// Check if complaint rate exceeds threshold (0.1%)
    pub fn should_pause_for_complaints(sent: u32, complaints: u32) -> bool {
        if sent == 0 {
            return false;
        }
        let complaint_rate = complaints as f64 / sent as f64;
        complaint_rate > 0.001 // 0.1% threshold
    }
}

/// API response for warmup status
#[derive(Debug, Serialize)]
pub struct WarmupStatusResponse {
    pub ip: String,
    pub day: u32,
    pub daily_limit: u32,
    pub status: String,
    pub paused_reason: Option<String>,
    pub started_at: DateTime<Utc>,
}

impl From<WarmupSchedule> for WarmupStatusResponse {
    fn from(schedule: WarmupSchedule) -> Self {
        let current_day = WarmupEngine::calculate_current_day(schedule.started_at);
        Self {
            ip: schedule.ip,
            day: current_day,
            daily_limit: schedule.daily_limit,
            status: schedule.status,
            paused_reason: schedule.paused_reason,
            started_at: schedule.started_at,
        }
    }
}

/// Request to start warmup
#[derive(Debug, Deserialize)]
pub struct StartWarmupRequest {
    pub ip: String,
}

/// Request to pause/resume warmup
#[derive(Debug, Deserialize)]
pub struct PauseWarmupRequest {
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_daily_limit() {
        assert_eq!(WarmupEngine::get_daily_limit(1), 50);
        assert_eq!(WarmupEngine::get_daily_limit(2), 50);
        assert_eq!(WarmupEngine::get_daily_limit(5), 500);
        assert_eq!(WarmupEngine::get_daily_limit(15), 10000);
        assert_eq!(WarmupEngine::get_daily_limit(29), u32::MAX);
        assert_eq!(WarmupEngine::get_daily_limit(100), u32::MAX);
    }

    #[test]
    fn test_should_pause_for_bounces() {
        assert!(!WarmupEngine::should_pause_for_bounces(100, 2)); // 2% bounce rate
        assert!(WarmupEngine::should_pause_for_bounces(100, 4)); // 4% bounce rate
        assert!(!WarmupEngine::should_pause_for_bounces(0, 0));
    }

    #[test]
    fn test_should_pause_for_complaints() {
        assert!(!WarmupEngine::should_pause_for_complaints(1000, 0)); // 0% complaint rate
        assert!(WarmupEngine::should_pause_for_complaints(1000, 2)); // 0.2% complaint rate
    }
}
