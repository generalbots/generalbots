pub mod schema;
pub mod types;
pub mod ui;

pub use types::{
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

pub fn configure_learn_ui_routes() -> axum::Router<()> {
    ui::configure_learn_ui_routes()
}
