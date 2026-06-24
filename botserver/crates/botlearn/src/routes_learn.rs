use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::certification::*;
use crate::course::*;
use crate::gamification::*;
use crate::models::*;
use crate::types::*;

pub fn configure_learn_api_routes() -> Router<Arc<GamificationService>> {
    Router::new()
        .route("/api/learn/courses", post(create_course_handler))
        .route("/api/learn/courses/{id}/publish", post(publish_course_handler))
        .route("/api/learn/enroll", post(enroll_handler))
        .route("/api/learn/progress/{enrollment_id}", put(update_progress_handler))
        .route("/api/learn/complete/{enrollment_id}", post(complete_course_handler))
        .route("/api/learn/certificates/issue", post(issue_certificate_handler))
        .route("/api/learn/certificates/verify", get(verify_certificate_handler))
        .route("/api/learn/badges/award", post(award_badge_handler))
        .route("/api/learn/achievements/{user_id}", get(get_achievements_handler))
        .route("/api/learn/leaderboard", get(get_leaderboard_handler))
        .route("/api/learn/xp/{user_id}", get(get_user_xp_handler))
        .route("/api/learn/badges", get(get_badge_definitions_handler))
}

async fn create_course_handler(
    Json(payload): Json<CreateCourseRequest>,
) -> Result<Json<Course>, (StatusCode, String)> {
    match CourseService::create_course(payload) {
        Ok(course) => Ok(Json(course)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn publish_course_handler(
    Path(course_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Err((StatusCode::NOT_FOUND, format!("Course {} not found", course_id)))
}

async fn enroll_handler(
    Json(payload): Json<EnrollRequest>,
) -> Result<Json<Enrollment>, (StatusCode, String)> {
    match CourseService::enroll_user(payload.user_id, payload.course_id) {
        Ok(enrollment) => Ok(Json(enrollment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn update_progress_handler(
    Path(enrollment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Err((StatusCode::NOT_FOUND, format!("Enrollment {} not found", enrollment_id)))
}

async fn complete_course_handler(
    Path(enrollment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Err((StatusCode::NOT_FOUND, format!("Enrollment {} not found", enrollment_id)))
}

#[derive(Deserialize)]
struct IssueCertPayload {
    user_id: Uuid,
    course_id: Uuid,
    user_name: String,
    course_title: String,
    score: Option<i32>,
}

async fn issue_certificate_handler(
    Json(payload): Json<IssueCertPayload>,
) -> Result<Json<CertificationResponse>, (StatusCode, String)> {
    let score = payload.score.unwrap_or(85);
    match CertificationService::issue_certificate(
        payload.user_id,
        payload.course_id,
        &payload.user_name,
        &payload.course_title,
        score,
    ) {
        Ok(cert) => Ok(Json(cert)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn verify_certificate_handler(
    Query(query): Query<VerifyQuery>,
) -> Result<Json<CertificateVerification>, (StatusCode, String)> {
    match CertificationService::verify_certificate(&query.code) {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct AwardBadgePayload {
    user_id: Uuid,
    badge_type: String,
}

async fn award_badge_handler(
    mut state: axum::extract::State<Arc<GamificationService>>,
    Json(payload): Json<AwardBadgePayload>,
) -> Result<Json<AchievementResponse>, (StatusCode, String)> {
    let service = Arc::make_mut(&mut state);
    let result = GamificationService::award_badge(service, payload.user_id, &payload.badge_type);
    match result {
        Some(ach) => Ok(Json(ach)),
        None => Err((StatusCode::NOT_FOUND, "Badge type not found".to_string())),
    }
}

async fn get_achievements_handler(
    state: axum::extract::State<Arc<GamificationService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<AchievementResponse>>, (StatusCode, String)> {
    let achievements = state.get_user_achievements(user_id);
    Ok(Json(achievements))
}

#[derive(Deserialize)]
struct LeaderboardQuery {
    limit: Option<usize>,
}

async fn get_leaderboard_handler(
    mut state: axum::extract::State<Arc<GamificationService>>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<Vec<LeaderboardEntry>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20);
    let leaderboard = Arc::get_mut(&mut state).map(|s| s.get_leaderboard(limit));
    match leaderboard {
        Some(entries) => Ok(Json(entries)),
        None => Ok(Json(Vec::new())),
    }
}

async fn get_user_xp_handler(
    state: axum::extract::State<Arc<GamificationService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserLevelInfo>, (StatusCode, String)> {
    let level = state.get_user_level(user_id);
    Ok(Json(level))
}

async fn get_badge_definitions_handler(
    state: axum::extract::State<Arc<GamificationService>>,
) -> Result<Json<Vec<BadgeDefinition>>, (StatusCode, String)> {
    let badges = state.get_badge_definitions().to_vec();
    Ok(Json(badges))
}
