use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::catalog;
use crate::install;
use crate::publish;
use crate::MarketplaceService;

pub fn configure_routes() -> Router<Arc<MarketplaceService>> {
    Router::new()
        .route(
            "/api/marketplace/skills",
            get(catalog::list_skills).post(publish::publish),
        )
        .route("/api/marketplace/skills/my", get(publish::my_packages))
        .route(
            "/api/marketplace/skills/:slug",
            get(catalog::skill_detail).delete(publish::unpublish),
        )
        .route(
            "/api/marketplace/skills/:slug/install",
            post(install::install).delete(install::uninstall),
        )
}
