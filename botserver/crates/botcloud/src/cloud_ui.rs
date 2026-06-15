use axum::{
    extract::Query,
    http::header,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::SaasService;

fn css(data: &'static str) -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        data,
    )
        .into_response()
}

fn js(data: &'static str) -> Response {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        data,
    )
        .into_response()
}

pub fn configure_cloud_ui_routes() -> Router<Arc<SaasService>> {
    Router::new()
        .route("/cloud", get(cloud_home))
        .route("/cloud/login", get(login_page))
        .route("/cloud/signup", get(signup_page))
        .route("/cloud/dashboard", get(dashboard_page))
        .route("/cloud/plans", get(plans_page))
        .route("/cloud/checkout", get(checkout_page))
        .route("/cloud/checkout/success", get(checkout_success_page))
        .route("/cloud/checkout/cancel", get(checkout_cancel))
        .route("/cloud/store", get(store_page))
        .route("/cloud/services", get(services_page))
        .route("/cloud/invoices", get(invoices_page))
        .route("/cloud/payment-cards", get(payment_cards_page))
        .route("/cloud/profile", get(profile_page))
        .route("/cloud/offers", get(offers_page))
        .route("/cloud/appstores", get(appstores_page))
        .route("/cloud/css/cloud.css", get(cloud_css))
        .route("/cloud/js/cloud.js", get(cloud_js))
        .route("/cloud/js/cloud-auth.js", get(cloud_auth_js))
        .route("/cloud/js/cloud-appstores.js", get(cloud_appstores_js))
        .route("/cloud/js/cloud-cards.js", get(cloud_cards_js))
        .route("/cloud/js/cloud-checkout.js", get(cloud_checkout_js))
        .route("/cloud/js/cloud-invoices.js", get(cloud_invoices_js))
        .route("/cloud/js/cloud-offers.js", get(cloud_offers_js))
        .route("/cloud/js/cloud-plans.js", get(cloud_plans_js))
        .route("/cloud/js/cloud-profile.js", get(cloud_profile_js))
        .route("/cloud/js/cloud-services.js", get(cloud_services_js))
        .route("/cloud/js/cloud-store.js", get(cloud_store_js))
        .route("/cloud/partials/sidebar.html", get(sidebar_partial))
}

async fn cloud_home() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/index.html"))
}

async fn plans_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/plans.html"))
}

async fn login_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/login.html"))
}

async fn signup_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/signup.html"))
}

async fn dashboard_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/dashboard.html"))
}

async fn checkout_page(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let payload = params.get("payload").cloned().unwrap_or_default();
    let html = include_str!("../../../../botui/ui/cloud/checkout.html");
    let html = html.replace("{{PAYLOAD}}", &payload);
    Html(html)
}

async fn checkout_success_page(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let html = include_str!("../../../../botui/ui/cloud/success.html");
    let html = html.replace("{{SESSION_ID}}", &session_id);
    Html(html)
}

async fn checkout_cancel() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/cancel.html"))
}

async fn store_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/store.html"))
}

async fn services_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/services.html"))
}

async fn invoices_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/invoices.html"))
}

async fn payment_cards_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/payment-cards.html"))
}

async fn profile_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/profile.html"))
}

async fn offers_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/offers.html"))
}

async fn appstores_page() -> Html<&'static str> {
    Html(include_str!("../../../../botui/ui/cloud/appstores.html"))
}

async fn cloud_css() -> Response {
    css(include_str!("../../../../botui/ui/cloud/css/cloud.css"))
}

async fn cloud_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud.js"))
}

async fn cloud_auth_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-auth.js"))
}

async fn cloud_appstores_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-appstores.js"))
}

async fn cloud_cards_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-cards.js"))
}

async fn cloud_checkout_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-checkout.js"))
}

async fn cloud_invoices_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-invoices.js"))
}

async fn cloud_offers_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-offers.js"))
}

async fn cloud_plans_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-plans.js"))
}

async fn cloud_profile_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-profile.js"))
}

async fn cloud_services_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-services.js"))
}

async fn cloud_store_js() -> Response {
    js(include_str!("../../../../botui/ui/cloud/js/cloud-store.js"))
}

async fn sidebar_partial() -> Response {
    let html = include_str!("../../../../botui/ui/cloud/partials/sidebar.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}
