use super::*;


// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct CreateBranchBody {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBranchBody {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// `GET /api/cloud/organizations/{org_id}/branches`
pub(crate) async fn list_branches(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    #[derive(diesel::QueryableByName, Debug)]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        slug: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        bots_count: i64,
    }

    let rows: Vec<BranchRow> = diesel::sql_query(
        r#"SELECT b.id, b.name, b.slug, b.description, b.is_active,
                  COUNT(DISTINCT bt.id) AS bots_count
           FROM branches b
           LEFT JOIN bots bt ON bt.branch_id = b.id
           WHERE b.org_id = $1
           GROUP BY b.id ORDER BY b.name"#,
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id_param)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "name": r.name,
            "slug": r.slug,
            "description": r.description,
            "is_active": r.is_active,
            "bots_count": r.bots_count,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "branches": result })))
}

/// `POST /api/cloud/organizations/{org_id}/branches`
pub(crate) async fn create_branch_handler(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
    Json(body): Json<CreateBranchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Branch name is required".to_string()));
    }

    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Get tenant_id for this organization
    #[derive(diesel::QueryableByName, Debug)]
    struct TenantIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        tenant_id: Uuid,
    }

    let tenant_row: Option<TenantIdRow> = diesel::sql_query(
        "SELECT tenant_id FROM organizations WHERE org_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id_param)
    .get_result(&mut conn)
    .ok();

    let tenant_id = match tenant_row {
        Some(row) => row.tenant_id,
        None => return Err((StatusCode::NOT_FOUND, "Organization not found".to_string())),
    };

    let branch_id = integration::create_branch_inner(&mut conn, org_id_param, tenant_id, body.name.trim())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "id": branch_id,
        "name": body.name.trim(),
        "org_id": org_id_param,
    })))
}

/// `PUT /api/cloud/organizations/{org_id}/branches/{branch_id}`
pub(crate) async fn update_branch_handler(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, branch_id_param)): axum::extract::Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateBranchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    if let Some(ref n) = body.name {
        if n.trim().is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Branch name cannot be empty".to_string()));
        }
        let slug = n.trim().to_lowercase().replace(' ', "-");
        let affected = diesel::sql_query(
            "UPDATE branches SET name = $1, slug = $2, updated_at = NOW() WHERE id = $3 AND org_id = $4",
        )
        .bind::<diesel::sql_types::Text, _>(n.trim())
        .bind::<diesel::sql_types::Text, _>(&slug)
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;

        if affected == 0 {
            return Err((StatusCode::NOT_FOUND, "Branch not found".to_string()));
        }
    }

    if let Some(ref desc) = body.description {
        diesel::sql_query(
            "UPDATE branches SET description = $1, updated_at = NOW() WHERE id = $2 AND org_id = $3",
        )
        .bind::<diesel::sql_types::Text, _>(desc)
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update desc: {e}")))?;
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/organizations/{org_id}/branches/{branch_id}`
pub(crate) async fn delete_branch_handler(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, branch_id_param)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Unlink bots first
    diesel::sql_query("UPDATE bots SET branch_id = NULL WHERE branch_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .execute(&mut conn)
        .ok();

    let deleted = diesel::sql_query("DELETE FROM branches WHERE id = $1 AND org_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(branch_id_param)
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Branch not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Services (add-ons purchased)
