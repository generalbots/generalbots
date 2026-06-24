pub mod database;
pub mod db;
pub mod ui_fragments;

use axum::Router;

pub fn register<S: Clone + Send + Sync + 'static>(r: Router<S>) -> Router<S> {
    r.merge(ui_fragments::configure())
}
