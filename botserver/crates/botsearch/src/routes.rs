use axum::routing::{get, post};
use axum::Router;

use crate::handlers::SearchAppState;
use crate::handlers::{
    get_settings, get_stats, reindex_all, reindex_source, save_settings, search_entities,
    search_entity,
};

pub fn configure_search_routes() -> Router<SearchAppState> {
    Router::new()
        .route("/api/search/settings", get(get_settings).post(save_settings))
        .route("/api/search/stats", get(get_stats))
        .route("/api/search/reindex", post(reindex_all))
        .route("/api/search/reindex/:source", post(reindex_source))
        .route("/api/search/entities", get(search_entities))
        .route("/api/search/entity", get(search_entity))
}
