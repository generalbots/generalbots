use axum::routing::get;
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/m365/sharepoint", get(handlers::list_sharepoint))
        .route("/api/m365/calendar", get(handlers::list_calendar))
        .route("/api/m365/onedrive", get(handlers::list_onedrive))
        .route("/api/m365/settings", get(handlers::get_settings))
        .route("/api/o365/sharepoint", get(handlers::list_sharepoint))
        .route("/api/o365/calendar", get(handlers::list_calendar))
        .route("/api/o365/onedrive", get(handlers::list_onedrive))
        .route("/api/o365/settings", get(handlers::get_settings))
}
