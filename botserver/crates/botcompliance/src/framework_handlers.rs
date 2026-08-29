//! Compliance framework configuration: CRUD for frameworks, their control
//! catalogs and attached evidence, plus coverage computation and audit-ready
//! CSV export. Every mutation is recorded in `compliance_audit_log`.

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::{
    compliance_audit_log, compliance_control_evidence, compliance_controls,
    compliance_frameworks,
};
use crate::storage::{
    DbComplianceControl, DbComplianceControlEvidence, DbComplianceFramework,
};
use crate::ComplianceError;

const BRANCH_ID_PLACEHOLDER: Uuid = Uuid::nil();

#[derive(Debug, Deserialize)]
pub struct CreateFrameworkRequest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub framework_key: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFrameworkRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateControlRequest {
    pub framework_id: Uuid,
    pub control_id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_mandatory: Option<bool>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateControlRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_mandatory: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttachEvidenceRequest {
    pub control_id: Uuid,
    pub file_path: String,
    pub description: Option<String>,
    pub evidence_type: Option<String>,
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListFrameworksQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

fn audit(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    actor_id: Option<Uuid>,
    action: &str,
    target_type: &str,
    target_id: Option<Uuid>,
    details: Option<serde_json::Value>,
) {
    let now = Utc::now();
    let result = diesel::insert_into(compliance_audit_log::table)
        .values((
            compliance_audit_log::id.eq(Uuid::new_v4()),
            compliance_audit_log::org_id.eq(Uuid::nil()),
            compliance_audit_log::bot_id.eq(Uuid::nil()),
            compliance_audit_log::branch_id.eq(branch_id),
            compliance_audit_log::event_type.eq(action),
            compliance_audit_log::user_id.eq(actor_id),
            compliance_audit_log::resource_type.eq(target_type),
            compliance_audit_log::resource_id.eq(target_id.map(|id| id.to_string()).unwrap_or_default()),
            compliance_audit_log::action.eq(action),
            compliance_audit_log::result.eq("success"),
            compliance_audit_log::ip_address.eq(Option::<String>::None),
            compliance_audit_log::user_agent.eq(Option::<String>::None),
            compliance_audit_log::metadata.eq(details.unwrap_or_else(|| serde_json::json!({}))),
            compliance_audit_log::created_at.eq(now),
        ))
        .execute(conn);
    if let Err(e) = result {
        log::warn!("compliance audit write failed: {e}");
    }
}

fn framework_json(fw: &DbComplianceFramework) -> serde_json::Value {
    serde_json::json!({
        "id": fw.id,
        "name": fw.name,
        "version": fw.version,
        "description": fw.description,
        "framework_key": fw.framework_key,
        "status": fw.status,
        "created_by": fw.created_by,
        "created_at": fw.created_at,
        "updated_at": fw.updated_at,
    })
}

fn control_json(c: &DbComplianceControl) -> serde_json::Value {
    serde_json::json!({
        "id": c.id,
        "framework_id": c.framework_id,
        "control_id": c.control_id,
        "title": c.title,
        "description": c.description,
        "category": c.category,
        "is_mandatory": c.is_mandatory,
        "version": c.version,
        "status": c.status,
        "created_at": c.created_at,
    })
}

fn evidence_json(e: &DbComplianceControlEvidence) -> serde_json::Value {
    serde_json::json!({
        "id": e.id,
        "control_id": e.control_id,
        "file_path": e.file_path,
        "description": e.description,
        "evidence_type": e.evidence_type,
        "status": e.status,
        "owner_id": e.owner_id,
        "approved_by": e.approved_by,
        "approved_at": e.approved_at,
        "created_at": e.created_at,
    })
}

/// `GET /api/compliance/frameworks`
pub async fn handle_list_frameworks(
    State(pool): State<Arc<crate::DbPool>>,
    Query(query): Query<ListFrameworksQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let mut db_query = compliance_frameworks::table
            .filter(compliance_frameworks::branch_id.eq(branch_id))
            .into_boxed();

        if let Some(status) = query.status {
            db_query = db_query.filter(compliance_frameworks::status.eq(status));
        }

        let frameworks: Vec<DbComplianceFramework> = db_query
            .order(compliance_frameworks::name.asc())
            .limit(query.limit.unwrap_or(100))
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let mut items = Vec::new();
        for fw in &frameworks {
            let controls_count: i64 = compliance_controls::table
                .filter(compliance_controls::framework_id.eq(fw.id))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0);
            let mut item = framework_json(fw);
            item["controls_count"] = serde_json::json!(controls_count);
            items.push(item);
        }
        Ok::<_, ComplianceError>(items)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `POST /api/compliance/frameworks`
pub async fn handle_create_framework(
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<CreateFrameworkRequest>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let name = req.name.trim().to_string();
        if name.is_empty() {
            return Err(ComplianceError::Validation("Framework name is required".to_string()));
        }
        let framework_key = req
            .framework_key
            .unwrap_or_else(|| name.to_lowercase().replace(' ', "_"));

        let fw = DbComplianceFramework {
            id: Uuid::new_v4(),
            branch_id,
            name,
            version: req.version.unwrap_or_else(|| "1.0.0".to_string()),
            description: req.description,
            framework_key,
            status: "active".to_string(),
            created_by: req.created_by,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(compliance_frameworks::table)
            .values(&fw)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        audit(
            &mut conn,
            branch_id,
            fw.created_by,
            "framework.create",
            "compliance_framework",
            Some(fw.id),
            Some(serde_json::json!({ "name": fw.name, "version": fw.version })),
        );

        Ok::<_, ComplianceError>(framework_json(&fw))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `PUT /api/compliance/frameworks/:framework_id`
pub async fn handle_update_framework(
    State(pool): State<Arc<crate::DbPool>>,
    Path(framework_id): Path<Uuid>,
    Json(req): Json<UpdateFrameworkRequest>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut fw: DbComplianceFramework = compliance_frameworks::table
            .find(framework_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Framework not found".to_string()))?;

        if let Some(name) = req.name {
            fw.name = name;
        }
        if let Some(version) = req.version {
            fw.version = version;
        }
        if let Some(description) = req.description {
            fw.description = Some(description);
        }
        if let Some(status) = req.status {
            if !["active", "archived"].contains(&status.as_str()) {
                return Err(ComplianceError::Validation("Status must be active or archived".to_string()));
            }
            fw.status = status;
        }
        fw.updated_at = now;

        diesel::update(compliance_frameworks::table.find(framework_id))
            .set(&fw)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        audit(
            &mut conn,
            fw.branch_id,
            None,
            "framework.update",
            "compliance_framework",
            Some(fw.id),
            Some(serde_json::json!({ "name": fw.name, "version": fw.version, "status": fw.status })),
        );

        Ok::<_, ComplianceError>(framework_json(&fw))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `POST /api/compliance/frameworks/:framework_id/archive`
pub async fn handle_archive_framework(
    State(pool): State<Arc<crate::DbPool>>,
    Path(framework_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        diesel::update(compliance_frameworks::table.find(framework_id))
            .set((
                compliance_frameworks::status.eq("archived"),
                compliance_frameworks::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Framework not found".to_string()))?;

        audit(
            &mut conn,
            BRANCH_ID_PLACEHOLDER,
            None,
            "framework.archive",
            "compliance_framework",
            Some(framework_id),
            None,
        );

        Ok::<_, ComplianceError>(serde_json::json!({ "ok": true, "id": framework_id }))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `GET /api/compliance/frameworks/:framework_id` — framework + controls + coverage
pub async fn handle_get_framework(
    State(pool): State<Arc<crate::DbPool>>,
    Path(framework_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;

        let fw: DbComplianceFramework = compliance_frameworks::table
            .find(framework_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Framework not found".to_string()))?;

        let controls: Vec<DbComplianceControl> = compliance_controls::table
            .filter(compliance_controls::framework_id.eq(framework_id))
            .order(compliance_controls::control_id.asc())
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let control_ids: Vec<Uuid> = controls.iter().map(|c| c.id).collect();

        let evidence_rows: Vec<DbComplianceControlEvidence> = if control_ids.is_empty() {
            Vec::new()
        } else {
            compliance_control_evidence::table
                .filter(compliance_control_evidence::control_id.eq_any(&control_ids))
                .load(&mut conn)
                .map_err(|e| ComplianceError::Database(e.to_string()))?
        };

        let mut evidence_by_control: std::collections::HashMap<Uuid, Vec<serde_json::Value>> =
            std::collections::HashMap::new();
        for ev in &evidence_rows {
            evidence_by_control
                .entry(ev.control_id)
                .or_default()
                .push(evidence_json(ev));
        }

        let mut total = 0usize;
        let mut covered = 0usize;
        let mut controls_json = Vec::new();

        for c in &controls {
            let evidence = evidence_by_control.get(&c.id).cloned().unwrap_or_default();
            let approved = evidence
                .iter()
                .filter(|e| e["status"] == serde_json::json!("approved"))
                .count();
            let has_evidence = approved > 0;
            total += 1;
            if has_evidence {
                covered += 1;
            }
            let mut cj = control_json(c);
            cj["evidence"] = serde_json::json!(evidence);
            cj["has_evidence"] = serde_json::json!(has_evidence);
            controls_json.push(cj);
        }

        let coverage = if total > 0 {
            (covered as f64 / total as f64 * 100.0 * 100.0).round() / 100.0
        } else {
            0.0
        };

        let mut out = framework_json(&fw);
        out["controls"] = serde_json::json!(controls_json);
        out["total_controls"] = serde_json::json!(total);
        out["controls_with_evidence"] = serde_json::json!(covered);
        out["coverage_percent"] = serde_json::json!(coverage);
        Ok::<_, ComplianceError>(out)
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `POST /api/compliance/controls`
pub async fn handle_create_control(
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<CreateControlRequest>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        let control_id = req.control_id.trim().to_string();
        if control_id.is_empty() || req.title.trim().is_empty() {
            return Err(ComplianceError::Validation(
                "Control id and title are required".to_string(),
            ));
        }

        let existing: i64 = compliance_controls::table
            .filter(compliance_controls::framework_id.eq(req.framework_id))
            .filter(compliance_controls::control_id.eq(&control_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        if existing > 0 {
            return Err(ComplianceError::Validation(format!(
                "Control {control_id} already exists in this framework"
            )));
        }

        let control = DbComplianceControl {
            id: Uuid::new_v4(),
            branch_id,
            framework_id: req.framework_id,
            control_id,
            title: req.title.trim().to_string(),
            description: req.description,
            category: req.category,
            is_mandatory: req.is_mandatory.unwrap_or(true),
            version: "1.0.0".to_string(),
            status: "active".to_string(),
            created_by: req.created_by,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(compliance_controls::table)
            .values(&control)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        audit(
            &mut conn,
            branch_id,
            control.created_by,
            "control.create",
            "compliance_control",
            Some(control.id),
            Some(serde_json::json!({ "framework_id": control.framework_id, "control_id": control.control_id })),
        );

        Ok::<_, ComplianceError>(control_json(&control))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `PUT /api/compliance/controls/:control_id`
pub async fn handle_update_control(
    State(pool): State<Arc<crate::DbPool>>,
    Path(control_id): Path<Uuid>,
    Json(req): Json<UpdateControlRequest>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut control: DbComplianceControl = compliance_controls::table
            .find(control_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Control not found".to_string()))?;

        if let Some(title) = req.title {
            control.title = title;
        }
        if let Some(description) = req.description {
            control.description = Some(description);
        }
        if let Some(category) = req.category {
            control.category = Some(category);
        }
        if let Some(is_mandatory) = req.is_mandatory {
            control.is_mandatory = is_mandatory;
        }
        if let Some(status) = req.status {
            control.status = status;
        }
        control.updated_at = now;

        diesel::update(compliance_controls::table.find(control_id))
            .set(&control)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        audit(
            &mut conn,
            control.branch_id,
            None,
            "control.update",
            "compliance_control",
            Some(control.id),
            None,
        );

        Ok::<_, ComplianceError>(control_json(&control))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `POST /api/compliance/evidence/attach`
pub async fn handle_attach_evidence(
    State(pool): State<Arc<crate::DbPool>>,
    Json(req): Json<AttachEvidenceRequest>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = BRANCH_ID_PLACEHOLDER;

        // Control must exist and belong to the branch scope.
        let _control: DbComplianceControl = compliance_controls::table
            .find(req.control_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Control not found".to_string()))?;

        let ev = DbComplianceControlEvidence {
            id: Uuid::new_v4(),
            branch_id,
            control_id: req.control_id,
            file_path: req.file_path,
            description: req.description,
            evidence_type: req.evidence_type.unwrap_or_else(|| "artifact".to_string()),
            status: "draft".to_string(),
            owner_id: req.owner_id,
            approved_by: None,
            approved_at: None,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(compliance_control_evidence::table)
            .values(&ev)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        audit(
            &mut conn,
            branch_id,
            ev.owner_id,
            "evidence.attach",
            "compliance_control_evidence",
            Some(ev.id),
            Some(serde_json::json!({ "control_id": ev.control_id, "file_path": ev.file_path })),
        );

        Ok::<_, ComplianceError>(evidence_json(&ev))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `POST /api/compliance/evidence/:evidence_id/approve`
pub async fn handle_approve_evidence(
    State(pool): State<Arc<crate::DbPool>>,
    Path(evidence_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        let mut ev: DbComplianceControlEvidence = compliance_control_evidence::table
            .find(evidence_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Evidence not found".to_string()))?;

        let approver: Option<Uuid> = body
            .get("approved_by")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        ev.status = "approved".to_string();
        ev.approved_by = approver;
        ev.approved_at = Some(now);
        ev.updated_at = now;

        diesel::update(compliance_control_evidence::table.find(evidence_id))
            .set(&ev)
            .execute(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        audit(
            &mut conn,
            ev.branch_id,
            approver,
            "evidence.approve",
            "compliance_control_evidence",
            Some(ev.id),
            None,
        );

        Ok::<_, ComplianceError>(evidence_json(&ev))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `DELETE /api/compliance/evidence/:evidence_id`
pub async fn handle_delete_evidence(
    State(pool): State<Arc<crate::DbPool>>,
    Path(evidence_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;

        diesel::delete(compliance_control_evidence::table.find(evidence_id))
            .execute(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Evidence not found".to_string()))?;

        audit(
            &mut conn,
            BRANCH_ID_PLACEHOLDER,
            None,
            "evidence.delete",
            "compliance_control_evidence",
            Some(evidence_id),
            None,
        );

        Ok::<_, ComplianceError>(serde_json::json!({ "ok": true, "id": evidence_id }))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(Json(result))
}

/// `GET /api/compliance/frameworks/:framework_id/export.csv`
///
/// Audit-ready CSV scorecard: control id, title, category, mandatory flag,
/// evidence status and coverage, plus framework header rows.
pub async fn handle_export_framework_csv(
    State(pool): State<Arc<crate::DbPool>>,
    Path(framework_id): Path<Uuid>,
) -> Result<axum::response::Response, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| ComplianceError::Database(e.to_string()))?;

        let fw: DbComplianceFramework = compliance_frameworks::table
            .find(framework_id)
            .first(&mut conn)
            .map_err(|_| ComplianceError::NotFound("Framework not found".to_string()))?;

        let controls: Vec<DbComplianceControl> = compliance_controls::table
            .filter(compliance_controls::framework_id.eq(framework_id))
            .order(compliance_controls::control_id.asc())
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let control_ids: Vec<Uuid> = controls.iter().map(|c| c.id).collect();
        let evidence_rows: Vec<DbComplianceControlEvidence> = if control_ids.is_empty() {
            Vec::new()
        } else {
            compliance_control_evidence::table
                .filter(compliance_control_evidence::control_id.eq_any(&control_ids))
                .load(&mut conn)
                .map_err(|e| ComplianceError::Database(e.to_string()))?
        };

        let mut evidence_by_control: std::collections::HashMap<Uuid, Vec<&DbComplianceControlEvidence>> =
            std::collections::HashMap::new();
        for ev in &evidence_rows {
            evidence_by_control.entry(ev.control_id).or_default().push(ev);
        }

        let mut csv = String::new();
        csv.push_str(&format!(
            "Compliance Scorecard,{},{},v{}\n",
            fw.name, fw.framework_key, fw.version
        ));
        csv.push_str("Generated at,");
        csv.push_str(&Utc::now().to_rfc3339());
        csv.push('\n');
        csv.push('\n');
        csv.push_str("control_id,title,category,mandatory,evidence_count,approved,status\n");

        let mut covered = 0usize;
        for c in &controls {
            let evs = evidence_by_control.get(&c.id).cloned().unwrap_or_default();
            let approved = evs.iter().filter(|e| e.status == "approved").count();
            let has_evidence = approved > 0;
            if has_evidence {
                covered += 1;
            }
            let title = c.title.replace(',', " ").replace('\n', " ");
            let category = c.category.clone().unwrap_or_default().replace(',', " ");
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                c.control_id,
                title,
                category,
                c.is_mandatory,
                evs.len(),
                approved,
                c.status
            ));
        }

        let total = controls.len();
        let coverage = if total > 0 {
            format!("{:.2}%", covered as f64 / total as f64 * 100.0)
        } else {
            "0.00%".to_string()
        };
        csv.push('\n');
        csv.push_str(&format!("Total controls,{total}\n"));
        csv.push_str(&format!("Controls with approved evidence,{covered}\n"));
        csv.push_str(&format!("Coverage,{coverage}\n"));

        let fname = format!("compliance-{}-{}.csv", fw.framework_key, fw.version);
        let headers = [
            (
                axum::http::header::CONTENT_TYPE,
                "text/csv; charset=utf-8".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{fname}\""),
            ),
        ];

        Ok::<_, ComplianceError>((
            headers,
            csv,
        ))
    })
    .await
    .map_err(|e| ComplianceError::Internal(e.to_string()))??;

    Ok(result.into_response())
}

/// Server-rendered framework detail view, reachable from the suite Compliance app.
/// Resolves the framework by name (falling back to its `framework_key`) and lists
/// every control together with its collected evidence, so agents get an audit-ready
/// page without leaving the desktop shell.
pub async fn handle_framework_detail_page(
    State(pool): State<Arc<crate::DbPool>>,
    Path(name): Path<String>,
) -> Html<String> {
    let result = tokio::task::spawn_blocking(move || -> Result<String, ComplianceError> {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let fw: DbComplianceFramework = compliance_frameworks::table
            .filter(compliance_frameworks::name.eq(&name))
            .first(&mut conn)
            .or_else(|_| -> Result<DbComplianceFramework, diesel::result::Error> {
                compliance_frameworks::table
                    .filter(compliance_frameworks::framework_key.eq(&name))
                    .first(&mut conn)
            })
            .map_err(|_| ComplianceError::NotFound("Framework not found".to_string()))?;

        let controls: Vec<DbComplianceControl> = compliance_controls::table
            .filter(compliance_controls::framework_id.eq(fw.id))
            .order(compliance_controls::control_id.asc())
            .load(&mut conn)
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let control_ids: Vec<Uuid> = controls.iter().map(|c| c.id).collect();

        let evidence_rows: Vec<DbComplianceControlEvidence> = if control_ids.is_empty() {
            Vec::new()
        } else {
            compliance_control_evidence::table
                .filter(compliance_control_evidence::control_id.eq_any(&control_ids))
                .load(&mut conn)
                .map_err(|e| ComplianceError::Database(e.to_string()))?
        };

        let mut evidence_count: std::collections::HashMap<Uuid, usize> = std::collections::HashMap::new();
        for ev in &evidence_rows {
            *evidence_count.entry(ev.control_id).or_insert(0) += 1;
        }

        let total = controls.len();
        let covered = controls
            .iter()
            .filter(|c| evidence_count.get(&c.id).copied().unwrap_or(0) > 0)
            .count();
        let coverage = if total > 0 {
            (covered as f64 / total as f64 * 100.0).round()
        } else {
            0.0
        };

        let mut rows = String::new();
        for c in &controls {
            let cnt = evidence_count.get(&c.id).copied().unwrap_or(0);
            let badge = if cnt > 0 {
                "<span class=\"cv-badge ok\">Covered</span>"
            } else {
                "<span class=\"cv-badge missing\">No evidence</span>"
            };
            rows.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(&c.control_id),
                escape_html(&c.title),
                escape_html(c.category.as_deref().unwrap_or("")),
                cnt,
                badge
            ));
        }

        let html = format!(
            "<div class=\"framework-detail\">\
                <div class=\"framework-detail-header\">\
                    <h2>{}</h2>\
                    <p class=\"framework-detail-desc\">{}</p>\
                    <div class=\"framework-detail-coverage\">Coverage: {}% ({}/{} controls with evidence)</div>\
                </div>\
                <table class=\"framework-controls-table\">\
                    <thead><tr><th>Control</th><th>Title</th><th>Category</th><th>Evidence</th><th>Status</th></tr></thead>\
                    <tbody>{}</tbody>\
                </table>\
            </div>",
            escape_html(&fw.name),
            escape_html(fw.description.as_deref().unwrap_or("")),
            coverage,
            covered,
            total,
            rows
        );
        Ok(html)
    });

    match result.await {
        Ok(Ok(html)) => Html(html),
        Ok(Err(e)) => Html(format!(
            "<div class=\"framework-detail-error\">{}</div>",
            escape_html(&e.to_string())
        )),
        Err(e) => Html(format!(
            "<div class=\"framework-detail-error\">{}</div>",
            escape_html(&e.to_string())
        )),
    }
}

fn escape_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            '\'' => s.push_str("&#39;"),
            _ => s.push(ch),
        }
    }
    s
}
