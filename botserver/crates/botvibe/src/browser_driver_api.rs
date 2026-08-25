//! HTTP surface for the #1182 browser driver service.

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::browser_driver::{BrowserDriverRef, DriverRequest, DriverRun, StepReport};

#[derive(Debug, Serialize)]
pub struct DriverResponse {
    pub success: bool,
    pub run: Option<DriverRun>,
    pub runs: Option<Vec<DriverRun>>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<DriverResponse>);

fn ok_run(run: DriverRun) -> ApiResult {
    (
        StatusCode::OK,
        Json(DriverResponse { success: true, run: Some(run), runs: None, error: None }),
    )
}

fn ok_runs(runs: Vec<DriverRun>) -> ApiResult {
    (
        StatusCode::OK,
        Json(DriverResponse { success: true, run: None, runs: Some(runs), error: None }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe browser-driver API error: {msg}");
    (
        StatusCode::OK,
        Json(DriverResponse { success: false, run: None, runs: None, error: Some(msg) }),
    )
}

pub fn browser_driver_router(driver: BrowserDriverRef) -> Router {
    Router::new()
        .route("/api/vibe/browser-driver/start", post(start))
        .route("/api/vibe/browser-driver/runs", get(list_runs))
        .route("/api/vibe/browser-driver/runs/:run_id", get(get_run))
        .route("/api/vibe/browser-driver/runs/:run_id/step", post(report_step))
        .route("/api/vibe/browser-driver/runs/:run_id/complete", post(complete))
        .layer(Extension(driver))
}

async fn start(
    Extension(driver): Extension<BrowserDriverRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<DriverRequest>,
) -> ApiResult {
    match driver.start(&req).await {
        Ok(run) => ok_run(run),
        Err(e) => err(e),
    }
}

async fn list_runs(
    Extension(driver): Extension<BrowserDriverRef>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> ApiResult {
    ok_runs(driver.list().await)
}

async fn get_run(
    Extension(driver): Extension<BrowserDriverRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(run_id): Path<Uuid>,
) -> ApiResult {
    match driver.get(&run_id).await {
        Some(run) => ok_run(run),
        None => err(format!("run {run_id} not found")),
    }
}

async fn report_step(
    Extension(driver): Extension<BrowserDriverRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(run_id): Path<Uuid>,
    Json(report): Json<StepReport>,
) -> ApiResult {
    if report.description.trim().is_empty() {
        return err("step description is required".to_string());
    }
    match driver.report_step(&run_id, &report).await {
        Ok(run) => ok_run(run),
        Err(e) => err(e),
    }
}

async fn complete(
    Extension(driver): Extension<BrowserDriverRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(run_id): Path<Uuid>,
) -> ApiResult {
    match driver.complete(&run_id).await {
        Some(run) => ok_run(run),
        None => err(format!("run {run_id} not found")),
    }
}
