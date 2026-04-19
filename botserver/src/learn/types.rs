//! Types for the Learn module (LMS)
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::shared::schema::learn::*;

// ============================================================================
// DATA MODELS
// ============================================================================

// ----- Course Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_courses)]
pub struct Course {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub difficulty: String,
    pub duration_minutes: i32,
    pub thumbnail_url: Option<String>,
    pub is_mandatory: bool,
    pub due_days: Option<i32>,
    pub is_published: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCourseRequest {
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub difficulty: Option<String>,
    pub duration_minutes: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub is_mandatory: Option<bool>,
    pub due_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCourseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub difficulty: Option<String>,
    pub duration_minutes: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub is_mandatory: Option<bool>,
    pub due_days: Option<i32>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub difficulty: String,
    pub duration_minutes: i32,
    pub thumbnail_url: Option<String>,
    pub is_mandatory: bool,
    pub due_days: Option<i32>,
    pub is_published: bool,
    pub lessons_count: i32,
    pub enrolled_count: i32,
    pub completion_rate: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseDetailResponse {
    pub course: CourseResponse,
    pub lessons: Vec<LessonResponse>,
    pub quiz: Option<QuizResponse>,
    pub user_progress: Option<UserProgressResponse>,
}

// ----- Lesson Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_lessons)]
pub struct Lesson {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub content_type: String,
    pub lesson_order: i32,
    pub duration_minutes: i32,
    pub video_url: Option<String>,
    pub attachments: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLessonRequest {
    pub title: String,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub duration_minutes: Option<i32>,
    pub video_url: Option<String>,
    pub attachments: Option<Vec<AttachmentInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLessonRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub lesson_order: Option<i32>,
    pub duration_minutes: Option<i32>,
    pub video_url: Option<String>,
    pub attachments: Option<Vec<AttachmentInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub name: String,
    pub url: String,
    pub file_type: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonResponse {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub content_type: String,
    pub lesson_order: i32,
    pub duration_minutes: i32,
    pub video_url: Option<String>,
    pub attachments: Vec<AttachmentInfo>,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
}

// ----- Quiz Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_quizzes)]
pub struct Quiz {
    pub id: Uuid,
    pub lesson_id: Option<Uuid>,
    pub course_id: Uuid,
    pub title: String,
    pub passing_score: i32,
    pub time_limit_minutes: Option<i32>,
    pub max_attempts: Option<i32>,
    pub questions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub id: Uuid,
    pub text: String,
    pub question_type: QuestionType,
    pub options: Vec<QuizOption>,
    pub correct_answers: Vec<usize>,
    pub explanation: Option<String>,
    pub points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    SingleChoice,
    MultipleChoice,
    TrueFalse,
    ShortAnswer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizOption {
    pub text: String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuizRequest {
    pub lesson_id: Option<Uuid>,
    pub title: String,
    pub passing_score: Option<i32>,
    pub time_limit_minutes: Option<i32>,
    pub max_attempts: Option<i32>,
    pub questions: Vec<QuizQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResponse {
    pub id: Uuid,
    pub course_id: Uuid,
    pub lesson_id: Option<Uuid>,
    pub title: String,
    pub passing_score: i32,
    pub time_limit_minutes: Option<i32>,
    pub max_attempts: Option<i32>,
    pub questions_count: i32,
    pub total_points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizSubmission {
    pub answers: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResult {
    pub quiz_id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    pub max_score: i32,
    pub percentage: f32,
    pub passed: bool,
    pub time_taken_minutes: i32,
    pub answers_breakdown: Vec<AnswerResult>,
    pub attempt_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerResult {
    pub question_id: Uuid,
    pub is_correct: bool,
    pub points_earned: i32,
    pub correct_answers: Vec<usize>,
    pub user_answers: Vec<usize>,
    pub explanation: Option<String>,
}

// ----- Progress Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_user_progress)]
pub struct UserProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub lesson_id: Option<Uuid>,
    pub status: String,
    pub quiz_score: Option<i32>,
    pub quiz_attempts: i32,
    pub time_spent_minutes: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_accessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgressResponse {
    pub course_id: Uuid,
    pub course_title: String,
    pub status: ProgressStatus,
    pub completion_percentage: f32,
    pub lessons_completed: i32,
    pub lessons_total: i32,
    pub quiz_score: Option<i32>,
    pub quiz_passed: bool,
    pub time_spent_minutes: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_accessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

impl From<&str> for ProgressStatus {
    fn from(s: &str) -> Self {
        match s {
            "in_progress" => Self::InProgress,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::NotStarted,
        }
    }
}

impl std::fmt::Display for ProgressStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "not_started"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

// ----- Assignment Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_course_assignments)]
pub struct CourseAssignment {
    pub id: Uuid,
    pub course_id: Uuid,
    pub user_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_mandatory: bool,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reminder_sent: bool,
    pub reminder_sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssignmentRequest {
    pub course_id: Uuid,
    pub user_ids: Vec<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_mandatory: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentResponse {
    pub id: Uuid,
    pub course_id: Uuid,
    pub course_title: String,
    pub user_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_mandatory: bool,
    pub is_overdue: bool,
    pub days_until_due: Option<i64>,
    pub status: ProgressStatus,
    pub assigned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ----- Certificate Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_certificates)]
pub struct Certificate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub score: i32,
    pub certificate_url: Option<String>,
    pub verification_code: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub course_id: Uuid,
    pub course_title: String,
    pub issued_at: DateTime<Utc>,
    pub score: i32,
    pub verification_code: String,
    pub certificate_url: Option<String>,
    pub is_valid: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateVerification {
    pub is_valid: bool,
    pub certificate: Option<CertificateResponse>,
    pub message: String,
}

// ----- Category Models -----

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = learn_categories)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub courses_count: i32,
    pub children: Vec<CategoryResponse>,
}

// ----- Query Filters -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseFilters {
    pub category: Option<String>,
    pub difficulty: Option<String>,
    pub is_mandatory: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressFilters {
    pub status: Option<String>,
    pub course_id: Option<Uuid>,
}

// ----- Statistics -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnStatistics {
    pub total_courses: i64,
    pub total_lessons: i64,
    pub total_users_learning: i64,
    pub courses_completed: i64,
    pub certificates_issued: i64,
    pub average_completion_rate: f32,
    pub mandatory_compliance_rate: f32,
    pub popular_categories: Vec<CategoryStats>,
    pub recent_completions: Vec<RecentCompletion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub courses_count: i64,
    pub enrolled_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentCompletion {
    pub user_id: Uuid,
    pub user_name: String,
    pub course_title: String,
    pub completed_at: DateTime<Utc>,
    pub score: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLearnStats {
    pub courses_enrolled: i64,
    pub courses_completed: i64,
    pub courses_in_progress: i64,
    pub total_time_spent_hours: f32,
    pub certificates_earned: i64,
    pub average_score: f32,
    pub pending_mandatory: i64,
    pub overdue_assignments: i64,
}
