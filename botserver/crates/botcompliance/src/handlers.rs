use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::{
    compliance_audit_log, compliance_checks, compliance_issues, compliance_risk_assessments,
    compliance_training_records,
};
use crate::storage::{
    db_audit_to_entry, db_check_to_result, db_issue_to_result, db_risk_assessment_to_json,
    DbAuditLog, DbComplianceCheck, DbComplianceIssue, DbRiskAssessment, DbTrainingRecord,
};
use crate::types::{
    AuditLogEntry, ComplianceCheckResult, ComplianceFramework, ComplianceIssueResult,
    ComplianceReport, CreateAuditLogRequest, CreateIssueRequest, CreateTrainingRequest,
    ListAuditLogsQuery, ListChecksQuery, ListIssuesQuery, RunCheckRequest, TrainingRecord,
    TrainingType, UpdateIssueRequest,
};
use crate::ComplianceError;

const BRANCH_ID_PLACEHOLDER: Uuid = Uuid::nil();

pub async fn handle_list_checks(
    State(pool): State<Arc<crate::DbPool>>,
    Query(query): Query<ListChecksQuery>,
) -> Result<Json<Vec<ComplianceCheckResult>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let mut db_query = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch_id))
            .into_boxed();

        if let Some(check_type) = query.check_type {
            db_query = db_query.filter(compliance_checks::check_type.eq(check_type));
        }

        if let Some(status) = query.status {
            db_query = db_query.filter(compliance_checks::status.eq(Some(status)));
        }

        let db_checks: Vec<DbComplianceCheck> = db_query
            .order(compliance_checks::updated_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for check in db_checks {
            results.push(db_check_to_result(check, vec![]));
        }

        Ok::<_, ComplianceError>(results)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_run_check(
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<RunCheckRequest>,
) -> Result<Json<Vec<ComplianceCheckResult>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let controls = match req.framework {
            ComplianceFramework::Gdpr => vec![
                ("gdpr_7.2", "Data Retention Policy", 95.0),
                ("gdpr_5.1.f", "Data Protection Measures", 100.0),
                ("gdpr_6.1", "Lawful Basis for Processing", 98.0),
            ],
            ComplianceFramework::Soc2 => vec![("cc6.1", "Logical and Physical Access Controls", 94.0)],
            ComplianceFramework::Iso27001 => vec![("a.8.1", "Inventory of Assets", 90.0)],
            ComplianceFramework::Hipaa => vec![("164.312", "Technical Safeguards", 85.0)],
            ComplianceFramework::PciDss => vec![("req_3", "Protect Stored Cardholder Data", 88.0)],
        };

        let mut results = Vec::new();
        for (control_id, control_name, score) in controls {
            let db_check = DbComplianceCheck {
                id: Uuid::new_v4(),
                org_id: None,
                bot_id: None,
                branch_id,
                check_type: req.framework.to_string(),
                status: Some("compliant".to_string()),
                target_type: Some("control".to_string()),
                target_id: None,
                result: Some(serde_json::json!({
                    "control_id": control_id,
                    "control_name": control_name,
                    "score": score,
                })),
                checked_at: Some(now),
                checked_by: None,
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
    State(pool): State<Arc<crate::DbPool>>,
    Path(check_id): Path<Uuid>,
) -> Result<Json<Option<ComplianceCheckResult>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let db_check: Option<DbComplianceCheck> = compliance_checks::table
            .find(check_id)
            .first(&mut conn)
            .optional()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        match db_check {
            Some(check) => {
                Ok::<_, ComplianceError>(Some(db_check_to_result(check, vec![])))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_list_issues(
    State(pool): State<Arc<crate::DbPool>>,
    Query(query): Query<ListIssuesQuery>,
) -> Result<Json<Vec<ComplianceIssueResult>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let mut db_query = compliance_issues::table
            .filter(compliance_issues::branch_id.eq(branch_id))
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

        let issues: Vec<ComplianceIssueResult> =
            db_issues.into_iter().map(db_issue_to_result).collect();
        Ok::<_, ComplianceError>(issues)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_issue(
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<CreateIssueRequest>,
) -> Result<Json<ComplianceIssueResult>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let db_issue = DbComplianceIssue {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            check_id: req.check_id,
            branch_id,
            title: req.title,
            severity: req.severity.to_string(),
            status: "open".to_string(),
            description: req.description,
            assigned_to: req.assigned_to,
            remediation: req.remediation,
            due_date: req.due_date,
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
    State(pool): State<Arc<crate::DbPool>>,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<UpdateIssueRequest>,
) -> Result<Json<ComplianceIssueResult>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
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
            if status == "resolved" {
                db_issue.resolved_at = Some(now);
            }
            db_issue.status = status;
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
    State(pool): State<Arc<crate::DbPool>>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let mut db_query = compliance_audit_log::table
            .filter(compliance_audit_log::branch_id.eq(branch_id))
            .into_boxed();

        if let Some(action) = query.action {
            db_query = db_query.filter(compliance_audit_log::action.eq(action));
        }

        if let Some(actor_id) = query.actor_id {
            db_query = db_query.filter(compliance_audit_log::user_id.eq(actor_id));
        }

        if let Some(target_type) = query.target_type {
            db_query = db_query.filter(compliance_audit_log::resource_type.eq(target_type));
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
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<CreateAuditLogRequest>,
) -> Result<Json<AuditLogEntry>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let metadata = serde_json::to_value(&req.metadata.unwrap_or_default()).unwrap_or(serde_json::json!({}));

        let db_log = DbAuditLog {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            branch_id,
            event_type: req.event_type.to_string(),
            user_id: req.actor_id.or(req.user_id),
            resource_type: req.target_type,
            resource_id: req.resource_id,
            action: req.action,
            result: req.result.to_string(),
            ip_address: req.ip_address,
            user_agent: req.user_agent,
            metadata,
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
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<CreateTrainingRequest>,
) -> Result<Json<TrainingRecord>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let db_training = DbTrainingRecord {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            branch_id,
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
            user_id: req.user_id,
            training_type: req.training_type,
            training_name: req.training_name,
            provider: req.provider,
            score: req.score,
            passed: req.passed,
            completion_date: now,
            valid_until: req.valid_until,
            certificate_url: req.certificate_url,
        })
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_list_training(
    State(pool): State<Arc<crate::DbPool>>,
) -> Result<Json<Vec<TrainingRecord>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let db_records: Vec<DbTrainingRecord> = compliance_training_records::table
            .filter(compliance_training_records::branch_id.eq(branch_id))
            .order(compliance_training_records::completion_date.desc())
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let records: Vec<TrainingRecord> = db_records
            .into_iter()
            .map(|r| TrainingRecord {
                id: r.id,
                user_id: r.user_id,
                training_type: r.training_type.parse().unwrap_or(TrainingType::SecurityAwareness),
                training_name: r.training_name,
                provider: r.provider,
                score: r.score,
                passed: r.passed,
                completion_date: r.completion_date,
                valid_until: r.valid_until,
                certificate_url: r.certificate_url,
            })
            .collect();

        Ok::<_, ComplianceError>(records)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_list_risks(
    State(pool): State<Arc<crate::DbPool>>,
) -> Result<Json<Vec<serde_json::Value>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let items: Vec<DbRiskAssessment> = compliance_risk_assessments::table
            .filter(compliance_risk_assessments::branch_id.eq(branch_id))
            .order(compliance_risk_assessments::created_at.desc())
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let json_items: Vec<serde_json::Value> =
            items.into_iter().map(db_risk_assessment_to_json).collect();

        Ok::<_, ComplianceError>(json_items)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_run_check_by_id(
    State(pool): State<Arc<crate::DbPool>>,
    Path(check_id): Path<Uuid>,
) -> Result<Json<ComplianceCheckResult>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut db_check: DbComplianceCheck = compliance_checks::table
            .find(check_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Check not found".to_string()))?;

        db_check.checked_at = Some(now);
        db_check.status = Some("in_progress".to_string());

        diesel::update(compliance_checks::table.find(check_id))
            .set(&db_check)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        Ok::<_, ComplianceError>(db_check_to_result(db_check, vec![]))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_report(
    State(pool): State<Arc<crate::DbPool>>,
    Query(query): Query<ListChecksQuery>,
) -> Result<Json<ComplianceReport>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let mut db_query = compliance_checks::table
            .filter(compliance_checks::branch_id.eq(branch_id))
            .into_boxed();

        if let Some(check_type) = query.check_type {
            db_query = db_query.filter(compliance_checks::check_type.eq(check_type));
        }

        let db_checks: Vec<DbComplianceCheck> = db_query
            .order(compliance_checks::updated_at.desc())
            .limit(100)
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let mut results = Vec::new();
        let mut total_score = 0.0;
        let mut compliant_count = 0;

        for check in db_checks {
            let score: f64 = check
                .result
                .as_ref()
                .and_then(|r| r.get("score").and_then(|v| v.as_f64()))
                .unwrap_or(0.0);
            total_score += score;

            if check.status.as_deref() == Some("compliant") {
                compliant_count += 1;
            }

            results.push(db_check_to_result(check, vec![]));
        }

        let total_controls = results.len();
        let overall_score = if total_controls > 0 {
            total_score / total_controls as f64
        } else {
            0.0
        };

        let all_issues: Vec<DbComplianceIssue> = compliance_issues::table
            .filter(compliance_issues::branch_id.eq(branch_id))
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

pub async fn handle_upload_evidence(
    State(_pool): State<Arc<crate::DbPool>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let mut file_name = String::new();
    let mut category = String::new();
    let mut file_size = 0usize;

    while let Some(field) = multipart.next_field().await.map_err(|e| ComplianceError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|e| ComplianceError::Internal(e.to_string()))?;
                file_size = data.len();
            }
            "category" => {
                category = field.text().await.map_err(|e| ComplianceError::Internal(e.to_string()))?;
            }
            _ => {}
        }
    }

    let evidence_id = Uuid::new_v4();

    Ok(Json(serde_json::json!({
        "success": true,
        "evidence_id": evidence_id,
        "file_name": file_name,
        "category": category,
        "file_size": file_size,
        "uploaded_at": Utc::now().to_rfc3339()
    })))
}
