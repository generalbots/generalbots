use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botvideo::{
    AnalyticsEngine, ExportProgressBroadcaster, VideoEngine, VideoRenderWorker,
    broadcast_export_progress, configure_video_routes as crate_configure_video_routes,
    configure_video_ui_routes as crate_configure_video_ui_routes,
    configure as crate_configure,
    get_analytics_handler, record_view_handler, start_render_worker,
    export_progress_websocket,
};

fn make_video_state(app_state: &Arc<AppState>) -> Arc<botvideo::routes::AppState> {
    Arc::new(botvideo::routes::AppState {
        conn: app_state.conn.clone(),
        cache: app_state.cache.clone(),
    })
}

pub fn configure_video_routes(app_state: Arc<AppState>) -> Router {
    crate_configure_video_routes()
        .with_state(make_video_state(&app_state))
}

pub fn configure_video_ui_routes(app_state: Arc<AppState>) -> Router {
    crate_configure_video_ui_routes()
        .with_state(make_video_state(&app_state))
}
