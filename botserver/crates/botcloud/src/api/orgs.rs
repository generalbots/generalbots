use super::*;


// ─────────────────────────────────────────────────────────────────────────────
/// `GET /api/cloud/organizations`
pub(crate) async fn list_organizations(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let admin = is_super_admin(&headers, &mut conn).unwrap_or(false);

    #[derive(diesel::QueryableByName, Debug)]
    struct OrgWithCounts {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        domain: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        plan_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        branches_count: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        bots_count: i64,
    }

    let base_query = r#"
        SELECT o.org_id AS id, o.name, o.domain,
            (SELECT br.description FROM billing_recurring br
             JOIN branches b2 ON b2.id = br.org_id
             WHERE b2.org_id = o.org_id
             ORDER BY br.created_at DESC LIMIT 1) AS plan_name,
            COUNT(DISTINCT b.id) AS branches_count,
            COUNT(DISTINCT bt.id) AS bots_count
        FROM organizations o
        LEFT JOIN branches b ON b.org_id = o.org_id
        LEFT JOIN bots bt ON bt.branch_id = b.id
    "#;

    let orgs: Vec<OrgWithCounts> = if admin {
        diesel::sql_query(format!(
            "{base_query} WHERE o.org_id <> '00000000-0000-0000-0000-000000000000' GROUP BY o.org_id, o.name ORDER BY o.name"
        ))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    } else if let Some(bid) = user_branch_id {
        diesel::sql_query(format!("{base_query} WHERE o.org_id = (SELECT org_id FROM branches WHERE id = $1) GROUP BY o.org_id, o.name"))
        .bind::<diesel::sql_types::Uuid, _>(bid)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    } else {
        diesel::sql_query(format!(
            "{base_query} WHERE o.org_id <> '00000000-0000-0000-0000-000000000000' GROUP BY o.org_id, o.name ORDER BY o.name"
        ))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
    };

    let result: Vec<serde_json::Value> = orgs.into_iter().map(|r| {
        let plan = if admin {
            "private-cloud".to_string()
        } else {
            r.plan_name.as_deref()
                .and_then(|d| d.split_once(" - ").map(|(p, _)| p).or(Some(d)))
                .map(|p| p.to_lowercase().replace(' ', "-"))
                .filter(|p| p == "free" || p == "shared" || p == "private-cloud")
                .unwrap_or_else(|| "free".to_string())
        };
        serde_json::json!({
            "id": r.id,
            "name": r.name,
            "plan": plan,
            "status": "active",
            "domain": r.domain,
            "branches_count": r.branches_count,
            "bots_count": r.bots_count,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "organizations": result, "is_admin": admin })))
}

/// `POST /api/cloud/organizations`
pub(crate) async fn create_organization(
    State(service): State<Arc<SaasService>>,
    Json(body): Json<CreateOrgBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Organization name is required".to_string()));
    }

    let org_id = integration::create_organization(service.pool(), &body.name, body.domain.as_deref())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let plan = body.plan.unwrap_or_else(|| "free".to_string());
    let period = body.period.unwrap_or_else(|| "monthly".to_string());
    let storage = body.storage_gb.unwrap_or(5.0);

    // If paid plan, create checkout session
    let config = botbilling::default_product_config();
    if let Some(plan_cfg) = config.plans.get(&plan) {
        if matches!(plan_cfg.price, botbilling::PlanPrice::Fixed { .. }) {
            let total = match &plan_cfg.price {
                botbilling::PlanPrice::Fixed { amount, .. } => *amount as f64 / 100.0,
                _ => 0.0,
            };
            let payload_json = serde_json::json!({
                "plan": plan, "period": period,
                "storage": storage, "ai": [], "total": total, "currency": "usd"
            });
            return Ok(Json(serde_json::json!({
                "status": "checkout_required",
                "org_id": org_id,
                "checkout_payload": payload_json,
                "checkout_url": format!("/cloud/checkout?payload={}", url::form_urlencoded::byte_serialize(payload_json.to_string().as_bytes()).collect::<String>()),
            })));
        }
    }

    Ok(Json(serde_json::json!({
        "status": "created",
        "org_id": org_id,
        "name": body.name,
        "plan": plan,
    })))
}

/// `GET /api/cloud/organizations/{org_id}/billing`
pub(crate) async fn org_billing_portal(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Redirects to Stripe customer billing portal
    let portal_url = format!(
        "{}/api/billing/portal?org_id={}",
        service.config.base_url, org_id
    );
    Ok(Json(serde_json::json!({ "url": portal_url, "org_id": org_id })))
}

/// `GET /api/cloud/organizations/{org_id}`
pub(crate) async fn get_organization(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let rows: Vec<OrgRow> = diesel::sql_query("SELECT org_id AS id, name FROM organizations WHERE org_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let org = rows.into_iter().next().ok_or_else(|| {
        (StatusCode::NOT_FOUND, "Organization not found".to_string())
    })?;

    Ok(Json(serde_json::json!({
        "id": org.id,
        "name": org.name,
        "plan": "free",
        "status": "active",
    })))
}

/// `PUT /api/cloud/organizations/{org_id}`
pub(crate) async fn update_organization(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
    Json(body): Json<UpdateOrgBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Organization name cannot be empty".to_string()));
        }
        let slug = name.to_lowercase().replace(' ', "-").replace(|c: char| !c.is_alphanumeric() && c != '-', "");
        let affected = diesel::sql_query("UPDATE organizations SET name = $1, slug = $2, updated_at = NOW() WHERE org_id = $3")
            .bind::<diesel::sql_types::Text, _>(name.trim())
            .bind::<diesel::sql_types::Text, _>(&slug)
            .bind::<diesel::sql_types::Uuid, _>(org_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;

        if affected == 0 {
            return Err((StatusCode::NOT_FOUND, "Organization not found".to_string()));
        }
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/organizations/{org_id}`
pub(crate) async fn delete_organization(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Remove workspace resources and workspaces for this org
    let branch_id: Option<Uuid> = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .get_result::<BranchIdRow>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?
        .map(|r| r.id);

    if let Some(bid) = branch_id {
        diesel::sql_query(
            "DELETE FROM workspace_resources WHERE workspace_id IN (SELECT id FROM cloud_workspaces WHERE branch_id = $1)"
        )
        .bind::<diesel::sql_types::Uuid, _>(bid)
        .execute(&mut conn)
        .ok();

        use crate::schema_ext::cloud_workspaces::dsl as cw;
        diesel::delete(cw::cloud_workspaces.filter(cw::branch_id.eq(bid)))
            .execute(&mut conn)
            .ok();
    }

    // Remove organization record
    let affected = diesel::sql_query("DELETE FROM organizations WHERE org_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Organization not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Workspaces
