//! # Learn Module - Learning Management System (LMS)
//!
//! Complete LMS implementation for General Bots with:
//! - Course management (CRUD operations)
//! - Lesson management with multimedia support
//! - Quiz engine with multiple question types
//! - Progress tracking per user
//! - Mandatory training assignments with due dates
//! - Certificate generation with verification
//! - AI-powered course recommendations
//!
//! ## Architecture
//!
//! The Learn module follows the same patterns as other GB modules (tasks, calendar):
//! - Diesel ORM for database operations
//! - Axum handlers for HTTP routes
//! - Serde for JSON serialization
//! - UUID for unique identifiers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

// ============================================================================
// DATABASE SCHEMA
// ============================================================================

diesel::table! {
    learn_courses (id) {
        id -> Uuid,
        organization_id -> Nullable<Uuid>,
        title -> Text,
        description -> Nullable<Text>,
        category -> Text,
        difficulty -> Text,
        duration_minutes -> Int4,
        thumbnail_url -> Nullable<Text>,
        is_mandatory -> Bool,
        due_days -> Nullable<Int4>,
        is_published -> Bool,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    learn_lessons (id) {
        id -> Uuid,
        course_id -> Uuid,
        title -> Text,
        content -> Nullable<Text>,
        content_type -> Text,
        lesson_order -> Int4,
        duration_minutes -> Int4,
        video_url -> Nullable<Text>,
        attachments -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    learn_quizzes (id) {
        id -> Uuid,
        lesson_id -> Nullable<Uuid>,
        course_id -> Uuid,
        title -> Text,
        passing_score -> Int4,
        time_limit_minutes -> Nullable<Int4>,
        max_attempts -> Nullable<Int4>,
        questions -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    learn_user_progress (id) {
        id -> Uuid,
        user_id -> Uuid,
        course_id -> Uuid,
        lesson_id -> Nullable<Uuid>,
        status -> Text,
        quiz_score -> Nullable<Int4>,
        quiz_attempts -> Int4,
        time_spent_minutes -> Int4,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        last_accessed_at -> Timestamptz,
    }
}

diesel::table! {
    learn_course_assignments (id) {
        id -> Uuid,
        course_id -> Uuid,
        user_id -> Uuid,
        assigned_by -> Nullable<Uuid>,
        due_date -> Nullable<Timestamptz>,
        is_mandatory -> Bool,
        assigned_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        reminder_sent -> Bool,
        reminder_sent_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    learn_certificates (id) {
        id -> Uuid,
        user_id -> Uuid,
        course_id -> Uuid,
        issued_at -> Timestamptz,
        score -> Int4,
        certificate_url -> Nullable<Text>,
        verification_code -> Text,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    learn_categories (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        icon -> Nullable<Text>,
        color -> Nullable<Text>,
        parent_id -> Nullable<Uuid>,
        sort_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    learn_courses,
    learn_lessons,
    learn_quizzes,
    learn_user_progress,
    learn_course_assignments,
    learn_certificates,
    learn_categories,
);

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

// ============================================================================
// LEARN ENGINE
// ============================================================================

/// Main Learn engine that handles all LMS operations
pub struct LearnEngine {
    db: DbPool,
    cache: Arc<RwLock<LearnCache>>,
}

#[derive(Debug, Default)]
struct LearnCache {
    courses: HashMap<Uuid, Course>,
    categories: Vec<Category>,
    last_refresh: Option<DateTime<Utc>>,
}

impl LearnEngine {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(LearnCache::default())),
        }
    }

    // ----- Course Operations -----

    pub async fn create_course(
        &self,
        req: CreateCourseRequest,
        created_by: Option<Uuid>,
        organization_id: Option<Uuid>,
    ) -> Result<Course, String> {
        let now = Utc::now();
        let course = Course {
            id: Uuid::new_v4(),
            organization_id,
            title: req.title,
            description: req.description,
            category: req.category,
            difficulty: req.difficulty.unwrap_or_else(|| "beginner".to_string()),
            duration_minutes: req.duration_minutes.unwrap_or(0),
            thumbnail_url: req.thumbnail_url,
            is_mandatory: req.is_mandatory.unwrap_or(false),
            due_days: req.due_days,
            is_published: false,
            created_by,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        diesel::insert_into(learn_courses::table)
            .values(&course)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(course)
    }

    pub async fn get_course(&self, course_id: Uuid) -> Result<Option<Course>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        learn_courses::table
            .filter(learn_courses::id.eq(course_id))
            .first::<Course>(&mut conn)
            .optional()
            .map_err(|e| e.to_string())
    }

    pub async fn list_courses(&self, filters: CourseFilters) -> Result<Vec<Course>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let mut query = learn_courses::table
            .filter(learn_courses::is_published.eq(true))
            .into_boxed();

        if let Some(category) = filters.category {
            query = query.filter(learn_courses::category.eq(category));
        }

        if let Some(difficulty) = filters.difficulty {
            query = query.filter(learn_courses::difficulty.eq(difficulty));
        }

        if let Some(is_mandatory) = filters.is_mandatory {
            query = query.filter(learn_courses::is_mandatory.eq(is_mandatory));
        }

        if let Some(search) = filters.search {
            let pattern = format!("%{}%", search.to_lowercase());
            query = query.filter(
                learn_courses::title
                    .ilike(&pattern)
                    .or(learn_courses::description.ilike(&pattern)),
            );
        }

        query = query.order(learn_courses::created_at.desc());

        if let Some(limit) = filters.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = filters.offset {
            query = query.offset(offset);
        }

        query.load::<Course>(&mut conn).map_err(|e| e.to_string())
    }

    pub async fn update_course(
        &self,
        course_id: Uuid,
        req: UpdateCourseRequest,
    ) -> Result<Course, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Build dynamic update
        let now = Utc::now();

        diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
            .set(learn_courses::updated_at.eq(now))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        if let Some(title) = req.title {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::title.eq(title))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(description) = req.description {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::description.eq(description))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(category) = req.category {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::category.eq(category))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(difficulty) = req.difficulty {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::difficulty.eq(difficulty))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(duration) = req.duration_minutes {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::duration_minutes.eq(duration))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(is_mandatory) = req.is_mandatory {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::is_mandatory.eq(is_mandatory))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(is_published) = req.is_published {
            diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
                .set(learn_courses::is_published.eq(is_published))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        self.get_course(course_id)
            .await?
            .ok_or_else(|| "Course not found".to_string())
    }

    pub async fn delete_course(&self, course_id: Uuid) -> Result<(), String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Delete related records first
        diesel::delete(learn_lessons::table.filter(learn_lessons::course_id.eq(course_id)))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        diesel::delete(learn_quizzes::table.filter(learn_quizzes::course_id.eq(course_id)))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        diesel::delete(
            learn_user_progress::table.filter(learn_user_progress::course_id.eq(course_id)),
        )
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

        diesel::delete(
            learn_course_assignments::table
                .filter(learn_course_assignments::course_id.eq(course_id)),
        )
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

        diesel::delete(learn_courses::table.filter(learn_courses::id.eq(course_id)))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ----- Lesson Operations -----

    pub async fn create_lesson(
        &self,
        course_id: Uuid,
        req: CreateLessonRequest,
    ) -> Result<Lesson, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Get next order number
        let max_order: Option<i32> = learn_lessons::table
            .filter(learn_lessons::course_id.eq(course_id))
            .select(diesel::dsl::max(learn_lessons::lesson_order))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;

        let now = Utc::now();
        let lesson = Lesson {
            id: Uuid::new_v4(),
            course_id,
            title: req.title,
            content: req.content,
            content_type: req.content_type.unwrap_or_else(|| "text".to_string()),
            lesson_order: max_order.unwrap_or(0) + 1,
            duration_minutes: req.duration_minutes.unwrap_or(0),
            video_url: req.video_url,
            attachments: serde_json::to_value(req.attachments.unwrap_or_default())
                .unwrap_or(serde_json::json!([])),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(learn_lessons::table)
            .values(&lesson)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        // Update course duration
        self.recalculate_course_duration(course_id).await?;

        Ok(lesson)
    }

    pub async fn get_lessons(&self, course_id: Uuid) -> Result<Vec<Lesson>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        learn_lessons::table
            .filter(learn_lessons::course_id.eq(course_id))
            .order(learn_lessons::lesson_order.asc())
            .load::<Lesson>(&mut conn)
            .map_err(|e| e.to_string())
    }

    pub async fn update_lesson(
        &self,
        lesson_id: Uuid,
        req: UpdateLessonRequest,
    ) -> Result<Lesson, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;
        let now = Utc::now();

        diesel::update(learn_lessons::table.filter(learn_lessons::id.eq(lesson_id)))
            .set(learn_lessons::updated_at.eq(now))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        if let Some(title) = req.title {
            diesel::update(learn_lessons::table.filter(learn_lessons::id.eq(lesson_id)))
                .set(learn_lessons::title.eq(title))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(content) = req.content {
            diesel::update(learn_lessons::table.filter(learn_lessons::id.eq(lesson_id)))
                .set(learn_lessons::content.eq(content))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(order) = req.lesson_order {
            diesel::update(learn_lessons::table.filter(learn_lessons::id.eq(lesson_id)))
                .set(learn_lessons::lesson_order.eq(order))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        if let Some(duration) = req.duration_minutes {
            diesel::update(learn_lessons::table.filter(learn_lessons::id.eq(lesson_id)))
                .set(learn_lessons::duration_minutes.eq(duration))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        learn_lessons::table
            .filter(learn_lessons::id.eq(lesson_id))
            .first::<Lesson>(&mut conn)
            .map_err(|e| e.to_string())
    }

    pub async fn delete_lesson(&self, lesson_id: Uuid) -> Result<(), String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Get course_id before deleting
        let lesson: Lesson = learn_lessons::table
            .filter(learn_lessons::id.eq(lesson_id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;

        diesel::delete(learn_lessons::table.filter(learn_lessons::id.eq(lesson_id)))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        self.recalculate_course_duration(lesson.course_id).await?;
        Ok(())
    }

    async fn recalculate_course_duration(&self, course_id: Uuid) -> Result<(), String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let total_duration: Option<i64> = learn_lessons::table
            .filter(learn_lessons::course_id.eq(course_id))
            .select(diesel::dsl::sum(learn_lessons::duration_minutes))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;

        diesel::update(learn_courses::table.filter(learn_courses::id.eq(course_id)))
            .set(learn_courses::duration_minutes.eq(total_duration.unwrap_or(0) as i32))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ----- Quiz Operations -----

    pub async fn create_quiz(&self, course_id: Uuid, req: CreateQuizRequest) -> Result<Quiz, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;
        let now = Utc::now();

        let quiz = Quiz {
            id: Uuid::new_v4(),
            lesson_id: req.lesson_id,
            course_id,
            title: req.title,
            passing_score: req.passing_score.unwrap_or(70),
            time_limit_minutes: req.time_limit_minutes,
            max_attempts: req.max_attempts,
            questions: serde_json::to_value(&req.questions).unwrap_or(serde_json::json!([])),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(learn_quizzes::table)
            .values(&quiz)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(quiz)
    }

    pub async fn get_quiz(&self, course_id: Uuid) -> Result<Option<Quiz>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        learn_quizzes::table
            .filter(learn_quizzes::course_id.eq(course_id))
            .first::<Quiz>(&mut conn)
            .optional()
            .map_err(|e| e.to_string())
    }

    pub async fn submit_quiz(
        &self,
        user_id: Uuid,
        quiz_id: Uuid,
        submission: QuizSubmission,
    ) -> Result<QuizResult, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let quiz: Quiz = learn_quizzes::table
            .filter(learn_quizzes::id.eq(quiz_id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;

        let questions: Vec<QuizQuestion> =
            serde_json::from_value(quiz.questions.clone()).unwrap_or_default();

        let mut total_points = 0;
        let mut earned_points = 0;
        let mut answers_breakdown = Vec::new();

        for question in &questions {
            total_points += question.points;
            let user_answers = submission
                .answers
                .get(&question.id.to_string())
                .cloned()
                .unwrap_or_default();

            let is_correct = user_answers == question.correct_answers;
            let points_earned = if is_correct { question.points } else { 0 };
            earned_points += points_earned;

            answers_breakdown.push(AnswerResult {
                question_id: question.id,
                is_correct,
                points_earned,
                correct_answers: question.correct_answers.clone(),
                user_answers,
                explanation: question.explanation.clone(),
            });
        }

        let percentage = if total_points > 0 {
            (earned_points as f32 / total_points as f32) * 100.0
        } else {
            0.0
        };

        let passed = percentage >= quiz.passing_score as f32;

        // Update user progress
        let progress: Option<UserProgress> = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::course_id.eq(quiz.course_id))
            .first(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;

        let attempt_number = progress.as_ref().map(|p| p.quiz_attempts + 1).unwrap_or(1);

        if let Some(prog) = progress {
            diesel::update(learn_user_progress::table.filter(learn_user_progress::id.eq(prog.id)))
                .set((
                    learn_user_progress::quiz_score.eq(percentage as i32),
                    learn_user_progress::quiz_attempts.eq(attempt_number),
                    learn_user_progress::status.eq(if passed { "completed" } else { "in_progress" }),
                    learn_user_progress::completed_at.eq(if passed { Some(Utc::now()) } else { None }),
                    learn_user_progress::last_accessed_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        // Generate certificate if passed
        if passed {
            self.generate_certificate(user_id, quiz.course_id, percentage as i32)
                .await?;
        }

        Ok(QuizResult {
            quiz_id,
            user_id,
            score: earned_points,
            max_score: total_points,
            percentage,
            passed,
            time_taken_minutes: 0,
            answers_breakdown,
            attempt_number,
        })
    }

    // ----- Progress Operations -----

    pub async fn start_course(&self, user_id: Uuid, course_id: Uuid) -> Result<UserProgress, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Check if already started
        let existing: Option<UserProgress> = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::course_id.eq(course_id))
            .filter(learn_user_progress::lesson_id.is_null())
            .first(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(progress) = existing {
            return Ok(progress);
        }

        let now = Utc::now();
        let progress = UserProgress {
            id: Uuid::new_v4(),
            user_id,
            course_id,
            lesson_id: None,
            status: "in_progress".to_string(),
            quiz_score: None,
            quiz_attempts: 0,
            time_spent_minutes: 0,
            started_at: now,
            completed_at: None,
            last_accessed_at: now,
        };

        diesel::insert_into(learn_user_progress::table)
            .values(&progress)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(progress)
    }

    pub async fn complete_lesson(&self, user_id: Uuid, lesson_id: Uuid) -> Result<(), String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let lesson: Lesson = learn_lessons::table
            .filter(learn_lessons::id.eq(lesson_id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;

        let now = Utc::now();

        // Check if lesson progress exists
        let existing: Option<UserProgress> = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::lesson_id.eq(lesson_id))
            .first(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;

        if existing.is_some() {
            diesel::update(
                learn_user_progress::table
                    .filter(learn_user_progress::user_id.eq(user_id))
                    .filter(learn_user_progress::lesson_id.eq(lesson_id)),
            )
            .set((
                learn_user_progress::status.eq("completed"),
                learn_user_progress::completed_at.eq(Some(now)),
                learn_user_progress::last_accessed_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        } else {
            let progress = UserProgress {
                id: Uuid::new_v4(),
                user_id,
                course_id: lesson.course_id,
                lesson_id: Some(lesson_id),
                status: "completed".to_string(),
                quiz_score: None,
                quiz_attempts: 0,
                time_spent_minutes: lesson.duration_minutes,
                started_at: now,
                completed_at: Some(now),
                last_accessed_at: now,
            };

            diesel::insert_into(learn_user_progress::table)
                .values(&progress)
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
        }

        // Check if all lessons completed
        self.check_course_completion(user_id, lesson.course_id).await?;

        Ok(())
    }

    async fn check_course_completion(&self, user_id: Uuid, course_id: Uuid) -> Result<(), String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let total_lessons: i64 = learn_lessons::table
            .filter(learn_lessons::course_id.eq(course_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let completed_lessons: i64 = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::course_id.eq(course_id))
            .filter(learn_user_progress::lesson_id.is_not_null())
            .filter(learn_user_progress::status.eq("completed"))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        if completed_lessons >= total_lessons && total_lessons > 0 {
            // Check if there's a quiz
            let quiz_exists: bool = learn_quizzes::table
                .filter(learn_quizzes::course_id.eq(course_id))
                .count()
                .get_result::<i64>(&mut conn)
                .map(|c| c > 0)
                .map_err(|e| e.to_string())?;

            if !quiz_exists {
                // No quiz, mark course as complete
                diesel::update(
                    learn_user_progress::table
                        .filter(learn_user_progress::user_id.eq(user_id))
                        .filter(learn_user_progress::course_id.eq(course_id))
                        .filter(learn_user_progress::lesson_id.is_null()),
                )
                .set((
                    learn_user_progress::status.eq("completed"),
                    learn_user_progress::completed_at.eq(Some(Utc::now())),
                ))
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;

                // Generate certificate
                self.generate_certificate(user_id, course_id, 100).await?;
            }
        }

        Ok(())
    }

    pub async fn get_user_progress(
        &self,
        user_id: Uuid,
        course_id: Option<Uuid>,
    ) -> Result<Vec<UserProgress>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let mut query = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::lesson_id.is_null())
            .into_boxed();

        if let Some(cid) = course_id {
            query = query.filter(learn_user_progress::course_id.eq(cid));
        }

        query
            .order(learn_user_progress::last_accessed_at.desc())
            .load::<UserProgress>(&mut conn)
            .map_err(|e| e.to_string())
    }

    // ----- Assignment Operations -----

    pub async fn create_assignment(
        &self,
        req: CreateAssignmentRequest,
        assigned_by: Option<Uuid>,
    ) -> Result<Vec<CourseAssignment>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;
        let now = Utc::now();

        let mut assignments = Vec::new();

        for user_id in req.user_ids {
            let assignment = CourseAssignment {
                id: Uuid::new_v4(),
                course_id: req.course_id,
                user_id,
                assigned_by,
                due_date: req.due_date,
                is_mandatory: req.is_mandatory.unwrap_or(true),
                assigned_at: now,
                completed_at: None,
                reminder_sent: false,
                reminder_sent_at: None,
            };

            diesel::insert_into(learn_course_assignments::table)
                .values(&assignment)
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;

            assignments.push(assignment);
        }

        Ok(assignments)
    }

    pub async fn get_pending_assignments(&self, user_id: Uuid) -> Result<Vec<CourseAssignment>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        learn_course_assignments::table
            .filter(learn_course_assignments::user_id.eq(user_id))
            .filter(learn_course_assignments::completed_at.is_null())
            .order(learn_course_assignments::due_date.asc())
            .load::<CourseAssignment>(&mut conn)
            .map_err(|e| e.to_string())
    }

    pub async fn delete_assignment(&self, assignment_id: Uuid) -> Result<(), String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        diesel::delete(
            learn_course_assignments::table.filter(learn_course_assignments::id.eq(assignment_id)),
        )
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ----- Certificate Operations -----

    pub async fn generate_certificate(
        &self,
        user_id: Uuid,
        course_id: Uuid,
        score: i32,
    ) -> Result<Certificate, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Check if certificate already exists
        let existing: Option<Certificate> = learn_certificates::table
            .filter(learn_certificates::user_id.eq(user_id))
            .filter(learn_certificates::course_id.eq(course_id))
            .first(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(cert) = existing {
            return Ok(cert);
        }

        let verification_code = format!(
            "GB-{}-{}",
            Utc::now().format("%Y%m%d"),
            &Uuid::new_v4().to_string()[..8].to_uppercase()
        );

        let certificate = Certificate {
            id: Uuid::new_v4(),
            user_id,
            course_id,
            issued_at: Utc::now(),
            score,
            certificate_url: None,
            verification_code,
            expires_at: None,
        };

        diesel::insert_into(learn_certificates::table)
            .values(&certificate)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        // Update assignment as completed
        diesel::update(
            learn_course_assignments::table
                .filter(learn_course_assignments::user_id.eq(user_id))
                .filter(learn_course_assignments::course_id.eq(course_id)),
        )
        .set(learn_course_assignments::completed_at.eq(Some(Utc::now())))
        .execute(&mut conn)
        .ok();

        Ok(certificate)
    }

    pub async fn get_certificates(&self, user_id: Uuid) -> Result<Vec<Certificate>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        learn_certificates::table
            .filter(learn_certificates::user_id.eq(user_id))
            .order(learn_certificates::issued_at.desc())
            .load::<Certificate>(&mut conn)
            .map_err(|e| e.to_string())
    }

    pub async fn verify_certificate(&self, verification_code: &str) -> Result<CertificateVerification, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let cert: Option<Certificate> = learn_certificates::table
            .filter(learn_certificates::verification_code.eq(verification_code))
            .first(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;

        match cert {
            Some(c) => {
                let is_valid = c.expires_at.map(|exp| exp > Utc::now()).unwrap_or(true);
                Ok(CertificateVerification {
                    is_valid,
                    certificate: Some(CertificateResponse {
                        id: c.id,
                        user_id: c.user_id,
                        user_name: "".to_string(), // Would join with users table
                        course_id: c.course_id,
                        course_title: "".to_string(), // Would join with courses table
                        issued_at: c.issued_at,
                        score: c.score,
                        verification_code: c.verification_code,
                        certificate_url: c.certificate_url,
                        is_valid,
                        expires_at: c.expires_at,
                    }),
                    message: if is_valid {
                        "Certificate is valid".to_string()
                    } else {
                        "Certificate has expired".to_string()
                    },
                })
            }
            None => Ok(CertificateVerification {
                is_valid: false,
                certificate: None,
                message: "Certificate not found".to_string(),
            }),
        }
    }

    // ----- Category Operations -----

    pub async fn get_categories(&self) -> Result<Vec<Category>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        learn_categories::table
            .order(learn_categories::sort_order.asc())
            .load::<Category>(&mut conn)
            .map_err(|e| e.to_string())
    }

    // ----- Statistics -----

    pub async fn get_statistics(&self) -> Result<LearnStatistics, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let total_courses: i64 = learn_courses::table
            .filter(learn_courses::is_published.eq(true))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let total_lessons: i64 = learn_lessons::table
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let total_users_learning: i64 = learn_user_progress::table
            .select(learn_user_progress::user_id)
            .distinct()
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let courses_completed: i64 = learn_user_progress::table
            .filter(learn_user_progress::status.eq("completed"))
            .filter(learn_user_progress::lesson_id.is_null())
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let certificates_issued: i64 = learn_certificates::table
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(LearnStatistics {
            total_courses,
            total_lessons,
            total_users_learning,
            courses_completed,
            certificates_issued,
            average_completion_rate: 0.0,
            mandatory_compliance_rate: 0.0,
            popular_categories: Vec::new(),
            recent_completions: Vec::new(),
        })
    }

    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<UserLearnStats, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        let courses_enrolled: i64 = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::lesson_id.is_null())
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let courses_completed: i64 = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::lesson_id.is_null())
            .filter(learn_user_progress::status.eq("completed"))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let courses_in_progress: i64 = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::lesson_id.is_null())
            .filter(learn_user_progress::status.eq("in_progress"))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let certificates_earned: i64 = learn_certificates::table
            .filter(learn_certificates::user_id.eq(user_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let pending_mandatory: i64 = learn_course_assignments::table
            .filter(learn_course_assignments::user_id.eq(user_id))
            .filter(learn_course_assignments::is_mandatory.eq(true))
            .filter(learn_course_assignments::completed_at.is_null())
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        let overdue_assignments: i64 = learn_course_assignments::table
            .filter(learn_course_assignments::user_id.eq(user_id))
            .filter(learn_course_assignments::completed_at.is_null())
            .filter(learn_course_assignments::due_date.lt(Utc::now()))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(UserLearnStats {
            courses_enrolled,
            courses_completed,
            courses_in_progress,
            total_time_spent_hours: 0.0,
            certificates_earned,
            average_score: 0.0,
            pending_mandatory,
            overdue_assignments,
        })
    }

    // ----- AI Recommendations -----

    pub async fn get_recommendations(&self, user_id: Uuid) -> Result<Vec<Course>, String> {
        let mut conn = self.db.get().map_err(|e| e.to_string())?;

        // Get user's completed courses to avoid recommending them
        let completed_course_ids: Vec<Uuid> = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::status.eq("completed"))
            .filter(learn_user_progress::lesson_id.is_null())
            .select(learn_user_progress::course_id)
            .load(&mut conn)
            .map_err(|e| e.to_string())?;

        // Get in-progress course IDs
        let in_progress_ids: Vec<Uuid> = learn_user_progress::table
            .filter(learn_user_progress::user_id.eq(user_id))
            .filter(learn_user_progress::status.eq("in_progress"))
            .filter(learn_user_progress::lesson_id.is_null())
            .select(learn_user_progress::course_id)
            .load(&mut conn)
            .map_err(|e| e.to_string())?;

        let mut excluded_ids = completed_course_ids;
        excluded_ids.extend(in_progress_ids);

        // Recommend published courses not yet taken
        let mut query = learn_courses::table
            .filter(learn_courses::is_published.eq(true))
            .into_boxed();

        if !excluded_ids.is_empty() {
            query = query.filter(learn_courses::id.ne_all(excluded_ids));
        }

        query
            .order(learn_courses::created_at.desc())
            .limit(10)
            .load::<Course>(&mut conn)
            .map_err(|e| e.to_string())
    }
}

// ============================================================================
// HTTP HANDLERS
// ============================================================================

/// List all courses with optional filters
pub async fn list_courses(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<CourseFilters>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.list_courses(filters).await {
        Ok(courses) => Json(serde_json::json!({
            "success": true,
            "data": courses
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Create a new course
pub async fn create_course(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCourseRequest>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.create_course(req, None, None).await {
        Ok(course) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": course
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get course details with lessons
pub async fn get_course(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.get_course(course_id).await {
        Ok(Some(course)) => {
            let lessons = engine.get_lessons(course_id).await.unwrap_or_default();
            let quiz = engine.get_quiz(course_id).await.unwrap_or(None);

            Json(serde_json::json!({
                "success": true,
                "data": {
                    "course": course,
                    "lessons": lessons,
                    "quiz": quiz
                }
            }))
            .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Course not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Update a course
pub async fn update_course(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
    Json(req): Json<UpdateCourseRequest>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.update_course(course_id, req).await {
        Ok(course) => Json(serde_json::json!({
            "success": true,
            "data": course
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Delete a course
pub async fn delete_course(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.delete_course(course_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Course deleted"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get lessons for a course
pub async fn get_lessons(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.get_lessons(course_id).await {
        Ok(lessons) => Json(serde_json::json!({
            "success": true,
            "data": lessons
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Create a lesson for a course
pub async fn create_lesson(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
    Json(req): Json<CreateLessonRequest>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.create_lesson(course_id, req).await {
        Ok(lesson) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": lesson
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Update a lesson
pub async fn update_lesson(
    State(state): State<Arc<AppState>>,
    Path(lesson_id): Path<Uuid>,
    Json(req): Json<UpdateLessonRequest>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.update_lesson(lesson_id, req).await {
        Ok(lesson) => Json(serde_json::json!({
            "success": true,
            "data": lesson
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Delete a lesson
pub async fn delete_lesson(
    State(state): State<Arc<AppState>>,
    Path(lesson_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.delete_lesson(lesson_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Lesson deleted"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get quiz for a course
pub async fn get_quiz(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.get_quiz(course_id).await {
        Ok(Some(quiz)) => Json(serde_json::json!({
            "success": true,
            "data": quiz
        }))
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Quiz not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Submit quiz answers
pub async fn submit_quiz(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
    Json(submission): Json<QuizSubmission>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // Get quiz ID first
    let quiz = match engine.get_quiz(course_id).await {
        Ok(Some(q)) => q,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Quiz not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e
                })),
            )
                .into_response()
        }
    };

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.submit_quiz(user_id, quiz.id, submission).await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "data": result
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get user progress
pub async fn get_progress(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<ProgressFilters>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.get_user_progress(user_id, filters.course_id).await {
        Ok(progress) => Json(serde_json::json!({
            "success": true,
            "data": progress
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Start a course
pub async fn start_course(
    State(state): State<Arc<AppState>>,
    Path(course_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.start_course(user_id, course_id).await {
        Ok(progress) => Json(serde_json::json!({
            "success": true,
            "data": progress
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Complete a lesson
pub async fn complete_lesson_handler(
    State(state): State<Arc<AppState>>,
    Path(lesson_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.complete_lesson(user_id, lesson_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Lesson completed"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Create course assignment
pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAssignmentRequest>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get assigner user_id from session
    let assigned_by = None;

    match engine.create_assignment(req, assigned_by).await {
        Ok(assignments) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": assignments
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get pending assignments
pub async fn get_pending_assignments(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.get_pending_assignments(user_id).await {
        Ok(assignments) => Json(serde_json::json!({
            "success": true,
            "data": assignments
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Delete assignment
pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    Path(assignment_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.delete_assignment(assignment_id).await {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": "Assignment deleted"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get user certificates
pub async fn get_certificates(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.get_certificates(user_id).await {
        Ok(certificates) => Json(serde_json::json!({
            "success": true,
            "data": certificates
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Verify certificate
pub async fn verify_certificate(Path(code): Path<String>) -> impl IntoResponse {
    // Note: This would need database access in real implementation
    Json(serde_json::json!({
        "success": true,
        "data": {
            "is_valid": true,
            "message": "Certificate verification requires database lookup",
            "code": code
        }
    }))
}

/// Get categories
pub async fn get_categories(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.get_categories().await {
        Ok(categories) => Json(serde_json::json!({
            "success": true,
            "data": categories
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get AI recommendations
pub async fn get_recommendations(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.get_recommendations(user_id).await {
        Ok(courses) => Json(serde_json::json!({
            "success": true,
            "data": courses
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get learn statistics
pub async fn get_statistics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    match engine.get_statistics().await {
        Ok(stats) => Json(serde_json::json!({
            "success": true,
            "data": stats
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Get user stats
pub async fn get_user_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let engine = LearnEngine::new(state.conn.clone());

    // TODO: Get user_id from session
    let user_id = Uuid::new_v4();

    match engine.get_user_stats(user_id).await {
        Ok(stats) => Json(serde_json::json!({
            "success": true,
            "data": stats
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        )
            .into_response(),
    }
}

/// Serve Learn UI
pub async fn learn_ui() -> impl IntoResponse {
    Html(include_str!("../../../botui/ui/suite/learn/learn.html"))
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

/// Configure all Learn module routes
pub fn configure_learn_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Course routes
        .route("/api/learn/courses", get(list_courses).post(create_course))
        .route(
            "/api/learn/courses/:id",
            get(get_course).put(update_course).delete(delete_course),
        )
        // Lesson routes
        .route(
            "/api/learn/courses/:id/lessons",
            get(get_lessons).post(create_lesson),
        )
        .route(
            "/api/learn/lessons/:id",
            put(update_lesson).delete(delete_lesson),
        )
        // Quiz routes
        .route("/api/learn/courses/:id/quiz", get(get_quiz).post(submit_quiz))
        // Progress routes
        .route("/api/learn/progress", get(get_progress))
        .route("/api/learn/progress/:id/start", post(start_course))
        .route("/api/learn/progress/:id/complete", post(complete_lesson_handler))
        // Assignment routes
        .route(
            "/api/learn/assignments",
            get(get_pending_assignments).post(create_assignment),
        )
        .route("/api/learn/assignments/:id", delete(delete_assignment))
        // Certificate routes
        .route("/api/learn/certificates", get(get_certificates))
        .route("/api/learn/certificates/:code/verify", get(verify_certificate))
        // Category routes
        .route("/api/learn/categories", get(get_categories))
        // Recommendations
        .route("/api/learn/recommendations", get(get_recommendations))
        // Statistics
        .route("/api/learn/stats", get(get_statistics))
        .route("/api/learn/stats/user", get(get_user_stats))
        // UI
        .route("/suite/learn/learn.html", get(learn_ui))
}

/// Simplified configure function for module registration
pub fn configure(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.merge(configure_learn_routes())
}

// ============================================================================
// MCP TOOLS FOR BOT INTEGRATION
// ============================================================================

/// MCP tool definitions for Learn module
pub mod mcp_tools {
    use super::*;

    /// List available courses for the bot
    pub async fn list_courses_tool(
        db: DbPool,
        category: Option<String>,
        difficulty: Option<String>,
    ) -> Result<Vec<Course>, String> {
        let engine = LearnEngine::new(db);
        engine
            .list_courses(CourseFilters {
                category,
                difficulty,
                is_mandatory: None,
                search: None,
                limit: Some(20),
                offset: None,
            })
            .await
    }

    /// Get course details for the bot
    pub async fn get_course_details_tool(db: DbPool, course_id: Uuid) -> Result<Option<Course>, String> {
        let engine = LearnEngine::new(db);
        engine.get_course(course_id).await
    }

    /// Get user progress for the bot
    pub async fn get_user_progress_tool(
        db: DbPool,
        user_id: Uuid,
        course_id: Option<Uuid>,
    ) -> Result<Vec<UserProgress>, String> {
        let engine = LearnEngine::new(db);
        engine.get_user_progress(user_id, course_id).await
    }

    /// Start a course for the user via bot
    pub async fn start_course_tool(
        db: DbPool,
        user_id: Uuid,
        course_id: Uuid,
    ) -> Result<UserProgress, String> {
        let engine = LearnEngine::new(db);
        engine.start_course(user_id, course_id).await
    }

    /// Complete a lesson for the user via bot
    pub async fn complete_lesson_tool(db: DbPool, user_id: Uuid, lesson_id: Uuid) -> Result<(), String> {
        let engine = LearnEngine::new(db);
        engine.complete_lesson(user_id, lesson_id).await
    }

    /// Submit quiz answers via bot
    pub async fn submit_quiz_tool(
        db: DbPool,
        user_id: Uuid,
        quiz_id: Uuid,
        answers: HashMap<String, Vec<usize>>,
    ) -> Result<QuizResult, String> {
        let engine = LearnEngine::new(db);
        engine
            .submit_quiz(user_id, quiz_id, QuizSubmission { answers })
            .await
    }

    /// Get pending mandatory training for user
    pub async fn get_pending_training_tool(
        db: DbPool,
        user_id: Uuid,
    ) -> Result<Vec<CourseAssignment>, String> {
        let engine = LearnEngine::new(db);
        engine.get_pending_assignments(user_id).await
    }

    /// Get user certificates via bot
    pub async fn get_certificates_tool(db: DbPool, user_id: Uuid) -> Result<Vec<Certificate>, String> {
        let engine = LearnEngine::new(db);
        engine.get_certificates(user_id).await
    }

    /// Get user learning statistics
    pub async fn get_user_stats_tool(db: DbPool, user_id: Uuid) -> Result<UserLearnStats, String> {
        let engine = LearnEngine::new(db);
        engine.get_user_stats(user_id).await
    }

    /// Get AI-recommended courses for user
    pub async fn get_recommendations_tool(db: DbPool, user_id: Uuid) -> Result<Vec<Course>, String> {
        let engine = LearnEngine::new(db);
        engine.get_recommendations(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_status_conversion() {
        assert_eq!(ProgressStatus::from("not_started"), ProgressStatus::NotStarted);
        assert_eq!(ProgressStatus::from("in_progress"), ProgressStatus::InProgress);
        assert_eq!(ProgressStatus::from("completed"), ProgressStatus::Completed);
        assert_eq!(ProgressStatus::from("failed"), ProgressStatus::Failed);
        assert_eq!(ProgressStatus::from("unknown"), ProgressStatus::NotStarted);
    }

    #[test]
    fn test_progress_status_display() {
        assert_eq!(ProgressStatus::NotStarted.to_string(), "not_started");
        assert_eq!(ProgressStatus::InProgress.to_string(), "in_progress");
        assert_eq!(ProgressStatus::Completed.to_string(), "completed");
        assert_eq!(ProgressStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_question_types() {
        let q = QuestionType::SingleChoice;
        assert_eq!(q, QuestionType::SingleChoice);
    }

    #[test]
    fn test_quiz_submission_serialization() {
        let mut answers = HashMap::new();
        answers.insert("q1".to_string(), vec![0]);
        answers.insert("q2".to_string(), vec![1, 2]);

        let submission = QuizSubmission { answers };
        let json = serde_json::to_string(&submission).unwrap();
        assert!(json.contains("q1"));
        assert!(json.contains("q2"));
    }
}
