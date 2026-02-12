#![cfg_attr(feature = "mail", allow(unused_imports))]
use axum::{Router, routing::{get, post}};
use std::sync::Arc;

/// Configure admin routes
pub fn configure() -> Router<Arc<crate::core::shared::state::AppState>> {
    use super::admin_config::*;

    Router::new()
        .route("/api/admin/config", get(get_config))
        .route("/api/admin/config", post(update_config))
}
