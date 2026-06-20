
pub mod database;
pub mod db;
pub mod ui_fragments;

use axum::Router;

/// Register HTMX fragment routes only.
/// All API routes are now in feature-gated extracted crates.
pub fn register<S: Clone + Send + Sync + 'static>(r: Router<S>) -> Router<S> {
    r.merge(ui_fragments::configure())
}
