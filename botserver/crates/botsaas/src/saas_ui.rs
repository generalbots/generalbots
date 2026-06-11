use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::SaasService;

pub fn configure_saas_ui_routes() -> Router<Arc<SaasService>> {
    Router::new()
        .route("/saas", get(saas_home))
        .route("/saas/login", get(login_page))
        .route("/saas/signup", get(signup_page))
        .route("/saas/dashboard", get(dashboard_page))
        .route("/saas/checkout", get(checkout_page))
        .route("/saas/checkout/success", get(checkout_success_page))
        .route("/saas/checkout/cancel", get(checkout_cancel))
}

async fn saas_home() -> Html<&'static str> {
    Html(include_str!("../../../botui/ui/suite/saas/index.html"))
}

async fn login_page() -> Html<&'static str> {
    Html(include_str!("../../../botui/ui/suite/saas/login.html"))
}

async fn signup_page() -> Html<&'static str> {
    Html(include_str!("../../../botui/ui/suite/saas/signup.html"))
}

async fn dashboard_page() -> Html<&'static str> {
    Html(include_str!("../../../botui/ui/suite/saas/dashboard.html"))
}

async fn checkout_page(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let payload = params.get("payload").cloned().unwrap_or_default();
    let html = include_str!("../../../botui/ui/suite/saas/checkout.html");
    let html = html.replace("{{PAYLOAD}}", &payload);
    Html(html)
}

async fn checkout_success_page(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let html = include_str!("../../../botui/ui/suite/saas/success.html");
    let html = html.replace("{{SESSION_ID}}", &session_id);
    Html(html)
}

async fn checkout_cancel() -> Html<&'static str> {
    Html(include_str!("../../../botui/ui/suite/saas/cancel.html"))
}
