use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use diesel::sql_types::{BigInt as SqlBigInt, Text as SqlText};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::{PackageRow, VersionRow};
use crate::MarketplaceService;

pub const CATALOG_PAGE_LIMIT: i64 = 50;

fn db_error(detail: &str) -> (StatusCode, String) {
    tracing::error!("marketplace DB failure: {detail}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Persistence error".to_string())
}

#[derive(diesel::QueryableByName, Debug)]
struct CatalogRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = SqlText)]
    slug: String,
    #[diesel(sql_type = SqlText)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<SqlText>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<SqlText>)]
    latest_version: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    publisher_org_id: Option<uuid::Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<SqlText>)]
    publisher_name: Option<String>,
    #[diesel(sql_type = SqlText)]
    visibility: String,
    #[diesel(sql_type = SqlText)]
    review_status: String,
    #[diesel(sql_type = SqlBigInt)]
    downloads: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<SqlText>)]
    icon_glyph: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    tags: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CatalogRow> for PackageRow {
    fn from(r: CatalogRow) -> Self {
        Self {
            id: r.id,
            slug: r.slug,
            name: r.name,
            description: r.description,
            latest_version: r.latest_version,
            publisher_org_id: r.publisher_org_id,
            publisher_name: r.publisher_name,
            visibility: r.visibility,
            review_status: r.review_status,
            downloads: r.downloads,
            icon_glyph: r.icon_glyph,
            tags: r.tags,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// `GET /api/marketplace/skills?q=&tag=&offset=` — anonymous public catalog.
pub async fn list_skills(
    State(service): State<Arc<MarketplaceService>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let term = params.get("q").map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let tag = params.get("tag").map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let offset: i64 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        .max(0);

    let mut conn = service.pool.get().map_err(|e| db_error(&format!("DB pool: {e}")))?;

    if let Some(tag_pattern) = tag {
        let like = format!("%{tag_pattern}%");
        let rows: Vec<CatalogRow> = diesel::sql_query(
            r#"SELECT id, slug, name, description, latest_version, publisher_org_id,
                      publisher_name, visibility, review_status, downloads, icon_glyph,
                      tags, created_at, updated_at
               FROM skill_packages
               WHERE visibility = 'public' AND review_status <> 'rejected'
                 AND tags::text ILIKE $1
               ORDER BY downloads DESC, updated_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind::<SqlText, _>(like)
        .bind::<SqlBigInt, _>(CATALOG_PAGE_LIMIT)
        .bind::<SqlBigInt, _>(offset)
        .load(&mut conn)
        .map_err(|e| db_error(&format!("Query catalog: {e}")))?;
        let items: Vec<PackageRow> = rows.into_iter().map(Into::into).collect();
        return Ok(Json(serde_json::json!({
            "items": items, "count": items.len(),
            "limit": CATALOG_PAGE_LIMIT, "offset": offset,
        })));
    }

    use crate::schema::skill_packages::dsl::*;
    let mut query = skill_packages
        .filter(visibility.eq("public"))
        .filter(review_status.ne("rejected"))
        .into_boxed();
    if let Some(t) = term {
        let pattern = format!("%{t}%");
        query = query.filter(slug.ilike(&pattern).or(name.ilike(&pattern)));
    }

    let items: Vec<PackageRow> = query
        .order((downloads.desc(), updated_at.desc()))
        .limit(CATALOG_PAGE_LIMIT)
        .offset(offset)
        .select(PackageRow::as_select())
        .load(&mut conn)
        .map_err(|e| db_error(&format!("Query catalog: {e}")))?;

    Ok(Json(serde_json::json!({
        "items": items, "count": items.len(),
        "limit": CATALOG_PAGE_LIMIT, "offset": offset,
    })))
}

/// `GET /api/marketplace/skills/{slug}` — anonymous package detail with latest manifest.
pub async fn skill_detail(
    State(service): State<Arc<MarketplaceService>>,
    Path(skill_slug): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool.get().map_err(|e| db_error(&format!("DB pool: {e}")))?;

    use crate::schema::skill_packages::dsl::*;
    let package: PackageRow = skill_packages
        .filter(slug.eq(&skill_slug))
        .filter(visibility.eq("public"))
        .filter(review_status.ne("rejected"))
        .select(PackageRow::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| db_error(&format!("Query package: {e}")))?
        .ok_or((StatusCode::NOT_FOUND, "Package not found".to_string()))?;

    let latest = package.latest_version.clone().ok_or((
        StatusCode::NOT_FOUND,
        "Package has no published version".to_string(),
    ))?;

    let version_row: VersionRow = {
        use crate::schema::skill_versions::dsl::{skill_versions, package_id as pkg_col, version as ver_col};
        skill_versions
            .filter(pkg_col.eq(package.id))
            .filter(ver_col.eq(latest))
            .select(VersionRow::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| db_error(&format!("Query version: {e}")))?
            .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?
    };

    Ok(Json(serde_json::json!({
        "id": package.id,
        "slug": package.slug,
        "name": package.name,
        "description": package.description,
        "publisher_org_id": package.publisher_org_id,
        "publisher_name": package.publisher_name,
        "downloads": package.downloads,
        "icon_glyph": package.icon_glyph,
        "tags": package.tags,
        "latest_version": package.latest_version,
        "changelog": version_row.changelog,
        "manifest": version_row.manifest,
        "object_key": version_row.object_key,
        "updated_at": package.updated_at,
    })))
}
