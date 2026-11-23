use super::*;

/// Whether the caller is trusted to append to the compliance audit trail.
///
/// Audit events are compliance evidence, so only platform administrators and
/// internal (service/bot) principals may create them. The public compliance
/// route table intentionally allows every logged-in user through the RBAC
/// layer (see `routes_admin.rs`), which is why this check lives here. An
/// anonymous or ordinary `Role::User` principal always fails closed.
fn may_create_audit_log(user: &AuthenticatedUser) -> bool {
    user.is_admin() || user.is_super_admin() || user.is_service() || user.is_bot()
}

/// Actor stored on a newly created audit event.
///
/// Always derived from the authenticated principal — never from the
/// client-supplied `user_id`/`actor_id` body fields. Those fields were removed
/// from `CreateAuditLogRequest` (#2749) because a forged actor is the whole
/// point of the attack this guards against.
fn audit_actor_id(user: &AuthenticatedUser) -> Uuid {
    user.user_id
}

pub async fn handle_list_audit_logs(
    State(pool): State<Arc<crate::DbPool>>,
    headers: HeaderMap,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);
        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);

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
    headers: HeaderMap,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateAuditLogRequest>,
) -> Result<Json<AuditLogEntry>, ComplianceError> {
    // #2749: a logged-in user must not be able to forge compliance evidence.
    // The RBAC route table leaves POST /api/compliance/** open to every
    // authenticated user, so authorization for this mutating endpoint is
    // enforced here against the authenticated principal.
    if !may_create_audit_log(&user) {
        return Err(ComplianceError::Forbidden(
            "Only administrators and internal services may create audit events".to_string(),
        ));
    }

    // Bind the actor from the authenticated principal and the timestamp from
    // the server clock. Neither is taken from the request body.
    let actor_id = audit_actor_id(&user);
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);

        let metadata = serde_json::to_value(req.metadata.unwrap_or_default()).unwrap_or(serde_json::json!({}));

        let db_log = DbAuditLog {
            id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            bot_id: Uuid::nil(),
            branch_id,
            event_type: req.event_type.to_string(),
            user_id: Some(actor_id),
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
    headers: HeaderMap,
    Json(req): Json<CreateTrainingRequest>,
) -> Result<Json<TrainingRecord>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();
        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);

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
    headers: HeaderMap,
) -> Result<Json<Vec<TrainingRecord>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);

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
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);

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
    headers: HeaderMap,
    Path(check_id): Path<Uuid>,
) -> Result<Json<ComplianceCheckResult>, ComplianceError> {
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;
        let now = Utc::now();

        let branch_id = crate::branch_scope::resolve_branch(&headers, &pool);
        let mut db_check: DbComplianceCheck = compliance_checks::table
            .filter(compliance_checks::id.eq(check_id))
            .filter(compliance_checks::branch_id.eq(branch_id))
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

#[cfg(test)]
mod tests {
    use super::*;
    use botsecurity_auth::Role;

    #[test]
    fn administrators_and_internal_principals_may_append_audit_events() {
        let admin =
            AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string()).with_role(Role::Admin);
        let super_admin =
            AuthenticatedUser::new(Uuid::new_v4(), "root".to_string()).with_role(Role::SuperAdmin);
        let service = AuthenticatedUser::service("internal");
        let bot = AuthenticatedUser::bot_user(Uuid::new_v4(), "worker");

        assert!(may_create_audit_log(&admin));
        assert!(may_create_audit_log(&super_admin));
        assert!(may_create_audit_log(&service));
        assert!(may_create_audit_log(&bot));
    }

    #[test]
    fn ordinary_and_anonymous_users_cannot_append_audit_events() {
        let regular = AuthenticatedUser::new(Uuid::new_v4(), "alice".to_string());
        let moderator =
            AuthenticatedUser::new(Uuid::new_v4(), "mod".to_string()).with_role(Role::Moderator);
        let anonymous = AuthenticatedUser::anonymous();

        assert!(!may_create_audit_log(&regular));
        assert!(!may_create_audit_log(&moderator));
        assert!(!may_create_audit_log(&anonymous));
    }

    #[test]
    fn actor_is_always_the_authenticated_principal_id() {
        let user_id = Uuid::new_v4();
        let user = AuthenticatedUser::new(user_id, "admin".to_string()).with_role(Role::Admin);

        assert_eq!(audit_actor_id(&user), user_id);
        assert_eq!(audit_actor_id(&AuthenticatedUser::anonymous()), Uuid::nil());
    }
}
