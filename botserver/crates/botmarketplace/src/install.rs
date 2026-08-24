use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use diesel::prelude::*;
use diesel::sql_types::{Nullable as SqlNullable, Text, Timestamptz, Uuid as SqlUuid};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth;
use crate::blobstore;
use crate::models::{InstallBody, PackageRow, VersionRow};
use crate::MarketplaceService;

fn storage_unavailable(detail: &str) -> (StatusCode, String) {
    tracing::error!("marketplace storage failure: {detail}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Storage backend unavailable".to_string())
}

fn db_error(detail: &str) -> (StatusCode, String) {
    tracing::error!("marketplace DB failure: {detail}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Persistence error".to_string())
}

async fn enforce_consent(
    service: &MarketplaceService,
    bot_id: Uuid,
    slug: &str,
    manifest: &serde_json::Value,
) -> Result<(), (StatusCode, String)> {
    if !service.require_consent {
        return Ok(());
    }
    let checker = match &service.consent_checker {
        Some(c) => c,
        None => return Ok(()),
    };
    let request = serde_json::json!({
        "package_slug": slug,
        "permissions": manifest.get("permissions").cloned().unwrap_or(serde_json::json!([])),
    });
    match checker(bot_id, request).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            "User consent denied for package permissions".to_string(),
        )),
        Err(e) => {
            tracing::error!("consent checker failure for {slug}: {e}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Consent evaluation failed".to_string()))
        }
    }
}

pub async fn install(
    State(service): State<Arc<MarketplaceService>>,
    headers: HeaderMap,
    Path(skill_slug): Path<String>,
    Json(body): Json<InstallBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org_id = auth::jwt_org_id(&headers);
    if !auth::jwt_is_admin(&headers) && org_id.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }

    let mut conn = service.pool.get().map_err(|e| db_error(&format!("DB pool: {e}")))?;

    use crate::schema::skill_packages::dsl::*;
    let package: PackageRow = skill_packages
        .filter(slug.eq(skill_slug.as_str()))
        .select(PackageRow::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| db_error(&format!("Query package: {e}")))?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;

    let is_publisher = org_id.map(|o| package.publisher_org_id == Some(o)).unwrap_or(false);
    let publicly_available = package.visibility == "public" && package.review_status != "rejected";
    if !publicly_available && !is_publisher && !auth::jwt_is_admin(&headers) {
        return Err((StatusCode::NOT_FOUND, "Package not available".to_string()));
    }

    let version_ref = body
        .version
        .clone()
        .or_else(|| package.latest_version.clone())
        .ok_or((StatusCode::CONFLICT, "Package has no published version".to_string()))?;

    let version_row: VersionRow = {
        use crate::schema::skill_versions::dsl::{skill_versions, package_id as pkg_col, version as ver_col};
        skill_versions
            .filter(pkg_col.eq(package.id))
            .filter(ver_col.eq(version_ref))
            .select(VersionRow::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| db_error(&format!("Query version: {e}")))?
            .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?
    };

    drop(conn);
    enforce_consent(&service, body.bot_id, &skill_slug, &version_row.manifest).await?;

    let content = blobstore::get_package(&service.mc_bin, &service.mc_alias, &version_row.object_key)
        .map_err(|e| storage_unavailable(&e))?;

    blobstore::upload_to_bot_bucket(
        &service.mc_bin,
        &service.mc_alias,
        &body.bot_id,
        &format!("skills/{skill_slug}.gbskill"),
        &content,
    )
    .map_err(|e| storage_unavailable(&e))?;

    let marker_json = serde_json::to_vec_pretty(&version_row.manifest)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Manifest serialization failed: {e}")))?;
    blobstore::upload_to_bot_bucket(
        &service.mc_bin,
        &service.mc_alias,
        &body.bot_id,
        &format!("skills/{skill_slug}/manifest.json"),
        &marker_json,
    )
    .map_err(|e| storage_unavailable(&e))?;

    let mut conn = service.pool.get().map_err(|e| db_error(&format!("DB pool: {e}")))?;
    diesel::sql_query(
        r#"INSERT INTO skill_installs
             (id, package_id, version_id, org_id, branch_id, bot_id, installed_by, status, installed_at)
           VALUES ($1, $2, $3, $4, $4, $5, $6, 'installed', $7)"#,
    )
    .bind::<SqlUuid, _>(Uuid::new_v4())
    .bind::<SqlUuid, _>(package.id)
    .bind::<SqlUuid, _>(version_row.id)
    .bind::<SqlNullable<SqlUuid>, _>(org_id)
    .bind::<SqlUuid, _>(body.bot_id)
    .bind::<SqlNullable<SqlUuid>, _>(auth::jwt_user_id(&headers))
    .bind::<Timestamptz, _>(chrono::Utc::now())
    .execute(&mut conn)
    .map_err(|e| db_error(&format!("Insert skill_install: {e}")))?;

    diesel::sql_query(
        "UPDATE skill_packages SET downloads = downloads + 1, updated_at = now() WHERE id = $1",
    )
    .bind::<SqlUuid, _>(package.id)
    .execute(&mut conn)
    .map_err(|e| db_error(&format!("Increment downloads: {e}")))?;

    tracing::info!("Installed skill '{skill_slug}' {} into bot {}", version_row.version, body.bot_id);

    Ok(Json(serde_json::json!({
        "status": "installed",
        "slug": skill_slug,
        "version": version_row.version,
        "bot_id": body.bot_id,
        "objects": [
            format!("skills/{skill_slug}.gbskill"),
            format!("skills/{skill_slug}/manifest.json")
        ],
    })))
}

pub async fn uninstall(
    State(service): State<Arc<MarketplaceService>>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    body: Option<Json<InstallBody>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if !auth::jwt_is_admin(&headers) && auth::jwt_org_id(&headers).is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }
    let InstallBody { bot_id, .. } = body
        .map(|Json(b)| b)
        .ok_or((StatusCode::BAD_REQUEST, "bot_id is required".to_string()))?;

    blobstore::remove_bot_bucket_prefix(&service.mc_bin, &service.mc_alias, &bot_id, &format!("skills/{slug}"))
        .map_err(|e| storage_unavailable(&e))?;

    let mut conn = service.pool.get().map_err(|e| db_error(&format!("DB pool: {e}")))?;
    diesel::sql_query(
        r#"UPDATE skill_installs SET status = 'removed'
           WHERE bot_id = $1 AND package_id IN (SELECT id FROM skill_packages WHERE slug = $2)"#,
    )
    .bind::<SqlUuid, _>(bot_id)
    .bind::<Text, _>(slug.as_str())
    .execute(&mut conn)
    .map_err(|e| db_error(&format!("Update installs: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "removed", "slug": slug, "bot_id": bot_id })))
}
