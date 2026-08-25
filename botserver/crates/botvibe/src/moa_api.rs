//! HTTP surface for the #1173 mixture-of-agents engine.
//! The `/api/vibe/moa/share/:token` route is intentionally anonymous so
//! published deliverables are shareable without auth (registered as
//! anonymous in the RBAC defaults).

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::moa::{MoaRef, MoaRequest, MoaRun};

#[derive(Debug, Serialize)]
pub struct MoaResponse {
    pub success: bool,
    pub run: Option<MoaRun>,
    pub runs: Option<Vec<MoaRun>>,
    pub share_url: Option<String>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<MoaResponse>);

fn ok_run(run: MoaRun) -> ApiResult {
    let share_url = run.share_token.as_ref().map(|t| format!("/api/vibe/moa/share/{t}"));
    (
        StatusCode::OK,
        Json(MoaResponse { success: true, run: Some(run), runs: None, share_url, error: None }),
    )
}

fn ok_runs(runs: Vec<MoaRun>) -> ApiResult {
    (
        StatusCode::OK,
        Json(MoaResponse { success: true, run: None, runs: Some(runs), share_url: None, error: None }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe MOA API error: {msg}");
    (
        StatusCode::OK,
        Json(MoaResponse { success: false, run: None, runs: None, share_url: None, error: Some(msg) }),
    )
}

pub fn moa_router(moa: MoaRef) -> Router {
    Router::new()
        .route("/api/vibe/moa/route", post(route))
        .route("/api/vibe/moa/runs", get(list_runs))
        .route("/api/vibe/moa/runs/:run_id", get(get_run))
        .route("/api/vibe/moa/share/:token", get(get_share))
        .layer(Extension(moa))
}

async fn route(
    Extension(moa): Extension<MoaRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<MoaRequest>,
) -> ApiResult {
    if req.prompt.trim().is_empty() {
        return err("prompt is required".to_string());
    }
    match moa.route(&req).await {
        Ok(run) => ok_run(run),
        Err(e) => err(e),
    }
}

async fn list_runs(
    Extension(moa): Extension<MoaRef>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> ApiResult {
    ok_runs(moa.list().await)
}

async fn get_run(
    Extension(moa): Extension<MoaRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(run_id): Path<Uuid>,
) -> ApiResult {
    match moa.get(&run_id).await {
        Some(run) => ok_run(run),
        None => err(format!("run {run_id} not found")),
    }
}

/// Anonymous share endpoint — no `AuthenticatedUser` extension required.
async fn get_share(
    Extension(moa): Extension<MoaRef>,
    Path(token): Path<String>,
) -> impl axum::response::IntoResponse {
    match moa.resolve_share(&token).await {
        Some(run) => {
            let html = match run.deliverable {
                Some(d) => format!(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title>\
                     <style>body{{font-family:system-ui;max-width:860px;margin:2rem auto;padding:0 1rem;line-height:1.6}}\
                     pre{{background:#f5f5f5;padding:1rem;overflow-x:auto}}</style></head><body>\
                     <h1>Shared deliverable</h1><div>{}</div></body></html>",
                    html_escape(&run.prompt),
                    html_escape(&d)
                ),
                None => "<h1>This deliverable is empty</h1>".to_string(),
            };
            (
                StatusCode::OK,
                [("Content-Type", "text/html; charset=utf-8")],
                html,
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            [("Content-Type", "text/plain; charset=utf-8")],
            "deliverable not found".to_string(),
        ),
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\n', "<br>")
}
