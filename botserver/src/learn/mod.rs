use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botlearn::{
    Course, CreateCourseRequest, UpdateCourseRequest, CourseResponse, CourseDetailResponse,
    Lesson, CreateLessonRequest, UpdateLessonRequest, LessonResponse, AttachmentInfo,
    Quiz, CreateQuizRequest, QuizResponse, QuizQuestion, QuizOption, QuizSubmission, QuizResult,
    AnswerResult, QuestionType,
    UserProgress, UserProgressResponse, ProgressStatus,
    CourseAssignment, CreateAssignmentRequest, AssignmentResponse,
    Certificate, CertificateResponse, CertificateVerification,
    Category, CategoryResponse,
    CourseFilters, ProgressFilters,
    LearnStatistics, CategoryStats, RecentCompletion, UserLearnStats,
};

pub mod ui {
    pub fn configure_learn_ui_routes() -> axum::Router<()> {
        botlearn::configure_learn_ui_routes()
    }
}
