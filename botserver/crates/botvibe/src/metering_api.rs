
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::metering::{LimitRow, UsageSummary, VMeteringRef};
use crate::rbac::ProjectRbac;


#[derive(Debug, Serialize)]
pub struct MeteringResponse {
    pub success: bool,
    pub summary: Option<UsageSummary>,
    pub limits: Option<Vec<LimitRow>>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LimitsQuery {
    pub org_id: Uuid,
}

type ApiResult = (StatusCode, Json<MeteringResponse>);

fn ok(summary: UsageSummary) -> ApiResult {
    (
        StatusCode::OK,
        Json(MeteringResponse {
            success: true,
            summary: Some(summary),
            limits: None,
            error: None,
        }),
    )
}

fn ok_limits(limits: Vec<LimitRow>) -> ApiResult {
    (
        StatusCode::OK,
        Json(MeteringResponse {
            success: true,
            summary: None,
            limits: Some(limits),
            error: None,
        }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe metering API error: {msg}");
    (
        StatusCode::OK,
        Json(MeteringResponse {
            success: false,
            summary: None,
            limits: None,
            error: Some(msg),
        }),
    )
}

fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe metering API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(MeteringResponse {
            success: false,
            summary: None,
            limits: None,
            error: Some(msg),
        }),
    )
}

async fn usage(
    Extension(metering): Extension<VMeteringRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, crate::rbac::ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match metering.summary(project_id) {
        Ok(summary) => ok(summary),
        Err(e) => err(e),
    }
}

async fn limits(
    Extension(metering): Extension<VMeteringRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<LimitsQuery>,
) -> ApiResult {
    match rbac.is_org_admin(user.user_id) {
        Ok(true) => {}
        Ok(false) => return forbidden("forbidden: org admin role required".into()),
        Err(e) => return err(e),
    }
    match metering.limits(query.org_id) {
        Ok(rows) => ok_limits(rows),
        Err(e) => err(e),
    }
}

pub fn metering_router(metering: VMeteringRef, rbac: ProjectRbac) -> Router {
    Router::new()
        .route("/api/vibe/projects/:project_id/metering", get(usage))
        .route("/api/vibe/metering/limits", get(limits))
        .layer(Extension(metering))
        .layer(Extension(rbac))
}