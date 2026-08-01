use axum::routing::{delete, get, post, put};
use axum::Router;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::handlers;

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/database/schema", get(handlers::get_schema))
        .route("/api/database/table/:name/data", get(handlers::get_table_data))
        .route("/api/database/query", post(handlers::execute_query))
        .route("/api/database/table/:name/row", post(handlers::insert_row).put(handlers::update_row))
        .route("/api/database/table/:name/row/:id", delete(handlers::delete_row))
        .route("/api/database/table/:name/rows/batch-delete", post(handlers::batch_delete))
        .route("/api/database/table", post(handlers::create_table))
        .route("/api/database/table/:name", put(handlers::alter_table).delete(handlers::drop_table))
        .route("/api/database/table/:name/column", post(handlers::add_column))
        .route("/api/database/table/:name/foreign-keys", get(handlers::get_foreign_keys))
}
