use axum::{
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::SaasService;

pub fn configure_cloud_ui_routes() -> Router<Arc<SaasService>> {
    Router::new()
        .route("/cloud/partials/sidebar.html", get(sidebar_partial))
}

async fn sidebar_partial() -> Response {
    let html = include_str!("../../../../botui/ui/cloud/partials/sidebar.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}
