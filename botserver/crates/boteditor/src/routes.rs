use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::handlers;

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/editor/files", get(handlers::list_files))
        .route("/api/editor/file/*path", get(handlers::read_file).post(handlers::save_file))
        .route("/api/editor/save", post(handlers::save_file_alt))
        .route("/api/editor/save-as", get(handlers::handle_save_as))
        .route("/api/editor/undo", post(handlers::handle_undo))
        .route("/api/editor/redo", post(handlers::handle_redo))
        .route("/api/editor/format", post(handlers::handle_format))
        .route("/api/editor/magic", post(handlers::handle_magic))
}
