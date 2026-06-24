use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub content_type: String,
    pub content_url: Option<String>,
    pub order: i32,
    pub duration_minutes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    pub title: String,
    pub content_type: String,
    pub content_url: Option<String>,
    pub order: i32,
    pub duration_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub progress_percent: f32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub certificate_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub course_title: String,
    pub progress_percent: f32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub certificate_id: Option<Uuid>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub credential_url: Option<String>,
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub course_id: Uuid,
    pub course_title: String,
    pub issued_at: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub credential_url: Option<String>,
    pub verification_code: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub badge_type: String,
    pub earned_at: DateTime<Utc>,
    pub criteria_met: serde_json::Value,
    pub badge_name: Option<String>,
    pub badge_description: Option<String>,
    pub badge_icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementResponse {
    pub id: Uuid,
    pub badge_type: String,
    pub badge_name: Option<String>,
    pub badge_description: Option<String>,
    pub badge_icon_url: Option<String>,
    pub earned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPath {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub required_courses: Vec<Uuid>,
    pub estimated_duration: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLearningPathRequest {
    pub name: String,
    pub description: Option<String>,
    pub required_courses: Vec<Uuid>,
    pub estimated_duration: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPathResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub required_courses: Vec<Uuid>,
    pub courses: Vec<crate::types::CourseResponse>,
    pub estimated_duration: i32,
    pub progress_percent: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeDefinition {
    pub badge_type: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub xp_reward: i32,
    pub criteria: BadgeCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeCriteria {
    pub action: String,
    pub threshold: i32,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub user_name: String,
    pub avatar_url: Option<String>,
    pub total_xp: i32,
    pub level: i32,
    pub badges_count: i32,
    pub courses_completed: i32,
    pub rank: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: i32,
    pub reason: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLevelInfo {
    pub user_id: Uuid,
    pub total_xp: i32,
    pub level: i32,
    pub xp_to_next_level: i32,
    pub xp_in_current_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollRequest {
    pub user_id: Uuid,
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdateRequest {
    pub progress_percent: f32,
}

fn xp_for_level(level: i32) -> i32 {
    level * 100 + (level * level * 10)
}

pub fn calculate_level(total_xp: i32) -> i32 {
    let mut level = 0;
    let mut remaining = total_xp;
    loop {
        let needed = xp_for_level(level + 1);
        if remaining < needed {
            break;
        }
        remaining -= needed;
        level += 1;
    }
    level
}

pub fn xp_progress(total_xp: i32) -> UserLevelInfo {
    let level = calculate_level(total_xp);
    let mut xp_accounted = 0;
    for l in 0..level {
        xp_accounted += xp_for_level(l + 1);
    }
    let xp_in_current = total_xp - xp_accounted;
    let xp_to_next = xp_for_level(level + 1);
    UserLevelInfo {
        user_id: Uuid::nil(),
        total_xp,
        level,
        xp_to_next_level: xp_to_next,
        xp_in_current_level: xp_in_current,
    }
}
