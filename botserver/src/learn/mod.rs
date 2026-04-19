//! Learn Module - Learning Management System (LMS)
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
//!
pub mod types;

use types::{
    Course, CreateCourseRequest, UpdateCourseRequest, CourseResponse, Lesson, CreateLessonRequest,
    UpdateLessonRequest, LessonResponse, AttachmentInfo,
    Quiz, CreateQuizRequest, QuizResponse, QuizQuestion, QuizOption,
    UserProgress, UserProgressResponse, ProgressStatus,
    CourseAssignment, CreateAssignmentRequest, AssignmentResponse,
    Certificate, CertificateResponse, CertificateVerification,
    Category, CategoryResponse,
};

pub mod ui;
