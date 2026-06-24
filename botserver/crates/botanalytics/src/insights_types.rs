use axum::response::IntoResponse;
use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_name: String,
    pub session_start: DateTime<chrono::Utc>,
    pub session_end: Option<DateTime<chrono::Utc>>,
    pub duration_seconds: i64,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyInsights {
    pub id: Uuid,
    pub user_id: Uuid,
    pub date: NaiveDate,
    pub total_active_time: i64,
    pub focus_time: i64,
    pub meeting_time: i64,
    pub email_time: i64,
    pub chat_time: i64,
    pub document_time: i64,
    pub collaboration_score: f32,
    pub wellbeing_score: f32,
    pub productivity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyInsights {
    pub user_id: Uuid,
    pub week_start: NaiveDate,
    pub week_end: NaiveDate,
    pub daily_breakdown: Vec<DailyInsights>,
    pub total_hours: f32,
    pub avg_daily_hours: f32,
    pub focus_hours: f32,
    pub meeting_hours: f32,
    pub top_apps: Vec<AppUsageSummary>,
    pub trends: InsightsTrends,
    pub recommendations: Vec<WellbeingRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSummary {
    pub app_name: String,
    pub total_seconds: i64,
    pub percentage: f32,
    pub sessions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsTrends {
    pub focus_time_trend: TrendDirection,
    pub meeting_time_trend: TrendDirection,
    pub collaboration_trend: TrendDirection,
    pub wellbeing_trend: TrendDirection,
    pub focus_time_change_pct: f32,
    pub meeting_time_change_pct: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellbeingRecommendation {
    pub id: Uuid,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub action_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    FocusTime,
    MeetingLoad,
    WorkLifeBalance,
    Collaboration,
    Breaks,
    AfterHours,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusMode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub enabled: bool,
    pub schedule: Option<FocusSchedule>,
    pub block_notifications: bool,
    pub auto_decline_meetings: bool,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSchedule {
    pub days: Vec<Weekday>,
    pub start_time: String,
    pub end_time: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub id: Uuid,
    pub user_id: Uuid,
    pub enabled: bool,
    pub start_time: String,
    pub end_time: String,
    pub days: Vec<Weekday>,
    pub allow_urgent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsSettings {
    pub user_id: Uuid,
    pub tracking_enabled: bool,
    pub daily_digest: bool,
    pub weekly_digest: bool,
    pub digest_time: String,
    pub focus_mode: Option<FocusMode>,
    pub quiet_hours: Option<QuietHours>,
    pub goals: Option<InsightsGoals>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsGoals {
    pub daily_focus_hours: f32,
    pub max_meeting_hours: f32,
    pub min_break_minutes: i32,
    pub max_after_hours_minutes: i32,
}

#[derive(Debug, Deserialize)]
pub struct TrackUsageRequest {
    pub app_name: String,
    pub event_type: UsageEventType,
    pub timestamp: Option<DateTime<chrono::Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageEventType {
    SessionStart,
    SessionEnd,
    ActivePing,
}

#[derive(Debug, Deserialize)]
pub struct InsightsQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub period: Option<InsightsPeriod>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InsightsPeriod {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub tracking_enabled: Option<bool>,
    pub daily_digest: Option<bool>,
    pub weekly_digest: Option<bool>,
    pub digest_time: Option<String>,
    pub goals: Option<InsightsGoals>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFocusModeRequest {
    pub enabled: bool,
    pub schedule: Option<FocusSchedule>,
    pub block_notifications: Option<bool>,
    pub auto_decline_meetings: Option<bool>,
    pub status_message: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InsightsError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl IntoResponse for InsightsError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}
