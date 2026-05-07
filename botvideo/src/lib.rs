pub mod models;
pub mod requests;
pub mod responses;
pub mod safe_command;
pub mod safe_error;
pub mod schema;

pub mod analytics;
pub mod engine;
pub mod handlers;
pub mod mcp_tools;
pub mod render;
pub mod routes;
pub mod ui;
pub mod websocket;

pub use analytics::{get_analytics_handler, record_view_handler, AnalyticsEngine};
pub use engine::VideoEngine;
pub use handlers::*;
pub use models::*;
pub use render::{start_render_worker, VideoRenderWorker};
pub use schema::*;
pub use websocket::{broadcast_export_progress, export_progress_websocket, ExportProgressBroadcaster};

use axum::Router;
use std::sync::Arc;

use crate::routes::AppState;

pub fn configure_video_routes() -> Router<Arc<AppState>> {
    crate::routes::configure_video_routes()
}

pub fn configure(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.merge(configure_video_routes())
}

pub fn configure_video_ui_routes() -> Router<Arc<AppState>> {
    crate::ui::routes::configure_video_ui_routes()
}
