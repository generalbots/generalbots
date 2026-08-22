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

async fn provider(
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderItem>, (StatusCode, Json<Value>)> {
    provider_by_id(&provider_id).map(Json).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "integration provider not found" })),
        )
    })
}
