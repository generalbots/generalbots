use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use crate::schema::{marketing_contacts, marketing_email_opens, warmup_schedules};
use crate::state::AppState;

pub const WARMUP_SCHEDULE: [(u32, u32, u32); 8] = [
    (1, 2, 50),
    (3, 4, 100),
    (5, 7, 500),
    (8, 10, 1000),
    (11, 14, 5000),
    (15, 21, 10000),
    (22, 28, 50000),
    (29, u32::MAX, u32::MAX),
];

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
            Self::HighComplaintRate { rate } => {
                format!("High complaint rate: {:.1}%", rate * 100.0)
            }
            Self::ManualPause => "Manually paused".to_string(),
            Self::Other(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = warmup_schedules)]
pub struct WarmupSchedule {
    pub id: Uuid,
    pub org_id: Uuid,
    pub ip: String,
    pub started_at: DateTime<Utc>,
    pub current_day: i32,
    pub daily_limit: i32,
    pub status: String,
    pub paused_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct WarmupEngine;

impl WarmupEngine {
    pub fn get_daily_limit(day: u32) -> u32 {
        for (start_day, end_day, limit) in WARMUP_SCHEDULE.iter() {
            if day >= *start_day && day <= *end_day {
                return *limit;
            }
        }
        u32::MAX
    }

    pub fn calculate_current_day(started_at: DateTime<Utc>) -> u32 {
        let now = Utc::now();
        let duration = now.signed_duration_since(started_at);
        let days = duration.num_days();
        if days < 0 {
            return 1;
        }
        (days as u32) + 1
    }

    pub async fn get_ip_daily_limit(
        state: &AppState,
        ip_addr: &IpAddr,
    ) -> Result<u32, String> {
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        let ip_str = ip_addr.to_string();

        let schedule: Option<WarmupSchedule> = warmup_schedules::table
            .filter(warmup_schedules::ip.eq(&ip_str))
            .filter(warmup_schedules::status.eq("active"))
            .first(&mut conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;

        match schedule {
            Some(sched) => {
                let current_day = Self::calculate_current_day(sched.started_at);
                Ok(Self::get_daily_limit(current_day))
            }
            None => Ok(u32::MAX),
        }
    }

    pub async fn start_warmup(
        state: &AppState,
        org_id_val: Uuid,
        ip_addr: &IpAddr,
    ) -> Result<WarmupSchedule, String> {
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        let ip_str = ip_addr.to_string();
        let now = Utc::now();

        let existing: Option<WarmupSchedule> = warmup_schedules::table
            .filter(warmup_schedules::ip.eq(&ip_str))
            .first(&mut conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;

        if let Some(mut sched) = existing {
            sched.started_at = now;
            sched.current_day = 1;
            sched.daily_limit = Self::get_daily_limit(1) as i32;
            sched.status = WarmupStatus::Active.as_str().to_string();
            sched.paused_reason = None;
            sched.updated_at = Some(now);

            diesel::update(warmup_schedules::table.filter(warmup_schedules::id.eq(sched.id)))
                .set(&sched)
                .execute(&mut conn)
                .map_err(|e| format!("Update error: {e}"))?;

            return Ok(sched);
        }

        let new_schedule = WarmupSchedule {
            id: Uuid::new_v4(),
            org_id: org_id_val,
            ip: ip_str,
            started_at: now,
            current_day: 1,
            daily_limit: Self::get_daily_limit(1) as i32,
            status: WarmupStatus::Active.as_str().to_string(),
            paused_reason: None,
            created_at: now,
            updated_at: None,
        };

        diesel::insert_into(warmup_schedules::table)
            .values(&new_schedule)
            .execute(&mut conn)
            .map_err(|e| format!("Insert error: {e}"))?;

        Ok(new_schedule)
    }

    pub async fn pause_warmup(
        state: &AppState,
        ip_addr: &IpAddr,
        reason: PausedReason,
    ) -> Result<(), String> {
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        let ip_str = ip_addr.to_string();
        let now = Utc::now();

        diesel::update(warmup_schedules::table.filter(warmup_schedules::ip.eq(&ip_str)))
            .set((
                warmup_schedules::status.eq(WarmupStatus::Paused.as_str()),
                warmup_schedules::paused_reason.eq(Some(reason.as_str())),
                warmup_schedules::updated_at.eq(Some(now)),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;

        Ok(())
    }

    pub async fn resume_warmup(
        state: &AppState,
        ip_addr: &IpAddr,
    ) -> Result<(), String> {
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        let ip_str = ip_addr.to_string();
        let now = Utc::now();

        diesel::update(warmup_schedules::table.filter(warmup_schedules::ip.eq(&ip_str)))
            .set((
                warmup_schedules::status.eq(WarmupStatus::Active.as_str()),
                warmup_schedules::paused_reason.eq(None::<String>),
                warmup_schedules::updated_at.eq(Some(now)),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;

        Ok(())
    }

    pub async fn get_engaged_subscribers(
        state: &AppState,
        list_id: Uuid,
        limit: usize,
    ) -> Result<Vec<String>, String> {
        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        let ninety_days_ago = Utc::now() - chrono::Duration::days(90);

        let engaged: Vec<String> = marketing_contacts::table
            .inner_join(
                marketing_email_opens::table
                    .on(marketing_contacts::email.eq(marketing_email_opens::email)),
            )
            .filter(marketing_contacts::list_id.eq(list_id))
            .filter(marketing_email_opens::opened_at.gt(ninety_days_ago))
            .select(marketing_contacts::email)
            .distinct()
            .limit(limit as i64)
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?;

        Ok(engaged)
    }

    pub fn should_pause_for_bounces(sent: u32, bounces: u32) -> bool {
        if sent == 0 {
            return false;
        }
        let bounce_rate = bounces as f64 / sent as f64;
        bounce_rate > 0.03
    }

    pub fn should_pause_for_complaints(sent: u32, complaints: u32) -> bool {
        if sent == 0 {
            return false;
        }
        let complaint_rate = complaints as f64 / sent as f64;
        complaint_rate > 0.001
    }
}

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
            daily_limit: schedule.daily_limit as u32,
            status: schedule.status,
            paused_reason: schedule.paused_reason,
            started_at: schedule.started_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StartWarmupRequest {
    pub ip: String,
}

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
        assert!(!WarmupEngine::should_pause_for_bounces(100, 2));
        assert!(WarmupEngine::should_pause_for_bounces(100, 4));
        assert!(!WarmupEngine::should_pause_for_bounces(0, 0));
    }

    #[test]
    fn test_should_pause_for_complaints() {
        assert!(!WarmupEngine::should_pause_for_complaints(1000, 0));
        assert!(WarmupEngine::should_pause_for_complaints(1000, 2));
    }
}
