use axum::routing::get;
use axum::Router;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::handlers;

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/editor/files", get(handlers::list_files))
        .route("/api/editor/file/*path", get(handlers::read_file).post(handlers::save_file))
}
