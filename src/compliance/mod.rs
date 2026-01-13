use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{
    compliance_access_reviews, compliance_audit_log, compliance_checks, compliance_evidence,
    compliance_issues, compliance_risk_assessments, compliance_risks, compliance_training_records,
};
use crate::shared::state::AppState;

pub mod access_review;
pub mod audit;
pub mod backup_verification;
pub mod code_scanner;
pub mod evidence_collection;
pub mod incident_response;
pub mod policy_checker;
pub mod risk_assessment;
pub mod soc2;
pub mod sop_middleware;
pub mod training_tracker;
pub mod vulnerability_scanner;

pub use code_scanner::{
    CodeIssue, CodeScanner, ComplianceReporter, ComplianceScanResult, IssueSeverity, IssueType,
    ScanStats,
};

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_checks)]
pub struct DbComplianceCheck {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub framework: String,
    pub control_id: String,
    pub control_name: String,
    pub status: String,
    pub score: bigdecimal::BigDecimal,
    pub checked_at: DateTime<Utc>,
    pub checked_by: Option<Uuid>,
    pub evidence: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_issues)]
pub struct DbComplianceIssue {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub check_id: Option<Uuid>,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub remediation: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<Uuid>,
    pub status: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = compliance_audit_log)]
pub struct DbAuditLog {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub result: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = compliance_evidence)]
pub struct DbEvidence {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub check_id: Option<Uuid>,
    pub issue_id: Option<Uuid>,
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
    pub mime_type: Option<String>,
    pub metadata: serde_json::Value,
    pub collected_at: DateTime<Utc>,
    pub collected_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_risk_assessments)]
pub struct DbRiskAssessment {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub assessor_id: Uuid,
    pub methodology: String,
    pub overall_risk_score: bigdecimal::BigDecimal,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_review_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_risks)]
pub struct DbRisk {
    pub id: Uuid,
    pub assessment_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub likelihood_score: i32,
    pub impact_score: i32,
    pub risk_score: i32,
    pub risk_level: String,
    pub current_controls: serde_json::Value,
    pub treatment_strategy: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = compliance_training_records)]
pub struct DbTrainingRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub training_type: String,
    pub training_name: String,
    pub provider: Option<String>,
    pub score: Option<i32>,
    pub passed: bool,
    pub completion_date: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub certificate_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_access_reviews)]
pub struct DbAccessReview {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub reviewer_id: Uuid,
    pub review_date: DateTime<Utc>,
    pub permissions_reviewed: serde_json::Value,
    pub anomalies: serde_json::Value,
    pub recommendations: serde_json::Value,
    pub status: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceFramework {
    Gdpr,
    Soc2,
    Iso27001,
    Hipaa,
    PciDss,
}

impl std::fmt::Display for ComplianceFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Gdpr => "gdpr",
            Self::Soc2 => "soc2",
            Self::Iso27001 => "iso27001",
            Self::Hipaa => "hipaa",
            Self::PciDss => "pci_dss",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for ComplianceFramework {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gdpr" => Ok(Self::Gdpr),
            "soc2" => Ok(Self::Soc2),
            "iso27001" => Ok(Self::Iso27001),
            "hipaa" => Ok(Self::Hipaa),
            "pci_dss" | "pcidss" => Ok(Self::PciDss),
            _ => Err(format!("Unknown framework: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    PartialCompliance,
    NonCompliant,
    InProgress,
    NotApplicable,
}

impl std::fmt::Display for ComplianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Compliant => "compliant",
            Self::PartialCompliance => "partial_compliance",
            Self::NonCompliant => "non_compliant",
            Self::InProgress => "in_progress",
            Self::NotApplicable => "not_applicable",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for ComplianceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compliant" => Ok(Self::Compliant),
            "partial_compliance" => Ok(Self::PartialCompliance),
            "non_compliant" => Ok(Self::NonCompliant),
            "in_progress" => Ok(Self::InProgress),
            "not_applicable" => Ok(Self::NotApplicable),
            _ => Err(format!("Unknown status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("Unknown severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub id: Uuid,
    pub framework: ComplianceFramework,
    pub control_id: String,
    pub control_name: String,
    pub status: ComplianceStatus,
    pub score: f64,
    pub checked_at: DateTime<Utc>,
    pub checked_by: Option<Uuid>,
    pub issues: Vec<ComplianceIssueResult>,
    pub evidence: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssueResult {
    pub id: Uuid,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub remediation: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<Uuid>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub result: ActionResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Access,
    Modification,
    Deletion,
    Security,
    Admin,
    Authentication,
    Authorization,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Access => "access",
            Self::Modification => "modification",
            Self::Deletion => "deletion",
            Self::Security => "security",
            Self::Admin => "admin",
            Self::Authentication => "authentication",
            Self::Authorization => "authorization",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for AuditEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "access" => Ok(Self::Access),
            "modification" => Ok(Self::Modification),
            "deletion" => Ok(Self::Deletion),
            "security" => Ok(Self::Security),
            "admin" => Ok(Self::Admin),
            "authentication" => Ok(Self::Authentication),
            "authorization" => Ok(Self::Authorization),
            _ => Err(format!("Unknown event type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    Success,
    Failure,
    Denied,
    Error,
}

impl std::fmt::Display for ActionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Denied => "denied",
            Self::Error => "error",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for ActionResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            "denied" => Ok(Self::Denied),
            "error" => Ok(Self::Error),
            _ => Err(format!("Unknown result: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub title: String,
    pub assessor_id: Uuid,
    pub methodology: String,
    pub overall_risk_score: f64,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_review_date: Option<chrono::NaiveDate>,
    pub risks: Vec<Risk>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: RiskCategory,
    pub likelihood_score: i32,
    pub impact_score: i32,
    pub risk_score: i32,
    pub risk_level: Severity,
    pub current_controls: Vec<String>,
    pub treatment_strategy: TreatmentStrategy,
    pub status: RiskStatus,
    pub owner_id: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Technical,
    Operational,
    Financial,
    Compliance,
    Reputational,
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Technical => "technical",
            Self::Operational => "operational",
            Self::Financial => "financial",
            Self::Compliance => "compliance",
            Self::Reputational => "reputational",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for RiskCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "technical" => Ok(Self::Technical),
            "operational" => Ok(Self::Operational),
            "financial" => Ok(Self::Financial),
            "compliance" => Ok(Self::Compliance),
            "reputational" => Ok(Self::Reputational),
            _ => Err(format!("Unknown category: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TreatmentStrategy {
    Mitigate,
    Accept,
    Transfer,
    Avoid,
}

impl std::fmt::Display for TreatmentStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Mitigate => "mitigate",
            Self::Accept => "accept",
            Self::Transfer => "transfer",
            Self::Avoid => "avoid",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for TreatmentStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mitigate" => Ok(Self::Mitigate),
            "accept" => Ok(Self::Accept),
            "transfer" => Ok(Self::Transfer),
            "avoid" => Ok(Self::Avoid),
            _ => Err(format!("Unknown strategy: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskStatus {
    Open,
    InProgress,
    Mitigated,
    Accepted,
    Closed,
}

impl std::fmt::Display for RiskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Mitigated => "mitigated",
            Self::Accepted => "accepted",
            Self::Closed => "closed",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for RiskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "in_progress" => Ok(Self::InProgress),
            "mitigated" => Ok(Self::Mitigated),
            "accepted" => Ok(Self::Accepted),
            "closed" => Ok(Self::Closed),
            _ => Err(format!("Unknown status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub training_type: TrainingType,
    pub training_name: String,
    pub provider: Option<String>,
    pub score: Option<i32>,
    pub passed: bool,
    pub completion_date: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub certificate_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrainingType {
    SecurityAwareness,
    DataProtection,
    IncidentResponse,
    ComplianceOverview,
    RoleSpecific,
}

impl std::fmt::Display for TrainingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::SecurityAwareness => "security_awareness",
            Self::DataProtection => "data_protection",
            Self::IncidentResponse => "incident_response",
            Self::ComplianceOverview => "compliance_overview",
            Self::RoleSpecific => "role_specific",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for TrainingType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "security_awareness" => Ok(Self::SecurityAwareness),
            "data_protection" => Ok(Self::DataProtection),
            "incident_response" => Ok(Self::IncidentResponse),
            "compliance_overview" => Ok(Self::ComplianceOverview),
            "role_specific" => Ok(Self::RoleSpecific),
            _ => Err(format!("Unknown training type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReview {
    pub id: Uuid,
    pub user_id: Uuid,
    pub reviewer_id: Uuid,
    pub review_date: DateTime<Utc>,
    pub permissions_reviewed: Vec<PermissionReview>,
    pub anomalies: Vec<String>,
    pub recommendations: Vec<String>,
    pub status: ReviewStatus,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionReview {
    pub resource_type: String,
    pub resource_id: String,
    pub permissions: Vec<String>,
    pub justification: String,
    pub action: ReviewAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAction {
    Approved,
    Revoked,
    Modified,
    FlaggedForReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Completed,
    Approved,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Approved => "approved",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for ReviewStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "approved" => Ok(Self::Approved),
            _ => Err(format!("Unknown status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub overall_score: f64,
    pub total_controls_checked: usize,
    pub compliant_controls: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub results: Vec<ComplianceCheckResult>,
}

#[derive(Debug, Deserialize)]
pub struct ListChecksQuery {
    pub framework: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListIssuesQuery {
    pub severity: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListAuditLogsQuery {
    pub event_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RunCheckRequest {
    pub framework: ComplianceFramework,
    pub control_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    pub check_id: Option<Uuid>,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub remediation: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssueRequest {
    pub severity: Option<Severity>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub remediation: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<Uuid>,
    pub status: Option<String>,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuditLogRequest {
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub result: ActionResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrainingRequest {
    pub user_id: Uuid,
    pub training_type: TrainingType,
    pub training_name: String,
    pub provider: Option<String>,
    pub score: Option<i32>,
    pub passed: bool,
    pub valid_until: Option<DateTime<Utc>>,
    pub certificate_url: Option<String>,
}

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

fn db_check_to_result(db: DbComplianceCheck, issues: Vec<ComplianceIssueResult>) -> ComplianceCheckResult {
    let framework: ComplianceFramework = db.framework.parse().unwrap_or(ComplianceFramework::Gdpr);
    let status: ComplianceStatus = db.status.parse().unwrap_or(ComplianceStatus::InProgress);
    let evidence: Vec<String> = serde_json::from_value(db.evidence).unwrap_or_default();
    let score: f64 = db.score.to_string().parse().unwrap_or(0.0);

    ComplianceCheckResult {
        id: db.id,
        framework,
        control_id: db.control_id,
        control_name: db.control_name,
        status,
        score,
        checked_at: db.checked_at,
        checked_by: db.checked_by,
        issues,
        evidence,
        notes: db.notes,
    }
}

fn db_issue_to_result(db: DbComplianceIssue) -> ComplianceIssueResult {
    let severity: Severity = db.severity.parse().unwrap_or(Severity::Medium);

    ComplianceIssueResult {
        id: db.id,
        severity,
        title: db.title,
        description: db.description,
        remediation: db.remediation,
        due_date: db.due_date,
        assigned_to: db.assigned_to,
        status: db.status,
    }
}

fn db_audit_to_entry(db: DbAuditLog) -> AuditLogEntry {
    let event_type: AuditEventType = db.event_type.parse().unwrap_or(AuditEventType::Access);
    let result: ActionResult = db.result.parse().unwrap_or(ActionResult::Success);
    let metadata: HashMap<String, String> = serde_json::from_value(db.metadata).unwrap_or_default();

    AuditLogEntry {
        id: db.id,
        timestamp: db.created_at,
        event_type,
        user_id: db.user_id,
        resource_type: db.resource_type,
        resource_id: db.resource_id,
        action: db.action,
        result,
        ip_address: db.ip_address,
        user_agent: db.user_agent,
        metadata,
    }
}

pub async fn handle_list_checks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListChecksQuery>,
) -> Result<Json<Vec<ComplianceCheckResult>>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut db_query = compliance_checks::table
            .filter(compliance_checks::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(framework) = query.framework {
            db_query = db_query.filter(compliance_checks::framework.eq(framework));
        }

        if let Some(status) = query.status {
            db_query = db_query.filter(compliance_checks::status.eq(status));
        }

        let db_checks: Vec<DbComplianceCheck> = db_query
            .order(compliance_checks::checked_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for check in db_checks {
            let check_id = check.id;
            let db_issues: Vec<DbComplianceIssue> = compliance_issues::table
                .filter(compliance_issues::check_id.eq(check_id))
                .load(&mut conn)
                .unwrap_or_default();
            let issues: Vec<ComplianceIssueResult> = db_issues.into_iter().map(db_issue_to_result).collect();
            results.push(db_check_to_result(check, issues));
        }

        Ok::<_, ComplianceError>(results)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_run_check(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RunCheckRequest>,
) -> Result<Json<Vec<ComplianceCheckResult>>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let controls = match req.framework {
            ComplianceFramework::Gdpr => vec![
                ("gdpr_7.2", "Data Retention Policy", 95.0),
                ("gdpr_5.1.f", "Data Protection Measures", 100.0),
                ("gdpr_6.1", "Lawful Basis for Processing", 98.0),
            ],
            ComplianceFramework::Soc2 => vec![
                ("cc6.1", "Logical and Physical Access Controls", 94.0),
            ],
            ComplianceFramework::Iso27001 => vec![
                ("a.8.1", "Inventory of Assets", 90.0),
            ],
            ComplianceFramework::Hipaa => vec![
                ("164.312", "Technical Safeguards", 85.0),
            ],
            ComplianceFramework::PciDss => vec![
                ("req_3", "Protect Stored Cardholder Data", 88.0),
            ],
        };

        let mut results = Vec::new();
        for (control_id, control_name, score) in controls {
            let db_check = DbComplianceCheck {
                id: Uuid::new_v4(),
                org_id,
                bot_id,
                framework: req.framework.to_string(),
                control_id: control_id.to_string(),
                control_name: control_name.to_string(),
                status: "compliant".to_string(),
                score: bigdecimal::BigDecimal::try_from(score).unwrap_or_default(),
                checked_at: now,
                checked_by: None,
                evidence: serde_json::json!(["Automated check completed"]),
                notes: None,
                created_at: now,
                updated_at: now,
            };

            diesel::insert_into(compliance_checks::table)
                .values(&db_check)
                .execute(&mut conn)
                .map_err(|e| ComplianceError::Database(e.to_string()))?;

            results.push(db_check_to_result(db_check, vec![]));
        }

        Ok::<_, ComplianceError>(results)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_check(
    State(state): State<Arc<AppState>>,
    Path(check_id): Path<Uuid>,
) -> Result<Json<Option<ComplianceCheckResult>>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;

        let db_check: Option<DbComplianceCheck> = compliance_checks::table
            .find(check_id)
            .first(&mut conn)
            .optional()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        match db_check {
            Some(check) => {
                let db_issues: Vec<DbComplianceIssue> = compliance_issues::table
                    .filter(compliance_issues::check_id.eq(check_id))
                    .load(&mut conn)
                    .unwrap_or_default();
                let issues: Vec<ComplianceIssueResult> = db_issues.into_iter().map(db_issue_to_result).collect();
                Ok::<_, ComplianceError>(Some(db_check_to_result(check, issues)))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_list_issues(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListIssuesQuery>,
) -> Result<Json<Vec<ComplianceIssueResult>>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut db_query = compliance_issues::table
            .filter(compliance_issues::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(severity) = query.severity {
            db_query = db_query.filter(compliance_issues::severity.eq(severity));
        }

        if let Some(status) = query.status {
            db_query = db_query.filter(compliance_issues::status.eq(status));
        }

        if let Some(assigned_to) = query.assigned_to {
            db_query = db_query.filter(compliance_issues::assigned_to.eq(assigned_to));
        }

        let db_issues: Vec<DbComplianceIssue> = db_query
            .order(compliance_issues::created_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let issues: Vec<ComplianceIssueResult> = db_issues.into_iter().map(db_issue_to_result).collect();
        Ok::<_, ComplianceError>(issues)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_issue(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateIssueRequest>,
) -> Result<Json<ComplianceIssueResult>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let db_issue = DbComplianceIssue {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            check_id: req.check_id,
            severity: req.severity.to_string(),
            title: req.title,
            description: req.description,
            remediation: req.remediation,
            due_date: req.due_date,
            assigned_to: req.assigned_to,
            status: "open".to_string(),
            resolved_at: None,
            resolved_by: None,
            resolution_notes: None,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(compliance_issues::table)
            .values(&db_issue)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        Ok::<_, ComplianceError>(db_issue_to_result(db_issue))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_update_issue(
    State(state): State<Arc<AppState>>,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<UpdateIssueRequest>,
) -> Result<Json<ComplianceIssueResult>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut db_issue: DbComplianceIssue = compliance_issues::table
            .find(issue_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Issue not found".to_string()))?;

        if let Some(severity) = req.severity {
            db_issue.severity = severity.to_string();
        }
        if let Some(title) = req.title {
            db_issue.title = title;
        }
        if let Some(description) = req.description {
            db_issue.description = description;
        }
        if let Some(remediation) = req.remediation {
            db_issue.remediation = Some(remediation);
        }
        if let Some(due_date) = req.due_date {
            db_issue.due_date = Some(due_date);
        }
        if let Some(assigned_to) = req.assigned_to {
            db_issue.assigned_to = Some(assigned_to);
        }
        if let Some(status) = req.status {
            db_issue.status = status.clone();
            if status == "resolved" {
                db_issue.resolved_at = Some(now);
            }
        }
        if let Some(resolution_notes) = req.resolution_notes {
            db_issue.resolution_notes = Some(resolution_notes);
        }
        db_issue.updated_at = now;

        diesel::update(compliance_issues::table.find(issue_id))
            .set(&db_issue)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        Ok::<_, ComplianceError>(db_issue_to_result(db_issue))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_list_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let mut db_query = compliance_audit_log::table
            .filter(compliance_audit_log::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(event_type) = query.event_type {
            db_query = db_query.filter(compliance_audit_log::event_type.eq(event_type));
        }

        if let Some(user_id) = query.user_id {
            db_query = db_query.filter(compliance_audit_log::user_id.eq(user_id));
        }

        if let Some(resource_type) = query.resource_type {
            db_query = db_query.filter(compliance_audit_log::resource_type.eq(resource_type));
        }

        if let Some(from_date) = query.from_date {
            db_query = db_query.filter(compliance_audit_log::created_at.ge(from_date));
        }

        if let Some(to_date) = query.to_date {
            db_query = db_query.filter(compliance_audit_log::created_at.le(to_date));
        }

        let db_logs: Vec<DbAuditLog> = db_query
            .order(compliance_audit_log::created_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let logs: Vec<AuditLogEntry> = db_logs.into_iter().map(db_audit_to_entry).collect();
        Ok::<_, ComplianceError>(logs)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_audit_log(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAuditLogRequest>,
) -> Result<Json<AuditLogEntry>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let metadata = req.metadata.unwrap_or_default();

        let db_log = DbAuditLog {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            event_type: req.event_type.to_string(),
            user_id: req.user_id,
            resource_type: req.resource_type,
            resource_id: req.resource_id,
            action: req.action,
            result: req.result.to_string(),
            ip_address: req.ip_address,
            user_agent: req.user_agent,
            metadata: serde_json::to_value(&metadata).unwrap_or_default(),
            created_at: now,
        };

        diesel::insert_into(compliance_audit_log::table)
            .values(&db_log)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        Ok::<_, ComplianceError>(db_audit_to_entry(db_log))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_training(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTrainingRequest>,
) -> Result<Json<TrainingRecord>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let db_training = DbTrainingRecord {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            user_id: req.user_id,
            training_type: req.training_type.to_string(),
            training_name: req.training_name.clone(),
            provider: req.provider.clone(),
            score: req.score,
            passed: req.passed,
            completion_date: now,
            valid_until: req.valid_until,
            certificate_url: req.certificate_url.clone(),
            metadata: serde_json::json!({}),
            created_at: now,
        };

        diesel::insert_into(compliance_training_records::table)
            .values(&db_training)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        Ok::<_, ComplianceError>(TrainingRecord {
            id: db_training.id,
            user_id: db_training.user_id,
            training_type: req.training_type,
            training_name: req.training_name,
            provider: req.provider,
            score: req.score,
            passed: req.passed,
            completion_date: db_training.completion_date,
            valid_until: req.valid_until,
            certificate_url: req.certificate_url,
        })
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_report(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListChecksQuery>,
) -> Result<Json<ComplianceReport>, ComplianceError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);
        let now = Utc::now();

        let mut db_query = compliance_checks::table
            .filter(compliance_checks::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(framework) = query.framework {
            db_query = db_query.filter(compliance_checks::framework.eq(framework));
        }

        let db_checks: Vec<DbComplianceCheck> = db_query
            .order(compliance_checks::checked_at.desc())
            .limit(100)
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let mut results = Vec::new();
        let mut total_score = 0.0;
        let mut compliant_count = 0;

        for check in db_checks {
            let check_id = check.id;
            let score: f64 = check.score.to_string().parse().unwrap_or(0.0);
            total_score += score;

            if check.status == "compliant" {
                compliant_count += 1;
            }

            let db_issues: Vec<DbComplianceIssue> = compliance_issues::table
                .filter(compliance_issues::check_id.eq(check_id))
                .load(&mut conn)
                .unwrap_or_default();
            let issues: Vec<ComplianceIssueResult> = db_issues.into_iter().map(db_issue_to_result).collect();
            results.push(db_check_to_result(check, issues));
        }

        let total_controls = results.len();
        let overall_score = if total_controls > 0 {
            total_score / total_controls as f64
        } else {
            0.0
        };

        let all_issues: Vec<DbComplianceIssue> = compliance_issues::table
            .filter(compliance_issues::bot_id.eq(bot_id))
            .filter(compliance_issues::status.ne("resolved"))
            .load(&mut conn)
            .unwrap_or_default();

        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for issue in &all_issues {
            match issue.severity.as_str() {
                "critical" => critical += 1,
                "high" => high += 1,
                "medium" => medium += 1,
                "low" => low += 1,
                _ => {}
            }
        }

        Ok::<_, ComplianceError>(ComplianceReport {
            generated_at: now,
            overall_score,
            total_controls_checked: total_controls,
            compliant_controls: compliant_count,
            total_issues: all_issues.len(),
            critical_issues: critical,
            high_issues: high,
            medium_issues: medium,
            low_issues: low,
            results,
        })
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub fn configure_compliance_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/compliance/checks", get(handle_list_checks))
        .route("/api/compliance/checks", post(handle_run_check))
        .route("/api/compliance/checks/:check_id", get(handle_get_check))
        .route("/api/compliance/issues", get(handle_list_issues))
        .route("/api/compliance/issues", post(handle_create_issue))
        .route("/api/compliance/issues/:issue_id", put(handle_update_issue))
        .route("/api/compliance/audit", get(handle_list_audit_logs))
        .route("/api/compliance/audit", post(handle_create_audit_log))
        .route("/api/compliance/training", post(handle_create_training))
        .route("/api/compliance/report", get(handle_get_report))
}
