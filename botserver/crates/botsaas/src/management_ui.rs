use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::SaasService;

pub fn configure_management_ui_routes() -> Router<Arc<SaasService>> {
    Router::new()
        .route("/management", get(management_home))
        .route("/management/login", get(login_page))
        .route("/management/signup", get(signup_page))
        .route("/management/dashboard", get(dashboard_page))
        .route("/management/plans", get(plans_page))
        .route("/management/checkout", get(checkout_page))
        .route("/management/checkout/success", get(checkout_success_page))
        .route("/management/checkout/cancel", get(checkout_cancel))
        .route("/management/store", get(store_page))
        .route("/management/services", get(services_page))
        .route("/management/invoices", get(invoices_page))
        .route("/management/payment-cards", get(payment_cards_page))
        .route("/management/profile", get(profile_page))
}

async fn management_home() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/index.html"))
}

async fn plans_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/plans.html"))
}

async fn login_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/login.html"))
}

async fn signup_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/signup.html"))
}

async fn dashboard_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/dashboard.html"))
}

async fn checkout_page(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let payload = params.get("payload").cloned().unwrap_or_default();
    let html = include_str!("../../../../botui/ui/management/checkout.html");
    let html = html.replace("{{PAYLOAD}}", &payload);
    Html(html)
}

async fn checkout_success_page(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let html = include_str!("../../../../botui/ui/management/success.html");
    let html = html.replace("{{SESSION_ID}}", &session_id);
    Html(html)
}

async fn checkout_cancel() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/cancel.html"))
}

async fn store_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/store.html"))
}

async fn services_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/services.html"))
}

async fn invoices_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/invoices.html"))
}

async fn payment_cards_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/payment-cards.html"))
}

async fn profile_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/management/profile.html"))
}
