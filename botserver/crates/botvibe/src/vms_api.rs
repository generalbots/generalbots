//! #744 — REST surface for the per-project VM lifecycle. Thin handlers over
//! `VmLifecycle`: create/ensure a VM for a project env, stop, list, status
//! (sync with the live container), delete. Project name + branch come from
//! the project registry; env tier + runner are requested in the body.

use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::projects::ProjectRegistryRef;
use crate::vm_lifecycle::{CreateVmRequest, VmLifecycle, VmResult};

pub type VmLifecycleRef = Arc<VmLifecycle>;

pub fn vms_router(lifecycle: VmLifecycleRef, registry: ProjectRegistryRef) -> Router {
    Router::new()
        .route("/api/vibe/projects/:project_id/vms", post(create_vm).get(list_vms))
        .route("/api/vibe/projects/:project_id/vms/:vm_id", get(get_vm).delete(delete_vm))
        .route("/api/vibe/vms/:vm_id/stop", post(stop_vm))
        .route("/api/vibe/vms/:vm_id/status", post(sync_vm))
        .layer(Extension(lifecycle))
        .layer(Extension(registry))
}

fn project_name(registry: &ProjectRegistryRef, project_id: Uuid) -> Result<(Uuid, String), String> {
    let project = registry
        .get(project_id)?
        .ok_or_else(|| format!("project {project_id} not found"))?;
    Ok((project.branch_id, project.name))
}

fn resolve_context(
    registry: &ProjectRegistryRef,
    project_id: Uuid,
) -> (Uuid, String) {
    match project_name(registry, project_id) {
        Ok((branch_id, name)) => (branch_id, name),
        Err(e) => {
            log::error!("resolve project context {project_id} for vm: {e}");
            (Uuid::nil(), project_id.to_string())
        }
    }
}

async fn create_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Extension(registry): Extension<ProjectRegistryRef>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateVmRequest>,
) -> Json<VmResult> {
    let (branch_id, name) = resolve_context(&registry, project_id);
    match lifecycle.create_project_vm(project_id, branch_id, &name, &req) {
        Ok(vm) => Json(VmResult::ok(vm)),
        Err(e) => {
            log::error!("create_vm failed for {project_id}: {e}");
            Json(VmResult::err(e))
        }
    }
}

async fn list_vms(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Path(project_id): Path<Uuid>,
) -> Json<VmResult> {
    match lifecycle.list(project_id) {
        Ok(vms) => Json(VmResult::ok_list(vms)),
        Err(e) => {
            log::error!("list_vms failed for {project_id}: {e}");
            Json(VmResult::err(e))
        }
    }
}

async fn get_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Path((_project_id, vm_id)): Path<(Uuid, Uuid)>,
) -> Json<VmResult> {
    match lifecycle.get(vm_id) {
        Ok(vm) => Json(VmResult::ok(vm)),
        Err(e) => {
            log::error!("get_vm {vm_id}: {e}");
            Json(VmResult::err(e))
        }
    }
}

async fn delete_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Path((_project_id, vm_id)): Path<(Uuid, Uuid)>,
) -> Json<VmResult> {
    match lifecycle.delete(vm_id) {
        Ok(()) => Json(VmResult::deleted()),
        Err(e) => {
            log::error!("delete_vm {vm_id}: {e}");
            Json(VmResult::err(e))
        }
    }
}

async fn stop_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Path(vm_id): Path<Uuid>,
) -> Json<VmResult> {
    match lifecycle.stop(vm_id) {
        Ok(vm) => Json(VmResult::ok(vm)),
        Err(e) => {
            log::error!("stop_vm {vm_id}: {e}");
            Json(VmResult::err(e))
        }
    }
}

async fn sync_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Path(vm_id): Path<Uuid>,
) -> Json<VmResult> {
    match lifecycle.sync_status(vm_id) {
        Ok(vm) => Json(VmResult::ok(vm)),
        Err(e) => {
            log::error!("sync_vm {vm_id}: {e}");
            Json(VmResult::err(e))
        }
    }
}