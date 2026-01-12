use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub fn configure_products_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/products/items", get(handle_products_items))
        .route("/api/products/services", get(handle_products_services))
        .route("/api/products/pricelists", get(handle_products_pricelists))
        .route(
            "/api/products/stats/total-products",
            get(handle_total_products),
        )
        .route(
            "/api/products/stats/total-services",
            get(handle_total_services),
        )
        .route("/api/products/stats/pricelists", get(handle_total_pricelists))
        .route("/api/products/stats/active", get(handle_active_products))
        .route("/api/products/search", get(handle_products_search))
}

async fn handle_products_items(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<ProductQuery>,
) -> impl IntoResponse {
    Html(
        r#"<div class="products-empty">
            <div class="empty-icon">📦</div>
            <p>No products yet</p>
            <p class="empty-hint">Add your first product to get started</p>
        </div>"#
            .to_string(),
    )
}

async fn handle_products_services(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<ProductQuery>,
) -> impl IntoResponse {
    Html(
        r#"<tr class="empty-row">
        <td colspan="6" class="empty-state">
            <div class="empty-icon">🔧</div>
            <p>No services yet</p>
            <p class="empty-hint">Add services to your catalog</p>
        </td>
    </tr>"#
            .to_string(),
    )
}

async fn handle_products_pricelists(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<ProductQuery>,
) -> impl IntoResponse {
    Html(
        r#"<tr class="empty-row">
        <td colspan="5" class="empty-state">
            <div class="empty-icon">💰</div>
            <p>No price lists yet</p>
            <p class="empty-hint">Create price lists for different customer segments</p>
        </td>
    </tr>"#
            .to_string(),
    )
}

async fn handle_total_products(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("0".to_string())
}

async fn handle_total_services(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("0".to_string())
}

async fn handle_total_pricelists(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("0".to_string())
}

async fn handle_active_products(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Html("0".to_string())
}

async fn handle_products_search(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }
    Html(format!(
        r#"<div class="search-results-empty">
            <p>No results for "{}"</p>
        </div>"#,
        q
    ))
}
