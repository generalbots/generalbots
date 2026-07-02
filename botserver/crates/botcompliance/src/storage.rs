use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::schema::{
    compliance_access_reviews, compliance_audit_log, compliance_checks, compliance_evidence,
    compliance_issues, compliance_risk_assessments, compliance_training_records,
};

use crate::types::{
    ActionResult, AuditEventType, AuditLogEntry, ComplianceCheckResult, ComplianceFramework,
    ComplianceIssueResult, ComplianceStatus, Severity,
};

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_checks)]
pub struct DbComplianceCheck {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub check_type: String,
    pub status: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub result: Option<serde_json::Value>,
    pub checked_at: Option<DateTime<Utc>>,
    pub checked_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_issues)]
pub struct DbComplianceIssue {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub title: String,
    pub severity: String,
    pub status: Option<String>,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub remediation: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = compliance_audit_log)]
pub struct DbAuditLog {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = compliance_evidence)]
pub struct DbEvidence {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub check_id: Option<Uuid>,
    pub file_path: String,
    pub description: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_risk_assessments)]
pub struct DbRiskAssessment {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub title: String,
    pub risk_level: String,
    pub probability: Option<i32>,
    pub impact: Option<i32>,
    pub mitigation: Option<String>,
    pub status: Option<String>,
    pub assessed_by: Option<Uuid>,
    pub assessed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = compliance_training_records)]
pub struct DbTrainingRecord {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub person_id: Option<Uuid>,
    pub course_name: String,
    pub completed: Option<bool>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = compliance_access_reviews)]
pub struct DbAccessReview {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub reviewer_id: Option<Uuid>,
    pub reviewed_type: String,
    pub reviewed_id: Uuid,
    pub decision: Option<String>,
    pub comments: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn db_check_to_result(
    db: DbComplianceCheck,
    issues: Vec<ComplianceIssueResult>,
) -> ComplianceCheckResult {
    let framework: ComplianceFramework = db.check_type.parse().unwrap_or(ComplianceFramework::Gdpr);
    let status: ComplianceStatus = db
        .status
        .as_deref()
        .unwrap_or("in_progress")
        .parse()
        .unwrap_or(ComplianceStatus::InProgress);
    let score: f64 = db
        .result
        .as_ref()
        .and_then(|r| r.get("score").and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    let checked_at = db.checked_at.unwrap_or(db.created_at);

    ComplianceCheckResult {
        id: db.id,
        framework,
        control_id: String::new(),
        control_name: String::new(),
        status,
        score,
        checked_at,
        checked_by: db.checked_by,
        issues,
        evidence: Vec::new(),
        notes: None,
    }
}

pub fn db_issue_to_result(db: DbComplianceIssue) -> ComplianceIssueResult {
    let severity: Severity = db.severity.parse().unwrap_or(Severity::Medium);
    let status = db.status.unwrap_or_else(|| "open".to_string());

    ComplianceIssueResult {
        id: db.id,
        severity,
        title: db.title,
        description: db.description.unwrap_or_default(),
        remediation: db.remediation,
        due_date: db.due_date.map(|d| {
            d.and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc())
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
        }),
        assigned_to: db.assigned_to,
        status,
    }
}

pub fn db_audit_to_entry(db: DbAuditLog) -> AuditLogEntry {
    let event_type: AuditEventType = db.action.parse().unwrap_or(AuditEventType::Access);
    let metadata: HashMap<String, String> =
        serde_json::from_value(db.details.unwrap_or(serde_json::json!({}))).unwrap_or_default();

    AuditLogEntry {
        id: db.id,
        timestamp: db.created_at,
        event_type,
        user_id: db.actor_id,
        resource_type: db.target_type.unwrap_or_default(),
        resource_id: db
            .target_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        action: db.action,
        result: ActionResult::Success,
        ip_address: db.ip_address,
        user_agent: None,
        metadata,
    }
}

pub fn db_risk_assessment_to_json(db: DbRiskAssessment) -> serde_json::Value {
    serde_json::json!({
        "id": db.id,
        "title": db.title,
        "risk_level": db.risk_level,
        "probability": db.probability,
        "impact": db.impact,
        "mitigation": db.mitigation,
        "status": db.status,
        "assessed_by": db.assessed_by,
        "assessed_at": db.assessed_at,
    })
}
