//! HTTP surface for the #1171 Planner-Executor-Verifier runtime.
//! All routes live under `/api/vibe/planner/**` so the RBAC registry
//! covers them with the `/api/vibe/**` wildcard family.

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::planner::{PlannerRef, PlannerRequest, PlannerRun};

#[derive(Debug, Serialize)]
pub struct PlannerResponse {
    pub success: bool,
    pub run: Option<PlannerRun>,
    pub runs: Option<Vec<PlannerRun>>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<PlannerResponse>);

fn ok_run(run: PlannerRun) -> ApiResult {
    (
        StatusCode::OK,
        Json(PlannerResponse { success: true, run: Some(run), runs: None, error: None }),
    )
}

fn ok_runs(runs: Vec<PlannerRun>) -> ApiResult {
    (
        StatusCode::OK,
        Json(PlannerResponse { success: true, run: None, runs: Some(runs), error: None }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe planner API error: {msg}");
    (
        StatusCode::OK,
        Json(PlannerResponse { success: false, run: None, runs: None, error: Some(msg) }),
    )
}

pub fn planner_router(planner: PlannerRef) -> Router {
    Router::new()
        .route("/api/vibe/planner/execute", post(execute))
        .route("/api/vibe/planner/runs", get(list_runs))
        .route("/api/vibe/planner/runs/:run_id", get(get_run))
        .layer(Extension(planner))
}

async fn execute(
    Extension(planner): Extension<PlannerRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<PlannerRequest>,
) -> ApiResult {
    if req.intent.trim().is_empty() {
        return err("intent is required".to_string());
    }
    match planner.execute(&req).await {
        Ok(run) => ok_run(run),
        Err(e) => err(e),
    }
}

async fn list_runs(
    Extension(planner): Extension<PlannerRef>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> ApiResult {
    ok_runs(planner.list().await)
}

async fn get_run(
    Extension(planner): Extension<PlannerRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(run_id): Path<Uuid>,
) -> ApiResult {
    match planner.get(&run_id).await {
        Some(run) => ok_run(run),
        None => err(format!("run {run_id} not found")),
    }
}
