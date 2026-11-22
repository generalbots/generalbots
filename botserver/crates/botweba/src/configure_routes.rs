use super::*;
use crate::forms::{self, FormSubmitRequest};
use crate::persistence::FormSubmission;
use crate::render_routes::{embed_app, embed_page, preview_app, render_app, render_page};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use botsecurity_core::error_sanitizer::log_and_sanitize_json;

pub fn configure_routes(state: Arc<WebaState>) -> Router {
    Router::new()
        .route("/apps", get(list_apps).post(create_app))
        .route("/apps/:id", get(get_app).put(update_app).delete(delete_app))
        .route("/apps/:id/pages", get(list_pages).post(create_page))
        .route(
            "/apps/:id/pages/:page_id",
            get(get_page).put(update_page).delete(delete_page),
        )
        .route("/apps/:id/publish", post(publish_app))
        .route("/apps/:id/domain", get(domain_status).post(set_domain))
        .route("/apps/:id/campaign-link", post(set_campaign_link))
        .route("/apps/:id/campaign-clicks", get(campaign_clicks))
        .route("/apps/:id/unpublish", post(unpublish_app))
        .route("/apps/:id/preview", get(preview_app))
        .route("/render/:slug", get(render_app))
        .route("/render/:slug/*path", get(render_page))
        // Isolated presentation surface: the suite embeds this instead of the
        // rendered document, so the app always runs in a sandboxed iframe.
        .route("/embed/:slug", get(embed_app))
        .route("/embed/:slug/*path", get(embed_page))
        // Public form capture for a published site.
        .route("/forms/:slug/submit", post(submit_form))
        .route("/forms/:slug/submit-form", post(submit_form_html))
        .with_state(state)
}

/// A published document is handed to every visitor, so publishing is an
/// administrative act. A single-user desktop install has no directory account:
/// the local OS user owns the instance and keeps the ability to publish.
pub(crate) fn authorize_publisher(user: &AuthenticatedUser) -> Result<(), StatusCode> {
    if user.is_admin()
        || user
            .metadata
            .get("desktop_local")
            .is_some_and(|value| value == "true")
    {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[path = "configure_routes_handlers.rs"]
mod handlers;
use handlers::*;

pub(super) async fn domain_status(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let apps = state.apps.read().await;
    let app = apps.get(&id).ok_or((StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"not_found"}))))?;
    Ok(Json(serde_json::json!({
        "domain": app.config.custom_domain,
        "state": app.config.domain_state,
        "tls_note": "TLS is terminated by the edge proxy; add an A/AAAA record pointing at the appliance and wait for verification."
    })))
}

#[derive(serde::Deserialize)]
pub struct DomainRequest { domain: String }

pub(super) async fn set_domain(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<DomainRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let domain = req.domain.trim().to_ascii_lowercase();
    let valid = domain.len() <= 253 && domain.split('.').count() >= 2 &&
        domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.');
    if !valid {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"invalid_domain"}))));
    }
    // persist() takes read locks on the same map, so the write guard must be
    // dropped before saving or the handler deadlocks.
    {
        let mut apps = state.apps.write().await;
        let app = apps.get_mut(&id).ok_or((StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"not_found"}))))?;
        app.config.custom_domain = Some(domain.clone());
        app.config.domain_state = "pending".to_string();
    }
    state.persist().await;
    Ok(Json(serde_json::json!({"domain":domain,"state":"pending"})))
}

#[derive(serde::Deserialize)]
pub struct CampaignLinkRequest { campaign_id: String, page: String }

pub(super) async fn set_campaign_link(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CampaignLinkRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !req.campaign_id.chars().all(|c| c.is_ascii_alphanumeric() || "-_".contains(c)) {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"invalid_campaign"}))));
    }
    let link;
    {
        let mut apps = state.apps.write().await;
        let app = apps.get_mut(&id).ok_or((StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"not_found"}))))?;
        link = format!("/render/{}/{}?utm_campaign={}", app.slug, req.page.trim_start_matches('/'), req.campaign_id);
        app.config.campaign_link = Some(link.clone());
    }
    state.persist().await;
    Ok(Json(serde_json::json!({"link":link,"campaign_id":req.campaign_id})))
}

/// #2538 — attributed clicks on campaign links to this site's pages.
pub(super) async fn campaign_clicks(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let apps = state.apps.read().await;
    let app = apps.get(&id).ok_or((
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "not_found" })),
    ))?;
    let slug = app.slug.clone();
    drop(apps);
    let clicks = state.clicks.read().await;
    let matched: Vec<&crate::persistence::CampaignClick> = clicks
        .iter()
        .filter(|click| click.app_id == id || click.slug == slug)
        .collect();
    Ok(Json(serde_json::json!({ "site": slug, "clicks": matched })))
}
