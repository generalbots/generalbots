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

#[derive(diesel::QueryableByName)]
struct BotScopeRow {
    #[diesel(sql_type = SqlUuid)]
    org_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    branch_id: Uuid,
}

fn bot_scope(service: &MarketplaceService, bot_id: Uuid) -> Result<Option<(Uuid, Uuid)>, String> {
    let mut conn = service.pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let rows: Vec<BotScopeRow> = diesel::sql_query("SELECT org_id, branch_id FROM bots WHERE id = $1")
        .bind::<SqlUuid, _>(bot_id)
        .load(&mut conn)
        .map_err(|e| format!("Query bot scope: {e}"))?;
    Ok(rows.into_iter().next().map(|row| (row.org_id, row.branch_id)))
}

/// Pure two-tenant predicate: a non-admin may only target a bot in their own
/// org, and when the verified token carries a branch it must match the bot's
/// branch too. A caller without an org never owns a bot.
fn bot_in_tenant(
    is_admin: bool,
    caller_org: Option<Uuid>,
    caller_branch: Option<Uuid>,
    bot_org: Uuid,
    bot_branch: Uuid,
) -> bool {
    if is_admin {
        return true;
    }
    match caller_org {
        Some(org) if org == bot_org => match caller_branch {
            Some(branch) => branch == bot_branch,
            None => true,
        },
        _ => false,
    }
}

fn ensure_bot_scope(
    service: &MarketplaceService,
    headers: &HeaderMap,
    bot_id: Uuid,
    is_admin: bool,
) -> Result<(), (StatusCode, String)> {
    if is_admin {
        return Ok(());
    }
    let scope = bot_scope(service, bot_id).map_err(|e| db_error(&e))?;
    let allowed = match scope {
        Some((bot_org, bot_branch)) => bot_in_tenant(
            is_admin,
            auth::jwt_org_id(headers),
            auth::jwt_branch_id(headers),
            bot_org,
            bot_branch,
        ),
        None => false,
    };
    if !allowed {
        return Err((StatusCode::FORBIDDEN, "Bot is outside the caller's organization".to_string()));
    }
    Ok(())
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
    if !blobstore::valid_slug(&skill_slug) {
        return Err((StatusCode::BAD_REQUEST, "Invalid package slug".to_string()));
    }
    let org_id = auth::jwt_org_id(&headers);
    let is_admin = auth::jwt_is_admin(&headers, &service.pool);
    if !is_admin && org_id.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }
    // Never touch a bucket before the caller is proven to own the target bot.
    ensure_bot_scope(&service, &headers, body.bot_id, is_admin)?;

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
    if !publicly_available && !is_publisher && !is_admin {
        return Err((StatusCode::NOT_FOUND, "Package not available".to_string()));
    }

    let version_ref = body
        .version
        .clone()
        .or_else(|| package.latest_version.clone())
        .ok_or((StatusCode::CONFLICT, "Package has no published version".to_string()))?;
    if !blobstore::valid_version(&version_ref) {
        return Err((StatusCode::BAD_REQUEST, "Invalid package version".to_string()));
    }

    let version_row: VersionRow = {
        use crate::schema::skill_versions::dsl::{skill_versions, package_id as pkg_col, version as ver_col};
        skill_versions
            .filter(pkg_col.eq(package.id))
            .filter(ver_col.eq(&version_ref))
            .select(VersionRow::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| db_error(&format!("Query version: {e}")))?
            .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?
    };

    // A tampered version row must not be able to point the installer at an
    // arbitrary marketplace object; the key is derived from slug and version.
    if version_row.object_key != blobstore::package_object_key(&package.slug, &version_ref) {
        return Err((StatusCode::CONFLICT, "Version object key is invalid".to_string()));
    }

    drop(conn);
    let content = blobstore::get_package(&service.mc_bin, &service.mc_alias, &version_row.object_key)
        .map_err(|e| storage_unavailable(&e))?;
    // Verify the signed checksum and the script allowlist BEFORE any write to
    // the bot bucket. An unverifiable or tampered bundle is never installed.
    crate::verify::verify_bundle(&package.slug, &version_ref, &content, &version_row.manifest)
        .map_err(|e| {
            tracing::error!("Refusing to install '{}' {}: {e}", package.slug, version_ref);
            (StatusCode::CONFLICT, "Package failed integrity verification".to_string())
        })?;
    enforce_consent(&service, body.bot_id, &skill_slug, &version_row.manifest).await?;

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
    if !blobstore::valid_slug(&slug) {
        return Err((StatusCode::BAD_REQUEST, "Invalid package slug".to_string()));
    }
    let is_admin = auth::jwt_is_admin(&headers, &service.pool);
    if !is_admin && auth::jwt_org_id(&headers).is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }
    let InstallBody { bot_id, .. } = body
        .map(|Json(b)| b)
        .ok_or((StatusCode::BAD_REQUEST, "bot_id is required".to_string()))?;
    // The path-derived slug and the caller-supplied bot_id are both attacker
    // controlled: validate the slug and the tenant scope before any bucket op.
    ensure_bot_scope(&service, &headers, bot_id, is_admin)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bot_scope_is_enforced_across_two_tenants() {
        let org_a = Uuid::new_v4();
        let org_b = Uuid::new_v4();
        let branch_a1 = Uuid::new_v4();
        let branch_a2 = Uuid::new_v4();
        let bot_a = (org_a, branch_a1);

        // Same org and branch: allowed. Same org, wrong branch: refused.
        assert!(bot_in_tenant(false, Some(org_a), Some(branch_a1), bot_a.0, bot_a.1));
        assert!(!bot_in_tenant(false, Some(org_a), Some(branch_a2), bot_a.0, bot_a.1));

        // A tenant without a branch claim still needs the right org.
        assert!(bot_in_tenant(false, Some(org_a), None, bot_a.0, bot_a.1));
        // A different tenant's bot is refused, as is a caller with no org.
        assert!(!bot_in_tenant(false, Some(org_b), None, bot_a.0, bot_a.1));
        assert!(!bot_in_tenant(false, None, Some(branch_a1), bot_a.0, bot_a.1));
        // A verified administrator is the only cross-tenant exception.
        assert!(bot_in_tenant(true, None, None, bot_a.0, bot_a.1));
    }
}
