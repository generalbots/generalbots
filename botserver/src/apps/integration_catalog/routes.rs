use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{provider_by_id, search, ProviderItem};

#[derive(Debug, Default, Deserialize)]
struct CatalogQuery {
    q: Option<String>,
    category: Option<String>,
    status: Option<String>,
}

pub(super) fn register<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    router
        .route("/api/apps/integrations/catalog", get(catalog))
        .route("/api/apps/integrations/catalog/:provider", get(provider))
}

async fn catalog(Query(filters): Query<CatalogQuery>) -> Json<super::CatalogResponse> {
    Json(search(
        filters.q.as_deref(),
        filters.category.as_deref(),
        filters.status.as_deref(),
    ))
}

/// #2346 — the 404 body uses the same sanitized envelope as the other catalog
/// routes (`error`/`code` machine token, `message` human copy). The requested
/// id is deliberately not echoed back: path input does not belong in an error
/// body. A hint plus the canonical id list makes a stale deep link fixable
/// without guessing and without a code change.
pub(super) fn not_found_diagnostic() -> Value {
    json!({
        "error": "provider_not_found",
        "code": "provider_not_found",
        "message": "Integration provider not found.",
        "hint": "List valid provider ids with GET /api/apps/integrations/catalog.",
        "valid_provider_ids": super::provider_ids(),
    })
}

async fn provider(
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderItem>, (StatusCode, Json<Value>)> {
    provider_by_id(&provider_id)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(not_found_diagnostic())))
}
