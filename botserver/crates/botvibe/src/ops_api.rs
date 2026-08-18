use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::backups::Backups;
use crate::metering::VMeteringRef;
use crate::ops::VmOps;
use crate::projects::ProjectRegistry;
use crate::rbac::{ProjectRbac, ProjectRole};
use crate::types::DbPool;

pub type VmOpsRef = Arc<VmOps>;
pub type BackupsRef = Arc<Backups>;
pub type ProjectRegistryRef = Arc<ProjectRegistry>;

#[derive(Clone)]
pub struct OpsRoutes {
    pub vm_ops: VmOpsRef,
    pub backups: BackupsRef,
    pub registry: ProjectRegistryRef,
    pub pool: DbPool,
    pub rbac: ProjectRbac,
    pub metering: VMeteringRef,
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Deserialize)]
pub struct BackupCreateRequest {
    #[serde(default = "default_env")]
    pub env: String,
}

fn default_env() -> String {
    "production".to_string()
}

pub fn ops_router(routes: OpsRoutes) -> Router {
    Router::new()
        .route("/api/vibe/projects/:project_id/envs/:env/probe", post(probe))
        .route("/api/vibe/projects/:project_id/envs/:env/restart", post(restart))
        .route("/api/vibe/projects/:project_id/preview", get(preview))
        .route("/api/vibe/projects/:project_id/deployments", get(history))
        .route("/api/vibe/projects/:project_id/deployments/:index/rollback", post(rollback))
        .route("/api/vibe/projects/:project_id/backups", get(list_backups))
        .route("/api/vibe/projects/:project_id/backups", post(create_backup))
        .route("/api/vibe/projects/:project_id/backups/:backup_id/restore", post(restore_backup))
        .layer(Extension(routes))
}

type ApiResult = (StatusCode, Json<Value>);

fn ok(data: Value) -> ApiResult {
    (StatusCode::OK, Json(json!({ "success": true, "data": data })))
}

fn err(e: String) -> ApiResult {
    log::error!("Vibe ops API error: {e}");
    (StatusCode::OK, Json(json!({ "success": false, "error": e })))
}

fn forbidden(e: String) -> ApiResult {
    log::warn!("Vibe ops API forbidden: {e}");
    (StatusCode::FORBIDDEN, Json(json!({ "success": false, "error": e })))
}

fn parse_project_id(project_id: &str) -> Result<Uuid, ApiResult> {
    Uuid::parse_str(project_id).map_err(|_| err(format!("invalid project id '{project_id}'")))
}

/// Probe a project env VM. Optional JSON body: `{ "auto_restart": bool }`.
async fn probe(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, env)): Path<(String, String)>,
    body: Option<Json<Value>>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    let auto = body
        .and_then(|b| b.get("auto_restart").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    match routes.vm_ops.probe_and_recover(pid, &env, auto).await {
        Ok(report) => ok(json!({ "probe": report })),
        Err(e) => err(e),
    }
}

/// Resolve the best browser-openable URL for the selected project. A
/// successful deployment URL is preferred; otherwise the VM probe URL is
/// returned so the UI can still diagnose the live environment.
async fn preview(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Query(query): Query<PreviewQuery>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    let deployments = match routes.registry.list_deployments(pid, Some(&query.env)) {
        Ok(rows) => rows,
        Err(e) => return err(e),
    };
    let deployed_url = deployments.iter().find_map(|row| {
        row.get("url")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
    });
    let probe = routes.vm_ops.probe_and_recover(pid, &query.env, false).await;
    match probe {
        Ok(report) => ok(json!({
            "preview_url": deployed_url.clone().or_else(|| report.url.clone()),
            "deployed_url": deployed_url,
            "probe": report,
            "env": query.env,
        })),
        Err(e) => {
            if let Some(url) = deployed_url {
                ok(json!({ "preview_url": url, "deployed_url": url, "env": query.env }))
            } else {
                err(e)
            }
        }
    }
}

/// Restart a project env VM and re-probe.
async fn restart(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, env)): Path<(String, String)>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match routes.vm_ops.probe_and_recover(pid, &env, true).await {
        Ok(report) => ok(json!({ "restarted": true, "probe": report })),
        Err(e) => err(e),
    }
}

/// Deployment history for a project (all envs).
async fn history(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match routes.registry.list_deployments(pid, None) {
        Ok(rows) => ok(json!({ "deployments": rows })),
        Err(e) => err(e),
    }
}

/// Roll back to deployment `:index` (0 = latest) — redeploys that revision
/// to the production env and records a `rollback` deployment entry.
async fn rollback(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, index)): Path<(String, String)>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    let idx: usize = match index.parse() {
        Ok(i) => i,
        Err(_) => return err(format!("invalid deployment index '{index}'")),
    };
    let rows = match routes.registry.list_deployments(pid, None) {
        Ok(rows) => rows,
        Err(e) => return err(e),
    };
    let target = match rows.get(idx) {
        Some(t) => t.clone(),
        None => return err(format!("deployment index {idx} not found (have {})", rows.len())),
    };
    let domain = target.get("domain").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match crate::publish::do_publish(
        json!({ "project_id": project_id, "env": "production", "domain": domain }),
        routes.pool.clone(),
    )
    .await
    {
        Ok(published) => ok(json!({ "rolled_back": true, "to_index": idx, "published": published })),
        Err(e) => err(e),
    }
}

async fn list_backups(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match routes.backups.list(pid) {
        Ok(rows) => ok(json!({ "backups": rows })),
        Err(e) => err(e),
    }
}

async fn create_backup(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(req): Json<BackupCreateRequest>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match routes.backups.create_snapshot(pid, &req.env) {
        Ok(rec) => {
            let _ = routes.metering.add_for_project(pid, &rec.env, crate::metering::MeterKind::SnapshotCount, 1.0);
            ok(json!({ "backup": rec }))
        }
        Err(e) => err(e),
    }
}

async fn restore_backup(
    Extension(routes): Extension<OpsRoutes>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, backup_id)): Path<(String, String)>,
) -> ApiResult {
    let pid = match parse_project_id(&project_id) {
        Ok(id) => id,
        Err(r) => return r,
    };
    match routes.rbac.require_role(user.user_id, pid, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    let bid = match Uuid::parse_str(&backup_id) {
        Ok(id) => id,
        Err(_) => return err(format!("invalid backup id '{backup_id}'")),
    };
    let record = match routes.backups.get(bid) {
        Ok(rec) => rec,
        Err(e) => return err(e),
    };
    if record.project_id != pid {
        return err(format!("backup {backup_id} does not belong to project {project_id}"));
    }
    match routes.backups.restore(bid, &routes.vm_ops).await {
        Ok((rec, probe)) => ok(json!({ "backup": rec, "probe_after_restore": probe })),
        Err(e) => err(e),
    }
}