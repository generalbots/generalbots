use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_name: String,
    pub session_start: DateTime<Utc>,
    pub session_end: Option<DateTime<Utc>>,
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
    pub timestamp: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone)]
pub struct InsightsService {}

impl InsightsService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn track_usage(
        &self,
        user_id: Uuid,
        req: TrackUsageRequest,
    ) -> Result<AppUsage, InsightsError> {
        let now = req.timestamp.unwrap_or_else(Utc::now);
        let today = now.date_naive();

        Ok(AppUsage {
            id: Uuid::new_v4(),
            user_id,
            app_name: req.app_name,
            session_start: now,
            session_end: if req.event_type == UsageEventType::SessionEnd {
                Some(now)
            } else {
                None
            },
            duration_seconds: 0,
            date: today,
        })
    }

    pub async fn get_daily_insights(
        &self,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyInsights, InsightsError> {
        Ok(DailyInsights {
            id: Uuid::new_v4(),
            user_id,
            date,
            total_active_time: 0,
            focus_time: 0,
            meeting_time: 0,
            email_time: 0,
            chat_time: 0,
            document_time: 0,
            collaboration_score: 0.0,
            wellbeing_score: 0.0,
            productivity_score: 0.0,
        })
    }

    pub async fn get_weekly_insights(
        &self,
        user_id: Uuid,
        _week_start: NaiveDate,
    ) -> Result<WeeklyInsights, InsightsError> {
        let today = Utc::now().date_naive();
        let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
        let week_end = week_start + Duration::days(6);

        Ok(WeeklyInsights {
            user_id,
            week_start,
            week_end,
            daily_breakdown: vec![],
            total_hours: 0.0,
            avg_daily_hours: 0.0,
            focus_hours: 0.0,
            meeting_hours: 0.0,
            top_apps: vec![],
            trends: InsightsTrends {
                focus_time_trend: TrendDirection::Stable,
                meeting_time_trend: TrendDirection::Stable,
                collaboration_trend: TrendDirection::Stable,
                wellbeing_trend: TrendDirection::Stable,
                focus_time_change_pct: 0.0,
                meeting_time_change_pct: 0.0,
            },
            recommendations: self.generate_recommendations(user_id).await,
        })
    }

    pub async fn get_trends(
        &self,
        _user_id: Uuid,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<Vec<DailyInsights>, InsightsError> {
        Ok(vec![])
    }

    async fn generate_recommendations(&self, _user_id: Uuid) -> Vec<WellbeingRecommendation> {
        vec![
            WellbeingRecommendation {
                id: Uuid::new_v4(),
                category: RecommendationCategory::FocusTime,
                title: "Schedule focus time".to_string(),
                description: "Block 2 hours daily for deep work without interruptions".to_string(),
                priority: RecommendationPriority::Medium,
                action_url: Some("/calendar/focus".to_string()),
            },
            WellbeingRecommendation {
                id: Uuid::new_v4(),
                category: RecommendationCategory::Breaks,
                title: "Take regular breaks".to_string(),
                description: "Consider a 5-minute break every hour".to_string(),
                priority: RecommendationPriority::Low,
                action_url: None,
            },
        ]
    }

    pub async fn get_settings(
        &self,
        user_id: Uuid,
    ) -> Result<InsightsSettings, InsightsError> {
        Ok(InsightsSettings {
            user_id,
            tracking_enabled: true,
            daily_digest: false,
            weekly_digest: true,
            digest_time: "09:00".to_string(),
            focus_mode: None,
            quiet_hours: None,
            goals: Some(InsightsGoals {
                daily_focus_hours: 4.0,
                max_meeting_hours: 6.0,
                min_break_minutes: 30,
                max_after_hours_minutes: 60,
            }),
        })
    }

    pub async fn update_settings(
        &self,
        user_id: Uuid,
        req: UpdateSettingsRequest,
    ) -> Result<InsightsSettings, InsightsError> {
        let mut settings = self.get_settings(user_id).await?;

        if let Some(enabled) = req.tracking_enabled {
            settings.tracking_enabled = enabled;
        }
        if let Some(daily) = req.daily_digest {
            settings.daily_digest = daily;
        }
        if let Some(weekly) = req.weekly_digest {
            settings.weekly_digest = weekly;
        }
        if let Some(time) = req.digest_time {
            settings.digest_time = time;
        }
        if let Some(goals) = req.goals {
            settings.goals = Some(goals);
        }

        Ok(settings)
    }

    pub async fn update_focus_mode(
        &self,
        user_id: Uuid,
        req: UpdateFocusModeRequest,
    ) -> Result<FocusMode, InsightsError> {
        Ok(FocusMode {
            id: Uuid::new_v4(),
            user_id,
            enabled: req.enabled,
            schedule: req.schedule,
            block_notifications: req.block_notifications.unwrap_or(true),
            auto_decline_meetings: req.auto_decline_meetings.unwrap_or(false),
            status_message: req.status_message,
        })
    }

    pub async fn get_app_breakdown(
        &self,
        _user_id: Uuid,
        _date: NaiveDate,
    ) -> Result<Vec<AppUsageSummary>, InsightsError> {
        Ok(vec![
            AppUsageSummary {
                app_name: "Chat".to_string(),
                total_seconds: 3600,
                percentage: 25.0,
                sessions: 15,
            },
            AppUsageSummary {
                app_name: "Mail".to_string(),
                total_seconds: 2700,
                percentage: 18.75,
                sessions: 8,
            },
            AppUsageSummary {
                app_name: "Documents".to_string(),
                total_seconds: 5400,
                percentage: 37.5,
                sessions: 5,
            },
            AppUsageSummary {
                app_name: "Calendar".to_string(),
                total_seconds: 1800,
                percentage: 12.5,
                sessions: 10,
            },
            AppUsageSummary {
                app_name: "Tasks".to_string(),
                total_seconds: 900,
                percentage: 6.25,
                sessions: 12,
            },
        ])
    }
}

impl Default for InsightsService {
    fn default() -> Self {
        Self::new()
    }
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
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn handle_track_usage(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TrackUsageRequest>,
) -> Result<Json<AppUsage>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let usage = service.track_usage(user_id, req).await?;
    Ok(Json(usage))
}

pub async fn handle_get_daily(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<DailyInsights>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let insights = service.get_daily_insights(user_id, date).await?;
    Ok(Json(insights))
}

pub async fn handle_get_weekly(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<WeeklyInsights>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let insights = service.get_weekly_insights(user_id, date).await?;
    Ok(Json(insights))
}

pub async fn handle_get_trends(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<Vec<DailyInsights>>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let end_date = query.end_date.unwrap_or_else(|| Utc::now().date_naive());
    let start_date = query.start_date.unwrap_or_else(|| end_date - Duration::days(30));
    let trends = service.get_trends(user_id, start_date, end_date).await?;
    Ok(Json(trends))
}

pub async fn handle_get_recommendations(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<WellbeingRecommendation>>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let recommendations = service.generate_recommendations(user_id).await;
    Ok(Json(recommendations))
}

pub async fn handle_get_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<InsightsSettings>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let settings = service.get_settings(user_id).await?;
    Ok(Json(settings))
}

pub async fn handle_update_settings(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<InsightsSettings>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let settings = service.update_settings(user_id, req).await?;
    Ok(Json(settings))
}

pub async fn handle_update_focus_mode(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<UpdateFocusModeRequest>,
) -> Result<Json<FocusMode>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let focus_mode = service.update_focus_mode(user_id, req).await?;
    Ok(Json(focus_mode))
}

pub async fn handle_get_app_breakdown(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<Vec<AppUsageSummary>>, InsightsError> {
    let service = InsightsService::new();
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let breakdown = service.get_app_breakdown(user_id, date).await?;
    Ok(Json(breakdown))
}

pub fn configure_insights_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/insights/track", post(handle_track_usage))
        .route("/api/insights/daily", get(handle_get_daily))
        .route("/api/insights/weekly", get(handle_get_weekly))
        .route("/api/insights/trends", get(handle_get_trends))
        .route("/api/insights/recommendations", get(handle_get_recommendations))
        .route("/api/insights/settings", get(handle_get_settings))
        .route("/api/insights/settings", put(handle_update_settings))
        .route("/api/insights/focus-mode", put(handle_update_focus_mode))
        .route("/api/insights/apps", get(handle_get_app_breakdown))
}
