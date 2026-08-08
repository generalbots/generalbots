//! #756/#774/#770 — REST surface for project domain bindings. Thin handlers
//! over `ProjectDomains`: bind/unbind domains on a project env (Caddy route
//! managed under the hood), list bindings, DNS ownership verification
//! (#774), and TLS/ACME issuance via Caddy (#770).

use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::routing::post;
use axum::{Json, Router};
use uuid::Uuid;

use crate::domains::{BindDomainRequest, DomainResult, ProjectDomains};

pub type ProjectDomainsRef = Arc<ProjectDomains>;

pub fn domains_router(domains: ProjectDomainsRef) -> Router {
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
        .layer(Extension(domains))
}

async fn bind_domain(
    Extension(domains): Extension<ProjectDomainsRef>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BindDomainRequest>,
) -> Json<DomainResult> {
    match domains.bind(project_id, &req).await {
        Ok(bind) => Json(DomainResult::ok(bind)),
        Err(e) => {
            log::error!("bind_domain {project_id} failed: {e}");
            Json(DomainResult::err(e))
        }
    }
}

async fn list_domains(
    Extension(domains): Extension<ProjectDomainsRef>,
    Path(project_id): Path<Uuid>,
) -> Json<DomainResult> {
    match domains.list(project_id) {
        Ok(binds) => Json(DomainResult::ok_list(binds)),
        Err(e) => {
            log::error!("list_domains {project_id} failed: {e}");
            Json(DomainResult::err(e))
        }
    }
}

async fn unbind_domain(
    Extension(domains): Extension<ProjectDomainsRef>,
    Path((_project_id, bind_id)): Path<(Uuid, Uuid)>,
) -> Json<DomainResult> {
    match domains.unbind(bind_id).await {
        Ok(()) => Json(DomainResult::deleted()),
        Err(e) => {
            log::error!("unbind_domain {bind_id} failed: {e}");
            Json(DomainResult::err(e))
        }
    }
}

async fn verify_domain(
    Extension(domains): Extension<ProjectDomainsRef>,
    Json(req): Json<VerifyDomainRequest>,
) -> Json<DomainResult> {
    match domains.verify_dns(&req.domain) {
        Ok(v) => Json(DomainResult::ok_verify(v)),
        Err(e) => {
            log::error!("verify_domain {} failed: {e}", req.domain);
            Json(DomainResult::err(e))
        }
    }
}

async fn issue_tls(
    Extension(domains): Extension<ProjectDomainsRef>,
    Path(domain): Path<String>,
) -> Json<DomainResult> {
    let bind = match domain_to_bind(&domains, &domain).await {
        Ok(b) => b,
        Err(e) => return Json(DomainResult::err(e)),
    };
    match domains.issue_tls(&bind).await {
        Ok(v) => Json(DomainResult::ok_verify(v)),
        Err(e) => {
            log::error!("issue_tls {domain} failed: {e}");
            Json(DomainResult::err(e))
        }
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