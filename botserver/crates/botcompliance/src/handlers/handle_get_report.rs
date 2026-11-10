use super::*;

pub async fn handle_get_report(
    State(pool): State<Arc<crate::DbPool>>,
    headers: HeaderMap,
    Query(query): Query<ListChecksQuery>,
) -> Result<Json<ComplianceReport>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);

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
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, ComplianceError> {
    // #2132 — refuse an oversized body before it is buffered.
    botsecurity_core::upload_policy::check_declared_length(
        botsecurity_core::upload_policy::UploadClass::Evidence,
        &headers,
    )
    .map_err(|error| {
        ComplianceError::Validation(
            botsecurity_core::upload_policy::rejection_message(&error, &headers).1,
        )
    })?;

    let mut file_name = String::new();
    let mut content_type = String::new();
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut category = String::new();
    let mut file_size = 0usize;

    while let Some(field) = multipart.next_field().await.map_err(|e| ComplianceError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("unknown").to_string();
                content_type = field.content_type().map(str::to_string).unwrap_or_default();
                let data = field.bytes().await.map_err(|e| ComplianceError::Internal(e.to_string()))?;
                file_size = data.len();
                file_bytes = data.to_vec();
            }
            "category" => {
                category = field.text().await.map_err(|e| ComplianceError::Internal(e.to_string()))?;
            }
            _ => {}
        }
    }

    // #2132 — the extension, the declared type and the magic bytes must agree.
    let upload = botsecurity_core::upload_policy::validate_upload(
        botsecurity_core::upload_policy::UploadClass::Evidence,
        &file_name,
        &content_type,
        &file_bytes,
    )
    .map_err(|error| {
        ComplianceError::Validation(
            botsecurity_core::upload_policy::rejection_message(&error, &headers).1,
        )
    })?;
    let file_name = upload.filename;

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
