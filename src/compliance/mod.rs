use axum::{
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use std::sync::Arc;

use crate::shared::state::AppState;

pub mod access_review;
pub mod audit;
pub mod backup_verification;
pub mod code_scanner;
pub mod evidence_collection;
pub mod handlers;
pub mod incident_response;
pub mod policy_checker;
pub mod risk_assessment;
pub mod soc2;
pub mod sop_middleware;
pub mod storage;
pub mod training_tracker;
pub mod types;
pub mod ui;
pub mod vulnerability_scanner;

pub use code_scanner::{
    CodeIssue, CodeScanner, ComplianceReporter, ComplianceScanResult, IssueSeverity, IssueType,
    ScanStats,
};

pub use storage::{
    DbAccessReview, DbAuditLog, DbComplianceCheck, DbComplianceIssue, DbEvidence, DbRisk,
    DbRiskAssessment, DbTrainingRecord,
};

pub use types::{
    AccessReview, ActionResult, AuditEventType, AuditLogEntry, ComplianceCheckResult,
    ComplianceFramework, ComplianceIssueResult, ComplianceReport, ComplianceStatus,
    CreateAuditLogRequest, CreateIssueRequest, CreateTrainingRequest, ListAuditLogsQuery,
    ListChecksQuery, ListIssuesQuery, PermissionReview, ReviewAction, ReviewStatus, Risk,
    RiskAssessment, RiskCategory, RiskStatus, RunCheckRequest, Severity, TrainingRecord,
    TrainingType, TreatmentStrategy, UpdateIssueRequest,
};

#[derive(Debug, thiserror::Error)]
pub enum ComplianceError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ComplianceError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) | Self::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub fn configure_compliance_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/compliance/checks", get(handlers::handle_list_checks))
        .route("/api/compliance/checks", post(handlers::handle_run_check))
        .route(
            "/api/compliance/checks/:check_id",
            get(handlers::handle_get_check),
        )
        .route("/api/compliance/issues", get(handlers::handle_list_issues))
        .route("/api/compliance/issues", post(handlers::handle_create_issue))
        .route(
            "/api/compliance/issues/:issue_id",
            put(handlers::handle_update_issue),
        )
        .route("/api/compliance/audit", get(handlers::handle_list_audit_logs))
        .route(
            "/api/compliance/audit",
            post(handlers::handle_create_audit_log),
        )
        .route(
            "/api/compliance/training",
            post(handlers::handle_create_training),
        )
        .route("/api/compliance/report", get(handlers::handle_get_report))
}
