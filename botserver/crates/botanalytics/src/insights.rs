use axum::{
    extract::{Query, State},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Date, Double, Uuid as DieselUuid};
use log::debug;
use std::sync::Arc;
use uuid::Uuid;

use crate::insights_types::*;
use crate::DbPool;

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
                        wellbeing_score: 75.0,
                        productivity_score: focus_score as f32,
                    });
                }
            } else {
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

pub async fn handle_track_usage(
    State(pool): State<Arc<DbPool>>,
    Json(req): Json<TrackUsageRequest>,
) -> Result<Json<AppUsage>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let usage = service.track_usage(user_id, req).await?;
    Ok(Json(usage))
}

pub async fn handle_get_daily(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<DailyInsights>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let insights = service.get_daily_insights(user_id, date).await?;
    Ok(Json(insights))
}

pub async fn handle_get_weekly(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<WeeklyInsights>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let insights = service.get_weekly_insights(user_id, date).await?;
    Ok(Json(insights))
}

pub async fn handle_get_trends(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<Vec<DailyInsights>>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let end_date = query.end_date.unwrap_or_else(|| Utc::now().date_naive());
    let start_date = query.start_date.unwrap_or_else(|| end_date - Duration::days(30));
    let trends = service.get_trends(user_id, start_date, end_date).await?;
    Ok(Json(trends))
}

pub async fn handle_get_recommendations(
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<Vec<WellbeingRecommendation>>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let recommendations = service.generate_recommendations(user_id).await;
    Ok(Json(recommendations))
}

pub async fn handle_get_settings(
    State(pool): State<Arc<DbPool>>,
) -> Result<Json<InsightsSettings>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let settings = service.get_settings(user_id).await?;
    Ok(Json(settings))
}

pub async fn handle_update_settings(
    State(pool): State<Arc<DbPool>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<InsightsSettings>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let settings = service.update_settings(user_id, req).await?;
    Ok(Json(settings))
}

pub async fn handle_update_focus_mode(
    State(pool): State<Arc<DbPool>>,
    Json(req): Json<UpdateFocusModeRequest>,
) -> Result<Json<FocusMode>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let focus_mode = service.update_focus_mode(user_id, req).await?;
    Ok(Json(focus_mode))
}

pub async fn handle_get_app_breakdown(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<InsightsQuery>,
) -> Result<Json<Vec<AppUsageSummary>>, InsightsError> {
    let service = InsightsService::new((*pool).clone());
    let user_id = Uuid::nil();
    let date = query.start_date.unwrap_or_else(|| Utc::now().date_naive());
    let breakdown = service.get_app_breakdown(user_id, date).await?;
    Ok(Json(breakdown))
}

pub fn configure_insights_routes() -> Router<Arc<DbPool>> {
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
