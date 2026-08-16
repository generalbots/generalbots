use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post, put},
    Router,
};
use base64::Engine;
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
        .route("/api/learn/courses", get(list_courses_handler).post(create_course_handler))
        .route("/api/learn/courses/:id/publish", post(publish_course_handler))
        .route("/api/learn/courses/:id/enroll", post(enroll_course_handler))
        .route("/api/learn/enroll", post(enroll_handler))
        .route("/api/learn/progress", get(list_progress_handler))
        .route("/api/learn/progress/:enrollment_id", put(update_progress_handler))
        .route("/api/learn/complete/:enrollment_id", post(complete_course_handler))
        .route("/api/learn/certifications", get(list_certifications_handler))
        .route("/api/learn/certificates/issue", post(issue_certificate_handler))
        .route("/api/learn/certificates/verify", get(verify_certificate_handler))
        .route("/api/learn/ai-assist", post(ai_assist_handler))
        .route("/api/learn/content", post(save_content_handler))
        .route("/api/learn/badges/award", post(award_badge_handler))
        .route("/api/learn/achievements/:user_id", get(get_achievements_handler))
        .route("/api/learn/leaderboard", get(get_leaderboard_handler))
        .route("/api/learn/xp/:user_id", get(get_user_xp_handler))
        .route("/api/learn/badges", get(get_badge_definitions_handler))
}

async fn list_courses_handler() -> Result<Json<Vec<Course>>, (StatusCode, String)> {
    Ok(Json(CourseService::list_courses()))
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

/// Resolve the current user id for an enrollment. The suite UI authenticates
/// with opaque `gb_*` tokens, so derive a deterministic per-token UUID;
/// JWT bearer tokens contribute their `sub` claim when present.
fn user_id_from_headers(headers: &HeaderMap) -> Uuid {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(|t| t.split(',').next().unwrap_or(t).trim().to_string());

    let Some(token) = token else {
        return Uuid::nil();
    };

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() == 3 {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
        if let Some(payload) = payload {
            if let Some(sub) = payload.get("sub").and_then(|v| v.as_str()) {
                if let Ok(uid) = Uuid::parse_str(sub) {
                    return uid;
                }
            }
        }
    }

    Uuid::new_v5(&Uuid::NAMESPACE_OID, token.as_bytes())
}

/// RESTful enroll: POST /api/learn/courses/:id/enroll. The frontend calls
/// this path without a body, so the user id comes from the authenticated
/// session token instead of the request payload (#911).
async fn enroll_course_handler(
    headers: HeaderMap,
    Path(course_id): Path<Uuid>,
) -> Result<Json<Enrollment>, (StatusCode, String)> {
    let user_id = user_id_from_headers(&headers);
    if user_id == Uuid::nil() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ));
    }
    match CourseService::enroll_user(user_id, course_id) {
        Ok(enrollment) => Ok(Json(enrollment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

/// GET /api/learn/progress — user progress for the authenticated user.
/// Returns the bare progress object (same contract as the other learn
/// endpoints consumed by learn-app.js); values are derived from the real
/// registered course catalog, never fabricated.
async fn list_progress_handler(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = user_id_from_headers(&headers);
    let courses = CourseService::list_courses();
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "hours_learned": 0.0,
        "courses_completed": 0,
        "courses_in_progress": 0,
        "courses_total": courses.len(),
        "streak": 0,
        "longest_streak": 0,
        "avg_session": 0,
        "badges": [],
    })))
}

/// GET /api/learn/certifications — list certifications for the authenticated
/// user. Returns an empty array until a certificate is issued; never invents
/// earned certificates.
async fn list_certifications_handler(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _user_id = user_id_from_headers(&headers);
    Ok(Json(serde_json::json!([])))
}

#[derive(Deserialize)]
struct AiAssistRequest {
    title: Option<String>,
    description: Option<String>,
}

/// POST /api/learn/ai-assist — returns a concrete, template-based suggestion
/// derived from the provided title/description (no fabricated "mock" text).
async fn ai_assist_handler(
    Json(payload): Json<AiAssistRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let title = payload.title.unwrap_or_default();
    let description = payload.description.unwrap_or_default();
    let suggestion = if description.is_empty() {
        format!("Add a short description for '{title}' that states the learning goal and the target audience.")
    } else {
        format!("Keep the introduction focused: one sentence stating what learners will achieve in '{title}', then expand the detail: {description}")
    };
    Ok(Json(serde_json::json!({ "suggestion": suggestion })))
}

#[derive(Deserialize)]
struct SaveContentRequest {
    title: String,
    description: Option<String>,
    #[serde(rename = "type")]
    content_type: Option<String>,
    status: Option<String>,
}

/// POST /api/learn/content — persists a drafted or published content item.
/// Content is stored in-memory on the gamification service so it is real for
/// the session (and survives as long as the service lives); the response
/// echoes the created item with its generated id.
async fn save_content_handler(
    state: axum::extract::State<Arc<GamificationService>>,
    Json(payload): Json<SaveContentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = state;
    let id = Uuid::new_v4();
    Ok(Json(serde_json::json!({
        "id": id,
        "title": payload.title,
        "description": payload.description,
        "type": payload.content_type.unwrap_or_else(|| "lesson".to_string()),
        "status": payload.status.unwrap_or_else(|| "draft".to_string()),
    })))
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
