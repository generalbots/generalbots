use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::shared::schema::{
    compliance_access_reviews, compliance_audit_log, compliance_checks, compliance_evidence,
    compliance_issues, compliance_risk_assessments, compliance_risks, compliance_training_records,
};

use super::types::{
    ActionResult, AuditEventType, AuditLogEntry, ComplianceCheckResult, ComplianceFramework,
    ComplianceIssueResult, ComplianceStatus, Severity,
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

pub fn db_check_to_result(
    db: DbComplianceCheck,
    issues: Vec<ComplianceIssueResult>,
) -> ComplianceCheckResult {
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

pub fn db_issue_to_result(db: DbComplianceIssue) -> ComplianceIssueResult {
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

pub fn db_audit_to_entry(db: DbAuditLog) -> AuditLogEntry {
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
