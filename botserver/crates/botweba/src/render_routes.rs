//! Render, preview and embed surfaces for WebA apps (#2124).
//!
//! Every document leaves through [`document_response`], which attaches the
//! sandbox CSP, and every embed shell through [`embed_response`]. Keeping the
//! render paths in one module makes that guarantee easy to audit.

use super::*;
use crate::csp::{document_response, embed_response, embed_shell};
use axum::http::StatusCode;
use axum::response::Response;

pub(super) async fn preview_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Response, axum::http::StatusCode> {
    let apps = state.apps.read().await;
    let app = apps.get(&id).ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let pages = state.pages.read().await;
    let index_page = pages.values().find(|p| p.app_id == id && p.is_index);

    let content = index_page
        .map(|p| p.content.clone())
        .unwrap_or_else(|| "<p>No content yet</p>".to_string());

    let html = render_html(app, &content);
    Ok(document_response(html, "no-store"))
}
pub(super) async fn render_app(
    State(state): State<Arc<WebaState>>,
    Path(slug): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, axum::http::StatusCode> {
    let (app, content) = published_page(&state, &slug, None).await?;
    if let Some(campaign_id) = query.get("utm_campaign") {
        state.record_click(crate::persistence::CampaignClick {
            app_id: app.id,
            slug: slug.clone(),
            campaign_id: campaign_id.clone(),
            path: "/".to_string(),
            clicked_at: chrono::Utc::now(),
        })
        .await;
    }
    Ok(document_response(
        render_html(&app, &content),
        "public, max-age=60",
    ))
}
pub(super) async fn render_page(
    State(state): State<Arc<WebaState>>,
    Path((slug, path)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, axum::http::StatusCode> {
    let normalized_path = format!("/{}", path.trim_start_matches('/'));
    let (app, content) = published_page(&state, &slug, Some(&normalized_path)).await?;
    if let Some(campaign_id) = query.get("utm_campaign") {
        state.record_click(crate::persistence::CampaignClick {
            app_id: app.id,
            slug: slug.clone(),
            campaign_id: campaign_id.clone(),
            path: normalized_path.clone(),
            clicked_at: chrono::Utc::now(),
        })
        .await;
    }
    Ok(document_response(
        render_html(&app, &content),
        "public, max-age=60",
    ))
}
/// Serves the sandboxed iframe shell for the app index.
pub(super) async fn embed_app(
    State(state): State<Arc<WebaState>>,
    Path(slug): Path<String>,
) -> Result<Response, axum::http::StatusCode> {
    let (app, _) = published_page(&state, &slug, None).await?;
    Ok(embed_response(embed_shell(
        &format!("/render/{}", app.slug),
        &app.name,
    )))
}
/// Serves the sandboxed iframe shell for a named page.
pub(super) async fn embed_page(
    State(state): State<Arc<WebaState>>,
    Path((slug, path)): Path<(String, String)>,
) -> Result<Response, axum::http::StatusCode> {
    let normalized_path = format!("/{}", path.trim_start_matches('/'));
    let (app, _) = published_page(&state, &slug, Some(&normalized_path)).await?;
    Ok(embed_response(embed_shell(
        &format!("/render/{}/{}", app.slug, path.trim_start_matches('/')),
        &app.name,
    )))
}
/// Resolves a published app from its immutable snapshot or `404`.
///
/// Draft page edits never affect this path: the `WebAppPage` values come from
/// the snapshot frozen at publish time, not from the live draft map (#2538).
async fn published_page(
    state: &Arc<WebaState>,
    slug: &str,
    path: Option<&str>,
) -> Result<(WebApp, String), StatusCode> {
    let site = state.live_site(slug).await.ok_or(StatusCode::NOT_FOUND)?;
    let apps = state.apps.read().await;
    let mut app = apps.get(&site.app_id).cloned().ok_or(StatusCode::NOT_FOUND)?;
    drop(apps);

    // The snapshot owns the public identity too, so renaming a draft cannot
    // change the live page title or its slug.
    app.name.clone_from(&site.name);
    app.slug.clone_from(&site.slug);

    let page = match path {
        Some(path) => site.pages.iter().find(|page| page.path == path),
        None => site.pages.iter().find(|page| page.is_index),
    };
    let content = page
        .map(|page| page.content.clone())
        .unwrap_or_else(|| "<p>Page not found</p>".to_string());

    Ok((app, content))
}
