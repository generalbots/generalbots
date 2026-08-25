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
use crate::models::{PackageRow, PublishBody};
use crate::MarketplaceService;

#[derive(diesel::QueryableByName, Debug)]
struct PackageIdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

fn storage_unavailable(detail: &str) -> (StatusCode, String) {
    tracing::error!("marketplace storage failure: {detail}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Storage backend unavailable".to_string())
}

pub(crate) fn decode_content(body: &PublishBody) -> Result<Vec<u8>, (StatusCode, String)> {
    crate::b64::decode_flexible(&body.content_base64)
        .filter(|c| !c.is_empty())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "content_base64 must be non-empty base64".to_string(),
        ))
}

pub(crate) fn validate_body(body: &PublishBody) -> Result<(), (StatusCode, String)> {
    if !blobstore::valid_slug(&body.slug) {
        return Err((StatusCode::BAD_REQUEST, "Invalid slug".to_string()));
    }
    if body.name.trim().is_empty() || body.name.len() > 160 {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }
    if !blobstore::valid_version(&body.version) {
        return Err((StatusCode::BAD_REQUEST, "Invalid version".to_string()));
    }
    if let Some(glyph) = &body.icon_glyph {
        if glyph.chars().count() > 8 {
            return Err((StatusCode::BAD_REQUEST, "icon_glyph too long".to_string()));
        }
    }
    Ok(())
}

pub(crate) fn upsert_package_and_version(
    conn: &mut PgConnection,
    body: &PublishBody,
    publisher_org: Option<Uuid>,
    publisher_name: Option<String>,
) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let tags_text = body.tags.to_string();
    let visibility = body.effective_visibility();

    let package_id: PackageIdRow = diesel::sql_query(
        r#"INSERT INTO skill_packages
             (id, slug, name, description, latest_version, publisher_org_id, publisher_name,
              visibility, review_status, downloads, icon_glyph, tags, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'auto', 0, $9, $10::jsonb, $11, $11)
           ON CONFLICT (slug) DO UPDATE SET
             name = EXCLUDED.name,
             description = EXCLUDED.description,
             latest_version = EXCLUDED.latest_version,
             publisher_org_id = COALESCE(EXCLUDED.publisher_org_id, skill_packages.publisher_org_id),
             publisher_name = EXCLUDED.publisher_name,
             icon_glyph = EXCLUDED.icon_glyph,
             tags = EXCLUDED.tags,
             updated_at = EXCLUDED.updated_at
           RETURNING id"#,
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(body.slug.as_str())
    .bind::<Text, _>(body.name.trim())
    .bind::<SqlNullable<Text>, _>(body.description.as_deref())
    .bind::<Text, _>(body.version.as_str())
    .bind::<SqlNullable<SqlUuid>, _>(publisher_org)
    .bind::<SqlNullable<Text>, _>(publisher_name.as_deref())
    .bind::<Text, _>(visibility)
    .bind::<SqlNullable<Text>, _>(body.icon_glyph.as_deref())
    .bind::<Text, _>(tags_text.as_str())
    .bind::<Timestamptz, _>(now)
    .get_result(conn)
    .map_err(|e| format!("Upsert skill_package: {e}"))?;

    let now = chrono::Utc::now();
    diesel::sql_query(
        r#"INSERT INTO skill_versions
             (id, package_id, version, manifest, object_key, changelog, created_at)
           VALUES ($1, $2, $3, $4::jsonb, $5, $6, $7)
           ON CONFLICT (package_id, version) DO UPDATE SET
             manifest = EXCLUDED.manifest,
             object_key = EXCLUDED.object_key,
             changelog = EXCLUDED.changelog"#,
    )
    .bind::<SqlUuid, _>(Uuid::new_v4())
    .bind::<SqlUuid, _>(package_id.id)
    .bind::<Text, _>(body.version.as_str())
    .bind::<Text, _>(body.manifest.to_string().as_str())
    .bind::<Text, _>(blobstore::package_object_key(&body.slug, &body.version))
    .bind::<SqlNullable<Text>, _>(body.changelog.as_deref())
    .bind::<Timestamptz, _>(now)
    .execute(conn)
    .map_err(|e| format!("Upsert skill_version: {e}"))?;

    Ok(package_id.id)
}

pub async fn publish(
    State(service): State<Arc<MarketplaceService>>,
    headers: HeaderMap,
    Json(body): Json<PublishBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    validate_body(&body)?;

    let admin = auth::jwt_is_admin(&headers);
    let org_id = auth::jwt_org_id(&headers);
    if !admin && org_id.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }

    let content = decode_content(&body)?;

    let mut conn = service.pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    use crate::schema::skill_packages::dsl::{skill_packages, slug as slug_col, publisher_org_id, id as pid};
    let existing_owner: Option<(Uuid, Option<Uuid>)> = skill_packages
        .filter(slug_col.eq(&body.slug))
        .select((pid, publisher_org_id))
        .first(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    if let Some((_, owner)) = &existing_owner {
        let is_publisher = org_id.map(|o| Some(o) == *owner).unwrap_or(false);
        if !admin && !is_publisher {
            return Err((
                StatusCode::FORBIDDEN,
                "Only the publisher or an administrator may update this package".to_string(),
            ));
        }
    }

    blobstore::put_package(
        &service.mc_bin,
        &service.mc_alias,
        &body.slug,
        &body.version,
        &content,
    )
    .map_err(|e| storage_unavailable(&e))?;

    let publisher_name = auth::jwt_claims(&headers)
        .and_then(|c| c.get("name").or_else(|| c.get("email")).and_then(|v| v.as_str()).map(|s| s.to_string()));

    let package_id = upsert_package_and_version(&mut conn, &body, org_id, publisher_name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Persist failed: {e}")))?;

    tracing::info!("Published skill '{}' version {} by org {:?}", body.slug, body.version, org_id);

    Ok(Json(serde_json::json!({
        "status": "published",
        "package_id": package_id,
        "slug": body.slug,
        "version": body.version,
        "visibility": body.effective_visibility(),
        "review_status": "auto",
    })))
}

pub async fn unpublish(
    State(service): State<Arc<MarketplaceService>>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = auth::jwt_is_admin(&headers);
    let org_id = auth::jwt_org_id(&headers);
    if !admin && org_id.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }

    let mut conn = service.pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    use crate::schema::skill_packages::dsl::{skill_packages, slug as slug_col, visibility as vis_col, updated_at as upd_col};
    let target: PackageRow = skill_packages
        .filter(slug_col.eq(&slug))
        .select(PackageRow::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;

    let is_publisher = org_id.map(|o| target.publisher_org_id == Some(o)).unwrap_or(false);
    if !admin && !is_publisher {
        return Err((StatusCode::FORBIDDEN, "Not authorized to unpublish this package".to_string()));
    }

    let updated = diesel::update(skill_packages.filter(slug_col.eq(&slug)))
        .set((vis_col.eq("private"), upd_col.eq(chrono::Utc::now())))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {e}")))?;

    if updated == 0 {
        return Err((StatusCode::NOT_FOUND, "Package not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "unpublished", "slug": slug, "visibility": "private" })))
}

pub async fn my_packages(
    State(service): State<Arc<MarketplaceService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin = auth::jwt_is_admin(&headers);
    let org_id = auth::jwt_org_id(&headers);
    if !admin && org_id.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }

    let mut conn = service.pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;

    use crate::schema::skill_packages::dsl::*;

    let rows = match org_id {
        Some(org) => skill_packages
            .filter(publisher_org_id.eq(org))
            .order(updated_at.desc())
            .limit(50)
            .select(PackageRow::as_select())
            .load(&mut conn),
        None => skill_packages
            .order(updated_at.desc())
            .limit(50)
            .select(PackageRow::as_select())
            .load(&mut conn),
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    Ok(Json(serde_json::json!({ "items": rows, "count": rows.len() })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_body() -> PublishBody {
        serde_json::from_value(serde_json::json!({
            "slug": "test-skill",
            "name": "Test Skill",
            "version": "1.0.0",
            "content_base64": crate::b64::encode_standard(b"hello"),
        }))
        .unwrap()
    }

    #[test]
    fn validates_good_body() {
        assert!(validate_body(&base_body()).is_ok());
    }

    #[test]
    fn rejects_bad_slug_and_version() {
        let mut b = base_body();
        b.slug = "../bad".into();
        assert!(validate_body(&b).is_err());
        let mut b = base_body();
        b.version = "".into();
        assert!(validate_body(&b).is_err());
    }

    #[test]
    fn decodes_content_and_rejects_garbage_or_empty() {
        assert_eq!(decode_content(&base_body()).unwrap(), b"hello".to_vec());
        let mut b = base_body();
        b.content_base64 = "!!!not-base64!!!".into();
        assert!(decode_content(&b).is_err());
        b.content_base64 = String::new();
        assert!(decode_content(&b).is_err());
    }

    #[test]
    fn visibility_defaults_to_public() {
        assert_eq!(base_body().effective_visibility(), "public");
        let mut b = base_body();
        b.visibility = Some("private".into());
        assert_eq!(b.effective_visibility(), "private");
    }
}
