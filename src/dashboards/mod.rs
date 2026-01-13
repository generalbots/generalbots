pub mod error;
pub mod handlers;
pub mod storage;
pub mod types;
pub mod ui;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::shared::state::AppState;

pub use error::DashboardsError;
pub use handlers::*;
pub use storage::*;
pub use types::*;

pub fn configure_dashboards_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/dashboards", get(handle_list_dashboards))
        .route("/api/dashboards", post(handle_create_dashboard))
        .route("/api/dashboards/templates", get(handle_get_templates))
        .route("/api/dashboards/:id", get(handle_get_dashboard))
        .route("/api/dashboards/:id", put(handle_update_dashboard))
        .route("/api/dashboards/:id", delete(handle_delete_dashboard))
        .route("/api/dashboards/:id/widgets", post(handle_add_widget))
        .route(
            "/api/dashboards/:id/widgets/:widget_id",
            put(handle_update_widget),
        )
        .route(
            "/api/dashboards/:id/widgets/:widget_id",
            delete(handle_delete_widget),
        )
        .route(
            "/api/dashboards/:id/widgets/:widget_id/data",
            get(handle_get_widget_data),
        )
        .route("/api/dashboards/sources", get(handle_list_data_sources))
        .route("/api/dashboards/sources", post(handle_create_data_source))
        .route(
            "/api/dashboards/sources/:id/test",
            post(handle_test_data_source),
        )
        .route(
            "/api/dashboards/sources/:id",
            delete(handle_delete_data_source),
        )
        .route("/api/dashboards/query", post(handle_conversational_query))
}
