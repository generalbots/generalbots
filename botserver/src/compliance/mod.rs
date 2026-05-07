use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botcompliance::{
    code_scanner::{
        CodeIssue, CodeScanner, ComplianceReporter, ComplianceScanResult, IssueSeverity, IssueType,
        ScanStats,
    },
    storage::{
        DbAccessReview, DbAuditLog, DbComplianceCheck, DbComplianceIssue, DbEvidence, DbRisk,
        DbRiskAssessment, DbTrainingRecord,
    },
    types::{
        AccessReview, ActionResult, AuditEventType, AuditLogEntry, ComplianceCheckResult,
        ComplianceFramework, ComplianceIssueResult, ComplianceReport, ComplianceStatus,
        CreateAuditLogRequest, CreateIssueRequest, CreateTrainingRequest, ListAuditLogsQuery,
        ListChecksQuery, ListIssuesQuery, PermissionReview, ReviewAction, ReviewStatus, Risk,
        RiskAssessment, RiskCategory, RiskStatus, RunCheckRequest, Severity, TrainingRecord,
        TrainingType, TreatmentStrategy, UpdateIssueRequest,
    },
    ComplianceError,
};

pub fn configure_compliance_routes(state: &Arc<AppState>) -> Router {
    let pool: Arc<botcompliance::DbPool> = Arc::new(state.conn.clone());
    botcompliance::configure_compliance_routes()
        .with_state(pool)
}

pub fn configure_compliance_ui_routes(state: &Arc<AppState>) -> Router {
    let pool: Arc<botcompliance::DbPool> = Arc::new(state.conn.clone());
    botcompliance::ui::configure_compliance_ui_routes()
        .with_state(pool)
}
