use axum::{routing::get, Router};
use std::sync::Arc;

use crate::shared::state::AppState;

pub fn configure_video_routes() -> Router<Arc<AppState>> {
    Router::new().route("/video", get(video_ui))
}

pub fn configure(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.merge(configure_video_routes())
}

async fn video_ui() -> &'static str {
    "Video module"
}
