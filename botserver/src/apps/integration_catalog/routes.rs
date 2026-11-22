use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use botsecurity::auth::AuthenticatedUser;
use botsecurity_core::error_sanitizer::log_and_sanitize_json;
use serde::Deserialize;
use serde_json::{json, Value};

use super::{provider_by_id, search, provider_store, ProviderItem};

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
        // #2346 — admin-only provider CRUD. The editor is enabled only when
        // GET succeeds, so a build without this route stays read-only.
        .route("/api/apps/integrations/providers", get(list_providers).post(create_provider))
        .route(
            "/api/apps/integrations/providers/:id",
            axum::routing::put(update_provider).delete(delete_provider),
        )
}

async fn catalog(Query(filters): Query<CatalogQuery>) -> Json<Value> {
    let base = search(
        filters.q.as_deref(),
        filters.category.as_deref(),
        filters.status.as_deref(),
    );
    let mut value = serde_json::to_value(base).unwrap_or_else(|_| json!({}));
    // #2346 — operator providers are part of the catalog immediately after
    // an editor save; no restart or recompile is required.
    if let Ok(extra) = provider_store::list() {
        if let Some(Value::Array(providers)) = value.get_mut("providers") {
            providers.extend(extra);
        }
    }
    Json(value)
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

fn admin_guard(user: &AuthenticatedUser) -> Result<(), (StatusCode, Json<Value>)> {
    if user.is_admin() || user.is_super_admin() {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "admin_required", "message": "Administrator access required" })),
        ))
    }
}

fn db_error(context: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(log_and_sanitize_json(&message.to_string(), context, None)),
    )
}

async fn list_providers(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    admin_guard(&user)?;
    provider_store::list()
        .map(|rows| Json(json!({ "providers": rows })))
        .map_err(|e| db_error("provider_list", &e))
}

async fn create_provider(
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    admin_guard(&user)?;
    provider_store::upsert(&payload, None)
        .map(Json)
        .map_err(|e| db_error("provider_create", &e))
}

async fn update_provider(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    admin_guard(&user)?;
    provider_store::upsert(&payload, Some(&id))
        .map(Json)
        .map_err(|e| db_error("provider_update", &e))
}

async fn delete_provider(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    admin_guard(&user)?;
    match provider_store::delete(&id) {
        Ok(true) => Ok(Json(json!({ "deleted": id }))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "provider_not_found", "message": "Provider not found" })),
        )),
        Err(e) => Err(db_error("provider_delete", &e)),
    }
}
