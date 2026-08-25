//! Universal search across all suite applications.
//!
//! `GET /api/ui/search?q=<query>` scans the primary entity tables (people,
//! CRM contacts, products, services, tickets, KB documents, drive objects,
//! bots) using PostgreSQL full-text search (migration 6.5.37-unified-search-fts
//! adds GIN indexes over the searchable column expressions). Results are
//! ranked with `ts_rank`; an `ILIKE` fallback catches partial/substring
//! matches. Table and column names are hardcoded constants — never derived
//! from user input — and all values are bound as query parameters.

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use botcore::shared::state::AppState;
use diesel::RunQueryDsl;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

const MAX_RESULTS_PER_SOURCE: i64 = 8;
const MAX_QUERY_LEN: usize = 128;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

#[derive(Debug)]
struct EntitySource {
    app: &'static str,
    entity_type: &'static str,
    table: &'static str,
    id_column: &'static str,
    searchable_columns: &'static [&'static str],
    subtitle_columns: &'static [&'static str],
    url: &'static str,
    /// Optional extra SQL predicate (e.g. `is_active = true`), appended to
    /// the WHERE clause before the search terms (#1186 connector facet).
    filter: Option<&'static str>,
    /// Org-scoping column (e.g. `org_id`) when the table is tenant-scoped.
    /// When set, results are restricted to the caller's org (#1179
    /// permissioned search). `None` for global tables (connectors).
    org_column: Option<&'static str>,
}

const SOURCES: &[EntitySource] = &[
    EntitySource {
        app: "people",
        entity_type: "person",
        table: "people",
        id_column: "id",
        searchable_columns: &["first_name", "last_name", "email"],
        subtitle_columns: &["email"],
        url: "/suite/people/people.html",
        filter: None,
        org_column: Some("org_id"),
    },
    EntitySource {
        app: "crm",
        entity_type: "contact",
        table: "crm_contacts",
        id_column: "id",
        searchable_columns: &["first_name", "last_name", "email"],
        subtitle_columns: &["email"],
        url: "/suite/crm/crm.html",
        filter: None,
        org_column: Some("org_id"),
    },
    EntitySource {
        app: "products",
        entity_type: "product",
        table: "products",
        id_column: "id",
        searchable_columns: &["name", "sku", "description"],
        subtitle_columns: &["sku"],
        url: "/suite/products/products.html",
        filter: None,
        org_column: Some("org_id"),
    },
    EntitySource {
        app: "tickets",
        entity_type: "ticket",
        table: "support_tickets",
        id_column: "id",
        searchable_columns: &["subject", "description"],
        subtitle_columns: &["status"],
        url: "/suite/tickets/tickets.html",
        filter: None,
        org_column: Some("org_id"),
    },
    EntitySource {
        app: "research",
        entity_type: "document",
        table: "kb_documents",
        id_column: "id",
        searchable_columns: &["file_path", "collection_name"],
        subtitle_columns: &["collection_name"],
        url: "/suite/research/research.html",
        filter: None,
        org_column: None,
    },
    EntitySource {
        app: "drive",
        entity_type: "file",
        table: "drive_files",
        id_column: "id",
        searchable_columns: &["name", "file_path"],
        subtitle_columns: &["file_path"],
        url: "/suite/drive/drive.html",
        filter: None,
        org_column: None,
    },
    EntitySource {
        app: "admin",
        entity_type: "bot",
        table: "bots",
        id_column: "id",
        searchable_columns: &["name", "description"],
        subtitle_columns: &["description"],
        url: "/suite/admin/index.html",
        filter: None,
        org_column: Some("org_id"),
    },
    // #1186 — connector sources facet: only active connectors surface.
    EntitySource {
        app: "integrations",
        entity_type: "connector",
        table: "connectors",
        id_column: "id",
        searchable_columns: &["name", "connector_type", "description"],
        subtitle_columns: &["connector_type"],
        url: "/suite/integrations/integrations.html",
        filter: Some("is_active = true"),
        org_column: None,
    },
];

#[derive(diesel::QueryableByName)]
struct SearchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    subtitle: String,
}

/// `COALESCE(col1::text,'') || ' ' || COALESCE(col2::text,'') ...`
fn concat_expr(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|c| format!("COALESCE({c}::text,'')"))
        .collect::<Vec<_>>()
        .join(" || ' ' || ")
}

/// The GIN-indexed expression the migration uses — must match
/// `migrations/6.5.37-unified-search-fts/up.sql` exactly.
fn fts_vector_expr(source: &EntitySource) -> String {
    format!(
        "to_tsvector('english', {})",
        concat_expr(source.searchable_columns)
    )
}

fn build_query(source: &EntitySource, org_clause: Option<&str>) -> String {
    let title_expr = concat_expr(source.searchable_columns);
    let subtitle_expr = concat_expr(source.subtitle_columns);
    let fts = fts_vector_expr(source);
    let filter = source
        .filter
        .map(|f| format!(" AND ({f})"))
        .unwrap_or_default();
    let org = org_clause.map(|c| format!(" AND {c}")).unwrap_or_default();
    format!(
        "SELECT {id} AS id, {title} AS title, {subtitle} AS subtitle \
         FROM {table} \
         WHERE ({fts} @@ websearch_to_tsquery('english', $1) \
            OR LOWER({title_expr}) LIKE $2){filter}{org} \
         ORDER BY ts_rank({fts}, websearch_to_tsquery('english', $1)) DESC, id \
         LIMIT {limit}",
        id = source.id_column,
        title = title_expr,
        subtitle = subtitle_expr,
        fts = fts,
        table = source.table,
        filter = filter,
        org = org,
        limit = MAX_RESULTS_PER_SOURCE,
    )
}

fn row_to_json(source: &EntitySource, row: &SearchRow) -> serde_json::Value {
    json!({
        "app": source.app,
        "type": source.entity_type,
        "id": row.id.to_string(),
        "title": row.title.trim(),
        "subtitle": row.subtitle.trim(),
        "url": source.url,
    })
}

pub async fn handle_unified_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
    user: Option<axum::extract::Extension<crate::security::auth_api::types::AuthenticatedUser>>,
) -> impl IntoResponse {
    let trimmed = params.q.unwrap_or_default();
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return Json(json!({"results": []}));
    }
    let query = &trimmed.chars().take(MAX_QUERY_LEN).collect::<String>();
    let fts_pattern = query.to_string();
    let ilike_pattern = format!("%{}%", query.to_lowercase());

    // #1179 — permissioned search: tenant-scoped sources are restricted to
    // the caller's org. Admins and anonymous requests see everything.
    let org_id = user.and_then(|u| {
        let is_admin = u
            .roles
            .iter()
            .any(|r| matches!(r, crate::security::auth_api::types::Role::Admin));
        if is_admin {
            None
        } else {
            u.organization_id
        }
    });

    let pool = state.conn.clone();

    let results = tokio::task::spawn_blocking(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("unified search: DB pool error: {e}");
                return Vec::new();
            }
        };

        let mut out: Vec<serde_json::Value> = Vec::new();
        for source in SOURCES {
            // Column is a hardcoded constant; the value is a Uuid parsed by
            // the trusted auth layer — no user-controlled SQL reaches here.
            let org_clause = match (source.org_column, org_id) {
                (Some(col), Some(id)) => Some(format!("{col} = '{id}'")),
                _ => None,
            };
            let sql = build_query(source, org_clause.as_deref());
            match diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Text, _>(&fts_pattern)
                .bind::<diesel::sql_types::Text, _>(&ilike_pattern)
                .load::<SearchRow>(&mut conn)
            {
                Ok(rows) => {
                    for row in rows {
                        out.push(row_to_json(source, &row));
                    }
                }
                Err(e) => {
                    log::warn!(
                        "unified search: query failed for table {}: {e}",
                        source.table
                    );
                }
            }
        }
        out
    })
    .await
    .unwrap_or_default();

    Json(json!({ "results": results }))
}

/// Registers the universal search endpoint.
pub fn configure_unified_search_routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route(
        "/api/ui/search",
        axum::routing::get(handle_unified_search),
    )
}
