use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::metering::VMeteringRef;
use crate::projects::ProjectRegistryRef;
use crate::rbac::{ProjectRbac, ProjectRole};
use crate::vm_lifecycle::{CreateVmRequest, VmInstance, VmLifecycle, VmResult};

pub type VmLifecycleRef = Arc<VmLifecycle>;

type ApiResult = (StatusCode, Json<VmResult>);

fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe VMs API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(VmResult {
            success: false,
            vm: None,
            vms: None,
            error: Some(msg),
        }),
    )
}

fn ok(vm: VmInstance) -> ApiResult {
    (StatusCode::OK, Json(VmResult::ok(vm)))
}

fn ok_list(vms: Vec<VmInstance>) -> ApiResult {
    (StatusCode::OK, Json(VmResult::ok_list(vms)))
}

fn ok_deleted() -> ApiResult {
    (StatusCode::OK, Json(VmResult::deleted()))
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe VMs API error: {msg}");
    (StatusCode::OK, Json(VmResult::err(msg)))
}

pub fn vms_router(
    lifecycle: VmLifecycleRef,
    registry: ProjectRegistryRef,
    rbac: ProjectRbac,
    metering: VMeteringRef,
) -> Router {
    Router::new()
        .route("/api/vibe/projects/:project_id/vms", post(create_vm).get(list_vms))
        .route("/api/vibe/projects/:project_id/vms/:vm_id", get(get_vm).delete(delete_vm))
        .route("/api/vibe/vms/:vm_id/stop", post(stop_vm))
        .route("/api/vibe/vms/:vm_id/status", post(sync_vm))
        .layer(Extension(lifecycle))
        .layer(Extension(registry))
        .layer(Extension(rbac))
        .layer(Extension(metering))
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
    Extension(rbac): Extension<ProjectRbac>,
    Extension(metering): Extension<VMeteringRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateVmRequest>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    if let Err(e) = metering.enforce_for_project(project_id, crate::metering::MeterKind::VmHours) {
        return forbidden(e);
    }
    let (branch_id, name) = resolve_context(&registry, project_id);
    match lifecycle.create_project_vm(project_id, branch_id, &name, &req) {
        Ok(vm) => ok(vm),
        Err(e) => err(e),
    }
}

async fn list_vms(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match lifecycle.list(project_id) {
        Ok(vms) => ok_list(vms),
        Err(e) => err(e),
    }
}

async fn get_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((_project_id, vm_id)): Path<(Uuid, Uuid)>,
) -> ApiResult {
    let vm = match lifecycle.get(vm_id) {
        Ok(vm) => vm,
        Err(e) => return err(e),
    };
    match rbac.require_role(user.user_id, vm.project_id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    ok(vm)
}

async fn delete_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(metering): Extension<VMeteringRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((_project_id, vm_id)): Path<(Uuid, Uuid)>,
) -> ApiResult {
    let vm = match lifecycle.get(vm_id) {
        Ok(vm) => vm,
        Err(e) => return err(e),
    };
    match rbac.require_role(user.user_id, vm.project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match lifecycle.delete(vm_id) {
        Ok(()) => {
            let _ = metering.accrue_vm_hours(vm.project_id, &vm.env, vm.created_at);
            ok_deleted()
        }
        Err(e) => err(e),
    }
}

async fn stop_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(metering): Extension<VMeteringRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(vm_id): Path<Uuid>,
) -> ApiResult {
    let vm = match lifecycle.get(vm_id) {
        Ok(vm) => vm,
        Err(e) => return err(e),
    };
    match rbac.require_role(user.user_id, vm.project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match lifecycle.stop(vm_id) {
        Ok(vm) => {
            let _ = metering.accrue_vm_hours(vm.project_id, &vm.env, vm.created_at);
            ok(vm)
        }
        Err(e) => err(e),
    }
}

async fn sync_vm(
    Extension(lifecycle): Extension<VmLifecycleRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(vm_id): Path<Uuid>,
) -> ApiResult {
    let vm = match lifecycle.get(vm_id) {
        Ok(vm) => vm,
        Err(e) => return err(e),
    };
    match rbac.require_role(user.user_id, vm.project_id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match lifecycle.sync_status(vm_id) {
        Ok(vm) => ok(vm),
        Err(e) => err(e),
    }
}
