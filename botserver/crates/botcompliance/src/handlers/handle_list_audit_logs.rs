use super::*;

/// Page-size ceiling: one request must never dump the whole trail (#2749).
const MAX_AUDIT_LOG_LIMIT: i64 = 500;
const DEFAULT_AUDIT_LOG_LIMIT: i64 = 100;

/// Effective tenant filter. `None` means "no filter" and is only ever produced
/// for an administrator who explicitly asked for `all_tenants`.
struct AuditLogScope {
    branch_id: Option<Uuid>,
    org_id: Option<Uuid>,
}

/// Pure scope decision (#2749): always the caller's verified tenant, widened
/// only for an authenticated admin who explicitly requests `all_tenants`.
fn audit_log_scope(
    is_admin: bool,
    all_tenants: bool,
    verified_org: Option<Uuid>,
    verified_branch: Uuid,
) -> AuditLogScope {
    if is_admin && all_tenants {
        AuditLogScope {
            branch_id: None,
            org_id: None,
        }
    } else {
        AuditLogScope {
            branch_id: Some(verified_branch),
            org_id: verified_org,
        }
    }
}

/// Pure page-size clamp: positive and capped.
fn audit_log_limit(requested: Option<i64>) -> i64 {
    requested
        .unwrap_or(DEFAULT_AUDIT_LOG_LIMIT)
        .clamp(1, MAX_AUDIT_LOG_LIMIT)
}

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
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, ComplianceError> {
    // Tenant comes from verified JWT claims / the principal, never from a
    // client parameter; `all_tenants` only widens it for an authenticated admin.
    let verified_org = botsecurity_core::tenant::org_from_claims(&headers).or(user.organization_id);
    let is_admin = user.is_admin();
    let all_tenants = query.all_tenants.unwrap_or(false);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| ComplianceError::Database(e.to_string()))?;

        let limit = audit_log_limit(query.limit);
        let offset = query.offset.unwrap_or(0).max(0);
        let verified_branch = crate::branch_scope::resolve_branch(&headers, &pool);
        let scope = audit_log_scope(is_admin, all_tenants, verified_org, verified_branch);

        let mut db_query = compliance_audit_log::table.into_boxed();

        if let Some(branch_id) = scope.branch_id {
            db_query = db_query.filter(compliance_audit_log::branch_id.eq(branch_id));
        }

        // Internal writer rows carry nil org_id (framework_handlers/audit.rs);
        // keep them alongside the caller's org. The branch filter still excludes
        // nil-org rows belonging to other tenants.
        if let Some(org_id) = scope.org_id {
            db_query =
                db_query.filter(compliance_audit_log::org_id.eq_any(vec![org_id, Uuid::nil()]));
        }

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

        let metadata = serde_json::to_value(req.metadata.unwrap_or_else(std::collections::HashMap::new))
            .unwrap_or(serde_json::json!({}));

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
                training_type: r
                    .training_type
                    .parse()
                    .unwrap_or(TrainingType::SecurityAwareness),
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

    #[test]
    fn non_admin_cannot_widen_scope_with_all_tenants_flag() {
        let org = Uuid::new_v4();
        let branch = Uuid::new_v4();
        let regular = AuthenticatedUser::new(Uuid::new_v4(), "alice".to_string());
        let moderator =
            AuthenticatedUser::new(Uuid::new_v4(), "mod".to_string()).with_role(Role::Moderator);

        for user in [&regular, &moderator] {
            let scope = audit_log_scope(user.is_admin(), true, Some(org), branch);
            assert_eq!((scope.branch_id, scope.org_id), (Some(branch), Some(org)));
        }

        // No org claim still leaves the verified branch as a hard boundary.
        let scope = audit_log_scope(regular.is_admin(), true, None, branch);
        assert_eq!((scope.branch_id, scope.org_id), (Some(branch), None));
    }

    #[test]
    fn admin_is_tenant_scoped_unless_broad_scope_is_explicit() {
        let org = Uuid::new_v4();
        let branch = Uuid::new_v4();
        let admin =
            AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string()).with_role(Role::Admin);

        let scoped = audit_log_scope(admin.is_admin(), false, Some(org), branch);
        assert_eq!((scoped.branch_id, scoped.org_id), (Some(branch), Some(org)));

        let super_admin =
            AuthenticatedUser::new(Uuid::new_v4(), "root".to_string()).with_role(Role::SuperAdmin);
        let broad = audit_log_scope(super_admin.is_admin(), true, Some(org), branch);
        assert_eq!((broad.branch_id, broad.org_id), (None, None));
    }

    #[test]
    fn page_size_is_capped_and_always_positive() {
        assert_eq!(audit_log_limit(None), DEFAULT_AUDIT_LOG_LIMIT);
        assert_eq!(audit_log_limit(Some(i64::MAX)), MAX_AUDIT_LOG_LIMIT);
        assert_eq!(audit_log_limit(Some(0)), 1);
        assert_eq!(audit_log_limit(Some(-25)), 1);
        assert_eq!(audit_log_limit(Some(50)), 50);
    }
}
