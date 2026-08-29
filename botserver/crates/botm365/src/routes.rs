use axum::routing::{get, post};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/m365/sharepoint", get(handlers::list_sharepoint))
        .route("/api/m365/calendar", get(handlers::list_calendar))
        .route("/api/m365/onedrive", get(handlers::list_onedrive))
        .route("/api/m365/teams", get(handlers::list_teams))
        .route("/api/m365/onedrive/download/:id", get(handlers::download_file))
        .route("/api/m365/connect", post(handlers::connect_account))
        .route("/api/m365/disconnect", post(handlers::disconnect_account))
        .route("/api/m365/sync", post(handlers::sync_now))
        .route("/api/m365/settings/sync", post(handlers::update_sync_settings))
        .route("/api/m365/settings", get(handlers::get_settings))
        .route("/api/o365/sharepoint", get(handlers::list_sharepoint))
        .route("/api/o365/calendar", get(handlers::list_calendar))
        .route("/api/o365/onedrive", get(handlers::list_onedrive))
        .route("/api/o365/teams", get(handlers::list_teams))
        .route("/api/o365/onedrive/download/:id", get(handlers::download_file))
        .route("/api/o365/connect", post(handlers::connect_account))
        .route("/api/o365/disconnect", post(handlers::disconnect_account))
        .route("/api/o365/sync", post(handlers::sync_now))
        .route("/api/o365/settings/sync", post(handlers::update_sync_settings))
        .route("/api/o365/settings", get(handlers::get_settings))
}
