use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::domains::{BindDomainRequest, DomainResult, ProjectDomains};
use crate::metering::VMeteringRef;
use crate::rbac::{ProjectRbac, ProjectRole};

pub type ProjectDomainsRef = Arc<ProjectDomains>;

type ApiResult = (StatusCode, Json<DomainResult>);

fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe domains API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(DomainResult {
            success: false,
            bind: None,
            binds: None,
            verify: None,
            error: Some(msg),
        }),
    )
}

fn ok(bind: crate::domains::DomainBind) -> ApiResult {
    (StatusCode::OK, Json(DomainResult::ok(bind)))
}

fn ok_list(binds: Vec<crate::domains::DomainBind>) -> ApiResult {
    (StatusCode::OK, Json(DomainResult::ok_list(binds)))
}

fn ok_verify(v: serde_json::Value) -> ApiResult {
    (StatusCode::OK, Json(DomainResult::ok_verify(v)))
}

fn ok_deleted() -> ApiResult {
    (StatusCode::OK, Json(DomainResult::deleted()))
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe domains API error: {msg}");
    (StatusCode::OK, Json(DomainResult::err(msg)))
}

pub fn domains_router(domains: ProjectDomainsRef, rbac: ProjectRbac, metering: VMeteringRef) -> Router {
    Router::new()
        .route(
            "/api/vibe/projects/:project_id/domains",
            post(bind_domain).get(list_domains),
        )
        .route(
            "/api/vibe/projects/:project_id/domains/:bind_id",
            axum::routing::delete(unbind_domain),
        )
        .route("/api/vibe/domains/verify", post(verify_domain))
        .route("/api/vibe/domains/:domain/tls", post(issue_tls))
        .route(
            "/api/vibe/domains/:bind_id/access",
            axum::routing::patch(update_domain_access),
        )
        .layer(Extension(domains))
        .layer(Extension(rbac))
        .layer(Extension(metering))
}

async fn bind_domain(
    Extension(domains): Extension<ProjectDomainsRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(metering): Extension<crate::metering::VMeteringRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BindDomainRequest>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    if let Err(e) = metering.enforce_for_project(project_id, crate::metering::MeterKind::DomainBindings) {
        return forbidden(e);
    }
    match domains.bind(project_id, &req).await {
        Ok(bind) => {
            let _ = metering.add_for_project(project_id, &bind.env, crate::metering::MeterKind::DomainBindings, 1.0);
            ok(bind)
        }
        Err(e) => err(e),
    }
}

async fn list_domains(
    Extension(domains): Extension<ProjectDomainsRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match domains.list(project_id) {
        Ok(binds) => ok_list(binds),
        Err(e) => err(e),
    }
}

async fn unbind_domain(
    Extension(domains): Extension<ProjectDomainsRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, bind_id)): Path<(Uuid, Uuid)>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match domains.unbind(bind_id).await {
        Ok(()) => ok_deleted(),
        Err(e) => err(e),
    }
}

async fn update_domain_access(
    Extension(domains): Extension<ProjectDomainsRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(bind_id): Path<Uuid>,
    Json(req): Json<UpdateDomainAccessRequest>,
) -> ApiResult {
    let bind = match domains.get(bind_id) {
        Ok(b) => b,
        Err(e) => return err(e),
    };
    match rbac.require_role(user.user_id, bind.project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match domains
        .update_access(bind_id, &req.access, req.allowed_emails.clone())
        .await
    {
        Ok(b) => ok(b),
        Err(e) => err(e),
    }
}

async fn verify_domain(
    Extension(domains): Extension<ProjectDomainsRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<VerifyDomainRequest>,
) -> ApiResult {
    if user.user_id.is_nil() {
        return forbidden("forbidden: anonymous users cannot verify domains".into());
    }
    match domains.verify_dns(&req.domain) {
        Ok(v) => ok_verify(v),
        Err(e) => err(e),
    }
}

async fn issue_tls(
    Extension(domains): Extension<ProjectDomainsRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(domain): Path<String>,
) -> ApiResult {
    let bind = match domain_to_bind(&domains, &domain).await {
        Ok(b) => b,
        Err(e) => return err(e),
    };
    match rbac.require_role(user.user_id, bind.project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match domains.issue_tls(&bind).await {
        Ok(v) => ok_verify(v),
        Err(e) => err(e),
    }
}

async fn domain_to_bind(
    domains: &ProjectDomainsRef,
    domain: &str,
) -> Result<crate::domains::DomainBind, String> {
    let d = ProjectDomains::validate_domain(domain)?;
    let binds = domains
        .list_for_domain(&d)
        .map_err(|e| format!("lookup bind: {e}"))?;
    binds.into_iter().next().ok_or_else(|| format!("no binding for {d}"))
}

#[derive(Debug, serde::Deserialize)]
pub struct VerifyDomainRequest {
    pub domain: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateDomainAccessRequest {
    pub access: String,
    pub allowed_emails: Option<String>,
}
