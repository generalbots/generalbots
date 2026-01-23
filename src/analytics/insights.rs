use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Date, Double, Uuid as DieselUuid};
use log::debug;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

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
pub struct InsightsService {
    pool: DbPool,
}

impl InsightsService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
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
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyInsights>, InsightsError> {
        let pool = self.pool.clone();

        let result = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| InsightsError::Database(e.to_string()))?;

            // Query daily insights from database
            let rows: Vec<DailyInsightsRow> = diesel::sql_query(
                "SELECT id, user_id, date, total_active_time, focus_time, meeting_time,
                        email_time, chat_time, document_time, collaboration_score,
                        wellbeing_score, productivity_score
                 FROM user_daily_insights
                 WHERE user_id = $1 AND date >= $2 AND date <= $3
                 ORDER BY date ASC"
            )
            .bind::<DieselUuid, _>(user_id)
            .bind::<Date, _>(start_date)
            .bind::<Date, _>(end_date)
            .load(&mut conn)
            .map_err(|e| InsightsError::Database(e.to_string()))?;

            if !rows.is_empty() {
                // Return real data from database
                return Ok(rows.into_iter().map(|r| DailyInsights {
                    id: r.id,
                    user_id: r.user_id,
                    date: r.date,
                    total_active_time: r.total_active_time,
                    focus_time: r.focus_time,
                    meeting_time: r.meeting_time,
                    email_time: r.email_time,
                    chat_time: r.chat_time,
                    document_time: r.document_time,
                    collaboration_score: r.collaboration_score as f32,
                    wellbeing_score: r.wellbeing_score as f32,
                    productivity_score: r.productivity_score as f32,
                }).collect());
            }

            // If no data exists, compute from activity logs
            let activity_rows: Vec<ActivityAggRow> = diesel::sql_query(
                "SELECT DATE(created_at) as activity_date,
                        SUM(CASE WHEN activity_type = 'focus' THEN duration_seconds ELSE 0 END) as focus_time,
                        SUM(CASE WHEN activity_type = 'meeting' THEN duration_seconds ELSE 0 END) as meeting_time,
                        SUM(CASE WHEN activity_type = 'email' THEN duration_seconds ELSE 0 END) as email_time,
                        SUM(CASE WHEN activity_type = 'chat' THEN duration_seconds ELSE 0 END) as chat_time,
                        SUM(CASE WHEN activity_type = 'document' THEN duration_seconds ELSE 0 END) as document_time,
                        SUM(duration_seconds) as total_time
                 FROM user_activity_logs
                 WHERE user_id = $1 AND DATE(created_at) >= $2 AND DATE(created_at) <= $3
                 GROUP BY DATE(created_at)
                 ORDER BY activity_date ASC"
            )
            .bind::<DieselUuid, _>(user_id)
            .bind::<Date, _>(start_date)
            .bind::<Date, _>(end_date)
            .load(&mut conn)
            .unwrap_or_default();

            let mut insights = Vec::new();

            if !activity_rows.is_empty() {
                for row in activity_rows {
                    let total = row.total_time.max(1);
                    let collab_score = ((row.meeting_time + row.chat_time) as f64 / total as f64 * 100.0).min(100.0);
                    let focus_score = (row.focus_time as f64 / total as f64 * 100.0).min(100.0);

                    insights.push(DailyInsights {
                        id: Uuid::new_v4(),
                        user_id,
                        date: row.activity_date,
                        total_active_time: row.total_time,
                        focus_time: row.focus_time,
                        meeting_time: row.meeting_time,
                        email_time: row.email_time,
                        chat_time: row.chat_time,
                        document_time: row.document_time,
                        collaboration_score: collab_score as f32,
                        wellbeing_score: 75.0, // Default baseline
                        productivity_score: focus_score as f32,
                    });
                }
            } else {
                // Generate minimal placeholder for date range when no activity data exists
                debug!("No activity data found for user {}, returning empty insights", user_id);
            }

            Ok(insights)
        })
        .await
        .map_err(|e| InsightsError::Database(e.to_string()))??;

        Ok(result)
    }

    async fn generate_recommendations(&self, user_id: Uuid) -> Vec<WellbeingRecommendation> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = match pool.get() {
                Ok(c) => c,
                Err(_) => return Vec::new(),
            };

            let mut recommendations = Vec::new();

            // Get user's recent activity patterns
            let stats: Result<ActivityStatsRow, _> = diesel::sql_query(
                "SELECT
                    AVG(focus_time) as avg_focus,
                    AVG(meeting_time) as avg_meeting,
                    AVG(total_active_time) as avg_active,
                    COUNT(*) as days_tracked
                 FROM user_daily_insights
                 WHERE user_id = $1 AND date >= CURRENT_DATE - INTERVAL '14 days'"
            )
            .bind::<DieselUuid, _>(user_id)
            .get_result(&mut conn);

            if let Ok(stats) = stats {
                let avg_focus_hours = stats.avg_focus.unwrap_or(0.0) / 3600.0;
                let avg_meeting_hours = stats.avg_meeting.unwrap_or(0.0) / 3600.0;
                let avg_active_hours = stats.avg_active.unwrap_or(0.0) / 3600.0;

                // Recommend more focus time if low
                if avg_focus_hours < 2.0 {
                    recommendations.push(WellbeingRecommendation {
                        id: Uuid::new_v4(),
                        category: RecommendationCategory::FocusTime,
                        title: "Increase focus time".to_string(),
                        description: format!(
                            "You're averaging {:.1} hours of focus time. Try blocking 2+ hours for deep work.",
                            avg_focus_hours
                        ),
                        priority: RecommendationPriority::High,
                        action_url: Some("/calendar/focus".to_string()),
                    });
                }

                // Warn about too many meetings
                if avg_meeting_hours > 5.0 {
                    recommendations.push(WellbeingRecommendation {
                        id: Uuid::new_v4(),
                        category: RecommendationCategory::MeetingLoad,
                        title: "Reduce meeting load".to_string(),
                        description: format!(
                            "You're averaging {:.1} hours in meetings. Consider declining some or making them shorter.",
                            avg_meeting_hours
                        ),
                        priority: RecommendationPriority::High,
                        action_url: Some("/calendar".to_string()),
                    });
                }

                // Recommend breaks if working long hours
                if avg_active_hours > 9.0 {
                    recommendations.push(WellbeingRecommendation {
                        id: Uuid::new_v4(),
                        category: RecommendationCategory::Breaks,
                        title: "Take more breaks".to_string(),
                        description: format!(
                            "You're averaging {:.1} active hours. Remember to take regular breaks.",
                            avg_active_hours
                        ),
                        priority: RecommendationPriority::Medium,
                        action_url: None,
                    });
                }
            }

            // Default recommendations if no data or few generated
            if recommendations.is_empty() {
                recommendations.push(WellbeingRecommendation {
                    id: Uuid::new_v4(),
                    category: RecommendationCategory::FocusTime,
                    title: "Schedule focus time".to_string(),
                    description: "Block 2 hours daily for deep work without interruptions".to_string(),
                    priority: RecommendationPriority::Medium,
                    action_url: Some("/calendar/focus".to_string()),
                });
            }

            recommendations
        })
        .await
        .unwrap_or_else(|_| vec![
            WellbeingRecommendation {
                id: Uuid::new_v4(),
                category: RecommendationCategory::FocusTime,
                title: "Schedule focus time".to_string(),
                description: "Block 2 hours daily for deep work without interruptions".to_string(),
                priority: RecommendationPriority::Medium,
                action_url: Some("/calendar/focus".to_string()),
            },
        ])
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

}

// QueryableByName structs for database queries
#[derive(diesel::QueryableByName)]
struct DailyInsightsRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    user_id: Uuid,
    #[diesel(sql_type = Date)]
    date: NaiveDate,
    #[diesel(sql_type = BigInt)]
    total_active_time: i64,
    #[diesel(sql_type = BigInt)]
    focus_time: i64,
    #[diesel(sql_type = BigInt)]
    meeting_time: i64,
    #[diesel(sql_type = BigInt)]
    email_time: i64,
    #[diesel(sql_type = BigInt)]
    chat_time: i64,
    #[diesel(sql_type = BigInt)]
    document_time: i64,
    #[diesel(sql_type = Double)]
    collaboration_score: f64,
    #[diesel(sql_type = Double)]
    wellbeing_score: f64,
    #[diesel(sql_type = Double)]
    productivity_score: f64,
}

#[derive(diesel::QueryableByName)]
struct ActivityAggRow {
    #[diesel(sql_type = Date)]
    activity_date: NaiveDate,
    #[diesel(sql_type = BigInt)]
    focus_time: i64,
    #[diesel(sql_type = BigInt)]
    meeting_time: i64,
    #[diesel(sql_type = BigInt)]
    email_time: i64,
    #[diesel(sql_type = BigInt)]
    chat_time: i64,
    #[diesel(sql_type = BigInt)]
    document_time: i64,
    #[diesel(sql_type = BigInt)]
    total_time: i64,
}

#[derive(diesel::QueryableByName)]
struct ActivityStatsRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<Double>)]
    avg_focus: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Double>)]
    avg_meeting: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Double>)]
    avg_active: Option<f64>,
    #[diesel(sql_type = BigInt)]
    _days_tracked: i64,
}

impl InsightsService {
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
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let usage = service.track_usage(user_id, req).await?;
    Ok(Json(usage))
}

pub async fn handle_get_daily(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<DailyInsights>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let insights = service.get_daily_insights(user_id, date).await?;
    Ok(Json(insights))
}

pub async fn handle_get_weekly(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<WeeklyInsights>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let insights = service.get_weekly_insights(user_id, date).await?;
    Ok(Json(insights))
}

pub async fn handle_get_trends(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<Vec<DailyInsights>>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let end_date = query.end_date.unwrap_or_else(|| Utc::now().date_naive());
    let start_date = query.start_date.unwrap_or_else(|| end_date - Duration::days(30));
    let trends = service.get_trends(user_id, start_date, end_date).await?;
    Ok(Json(trends))
}

pub async fn handle_get_recommendations(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<WellbeingRecommendation>>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let recommendations = service.generate_recommendations(user_id).await;
    Ok(Json(recommendations))
}

pub async fn handle_get_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<InsightsSettings>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let settings = service.get_settings(user_id).await?;
    Ok(Json(settings))
}

pub async fn handle_update_settings(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<InsightsSettings>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let settings = service.update_settings(user_id, req).await?;
    Ok(Json(settings))
}

pub async fn handle_update_focus_mode(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<UpdateFocusModeRequest>,
) -> Result<Json<FocusMode>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
    let user_id = Uuid::nil();
    let focus_mode = service.update_focus_mode(user_id, req).await?;
    Ok(Json(focus_mode))
}

pub async fn handle_get_app_breakdown(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<Vec<AppUsageSummary>>, InsightsError> {
    let service = InsightsService::new(_state.conn.clone());
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
        .route("/api/insights/settings", get(handle_get_settings).put(handle_update_settings))
        .route("/api/insights/focus-mode", put(handle_update_focus_mode))
        .route("/api/insights/apps", get(handle_get_app_breakdown))
}
