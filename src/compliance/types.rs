use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
