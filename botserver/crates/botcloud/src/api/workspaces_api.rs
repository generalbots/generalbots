use super::*;


// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceBody {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignResourceBody {
    pub store_item_id: String,
    pub name: Option<String>,
}

/// `GET /api/cloud/organizations/{org_id}/workspaces`
pub(crate) async fn list_workspaces(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    use crate::schema_ext::cloud_workspaces::dsl as cw;
    let rows = cw::cloud_workspaces
        .filter(cw::branch_id.eq(branch_id))
        .order(cw::created_at.desc())
        .load::<(Uuid, Uuid, Uuid, String, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|(wid, _, _, wname, wdesc, wicon, _, _)| {
        serde_json::json!({
            "id": wid,
            "name": wname,
            "description": wdesc,
            "icon": wicon,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "workspaces": result })))
}

/// `POST /api/cloud/organizations/{org_id}/workspaces`
pub(crate) async fn create_workspace(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path(org_id_param): axum::extract::Path<Uuid>,
    Json(body): Json<CreateWorkspaceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Workspace name is required".to_string()));
    }

    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    let now = chrono::Utc::now();
    let ws_id = Uuid::new_v4();

    use crate::schema_ext::cloud_workspaces::dsl as cw;
    diesel::insert_into(cw::cloud_workspaces)
        .values((
            cw::id.eq(ws_id),
            cw::org_id.eq(org_id_param),
            cw::branch_id.eq(branch_id),
            cw::name.eq(body.name.trim()),
            cw::description.eq(body.description),
            cw::icon.eq(body.icon),
            cw::created_at.eq(now),
            cw::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert: {e}")))?;

    Ok(Json(serde_json::json!({
        "id": ws_id,
        "name": body.name.trim(),
        "org_id": org_id_param,
    })))
}

/// `PUT /api/cloud/organizations/{org_id}/workspaces/{ws_id}`
pub(crate) async fn update_workspace(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateWorkspaceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    use crate::schema_ext::cloud_workspaces::dsl as cw;
    let filter = cw::id.eq(ws_id_param).and(cw::branch_id.eq(branch_id));

    if let Some(n) = &body.name {
        diesel::update(cw::cloud_workspaces).filter(filter).set(cw::name.eq(n.trim())).execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update: {e}")))?;
    }
    if let Some(d) = &body.description {
        diesel::update(cw::cloud_workspaces).filter(filter).set(cw::description.eq(d)).execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update desc: {e}")))?;
    }
    if let Some(i) = &body.icon {
        diesel::update(cw::cloud_workspaces).filter(filter).set(cw::icon.eq(i)).execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update icon: {e}")))?;
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/organizations/{org_id}/workspaces/{ws_id}`
pub(crate) async fn delete_workspace(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let branch_id: Uuid = diesel::sql_query("SELECT id FROM branches WHERE org_id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(org_id_param)
        .get_result::<BranchIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Branch lookup: {e}")))?
        .id;


    // Remove resources first
    use crate::schema_ext::workspace_resources::dsl as wr;
    diesel::delete(wr::workspace_resources.filter(wr::workspace_id.eq(ws_id_param)))
        .execute(&mut conn)
        .ok();

    use crate::schema_ext::cloud_workspaces::dsl as cw;
    let deleted = diesel::delete(cw::cloud_workspaces.filter(cw::id.eq(ws_id_param).and(cw::branch_id.eq(branch_id))))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Workspace not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/// `GET /api/cloud/organizations/{org_id}/workspaces/{ws_id}/resources`
pub(crate) async fn list_workspace_resources(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((_org_id, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::workspace_resources::dsl as wr;
    let rows = wr::workspace_resources
        .filter(wr::workspace_id.eq(ws_id_param))
        .order(wr::created_at.desc())
        .load::<(Uuid, Uuid, Uuid, String, String, String, String, Option<serde_json::Value>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    let result: Vec<serde_json::Value> = rows.into_iter().map(|(rid, _, _, sid, rname, rtype, rstatus, rconfig, rprov, _, _)| {
        serde_json::json!({
            "id": rid,
            "store_item_id": sid,
            "name": rname,
            "resource_type": rtype,
            "status": rstatus,
            "config": rconfig,
            "provisioned_at": rprov,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "resources": result })))
}

/// `POST /api/cloud/organizations/{org_id}/workspaces/{ws_id}/resources`
pub(crate) async fn assign_resource(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((org_id_param, ws_id_param)): axum::extract::Path<(Uuid, Uuid)>,
    Json(body): Json<AssignResourceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let restype = if body.store_item_id.starts_with("vps-") || body.store_item_id.starts_with("gpu-") {
        "compute"
    } else if body.store_item_id.starts_with("storage-") {
        "storage"
    } else if body.store_item_id.starts_with("number-") {
        "phone"
    } else if body.store_item_id.starts_with("domain-") || body.store_item_id.starts_with("calls-") {
        "comms"
    } else {
        "other"
    };


    let now = chrono::Utc::now();
    let res_id = Uuid::new_v4();

    use crate::schema_ext::workspace_resources::dsl as wr;
    diesel::insert_into(wr::workspace_resources)
        .values((
            wr::id.eq(res_id),
            wr::workspace_id.eq(ws_id_param),
            wr::org_id.eq(org_id_param),
            wr::store_item_id.eq(&body.store_item_id),
            wr::name.eq(body.name.unwrap_or_else(|| body.store_item_id.clone())),
            wr::resource_type.eq(restype),
            wr::status.eq("provisioning"),
            wr::provisioned_at.eq(now),
            wr::created_at.eq(now),
            wr::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert: {e}")))?;

    let db_pool = service.pool().clone();
    let item_id = body.store_item_id.clone();
    let rid = res_id;
    let wid = ws_id_param;
    let oid = org_id_param;

    tokio::spawn(async move {
        if let Err(e) = provision_resource(&db_pool, &item_id, rid, wid, oid).await {
            tracing::error!("Provisioning failed for resource {rid} ({item_id}): {e}");
        }
    });

    Ok(Json(serde_json::json!({
        "id": res_id,
        "store_item_id": body.store_item_id,
        "resource_type": restype,
        "status": "provisioning",
    })))
}
