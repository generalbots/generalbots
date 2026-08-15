use axum::{Json, extract::State};
use std::sync::Arc;
use log::{info, warn};
use botcore::shared::state::AppState;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::QueryDsl;

pub async fn handle_get_organization(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let auth_service = state.auth_service.as_ref().ok_or_else(|| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "No auth service"
        })))
    })?.lock().await;

    match auth_service.list_organizations().await {
        Ok(data) => {
            let orgs = data.as_array().cloned().unwrap_or_default();
            if let Some(org) = orgs.first() {
                Ok(Json(org.clone()))
            } else {
                Ok(Json(serde_json::json!({
                    "name": "Default Organization",
                    "id": "default",
                    "description": ""
                })))
            }
        }
        Err(e) => {
            warn!("Failed to list organizations: {}", e);
            Ok(Json(serde_json::json!({
                "name": "Default Organization",
                "id": "default",
                "description": ""
            })))
        }
    }
}

pub async fn handle_get_org_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let stack = botcore::shared::utils::get_stack_path();
    let config_path = format!("{}/conf/directory/org-settings.json", stack);

    let settings = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(Json(settings))
}

pub async fn handle_get_org_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut users_total: i64 = 0;
    let mut bots_total: i64 = 0;

    if let Some(auth) = state.auth_service.as_ref() {
        let auth_service = auth.lock().await;
        if let Ok(data) = auth_service.list_users(1000, 0).await {
            users_total = data.as_array().map(|a| a.len() as i64).unwrap_or(0);
        }
    }

    if let Ok(mut conn) = state.conn.get() {
        use botcore::shared::models::schema::bots::dsl::*;
        if let Ok(count) = bots.count().get_result::<i64>(&mut conn) {
            bots_total = count;
        }
    }

    let mut kb_total: i64 = 0;
    let mut storage_bytes: i64 = 0;

    if let Ok(mut conn) = state.conn.get() {
        use botcore::shared::models::schema::drive_files::dsl::*;
        if let Ok(count) = drive_files
            .filter(file_type.eq("gbkb"))
            .count()
            .get_result::<i64>(&mut conn)
        {
            kb_total = count;
        }
        if let Ok(bytes) = drive_files
            .filter(file_size.is_not_null())
            .select(diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(file_size), 0)"))
            .first::<i64>(&mut conn)
        {
            storage_bytes = bytes;
        }
    }

    let storage_mb_val = storage_bytes / 1_048_576;

    Ok(Json(serde_json::json!({
        "users": { "used": users_total, "limit": 50 },
        "bots": { "used": bots_total, "limit": 20 },
        "kb_documents": { "used": kb_total, "limit": 500 },
        "storage_mb": { "used": storage_mb_val, "limit": 5120 }
    })))
}

pub async fn handle_delete_organization(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization deletion requested");

    let auth = state.auth_service.as_ref().ok_or_else(|| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "No auth service"
        })))
    })?;

    let auth_service = auth.lock().await;

    let orgs_data = auth_service.list_organizations().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Failed to list organizations: {}", e)
        }))))?;

    let orgs = orgs_data.as_array().cloned().unwrap_or_default();
    let org_id = orgs.first()
        .and_then(|o| o.get("id").or_else(|| o.get("orgId")))
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    if org_id == "default" {
        return Err((axum::http::StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Cannot delete the default organization"
        }))));
    }

    let _ = auth_service.http_delete(format!("{}/v2/organizations/{}", auth_service.api_url(), org_id)).await;

    let stack = botcore::shared::utils::get_stack_path();
    let settings_path = std::path::PathBuf::from(format!("{}/conf/directory/org-settings.json", stack));
    let _ = std::fs::remove_file(settings_path);

    Ok(Json(serde_json::json!({"success": true, "message": format!("Organization {} deleted", org_id)})))
}

pub async fn handle_get_org_audit(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let stack = botcore::shared::utils::get_stack_path();
    let log_path = std::path::PathBuf::from(format!("{}/conf/directory/audit-log.json", stack));

    let entries: Vec<serde_json::Value> = std::fs::read_to_string(&log_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let recent: Vec<serde_json::Value> = entries.into_iter().rev().take(50).collect();

    Ok(Json(serde_json::json!({
        "entries": recent,
        "total": recent.len()
    })))
}

pub async fn handle_export_org_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization data export requested");

    let mut export = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "users": [],
        "bots": [],
        "settings": {}
    });

    if let Some(auth) = state.auth_service.as_ref() {
        let auth_service = auth.lock().await;
        if let Ok(data) = auth_service.list_users(1000, 0).await {
            export["users"] = data;
        }
    }

    if let Ok(mut conn) = state.conn.get() {
        use botcore::shared::models::schema::bots::dsl::*;
        if let Ok(bot_list) = bots.limit(100).load::<botcore::shared::models::core::Bot>(&mut conn) {
            let bot_names: Vec<serde_json::Value> = bot_list.iter().map(|b| {
                serde_json::json!({"id": b.id.to_string(), "name": b.name})
            }).collect();
            export["bots"] = serde_json::json!(bot_names);
        }
    }

    let stack = botcore::shared::utils::get_stack_path();
    let settings_path = format!("{}/conf/directory/org-settings.json", stack);
    if let Ok(settings_str) = std::fs::read_to_string(&settings_path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&settings_str) {
            export["settings"] = settings;
        }
    }

    let export_path = format!("{}/tmp/org-export-{}.json", stack, chrono::Utc::now().timestamp());
    let _ = std::fs::write(&export_path, serde_json::to_string_pretty(&export).unwrap_or_default());

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Export complete",
        "download_url": format!("/api/files/download?path={}", export_path)
    })))
}

pub async fn handle_update_organization(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization settings update: {:?}", body);

    let stack = botcore::shared::utils::get_stack_path();
    let config_path = format!("{}/conf/directory/org-settings.json", stack);

    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("Failed to create config directory: {}", e);
        }
    }

    let existing = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let mut merged = existing;
    if let (Some(obj), Some(patch)) = (merged.as_object_mut(), body.as_object()) {
        for (k, v) in patch {
            obj.insert(k.clone(), v.clone());
        }
    }

    if let Err(e) = std::fs::write(&config_path, serde_json::to_string_pretty(&merged).unwrap_or_default()) {
        log::error!("Failed to save organization settings: {}", e);
        return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to save settings"
        }))));
    }

    append_audit_log("settings_updated", &body.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default());

    Ok(Json(serde_json::json!({"success": true})))
}

fn append_audit_log(action: &str, detail: &str) {
    let stack = botcore::shared::utils::get_stack_path();
    let log_path = format!("{}/conf/directory/audit-log.json", stack);

    if let Some(parent) = std::path::Path::new(&log_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut entries: Vec<serde_json::Value> = std::fs::read_to_string(&log_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    entries.push(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "actor": "admin",
        "action": action,
        "detail": detail
    }));

    if entries.len() > 500 {
        entries = entries.split_off(entries.len() - 500);
    }

    let _ = std::fs::write(&log_path, serde_json::to_string_pretty(&entries).unwrap_or_default());
}

pub async fn handle_update_organization_contact(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization contact update: {:?}", body);
    let result = handle_update_organization(State(state), Json(body.clone())).await?;
    append_audit_log("contact_updated", &body.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default());
    Ok(result)
}

pub async fn handle_update_organization_branding(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Organization branding update: {:?}", body);
    let result = handle_update_organization(State(state), Json(body.clone())).await?;
    append_audit_log("branding_updated", &body.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>().join(",")).unwrap_or_default());
    Ok(result)
}

#[derive(serde::Deserialize)]
pub struct Office365MigrationRequest {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub sync_mode: Option<String>,
}

pub async fn handle_office365_migration(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<Office365MigrationRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Office 365 migration requested for tenant: {}", req.tenant_id);

    let mode = match req.sync_mode.as_deref() {
        Some("delta") => crate::directory::scim::sync::SyncMode::Delta,
        _ => crate::directory::scim::sync::SyncMode::Full,
    };

    let config = crate::directory::scim::sync::AzureAdConfig {
        tenant_id: req.tenant_id,
        client_id: req.client_id,
        client_secret: req.client_secret,
        sync_mode: mode,
    };

    let syncer = crate::directory::scim::sync::AzureAdSyncer::new(config);

    let auth_service = _state.auth_service.as_ref().ok_or_else(|| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "No auth service available"
        })))
    })?.lock().await;

    match syncer.sync(&*auth_service).await {
        Ok(result) => {
            info!("Office 365 migration complete: {:?}", result);
            Ok(Json(serde_json::json!({
                "success": true,
                "groups_created": result.groups_created,
                "groups_updated": result.groups_updated,
                "users_mapped": result.users_mapped,
                "users_created": result.users_created,
                "users_updated": result.users_updated,
                "errors": result.errors,
                "duration_ms": result.duration_ms
            })))
        }
        Err(e) => {
            log::error!("Office 365 migration failed: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Migration failed: {}", e)
                }))
            ))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateOrgRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
}

pub async fn handle_create_organization(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<CreateOrgRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let name = form.name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Organization name is required"
        }))));
    }

    let slug = form
        .slug
        .unwrap_or_else(|| name.to_lowercase().replace(' ', "-"));
    let org_id = uuid::Uuid::new_v4();

    let mut conn = state.conn.get().map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Database unavailable: {e}")
        })))
    })?;

    // Resolve a real tenant row — the FK requires an existing tenants.id
    // (fix #840: the previous code bound Uuid::nil(), which never exists,
    // making every insert fail silently).
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct TenantRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
    }
    let tenant_id: uuid::Uuid = diesel::sql_query(
        "SELECT id FROM tenants ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<TenantRow>(&mut conn)
    .optional()
    .map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Tenant lookup failed: {e}")
        })))
    })?
    .map(|r| r.id)
    .ok_or((axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
        "error": "No tenant exists — run migrations/bootstrap first"
    }))))?;

    use diesel::RunQueryDsl;
    let inserted = diesel::sql_query(
        "INSERT INTO organizations (org_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(&org_id)
    .bind::<diesel::sql_types::Uuid, _>(&tenant_id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Organization insert failed: {e}")
        })))
    })?;

    if inserted == 0 {
        // Slug already exists (ON CONFLICT DO NOTHING) — resolve the existing row.
        let existing: TenantRow = diesel::sql_query(
            "SELECT org_id AS id FROM organizations WHERE slug = $1 LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(&slug)
        .get_result::<TenantRow>(&mut conn)
        .map_err(|e| {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Organization lookup failed: {e}")
            })))
        })?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "org_id": existing.id,
            "name": name,
            "slug": slug,
            "duplicate": true,
        })));
    }

    // Create a default branch so the org has a usable workspace scope.
    let branch_id = uuid::Uuid::new_v4();
    let branch_slug = format!("{slug}-default");
    let _ = diesel::sql_query(
        "INSERT INTO branches (id, org_id, tenant_id, slug, name) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (org_id, slug) DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(&branch_id)
    .bind::<diesel::sql_types::Uuid, _>(&org_id)
    .bind::<diesel::sql_types::Uuid, _>(&tenant_id)
    .bind::<diesel::sql_types::Text, _>(&branch_slug)
    .bind::<diesel::sql_types::Text, _>(&name)
    .execute(&mut conn);

    append_audit_log("organization_created", &name);

    Ok(Json(serde_json::json!({
        "success": true,
        "org_id": org_id,
        "branch_id": branch_id,
        "name": name,
        "slug": slug,
    })))
}
